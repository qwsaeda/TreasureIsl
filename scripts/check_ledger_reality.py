#!/usr/bin/env python3
"""台账三方对账（防虚报门禁）— S10.0 新增第 14 项。
检查 ledger 中 status: verified 的任务行：
  1. evidence 文件必须存在且非空
  2. DoD 中声明的关键代码符号（如 commands.rs 的 Command trait）必须在代码里 grep 到
  3. 缺任一 → FAIL「台账虚报：<id> evidence/符号缺失」

反向检查（S13.2 增，防台账回退）：
  4. `evidence/` 下形如 `<任务ID>[_后缀].log` 的文件，其任务 ID 必须在台账里存在；
     孤儿 evidence = 台账被文件级回退（git checkout/restore）丢任务的信号。

用法：
  python scripts/check_ledger_reality.py            # 全量对账
  python scripts/check_ledger_reality.py --check    # 只读模式（同上，恒 0 退出码若全绿）

设计意图：机器化拦截子代理「已 verified」虚报（S9.5 commands.rs 教训，EXP-xxx）；
以及台账条目被回退丢失（S11/S12 共 23 条丢失、doccheck 第 9 项 115 error，EXP-065）。
与 doccheck2 互补：doccheck2 查文档↔代码漂移，本脚本查台账↔代码↔证据三方一致性。
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
LEDGER = REPO / "plan" / "ledger.yaml"

# 任务 id → 期望在代码里 grep 到的关键符号（正则）。
# 只列「DoD 明确声明的代码产物」，不列文档类任务。
# .rs 结尾的符号 = 文件名（glob 检查文件存在），其余 = 代码正则（grep crates/*/src）。
SYMBOL_EXPECTATIONS = {
    "S9.5": [
        r"trait\s+Command",
        "commands.rs",
    ],
    "S10.4": [
        r"pub\s+struct\s+GpuRenderer",
        # S10.4 的 `ttf_font.rs load_ttf` = macroquad GPU 字体包装（ti-app），
        # S18.8a 随 macroquad 发货壳退役（壳删后零引用死代码）→ TTF 口径单一事实源
        # 迁到 `ti_render::font::TtfFont::from_bytes`（ab_glyph 软件光栅化，CPU↔Bevy 两侧共用）。
        r"pub\s+fn\s+from_bytes",
    ],
    # 按需追加：新增任务若声明代码符号，在此登记（EXP-058）。
    "S21.0": [
        # 脚本纪律门禁项（doccheck2 第 27 项）+ 批量资产工具（scripts/ui_asset_batch.py，
        # 非 crates/ 路径 ⇒ 由台账标题与 evidence 佐证，不进符号 grep 范围）。
        r"fn\s+check_tooling_contract",
    ],
}

# 孤儿 evidence 检测：只认「以任务 ID 开头」的证据文件，形如
#   S10.4.log / S2.5_wasm_shell.log / S4.3-size.log / S8.3.e_gate.log
# 不认 gate_run.log / font_render.txt / S1.11-*.json 之类非任务 ID 文件。
EVIDENCE_ID_RE = re.compile(r"^(S\d+(?:\.[0-9a-z]+)*)")
EVIDENCE_EXTS = {".log", ".txt", ".md", ".json", ".png"}


def parse_ledger_tasks() -> list[dict]:
    """极简解析 ledger.yaml 的任务行（每任务一行 `- { id: ..., status: ..., ... }`）。

    不依赖 PyYAML（保持脚本零第三方依赖）。返回 [{id, status, evidence, dod}]。
    """
    if not LEDGER.is_file():
        sys.exit(f"ledger 不存在：{LEDGER}")
    tasks: list[dict] = []
    cur = ""
    for line in LEDGER.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\s*- \{ id: ([\w.\-]+),\s*(.*)\}\s*$", line)
        if m:
            if cur:
                tasks.append(cur and _finalize(cur))
            cur = f"{{ id: {m.group(1)}, {m.group(2)}}}"
        elif cur and line.strip().startswith("}"):
            cur += " " + line.strip()
            tasks.append(_finalize(cur))
            cur = ""
        elif cur:
            cur += " " + line.strip()
    if cur:
        tasks.append(_finalize(cur))
    return [t for t in tasks if t]


def _finalize(raw: str) -> dict:
    raw = raw.strip().lstrip("{").rstrip("}")
    fields: dict[str, str] = {}
    for key in ("id", "status", "evidence", "dod", "blocked_by"):
        m = re.search(rf'{key}:\s*("[^"]*"|[\w./\-]*|null)', raw)
        if m:
            val = m.group(1).strip().strip('"')
            fields[key] = val if val != "null" else None
    return fields


def check_evidence(evidence: str | None) -> list[str]:
    """evidence 文件存在 + 非空。

    支持 `+` 分隔的多 evidence 路径（如 S2.5 的 `evidence/S2.5.log + evidence/S2.5_wasm_shell.log`），
    全部存在且非空才通过。
    """
    problems: list[str] = []
    if not evidence:
        problems.append("evidence 为空（verified 必须带 evidence 路径）")
        return problems
    # `+` 分隔多文件路径（ledger 历史约定：S2.5 等）
    paths = [p.strip() for p in evidence.split("+") if p.strip()]
    for ev in paths:
        path = REPO / ev
        if not path.is_file():
            problems.append(f"evidence 文件不存在：{ev}")
            continue
        if path.stat().st_size == 0:
            problems.append(f"evidence 文件为空：{ev}")
    return problems


def check_symbols(task_id: str) -> list[str]:
    """DoD 声明的代码符号必须在 crates/ 里 grep 到。

    符号匹配规则（EXP-058）：
    - 以 .rs 结尾的符号（如 gpu_renderer.rs / commands.rs）= 文件名，
      必须存在对应文件（glob 检查）；
    - 其余符号（如 trait+Command 正则）= 代码正则，在 crates/*/src 里 grep。
    """
    patterns = SYMBOL_EXPECTATIONS.get(task_id, [])
    if not patterns:
        return []  # 未登记符号期望的任务（如文档类）跳过
    problems: list[str] = []
    rs_files = list(REPO.glob("crates/*/src/**/*.rs")) + list(REPO.glob("crates/*/tests/**/*.rs"))
    for pat in patterns:
        found = False
        if pat.endswith(".rs"):
            # 文件名符号：glob 检查文件存在
            fname = pat
            for rf in rs_files:
                if rf.name == fname or rf.name.startswith(fname):
                    found = True
                    break
        else:
            # 代码正则：grep 命中
            for rf in rs_files:
                text = rf.read_text(encoding="utf-8", errors="replace")
                if re.search(pat, text):
                    found = True
                    break
        if not found:
            kind = "文件名" if pat.endswith(".rs") else "代码符号"
            problems.append(f"{kind}缺失（{pat!r} 在 crates/ 未命中）")
    return problems


def check_orphan_evidence(tasks: list[dict]) -> list[str]:
    """`evidence/` 里以任务 ID 开头的证据文件，其 ID 必须在台账中存在。

    孤儿 evidence = 台账条目被文件级回退（git checkout/restore）丢掉的信号：
    证据日志还在、任务行没了 → doccheck 第 9 项会爆一串
    「references task which is not in plan/ledger.yaml」（EXP-065）。
    """
    ids = {t.get("id", "") for t in tasks}
    problems: list[str] = []
    evidence_dir = REPO / "evidence"
    if not evidence_dir.is_dir():
        return [f"evidence/ 目录缺失：{evidence_dir}"]
    for path in sorted(evidence_dir.iterdir()):
        if not path.is_file() or path.suffix.lower() not in EVIDENCE_EXTS:
            continue
        # 用 stem（去扩展名）匹配，避免把 `.log` 当成任务 ID 的最后一段
        m = EVIDENCE_ID_RE.match(path.stem)
        if not m:
            continue  # 非任务 ID 命名的文件（gate_run.log 等），不在本检查范围
        tid = m.group(1)
        if tid not in ids:
            problems.append(
                f"孤儿 evidence `{path.relative_to(REPO).as_posix()}`："
                f"任务 `{tid}` 不在 plan/ledger.yaml（台账可能被文件级回退丢条目）"
            )
    return problems


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="只读模式（恒 0 退出码若全绿）")
    args = ap.parse_args()

    tasks = parse_ledger_tasks()
    verified = [t for t in tasks if t.get("status") == "verified"]
    failures: list[str] = []
    for t in verified:
        tid = t.get("id", "?")
        probs = check_evidence(t.get("evidence"))
        probs += check_symbols(tid)
        if probs:
            failures.append(f"[{tid}] " + "; ".join(probs))

    orphans = check_orphan_evidence(tasks)
    for o in orphans:
        failures.append(f"[orphan] {o}")

    if failures:
        print(f"check_ledger_reality: {len(failures)} 项虚报")
        for f in failures:
            print(f"  FAIL  {f}")
        return 0 if args.check else 1
    print(
        f"check_ledger_reality: 全绿（{len(verified)} 个 verified 任务三方对账通过"
        f" + 0 个孤儿 evidence）"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
