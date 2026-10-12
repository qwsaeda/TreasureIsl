#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""check_module_contracts.py — 模块并行门禁（S11.4，gate 第 16 项）。

校验「模块并行开发」的机读前置条件（docs/process/05_module_parallel.md §4）：
  M1 白名单互斥：plan/packages.yaml 任意两包 files[] 交集 = ∅；
  M2 契约锚点存在：docs/contracts/*.md `## 校验锚点` 里每个符号在 crates/ 下 grep 到；
  M3 跨 crate 可见面只许走契约：crates/ti-app/** 的 `use ti_core::` 路径必须落在
     c1 契约锚点声明的模块前缀（domain/rules/rng/data）之内（防包私有类型外泄）；
  M4 无 draft 残留：并行轮启动前 packages.yaml 各包与 contracts 均须 frozen
     （当前 S11 阶段允许 draft，只 WARN；S12 契约冻结轮后转 FAIL 判据）。

输出：M1-M3 违规 + M4 WARN 全部走 stdout（gate.ps1 逐行渲染）；stdout 恒为 UTF-8 安全文本
（Windows 默认 console codepage 会把中文变乱码，故文案里只用包 ID / 契约文件名，
违规明细里可能带出的中文源码上下文由调用方负责 utf-8 落盘，见 EXP-038）。

幂等 + `--check` 只读（M1-M3 非 0 退出码 = gate 判 FAIL，M4 只 WARN）；
`--report` 打印明细恒 0。
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
PACKAGES = REPO / "plan" / "packages.yaml"
CONTRACTS = REPO / "docs" / "contracts"

# 极简 YAML 解析（只取 packages.yaml 的 files/tests/status 行，保持零第三方依赖）
def parse_packages(text: str) -> list[dict]:
    """极简解析（零第三方依赖）：取 id/name/status/depends_on/files/tests。"""
    pkgs: list[dict] = []
    cur: dict | None = None
    list_field = ""
    for raw in text.splitlines():
        line = raw.strip()
        m = re.match(r"- id:\s*(\S+)", line)
        if m:
            cur = {"id": m.group(1), "files": [], "tests": [], "status": "draft", "depends_on": []}
            pkgs.append(cur)
            list_field = ""
            continue
        if cur is None:
            continue
        m = re.match(r"status:\s*(\S+)", line)
        if m:
            cur["status"] = m.group(1)
            continue
        m = re.match(r"depends_on:\s*\[([^\]]*)\]", line)
        if m:
            cur["depends_on"] = [x.strip() for x in m.group(1).split(",") if x.strip()]
            continue
        m = re.match(r"(files|tests):\s*$", line)
        if m:
            list_field = m.group(1)
            continue
        m = re.match(r"-\s*(\S+)\s*$", line)
        if m and list_field:
            indent_ok = raw.startswith("      -")
            if indent_ok:
                cur[list_field].append(m.group(1).strip('"'))
    return pkgs


def read_files(paths: list[Path]) -> list[str]:
    out = []
    for p in paths:
        if p.is_file():
            out.append(p.read_text(encoding="utf-8"))
    return out


def m1_disjoint(pkgs: list[dict]) -> list[str]:
    errs = []
    for i in range(len(pkgs)):
        for j in range(i + 1, len(pkgs)):
            a, b = pkgs[i], pkgs[j]
            shared: set[str] = set()
            for fa in a["files"]:
                for fb in b["files"]:
                    pref_a = fa if fa.endswith("/") else fa + "/"
                    pref_b = fb if fb.endswith("/") else fb + "/"
                    if fa == fb or fa.startswith(pref_b) or fb.startswith(pref_a):
                        shared.add(fa if fa.endswith("/") else fb)
            if shared:
                errs.append(f"M1 {a['id']} × {b['id']} 白名单重叠：{sorted(shared)}")
    return errs


def m2_anchor_exists() -> list[str]:
    """契约 `## 校验锚点` 里每个符号须在 crates/ 下 grep 到；数据表锚点须文件存在。"""
    errs = []
    blob = "\n".join(read_files(list((REPO / "crates").rglob("*.rs"))))
    for md in sorted(CONTRACTS.glob("*.md")):
        text = md.read_text(encoding="utf-8")
        if "## 校验锚点" not in text:
            continue
        in_block = False
        anchors: list[str] = []
        for line in text.splitlines():
            s = line.strip()
            if s.startswith("```"):
                in_block = not in_block
                continue
            if in_block and s:
                anchors.append(s)
        for a in anchors:
            if a.startswith("assets/") or a.startswith("legacy/"):
                if not (REPO / a).exists():
                    errs.append(f"M2 {md.name}: 数据表锚点不存在 `{a}`")
                continue
            name = a.split("::")[-1]
            if not re.search(rf"\b{re.escape(name)}\b", blob):
                errs.append(f"M2 {md.name}: 契约符号未 grep 到 `{a}`")
    return errs


def m3_app_imports_only_contract() -> list[str]:
    errs = []
    for f in (REPO / "crates" / "ti-app").rglob("*.rs"):
        text = f.read_text(encoding="utf-8")
        for m in re.finditer(r"^\s*use\s+(ti_core::[a-zA-Z0-9_:]+)", text, re.M):
            path = m.group(1)
            top2 = path.split("::")
            if len(top2) < 3:
                continue
            mod = top2[1]
            if mod not in ("domain", "rules", "data"):
                errs.append(f"M3 {f.relative_to(REPO)}: 跨 crate 可见面 `{path}` 不在 c1 契约模块（domain/rules/data）内")
    return errs


def m4_draft_status() -> tuple[list[str], bool]:
    """draft 残留：S12 契约冻结轮后 = FAIL；当前 S11 阶段 = WARN。"""
    warns = []
    pkgs = parse_packages(PACKAGES.read_text(encoding="utf-8"))
    for p in pkgs:
        if p["status"] != "frozen":
            warns.append(f"M4 包 {p['id']} status={p['status']}（并行轮启动前须全票 frozen）")
    for md in sorted(CONTRACTS.glob("c*.md")):
        text = md.read_text(encoding="utf-8")
        if "**状态：frozen**" not in text:
            warns.append(f"M4 契约 {md.name} 未 frozen（并行轮启动前须全票 frozen）")
    return warns, False  # 当前只 WARN


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    grp = ap.add_mutually_exclusive_group()
    grp.add_argument("--check", action="store_true", help="门禁模式：M1-M3 违规即非 0；M4 仅 WARN")
    grp.add_argument("--report", action="store_true", help="报告模式：列明细，恒 0")
    a = ap.parse_args()

    pkgs = parse_packages(PACKAGES.read_text(encoding="utf-8"))
    errs: list[str] = []
    errs += m1_disjoint(pkgs)
    errs += m2_anchor_exists()
    errs += m3_app_imports_only_contract()
    m4_warns, _ = m4_draft_status()

    print(f"check_module_contracts: {len(pkgs)} packages, M1-M3 {len(errs)} violation(s), M4 {len(m4_warns)} warn(s)")
    for e in errs:
        print(f"  {e}", file=sys.stderr if a.check and not a.report else sys.stdout)
    for w in m4_warns:
        print(f"  WARN {w}")

    if a.report:
        return 0
    if errs:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
