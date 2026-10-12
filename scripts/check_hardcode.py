#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""check_hardcode.py — S8.2 硬编码扫描（gate 第 12 项，严格模式：命中即 FAIL）。

背景（docs/process/02_conventions.md 数据化红线）：
「表现层不含数值」：玩家可见中文（菜单/按钮/标签/状态行）必须来自
`assets/data/strings.json`（i18n sget/sgetf，按 key）；布局色值必须来自
`menu_shell` 的 COLOR_* 常量。代码里的中文字面量/硬编码色值 = 漂移源。

豁免（带理由）：
1. `oracle/`（含 `ti-core/src/oracle` 桥接层与 `legacy/oracle-rust`）— S8/S11
   定位变更后为冻结参照层，不再改，不追（docs/plan-s8-refactor.md 1）；
2. `ti-tools` CLI — 输出是命令行终端文本，不是游戏内 UI，不进 strings.json；
3. 注释行与 `#[cfg(test)]` 模块 — 非生产代码；
4. `settings.rs` 配置键名 — storage key 不是显示文本。

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

SKIP_DIR_PARTS = {"tests", "target", "legacy"}

# -- 豁免规则（每条必须有理由，改动时先看这里）--
EXEMPT: list[tuple[str, str, str]] = [
    # (path-substring, category, reason)
    ("oracle/", "any", "frozen reference layer: post-S8/S11 archived, do not chase"),
    ("legacy/", "any", "archived artifacts: not in the active build"),
    ("ti-tools/", "cjk", "CLI tool: terminal output text, not in-game UI"),
    ("treasure_headless.rs", "cjk", "headless smoke CLI prints (P3-3 machine check), not in-game UI"),
    ("treasure_android.rs", "cjk", "Android shell diagnostic prints, not in-game UI"),
    ("settings.rs", "cjk", "storage config key names, not display text"),
    ("mod_pcm.rs", "cjk", "PCM format diagnostics, debug prints only"),
    ("audio.rs", "cjk", "WAV parse error descriptions, not player-visible UI"),
    ("audio_prep.rs", "cjk", "audio asset diagnostics / test messages, not player-visible UI"),
    ("sprite_atlas.rs", "cjk", "atlas loading diagnostics / assertion messages, not player UI"),
    ("terrain_atlas.rs", "cjk", "terrain pak loading diagnostics / assertion messages, not player UI"),
    ("game_app.rs", "cjk", "runtime error eprintln diagnostics (balance load failures), not player UI"),
]


def exempt_for(rel: str, cat: str) -> str | None:
    """Return the exemption reason, or None if not exempt."""
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
    """Return a list of (rel, category, lineno, detail) hits."""
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
    ap = argparse.ArgumentParser()
    ap.add_argument("--warn", action="store_true", help="report but do not fail")
    ap.add_argument("--root", default=".", help="repo root")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    crates = root / "crates"
    files = 0
    all_hits: list[tuple[str, str, int, str]] = []
    for rs in sorted(crates.rglob("*.rs")):
        if not rs.is_file():
            continue
        rel = rs.relative_to(root).as_posix()
        parts = Path(rel).parts
        if any(p in SKIP_DIR_PARTS for p in parts):
            continue
        if "src" not in parts:
            continue
        files += 1
        all_hits.extend(scan_file(root / rel))

    by_file: dict[str, list[tuple[str, str, int, str]]] = {}
    for h in all_hits:
        by_file.setdefault(h[0], []).append(h)

    if args.warn:
        for f in sorted(by_file):
            hits = by_file[f]
            n = len(hits)
            k = sum(1 for h in hits if h[1] == "cjk")
            print("  %3d hits (cjk=%d color=%d)  %s" % (n, k, n - k, f))
        print("check_hardcode: %d hit(s) in %d file(s) scanned" % (len(all_hits), files))
        return 0

    for rel, cat, lineno, detail in all_hits:
        print("  %s:%d [%s] %s" % (rel, lineno, cat, detail))

    if all_hits:
        cjk = sum(1 for h in all_hits if h[1] == "cjk")
        print("check_hardcode: %d hit(s) (cjk=%d color=%d) in %d file(s)" % (len(all_hits), cjk, len(all_hits) - cjk, files))
        if args.warn:
            print("(--warn mode, not failing)")
            return 0
        return 1

    print("check_hardcode: OK (%d file(s), 0 hits)" % files)
    return 0


if __name__ == "__main__":
    sys.exit(main())
