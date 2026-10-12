#!/usr/bin/env python3
"""校验 assets/data/*.json 的完整性与数值合法域。

用法：
    python scripts/validate_data.py            # 校验
    python scripts/validate_data.py --check    # 同上（当前仅校验模式）

检查项：
1. 必填文件存在且是合法 JSON；
2. balance 数值域：概率 ∈ [0,1]、价格/HP/造价 > 0、权重 > 0、上限 > 下限；
3. strings 覆盖 balance 中每个 name_key（无孤儿引用）；
4. scenes 覆盖全部游戏场景；
5. controls.json（若存在）与 ref/controls.json 记录数一致。

退出码：0 通过；非 0 有问题（gate 判 FAIL）。
判据来源：docs/business/*（业务唯一事实源）+ docs/process/02_conventions.md §1。
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

DATA_FILES = ["balance.json", "strings.json", "scenes.json"]

GAME_SCENES = ["index", "select_role", "select_map", "main_map", "vs", "message", "saveload"]


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
        if "bg" not in layout:
            problems.append(f"scene `{sid}` 缺少 bg")
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
    src = root / "ref/controls.json"
    if src.is_file():
        with src.open(encoding="utf-8") as fh:
            orig = json.load(fh)
        if c.get("record_total") != orig.get("record_total"):
            problems.append(
                f"controls.json record_total={c.get('record_total')} != ref/controls.json {orig.get('record_total')}"
            )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
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
    if "scenes.json" in loaded:
        check_scenes(loaded["scenes.json"], problems)
    check_controls(root, problems)

    for p in problems:
        print(f"ERROR {p}")
    if problems:
        print(f"validate_data: {len(problems)} problem(s)")
        return 1
    n_str = len([k for k in loaded.get("strings.json", {}) if k != "schema_version"])
    print(f"validate_data: OK（{len(loaded)} 表，strings {n_str} 键）")
    return 0


if __name__ == "__main__":
    sys.exit(main())