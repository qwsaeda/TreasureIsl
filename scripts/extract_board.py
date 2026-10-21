#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""生成主地图「环岛赛道」数据 `assets/data/board.json`。

设计（`docs/business/progression.md` §环岛赛道）：
- 赛道 = 岛屿外环（Moore 邻域轮廓追踪），保留原版地图形状与坐标；
- 起点 = `balance.selection.maps[].player_start` 最近的环格；
- BOSS / 据点 / 原版特殊地块（event/card/war/MAZE/pool/Castle）按坐标就近吸附到环上；
- 分区 = 9 个连续段（对齐原版地形类别码 1..9，见 `legacy/docs/facts/map.md` F-308）。

输入：`assets/data/terrain.json` + `assets/data/balance.json`
      + `legacy/data/glmapm0*.Map`（原版地形/特殊地块）
      + `legacy/assets_raw/glmap_m0*/mapinfo.ini`（瓦片索引 → 文件名）
输出：`assets/data/board.json`

用法：`python scripts/extract_board.py [--check]`
  --check = 只校验已生成文件与本脚本输出一致（CI 用，不写文件）
"""
from __future__ import annotations

import argparse
import json
import re
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TERRAIN_JSON = ROOT / "assets" / "data" / "terrain.json"
BALANCE_JSON = ROOT / "assets" / "data" / "balance.json"
OUT_JSON = ROOT / "assets" / "data" / "board.json"

# 原版地图文件 / 图例（m03 缺原版资源 → 仅用 terrain.json）
MAP_FILES = {
    "m01": ROOT / "legacy" / "data" / "glmapm01.Map",
    "m02": ROOT / "legacy" / "data" / "glmapm02.Map",
    "m03": ROOT / "legacy" / "data" / "glmapm03.Map",
}
MAPINFO_FILES = {
    "m01": ROOT / "legacy" / "assets_raw" / "glmap_m01" / "mapinfo.ini",
    "m02": ROOT / "legacy" / "assets_raw" / "glmap_m02" / "mapinfo.ini",
}

# 8 邻域顺序（顺时针，起 = 北）
NBR8 = [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)]

# 原版瓦片名 → 赛道格类型
TILE_KIND = {
    "event": "event",
    "event0": "event",
    "card": "card",
    "card0": "card",
    "war": "battle",
    "maze": "battle",
    "pool": "treasure",
    "castle": "city",
}


def load_original_cells(path: Path) -> dict[int, dict]:
    """解析原版地形图（`ti-formats` 同款布局）：cell 索引 → 瓦片索引。"""
    buf = path.read_bytes()
    grid_w, grid_h, _flags, _off = struct.unpack_from("<4I", buf, 0)
    words = struct.unpack_from(f"<{(len(buf) - 16) // 4}I", buf, 16)
    cells: dict[int, dict] = {}
    for k in range(grid_w * grid_h - 1):
        base = 2 + k * 9
        rec = words[base : base + 9]
        cell = rec[8] & 0xFFFF
        cells[cell] = {"row": cell // grid_w, "col": cell % grid_w, "tile": rec[5]}
    return cells


def load_mapinfo(path: Path) -> dict[int, str]:
    """`mapinfo.ini`（UTF-16LE）→ 瓦片索引 → 文件名（去扩展名）。"""
    text = path.read_bytes().decode("utf-16")
    names: dict[int, str] = {}
    for idx, body in re.findall(r"\[(\d+)\](.*?)(?=\[|\Z)", text, re.S):
        m = re.search(r"filename=(\S+)", body)
        if m:
            names[int(idx)] = m.group(1).removesuffix(".pcx").lower()
    return names


def trace_ring(rows: list[str]) -> list[tuple[int, int]]:
    """陆地掩码的 Moore 邻域轮廓追踪 → 闭合环格序列（岛屿外岸线）。"""
    h = len(rows)
    w = len(rows[0])
    land = [[rows[r][c] in "LRT" for c in range(w)] for r in range(h)]

    def is_land(r: int, c: int) -> bool:
        return 0 <= r < h and 0 <= c < w and land[r][c]

    start = None
    for r in range(h):
        for c in range(w):
            if land[r][c]:
                start = (r, c)
                break
        if start:
            break
    if start is None:
        return []

    ring: list[tuple[int, int]] = []
    cur = start
    back = (start[0], start[1] - 1)
    for _ in range(h * w * 8):
        ring.append(cur)
        idx = NBR8.index((back[0] - cur[0], back[1] - cur[1]))
        found = None
        for k in range(1, 9):
            i = (idx + k) % 8
            rr, cc = cur[0] + NBR8[i][0], cur[1] + NBR8[i][1]
            if is_land(rr, cc):
                found = (i, (rr, cc))
                break
        if found is None:
            break
        i, (nr, nc) = found
        pb = (i - 1) % 8
        back = (cur[0] + NBR8[pb][0], cur[1] + NBR8[pb][1])
        cur = (nr, nc)
        if cur == start:
            break
    # 轮廓追踪在“厚岸线”处会重复访问同一格 → 按首次出现去重（保持环序）
    seen: set[tuple[int, int]] = set()
    uniq: list[tuple[int, int]] = []
    for cell in ring:
        if cell not in seen:
            seen.add(cell)
            uniq.append(cell)
    return uniq


def rotate_to(ring: list[tuple[int, int]], target: tuple[int, int]) -> list[tuple[int, int]]:
    """把环旋转到以离 `target` 最近的格开头。"""
    if not ring:
        return ring
    best = min(
        range(len(ring)),
        key=lambda i: (abs(ring[i][0] - target[0]) + abs(ring[i][1] - target[1]), i),
    )
    return ring[best:] + ring[:best]


def nearest_index(ring: list[tuple[int, int]], target: tuple[int, int]) -> int:
    return min(
        range(len(ring)),
        key=lambda i: (abs(ring[i][0] - target[0]) + abs(ring[i][1] - target[1]), i),
    )


def build() -> dict:
    terrain = json.loads(TERRAIN_JSON.read_text(encoding="utf-8"))
    balance = json.loads(BALANCE_JSON.read_text(encoding="utf-8"))
    map_defs = {m["id"]: m for m in balance["selection"]["maps"]}

    out: dict = {"schema_version": 1, "maps": {}}
    for map_id, map_def in map_defs.items():
        rows = terrain["maps"][map_id]["rows"]
        ring = rotate_to(
            trace_ring(rows),
            (int(map_def["player_start"][0]), int(map_def["player_start"][1])),
        )
        if not ring:
            raise SystemExit(f"[extract_board] {map_id} 追踪不到环岛轨道")

        # 原版特殊地块（event/card/war/MAZE/pool/Castle）按坐标就近吸附
        tile_names: dict[tuple[int, int], str] = {}
        cells = load_original_cells(MAP_FILES[map_id])
        info = MAPINFO_FILES.get(map_id)
        if info is not None and info.exists():
            names = load_mapinfo(info)
            for cell, rec in cells.items():
                name = names.get(rec["tile"])
                if name is None:
                    continue
                kind = TILE_KIND.get(name)
                if kind is not None:
                    tile_names[(rec["row"], rec["col"])] = kind

        kinds: list[str] = []
        for cell in ring:
            terrain_ch = rows[cell[0]][cell[1]]
            if terrain_ch == "T":
                kinds.append("treasure")
            else:
                kinds.append(tile_names.get(cell, "plain"))

        # 起点 / BOSS / 据点（按 balance 坐标就近吸附，互斥占位）
        taken: set[int] = set()
        start_idx = nearest_index(ring, (int(map_def["player_start"][0]), int(map_def["player_start"][1])))
        kinds[start_idx] = "start"
        taken.add(start_idx)

        boss_tile = map_def.get("boss_tile")
        if boss_tile:
            bi = nearest_index(ring, (int(boss_tile[0]), int(boss_tile[1])))
            if bi not in taken:
                kinds[bi] = "boss"
                taken.add(bi)

        landmark_kinds = ["port", "city"]
        for i, lm in enumerate(map_def.get("landmarks", [])):
            li = nearest_index(ring, (int(lm[0]), int(lm[1])))
            if li in taken:
                continue
            kinds[li] = landmark_kinds[i % len(landmark_kinds)]
            taken.add(li)

        # 普通格按固定循环铺事件格（对齐大富翁式棋盘节奏：每 7 格 1 个事件/卡牌/遭遇）
        cycle = ["plain", "plain", "event", "plain", "card", "plain", "battle"]
        for i, kind in enumerate(kinds):
            if kind == "plain":
                kinds[i] = cycle[i % len(cycle)]

        # 9 个连续分区（对齐原版地形类别码 1..9）
        n = len(ring)
        spaces = []
        for i, ((r, c), kind) in enumerate(zip(ring, kinds)):
            group = i * 9 // n + 1
            spaces.append(
                {
                    "cell": [r, c],
                    "kind": kind,
                    "group": group,
                    "terrain": rows[r][c],
                }
            )
        out["maps"][map_id] = {
            "grid_w": int(map_def["grid_w"]),
            "grid_h": int(map_def["grid_h"]),
            "spaces": spaces,
        }
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="生成 assets/data/board.json（环岛赛道）")
    ap.add_argument("--check", action="store_true", help="只校验一致性，不写文件")
    args = ap.parse_args()

    data = build()
    text = json.dumps(data, ensure_ascii=False, indent=1, sort_keys=True) + "\n"
    if args.check:
        if not OUT_JSON.exists():
            print("[extract_board] board.json 缺失", file=sys.stderr)
            return 1
        if OUT_JSON.read_text(encoding="utf-8") != text:
            print("[extract_board] board.json 与脚本输出不一致（请重跑生成）", file=sys.stderr)
            return 1
        print("[extract_board] board.json 与脚本输出一致")
        return 0

    OUT_JSON.write_text(text, encoding="utf-8")
    for map_id, m in data["maps"].items():
        counts: dict[str, int] = {}
        for sp in m["spaces"]:
            counts[sp["kind"]] = counts.get(sp["kind"], 0) + 1
        print(f"[extract_board] {map_id}: {len(m['spaces'])} 格 {counts}")
    print(f"[extract_board] 已写入 {OUT_JSON}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
