#!/usr/bin/env python3
"""map_pack.py — 地图打包标准（S30 / `18_asset_content_pipeline`「不手写像素」）。

一张图 = 一个描述符（`assets/maps/<id>.json`）+ 一份共享版面（`assets/maps/_template.json`）。
**加图 / 换图 / 改环几何 / 改 CTA·键位 = 改描述符或模板 + 跑 `python scripts/map_pack.py sync`，
不改 Rust、不散改多份 JSON。**

生成（幂等）：
- 逐图内容 `assets/maps/<id>.json` → `assets/data/board.json` `maps.<id>`（格序 + 岛型）
- 共享版面 `assets/maps/_template.json` → `scenes.json` `main_map`（ring/world_canvas/island_home/
  deck_* + CTA·三键 primitives）+ `scene_manifest_v3.json` `main_map`（focus.bbox_pct + rhythm.beats）
- 资产登记校验：描述符引用的 `isle_<id>` / `island_home_<id>` 须已在 `ui_manifest.json` 登记（缺 = 报，不自动造 prompt）

子命令：
  sync           幂等回注（读描述符/模板 → 写出上面各文件）
  check / --check 只读对账：重推 → 与实际 diff，漂移/缺登记 = 非 0 退出（门禁判定）
  add <id>       生成骨架 `assets/maps/<id>.json`（空格序列，人工补）

工具契约（`14_tooling_catalog` 第 26/27 项）：幂等 + 只读模式 + 非 0 退出。
"""

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAPS_DIR = ROOT / "assets" / "maps"
BOARD_JSON = ROOT / "assets" / "data" / "board.json"
SCENES_JSON = ROOT / "assets" / "data" / "scenes.json"
MANIFEST_JSON = ROOT / "assets" / "data" / "ui_schema" / "scene_manifest_v3.json"
UI_MANIFEST = ROOT / "assets" / "ui_manifest.json"
TEMPLATE = MAPS_DIR / "_template.json"

# 逐图描述符允许的键（内容面 = 该图 board.json maps.<id>；资产面 = 注册校验）
DESC_KEYS = ("id", "board", "central_asset", "isle_assets")


def dump_canonical(path: Path, obj) -> None:
    """indent=1 逐字节规范形（与 `check_scenes_canonical` 同一序列化口径，防整文件 churn）。"""
    path.write_text(json.dumps(obj, ensure_ascii=False, indent=1) + "\n", encoding="utf-8", newline="\n")


def load(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def map_descriptors() -> dict:
    """`assets/maps/<id>.json`（排除 `_` 前缀模板）→ {id: desc}。"""
    if not MAPS_DIR.is_dir():
        return {}
    out = {}
    for p in sorted(MAPS_DIR.glob("*.json")):
        if p.name.startswith("_"):
            continue
        desc = load(p)
        id_ = desc.get("id") or p.stem
        out[id_] = desc
    return out


def derive_board() -> dict:
    """描述符 → 完整 `board.json` 对象（逐图 `maps.<id>` = 描述符 `board`；缺 maps 补空）。"""
    board = load(BOARD_JSON) if BOARD_JSON.is_file() else {"maps": {}}
    board.setdefault("maps", {})
    for id_, desc in map_descriptors().items():
        board["maps"][id_] = json.loads(json.dumps(desc["board"]))  # 深拷贝，避免共享
    return board


def apply_template_to_scenes(scenes: dict, tm: dict) -> None:
    """共享版面 → `scenes.json main_map`（extra 键 + CTA/三键 primitives + hint.y）。"""
    mm = scenes.setdefault("main_map", {})
    mm["ring"] = json.loads(json.dumps(tm["ring"]))
    mm["world_canvas"] = json.loads(json.dumps(tm["world_canvas"]))
    mm.setdefault("focus_anchor", {})["bbox"] = json.loads(json.dumps(tm["world_canvas"]))
    mm["island_home"] = json.loads(json.dumps(tm["island_home"]))
    mm.setdefault("hint", {})["y"] = tm["hint_y"]
    for key in ("spin", "trade", "sell", "build"):
        mm[f"deck_{key}"] = json.loads(json.dumps(tm["deck"][key]))
    reposition_deck_primitives(mm, tm)


def reposition_deck_primitives(mm: dict, tm: dict) -> None:
    """CTA/三键的 primitives（image 全 rect；icon/text 只移 y，x/w/h 不动）跟随 extra deck_* 键。"""
    off = tm.get("deck_offsets", {"icon_dx": 48, "icon_dy": 40, "text_dy": 0})
    for prim in mm.get("primitives", []):
        rect = prim.get("rect")
        if not rect:
            continue
        kind = prim.get("kind")
        asset = prim.get("params", {}).get("asset", "")
        icon = prim.get("params", {}).get("icon", "")
        text_key = prim.get("text_key", "")
        if kind == "image" and asset == "assets/ui/btn_spin.png":
            prim["rect"] = json.loads(json.dumps(tm["deck"]["spin"]))
        elif kind == "image" and asset == "assets/ui/deck_plaque.png":
            for k in ("trade", "sell", "build"):
                if tm["deck"][k]["x"] == rect.get("x"):
                    prim["rect"] = json.loads(json.dumps(tm["deck"][k]))
                    break
        elif kind == "icon" and icon in ("coin", "chest", "castle"):
            for k in ("trade", "sell", "build"):
                if tm["deck"][k]["x"] + off["icon_dx"] == rect.get("x"):
                    rect["y"] = tm["deck"][k]["y"] + off["icon_dy"]
                    break
        elif kind == "text" and text_key.startswith("scene.main_map.hud."):
            k = text_key.rsplit(".", 1)[-1]
            if k in ("trade", "sell", "build"):
                rect["y"] = tm["deck"][k]["y"] + off["text_dy"]


def apply_template_to_manifest(manifest: dict, tm: dict) -> None:
    """共享版面 → `scene_manifest_v3.json scenes.main_map`（focus.bbox_pct + rhythm.beats）。

    真实叙事在 `manifest["scenes"]["main_map"]`（**非**顶层 `main_map`；早期误写顶层 = 已清理）。
    """
    manifest.pop("main_map", None)  # 清理误加的顶层 stray（真实键在 scenes 下）
    narr = manifest.setdefault("scenes", {}).setdefault("main_map", {}).setdefault("narrative", {})
    narr.setdefault("focus", {})["bbox_pct"] = json.loads(json.dumps(tm["manifest_focus_bbox_pct"]))
    narr.setdefault("rhythm", {})["beats"] = json.loads(json.dumps(tm["manifest_beats"]))


def cmd_sync() -> int:
    tm = load(TEMPLATE)["main_map"]
    board = derive_board()
    dump_canonical(BOARD_JSON, board)
    scenes = load(SCENES_JSON)
    apply_template_to_scenes(scenes, tm)
    dump_canonical(SCENES_JSON, scenes)
    manifest = load(MANIFEST_JSON)
    apply_template_to_manifest(manifest, tm)
    dump_canonical(MANIFEST_JSON, manifest)
    print(f"map_pack sync: board.json（{len(board.get('maps', {}))} 图）+ scenes.json main_map + manifest main_map 已回注（幂等）")
    return 0


def check_asset_registry(problems) -> None:
    """描述符引用的 `isle_<id>`（裸岛型 id）/ `island_home_<id>`（central_asset 全名）须已登记。"""
    ui = load(UI_MANIFEST)
    registered = {a.get("file") for a in ui.get("assets", []) if a.get("file")}
    for id_, desc in map_descriptors().items():
        for isle in desc.get("isle_assets", []):
            fpath = f"assets/ui/isle_{isle}.png"
            if fpath not in registered:
                problems.append(f"maps.{id_} 岛型 `{isle}` 未登记（缺 {fpath} 资产条目）")
        central = desc.get("central_asset", "")
        if central and f"assets/ui/{central}.png" not in registered:
            problems.append(f"maps.{id_} 中央岛 `{central}` 未登记（缺 assets/ui/{central}.png 资产条目）")


def cmd_check() -> int:
    """只读对账：重推 board/scenes/manifest 关键值 vs 实际；漂移或缺登记 = 非 0。"""
    problems = []
    tm = load(TEMPLATE)["main_map"]
    board, actual_board = derive_board(), load(BOARD_JSON)
    for id_, desc in map_descriptors().items():
        if actual_board.get("maps", {}).get(id_) != desc["board"]:
            problems.append(f"board.json maps.{id} 与描述符 assets/maps/{id}.json 漂移")
    scenes = load(SCENES_JSON)
    probe = json.loads(json.dumps(scenes))  # 副本套用模板，比对 main_map 版面键
    apply_template_to_scenes(probe, tm)
    for key in ("ring", "world_canvas", "island_home"):
        if scenes["main_map"].get(key) != probe["main_map"][key]:
            problems.append(f"scenes.json main_map.{key} 与模板漂移")
    for key in ("deck_spin", "deck_trade", "deck_sell", "deck_build"):
        if scenes["main_map"].get(key) != probe["main_map"].get(key):
            problems.append(f"scenes.json main_map.{key} 与模板漂移")
    if scenes["main_map"].get("hint", {}).get("y") != probe["main_map"]["hint"]["y"]:
        problems.append("scenes.json main_map.hint.y 与模板漂移")
    manifest = load(MANIFEST_JSON)
    pmanifest = json.loads(json.dumps(manifest))
    apply_template_to_manifest(pmanifest, tm)
    cur = manifest.get("scenes", {}).get("main_map", {})
    tgt = pmanifest["scenes"]["main_map"]
    if cur.get("narrative", {}).get("focus", {}).get("bbox_pct") != tgt["narrative"]["focus"]["bbox_pct"]:
        problems.append("manifest scenes.main_map.focus.bbox_pct 与模板漂移")
    if cur.get("narrative", {}).get("rhythm", {}).get("beats") != tgt["narrative"]["rhythm"]["beats"]:
        problems.append("manifest scenes.main_map.rhythm.beats 与模板漂移")
    if "main_map" in manifest:
        problems.append("manifest 误加顶层 main_map（真实键在 scenes 下）")
    # 资产登记校验（描述符引用的 isle/central 资产须在 ui_manifest 登记）
    check_asset_registry(problems)
    if problems:
        for p in problems:
            print(f"[FAIL] {p}")
        print(f"map_pack check: {len(problems)} 处漂移/缺登记")
        return 1
    print("map_pack check: 0 漂移（board/scenes/manifest 与描述符·模板一致 + 资产已登记）")
    return 0


def cmd_add(map_id: str) -> int:
    target = MAPS_DIR / f"{map_id}.json"
    if target.exists():
        print(f"map_pack add: assets/maps/{map_id}.json 已存在，拒覆盖（人工改）")
        return 1
    MAPS_DIR.mkdir(parents=True, exist_ok=True)
    skeleton = {
        "id": map_id,
        "board": {"grid_w": 20, "grid_h": 20, "spaces": []},
        "central_asset": f"island_home_{map_id}",
        "isle_assets": [],
    }
    dump_canonical(target, skeleton)
    print(f"map_pack add: 生成骨架 assets/maps/{map_id}.json（补 spaces + 跑 sync + 建 isle/central 资产）")
    return 0


def main(argv) -> int:
    argv = argv[1:]
    if argv and argv[0] in ("add",):
        if len(argv) < 2:
            print("用法: map_pack.py add <id>", file=sys.stderr)
            return 2
        return cmd_add(argv[1])
    if argv and argv[0] in ("check", "--check"):
        return cmd_check()
    if argv and argv[0] == "sync":
        return cmd_sync()
    print("用法: python scripts/map_pack.py [sync | check | add <id>]", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
