#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""check_oracle_boundary.py — S8 定位边界门禁：shell 层禁 oracle_* 函数调用。

背景（docs/plan-s8-refactor.md §1 + S8.8）：
- `crates/ti-core/src/oracle/*` 降级为**参照物**（类型/几何/格式），冻结不再新增；
- 业务数值/公式已全部迁 `assets/data/*.json`（经 `ti_core::data::balance()/strings()/controls()`）；
- 玩法逻辑在 `crates/ti-core/src/rules/*`。
- **表现层 / 壳层**（`ti-app/src/game_app.rs`、`bins/*`、`scenes/*`、`widgets.rs`、`i18n.rs`）
  只允许调 `ti_core::rules::*`，**不得直调任何 `oracle_*` 函数**（oracle 的公式/数值是「推测」，
  S8 起不再作业务依据）。oracle 的**类型/枚举重导出**（如 `oracle::main_map::Dir`、
  `oracle::battle::Side`）允许保留（它们是稳定标识，几何/像素布局属事实层）。

本脚本判定：
- shell 层文件里出现 `oracle_` 前缀**函数调用**（`oracle_xxx(` 或 `.oracle_xxx` 或
  全限定 `oracle::xxx::oracle_yyy(`）→ 违规（FAIL）；
- 仅 `oracle::xxx::Type`（类型/枚举/结构体路径，后不跟 `(`）→ 允许（PASS，属类型重导出）。

三契约（EXP-054）：幂等（只读、连跑两次输出一致）+ `--check`（默认，退出码判 PASS/FAIL）
+ `--report`（打印命中明细，退出码 0 恒真，供人工看）。被 gate.ps1 第 12 项消费。

用法：
  python scripts/check_oracle_boundary.py --check     # gate 用：违规即非 0
  python scripts/check_oracle_boundary.py --report     # 人工看：列明每处命中 + 0 退出

豁免（EXEMPT）：oracle 本体（`ti-core/src/oracle/`）+ `rules/` 内部（允许调 oracle 几何/类型）
+ `tests/`（集成测试可做 oracle diff 参照）。仅 shell 层受约束。
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

# shell 层（受约束）：ti-app 的表现层文件，不含 oracle 本体 / rules 内部 / tests
SHELL_GLOBS = [
    "crates/ti-app/src/game_app.rs",
    "crates/ti-app/src/widgets.rs",
    "crates/ti-app/src/i18n.rs",
]
SHELL_DIRS = [
    "crates/ti-app/src/scenes",
    "crates/ti-app/src/bin",
]

# oracle_* 函数调用模式：
#   - `oracle_xxx(` 直接调用
#   - `.oracle_xxx` 方法式（本仓 oracle 多为自由函数，少见，保留兜底）
#   - 全限定 `oracle::<module>::oracle_xxx(`
CALL_PAT = re.compile(r"(\.\s*)?oracle_[a-z_0-9]+\s*\(")
# 但 `use ti_core::oracle::` 的 import 行 + 注释 `//!`/`//` + doc `///` 不算调用
IMPORT_OR_COMMENT = re.compile(r"^\s*(//|///|//!|use\b|pub\s+use\b)")


def shell_files() -> list[Path]:
    files: list[Path] = []
    for g in SHELL_GLOBS:
        p = REPO / g
        if p.exists():
            files.append(p)
    for d in SHELL_DIRS:
        base = REPO / d
        if base.exists():
            files.extend(sorted(base.rglob("*.rs")))
    return files


def scan() -> list[str]:
    """返回违规行（'file:line: text'）。oracle_ 函数调用且非 import/注释。"""
    hits: list[str] = []
    for f in shell_files():
        text = f.read_text(encoding="utf-8")
        rel = f.relative_to(REPO).as_posix()
        for i, line in enumerate(text.splitlines(), 1):
            if IMPORT_OR_COMMENT.match(line):
                continue
            if CALL_PAT.search(line):
                hits.append(f"{rel}:{i}  {line.strip()}")
    return hits


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    grp = ap.add_mutually_exclusive_group()
    grp.add_argument("--check", action="store_true", help="门禁模式：有违规即非 0")
    grp.add_argument("--report", action="store_true", help="报告模式：列命中，恒 0")
    a = ap.parse_args()
    report = a.report or not (a.check)

    hits = scan()
    if report:
        for h in hits:
            print(h)
        print(f"check_oracle_boundary: {len(hits)} violation(s) in shell layer")
        return 0

    # --check：门禁
    if not hits:
        print("check_oracle_boundary: OK (0 oracle_* function calls in shell layer)")
        return 0
    for h in hits:
        print(h, file=sys.stderr)
    print(f"check_oracle_boundary: FAIL ({len(hits)} oracle_* function call(s) in shell layer)", file=sys.stderr)
    print("        shell 层须改调 ti_core::rules::*；oracle 仅作类型/几何参照物（rules 内部允许）", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
