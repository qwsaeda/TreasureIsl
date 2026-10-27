#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""门禁项数 / 台账计数 **实测回写**（`--fix`），消灭「手抄 N 处」的漂移源。

**为什么有这个工具**（`docs/process/15_process_ledger.md` 第 12 条流程）：
门禁项数一改，`parity.md` / `git.md` / `docs/README.md` / `loop.md` / `AGENTS.md` / `style.md` /
`playbook.ps1` / `gate.ps1` 横幅共 8 处要手改（S20.0c dod 自认牵动 7 处）⇒ 漏一处 = 文档说谎
（假安全感）。本工具**从权威源实测**后统一回写：

- 门禁项数口径 = `scripts/gate.ps1` 顶层（不缩进）`Report ` 调用数（与
  `doccheck2::gate_item_actual_count` **同口径**）；
- 台账计数 = `plan/ledger.yaml` 任务行数 / `status: verified` 行数（与 `cargo run -p ti-tools --
  bin ledger` 同口径）。

用法：
    python scripts/doc_numbers.py            # 只读体检：列出各声称点当前值 vs 实测值（漂移即非 0）
    python scripts/doc_numbers.py --fix      # 实测回写全部声称点
    python scripts/doc_numbers.py --fix --dry-run   # 只打印将要改什么

退出码：0 = 无漂移（或 `--fix` 后已一致）；1 = 有漂移且未 `--fix`。
幂等：`--fix` 连跑两次 = 第二次 0 改动（实测值回填后再次测得一致）。
"""
from __future__ import annotations

import argparse
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
GATE = ROOT / "scripts" / "gate.ps1"
LEDGER = ROOT / "plan" / "ledger.yaml"

# 门禁项数声称点所在文件（gate-total 声称的载体；正则按行做**护栏匹配**，见 claim_sites()）
GATE_CLAIM_FILES: list[str] = [
    "docs/parity.md",
    "docs/git.md",
    "docs/README.md",
    "docs/loop.md",
    "AGENTS.md",
    "docs/style.md",
    "scripts/playbook.ps1",
    "scripts/gate.ps1",
]
# 台账计数声称点：release.md 的「plan/ledger.yaml` N 任务（M verified」
LEDGER_CLAIM = re.compile(r"(`plan/ledger\.yaml`\s*)(\d+)\s*任务\s*（(\d+)\s*verified")

# gate-total 语境标记（本行在谈门禁总数）
_GATE_CTX = re.compile(r"门禁|全绿|全门禁|gate|Report|当前")
# 「N 项」后的其它总数（命令数 / 图标数 …）——非门禁项数，须排除
_OTHER_AFTER = re.compile(r"(命令|图标)")
# 其它总数的前缀（残留 / 图标 计数）
_OTHER_BEFORE = re.compile(r"(残留|图标|各算)\s*$")
# 阈值（≥/≤ N 项 = 下限断言，非当前总数）
_THRESHOLD = re.compile(r"[≥<=]{1,2}\s*$")
# 历史快照 / 阶段时点值（「均为该阶段时点快照」「历史上因…」）——不得回写
_SNAPSHOT = re.compile(r"快照|时点|历史上|各算|单列")
# 非门禁的「N 项」语境（EXP-118）：doccheck2 / doccheck / 其它工具自身的项数
_NOT_GATE_COUNT = re.compile(r"doccheck2?\s*(?:第\s*)?\d*\s*(?:→|->|~|项)|check_code_standards\.py`?\s*\d+|check_code_standards\.py\s*\d+")


def _is_gate_total(line: str, m: re.Match) -> bool:
    """护栏：m 是 数字+「项」匹配 且本行在谈**当前**门禁总数。

    排除：序数（第 N 项）/ 其它总数（N 项命令·图标·残留·各算）/ 阈值（≥·≤ N 项）/
    历史快照（N 项均为时点快照）。仅「当前总数」的断言才允许被 --fix 回写。
    """
    before = line[max(0, m.start() - 12): m.start()]
    after = line[m.end(): m.end() + 3]
    if "第" in before[-6:]:
        return False  # 序数「第 N 项」/「第 15/16 项」= 门禁第几项，非总数
    if _OTHER_BEFORE.search(before):
        return False  # 「残留 N 项」「图标 N 项」「各算 1 项」
    if _OTHER_AFTER.search(after):
        return False  # 「N 项命令」「N 项图标」
    if _THRESHOLD.search(before[-3:]):
        return False  # 「≥ N 项」「≤ N 项」= 下限/上限断言
    if _SNAPSHOT.search(line):
        return False  # 历史时点快照，非当前值
    if _NOT_GATE_COUNT.search(line[max(0, m.start() - 40): m.start() + 1]):
        return False  # 「doccheck2 34→12 项」「check_code_standards.py 12 项」= 其它工具项数
    if not _GATE_CTX.search(line):
        return False  # 本行与门禁无关
    return True


def claim_sites(rel: str, text: str, gate_total: int) -> list[tuple[int, re.Match]]:
    """返回某文件中「应等于 gate_total 但目前不是」的 (line_idx, match) 列表。"""
    out = []
    for i, line in enumerate(text.splitlines()):
        for m in re.finditer(r"(\d+)\s*项", line):
            if m.group(1) == str(gate_total):
                continue
            if _is_gate_total(line, m):
                out.append((i, m))
    return out


def gate_item_count() -> int:
    """与 doccheck2::gate_item_actual_count **同口径**：`Report ` 调用名去重计数。

    ⚠️ 历史坑（EXP-118）：旧实现只数「顶层（不缩进）`Report ` 行」= 12，与 doccheck2
    的真实口径（任意缩进 + 调用名去重，`Skip` 与 `function Report` 不计）= 28 不一致
    ⇒ `--fix` 会把全库声称的 28 项回写成 12 ⇒ doccheck2 立即报「门禁项数声称漂移」。
    现改为与 doccheck2 逐字对齐（同名调用去重）。
    """
    if not GATE.is_file():
        return 0
    names: set[str] = set()
    for line in GATE.read_text(encoding="utf-8").splitlines():
        trimmed = line.lstrip()
        if not trimmed.startswith("Report "):
            continue
        rest = trimmed[len("Report "):]
        quotes = [i for i, ch in enumerate(rest) if ch in "'\""]
        if len(quotes) >= 2 and quotes[-1] > quotes[0] + 1:
            names.add(rest[quotes[0] + 1:quotes[-1]])
    return len(names)


def ledger_counts() -> tuple[int, int]:
    if not LEDGER.is_file():
        return (0, 0)
    text = LEDGER.read_text(encoding="utf-8")
    tasks = len(re.findall(r"^\s*-\s*\{\s*id:\s*", text, re.M))
    verified = sum(1 for line in text.splitlines() if re.match(r"^\s*-\s*\{\s*id:\s*", line) and "status: verified" in line)
    return (tasks, verified)


def main() -> int:
    ap = argparse.ArgumentParser(description="门禁项数 / 台账计数 实测回写（--fix 消灭手抄漂移）")
    ap.add_argument("--fix", action="store_true", help="实测回写全部声称点（默认只体检）")
    ap.add_argument("--dry-run", action="store_true", help="配合 --fix：只打印将要改什么")
    args = ap.parse_args()

    g = gate_item_count()
    tasks, verified = ledger_counts()
    print(f"  实测：门禁 {g} 项 / 台账 {tasks} 任务（{verified} verified）")

    drift = 0
    for rel in GATE_CLAIM_FILES:
        p = ROOT / rel
        if not p.is_file():
            continue
        text = p.read_text(encoding="utf-8")
        sites = claim_sites(rel, text, g)
        if not sites:
            continue
        out_lines = text.splitlines()
        for i, m in reversed(sites):  # 逆序替换避免行号漂移
            new_line = out_lines[i][: m.start(1)] + str(g) + out_lines[i][m.end(1):]
            out_lines[i] = new_line
            drift += 1
            if args.fix and not args.dry_run:
                pass
            elif args.fix:
                print(f"    将改 {rel}:{i + 1}: {m.group(1)} → {g} :: {out_lines[i].strip()[:90]}")
        if args.fix and not args.dry_run:
            p.write_text("\n".join(out_lines) + ("\n" if text.endswith("\n") else ""), encoding="utf-8", newline="")

    # 台账计数（release.md）
    rel_path = ROOT / "docs" / "release.md"
    if rel_path.is_file():
        text = rel_path.read_text(encoding="utf-8")
        m = LEDGER_CLAIM.search(text)
        if m and (m.group(2) != str(tasks) or m.group(3) != str(verified)):
            drift += 1
            new_text = LEDGER_CLAIM.sub(
                lambda mm: f"{mm.group(1)}{tasks} 任务（{verified} verified", text, count=1
            )
            if args.fix and not args.dry_run:
                rel_path.write_text(new_text, encoding="utf-8", newline="")
            else:
                print(f"    将改 docs/release.md: {m.group(2)}/{m.group(3)} → {tasks}/{verified}")

    if drift == 0:
        print("  声称点与实测一致（0 漂移）")
        return 0
    if not args.fix:
        print(f"  漂移 {drift} 处（用 --fix 回写）", file=sys.stderr)
        return 1
    print(f"  已回写 {drift} 处声称点")
    return 0


if __name__ == "__main__":
    sys.exit(main())
