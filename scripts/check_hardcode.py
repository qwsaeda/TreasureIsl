#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""check_hardcode.py — S8.2 硬编码扫描（gate 第 12 项，严格模式：命中即 FAIL）。

背景（docs/process/02_conventions.md 数据化红线）：
「表现层不含数值」：玩家可见中文（菜单/按钮/标签/状态行）必须来自
`assets/data/strings.json`（i18n sget/sgetf，按 key）；布局色值必须来自
`menu_shell` 的 COLOR_* 常量。代码里的中文字面量/硬编码色值 = 漂移源。

豁免（带理由）：
1. `oracle/`（`ti-core/src/oracle` 冻结参照层；重复 crate `legacy/oracle-rust`已于 S15.8 删除）— S8/S11
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
# R24 FIND-002 / R25 F-03：诊断宏（终端输出，非玩家 UI）——文件级 CJK 豁免仅对
# 字面量确实处于诊断宏**未闭合括号**内时成立（非整行 search：同行混合/跨行都判准）。
MACRO_NAMES = {"eprintln", "println", "eprint", "print", "dbg", "debug", "info", "warn", "error", "trace", "write", "writeln", "format_args"}
# 诊断宏参数子树内的嵌套包装（继续向外找外层诊断宏，而非当作玩家可见调用拒绝）。
NESTED_IN_DIAGNOSTIC = {"format", "format_args", "Some", "Ok", "Err", "vec", "AsRef", "into", "as_str"}
# R26 F-04/F-05：用真括号栈判定替代 R25 的「最近调用名 + 固定 4 行」启发式。
# 向前扫描源码（从字面量起点回溯到文件开头），逆序配对括号：
#   遇 `)`/`}` → 期望的配对数 +1（表示前面有未闭合开括号需跨过）；
#   遇 `(`/`{` → 若期望配对数 == 0，则它是字面量的**最近未闭合包裹括号**，
#                取它前面的标识符判断是否为诊断宏；否则期望配对数 -1。
# 这样：`eprintln!("诊断", format!("x"))` 中 format!( 被 `)` 配对抵消 → 落到 eprintln!（真豁免）；
#       `eprintln!("a"); draw_text("玩家可见")` 中 draw_text( 直接是最近未闭合 → 不豁免（真拒绝）；
#       跨行 eprintln!(\n  "诊断") 也正确（无固定窗口）。
OPEN_TO_CLOSE = {"(": ")", "{": "}"}


def in_diagnostic_macro(lines: list[str], lineno: int, col: int) -> bool:
    """R26 F-04/F-05：判定字面量是否处于诊断宏的未闭合括号参数子树内（真括号栈）。"""
    need_close = 0  # 向前遇到 `)`/`}` 时累加（需由更早的 `(`/`{` 配对）
    for ln in range(lineno, -1, -1):
        text = lines[ln]
        end = col if ln == lineno else len(text)
        for idx in range(end - 1, -1, -1):
            ch = text[idx]
            if ch in (")", "}"):
                need_close += 1
            elif ch in ("(", "{"):
                if need_close > 0:
                    need_close -= 1
                    continue
                # 找开括号前的标识符（允许 `!`/空白），即包裹调用名
                j = idx - 1
                while j >= 0 and text[j].isspace():
                    j -= 1
                bang = False
                if j >= 0 and text[j] == "!":
                    bang = True
                    j -= 1
                    while j >= 0 and text[j].isspace():
                        j -= 1
                k = j
                while k >= 0 and (text[k].isalnum() or text[k] == "_" or text[k] == ":"):
                    k -= 1
                name = text[k + 1:j + 1] if j > k else ""
                if name in MACRO_NAMES or (bang and name.endswith("log")):
                    return True
                # R26 F-05：诊断宏的参数子树内嵌套 format!/Some/vec!/& 等 → 继续向外找外层
                # 包裹调用（它们是诊断串的一部分，不是独立玩家可见渲染）。
                if name in NESTED_IN_DIAGNOSTIC:
                    continue  # 跳过嵌套包装的 `(`，继续向外找诊断宏的包裹括号
                return False  # draw_text/draw_hint_text 等玩家可见调用 → 不豁免
    return False
# 飞轮 R1 ⑤-05 修复：旧式 `"([^"\\]|\\.)*"` 捕获组只保留 `*` 量词内最后一次迭代
# （= 字面量末字符），CJK 通道对「以标点/ASCII 结尾」的中文字面量失明（如
# `…不 fallback）` 只捕获到 `）`）。改用非捕获外层 + 原子式内层，group(1) = 完整字面量内容。
STR_LIT = re.compile(r'"((?:[^"\\]|\\.)*)"')
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
    # S32（ti-console/ti-devmeta）：开发管理平台 = CLI 诊断/锁/HTTP 错误文本，非玩家 UI（口径同 ti-tools/）
    ("ti-console/", "cjk", "S32 dev console: lock/HTTP/binding diagnostic prints, not in-game UI"),
    ("ti-devmeta/", "cjk", "S32 dev-meta shared layer: HTTP/ledger diagnostic prints, not in-game UI"),
    ("settings.rs", "cjk", "storage config key names, not display text"),
    ("mod_pcm.rs", "cjk", "PCM format diagnostics, debug prints only"),
    ("audio.rs", "cjk", "WAV parse error descriptions, not player-visible UI"),
    ("audio_prep.rs", "cjk", "audio asset diagnostics / test messages, not player-visible UI"),
    ("sprite_atlas.rs", "cjk", "atlas loading diagnostics / assertion messages, not player UI"),
    ("terrain_atlas.rs", "cjk", "terrain pak loading diagnostics / assertion messages, not player UI"),
    ("game_app/", "cjk", "runtime eprintln diagnostics after S20.0b split (board/host/mod/assets), not player UI (flywheel R1 ⑤-06: old `game_app.rs` line no longer matches post-split paths; S28.6 玩家可见 toast 已走 i18n `i18n_fmt`，玩家可见 CJK = 0)"),
    ("scenes/main_map.rs", "cjk", "[stats]-gated TradeBuy/TradeSell/Build hard-error diagnostics, not player UI"),
    ("scenes/mod.rs", "cjk", "scene_transition! macro 3rd arg is doc-only ($doc -> #[doc] attribute), not player-visible text"),
    ("bevy_app.rs", "cjk", "Bevy spike shell window title, dev tool, not in-game UI (flywheel R1 ⑤-04 pending human ruling: may move to strings.json later)"),
    ("treasure_bevy.rs", "cjk", "Bevy delivery shell offscreen/selftest/shot diagnostics + window title, terminal output, not in-game UI"),
    ("android_entry.rs", "cjk", "S18.8d Android JNI 入口诊断文本（android-activity 壳 crate include!；native 目标 cfg-gated 不编译），not in-game UI"),
    ("android_shell.rs", "cjk", "Android shell [hd] asset-registration diagnostics, not in-game UI"),
    ("sprite_step.rs", "cjk", "sprite-step debug bin usage/error/sprite-dump terminal output, not in-game UI"),
    ("data/mod.rs", "cjk", "balance data validation LoadError wording (crash-diagnostic channel), not display text"),
    ("data/map_pack.rs", "cjk", "S31.2 map-pack runtime load diagnostics (manifest reject / section fallback eprintln), not display text"),
    ("data/table_source.rs", "cjk", "S31.2 table-injection late-install warning (crash-diagnostic channel), not display text"),
    ("hd_png.rs", "cjk", "PNG HD asset loading diagnostics / error wording, not in-game UI"),
    ("ogg_encode.rs", "cjk", "OGG/Opus encode error diagnostics, not in-game UI"),
    ("ids.rs", "cjk", "ti-formats ControlKind parse error wording, not in-game UI"),
    ("element_dump.rs", "cjk", "diagnostic bin: §4 layout zone / bleed terminal output, not in-game UI"),
    ("forward_nav.rs", "cjk", "S14 forward navigation diagnostic prints, not in-game UI text"),
    ("render_png.rs", "cjk", "S14 forward PNG export bin terminal output, not in-game UI text"),
    ("territory_v2.rs", "cjk", "S52.3 #[deprecated(note=...)] 编译器诊断文本（v2 死代码 3 函数留作语义参照；note 走 clippy/rustc 警告通道，非玩家可见 UI），not in-game UI"),
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
                # R24 FIND-002 / R25 F-03：宽豁免文件（main_map.rs + game_app/）的 CJK 豁免收窄为
                # 「字面量处于诊断宏未闭合括号内」；玩家可见 draw_* 文本不许借豁免隐身。
                if why is not None and ("scenes/main_map.rs" in rel or "game_app/" in rel) \
                        and not in_diagnostic_macro(lines, i, m.start()):
                    why = None
                if why is None:
                    hits.append((rel, "cjk", i + 1, lit[:24]))
        if LAYOUT_CALL.search(code) and COLOR_HEX.search(code):
            why = exempt_for(rel, "color")
            if why is None:
                hits.append((rel, "color", i + 1, "layout call with hardcoded color"))
    return hits


def selftest() -> int:
    """飞轮 R1 ⑤-05 回归防线：断言 STR_LIT 捕获组 = 完整字面量内容（非末字符）。

    旧式 `"([^"\\]|\\.)*"` 的捕获组在 `*` 量词内只保留最后一次迭代，
    导致「以 CJK 标点/ASCII 结尾」的中文字面量漏检（gate 12 CJK 通道失明）。
    任一断言失败 = 回归，退出码非 0。
    """
    failures: list[str] = []

    def expect_cjk_found(src: str, label: str) -> None:
        found = [m.group(1) for m in STR_LIT.finditer(src) if m.group(1)]
        hits = [lit for lit in found if CJK.search(lit)]
        if not hits:
            failures.append("CJK 字面量漏检: %s (captured=%r)" % (label, found))

    def expect_full_capture(src: str, want: str, label: str) -> None:
        for m in STR_LIT.finditer(src):
            if want in (m.group(1) or "") and m.group(1) != want:
                failures.append("捕获组不完整: %s (captured=%r want=%r)" % (label, m.group(1), want))

    # 1) 关键回归：CJK 字面量以全角括号 `）` 结尾（旧式只捕获 `）` 本身 → 漏检）
    expect_cjk_found('eprintln!("[stats] xx（硬错误，不 fallback）")', "ends-with-CJK-punct")
    expect_full_capture(
        'eprintln!("[stats] xx（硬错误，不 fallback）")',
        "[stats] xx（硬错误，不 fallback）",
        "full-capture-punct-end",
    )
    # 2) 以汉字结尾（旧式末字符恰好命中，双式都必须命中）
    expect_cjk_found('let s = "加载失败";', "ends-with-CJK-char")
    # 3) 以 ASCII 结尾的 CJK 串（旧式也漏检）
    expect_cjk_found('let t = "地图1号";', "ends-with-ascii")
    # 4) 转义引号内的 CJK（`\"` 分支）
    expect_cjk_found('let u = "含\\"引号\\"的中文";', "escaped-quote")
    # 5) 纯 ASCII 串不得误报 CJK
    for m in STR_LIT.finditer('let v = "plain ascii";'):
        if m.group(1) and CJK.search(m.group(1)):
            failures.append("ASCII 串误报 CJK")
    # 6) R26 F-04/F-05：诊断宏括号栈判定边界（7 合法 fixture）
    def expect_diag(lines: list[str], ln: int, lit: str, want: bool, label: str) -> None:
        col = lines[ln].find(lit) + 1
        got = in_diagnostic_macro(lines, ln, col)
        if got != want:
            failures.append(f"诊断宏括号栈判定错误: {label} got={got} want={want}")

    expect_diag(['eprintln!("诊断"); draw_text("玩家可见");'], 0, "玩家可见", False, "同行混合玩家可见")
    expect_diag(['eprintln!(', '    "诊断"', ');'], 1, "诊断", True, "跨行 eprintln")
    expect_diag(['eprintln!("{}", format!("诊断"));'], 0, "诊断", True, "嵌套 format")
    expect_diag(['eprintln!("{:?}", Some("诊断"));'], 0, "诊断", True, "Some 包裹")
    expect_diag(['eprintln!("{:?}", vec!["诊断"]);'], 0, "诊断", True, "vec! 包裹")
    expect_diag(['eprintln!("{}", &"诊断");'], 0, "诊断", True, "& 引用")
    expect_diag(['info! { "诊断" }'], 0, "诊断", True, "花括号宏")

    if failures:
        for f in failures:
            print("  selftest FAIL: %s" % f)
        print("check_hardcode: selftest FAILED (%d)" % len(failures))
        return 1
    print("check_hardcode: selftest OK (STR_LIT full-capture + CJK + 诊断宏括号栈 7 fixtures)")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--warn", action="store_true", help="report but do not fail")
    ap.add_argument("--root", default=".", help="repo root")
    ap.add_argument("--selftest", action="store_true",
                    help="flywheel R1 regression: assert STR_LIT captures the full literal")
    ap.add_argument("--check", action="store_true",
                    help="门禁模式（S21.8 工具契约只读开关；默认严格行为即只读扫描，命中非 0 退出）")
    ap.add_argument("--list", action="store_true",
                    help="飞轮 R3 B-02：按文件分类汇总命中（只读，与 14 号/04 号台账声称对齐；命中即按文件分组打印，退出码同严格模式）")
    args = ap.parse_args()

    if args.selftest:
        return selftest()

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

    if args.list:
        # 飞轮 R3 B-02：按文件分类汇总（只读模式；命中按文件分组 + 逐条，退出码同严格模式）
        for f in sorted(by_file):
            hits = by_file[f]
            k = sum(1 for h in hits if h[1] == "cjk")
            print("  %3d hits (cjk=%d color=%d)  %s" % (len(hits), k, len(hits) - k, f))
            for rel, cat, lineno, detail in hits:
                print("    %s:%d [%s] %s" % (rel, lineno, cat, detail))
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
