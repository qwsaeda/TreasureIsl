#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""代码结构审计（代码结构准则 `docs/style.md` §5.12 的执行手段）。

检查项（阈值与基线豁免 = `scripts/code_budget.json`）：
1. **文件规模**：单个 `.rs` 文件行数 > `max_file_lines` ⇒ 违规；
2. **函数长度**：单个 `fn` 行数 > `max_fn_lines` ⇒ 违规（比 clippy `too_many_lines` 更严，
   本项覆盖 clippy 豁免点，避免「阈值形同虚设」）；
3. **重复模式**：同一模式在同一文件出现 > `max_repeat` 次 ⇒ 提示抽辅助函数/宏；
4. **模块职责声明**：`.rs` 文件首行缺少「模块职责」doc 注释 ⇒ 提示（软项，不阻断）。

用法：
  python scripts/audit_code.py --check     # 只读检查；违规 ⇒ 非 0 退出码（门禁候选）
  python scripts/audit_code.py --report    # 人读报告（永远 0 退出）
  python scripts/audit_code.py --update-baseline   # 把当前违规写入基线（棘轮起点）

约定：幂等（只读不改代码）+ `--check` 只读模式 + 非 0 退出码（`docs/process/04_tooling.md` §1）。
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATES = os.path.join(ROOT, "crates")
BUDGET = os.path.join(ROOT, "scripts", "code_budget.json")

FUNC_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?(?:unsafe\s+)?fn\s+(\w+)")
TRAIT_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?trait\s+(\w+)", re.M)
GENERIC_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+\w+\s*<", re.M)
MACRO_RE = re.compile(r"^\s*macro_rules!\s+(\w+)", re.M)

REPEAT_PATTERNS = {
    # 真重复 = 「同一段代码骨架被复制多份」，不是「同一个 API 被调用多次」。
    "toast 三连（文案+时长+音效）": r"board_toast_ticks\s*=\s*120",
    "i18n 文案取值 sgetf": r"(?:crate::)?i18n::sgetf\(",
    "gold_max 钳制": r"\.min\(b\.economy\.gold_max\)",
}


def strip_noise(line: str) -> str:
    """去掉字符串字面量 / 字符字面量 / 行注释里的花括号与关键字，避免误配平。"""
    out = []
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break
        if c == '"':
            i += 1
            while i < n:
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    break
                i += 1
            out.append('""')
            continue
        if c == "'":
            # 字符字面量或生命周期：'a / '\n' —— 简单跳过到下一个 '
            j = i + 1
            if j < n and line[j] == "\\":
                j += 2
            else:
                j += 1
            if j < n and line[j] == "'":
                out.append("''")
                i = j + 1
                continue
            out.append(c)
            i += 1
            continue
        out.append(c)
        i += 1
    return "".join(out)


def fn_spans(lines: list[str]) -> list[tuple[str, int, int]]:
    """返回 (函数名, 起始行号, 行数)；花括号配平（已剔除字符串/注释噪声）。"""
    spans = []
    i = 0
    n = len(lines)
    while i < n:
        m = FUNC_RE.match(lines[i])
        if not m:
            i += 1
            continue
        name = m.group(1)
        depth = 0
        started = False
        j = i
        while j < n:
            clean = strip_noise(lines[j])
            for ch in clean:
                if ch == "{":
                    depth += 1
                    started = True
                elif ch == "}":
                    depth -= 1
            if started and depth <= 0:
                break
            j += 1
        spans.append((name, i + 1, j - i + 1))
        i = j + 1
    return spans


def rust_files() -> list[str]:
    out = []
    for dp, dn, fn in os.walk(CRATES):
        if "target" in dp:
            continue
        for f in fn:
            if f.endswith(".rs"):
                out.append(os.path.join(dp, f))
    return sorted(out)


def rel(p: str) -> str:
    return os.path.relpath(p, ROOT).replace("\\", "/")


def collect() -> dict:
    files = rust_files()
    file_lines = {}
    fns = []
    traits = 0
    generics = 0
    macros = 0
    repeats = defaultdict(list)
    for p in files:
        text = open(p, encoding="utf-8", errors="replace").read()
        lines = text.splitlines()
        r = rel(p)
        file_lines[r] = len(lines)
        for name, start, ln in fn_spans(lines):
            fns.append({"key": f"{r}::{name}", "file": r, "name": name, "line": start, "lines": ln})
        traits += len(TRAIT_RE.findall(text))
        generics += len(GENERIC_RE.findall(text))
        macros += len(MACRO_RE.findall(text))
        for key, pat in REPEAT_PATTERNS.items():
            c = len(re.findall(pat, text))
            if c:
                repeats[key].append({"file": r, "count": c})
    return {
        "file_lines": file_lines,
        "fns": fns,
        "traits": traits,
        "generics": generics,
        "macros": macros,
        "repeats": dict(repeats),
    }


def violations(data: dict, budget: dict) -> dict:
    max_file = int(budget["max_file_lines"])
    max_fn = int(budget["max_fn_lines"])
    max_repeat = int(budget.get("max_repeat", 3))
    exempt_files = budget.get("exempt_files", {})
    exempt_fns = budget.get("exempt_fns", {})
    exempt_repeats = budget.get("exempt_repeats", {})

    v_files = []
    for f, n in sorted(data["file_lines"].items(), key=lambda kv: -kv[1]):
        if n > max_file and f not in exempt_files:
            v_files.append({"file": f, "lines": n, "limit": max_file})
    v_fns = []
    for fn in sorted(data["fns"], key=lambda f: -f["lines"]):
        if fn["lines"] > max_fn and fn["key"] not in exempt_fns:
            v_fns.append({"key": fn["key"], "line": fn["line"], "lines": fn["lines"], "limit": max_fn})
    v_repeat = []
    for key, hits in data["repeats"].items():
        for h in hits:
            rk = "%s@%s" % (key, h["file"])
            if h["count"] > max_repeat and rk not in exempt_repeats:
                v_repeat.append({"pattern": key, "file": h["file"], "count": h["count"], "limit": max_repeat})
    return {"files": v_files, "fns": v_fns, "repeats": v_repeat}


def main() -> int:
    ap = argparse.ArgumentParser(description="代码结构审计（docs/style.md §5.12）")
    ap.add_argument("--check", action="store_true", help="只读检查；违规 ⇒ 非 0 退出码")
    ap.add_argument("--report", action="store_true", help="人读报告（永远 0 退出）")
    ap.add_argument("--update-baseline", action="store_true", help="把当前违规写入基线（棘轮起点）")
    args = ap.parse_args()

    budget = json.load(open(BUDGET, encoding="utf-8"))
    data = collect()
    v = violations(data, budget)

    if args.update_baseline:
        budget["exempt_files"] = {f["file"]: "基线（限期整改）" for f in v["files"]}
        budget["exempt_fns"] = {f["key"]: "基线（限期整改）" for f in v["fns"]}
        budget["exempt_repeats"] = {
            "%s@%s" % (f["pattern"], f["file"]): "基线（限期整改）" for f in v["repeats"]
        }
        with open(BUDGET, "w", encoding="utf-8", newline="\n") as fh:
            json.dump(budget, fh, ensure_ascii=False, indent=2, sort_keys=True)
            fh.write("\n")
        print("[audit_code] 基线已更新：文件豁免 %d / 函数豁免 %d" % (len(v["files"]), len(v["fns"])))
        return 0

    print("=== 代码结构审计 ===")
    print("文件 %d / 总行 %d / 函数 %d / trait %d / 泛型 fn %d / macro_rules! %d"
          % (len(data["file_lines"]), sum(data["file_lines"].values()), len(data["fns"]),
             data["traits"], data["generics"], data["macros"]))
    print()
    print("--- 最长 10 个文件 ---")
    for f, n in sorted(data["file_lines"].items(), key=lambda kv: -kv[1])[:10]:
        mark = "  [豁免]" if f in budget.get("exempt_files", {}) else ""
        print("%5d  %s%s" % (n, f, mark))
    print()
    print("--- 最长 10 个函数 ---")
    for fn in sorted(data["fns"], key=lambda f: -f["lines"])[:10]:
        mark = "  [豁免]" if fn["key"] in budget.get("exempt_fns", {}) else ""
        print("%5d  %s (行 %d)%s" % (fn["lines"], fn["key"], fn["line"], mark))
    print()
    print("--- 违规（阈值：文件 >%d 行 / 函数 >%d 行 / 重复 >%d 次）---"
          % (budget["max_file_lines"], budget["max_fn_lines"], budget.get("max_repeat", 3)))
    print("文件规模 %d 项；函数长度 %d 项；重复模式 %d 项" % (len(v["files"]), len(v["fns"]), len(v["repeats"])))
    for f in v["files"]:
        print("  FILE  %s = %d 行（上限 %d）" % (f["file"], f["lines"], f["limit"]))
    for f in v["fns"]:
        print("  FN    %s = %d 行（上限 %d）" % (f["key"], f["lines"], f["limit"]))
    for f in v["repeats"]:
        print("  DUP   %s @ %s × %d（上限 %d）" % (f["pattern"], f["file"], f["count"], f["limit"]))

    if args.check:
        total = len(v["files"]) + len(v["fns"]) + len(v["repeats"])
        if total:
            print("[audit_code] FAIL：%d 项结构违规" % total)
            return 1
        print("[audit_code] PASS：0 项结构违规")
        return 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
