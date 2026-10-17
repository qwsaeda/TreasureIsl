#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""validate_data.py — 数据表兜底校验（gate 第 10a 项，统一入口）。

自 S11 归档起取代 `legacy/scripts/validate_data.py` 成为 gate 唯一数据表校验入口：
原 balance/strings/scenes/controls 基础校验（FIND-201 合并）+ 场景标量兜底
（V1/V2，FIND-106 配套）全部并入本脚本，单一脚本挂 gate。

背景：`crates/ti-core/src/data/layout.rs` 的 `extra_scalar` 解析器对
「非 i64 标量」（浮点/字符串/嵌套对象）只 `eprintln` 不致命（layout.rs 保持只读
缓存语义，解析失败不 fallback），本脚本做**数据侧兜底**。

检查项（全部机器可判，只读、幂等）：
  B  基础校验（原 legacy/scripts/validate_data.py，FIND-201 合并）：
     必填文件存在且是合法 JSON；balance 数值域（概率 ∈ [0,1]、价格/HP/造价 > 0、
     权重 > 0、上限 > 下限）；strings 覆盖 balance 中每个 name_key（无孤儿引用）；
     scenes 覆盖全部游戏场景；controls.json（若存在）与 legacy/ref/controls.json
     记录数一致。
   V1 `main_map.grid` 段必须含 `x0`/`y0`/`tile_w`/`tile_h` 四键，且均为 >0 的整数字面量
      （`chrome::iso_cell`/`iso_tile` 等距投影依赖此四键，S15.3 1080 重锚后 `step` 改 `tile_w`/`tile_h`）。
  V2 全场景 `extra` 中的非 Rect 标量键（int/float 值）必须是 >0 的整数字面量
     （`extra_scalar` 抽取判据一致，防静默丢）。

退出码：0 = PASS；1 = FAIL（列出违规项）。

用法：
  python scripts/validate_data.py            # 校验（全量）
  python scripts/validate_data.py --check    # 同上（参数与 legacy 脚本保持一致）
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

DATA_FILES = ["balance.json", "strings.json", "scenes.json", "theme.json"]

GAME_SCENES = ["index", "select_role", "select_map", "main_map", "vs", "message", "saveload"]

# V1：main_map 占位网格渲染四键（`chrome::iso_cell`/`iso_tile` 读 `grid.x0`/`grid.y0`/`grid.tile_w`/`grid.tile_h`，
# S15.3 1080 重锚后 `step` 已改 `tile_w`/`tile_h`；无配置 = 0，调用方据此跳过绘制）
V1_REQUIRED = ("x0", "y0", "tile_w", "tile_h")


def load(p: Path, problems: list[str]) -> dict | None:
    if not p.is_file():
        problems.append(f"缺少文件 {p}")
        return None
    try:
        with p.open(encoding="utf-8") as fh:
            return json.load(fh)
    except json.JSONDecodeError as e:
        problems.append(f"{p}: JSON 语法错误 {e}")
        return None


def check_balance(b: dict, problems: list[str]) -> set[str]:
    """返回 balance 中所有被引用的 name_key 集合。"""
    keys: set[str] = set()

    ec = b.get("economy", {})
    if ec.get("gold_max", 0) <= 0:
        problems.append("economy.gold_max 必须 > 0")
    if not (0.0 <= ec.get("bank_rate", -1) <= 1.0):
        problems.append(f"economy.bank_rate={ec.get('bank_rate')} 越界 [0,1]")
    if ec.get("initial_gold", 0) < 0:
        problems.append("economy.initial_gold 不得为负")

    cc = b.get("combat", {})
    if cc.get("max_rounds", 0) <= 0:
        problems.append("combat.max_rounds 必须 > 0")
    pl = cc.get("player", {})
    if pl.get("hp", 0) <= 0:
        problems.append("combat.player.hp 必须 > 0")
    if pl.get("atk", 0) <= 0:
        problems.append("combat.player.atk 必须 > 0")
    dmg = cc.get("damage", {})
    if dmg.get("flat", 0) <= 0:
        problems.append("combat.damage.flat 必须 > 0（否则战斗不收敛）")
    if dmg.get("atk_coef", 0) <= 0:
        problems.append("combat.damage.atk_coef 必须 > 0")
    tiers = cc.get("boss_tiers", [])
    if not tiers:
        problems.append("combat.boss_tiers 不得为空")
    else:
        expect = list(range(len(tiers)))
        got = [t.get("tier", -1) for t in tiers]
        if got != expect:
            problems.append(f"combat.boss_tiers 档位应连续 0..{len(tiers) - 1}，实为 {got}")
        for t in tiers:
            if t.get("hp", 0) <= 0:
                problems.append(f"boss_tier {t.get('tier')} hp 必须 > 0")
            if t.get("atk", 0) <= 0:
                problems.append(f"boss_tier {t.get('tier')} atk 必须 > 0")

    tc = b.get("trade", {})
    if tc.get("markup", 0) <= 0:
        problems.append("trade.markup 必须 > 0")
    for g in tc.get("goods", []):
        keys.add(g.get("name_key", ""))
        if g.get("buy", 0) <= 0:
            problems.append(f"goods {g.get('id')} buy 必须 > 0")
        if g.get("qty_max", 0) <= 0:
            problems.append(f"goods {g.get('id')} qty_max 必须 > 0")

    bd = b.get("build", {})
    if bd.get("max_level", 0) <= 0:
        problems.append("build.max_level 必须 > 0")
    ids = [x.get("id") for x in bd.get("buildings", [])]
    if len(ids) != len(set(ids)):
        problems.append(f"build.buildings id 重复：{ids}")
    for x in bd.get("buildings", []):
        keys.add(x.get("name_key", ""))
        if x.get("base_cost", 0) <= 0:
            problems.append(f"building {x.get('id')} base_cost 必须 > 0")
        if x.get("gold_max", 0) <= 0:
            problems.append(f"building {x.get('id')} gold_max 必须 > 0")

    cd = b.get("cards", {})
    if not (0.0 <= cd.get("event_chance", -1) <= 1.0):
        problems.append(f"cards.event_chance={cd.get('event_chance')} 越界 [0,1]")
    if cd.get("hand_size", 0) <= 0:
        problems.append("cards.hand_size 必须 > 0")
    for c in cd.get("deck", []):
        keys.add(c.get("name_key", ""))
        if c.get("weight", 0) <= 0:
            problems.append(f"card {c.get('id')} weight 必须 > 0")
        eff = c.get("effect", {})
        if eff.get("kind") not in {"damage", "heal", "gold", "move", "unknown"}:
            problems.append(f"card {c.get('id')} effect.kind 非法：{eff.get('kind')}")

    sv = b.get("save", {})
    if sv.get("slot_count", 0) <= 0:
        problems.append("save.slot_count 必须 > 0")
    if sv.get("magic", 0) <= 0:
        problems.append("save.magic 必须 > 0")
    if sv.get("v2_size", 0) <= 0:
        problems.append("save.v2_size 必须 > 0")

    ui = b.get("ui", {})
    for k in ("logical_width", "logical_height", "glyph_advance", "max_back_steps"):
        if ui.get(k, 0) <= 0:
            problems.append(f"ui.{k} 必须 > 0")

    sel = b.get("selection", {})
    if not sel.get("characters"):
        problems.append("selection.characters 不得为空")
    if not sel.get("maps"):
        problems.append("selection.maps 不得为空")
    for m in sel.get("maps", []):
        if m.get("grid_w", 0) <= 0:
            problems.append(f"selection.maps {m.get('id')} grid_w 必须 > 0")
        if m.get("grid_h", 0) <= 0:
            problems.append(f"selection.maps {m.get('id')} grid_h 必须 > 0")
        if not m.get("landmarks"):
            problems.append(f"selection.maps {m.get('id')} landmarks 不得为空")
        if not m.get("player_start"):
            problems.append(f"selection.maps {m.get('id')} player_start 不得为空")

    return keys


def check_strings(s: dict, referenced: set[str], problems: list[str]) -> None:
    for k in sorted(referenced):
        if not k:
            problems.append("balance 中存在空 name_key")
        elif k not in s:
            problems.append(f"strings 缺少被引用的 key `{k}`")
    orphan_prefixes = ("scene.", "goods.", "build.", "cards.", "msg.")
    used = {k for k in s if any(k.startswith(p) for p in orphan_prefixes)}
    for k in sorted(used - referenced):
        # 允许场景标题/提示这类纯 UI key（不在 balance 里引用）
        if k.startswith("scene.") or k.startswith("msg."):
            continue
        problems.append(f"strings key `{k}` 未被 balance 引用（孤儿）")


def check_scenes(sc: dict, problems: list[str]) -> None:
    for sid in GAME_SCENES:
        if sid not in sc:
            problems.append(f"scenes 缺少场景 `{sid}`")
    for sid, layout in sc.items():
        if sid == "schema_version":
            continue
        # S15.6 裁决①：`bg` 键已删（运行时零消费——画布底色 = chrome COLOR_PARCH_DARK
        # 深底；旧 65440 浅米白与实绘相反，QA029）。解析容错：缺省 0，无消费方。
        lst = layout.get("list")
        if lst is not None:
            if lst.get("row_h", 0) <= 0:
                problems.append(f"scene `{sid}` list.row_h 必须 > 0")
            if lst.get("item_h", 0) <= 0:
                problems.append(f"scene `{sid}` list.item_h 必须 > 0")
            if lst.get("item_h", 0) > lst.get("row_h", 0):
                problems.append(f"scene `{sid}` list.item_h 不得超过 row_h")


def check_controls(root: Path, problems: list[str]) -> None:
    cpath = root / "assets/data/controls.json"
    if not cpath.is_file():
        return
    c = load(cpath, problems)
    if c is None:
        return
    # S11 归档后 ref/ 源在 legacy/ref/controls.json（旧路径 ref/ 已移入 legacy/）
    src = root / "legacy/ref/controls.json"
    if src.is_file():
        with src.open(encoding="utf-8") as fh:
            orig = json.load(fh)
        if c.get("record_total") != orig.get("record_total"):
            problems.append(
                f"controls.json record_total={c.get('record_total')} != legacy/ref/controls.json {orig.get('record_total')}"
            )


# ---- V1/V2：场景标量兜底（FIND-106 配套）----

def is_positive_int(v: object) -> bool:
    """整数字面量且 >0（bool 在 Python 是 int 子类，须排除）。"""
    return isinstance(v, int) and not isinstance(v, bool) and v > 0


def check_v1(main_map: dict, problems: list[str]) -> None:
    """V1：main_map.grid 四键存在且为正整数字面量。"""
    grid = main_map.get("grid")
    if not isinstance(grid, dict):
        problems.append("V1: main_map.grid 段缺失或形状非对象")
        return
    for key in V1_REQUIRED:
        if key not in grid:
            problems.append(f"V1: main_map.grid 缺键 `{key}`")
        elif not is_positive_int(grid[key]):
            problems.append(f"V1: main_map.grid.{key} = {grid[key]!r} 非正整数字面量")


THEME_PALETTE_KEYS = (
    "parch_dark", "parch_light", "wood", "gold", "gold_glow", "gold_bright",
    "menu_bg", "menu_border", "hilite", "bevel_hi", "bevel_shadow",
    "text_on_parch", "text_on_wood", "danger", "player", "landmark",
    "bg", "title_bar", "selected", "secondary", "black",
)
THEME_FONT_KEYS = (
    "font_size", "title_font_size", "hint_font_size",
    "bitmap_advance", "bitmap_advance_title", "bitmap_advance_hint",
)


def _as_hex16(v: object) -> int | None:
    """主题色值接受 "0x10A3" / "10A3" 字符串或裸数字，统一成 int（非法 → None）。"""
    if isinstance(v, bool):
        return None
    if isinstance(v, int):
        return v
    if isinstance(v, str):
        t = v.strip()
        t = t[2:] if t[:2].lower() == "0x" else t
        try:
            return int(t, 16)
        except ValueError:
            return None
    return None


def check_theme(theme: dict, problems: list[str]) -> None:
    """theme.json：palette 21 键全在 + 0..=0xFFFF；fonts 6 键全在 + > 0。

    S15.7 数据分离：色板与字号唯一事实源 = assets/data/theme.json，
    Rust 侧只留兜底值。键缺失 = 解析时静默回落 default = 隐藏漂移，故此处守键存在性。
    """
    palette = theme.get("palette")
    if not isinstance(palette, dict):
        problems.append("theme.json 缺 palette 组（色板唯一事实源）")
        palette = {}
    for key in THEME_PALETTE_KEYS:
        if key not in palette:
            problems.append(f"theme.json palette 缺 `{key}`（键缺失 = 静默回落 default）")
            continue
        n = _as_hex16(palette[key])
        if n is None:
            problems.append(f"theme.json palette.{key} 非 RGB565（值 {palette[key]!r}）")
        elif not (0 <= n <= 0xFFFF):
            problems.append(f"theme.json palette.{key} = {n} 越界（须 0..=0xFFFF）")

    fonts = theme.get("fonts")
    if not isinstance(fonts, dict):
        problems.append("theme.json 缺 fonts 组（字号唯一事实源）")
        fonts = {}
    for key in THEME_FONT_KEYS:
        if key not in fonts:
            problems.append(f"theme.json fonts 缺 `{key}`（键缺失 = 静默回落 default）")
            continue
        n = fonts[key]
        if not isinstance(n, int) or isinstance(n, bool):
            problems.append(f"theme.json fonts.{key} 非整数（值 {n!r}）")
        elif n <= 0:
            problems.append(f"theme.json fonts.{key} = {n} 非正（字号须 > 0）")


def check_v2(scenes: dict, problems: list[str]) -> None:
    """V2：全场景 extra 标量键（int/float 值）须为正整数字面量。"""
    known_rect = {"x", "y", "w", "h"}
    for sid, scene in scenes.items():
        if sid in ("schema_version",) or not isinstance(scene, dict):
            continue
        for key, val in scene.items():
            if key in ("schema_version", "bg", "title", "list", "hint", "hud", "page") or key in known_rect:
                continue
            if isinstance(val, dict):
                for fk, fv in val.items():
                    if fk in known_rect:
                        continue
                    if isinstance(fv, (int, float)) and not isinstance(fv, bool):
                        if not is_positive_int(fv):
                            problems.append(f"V2: {sid}.{key}.{fk} = {fv!r} 非正整数字面量")


def main() -> int:
    ap = argparse.ArgumentParser(description="数据表兜底校验（gate 第 10a 项统一入口）")
    ap.add_argument("--root", type=Path, default=Path("."))
    ap.add_argument("--check", action="store_true", help="（当前仅校验模式，保留参数一致性）")
    args = ap.parse_args()

    root = args.root
    problems: list[str] = []

    data_dir = root / "assets/data"
    loaded: dict[str, dict] = {}
    for name in DATA_FILES:
        obj = load(data_dir / name, problems)
        if obj is not None:
            loaded[name] = obj

    referenced: set[str] = set()
    if "balance.json" in loaded:
        referenced = check_balance(loaded["balance.json"], problems)
    if "strings.json" in loaded:
        check_strings(loaded["strings.json"], referenced, problems)
    if "theme.json" in loaded:
        check_theme(loaded["theme.json"], problems)
    if "scenes.json" in loaded:
        check_scenes(loaded["scenes.json"], problems)
        scenes = loaded["scenes.json"]
        main_map = scenes.get("main_map")
        if not isinstance(main_map, dict):
            problems.append("V1: scenes.json 缺 main_map 段")
        else:
            check_v1(main_map, problems)
        check_v2(scenes, problems)
    check_controls(root, problems)

    for p in problems:
        print(f"ERROR {p}")
    if problems:
        print(f"validate_data: {len(problems)} problem(s)")
        return 1
    n_str = len([k for k in loaded.get("strings.json", {}) if k != "schema_version"])
    print(f"validate_data: OK（{len(loaded)} 表，strings {n_str} 键；main_map.grid 三键 + 全场景 extra 标量正整数）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
