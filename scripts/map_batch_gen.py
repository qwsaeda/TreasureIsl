#!/usr/bin/env python3
"""map_batch_gen.py — 批量生成 20 张地图包（S37）。

基于 m01 包模板，程序化生成 20 个不同场景地图包：
  - 20 个不同主题（island_home 资产 + vs_theme tint + 事件/文案差异）
  - 每个包 = 独立可分发的 .map-pack
  - 复用共享池 10 类岛卡面（isle_*.png）
  - 每图 2 个特有海事事件（`rules::events::MaritimeEvent` kind）

工具契约（14_tooling_catalog）：
  - `--check` 只读模式（生成清单 + 校验已装包，不写文件）
  - 幂等（同输入同产物；已存在的包目录跳过）
  - 非 0 退出 = 违规

用法：
  python3 scripts/map_batch_gen.py               # 生成全部 20 包
  python3 scripts/map_batch_gen.py --check       # 只读校验
  python3 scripts/map_batch_gen.py --only m05     # 只生成指定包
"""
import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAPS_DIR = ROOT / "assets" / "maps"
POOL = ROOT / "assets" / "ui"
BOARD_JSON = ROOT / "assets" / "data" / "board.json"
UI_MANIFEST = ROOT / "assets" / "ui_manifest.json"

# ----------------------------------------------------------------- 常量
TEMPLATE_ID = "m01"
TARGET_COUNT = 20
MAP_IDS = [f"m{i:02d}" for i in range(1, TARGET_COUNT + 1)]

# 20 主题定义（每图：名称 / vs_theme tint / 2 个事件 / 中央岛资产 / 场景标签）
THEMES: list[dict] = [
    # m01-m03 已存在，保留现有包（verify 时不重建）
    {"id": "m01", "name": "海图一", "tint": "0xFFFF", "events": [
        {"id": "m01_reef_current", "kind": "shipwreck"},
        {"id": "m01_trade_wind", "kind": "wind_god", "jump_cells": 3},
    ]},
    {"id": "m02", "name": "海图二", "tint": "0xEF5D", "events": [
        {"id": "m02_mirage", "kind": "wind_god", "jump_cells": 2},
        {"id": "m02_red_tide", "kind": "storm"},
    ]},
    {"id": "m03", "name": "海图三", "tint": "0x9DDF", "events": [
        {"id": "m03_whirlpool", "kind": "shipwreck"},
        {"id": "m03_rescue", "kind": "island_rescue"},
    ]},
    # m04-m20 新生成 17 张
    {"id": "m04", "name": "冰霜极地", "tint": "0x78FF", "events": [
        {"id": "m04_iceberg", "kind": "shipwreck"},
        {"id": "m04_arctic_wind", "kind": "wind_god", "jump_cells": 2},
    ]},
    {"id": "m05", "name": "热带乐园", "tint": "0xF81F", "events": [
        {"id": "m05_monsoon", "kind": "storm"},
        {"id": "m05_spice_trade", "kind": "treasure_wreck"},
    ]},
    {"id": "m06", "name": "沙漠绿洲", "tint": "0xFDB9", "events": [
        {"id": "m06_sandstorm", "kind": "storm"},
        {"id": "m06_oasis_rescue", "kind": "island_rescue"},
    ]},
    {"id": "m07", "name": "迷雾海藻", "tint": "0x1D7F", "events": [
        {"id": "m07_ghost_fog", "kind": "shipwreck"},
        {"id": "m07_smuggler", "kind": "blacksmith"},
    ]},
    {"id": "m08", "name": "怒海断崖", "tint": "0x5B7F", "events": [
        {"id": "m08_roaring_surge", "kind": "storm"},
        {"id": "m08_cave_rescue", "kind": "island_rescue"},
    ]},
    {"id": "m09", "name": "落日浅滩", "tint": "0xFFDF", "events": [
        {"id": "m09_golden_tide", "kind": "treasure_wreck"},
        {"id": "m09_trade_wind", "kind": "wind_god", "jump_cells": 4},
    ]},
    {"id": "m10", "name": "深渊海沟", "tint": "0x081F", "events": [
        {"id": "m10_abyss", "kind": "shipwreck"},
        {"id": "m10_ghost_sail", "kind": "treasure_wreck"},
    ]},
    {"id": "m11", "name": "双岩骷髅", "tint": "0xC11F", "events": [
        {"id": "m11_red_reef", "kind": "storm"},
        {"id": "m11_cannibal", "kind": "shipwreck"},
    ]},
    {"id": "m12", "name": "宝藏之岛", "tint": "0xFF1B", "events": [
        {"id": "m12_gold_rush", "kind": "treasure_wreck"},
        {"id": "m12_rival_raid", "kind": "storm"},
    ]},
    {"id": "m13", "name": "红树林湾", "tint": "0x843F", "events": [
        {"id": "m13_smuggler_cove", "kind": "blacksmith"},
        {"id": "m13_mangrove", "kind": "shipwreck"},
    ]},
    {"id": "m14", "name": "幽灵浅礁", "tint": "0xDFFF", "events": [
        {"id": "m14_phantom_sail", "kind": "treasure_wreck"},
        {"id": "m14_ghost_light", "kind": "wind_god", "jump_cells": 1},
    ]},
    {"id": "m15", "name": "熔岩海岸", "tint": "0xFF79", "events": [
        {"id": "m15_lava_flood", "kind": "storm"},
        {"id": "m15_smith", "kind": "blacksmith"},
    ]},
    {"id": "m16", "name": "月夜海湾", "tint": "0x5E1B", "events": [
        {"id": "m16_moonlit", "kind": "wind_god", "jump_cells": 2},
        {"id": "m16_moon_rescue", "kind": "island_rescue"},
    ]},
    {"id": "m17", "name": "水晶港湾", "tint": "0xC0FF", "events": [
        {"id": "m17_crystal", "kind": "treasure_wreck"},
        {"id": "m17_glass_wind", "kind": "wind_god", "jump_cells": 3},
    ]},
    {"id": "m18", "name": "风暴残骸", "tint": "0xB890", "events": [
        {"id": "m18_wreck_beach", "kind": "shipwreck"},
        {"id": "m18_torment", "kind": "storm"},
    ]},
    {"id": "m19", "name": "珊瑚环礁", "tint": "0xF9D4", "events": [
        {"id": "m19_coral_reef", "kind": "treasure_wreck"},
        {"id": "m19_atoll_wind", "kind": "wind_god", "jump_cells": 2},
    ]},
    {"id": "m20", "name": "火山之眼", "tint": "0xFF39", "events": [
        {"id": "m20_volcanic_eye", "kind": "storm"},
        {"id": "m20_lava_vent", "kind": "blacksmith"},
    ]},
]

# 岛型顺序（按 m01 实测领地对 group 序：group 1~10 = 岛型对 1~10）
ISLE_KINDS: list[str] = [
    "harbour", "lighthouse", "wreck", "pirate_lair", "shipyard",
    "trading_post", "skull_cliff", "volcano", "lightship", "boss_fort",
]

# 格 kind 序列（I1~I5：start@0 / boss@15(31//2) / 20 领地 + 9 功能）
# 与 m01 实测 kind 序列完全一致：功能格 3/6/9/12/15/18/21/24/29/30
# 领地对 = 1-2,4-5,7-8,10-11,13-14,16-17,19-20,22-23,25-26,27-28（group 1~10）
# group 序 → 岛型（m01 实测抄录）：
# 1=harbour 2=lighthouse 3=wreck(gold) 4=pirate_lair(loot) 5=shipyard(ship)
# 6=trading_post(trade) 7=skull_cliff(loot) 8=volcano(gold) 9=lightship(dice) 10=boss_fort(boss)
GROUP_TO_ISLE: dict[int, str] = {
    1: "harbour", 2: "lighthouse", 3: "wreck", 4: "pirate_lair", 5: "shipyard",
    6: "trading_post", 7: "skull_cliff", 8: "volcano", 9: "lightship", 10: "boss_fort",
}
# 岛型全集（manifest isle_assets / pool 资产索引用）
ISLE_KINDS: list[str] = [
    "harbour", "lighthouse", "wreck", "pirate_lair", "shipyard",
    "trading_post", "skull_cliff", "volcano", "lightship", "boss_fort",
]

KIND_SEQUENCE: list[str] = [
    "start",    # 0
    "territory", "territory",  # 1,2   group=1
    "event",    # 3
    "territory", "territory",  # 4,5   group=2
    "card",     # 6
    "territory", "territory",  # 7,8   group=3
    "treasure", # 9
    "territory", "territory",  # 10,11 group=4
    "event",    # 12
    "territory", "territory",  # 13,14 group=5
    "boss",     # 15
    "territory", "territory",  # 16,17 group=6
    "event",    # 18
    "territory", "territory",  # 19,20 group=7
    "treasure", # 21
    "territory", "territory",  # 22,23 group=8
    "card",     # 24
    "territory", "territory",  # 25,26 group=9
    "territory", "territory",  # 27,28 group=10
    "event",    # 29
    "card",     # 30
]

# 领地格资源类型（m01 实测抄录：group 1~10 对应 resource）
ISLE_RESOURCES: dict[str, str] = {
    "harbour": "fishing",      # group 1
    "lighthouse": "reef",     # group 2
    "wreck": "gold",          # group 3
    "pirate_lair": "loot",    # group 4
    "shipyard": "ship",       # group 5
    "trading_post": "trade",  # group 6
    "skull_cliff": "loot",    # group 7
    "volcano": "gold",        # group 8
    "lightship": "dice",      # group 9
    "boss_fort": "boss",      # group 10
}


# m01 实际 cell 序列（实测抄录；m02/m03 同构）
M01_CELLS: list[list[int]] = [
    [10, 0], [11, 1], [13, 1], [15, 2], [16, 3], [17, 5], [18, 6], [18, 8],
    [18, 10], [18, 12], [18, 13], [17, 15], [15, 16], [14, 17], [12, 18],
    [10, 18], [9, 18], [7, 18], [5, 17], [4, 16], [2, 15], [1, 13], [1, 12],
    [1, 10], [1, 8], [1, 6], [2, 5], [3, 3], [4, 2], [6, 1], [8, 1],
]


def cell_for(i: int) -> list[int]:
    """格 i 的等距格坐标 = m01 实测序列（31 格全在 20×20 内，31 不重复）。"""
    return list(M01_CELLS[i])


# 功能格 group（m01 实测抄录：每个功能格有自己的 group 值）
FEATURE_GROUP: dict[int, int] = {0: 1, 3: 3, 6: 4, 9: 5, 12: 3, 15: 6, 18: 3, 21: 5, 24: 4, 29: 3, 30: 4}


def build_map_data(theme: dict, map_id: str) -> dict:
    """构造 map.json 内容（31 格 + events + merchants + grid，与 m01 实测同构）。"""
    grid_w = 20
    grid_h = 20
    spaces: list[dict] = []
    isle_pair_idx = 0
    territory_pair_starts = {1, 4, 7, 10, 13, 16, 19, 22, 25, 27}
    for i, kind in enumerate(KIND_SEQUENCE):
        if kind == "territory":
            if i in territory_pair_starts:
                isle_pair_idx += 1
            entry: dict = {"cell": cell_for(i), "group": isle_pair_idx, "kind": kind, "terrain": "L"}
            isle_id = GROUP_TO_ISLE[isle_pair_idx]
            entry["isle"] = isle_id
            entry["resource"] = ISLE_RESOURCES[isle_id]
            entry["building"] = 1
        else:
            entry = {"cell": cell_for(i), "group": FEATURE_GROUP[i], "kind": kind, "terrain": "L"}
        spaces.append(entry)

    return {
        "grid_w": grid_w,
        "grid_h": grid_h,
        "spaces": spaces,
        "events": theme["events"],
        "merchants": [
            {"id": "weapon", "cell": [15, 2], "price_mult": 1.0},
            {"id": "slave", "cell": [15, 16], "price_mult": 1.0},
        ],
    }


def pool_assets() -> list[dict]:
    """共享池 10 类岛卡面资产（isle_*.png，hash 实时计算）。"""
    assets: list[dict] = []
    for isle in ISLE_KINDS:
        path = POOL / f"isle_{isle}.png"
        if not path.is_file():
            continue
        assets.append({
            "file": f"assets/ui/isle_{isle}.png",
            "source": "pool",
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "bytes": path.stat().st_size,
        })
    return assets


def island_home_asset(map_id: str) -> dict:
    """中央岛专属资产（island_home_<id>.png，pool 来源）。"""
    path = POOL / f"island_home_{map_id}.png"
    if not path.is_file():
        raise SystemExit(f"map_batch_gen: 缺 island_home 资产 {path}（先 ui_assets.py add）")
    return {
        "file": f"assets/ui/island_home_{map_id}.png",
        "source": "pool",
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }


LAYOUT_TEMPLATE = {
    "ring": {"center_x": 540, "center_y": 784, "radius": 480, "card_w": 96, "card_h": 120},
    "world_canvas": {"x": 11, "y": 250, "w": 1048, "h": 1056},
    "island_home": {"x": 160, "y": 400, "w": 760, "h": 760},
    "hint": {"x": 540, "y": 1760},
    "deck_trade": {"x": 48, "y": 1608, "w": 312, "h": 136},
    "deck_sell": {"x": 384, "y": 1608, "w": 312, "h": 136},
    "deck_build": {"x": 720, "y": 1608, "w": 312, "h": 136},
    "focus_anchor": {"type": "world_canvas", "bbox": {"x": 11, "y": 250, "w": 1048, "h": 1056}},
    "focus_bbox_pct": {"x": 1, "y": 13, "w": 97, "h": 55},
}


def vs_bg_asset(map_id: str) -> dict:
    """VS 战斗背景专属资产（vs_bg_<id>.png，vs_v2 目录，pool 来源）。"""
    path = ROOT / "assets" / "ui" / "vs_v2" / f"vs_bg_{map_id}.png"
    if not path.is_file():
        # 回退到默认共享池（m01/m02/m03 用共享池）
        return {
            "file": "assets/ui/vs_v2/vs_bg.png",
            "source": "pool",
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else "",
            "bytes": path.stat().st_size if path.is_file() else 0,
        }
    return {
        "file": f"assets/ui/vs_v2/vs_bg_{map_id}.png",
        "source": "pool",
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "bytes": path.stat().st_size,
    }


def build_manifest(map_id: str, name: str, theme: dict) -> dict:
    """构造 manifest.json（21_map_package_spec.md §2 全字段）。"""
    isle_assets = ISLE_KINDS
    # vs_theme 背景：m04~m20 用专属 vs_bg_<id>，m01~m03 用共享池默认
    vs_bg_id = f"vs_bg_{map_id}" if map_id >= "m04" else "vs_bg"
    vs_bg_path = ROOT / "assets" / "ui" / "vs_v2" / f"{vs_bg_id}.png"
    if not vs_bg_path.is_file():
        vs_bg_id = "vs_bg"  # 回退
    vs_theme = {
        "bg": vs_bg_id,
        "siege": "bg_siege",
        "dungeon": "bg_dungeon",
        "boss": "bg_boss",
        "encounter": vs_bg_id,
        "tint": theme["tint"],
    }
    # vs_theme bg 资产（若非默认共享池，加入 assets 索引）
    extra_assets: list[dict] = []
    if vs_bg_id != "vs_bg":
        extra_assets.append(vs_bg_asset(map_id))
    return {
        "schema": "map-pack/1",
        "id": map_id,
        "version": "1.0.0",
        "data_version": "board-v5",
        "central_asset": f"island_home_{map_id}",
        "isle_assets": isle_assets,
        "map": build_map_data(theme, map_id),
        "layout": LAYOUT_TEMPLATE,
        "strings": {f"glmap{map_id}.Map": name},
        "assets": pool_assets() + [island_home_asset(map_id)] + extra_assets,
        "invariants": {"spaces": 31, "start": 1, "territory": 20, "feature": 9, "boss": 1, "isle_kinds": 10},
        "vs_theme": vs_theme,
    }


def dump_canonical(path: Path, obj) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(obj, ensure_ascii=False, indent=1) + "\n",
        encoding="utf-8", newline="\n",
    )


def cmd_check() -> int:
    """只读：校验 20 张岛资产在位 + board.json 含 m01~m20 段 + 包目录状态。"""
    problems: list[str] = []
    for map_id in MAP_IDS:
        asset = POOL / f"island_home_{map_id}.png"
        if not asset.is_file():
            problems.append(f"缺 island_home_{map_id}.png")
        pack_dir = MAPS_DIR / map_id
        if not (pack_dir / "manifest.json").is_file():
            problems.append(f"缺包 {pack_dir}（无 manifest.json）")
    board = json.loads(BOARD_JSON.read_text(encoding="utf-8"))
    for map_id in MAP_IDS:
        if map_id not in board.get("maps", {}):
            problems.append(f"board.json 缺 maps.{map_id}")
    if problems:
        print(f"map_batch_gen check: {len(problems)} 项缺")
        for item in problems:
            print(f"  [FAIL] {item}")
        return 1
    print(f"map_batch_gen check: 0 缺（20 岛资产 + 20 包 + board.json 全在）")
    return 0


def cmd_generate(only: list[str] | None) -> int:
    """生成包目录（幂等；已存在 m01/m02/m03 跳过，m04 补齐 + m05~m20 新建）。"""
    existing = {p.name for p in MAPS_DIR.iterdir() if p.is_dir() and (p / "manifest.json").is_file()}
    skip_ids = {"m01", "m02", "m03"}  # 已存在的 3 张保留
    created = 0
    for theme in THEMES:
        map_id = theme["id"]
        if only and map_id not in only:
            continue
        if map_id in skip_ids:
            print(f"map_batch_gen: 跳过 {map_id}（已存在）")
            continue
        pack_dir = MAPS_DIR / map_id
        manifest = build_manifest(map_id, theme["name"], theme)
        # 写 4 文件
        dump_canonical(pack_dir / "manifest.json", manifest)
        dump_canonical(pack_dir / "map.json", manifest["map"])
        dump_canonical(pack_dir / "layout.json", manifest["layout"])
        dump_canonical(pack_dir / "strings.json", manifest["strings"])
        print(f"map_batch_gen: {map_id} → {pack_dir.relative_to(ROOT)}（4 文件已建）")
        created += 1
    print(f"map_batch_gen: 共生成 {created} 个新包（跳过 {len(skip_ids)} 个已存在）")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="批量生成 20 张地图包（S37）")
    parser.add_argument("--check", action="store_true",
                        help="工具契约只读模式（= check 子命令；不写文件；缺包/资产未登记 ⇒ 非 0 退出）")
    sub = parser.add_subparsers(dest="cmd", required=False)
    p_check = sub.add_parser("check", help="只读校验（资产 + 包 + board）")
    p_gen = sub.add_parser("generate", help="生成包目录（幂等）")
    p_gen.add_argument("--only", help="只生成指定 id（逗号分隔）")
    args = parser.parse_args(argv[1:])
    if args.check or args.cmd == "check":
        return cmd_check()
    if args.cmd == "generate":
        only = args.only.split(",") if args.only else None
        return cmd_generate(only)
    parser.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
