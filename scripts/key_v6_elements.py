#!/usr/bin/env python3
"""key_v6_elements.py — main_map v6 / VS v2 元素去白背景键控 → assets/ui/（引擎 blit 用）。

AI 生成元素是白底；引擎 `blit_image` 不抠白 ⇒ 需预键控透明 PNG（flood 去白，保内部白）。
源: evidence/ui_concept/{main_map_v6,vs_anim_v2}/elements/*.png
出: assets/ui/v6/*.png（main_map）+ assets/ui/vs_v2/*.png（VS）

用法: python3 scripts/key_v6_elements.py [--check]
"""
from __future__ import annotations
import argparse
import os
import sys

from PIL import Image, ImageFilter


def flood_white_bg(img: Image.Image, thresh: int = 232,
                   clear_enclosed: bool = False,
                   parchment_key: bool = False) -> Image.Image:
    import numpy as np
    src = img.convert("RGBA")
    a = np.asarray(src).astype(np.int16)
    if parchment_key:
        # 羊皮黄底键控（2026-10-31 中央岛改羊皮手绘风）：判定 = 暖黄/羊皮/灰背景
        # （亮度够 + 非深色），从四边连通 flood，保留岛内部棕榈/灯塔/宝箱（深色描边 R<140 保留）。
        # 背景 = 羊皮黄（R-B>10）或灰角（R-B<10 但 R>140/G>120）；岛主体深色 R<140 不判。
        near_white = (
            (a[..., 0] > 140) & (a[..., 1] > 120) & (a[..., 2] > 40)
        )
    else:
        near_white = (a[..., 0] >= thresh) & (a[..., 1] >= thresh) & (a[..., 2] >= thresh)
    h, w = near_white.shape
    bg = np.zeros((h, w), dtype=bool)
    from collections import deque
    dq = deque()
    for x in range(w):
        for y in (0, h - 1):
            if near_white[y, x] and not bg[y, x]:
                bg[y, x] = True
                dq.append((y, x))
    for y in range(h):
        for x in (0, w - 1):
            if near_white[y, x] and not bg[y, x]:
                bg[y, x] = True
                dq.append((y, x))
    while dq:
        y, x = dq.popleft()
        for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
            if 0 <= ny < h and 0 <= nx < w and near_white[ny, nx] and not bg[ny, nx]:
                bg[ny, nx] = True
                dq.append((ny, nx))
    # 2026-10-31 用户「整体重做背景/角色」→ 键控后 enemy/hero 腿间/身侧仍留被包围白块
    # （大腿+裤+衣摆围出的背景空隙不连通四边，flood 到不了）。骷髅牙/帆/灯塔等内部高光
    # 是小面积近白，需保留 ⇒ 只透明化「被包围近白大块」（面积阈值区分）。
    # 仅 `clear_enclosed=True`（VS 角色）启用：main_map v6 元素（island 帆/灯塔/骷髅旗 /
    # creatures 高光）内部白是大面积真实内容，禁用避免误透明。
    if clear_enclosed:
        MIN_AREA = 2500
        # 深色像素（R<130 三通道均值低）= 墨线/内容；背景缝无深色 ⇒ 判为背景可透明。
        dark = (a[..., 0] < 130) & (a[..., 1] < 110) & (a[..., 2] < 90)
        visited = np.zeros((h, w), dtype=bool)
        for y0 in range(h):
            for x0 in range(w):
                if not near_white[y0, x0] or bg[y0, x0] or visited[y0, x0]:
                    continue
                q = deque([(y0, x0)])
                visited[y0, x0] = True
                pts = []
                touches_edge = False
                dark_count = 0
                while q:
                    cy, cx = q.popleft()
                    pts.append((cy, cx))
                    if dark[cy, cx]:
                        dark_count += 1
                    if cy in (0, h - 1) or cx in (0, w - 1):
                        touches_edge = True
                    for ny, nx in ((cy - 1, cx), (cy + 1, cx), (cy, cx - 1), (cy, cx + 1)):
                        if 0 <= ny < h and 0 <= nx < w and near_white[ny, nx] and not bg[ny, nx] and not visited[ny, nx]:
                            visited[ny, nx] = True
                            q.append((ny, nx))
                # 被包围 + 面积够 + 深色占比低（背景缝无墨线）⇒ 透明
                if not touches_edge and len(pts) >= MIN_AREA and dark_count < len(pts) * 0.02:
                    for (py, px) in pts:
                        bg[py, px] = True
    out = np.array(src, copy=True)
    out[bg, 3] = 0
    res = Image.fromarray(out, "RGBA")
    res.putalpha(res.getchannel("A").filter(ImageFilter.GaussianBlur(1.2)))
    return res


def fit_crop_white(im: Image.Image) -> Image.Image:
    import numpy as np
    a = np.asarray(im.convert("RGBA"))
    alpha = a[..., 3] > 8
    if not alpha.any():
        return im
    ys, xs = np.where(alpha)
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    return im.crop((max(0, x0 - 4), max(0, y0 - 4), min(im.width, x1 + 5), min(im.height, y1 + 5)))


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MAP_SRC = os.path.join(ROOT, "evidence", "ui_concept", "main_map_v6", "elements")
VS_SRC = os.path.join(ROOT, "evidence", "ui_concept", "vs_anim_v2", "elements")
MAP_DST = os.path.join(ROOT, "assets", "ui", "v6")
VS_DST = os.path.join(ROOT, "assets", "ui", "vs_v2")

MAP_ITEMS = ["tile_owned", "tile_deny", "island", "isle_land", "isle_lighthouse",
             "isle_treasure", "isle_village", "creatures", "helm", "compass", "ship"]
VS_ITEMS = ["hero", "enemy", "crew", "banner_win", "banner_lose"]
# 满屏背景不做键控
BG_ITEMS = ["ocean_bg", "parchment", "vs_bg", "bg_siege", "bg_dungeon", "bg_boss"]


def key(src_dir: str, dst_dir: str, items: list[str], copy_bg: list[str],
        enclosed_items: list[str] | None = None,
        parchment_items: list[str] | None = None) -> list[str]:
    os.makedirs(dst_dir, exist_ok=True)
    enclosed = set(enclosed_items or [])
    parch = set(parchment_items or [])
    out = []
    for f in items:
        s = os.path.join(src_dir, f"{f}.png")
        if not os.path.exists(s):
            continue
        im = Image.open(s).convert("RGB")
        im = fit_crop_white(flood_white_bg(im, clear_enclosed=f in enclosed,
                                           parchment_key=f in parch))
        d = os.path.join(dst_dir, f"{f}.png")
        im.save(d)
        out.append(d)
    for f in copy_bg:
        s = os.path.join(src_dir, f"{f}.png")
        if not os.path.exists(s):
            continue
        d = os.path.join(dst_dir, f"{f}.png")
        Image.open(s).convert("RGB").save(d)
        out.append(d)
    return out


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args()
    map_dst_files = [os.path.join(MAP_DST, f"{f}.png") for f in MAP_ITEMS + BG_ITEMS]
    vs_dst_files = [os.path.join(VS_DST, f"{f}.png") for f in VS_ITEMS + BG_ITEMS]
    all_files = map_dst_files + vs_dst_files
    missing = [p for p in all_files if not os.path.exists(p)]
    if args.check:
        print(f"[key] check: {'ok' if not missing else 'missing ' + str(missing)}")
        return 0 if not missing else 1
    a = key(MAP_SRC, MAP_DST, MAP_ITEMS, BG_ITEMS,
            enclosed_items=["island"])
    b = key(VS_SRC, VS_DST, VS_ITEMS, BG_ITEMS, enclosed_items=["hero", "enemy", "crew"])
    print(f"[key] wrote {len(a)} main_map + {len(b)} vs assets")
    return 0


if __name__ == "__main__":
    sys.exit(main())
