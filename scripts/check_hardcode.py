#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""扫描 crates/**/src/** 的硬编码中文与布局魔法数（数据化红线）。

用法：
    python scripts/check_hardcode.py           # 扫描并报告（发现即失败）
    python scripts/check_hardcode.py --warn    # 只报告不失败（迁移期用）
    python scripts/check_hardcode.py --list    # 按类别汇总（不清单明细）

判据（docs/process/02_conventions.md §1）：
- **玩家可见中文**（场景标题/提示/商品名/建筑名/消息标签）必须进 `assets/data/strings.json`；
- 布局调用（`fill_rect`/`draw_text_row`/...）里的硬编码颜色必须进 `assets/data/scenes.json`。

豁免（带理由，见下方 EXEMPT）：
1. `oracle/` 冻结参照层 —— S8 定位变更后不再改，不追（`docs/plan-s8-refactor.md` §1）；
2. `ti-tools` CLI —— 输出是**命令行终端文本**，不是游戏内 UI，不进 strings.json；
3. 注释与 `#[cfg(test)]` 模块 —— 非生产代码；
4. `settings.rs` 的**配置键名** —— 是 storage key 不是显示文本。

退出码：0 通过；非 0 发现硬编码（gate 判 FAIL）。迁移期可用 --warn。
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

CJK = re.compile(r"[\u4e00-\u9fff]")
STR_LIT = re.compile(r'"([^"\\]|\\.)*"')
LAYOUT_CALL = re.compile(r"\b(fill_rect|draw_text_row|draw_text|blit|fill)\s*\(")
COLOR_HEX = re.compile(r"\b0x[0-9A-Fa-f]{4}\b")

SKIP_DIR_PARTS = {"tests", "target"}

# ── 豁免规则（每条必须有理由，改动时先看这里）──────────────────────────────
EXEMPT: list[tuple[str, str, str]] = [
    # (路径子串, 类别, 理由)
    ("oracle/", "any", "冻结参照层：S8 后不再改，不追硬编码"),
    ("ti-tools/", "cjk", "CLI 工具：终端输出文本，非游戏内 UI"),
    ("treasure_headless.rs", "cjk", "无头冒烟 CLI 打印（P3-3 机器判定），非游戏内 UI"),
    ("settings.rs", "cjk", "storage 配置键名，非显示文本"),
    ("mod_pcm.rs", "cjk", "PCM 格式诊断信息，仅 debug 打印"),
    ("audio.rs", "cjk", "WAV 解析错误描述（Error::Asset 原因），非玩家可见 UI 文案"),
    ("sprite_atlas.rs", "cjk", "贴图加载诊断 / 断言消息，非玩家可见 UI 文案"),
    ("terrain_atlas.rs", "cjk", "地形 pak 加载诊断 / 断言消息，非玩家可见 UI 文案"),
]


def exempt_for(rel: str, cat: str) -> str | None:
    """返回豁免理由；未豁免返回 None。"""
    norm = rel.replace("\\", "/")
    for pat, c, why in EXEMPT:
        if pat in norm and (c == "any" or c == cat):
            return why
    return None


def strip_comment(line: str) -> str:
    idx = line.find("//")
    return line if idx == -1 else line[:idx]


def in_test_region(lines: list[str], idx0: int) -> bool:
    for j in range(idx0, -1, -1):
        if "#[cfg(test)]" in lines[j]:
            return True
    return False


def scan_file(path: Path) -> list[tuple[str, str, int, str]]:
    """返回 (rel, category, lineno, detail) 列表。"""
    hits: list[tuple[str, str, int, str]] = []
    rel = path.as_posix()
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return hits
    lines = text.splitlines()
    for i, raw in enumerate(lines):
        if in_test_region(lines, i):
            continue
        code = strip_comment(raw)
        if not code.strip():
            continue
        for m in STR_LIT.finditer(code):
            lit = m.group(1)
            if lit and CJK.search(lit):
                why = exempt_for(rel, "cjk")
                if why is None:
                    hits.append((rel, "cjk", i + 1, lit[:24]))
        if LAYOUT_CALL.search(code) and COLOR_HEX.search(code):
            why = exempt_for(rel, "color")
            if why is None:
                hits.append((rel, "color", i + 1, "layout call with hardcoded color"))
    return hits


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=Path("."))
    ap.add_argument("--warn", action="store_true", help="report only, do not fail")
    ap.add_argument("--list", action="store_true", help="summarize by file")
    args = ap.parse_args()

    crates = args.root / "crates"
    hits: list[tuple[str, str, int, str]] = []
    files = 0
    for rs in sorted(crates.rglob("*.rs")):
        if any(p in SKIP_DIR_PARTS for p in rs.parts):
            continue
        files += 1
        hits.extend(scan_file(rs))

    if args.list:
        from collections import Counter

        c = Counter(h[0] for h in hits)
        for f, n in c.most_common():
            k = sum(1 for h in hits if h[0] == f and h[1] == "cjk")
            print("  %3d hits (cjk=%d color=%d)  %s" % (n, k, n - k, f))
        print("check_hardcode: %d hit(s) in %d file(s) scanned" % (len(hits), files))
        return 0

    for rel, cat, lineno, detail in hits:
        print("  %s:%d [%s] %s" % (rel, lineno, cat, detail))
    if hits:
        cjk = sum(1 for h in hits if h[1] == "cjk")
        print("check_hardcode: %d hit(s) (cjk=%d color=%d) in %d file(s)" % (len(hits), cjk, len(hits) - cjk, files))
        if args.warn:
            print("(--warn mode, not failing)")
            return 0
        return 1
    print("check_hardcode: OK (%d file(s), 0 hits)" % files)
    return 0


if __name__ == "__main__":
    sys.exit(main())