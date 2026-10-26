#!/usr/bin/env python3
"""check_code_standards.py — 代码标准机器检查器（docs/process/16_code_standards.md）。

四项检查（CS-1~CS-4），全部「存量豁免 + 新增零豁免」（棘轮只收紧不放宽，
基线 = scripts/process_ratchet.json 的 `code_standards` 段）：

  CS-1  asset_path_hardcode   资产路径字面量只允许在 chrome.rs（资产表）/ ti-core data 层 / ti-tools
                              （单一事实源，02_conventions §1；表现层散写 assets/ = 耦合）
  CS-2  file_doc_present      src/ 每个 .rs 文件首部必须有 //! 模块文档（负责什么/不负责什么，
                              02_conventions §9.1「一个文件 = 一个职责」）
  CS-3  fn_param_naming       fn 参数名禁单字母（例外 {x, y} 坐标；02_conventions §3 禁无意义缩写）
  CS-4  scene_session_ctor    场景层禁构造持久会话（BoatSession/BoardSession/MapSession/BattleSession
                              = 架构窄接缝，会话归属 game_app；scenes 只经 host 读写）

  规模维度（文件 >900 行 / 函数 >140 行 / 重复模式）归 `audit_code.py`（gate 独立项，
  code_budget.json 基线），本工具不重复 —— 单一事实源纪律。

豁免语义：
  - CS-1/CS-2（文件级）：豁免键 = 仓库相对文件路径；当前违例文件若不在豁免表 = FAIL。
  - CS-3/CS-4（函数级）：豁免键 = `relpath::fn_name`；当前违例函数若不在豁免表 = FAIL。
  - `--update-baseline` 取「现有豁免 ∪ 当前违例」写回（只增不删，防放宽）；新增条目
    reason=「新增豁免」，须人工补 fix_in 任务号（同 15_process_ledger §2 纪律）。

用法：
  python scripts/check_code_standards.py --report          # 打印各项违例明细 + 计数（信息性，exit 0）
  python scripts/check_code_standards.py --check           # 对照基线门禁判定（违例超基线 = 非 0 退出）
  python scripts/check_code_standards.py --update-baseline # 写回基线（人工确认后执行）
  只读：本脚本从不修改任何 .rs；`--update-baseline` 只写 process_ratchet.json。
"""
from __future__ import annotations

import argparse
import io
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RATCHET = os.path.join(ROOT, "scripts", "process_ratchet.json")

PARAM_EXEMPT = {"x", "y"}  # CS-3 坐标例外

# CS-1 资产路径字面量允许驻留：chrome.rs（资产表单一读点）/ ti-core data 层 / ti-tools 工具链
ASSET_RE = re.compile(r"assets/[\w./-]+\.(?:png|json|jpg|jpeg|svg|gif|webp)")

# CS-4 场景层禁止构造的**持久**会话（构造 = game_app 层职责；一次性规则对象
# BuildManager/TradeLedger 按次结算即弃，不在此列）
SCENE_SESSION_RE = re.compile(
    r"\b(BoatSession|BoardSession|MapSession|BattleSession)\s*::\s*(new|from_balance|restored)\b"
)
FN_SIG_RE = re.compile(r"fn\s+([A-Za-z0-9_]+)\s*\(([^)]*)\)")


def rel(p: str) -> str:
    return os.path.relpath(p, ROOT).replace("\\", "/")


def iter_src_rs():
    for base, dirs, files in os.walk(os.path.join(ROOT, "crates")):
        dirs[:] = [d for d in dirs if d != "target"]
        for f in files:
            if f.endswith(".rs") and os.sep + "src" in base + os.sep:
                yield os.path.join(base, f)


def read(p):
    with io.open(p, "r", encoding="utf-8", errors="replace") as fh:
        return fh.readlines()


def is_comment(line: str) -> bool:
    return line.lstrip().startswith("//")


def cs1_asset_violations():
    out = []
    for p in iter_src_rs():
        rp = rel(p)
        if ("chrome.rs" in rp and "scenes" in rp) or rp.startswith("crates/ti-core/src/data/") \
                or rp.startswith("crates/ti-tools/"):
            continue
        for i, line in enumerate(read(p), 1):
            if is_comment(line):
                continue
            m = ASSET_RE.search(line)
            if m:
                out.append((rp, i, m.group(0)))
    return out


def cs2_doc_violations():
    out = []
    for p in iter_src_rs():
        lines = read(p)
        if not any(l.lstrip().startswith("//!") for l in lines[:6]):
            out.append(rel(p))
    return out


def cs3_param_violations():
    out = []
    for p in iter_src_rs():
        rp = rel(p)
        for i, line in enumerate(read(p), 1):
            if is_comment(line):
                continue
            for m in FN_SIG_RE.finditer(line):
                fname = m.group(1)
                for seg in m.group(2).split(","):
                    seg = seg.strip()
                    if not seg:
                        continue
                    nm = re.match(r"(?:\*const\s+|\*mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:", seg)
                    if nm:
                        ident = nm.group(1)
                        if len(ident) == 1 and ident not in PARAM_EXEMPT:
                            out.append((f"{rp}::{fname}", i, ident))
    return out


def cs4_scene_violations():
    out = []
    for p in iter_src_rs():
        rp = rel(p)
        if not rp.startswith("crates/ti-app/src/scenes/"):
            continue
        for i, line in enumerate(read(p), 1):
            if is_comment(line):
                continue
            m = SCENE_SESSION_RE.search(line)
            if m:
                out.append((f"{rp}::(scene)", i, m.group(0)))
    return out


CHECKS = {
    "CS-1 asset_path_hardcode": cs1_asset_violations,
    "CS-2 file_doc_present": cs2_doc_violations,
    "CS-3 fn_param_naming": cs3_param_violations,
    "CS-4 scene_session_ctor": cs4_scene_violations,
}

BASELINE_KEY = {
    "CS-1 asset_path_hardcode": "cs1_asset_grandfathered",
    "CS-2 file_doc_present": "cs2_doc_grandfathered",
    "CS-3 fn_param_naming": "cs3_param_grandfathered",
    "CS-4 scene_session_ctor": "cs4_scene_grandfathered",
}

DESC = {
    "CS-1 asset_path_hardcode": "资产路径字面量只在 chrome/data/ti-tools",
    "CS-2 file_doc_present": "src 文件缺 //! 模块文档（职责声明）",
    "CS-3 fn_param_naming": "fn 参数单字母（非 x/y）",
    "CS-4 scene_session_ctor": "场景层构造持久会话（窄接缝）",
}


def item_key(item):
    return item[0] if isinstance(item, (list, tuple)) else item


def load_baseline():
    if not os.path.exists(RATCHET):
        return {}
    with io.open(RATCHET, "r", encoding="utf-8") as fh:
        return json.load(fh).get("code_standards", {})


def save_baseline(sec):
    data = {}
    if os.path.exists(RATCHET):
        with io.open(RATCHET, "r", encoding="utf-8") as fh:
            data = json.load(fh)
    data["code_standards"] = sec
    with io.open(RATCHET, "w", encoding="utf-8") as fh:
        json.dump(data, fh, ensure_ascii=False, indent=2)
        fh.write("\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--report", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--update-baseline", action="store_true")
    args = ap.parse_args()

    base = load_baseline()
    fails = 0
    for check, fn in CHECKS.items():
        v = fn()
        if args.report or args.check:
            print(f"\n=== {check}（{DESC[check]}）— 当前违例 {len(v)} ===")
            for item in v[:40]:
                print("   ", item)
            if len(v) > 40:
                print(f"    ...（{len(v) - 40} 更多）")
        if args.update_baseline:
            key = BASELINE_KEY[check]
            cur = {item_key(item) for item in v}
            existing = base.get(key, {})
            merged = dict(existing)
            for c in sorted(cur - set(existing)):
                merged[c] = {"reason": "新增豁免", "fix_in": "TBD"}
            base[key] = merged
            print(f"update {key}: 豁免 {len(merged)}（新加 {len(cur - set(existing))}）")
        if args.check:
            allowed = set(base.get(BASELINE_KEY[check], {}))
            offenders = {item_key(item) for item in v}
            new = offenders - allowed
            if new:
                fails += 1
                print(f"\n[FAIL] {check}：{len(new)} 个新增违例（不在豁免基线）：")
                for n in sorted(new)[:30]:
                    print("   +", n)
            else:
                print(f"[ok] {check}：{len(offenders)} 存量违例全在豁免基线内")

    if args.update_baseline:
        save_baseline(base)
        print("基线已写回 scripts/process_ratchet.json::code_standards（人工补 fix_in）")
        return
    if args.check:
        print(f"\ncheck_code_standards: {'FAIL' if fails else 'PASS'}（{fails} 项超基线）")
        sys.exit(1 if fails else 0)
    if not (args.report or args.check or args.update_baseline):
        ap.print_help()


if __name__ == "__main__":
    main()
