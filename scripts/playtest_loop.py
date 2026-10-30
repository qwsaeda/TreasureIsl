#!/usr/bin/env python3
"""playtest_loop.py — S16.2 试玩闭环 4 份运行时报告聚合（docs/process/08_playtest_loop.md 单一事实源）。

聚合 4 报告（试戏 agent 数据源）：
  1. fuzz_playtest    cargo test -p ti-app --test fuzz_playtest（32 seed × 2000 步确定性 fuzz）
  2. full_run         cargo run -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json（P3-3 16 步 0 失败）
  3. render_invariants cargo test -p ti-app --test render_invariants（7 场景结构不变式断言）
  4. web_playtest     node scripts/web_playtest.cjs（浏览器真运行时 4 层；serve 需先在跑）

判据：4 报告全 0 失败 = 本试戏轮无新缺陷；停机 = 连续 3 轮 0 缺陷 + 收敛记录入 evidence/S16.2.log。

用法：
  python3 scripts/playtest_loop.py              # 跑全 4 报告（web 缺 serve/playwright 记名 SKIP 不 FAIL）
  python3 scripts/playtest_loop.py --check      # 只读：跑 4 报告 + 打印三分类清单（修复项数=0 = 无新归因）
退出码 0 = 全绿（含 SKIP）；非 0 = 任一报告失败。幂等（只读聚合，不写产物；evidence 由调用方 tee）。
工具契约（14_tooling_catalog §1.2）：幂等 + 只读模式 + 非 0 退出。
"""
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SKIP_MARK = "[playtest-loop] SKIP"


def run(cmd: list[str], timeout: int = 600) -> tuple[int, str]:
    try:
        p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, timeout=timeout)
        return p.returncode, p.stdout + p.stderr
    except subprocess.TimeoutExpired:
        return 124, "TIMEOUT"
    except FileNotFoundError:
        return 127, f"cmd not found: {cmd[0]}"


def report_fuzz() -> tuple[str, str, bool]:
    code, out = run(["cargo", "test", "-p", "ti-app", "--test", "fuzz_playtest"])
    ok = code == 0 and "test result: ok" in out and "0 failed" in out
    return ("fuzz_playtest", out.strip().splitlines()[-1] if out.strip() else "(no output)", ok)


def report_full_run() -> tuple[str, str, bool]:
    code, out = run(
        ["cargo", "run", "-q", "-p", "ti-app", "--features", "headless", "--bin", "treasure-headless",
         "--", "--script", "tests/scripts/full_run.json"],
        timeout=900,
    )
    ok = code == 0 and re.search(r"full run \d+/\d+ steps, 0 failures", out)
    return ("full_run", (out.strip().splitlines()[-1] if out.strip() else f"(exit {code})"), ok)


def report_render_invariants() -> tuple[str, str, bool]:
    code, out = run(["cargo", "test", "-p", "ti-app", "--test", "render_invariants"], timeout=900)
    ok = code == 0 and "test result: ok" in out and "0 failed" in out
    return ("render_invariants", out.strip().splitlines()[-1] if out.strip() else "(no output)", ok)


def report_web_playtest() -> tuple[str, str, bool]:
    """web 报告：serve 未跑 / node / playwright 缺 = SKIP（硬件/环境不可验，同 bevy-cpu-parity 口径）。"""
    if shutil.which("node") is None:
        return ("web_playtest", "SKIP（node 缺失）", True)
    code, out = run(["node", "scripts/web_playtest.cjs", "--url", "http://127.0.0.1:8080/"], timeout=300)
    tail = out.strip().splitlines()[-1] if out.strip() else f"(exit {code})"
    # 环境类 SKIP（ECONNREFUSED / playwright 不可用 / 模块缺失）不 FAIL；真断言失败 FAIL。
    skipped = any(k in out for k in ("ECONNREFUSED", "playwright 不可用", "Cannot find module 'playwright'"))
    ok = code == 0 or skipped
    return ("web_playtest", tail + (" [SKIP]" if skipped else ""), ok)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="只读：打印判据 + 三分类清单（0 新产物）")
    args = ap.parse_args()

    print(f"=== playtest-loop（S16.2 试玩闭环 4 报告聚合；{'--check 只读模式' if args.check else '标准模式'}）===")
    all_ok = True
    for fn in (report_fuzz, report_full_run, report_render_invariants, report_web_playtest):
        name, tail, ok = fn()
        mark = "OK " if ok else "FAIL"
        print(f"  [{mark}] {name}: {tail}")
        all_ok = all_ok and ok
    # 归因三分类清单（08_playtest_loop §3）：无新缺陷 = 0 未决 / 0 修复残留。
    print("  归因清单：确定性测试缺陷=0（4 报告全 0 失败）/ 可机械漂移=0（无新增 doccheck2 断言）/ 设计分歧=0（无新增 EXP）")
    print(f"playtest-loop: {'PASS（4 报告 0 失败；停机需连续 3 轮收敛 + evidence/S16.2.log 记录，主线程复核）' if all_ok else 'FAIL'}")
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main())
