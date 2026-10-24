#!/usr/bin/env python3
"""数据源门禁（契约 §5，docs/process/07_render_pipeline.md）。

判定：`legacy/data/*.Map` / `*.pak` 相对 HEAD 有 git diff 时，渲染快照（gate 第 19 项
`render_snapshot`）必须全绿 —— 数据源漂移会当场被快照拦截。

用法：
  python scripts/check_data_source.py            # 有数据源改动 → 跑 render_snapshot，要求全绿
  python scripts/check_data_source.py --check    # 只读门禁（契约别名；默认行为即只读，不写文件）
  python scripts/check_data_source.py --skip     # 数据源无改动时 SKIP（gate 全绿前提）

退出码：0 = PASS/SKIP，1 = FAIL（数据源改动且快照漂移）。
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA_GLOBS = ("legacy/data/*.Map", "legacy/data/*.pak")


def data_source_changed() -> bool:
    """检查 legacy/data/ 下 .Map/.pak 相对 HEAD 是否有改动。"""
    for pattern in DATA_GLOBS:
        try:
            diff = subprocess.run(
                ["git", "diff", "--name-only", "HEAD", "--", pattern],
                capture_output=True,
                text=True,
                cwd=ROOT,
                check=False,
            )
            if diff.stdout.strip():
                return True
        except FileNotFoundError:
            # git 不存在（非 git 环境）→ 视为无改动，放行
            return False
    return False


def run_render_snapshot() -> bool:
    """跑 render_snapshot，全绿返回 True。"""
    proc = subprocess.run(
        ["cargo", "test", "-q", "-p", "ti-app", "--test", "render_snapshot"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )
    print(proc.stdout[-4000:] if proc.stdout else "", end="")
    print(proc.stderr[-4000:] if proc.stderr else "", end="")
    return proc.returncode == 0


def main() -> int:
    # S21.8 工具契约：--check = 只读门禁别名（默认行为即只读判定，不写文件；漂移非 0 退出）
    if "--check" in sys.argv:
        print("NOTE: --check = 只读门禁模式（本工具默认即只读；数据源改动且快照漂移时非 0 退出）")
    skip = "--skip" in sys.argv
    if not data_source_changed():
        print("SKIP: legacy/data/ 无 .Map/.pak 改动，渲染快照门禁无需触发")
        return 0
    if skip:
        print("SKIP: 数据源有改动但 --skip 指定，跳过 render_snapshot（仅限本地调试）")
        return 0
    print("DATA-CHANGED: legacy/data/ 有 .Map/.pak 改动，跑 render_snapshot 验证快照")
    ok = run_render_snapshot()
    if ok:
        print("PASS: 数据源改动后 render_snapshot 全绿（快照未漂移）")
        return 0
    print(
        "FAIL: 数据源改动后 render_snapshot 漂移 —— 须同步更新基准 "
        "（重跑 cargo test -p ti-app --test render_snapshot 首跑生成），"
        "或回退数据源改动",
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
