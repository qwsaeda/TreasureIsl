#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""check_forward_closed.py — S11 业务闭环门禁（gate 第 15 项）。

背景（`docs/plan-s11-forward.md` §1 闭环判据 1~4 + §5）：
逆向产物物理归档到 `legacy/` 后，正向开发（business + impl + process）必须**自洽闭环**：
业务文档不再引用逆向产物、表现层不再引用参照实现、根目录不再散落原版文件。

本脚本判定 4 条（全部机器可判，只读、幂等）：
  C1 业务文档零逆向引用：`docs/business/**/*.md` 不出现 `oracle` / `facts` /
     `spec/` / `ref/` / `推测`；**唯一豁免** = 同行带 `legacy/` 出处标注
     （实测值的出处引用，例如 `legacy/docs/facts/map.md` F-307）。
  C2 表现层零参照实现引用：`crates/ti-app/**` 全文 0 个 `oracle`（含测试）。
  C3 根目录已清空原版文件：不存在 `TreasureIsl.exe` / `TreasureIsl.exe.i64` /
     `data/` / `ref/`；且 `legacy/` 下 `TreasureIsl.exe` / `data/` / `ref/` /
     `oracle-rust/` 均在位。
  C4 桥接层与归档同源：`crates/ti-core/src/oracle/<m>.rs` 与
     `legacy/oracle-rust/src/<m>.rs` 文件集合一致（防双份漂移，EXEMPT：mod.rs）。

**未收口项（不进本门禁，另立台账任务跟踪）**：
  判据 3「默认构建 0 行参照代码参与」尚未达成——`ti-core/src/rules/*` 仍调用
  `oracle_*` 函数（S12 起逐模块摘除，见 `plan/ledger.yaml`）。此处刻意不粉饰。

用法：
  python scripts/check_forward_closed.py --check    # gate 用：任一不通过即非 0
  python scripts/check_forward_closed.py --report   # 人工看：列命中明细，恒 0
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

# C1：逆向语境关键词（`facts` 单独出现也命中，`assets/data` 等不算）
C1_PAT = re.compile(r"oracle|facts|spec/|ref/|推测")
C1_EXEMPT = "legacy/"  # 同行带 legacy/ 前缀 = 出处标注，豁免

# C3：根目录不得再有
ROOT_FORBIDDEN = ["TreasureIsl.exe", "TreasureIsl.exe.i64", "data", "ref"]
# C4 豁免：两份 crate 根文件职责不同（桥接层=ti-core 子模块，legacy=独立 crate 根）
EXEMPT_C4 = {"mod.rs", "lib.rs"}
LEGACY_REQUIRED = [
    "legacy/TreasureIsl.exe",
    "legacy/data",
    "legacy/ref",
    "legacy/oracle-rust",
    "legacy/docs",
    "legacy/scripts",
]


def check_c1() -> list[str]:
    """业务文档零逆向引用（legacy 出处标注豁免）。"""
    hits: list[str] = []
    for md in sorted((REPO / "docs" / "business").rglob("*.md")):
        rel = md.relative_to(REPO).as_posix()
        for i, line in enumerate(md.read_text(encoding="utf-8").splitlines(), 1):
            if not C1_PAT.search(line):
                continue
            if C1_EXEMPT in line:
                continue
            hits.append(f"C1 {rel}:{i}  {line.strip()[:140]}")
    return hits


def check_c2() -> list[str]:
    """表现层（ti-app，含 tests）0 个 `oracle`。"""
    hits: list[str] = []
    for rs in sorted((REPO / "crates" / "ti-app").rglob("*.rs")):
        rel = rs.relative_to(REPO).as_posix()
        for i, line in enumerate(rs.read_text(encoding="utf-8").splitlines(), 1):
            if "oracle" in line:
                hits.append(f"C2 {rel}:{i}  {line.strip()[:140]}")
    return hits


def check_c3() -> list[str]:
    """根目录已清空原版文件 + legacy/ 归档在位。"""
    hits: list[str] = []
    for name in ROOT_FORBIDDEN:
        if (REPO / name).exists():
            hits.append(f"C3 根目录仍存在 `{name}`（应只在 legacy/ 下）")
    for rel in LEGACY_REQUIRED:
        if not (REPO / rel).exists():
            hits.append(f"C3 legacy 归档缺失 `{rel}`")
    return hits


def check_c4() -> list[str]:
    """桥接层与归档同源（文件集合一致，mod.rs 除外）。"""
    bridge = REPO / "crates" / "ti-core" / "src" / "oracle"
    legacy = REPO / "legacy" / "oracle-rust" / "src"
    hits: list[str] = []
    if not legacy.is_dir():
        return [f"C4 legacy oracle 归档缺失：{legacy.relative_to(REPO).as_posix()}"]
    b = {p.name for p in bridge.glob("*.rs") if p.name not in EXEMPT_C4}
    l = {p.name for p in legacy.glob("*.rs") if p.name not in EXEMPT_C4}
    for name in sorted(b - l):
        hits.append(f"C4 桥接层有 `{name}` 但 legacy/oracle-rust/src 无同源文件（双份漂移）")
    for name in sorted(l - b):
        hits.append(f"C4 legacy/oracle-rust/src 有 `{name}` 但桥接层缺（归档与镜像不同源）")
    return hits


CHECKS = {
    "C1 业务文档零逆向引用": check_c1,
    "C2 表现层零 oracle 引用": check_c2,
    "C3 根目录清空 + legacy 在位": check_c3,
    "C4 oracle 桥接层与归档同源": check_c4,
}


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    grp = ap.add_mutually_exclusive_group()
    grp.add_argument("--check", action="store_true", help="门禁模式：任一不通过即非 0")
    grp.add_argument("--report", action="store_true", help="报告模式：列命中，恒 0")
    ap.add_argument("--only", choices=sorted(CHECKS), help="只跑一条检查")
    a = ap.parse_args()

    names = [a.only] if a.only else list(CHECKS)
    total = 0
    for name in names:
        hits = CHECKS[name]()
        total += len(hits)
        status = "OK" if not hits else f"FAIL {len(hits)}"
        print(f"check_forward_closed: {name}: {status}")
        for h in hits:
            print(f"    {h}", file=sys.stderr if a.check and not a.report else sys.stdout)

    print(f"check_forward_closed: {total} violation(s) across {len(names)} check(s)")
    if a.report:
        return 0
    if total:
        print(
            "        业务文档零逆向引用（legacy/ 出处标注除外）；表现层只许用 ti_core::domain/rules；"
            "根目录不得再有原版文件；桥接层与 legacy/oracle-rust 同源",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())