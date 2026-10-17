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
      `docs/` / `scripts/` 均在位。
   C4 oracle 单点源 + 禁重复 crate：唯一镜像 = `crates/ti-core/src/oracle/`；
      `legacy/oracle-rust/` 不得被重新引入（S15.8 已删除该逐字节重复 crate）。

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

# C3 豁免：根目录不得再有
ROOT_FORBIDDEN = ["TreasureIsl.exe", "TreasureIsl.exe.i64", "data", "ref"]
# C3：legacy/ 归档必备目录
LEGACY_REQUIRED = [
    "legacy/TreasureIsl.exe",
    "legacy/data",
    "legacy/ref",
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
    """oracle 单点源：唯一镜像 = `ti-core/src/oracle/`，重复 crate 不得重引入。

    S11 原本此处校验「桥接层与 `legacy/oracle-rust/` 文件集合同源」（防双份漂移）。
    S15.8 查明 `legacy/oracle-rust/` 与桥接层逐字节同源且**全库零代码引用**
    （集成测试一律 `use ti_core::oracle::`），遂删除该重复 crate 并摘除 workspace
    member 登记。故本检查改为**反向守卫**：重复 crate 一旦被重新引入即 FAIL，
    防止两份镜像再次漂移（S11 同源判据的原始意图由「单点源」更强地满足）。
    """
    bridge = REPO / "crates" / "ti-core" / "src" / "oracle"
    duplicate = REPO / "legacy" / "oracle-rust"
    hits: list[str] = []
    if duplicate.exists():
        hits.append(
            "C4 重复 crate `legacy/oracle-rust/` 被重引入（oracle 唯一源 = "
            "`crates/ti-core/src/oracle/`，见 crates/ti-core/src/oracle/mod.rs 头注）"
        )
    if not bridge.is_dir():
        hits.append("C4 oracle 唯一源缺失：crates/ti-core/src/oracle/")
    return hits


CHECKS = {
    "C1 业务文档零逆向引用": check_c1,
    "C2 表现层零 oracle 引用": check_c2,
    "C3 根目录清空 + legacy 在位": check_c3,
    "C4 oracle 单点源 + 禁重复 crate": check_c4,
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
            "根目录不得再有原版文件；oracle 单点源在 ti-core 且 legacy/oracle-rust 不得重引入",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())