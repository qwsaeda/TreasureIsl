#!/usr/bin/env python
# -*- coding: utf-8 -*-
"""ui_islands.py — 主地图 v4 岛屿资产生成（terrain.json → 抗锯齿暖砂剪影 PNG ×3）。

背景：S21.4 起主地图岛屿 = `main_map_island::draw_board` 代码逐格菱形（暖砂底 + 墨线 +
礁石浅滩 + 纹理短划）。代码画的优势是跟 `board_fit` 同源；劣势 = 硬像素锯齿边，
在满屏羊皮海图（`chart_full`，v4 底图）上观感「色块感」重（2026-10-26 用户「地图是个
垃圾」审计 FIND-305）。

本工具 = 把 3 张地图的岛屿渲染成**透明底 PNG 资产**（4× 超采样 + alpha 抗锯齿 + 落影 +
暖砂渐变 + 礁石浅滩晕 + 墨线海岸 + 确定性纹理），落进 `world_canvas` 与 `board_fit`
**同几何**（公式抄 `chrome::board_fit`：`tw = w/(2g)` 8 对齐、`th = tw/2`、
`x0 = cx - tw`、`y0 = cy - g*th`），运行时 `blit_image` 1:1 贴入 `world_canvas`
（缺资产 = 回退代码绘制，不 panic）。节点牌/航线/船 = 代码层不变（命中判定仍数据驱动，
EXP-067 不破）。

用法（工具契约：落 scripts/ + 幂等 + check 只读 + 非 0 退出）：
  python scripts/ui_islands.py            # generate：重建 3 张 + 登记 ui_manifest.json
  python scripts/ui_islands.py --check    # 只读校验：文件在位 + 尺寸 = world_canvas + 登记齐全

产物（唯一事实源 = 本工具 + ui_manifest.json）：
  assets/ui/island_m01.png  (994×826 RGBA)
  assets/ui/island_m02.png  (994×826 RGBA)
  assets/ui/island_m03.png  (994×826 RGBA)
"""

import io
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SUPER = 4  # 超采样倍数（4× 画 → LANCZOS 回 1×，边抗锯齿）


def _load_json(rel: str):
    return json.load(io.open(os.path.join(ROOT, rel), encoding="utf-8"))


def _board_fit(world_canvas: dict, g: int):
    """抄 `chrome::board_fit`（world_canvas 分支）：tw 8 对齐、th = tw/2。"""
    w, h = world_canvas["w"], world_canvas["h"]
    tw = w // (2 * g)
    tw = (tw // 8) * 8
    if tw < 8:
        return None
    th = tw // 2
    cx = world_canvas["x"] + w // 2
    cy = world_canvas["y"] + h // 2
    return {
        "tw": tw,
        "th": th,
        "x0": cx - tw,
        "y0": cy - g * th,
        "gw": w,
        "gh": h,
        "wx": world_canvas["x"],
        "wy": world_canvas["y"],
    }


def _fnv(seed: int) -> int:
    h = 2166136261
    for b in seed.to_bytes(8, "little"):
        h ^= b
        h = (h * 16777619) & 0xFFFFFFFF
    return h


def _diamond(fit: dict, r: int, c: int) -> list:
    """格 (r,c) 菱形 4 顶点（**画布坐标**：绝对坐标 - world_canvas 原点 (wx, wy)）。"""
    sx = fit["x0"] + (c - r) * fit["tw"] - fit["wx"]
    sy = fit["y0"] + (r + c) * fit["th"] - fit["wy"]
    tw, th = fit["tw"], fit["th"]
    return [
        (sx + tw, sy),
        (sx + 2 * tw, sy + th),
        (sx + tw, sy + 2 * th),
        (sx, sy + th),
    ]


def render_island(rows, legend, fit, out_path):
    """渲染单张地图岛屿 → RGBA PNG（尺寸 = world_canvas w×h）。"""
    from PIL import Image, ImageDraw, ImageFilter

    gw, gh = fit["gw"], fit["gh"]
    S = SUPER
    img = Image.new("RGBA", (gw * S, gh * S), (0, 0, 0, 0))
    shadow = Image.new("RGBA", (gw * S, gh * S), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    ds = ImageDraw.Draw(shadow)

    g = len(rows)
    code_at = lambda r, c: (rows[r][c] if 0 <= r < g and 0 <= c < g else "S")

    # 色板（资产级设计定值；与 theme 令牌语义对齐：暖砂/浅滩/墨线/宝）
    SAND = (214, 176, 122, 235)
    SAND_HI = (228, 198, 148, 90)
    REEF = (150, 198, 184, 200)
    REEF_HALO = (150, 198, 184, 90)
    TREASURE = (201, 162, 39, 235)
    INK = (58, 38, 22, 235)
    TEX = (120, 84, 48, 80)
    SHADOW = (20, 14, 8, 110)

    # 1) 落影（全岛剪影 + 偏移 + 模糊）
    for r in range(g):
        for c in range(g):
            if code_at(r, c) == "S":
                continue
            poly = [(x * S + 4 * S, y * S + 6 * S) for x, y in _diamond(fit, r, c)]
            ds.polygon(poly, fill=SHADOW)
    shadow = shadow.filter(ImageFilter.GaussianBlur(10 * S // 2))
    img = Image.alpha_composite(shadow, img)
    d = ImageDraw.Draw(img)

    # 2) 礁石浅滩晕（礁格外扩 3px×S）
    for r in range(g):
        for c in range(g):
            if code_at(r, c) != "R":
                continue
            base = _diamond(fit, r, c)
            off = 3 * S
            halo = [
                (base[0][0], base[0][1] - off),
                (base[1][0] + off, base[1][1]),
                (base[2][0], base[2][1] + off),
                (base[3][0] - off, base[3][1]),
            ]
            d.polygon([(x * S, y * S) for x, y in halo], fill=REEF_HALO)

    # 3) 格底色（陆/宝/礁）
    for r in range(g):
        for c in range(g):
            code = code_at(r, c)
            if code == "S":
                continue
            poly = [(x * S, y * S) for x, y in _diamond(fit, r, c)]
            if code == "T":
                d.polygon(poly, fill=TREASURE)
            elif code == "R":
                d.polygon(poly, fill=REEF)
            else:
                d.polygon(poly, fill=SAND)
                # 陆内渐变高光（顶半更亮）
                sx, sy = _diamond(fit, r, c)[0]
                d.polygon(
                    [
                        ((sx + fit["tw"]) * S, sy * S),
                        ((sx + 2 * fit["tw"]) * S, (sy + fit["th"]) * S),
                        ((sx + fit["tw"]) * S, (sy + 2 * fit["th"]) * S),
                        ((sx) * S, (sy + fit["th"]) * S),
                    ],
                    fill=SAND_HI,
                )

    # 4) 墨线海岸（临海边的上两条棱）
    for r in range(g):
        for c in range(g):
            if code_at(r, c) == "S":
                continue
            sx, sy = _diamond(fit, r, c)[0]
            tw, th = fit["tw"], fit["th"]
            top = ((sx + tw) * S, sy * S)
            left = ((sx) * S, (sy + th) * S)
            right = ((sx + 2 * tw) * S, (sy + th) * S)
            if r > 0 and c > 0 and (code_at(r - 1, c) == "S" or code_at(r, c - 1) == "S"):
                d.line([left, top], fill=INK, width=6 * S // 4)
                d.line([top, right], fill=INK, width=6 * S // 4)

    # 5) 确定性纹理短划（FNV；禁 RNG ⇒ 快照可复现）
    for r in range(g):
        for c in range(g):
            if code_at(r, c) not in ("L", "T"):
                continue
            for k in (0, 1):
                h = _fnv((r * 7919 + c * 104729) ^ (0x5DEECE66D if k else 0x1234))
                sx, sy = _diamond(fit, r, c)[0]
                tw, th = fit["tw"], fit["th"]
                ix = sx + tw + (h % max(1, (2 * tw - 10)))
                iy = sy + th + ((h >> 8) % max(1, th))
                d.line([(ix * S, iy * S), ((ix + 5) * S, iy * S)], fill=TEX, width=2 * S // 4)

    out = img.resize((gw, gh), Image.LANCZOS)
    out.save(out_path)
    return out.size


def generate():
    terrain = _load_json("assets/data/terrain.json")
    scenes = _load_json("assets/data/scenes.json")
    world_canvas = scenes["main_map"]["world_canvas"]
    legend = terrain.get("legend", {})
    manifest_path = os.path.join(ROOT, "assets", "ui_manifest.json")
    manifest = _load_json("assets/ui_manifest.json")
    entries = manifest if isinstance(manifest, list) else manifest.get("entries", manifest.get("assets"))

    ok = 0
    for mid, m in terrain["maps"].items():
        g = len(m["rows"])
        fit = _board_fit(world_canvas, g)
        if fit is None:
            print(f"FAIL {mid}: world_canvas 放不下 g={g}")
            return 1
        out = os.path.join(ROOT, "assets", "ui", f"island_{mid}.png")
        size = render_island(m["rows"], legend, fit, out)
        # 登记 ui_manifest.json（幂等 upsert）
        rec = {"name": f"island_{mid}", "role": "island", "file": f"assets/ui/island_{mid}.png",
               "params": {"map": mid, "grid": g, "canvas": f"{fit['gw']}x{fit['gh']}"},
               "generated_at": "2026-10-26", "generator": "scripts/ui_islands.py"}
        names = [e.get("name") for e in entries]
        if rec["name"] in names:
            entries[names.index(rec["name"])] = rec
        else:
            entries.append(rec)
        print(f"[ok] {mid} g={g} -> assets/ui/island_{mid}.png {size[0]}x{size[1]}")
        ok += 1

    if isinstance(manifest, list):
        io.open(manifest_path, "w", encoding="utf-8").write(json.dumps(manifest, ensure_ascii=False, indent=1) + "\n")
    else:
        io.open(manifest_path, "w", encoding="utf-8").write(json.dumps(manifest, ensure_ascii=False, indent=1) + "\n")
    print(f"ui_islands: {ok} 张已生成 + ui_manifest.json 已登记")
    return 0 if ok else 1


def check():
    """只读模式：3 张在位 + 尺寸 = world_canvas + manifest 登记齐全；不齐 = 非 0 退出。"""
    terrain = _load_json("assets/data/terrain.json")
    scenes = _load_json("assets/data/scenes.json")
    world_canvas = scenes["main_map"]["world_canvas"]
    manifest = _load_json("assets/ui_manifest.json")
    names = [e.get("name") for e in (manifest if isinstance(manifest, list) else manifest.get("entries", manifest.get("assets")))]
    bad = 0
    for mid, m in terrain["maps"].items():
        p = os.path.join(ROOT, "assets", "ui", f"island_{mid}.png")
        if not os.path.isfile(p):
            print(f"FAIL {mid}: 缺 assets/ui/island_{mid}.png")
            bad += 1
            continue
        from PIL import Image
        im = Image.open(p)
        want = (world_canvas["w"], world_canvas["h"])
        if im.size != want or im.mode != "RGBA":
            print(f"FAIL {mid}: 尺寸/模式 {im.size} {im.mode}（期望 {want} RGBA）")
            bad += 1
            continue
        if f"island_{mid}" not in names:
            print(f"FAIL {mid}: ui_manifest.json 未登记 island_{mid}")
            bad += 1
            continue
        print(f"[ok] island_{mid}: {im.size[0]}x{im.size[1]} RGBA + 已登记")
    if bad:
        print(f"ui_islands check: {bad} 项不齐")
        return 1
    print("ui_islands check: 0 不齐")
    return 0


if __name__ == "__main__":
    sys.exit(check() if "--check" in sys.argv else generate())
