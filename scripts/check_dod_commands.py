#!/usr/bin/env python3
"""DoD 判据命令载体可达性自检（S46.1；EXP-170 复发防护）。

扫描文档里的判据命令，校验其**载体真实存在且形式可跑**。防 EXP-170 类
「判据命令写在文档里但载体缺失 / 形式歧义 ⇒ 判据从未执行」的复发。

口径：
- `cargo run -p <crate> --bin <bin>` ⇒ `<bin>` 须在该 crate 的 bin 集合内；
- `cargo run -p <crate> -- <args>`（无 `--bin`）⇒ 该 crate 须**恰好 1 个 bin**
  （多 bin 且无 `default-run` ⇒ cargo 报「could not determine which binary」⇒ 不可跑）；
- `cargo test -p <crate> --test <name>` ⇒ `<name>` 须在该 crate 的 test 集合内；
- `python[3] scripts/<x>.py` / `pwsh -File scripts/<x>.ps1` ⇒ 文件须存在。

扫描面 = `docs/**/*.md` + `AGENTS.md`（**排除 `docs/experience/**`**：叙事卷常引用反例）。
退出码：0 = 全过；1 = 存在不可达判据命令；2 = cargo metadata 不可用（前置缺失）。

用法：
    python3 scripts/check_dod_commands.py            # 实跑
    python3 scripts/check_dod_commands.py --check    # 同上（只读别名，幂等）
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

# 命令模式（在文档反引号 / 代码块里出现的判据命令）。
# `cargo run ... -p <crate> ... --bin <bin> ...`
RE_CARGO_RUN_BIN = re.compile(
    r"cargo\s+run\b[^`|\n]*?\s-p\s+(?P<crate>[a-z0-9_-]+)[^`|\n]*?--bin\s+(?P<bin>[a-z0-9_-]+)"
)
# `cargo run ... -p <crate> ... -- <arg>`（无 --bin；<arg> 非选项）
RE_CARGO_RUN_DASH = re.compile(
    r"cargo\s+run\b[^`|\n]*?\s-p\s+(?P<crate>[a-z0-9_-]+)[^`|\n]*?(?<!--bin\s)(?P<dash>--)\s+(?P<arg>[a-z][a-z0-9_-]*)"
)
# `cargo test -p <crate> --test <name>`
RE_CARGO_TEST = re.compile(
    r"cargo\s+test\b[^`|\n]*?\s-p\s+(?P<crate>[a-z0-9_-]+)[^`|\n]*?--test\s+(?P<name>[a-z0-9_-]+)"
)
# `python[3] scripts/<x>.py`
RE_PY_SCRIPT = re.compile(r"\bpython3?\s+scripts/(?P<x>[A-Za-z0-9_./-]+\.py)\b")
# `pwsh -File scripts/<x>.ps1` / `powershell -File scripts/<x>.ps1`
RE_PS_SCRIPT = re.compile(r"\b(?:pwsh|powershell)\s+(?:-ExecutionPolicy\s+\S+\s+-File|-File)\s+scripts/(?P<x>[A-Za-z0-9_./-]+\.ps1)\b")


def cargo_inventory() -> tuple[dict[str, list[str]], dict[str, list[str]]] | None:
    """`cargo metadata` 取每个 crate 的 bin / test 名清单（权威）。"""
    try:
        out = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=REPO,
            capture_output=True,
            text=True,
            timeout=120,
            check=False,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return None
    if out.returncode != 0:
        return None
    try:
        m = json.loads(out.stdout)
    except json.JSONDecodeError:
        return None
    bins: dict[str, list[str]] = {}
    tests: dict[str, list[str]] = {}
    for p in m.get("packages", []):
        name = p["name"]
        bins[name] = [t["name"] for t in p["targets"] if "bin" in t["kind"]]
        tests[name] = [t["name"] for t in p["targets"] if "test" in t["kind"]]
    # workspace 成员名（`-p <crate>` 用包名）。
    return bins, tests


def scan_docs() -> list[Path]:
    """扫描面：AGENTS.md + docs/**/*.md，排除叙事卷 / 历史规格 / 阶段 WBS 计划。

    排除依据：
    - `docs/experience/**`：叙事卷常引用反例命令（EXP 正文）；
    - `docs/impl/**`：历史方案规格（AGENTS 路由表标注「历史阶段」）；
    - `docs/plan-*.md`：阶段 WBS / 历史计划（描述任务，非命令参考；命令多被 superseded）。
    """
    docs = [REPO / "AGENTS.md"]
    for p in (REPO / "docs").rglob("*.md"):
        rel = p.relative_to(REPO / "docs")
        parts = rel.parts
        if "experience" in parts or "impl" in parts:
            continue
        if rel.as_posix().startswith("plan-"):
            continue
        docs.append(p)
    return sorted(set(docs))


# 占位符 / 泛指名（文档里的「`cargo test -p X --test Y`」「`--test diff_<module>`」等模板）⇒ 跳过。
PLACEHOLDER_RE = re.compile(r"<[^>]+>")  # `<module>` / `<m>` / `<id>` 等
GENERIC_NAMES = {"x", "y", "z", "crate", "name", "bin", "module", "id"}


def _is_placeholder(name: str, ctx: str) -> bool:
    """名称是模板占位符（`<…>` 包裹 / 单字母泛指 / 上下文含 `<…>`）⇒ 跳过。"""
    if PLACEHOLDER_RE.search(ctx):
        return True
    return name.lower() in GENERIC_NAMES


def check_file(path: Path, bins: dict[str, list[str]], tests: dict[str, list[str]]) -> list[str]:
    """单文件内的命令可达性检查；返回问题列表。"""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return []
    problems: list[str] = []
    rel = path.relative_to(REPO).as_posix()

    for m in RE_CARGO_RUN_BIN.finditer(text):
        crate, bin_ = m.group("crate"), m.group("bin")
        ctx = text[max(0, m.start() - 8) : m.end() + 24]
        if _is_placeholder(bin_, ctx) or _is_placeholder(crate, ctx):
            continue
        known = bins.get(crate)
        if known is None:
            problems.append(f"{rel}: crate `{crate}` 不在 workspace（`--bin {bin_}`）")
        elif bin_ not in known:
            problems.append(f"{rel}: `{crate}` 无 bin `{bin_}`（实际 {known}）")

    for m in RE_CARGO_RUN_DASH.finditer(text):
        crate, arg = m.group("crate"), m.group("arg")
        ctx = text[max(0, m.start() - 8) : m.end() + 24]
        if _is_placeholder(arg, ctx) or _is_placeholder(crate, ctx):
            continue
        # 已显式 `--bin <name>` 的命令 ⇒ `-- <args>` 是给该 bin 的参数，非歧义（跳过）。
        if "--bin" in m.group(0):
            continue
        known = bins.get(crate)
        if known is None:
            problems.append(f"{rel}: crate `{crate}` 不在 workspace（`-- {arg}` 形式）")
            continue
        if len(known) > 1:
            # 多 bin 且无 default-run ⇒ `cargo run -p X -- args` 报「could not determine which binary」。
            problems.append(
                f"{rel}: `cargo run -p {crate} -- {arg}…` 形式歧义（{crate} 多 bin {known}，"
                f"须显式 `--bin <name> --`；EXP-170 同类）"
            )

    for m in RE_CARGO_TEST.finditer(text):
        crate, name = m.group("crate"), m.group("name")
        ctx = text[max(0, m.start() - 8) : m.end() + 24]
        if _is_placeholder(name, ctx) or _is_placeholder(crate, ctx):
            continue
        known = tests.get(crate)
        if known is None:
            problems.append(f"{rel}: crate `{crate}` 不在 workspace（`--test {name}`）")
        elif name not in known:
            problems.append(f"{rel}: `{crate}` 无 test `{name}`（实际 {known}）")

    for m in RE_PY_SCRIPT.finditer(text):
        x = m.group("x")
        ctx = text[max(0, m.start() - 8) : m.end() + 8]
        stem = x.rsplit("/", 1)[-1].rsplit(".", 1)[0]
        if _is_placeholder(stem, ctx):
            continue
        if not (REPO / "scripts" / x).is_file():
            problems.append(f"{rel}: `scripts/{x}` 不存在")

    for m in RE_PS_SCRIPT.finditer(text):
        x = m.group("x")
        if not (REPO / "scripts" / x).is_file():
            problems.append(f"{rel}: `scripts/{x}` 不存在")

    return problems


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true", help="只读模式（默认即只读，幂等）")
    ap.add_argument("--json", action="store_true", help="JSON 输出")
    args = ap.parse_args()
    inv = cargo_inventory()
    if inv is None:
        print("FAIL: cargo metadata 不可用（前置缺失）", file=sys.stderr)
        return 2
    bins, tests = inv
    problems: list[str] = []
    for doc in scan_docs():
        problems.extend(check_file(doc, bins, tests))
    problems.sort()
    if args.json:
        print(json.dumps({"ok": not problems, "problems": problems}, ensure_ascii=False, indent=2))
    else:
        print("=== check_dod_commands（DoD 判据命令载体可达性自检；EXP-170 复发防护）===")
        if problems:
            print(f"FAIL：{len(problems)} 条不可达判据命令：")
            for p in problems:
                print(f"  ✗ {p}")
        else:
            print("PASS：文档判据命令载体全部可达")
    return 0 if not problems else 1


if __name__ == "__main__":
    sys.exit(main())
