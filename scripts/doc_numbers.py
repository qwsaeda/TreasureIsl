#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""门禁项数 / 台账计数 **实测回写**（`--fix`），消灭「手抄 N 处」的漂移源。

**为什么有这个工具**（`docs/process/15_process_ledger.md` 第 12 条流程）：
门禁项数一改，`parity.md` / `git.md` / `docs/README.md` / `loop.md` / `AGENTS.md` / `style.md` /
`playbook.ps1` / `gate.ps1` 中文横幅 + **`gate.ps1` 英文横幅 `(N items)`** + **`doccheck2.rs` 门禁项数
期望值**（`let expected = Nusize;`）共 **10 处**要手改（S20.0c dod 自认牵动 7 处；S46.2 补上
英文横幅与 doccheck2.rs 期望值两个盲区——此前每改门禁项数仍须人肉补刀 2 处，EXP-172）⇒ 漏一处 = 文档说谎
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

S57.2b 扩面（2026-11-16）：**符号覆盖率回写面**——权威源 = `scripts/check_ledger_reality.py`
实跑（`python3 scripts/check_ledger_reality.py` 输出「符号 grep 第三方验 C/D 声明代码产物任务 =
覆盖率 P%」+ 覆盖率下限常量）；声称点 = docs/process/14_tooling_catalog.md +
docs/check_ledger_reality.md 里「覆盖率 P%（C/D）」形态数值对账 + 覆盖率下限 N% 与 check 脚本常量
同值断言（`SYMBOL_COVERAGE_FLOOR`，阶梯 28→60→100，棘轮只收紧）。
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
# gate.ps1 英文横幅 `=== gate: TreasureIsl (N items) ===`（claim_sites 只认中文「N 项」，此为其盲区）
GATE_BANNER_RE = re.compile(r"\((\d+)\s*items\)")
# doccheck2.rs 门禁项数期望值（`gate_item_actual_count` 断言 gate.ps1 实测 == expected；与横幅同盲区）
DOCCHECK2_EXPECTED_RE = re.compile(r"let expected = (\d+)usize;")
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
# S56.0 扩面（15 号行 36）：排除「时点快照」标注行——带时点注的历史行不回写
_TIMING_NOTE = re.compile(r"时点|历史|S\d+\.?\d*\s*(?:立项|时点|起)|修订前口径|不回改")

# --- S56.0 扩面（15 号行 36 / M-13+M-23 拍板）：项号文字 + 数值声称点对账 --------------------------
# 权威源 = assets/gate_data.json `doccheck2_checks`（47 项数组，1-based 位 = 文档「第 N 项」的机读基准）。
# 声称点形态 = `doccheck2` 第 N 项 / 第 N 项 `check_xxx` / `xxx`（第 N 项）——项号数字（N）与 check id 对账。
DOC2_CLAIM_FILES: list[str] = [
    "AGENTS.md",
    "docs/README.md",
    "docs/loop.md",
    "docs/style.md",
    "docs/parity.md",
    "docs/check_ledger_reality.md",
    "docs/process/04_tooling.md",
    "docs/process/08_render_arch.md",
    "docs/process/09_render_framework.md",
    "docs/process/10_ui_redesign.md",
    "docs/process/11_bevy_migration.md",
    "docs/process/12_bevy_usage.md",
    "docs/process/14_tooling_catalog.md",
    "docs/process/15_process_ledger.md",
    "docs/process/16_code_standards.md",
    "docs/process/28_engine_best_practice.md",
    "docs/rules/scripts-rules.md",
    "docs/rules/ti-engine-rules.md",
    "docs/experience/index.md",
]
# 项号声称：`doccheck2` 第 N 项 / `第 N 项` 后跟 check 短名（xxx 形，含 '-' 的短 id）
DOC2_CLAIM_RE = re.compile(
    r"(?:doccheck2`?\s*第|第)\s*(\d+)\s*项(?:\s*`?([a-z][a-z0-9]*(?:-[a-z0-9]+)+|check_[a-z][a-z0-9]*(?:_[a-z0-9]+)*)`?)?"
)


def _norm_check_id(raw: str | None) -> str | None:
    """归一 check 短名 → gate_data.json id：`check_xxx_yyy` → `xxx-yyy`（Rust fn 名去下划线连字符化）。"""
    if not raw:
        return None
    if raw.startswith("check_"):
        return raw[6:].replace("_", "-")
    return raw
# 台账计数（AGENTS/26 号/release 现状区「N 任务 / M verified」，cross-doc-counts 同源）
LEDGER_LINE_CLAIM_RE = re.compile(r"(\d+)\s*任务\s*/\s*(\d+)\s*verified")


def doc2_position_map() -> dict[str, int]:
    """gate_data.json doccheck2_checks → {check_id: 1-based 位}（权威源；缺 = 空表）。"""
    g = ROOT / "assets" / "gate_data.json"
    if not g.is_file():
        return {}
    import json

    data = json.loads(g.read_text(encoding="utf-8"))
    checks = data.get("doccheck2_checks", [])
    out: dict[str, int] = {}
    for i, c in enumerate(checks, 1):
        cid = c.get("id") if isinstance(c, dict) else c
        if isinstance(cid, str):
            out[cid] = i
    return out


def doc2_claim_sites(rel: str, text: str, posmap: dict[str, int]) -> list[dict[str, object]]:
    """项号文字声称点：(N, check_id) 不匹配 posmap = 漂移。

    护栏：① 带时点标注行（时点/历史/Sxx 立项/修订前口径）不回写；② N 匹配但无 check 短名
    且本行无「doccheck2」字面（= 其它工具「第 N 项」如 gate 项号，非 doccheck2 项号）跳过；
    ③ check 短名在 posmap 无 = 未知 id（可能拼错/新断言未登记）= 报告不回写。
    """
    out: list[dict[str, object]] = []
    for i, line in enumerate(text.splitlines()):
        if _TIMING_NOTE.search(line):
            continue
        for m in DOC2_CLAIM_RE.finditer(line):
            n = int(m.group(1))
            raw_cid = m.group(2)
            if raw_cid is None:
                # 无 check 短名的裸「第 N 项」（如 08_render_arch「gate 第 22 项 doccheck2」行）= 低信息量面：
                # N 可能是 gate 项号（36 位面）也可能是 doccheck2 位（47 位面），无短名无法区分 ⇒ 不回写不报告
                #（本轮轨道 1/3 已人工清零现存裸项号；后续新声称点建议带 check 短名以纳入对账面）。
                continue
            # gate 项名（dod-commands/zero-unknown/studio-tests/check_scene_narrative/check_data_driven 等
            # = 门禁项名而非 doccheck2_checks id）：「gate/门禁 第 N 项 <名>」是 gate 项号面（归 doccheck2
            # gate-items 断言守），非本面；「第 N 项 <gate项名>」紧跟 gate 语境词或本身是 check_* 工具名均排除。
            ctx = line[max(0, m.start() - 30): m.end() + 10]
            if "gate" in ctx.lower() or "门禁" in ctx or raw_cid.startswith("check_"):
                continue
            cid = _norm_check_id(raw_cid)
            if cid is None or cid not in posmap:
                out.append({"line": i + 1, "kind": "unknown-id", "id": raw_cid, "claimed": n, "actual": None, "rel": rel})
                continue
            if posmap[cid] != n:
                out.append({"line": i + 1, "kind": "pos-drift", "id": cid, "claimed": n, "actual": posmap[cid], "rel": rel})
    return out


def ledger_line_sites(rel: str, text: str, tasks: int, verified: int) -> list[dict[str, object]]:
    """「N 任务 / M verified」现状声称点（cross-doc-counts 同源，回写面）。"""
    out: list[dict[str, object]] = []
    for i, line in enumerate(text.splitlines()):
        if _TIMING_NOTE.search(line) or "里程碑" in line:
            continue
        m = LEDGER_LINE_CLAIM_RE.search(line)
        if m and (int(m.group(1)) != tasks or int(m.group(2)) != verified):
            out.append({"line": i + 1, "kind": "ledger-line", "claimed": f"{m.group(1)}/{m.group(2)}", "actual": f"{tasks}/{verified}", "rel": rel})
    return out


def _is_gate_total(line: str, m: re.Match) -> bool:
    """护栏：m 是 数字+「项」匹配 且本行在谈**当前**门禁总数。

    排除：序数（第 N 项）/ 其它总数（N 项命令·图标·残留·各算）/ 阈值（≥·≤ N 项）/
    历史快照（N 项均为时点快照）/ 跨文档名（doccheck2 第 N 项 类「项号文字」声称点，
    归 15 号行 36 S56.0 扩面，doc_numbers 只管 gate 总数 + 台账计数）。
    仅「当前总数」的断言才允许被 --fix 回写。
    """
    # 跨文档名上下文（doccheck2/doccheck 等 + 「第 N 项」项号文字）= S56.0 扩面，勿混入 gate 总数回写
    ctx = line[max(0, m.start() - 30): m.end() + 10]
    if re.search(r"doccheck2?|check_?[a-z_]+\s*第\s*\d+\s*项|第\s*\d+\s*项\s*`?[a-z-]*`?\s*（?gate", ctx):
        return False
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


# S57.2b：符号覆盖率声称点所在文件（14 号工具台账行 + check_ledger_reality.md 判据段）
SYMBOL_CLAIM_FILES: list[str] = [
    "docs/process/14_tooling_catalog.md",
    "docs/check_ledger_reality.md",
]
# 「覆盖率 P%（C/D 声明代码产物任务）」形态（P/C/D = 数值声称点）
SYMBOL_COVERAGE_RE = re.compile(r"覆盖率\s*(\d+)%（(\d+)/(\d+)")  # 前缀允许「（可提取面分母）」任意插入，回写只改数值段
# 「覆盖率下限 N%」/「< N% 下限」形态（N 须与 check 脚本 SYMBOL_COVERAGE_FLOOR 同值）
SYMBOL_FLOOR_RE = re.compile(r"(?:覆盖率下限|< )\s*(\d+)\s*%（?S57\.2a")


def symbol_claim_sites(rel: str, text: str, c: str, d: str, p: str, floor: str) -> list[dict[str, object]]:
    """扫 SYMBOL_CLAIM_FILES 里的符号覆盖率声称点 ↔ check_ledger_reality 实测对账。

    返回漂移声称点（claimed ≠ 实测）；--fix 时回写为实测值（数值面 = 覆盖率% + C/D 计数）。
    覆盖率下限 N% 与 check 脚本常量不同值 = 漂移（报告不回写，须改 check 脚本常量走棘轮流程）。
    """
    sites: list[dict[str, object]] = []
    for i, line in enumerate(text.splitlines(), 1):
        m = SYMBOL_COVERAGE_RE.search(line)
        if m and (m.group(1) != p or m.group(2) != c or m.group(3) != d):
            sites.append({
                "line": i,
                "kind": "coverage-value",
                "claimed": f"{m.group(1)}%（{m.group(2)}/{m.group(3)}）",
            })
        fm = re.search(r"(?:覆盖率下限|< )\s*(\d+)\s*%", line)
        if fm and fm.group(1) != floor and "S57.2a" in line:
            sites.append({
                "line": i,
                "kind": "floor-value",
                "claimed": fm.group(1),
            })
    return sites


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

    # gate.ps1 英文横幅 `(N items)`（S46.2 盲区补齐：claim_sites 只认中文「N 项」，英文横幅曾漏网，
    # 上轮 S46.1 手改；现纳入回写）。
    if GATE.is_file():
        text = GATE.read_text(encoding="utf-8")
        m = GATE_BANNER_RE.search(text)
        if m and m.group(1) != str(g):
            drift += 1
            new_text = GATE_BANNER_RE.sub(lambda mm: f"({g} items)", text, count=1)
            if args.fix and not args.dry_run:
                GATE.write_text(new_text, encoding="utf-8", newline="")
            else:
                print(f"    将改 scripts/gate.ps1 英文横幅: {m.group(1)} → {g}")

    # doccheck2.rs 门禁项数期望值（S46.2 盲区补齐：gate_item_actual_count 断言 gate.ps1 实测 ==
    # expected；此前每改门禁项数须人肉同步 doccheck2.rs，现纳入回写同口径）。
    dc2 = ROOT / "crates" / "ti-tools" / "src" / "bin" / "doccheck2.rs"
    if dc2.is_file():
        text = dc2.read_text(encoding="utf-8")
        m = DOCCHECK2_EXPECTED_RE.search(text)
        if m and m.group(1) != str(g):
            drift += 1
            new_text = DOCCHECK2_EXPECTED_RE.sub(lambda mm: f"let expected = {g}usize;", text, count=1)
            if args.fix and not args.dry_run:
                dc2.write_text(new_text, encoding="utf-8", newline="")
            else:
                print(f"    将改 crates/ti-tools/src/bin/doccheck2.rs 期望值: {m.group(1)} → {g}")

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

    # --- S56.0 扩面（15 号行 36）：doccheck2 项号文字 + 台账行计数声称点 --------------------------
    posmap = doc2_position_map()
    if posmap:
        print(f"  实测：doccheck2 断言 {len(posmap)} 项（gate_data.json 1-based 位表）")
        for rel in DOC2_CLAIM_FILES:
            p = ROOT / rel
            if not p.is_file():
                continue
            text = p.read_text(encoding="utf-8")
            sites = doc2_claim_sites(rel, text, posmap)
            ledger_sites = ledger_line_sites(rel, text, tasks, verified)
            if not sites and not ledger_sites:
                continue
            out_lines = text.splitlines()
            for s in reversed(sites):
                if s["kind"] == "no-id":
                    # 无 check 短名 = 仅报告（回写须人工补 id，防误改 gate 项号面）
                    drift += 1
                    print(f"  [报告] {rel}:{s['line']} doccheck2 项号声称「第 {s['claimed']} 项」无 check 短名 = 无法对账（人工核）")
                    continue
                if s["kind"] == "unknown-id":
                    drift += 1
                    print(f"  [报告] {rel}:{s['line']} check id `{s['id']}` 不在 gate_data.json（新断言未登记或拼错）")
                    continue
                # pos-drift：把「第 N 项」的 N 改回 posmap 实测位（逆序替换；保留原前缀 doccheck2 字面/反引号形态）
                drift += 1
                line = out_lines[s["line"] - 1]
                new_line = re.sub(
                    rf"(?:doccheck2`?\s*第|第)\s*{s['claimed']}\s*项(?:\s*`?{re.escape(str(s['id']))}`?)?",
                    lambda mm, cid=s["id"], actual=s["actual"], prefix="doccheck2": (
                        f"{prefix} 第 {actual} 项 `{cid}`" if "doccheck2" in line[: mm.start()] else f"第 {actual} 项 `{cid}`"
                    ),
                    line,
                    count=1,
                )
                out_lines[s["line"] - 1] = new_line
                if not args.fix:
                    print(f"    将改 {rel}:{s['line']}: 第 {s['claimed']} 项 → 第 {s['actual']} 项（{s['id']}）")
            for s in reversed(ledger_sites):
                drift += 1
                line = out_lines[s["line"] - 1]
                new_line = LEDGER_LINE_CLAIM_RE.sub(
                    lambda mm: f"{tasks} 任务 / {verified} verified", line, count=1
                )
                out_lines[s["line"] - 1] = new_line
                if not args.fix:
                    print(f"    将改 {rel}:{s['line']}: {s['claimed']} → {s['actual']}（台账行计数）")
            if args.fix and not args.dry_run and (sites or ledger_sites):
                p.write_text("\n".join(out_lines) + ("\n" if text.endswith("\n") else ""), encoding="utf-8", newline="")

    # --- S57.2b 扩面（M-04 第二批）：符号覆盖率声称点对账 ------------------------------------
    import subprocess
    r = subprocess.run(
        ["python3", str(ROOT / "scripts" / "check_ledger_reality.py")],
        capture_output=True, text=True,
    )
    m = re.search(r"符号 grep 第三方验 (\d+)/(\d+) 声明代码产物任务（[^）]*）?\s*= 覆盖率 (\d+)%", r.stdout)
    if m:
        c, d, p = m.group(1), m.group(2), m.group(3)
        floor_m = re.search(r"if cov < (\d+):", (ROOT / "scripts" / "check_ledger_reality.py").read_text(encoding="utf-8"))
        floor = floor_m.group(1) if floor_m else "?"
        print(f"  实测：符号覆盖率 {p}%（{c}/{d} 声明代码产物任务；覆盖率下限 {floor}% 棘轮）")
        for rel in SYMBOL_CLAIM_FILES:
            pp = ROOT / rel
            if not pp.is_file():
                continue
            text = pp.read_text(encoding="utf-8")
            sites = symbol_claim_sites(rel, text, c, d, p, floor)
            if not sites:
                continue
            out_lines = text.splitlines()
            for s2 in reversed(sites):
                drift += 1
                line = out_lines[s2["line"] - 1]
                new_line = re.sub(
                    r"覆盖率\s*\d+%（\d+/\d+）",
                    f"覆盖率 {p}%（{c}/{d}）",
                    line, count=1,
                )
                out_lines[s2["line"] - 1] = new_line
                if not args.fix:
                    print(f"    将改 {rel}:{s2['line']}: 覆盖率声称 {s2['claimed']} → 实测 {p}%（{c}/{d}）")
            if args.fix and not args.dry_run and sites:
                pp.write_text("\n".join(out_lines) + ("\n" if text.endswith("\n") else ""), encoding="utf-8", newline="")
    else:
        print("  [报告] check_ledger_reality 实跑输出无符号覆盖率行（扩面未生效？负例护栏面）")

    if drift == 0:
        print("  声称点与实测一致（0 漂移）")
        return 0
    if not args.fix:
        print(f"  漂移 {drift} 处（用 --fix 回写；[报告] 类须人工裁决）", file=sys.stderr)
        return 1
    print(f"  已回写 {drift} 处声称点")
    return 0


if __name__ == "__main__":
    sys.exit(main())
