# opencode.db 瘦身：删归档会话（time_archived）及其递归子孙会话 + 关联 event/message/part
# 用法：
#   python scripts/cleanup_opencode_db.py                  # 演练（只打印，不写）
#   python scripts/cleanup_opencode_db.py --yes            # 实际执行（备份+删+VACUUM）
#   python scripts/cleanup_opencode_db.py --yes --no-children  # 只删顶层归档会话，不递归子孙
#
# 退出码 0 = 成功（演练或实跑），非 0 = 失败
#
# 背景与坑（2026-10 实测，另见 macOS 侧 cleanup-mac-disk.sh 经验）：
#   - DB 默认在 %USERPROFILE%\.local\share\opencode\opencode.db
#   - 92% 体积在 event 表（event sourcing 全量快照，每条重复携带外层信封）
#   - event → event_sequence 有 ON DELETE CASCADE，但 event_sequence → session 没有外键，
#     所以必须手动先删 event_sequence，再删 session（FK=ON 时 message/part 自动 CASCADE 带走）
#   - 85% 数据在归档顶层会话的**子代理会话**里，只删顶层等于没删，务必递归
#   - VACUUM 需要独占写锁：opencode 进程（含 web/serve）持有 db 时先退出，本脚本检测到占用即报错，不自动杀进程

import argparse
import os
import shutil
import sqlite3
import sys
import time

DB_PATH = os.path.join(os.environ.get("USERPROFILE", os.path.expanduser("~")),
                       ".local", "share", "opencode", "opencode.db")
BACKUP_DIR = r"C:\金银岛\backup"

RECURSIVE_CTE = """
WITH RECURSIVE d(id) AS (
    SELECT id FROM session WHERE time_archived IS NOT NULL
    UNION ALL
    SELECT s.id FROM session s JOIN d ON s.parent_id = d.id
)
SELECT id FROM d
"""

TOP_CTE = """
WITH RECURSIVE d(id) AS (
    SELECT id FROM session WHERE time_archived IS NOT NULL
    UNION ALL
    SELECT s.id FROM session s JOIN d ON s.parent_id = d.id
)
SELECT id FROM d
"""


def open_db():
    if not os.path.exists(DB_PATH):
        print(f"DB 不存在: {DB_PATH}")
        sys.exit(1)
    try:
        con = sqlite3.connect(f"file:{DB_PATH}?mode=ro", uri=True, timeout=5)
        con.execute("SELECT 1")
    except sqlite3.OperationalError as e:
        print(f"无法打开 DB（可能有 opencode 进程占用，请先全部退出）: {e}")
        sys.exit(1)
    return con


def file_size_gb(p):
    if not os.path.exists(p):
        return 0.0
    return os.path.getsize(p) / 1024 / 1024 / 1024


def counts(con):
    tables = ["session", "message", "part", "event", "event_sequence"]
    out = {}
    for t in tables:
        out[t] = con.execute(f"SELECT COUNT(*) FROM {t}").fetchone()[0]
    out["top_archived"] = con.execute(
        "SELECT COUNT(*) FROM session WHERE time_archived IS NOT NULL").fetchone()[0]
    out["recursive_target"] = con.execute(
        f"SELECT COUNT(*) FROM ({RECURSIVE_CTE})").fetchone()[0]
    out["active_kept"] = out["session"] - out["recursive_target"]
    out["events_in_target"] = con.execute(
        f"SELECT COUNT(*) FROM event WHERE aggregate_id IN ({RECURSIVE_CTE})").fetchone()[0]
    return out


def main():
    ap = argparse.ArgumentParser(description="opencode.db 瘦身（删归档会话+递归子孙）")
    ap.add_argument("--yes", action="store_true", help="实际执行（默认演练）")
    ap.add_argument("--no-children", action="store_true",
                    help="只删顶层归档会话，不递归子孙（不推荐：85%% 数据在子会话）")
    args = ap.parse_args()

    dry = not args.yes
    mode = "演练" if dry else "实际执行"
    children = not args.no_children
    print(f"=== opencode.db 瘦身（{mode}，递归子孙={children}）===")
    print(f"DB: {DB_PATH}")
    print(f"当前大小: {file_size_gb(DB_PATH):.2f} GB")

    con = open_db()
    c = counts(con)
    for k in ("session", "top_archived", "recursive_target", "active_kept",
              "message", "part", "event", "events_in_target", "event_sequence"):
        print(f"  {k}: {c[k]}")

    if c["recursive_target"] == 0:
        print("无归档会话可删，退出。")
        con.close()
        return

    target_cte = RECURSIVE_CTE if children else TOP_CTE
    sql_event_sequence = (
        f"DELETE FROM event_sequence WHERE aggregate_id IN ({target_cte})")
    sql_session = (
        f"DELETE FROM session WHERE id IN ({target_cte})")

    print("\n将执行的 SQL（FK=ON，message/part 由 CASCADE 自动带走）:")
    print(f"  PRAGMA foreign_keys=ON;")
    print(f"  {sql_event_sequence};")
    print(f"  {sql_session};")
    print(f"  VACUUM;")
    print(f"\n预计: session {c['session']} -> {c['active_kept']}，"
          f"event 删 {c['events_in_target']} 条")

    if dry:
        print("\n演练结束，未写库。加 --yes 实际执行。")
        con.close()
        return

    # --- 实际执行 ---
    ts = time.strftime("%Y%m%d-%H%M%S")
    os.makedirs(BACKUP_DIR, exist_ok=True)
    bak = os.path.join(BACKUP_DIR, f"opencode.db.bak-{ts}")
    print(f"\n备份到 {bak}（{file_size_gb(DB_PATH):.1f} GB，请耐心等待）...")
    shutil.copy2(DB_PATH, bak)
    print(f"备份完成: {file_size_gb(bak):.1f} GB")

    # 切换到可写连接
    con.close()
    try:
        con = sqlite3.connect(DB_PATH, timeout=10)
        con.execute("PRAGMA foreign_keys=ON")
        t0 = time.time()
        n1 = con.execute(sql_event_sequence).rowcount
        n2 = con.execute(sql_session).rowcount
        con.commit()
        print(f"DELETE event_sequence: {n1} 行, DELETE session: {n2} 行"
              f"（耗时 {time.time() - t0:.1f}s）")
        print("VACUUM 中（大库耗时数分钟）...")
        t1 = time.time()
        con.execute("VACUUM")
        print(f"VACUUM 完成（{time.time() - t1:.1f}s）")
    except sqlite3.OperationalError as e:
        print(f"\n执行失败（可能 DB 被 opencode 进程占用）: {e}")
        print(f"备份已保留: {bak}")
        con.close()
        sys.exit(1)

    con2 = open_db()
    c2 = counts(con2)
    con2.close()
    print(f"\n结果: session {c2['session']}（预期 {c['active_kept']}），"
          f"event {c2['event']}，文件 {file_size_gb(DB_PATH):.2f} GB"
          f"（清理前 {file_size_gb(bak):.2f} GB）")
    if c2["session"] != c["active_kept"]:
        print("警告: session 数与预期不符，请核查。")
    print("\n清理完成。")


if __name__ == "__main__":
    main()
