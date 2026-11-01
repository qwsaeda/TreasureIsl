#!/usr/bin/env python3
"""compose_main_map_v6.py — main_map v6「古典羊皮航海图」元素合成设计工具.

用户拍板「元素级生成 + 组合合成」：先 AI 生成各元素（海洋全景底/羊皮框/
铜牌×2/中央岛/舵轮/罗盘/帆船），再用 PIL 按规格组合成完整游戏界面设计图。
AI 给画质（海洋/做旧），PIL 给精度（31 格环 + 每格随机旋转/缩放/可占领随机）。

元素源: evidence/ui_concept/main_map_v6/elements/*.png（白底 AI 元素 + ocean_bg 全景底）
输出:   evidence/ui_concept/main_map_v6/main_map_v6_composite.png

用法:
    python3 scripts/compose_main_map_v6.py [--out PATH] [--seed N] [--owned-ratio 0.62]
    python3 scripts/compose_main_map_v6.py --check    # 只读：元素齐备 + 输出存在

工具契约（scripts/ 正式工具）：幂等（--seed 固定则输出字节一致）+ --check 只读。
"""
from __future__ import annotations
import argparse
import math
import os
import random
import sys

from PIL import Image, ImageDraw, ImageFilter, ImageOps

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ELEM = os.path.join(ROOT, "evidence", "ui_concept", "main_map_v6", "elements")
OUT = os.path.join(ROOT, "evidence", "ui_concept", "main_map_v6", "main_map_v6_composite.png")

# 画布 = 游戏逻辑分辨率 1080×1920（竖屏）
W, H = 1080, 1920

# 布局参数：31 格（start1 + territory + function + boss 同 S31 不变量）
# 2026-10-31 用户「不规则岛，格子紧围岛不规则排列」→ 推翻环形赛道 + 圆岛：
# 岛 = 不规则自然岛形（非圆盘）；31 格沿岛轮廓凸包弧长等分 + 径向贴沿 = 紧围岛不规则排布。
N_TILES = 31
SCREEN_CX, SCREEN_CY = 540, 890
ISLAND_W = 780        # 岛显示宽度（px，视觉焦点，用户确认尺寸）
TILE_R = 36           # 格显示半径（px，木片直径 72）
TILE_GAP = 20         # 格与岛轮廓间隙（px，贴沿留呼吸缝）

# 羊皮纸边框宽（外沿做旧）
FRAME_W = 56


def flood_white_bg(img: Image.Image, thresh: int = 232) -> Image.Image:
    """去掉与四边连通的白背景，保留内部白（骷髅牙/帆/灯塔等）。"""
    import numpy as np
    src = img.convert("RGBA")
    a = np.asarray(src).astype(np.int16)
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
    out = np.array(src, copy=True)
    out[bg, 3] = 0
    res = Image.fromarray(out, "RGBA")
    alpha = res.getchannel("A").filter(ImageFilter.GaussianBlur(1.2))
    res.putalpha(alpha)
    return res


def load_elem(name: str, remove_bg: bool = True) -> Image.Image:
    p = os.path.join(ELEM, f"{name}.png")
    if not os.path.exists(p):
        raise FileNotFoundError(f"缺元素: {p}")
    im = Image.open(p).convert("RGB")
    return flood_white_bg(im) if remove_bg else im


def fit_crop_white(im: Image.Image) -> Image.Image:
    """裁掉白边 + 返回含透明 alpha 的紧贴元素。"""
    import numpy as np
    a = np.asarray(im.convert("RGBA"))
    alpha = a[..., 3] > 8
    if not alpha.any():
        return im
    ys, xs = np.where(alpha)
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    return im.crop((max(0, x0 - 4), max(0, y0 - 4), min(im.width, x1 + 5), min(im.height, y1 + 5)))


def place(im: Image.Image, target_w: int, rot: float, flip_h: bool = False) -> Image.Image:
    """缩放 + 旋转 + 可选水平翻转 → RGBA（紧贴内容）。"""
    im = fit_crop_white(im)
    r = target_w / im.width
    im = im.resize((max(1, int(im.width * r)), max(1, int(im.height * r))), Image.LANCZOS)
    if flip_h:
        im = ImageOps.mirror(im)
    if rot:
        im = im.rotate(rot, expand=True, resample=Image.BICUBIC)
    return im


def desaturate_dark(im: Image.Image) -> Image.Image:
    """不可占领 = 灰暗牌：去饱和 + 冷灰，比可占领明显暗但可辨细节。"""
    im = im.convert("RGBA")
    gray = ImageOps.grayscale(im)
    gray = ImageOps.colorize(gray, black=(52, 55, 60), white=(150, 154, 160))
    gray = Image.merge("RGBA", (*gray.split()[:3], im.getchannel("A")))
    return Image.composite(gray, im, im.getchannel("A"))


def draw_parchment_frame(base: Image.Image) -> None:
    """外沿羊皮纸做旧边框：暖黄米色 + 纤维噪点 + 边缘暗角 + 木框。

    在 base 四边画厚羊皮纸带（内边 = 海洋矩形留 FRAME_W 后收进）。
    """
    import numpy as np
    d = ImageDraw.Draw(base)
    # 羊皮纸色：暖黄米色多段渐变（由外到内略亮）
    for i, col in enumerate([(158, 122, 66), (182, 146, 88), (200, 168, 110),
                             (214, 186, 130), (224, 200, 150), (230, 212, 166)]):
        inset = FRAME_W - i * (FRAME_W // 6)
        d.rectangle((inset // 2, inset // 2, W - 1 - inset // 2, H - 1 - inset // 2),
                    outline=col, width=max(2, (FRAME_W // 6) + 2))
    # 纤维噪点（只在边框带内）
    rng = np.random.RandomState(11)
    for _ in range(1600):
        x, y = rng.randint(0, W), rng.randint(0, H)
        if min(x, W - x, y, H - y) < FRAME_W:
            r = int(rng.randint(1, 4))
            d.ellipse((x, y, x + r, y + r), fill=(int(rng.randint(96, 216)),
                                                  int(rng.randint(76, 190)),
                                                  int(rng.randint(40, 140))))
    # 暗角（内缘阴影，做旧）
    for i, col in enumerate([(0, 0, 0, 70), (0, 0, 0, 46), (0, 0, 0, 24)]):
        inset = FRAME_W + i * 6
        d.rectangle((inset, inset, W - 1 - inset, H - 1 - inset), outline=col, width=4)
    # 木框最外沿
    d.rectangle((0, 0, W - 1, H - 1), outline=(40, 26, 14, 255), width=9)
    d.rectangle((9, 9, W - 10, H - 10), outline=(66, 44, 24, 255), width=3)


def draw_ocean_inset(base: Image.Image, ob: Image.Image) -> None:
    """海洋全景底裁进羊皮纸框内（四周留 FRAME_W 边）。"""
    iw, ih = W - 2 * FRAME_W, H - 2 * FRAME_W
    ow, oh = ob.size
    scale = max(iw / ow, ih / oh)
    ob = ob.resize((int(ow * scale), int(oh * scale)), Image.LANCZOS)
    ox = (ob.width - iw) // 2
    oy = int((ob.height - ih) * 0.10)
    ob = ob.crop((ox, oy, ox + iw, oy + ih))
    base.paste(ob, (FRAME_W, FRAME_W))


def tile_positions_around_island(island_png: str, n: int,
                                 tile_r: int, gap: int,
                                 island_w: int) -> tuple[list[tuple[float, float]], int]:
    """31 格紧围岛不规则排列：岛轮廓凸包弧长等分 + 径向外推贴沿。

    2026-10-31 用户「不规则岛，格子紧围岛不规则排列」：岛 = 不规则自然岛形，
    格子沿岛轮廓排布（非规则环）。实现 = 读**键控后**岛 alpha 掩码 → 逐 0.5° 求
    轮廓半径 → Andrew monotone chain 凸包（消除海湾内凹，格子横跨海湾口围住岛）→
    凸包弧长等分 n 段 → 每格沿径向（质心→轮廓点）外推**显示 (tile_r+gap)px** 贴沿。

    外推量必须换算到元素px（/scale），否则折成显示后 < 格半径 ⇒ 格子叠岛
    （审计 2026-10-31 发现：56 元素px 只折 23.9 显示px，格半径 36 ⇒ 侵入岛 12px）。

    返回 (格中心列表相对裁剪岛图质心[元素px], 裁剪后岛图宽[px])。
    判据：相邻格直线距离 >= 2*tile_r + 6（不重叠 + 留缝）。
    """
    import numpy as np
    im = Image.open(island_png).convert("RGBA")
    im = fit_crop_white(flood_white_bg(im))   # 与 load_elem 同源同键控（清白底）+ 裁剪
    a = np.asarray(im)
    alpha = a[..., 3]
    h, w = alpha.shape
    ys, xs = np.where(alpha > 10)
    cx, cy = xs.mean(), ys.mean()
    scale = island_w / w          # 岛显示宽 / 裁剪岛宽
    ext = (tile_r + gap) / scale  # 径向外推（元素px，折显示 = tile_r+gap）
    # 逐 0.5° 轮廓半径（质心→最后不透明像素）
    raw = []
    for k in range(720):
        ang = k * 0.5
        th = math.radians(ang)
        dx, dy = math.cos(th), math.sin(th)
        rmax = 0
        for r in range(0, max(w, h)):
            x = int(cx + dx * r)
            y = int(cy + dy * r)
            if 0 <= x < w and 0 <= y < h and alpha[y, x] > 10:
                rmax = r
        raw.append((cx + rmax * dx, cy + rmax * dy))

    # Andrew monotone chain 凸包
    def cross(o, aa, bb):
        return (aa[0] - o[0]) * (bb[1] - o[1]) - (aa[1] - o[1]) * (bb[0] - o[0])

    pts = sorted(set(raw))
    lower = []
    for p in pts:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], p) <= 0:
            lower.pop()
        lower.append(p)
    upper = []
    for p in reversed(pts):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], p) <= 0:
            upper.pop()
        upper.append(p)
    hull = lower[:-1] + upper[:-1]

    # 凸包累计弧长
    cum = [0.0]
    for i in range(len(hull)):
        cum.append(cum[-1] + math.dist(hull[i], hull[(i + 1) % len(hull)]))
    total = cum[-1]

    def at(s: float):
        s = s % total
        lo, hi = 0, len(hull)
        while lo < hi:
            mid = (lo + hi) // 2
            if cum[mid] <= s:
                lo = mid + 1
            else:
                hi = mid
        i = (lo - 1) % len(hull)
        p0, p1 = hull[i], hull[(i + 1) % len(hull)]
        seg = cum[i + 1] - cum[i] if cum[i + 1] > cum[i] else 1e-9
        t = (s - cum[i]) / seg
        return (p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t)

    # 弧长等分 n 段，每格沿径向外推贴沿（相对质心）
    pos = []
    for i in range(n):
        p = at(i * total / n)
        dx, dy = p[0] - cx, p[1] - cy
        L = math.hypot(dx, dy) or 1.0
        ux, uy = dx / L, dy / L
        pos.append((p[0] - cx + ux * ext,
                    p[1] - cy + uy * ext))
    # 判据：相邻格直线距离 >= 2*tile_r + 6
    min_d = min(math.dist(pos[i], pos[(i + 1) % n]) for i in range(n))
    if min_d < 2 * tile_r + 6:
        raise ValueError(f"格子紧围岛重叠：相邻最小弦 {min_d:.0f} < 需 {2 * tile_r + 6}")
    return pos, w


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=OUT)
    ap.add_argument("--seed", type=int, default=20261031)
    ap.add_argument("--owned-ratio", type=float, default=0.62)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--export-layout", default=None, metavar="JSON",
                    help="导出 31 格布局 JSON（Rust `render_v6_elem` 数据驱动读取，不硬编码坐标）")
    args = ap.parse_args()

    need = ["ocean_bg", "tile_owned", "tile_deny", "island",
            "helm", "compass", "ship"]
    missing = [n for n in need if not os.path.exists(os.path.join(ELEM, f"{n}.png"))]
    if missing:
        print(f"[compose] MISSING elements: {missing}", file=sys.stderr)
        return 2
    if args.check:
        ok = os.path.exists(args.out)
        print(f"[compose] check: out={'ok' if ok else 'missing'} seed={args.seed}")
        return 0 if ok else 1

    random.seed(args.seed)
    base = Image.new("RGBA", (W, H), (0, 0, 0, 0))

    # 1) 羊皮纸做旧边框（先画，海洋覆盖内区）
    draw_parchment_frame(base)

    # 2) 海洋全景底（裁进框内）
    ob = Image.open(os.path.join(ELEM, "ocean_bg.png")).convert("RGB")
    draw_ocean_inset(base, ob)

    # 3) 31 格紧围岛不规则排列（凸包弧长等分 + 贴沿；岛质心对齐屏幕中心）
    island_src = os.path.join(ELEM, "island.png")
    island = place(load_elem("island"), ISLAND_W, 0)
    # 岛质心在元素坐标 → 屏幕中心
    import numpy as np
    ia = np.asarray(island.convert("RGBA"))
    iys, ixs = np.where(ia[..., 3] > 10)
    i_cx, i_cy = ixs.mean(), iys.mean()
    island_x = int(SCREEN_CX - i_cx)
    island_y = int(SCREEN_CY - i_cy)
    # 格中心（相对裁剪岛图质心，裁剪岛图宽）+ 显示缩放
    tile_pos, crop_w = tile_positions_around_island(island_src, N_TILES, TILE_R, TILE_GAP, ISLAND_W)
    scale = ISLAND_W / crop_w
    # 3a) 格间缆绳路径（沿格子顺序，先画压在格下；不规则连线 = 前进路线）
    d = ImageDraw.Draw(base)
    rope_pts = [(SCREEN_CX + px * scale, SCREEN_CY + py * scale) for (px, py) in tile_pos]
    for i in range(N_TILES):
        p0 = rope_pts[i]
        p1 = rope_pts[(i + 1) % N_TILES]
        d.line((p0[0], p0[1], p1[0], p1[1]), fill=(74, 52, 28, 255), width=13)
        d.line((p0[0], p0[1], p1[0], p1[1]), fill=(102, 72, 40, 255), width=8)
    # 3b) 中央岛（质心对齐屏幕中心）
    base.paste(island, (island_x, island_y), island)

    # 3c) 31 格木片（随机可占领/不可占领 + 每格旋转/缩放差异，贴轮廓不规则排布）
    owned = place(load_elem("tile_owned"), TILE_R * 2, 0)
    deny = desaturate_dark(place(load_elem("tile_deny"), TILE_R * 2, 0))
    n_owned = int(N_TILES * args.owned_ratio)
    kinds = [True] * n_owned + [False] * (N_TILES - n_owned)
    random.shuffle(kinds)
    for i, is_owned in enumerate(kinds):
        px, py = tile_pos[i]
        tx = SCREEN_CX + px * scale
        ty = SCREEN_CY + py * scale
        scale_i = random.uniform(0.94, 1.06)
        rot = random.uniform(-10, 10)
        tile = place(owned if is_owned else deny, TILE_R * 2 * scale_i, rot)
        base.paste(tile, (int(tx - tile.width / 2), int(ty - tile.height / 2)), tile)

    # 4) 玩家帆船（停岛侧海面，醒目不遮邻格；船右移 300 避格 14）
    ship = place(load_elem("ship"), 150, 0)
    base.paste(ship, (int(SCREEN_CX + 300 - ship.width / 2),
                      int(SCREEN_CY - 30 - ship.height / 2)), ship)

    # 5) 底部舵轮 CTA + 罗盘装饰
    helm = place(load_elem("helm"), 230, 0)
    base.paste(helm, (int(W * 0.27 - helm.width / 2), int(H - 180)), helm)
    compass = place(load_elem("compass"), 205, 0)
    base.paste(compass, (int(W * 0.73 - compass.width / 2), int(H - 178)), compass)

    base = base.convert("RGB")
    base.save(args.out)
    print(f"[compose] saved {args.out} ({base.size}) seed={args.seed} "
          f"owned={n_owned}/{N_TILES}")

    # 布局 JSON 导出（Rust `render_v6_elem` 数据驱动；单一事实源 = 本脚本算法）
    if args.export_layout:
        import json
        rope_pts_disp = [(round(x, 1), round(y, 1)) for (x, y) in rope_pts]
        tile_centers = [(round(SCREEN_CX + px * scale, 1),
                         round(SCREEN_CY + py * scale, 1)) for (px, py) in tile_pos]
        layout = {
            "schema": "main_map_v6_layout/1",
            "screen": [W, H],
            "seed": args.seed,
            "tile_r": TILE_R,
            "island": {"cx": SCREEN_CX, "cy": SCREEN_CY, "w": ISLAND_W},
            "ship": {"cx": SCREEN_CX + 300, "cy": SCREEN_CY - 30, "w": 150},
            "helm": {"cx": int(W * 0.27), "cy": H - 180, "w": 230},
            "compass": {"cx": int(W * 0.73), "cy": H - 178, "w": 205},
            "tiles": [
                {"index": i, "cx": cx, "cy": cy, "owned": bool(kinds[i])}
                for i, (cx, cy) in enumerate(tile_centers)
            ],
            "rope": rope_pts_disp,
        }
        with open(args.export_layout, "w", encoding="utf-8") as f:
            json.dump(layout, f, ensure_ascii=False, indent=1)
        print(f"[compose] exported layout → {args.export_layout}（{len(tile_centers)} 格）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
