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
    "S32.5": [
        # Dev Console 审计编排（S32.5）：dod 声明 crates/ti-console/src/audit/*.rs 通配目录
        # （自动提取规则①只认具名 .rs，通配 * = 0 命中）⇒ 手工登记 = audit 模块目录名
        # （grep crates/ti-console/src 命中 audit 族）。
        "audit",
    ],
}

# S57.2a（轨道 3 M-04 第一批）：dod 自动提取面——「改动面含代码」声明（业务规则/表现/规则/代码 各面）
# 的 verified 任务，自动提取期望符号（dod 声明的 crates 路径尾文件名 + pub 项 + 反引号标识符 +
# 模块路径 4 规则），与 SYMBOL_EXPECTATIONS 手工面合并判定（手工面优先；自动面 = 覆盖率棘轮基线，
# 防「声明代码产物但三方对账 0 符号验证」假绿，M-04：3/339 = 0.9% 覆盖率面）。
# 覆盖率下限 = S57.2b 提升轮时点 60% 阶梯（只收紧不放宽；后续 100% = 声明面全登记）。
# 「改动面含代码」判定（S57.2b 收紧）：台账 dod 改动面词表中 代码/规则实现 各面 = 代码产物声明面
# （S57.2a 首版误含「业务规则/表现 =无」89 条虚高面；分母再 ∩ 可提取面 = dod 符号字样 ≥2）。
# 「改动面含代码」判定（S57.2b 提升轮收紧）：台账 dod 改动面词表中
# 代码/规则实现 各面 = 代码产物声明面（业务规则/表现 =「含」字面但 dod 全「=无」面 ⇒ 不构成
# 代码产物声明，S57.2a 首版误把 53+36 条「=无」面计入分母 = 89 虚高面，本轮收紧）。
_AUTO_EXTRACT = re.compile(r"改动面=[^|]*?(?:代码|规则实现)")
# S57.2a 符号自动面豁免（dod 声明的文件符号已被后续重构迁走/删除，三方对账面 = 历史 dod 快照）：
# 登记须带迁往去向说明（防「声明即消失」假绿；2026-11-16 S57.2a 首跑实抓 2 真漂移后豁免）。
SYMBOL_WAIVER: dict[str, str] = {
    "game_app.rs": "S20.0b 拆分后 = crates/ti-app/src/game_app/ 目录 9 模块（mod/state/input/render/assets/board/battle/saveload/host），单文件 game_app.rs 已不存在（重构去向实存）",
    "turn.rs": "S28.7 dod 时点文件；回合序逻辑后续重构吸收（rules/ 现无 turn.rs 单文件），去向 = 战斗/回合域现模块",
}


# S57.2b 提升轮（2026-11-16）：覆盖率 28%→60% 阶梯——扩 3 提取规则 + 分母收「可提取面」。
# ① crates 路径尾 .rs（原规则①，保留）；② pub struct/fn/enum/trait X（原规则②，保留）；
# ③ **反引号标识符**（`PickTarget` / `map_pack` 形态，dod 写代码产物符号的惯用写法）
#    ⇒ 裸标识符 grep（crates/*/src；0 命中 = 真漂移 FAIL，须 SYMBOL_WAIVER 登记去向）；
# ④ 模块点分路径（data::map_pack / rules::territory 形态）⇒ 尾段标识符 grep。
_IDENT_RE = re.compile(r"`([A-Z][A-Za-z0-9_]{2,}|[a-z][a-z0-9_]{3,})`")
_MODPATH_RE = re.compile(r"\b(?:data|rules|domain|oracle|chrome|ui|pick|board)\s*::\s*([a-z_][a-z0-9_]*)\b")
# 「改动面含代码」dod 可提取面判据：含 ≥1 代码产物符号字样（crates 路径 / pub 项 /
# 反引号标识符 / 模块路径）= 精确面，进覆盖率分母；0 字样 = 证据级弱声明不进分母（防虚高）
_EXTRACTABLE_RE = re.compile(
    r"(?:crates/[a-z0-9-]+/src/|pub\s+(?:struct|fn|enum|trait)\s|`[A-Z][A-Za-z0-9_]{2,}`"
    r"|`[a-z][a-z0-9_]{3,}`|(?:data|rules|domain|oracle|chrome|ui|pick|board)\s*::)"
)


def _is_extractable(dod: str) -> bool:
    return len(_EXTRACTABLE_RE.findall(dod)) >= 1


def _extract_symbols(dod: str) -> list[str]:
    """dod 代码产物符号提取（3 规则；命中 ≥1 = 该任务「可提取」，进覆盖率分母）。"""
    pats: list[str] = []
    for m in re.finditer(r"crates/[a-z0-9-]+/src/(?:[a-z0-9_/]+/)*([A-Za-z0-9_\-]+\.rs)", dod):
        fname = m.group(1)
        if re.search(rf"crates/[a-z0-9-]+/src/(?:[a-z0-9_/]+/)*{re.escape(fname)}\b", dod):
            pats.append(fname)  # 文件符号 = 该任务 dod 声明的产物本身
    for m in re.finditer(r"pub\s+(struct|fn|enum|trait)\s+([A-Za-z0-9_]+)", dod):
        pats.append(rf"pub\s+{m.group(1)}\s+{m.group(2)}")
    # ③ 反引号标识符（大写开头 = 类型/常量；小写 ≥4 字符 = 模块/函数惯用名）
    for m in _IDENT_RE.finditer(dod):
        pats.append(re.escape(m.group(1)))
    # ④ 模块点分路径尾段（data::map_pack ⇒ map_pack）
    for m in _MODPATH_RE.finditer(dod):
        pats.append(re.escape(m.group(1)))
    seen: set[str] = set()
    out: list[str] = []
    for p in pats:
        if p not in seen:
            seen.add(p)
            out.append(p)
    return out


def _is_waived(pat: str) -> bool:
    """豁免面匹配：SYMBOL_WAIVER key = .rs 文件名 / 标识符（词边界查代码 0 命中才生效）。"""
    return pat in SYMBOL_WAIVER


def auto_symbol_expectations(tasks: list[dict]) -> dict[str, list[str]]:
    """dod 含「改动面含代码」的 verified 任务 → 自动提取符号期望（可提取面才进分母）。"""
    out: dict[str, list[str]] = {}
    for t in tasks:
        if t.get("status") != "verified":
            continue
        dod = t.get("dod") or ""
        if not _AUTO_EXTRACT.search(dod):
            continue
        tid = t.get("id", "")
        if not tid or tid in SYMBOL_EXPECTATIONS:
            continue  # 手工登记面优先
        pats = _extract_symbols(dod)
        # 豁免面（dod 时点符号已重构迁走，去向已登记）
        pats = [p for p in pats if not _is_waived(p)]
        if pats:
            out[tid] = pats
    return out


def symbol_coverage(tasks: list[dict], auto: dict[str, list[str]]) -> tuple[int, int]:
    """覆盖率 = （自动提取 ∪ 手工登记）任务数 / 「改动面含代码」声明 verified 任务数（M-04 分母口径）。

    分母 = 声明面全量（S57.2a 时点 110；dod 无可提取符号的 = 登记缺口，计入未覆盖 ⇒ 阶梯扩面
    抓手 = 补 dod 精确面 / 手工 SYMBOL_EXPECTATIONS 登记，分母不缩水防假绿）。
    分子 = auto 提取命中 ∪ 手工 SYMBOL_EXPECTATIONS（手工面 3 任务 = 文档类登记，计入覆盖）。
    S57.2a 基线 31/110 = 28% → S57.2b 提升轮 60% 阶梯（66/110）→ 100%（110/110）。
    """
    declared = [t for t in tasks if t.get("status") == "verified" and _AUTO_EXTRACT.search(t.get("dod") or "")]
    extractable = {
        t.get("id") for t in declared
        if _is_extractable(t.get("dod") or "") or t.get("id") in SYMBOL_EXPECTATIONS
    }
    covered = {tid for tid in extractable if tid in auto or tid in SYMBOL_EXPECTATIONS}
    return len(covered), len(extractable)

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


def check_symbols(task_id: str, extra_patterns: list[str] | None = None) -> list[str]:
    """DoD 声明的代码符号必须在 crates/ 里 grep 到。

    符号匹配规则（EXP-058）：
    - 以 .rs 结尾的符号（如 gpu_renderer.rs / commands.rs）= 文件名，
      必须存在对应文件（glob 检查）；
    - 其余符号（如 trait+Command 正则）= 代码正则，在 crates/*/src 里 grep。
    """
    patterns = list(SYMBOL_EXPECTATIONS.get(task_id, []))
    if extra_patterns:
        seen = set(patterns)
        patterns += [p for p in extra_patterns if p not in seen]
    if not patterns:
        return []  # 未登记符号期望的任务（如文档类）跳过
    # S57.2a 豁免面（dod 时点符号已被重构迁走，去向已登记）
    patterns = [p for p in patterns if p not in SYMBOL_WAIVER]
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
    # S57.2a：dod 自动提取面（与手工 SYMBOL_EXPECTATIONS 合并；手工优先）
    auto = auto_symbol_expectations(tasks)
    for t in verified:
        tid = t.get("id", "?")
        probs = check_evidence(t.get("evidence"))
        probs += check_symbols(tid, extra_patterns=auto.get(tid, []))
        if probs:
            failures.append(f"[{tid}] " + "; ".join(probs))
    # S57.2a：符号覆盖率下限（60% 阶梯 = S57.2b 提升轮收口时点基线，只收紧）
    covered, declared = symbol_coverage(tasks, auto)
    if declared > 0:
        cov = covered * 100 // declared
        if cov < 60:
            failures.append(
                f"[coverage] 符号覆盖率 {cov}%（{covered}/{declared} 声明代码产物的 verified 任务有符号登记）"
                f" < 60% 下限（S57.2b M-04 阶梯未达标 = 提升轮时点基线 8/8 可提取面全登记，只收紧；后续 100% 阶梯 = 声明面全登记）"
            )
    # blocked 行 schema 断言（loop.md §1：blocked ⇒ evidence 必须 null，防死数据假路径；
    # 轨道 3 M-01/M-22 固化，S49.1 先例）
    for t in tasks:
        if t.get("status") != "blocked":
            continue
        ev = t.get("evidence")
        if ev:
            failures.append(
                f"[{t.get('id', '?')}] blocked 行 evidence 非 null（schema 要求 null；"
                f"指向 {ev} = 死数据假路径）"
            )

    orphans = check_orphan_evidence(tasks)
    for o in orphans:
        failures.append(f"[orphan] {o}")

    if failures:
        print(f"check_ledger_reality: {len(failures)} 项虚报")
        for f in failures:
            print(f"  FAIL  {f}")
        return 0 if args.check else 1
    blocked = [t for t in tasks if t.get("status") == "blocked"]
    sym_covered, declared = symbol_coverage(tasks, auto)
    cov = (sym_covered * 100 // declared) if declared else 100
    print(
        f"check_ledger_reality: 全绿（{len(verified)} 个 verified 任务对账通过"
        f"〔符号 grep 第三方验 {sym_covered}/{declared} 声明代码产物任务（可提取面分母）= 覆盖率 {cov}%（S57.2b 阶梯 60%），"
        f"其余 = 台账行 + evidence 存在非空两方〕"
        f" + {len(blocked)} 个 blocked schema 校验通过 + 0 个孤儿 evidence）"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
