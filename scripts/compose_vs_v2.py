#!/usr/bin/env python3
"""compose_vs_v2.py — VS 战斗界面 v2「重设计」元素合成设计工具.

用户 2026-10-31 否决旧 VS 概念稿（AI 插画海报）→ 按 `12_main_map_v6_vs_v2_spec.md` §2 重设计
（参考 FGO/宝可梦/Pirates/黑旗/炉石：敌我分列 + 顶/底状态框 + 中央战场 + 结果横幅）。
元素级生成 + 组合合成（同 main_map v6 方法论）。

元素源: evidence/ui_concept/vs_anim_v2/elements/*.png（hero/enemy/crew/vs_bg/banner_win/banner_lose）
输出:   evidence/ui_concept/vs_anim_v2/vs_v2_{clash,duel,result_win,result_lose}.png

用法:
    python3 scripts/compose_vs_v2.py [--seed N] [--atk-crew 30] [--atk-arms 18] ...
    python3 scripts/compose_vs_v2.py --check    # 只读：元素齐备

工具契约：幂等（--seed 固定则输出字节一致）+ --check 只读。
"""
from __future__ import annotations
import argparse
import math
import os
import random
import sys

from PIL import Image, ImageDraw, ImageFilter, ImageOps

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ELEM = os.path.join(ROOT, "evidence", "ui_concept", "vs_anim_v2", "elements")
OUTDIR = os.path.join(ROOT, "evidence", "ui_concept", "vs_anim_v2")

W, H = 1080, 1920

# 兵力模型（combat.md）：武装 = min(人手, 武器)
WEAPON_COEF = 2.0
BAREHAND_COEF = 0.5


def flood_white_bg(img: Image.Image, thresh: int = 232) -> Image.Image:
    """去与四边连通白背景（保留内部白）。"""
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
    res.putalpha(res.getchannel("A").filter(ImageFilter.GaussianBlur(1.2)))
    return res


def load_elem(name: str, remove_bg: bool = True) -> Image.Image:
    p = os.path.join(ELEM, f"{name}.png")
    if not os.path.exists(p):
        raise FileNotFoundError(f"缺元素: {p}")
    im = Image.open(p).convert("RGB")
    return flood_white_bg(im) if remove_bg else im


def fit_crop_white(im: Image.Image) -> Image.Image:
    import numpy as np
    a = np.asarray(im.convert("RGBA"))
    alpha = a[..., 3] > 8
    if not alpha.any():
        return im
    ys, xs = np.where(alpha)
    x0, x1, y0, y1 = xs.min(), xs.max(), ys.min(), ys.max()
    return im.crop((max(0, x0 - 4), max(0, y0 - 4), min(im.width, x1 + 5), min(im.height, y1 + 5)))


def place(im: Image.Image, target_w: int, rot: float = 0, flip_h: bool = False) -> Image.Image:
    im = fit_crop_white(im)
    r = target_w / im.width
    im = im.resize((max(1, int(im.width * r)), max(1, int(im.height * r))), Image.LANCZOS)
    if flip_h:
        im = ImageOps.mirror(im)
    if rot:
        im = im.rotate(rot, expand=True, resample=Image.BICUBIC)
    return im


def paste_center(base: Image.Image, im: Image.Image, cx: int, cy: int) -> None:
    base.paste(im, (int(cx - im.width / 2), int(cy - im.height / 2)), im)


def calc_power(crew: int, arms: int) -> tuple[int, int, int, int]:
    """武装/徒手/战力/武装数。"""
    armed = min(crew, arms)
    unarmed = crew - armed
    power = armed * WEAPON_COEF + unarmed * BAREHAND_COEF
    return armed, unarmed, int(power), armed


def draw_power_bar(base: Image.Image, x: int, y: int, w: int, h: int,
                   frac: float, color: tuple) -> None:
    """兵力条（黄铜槽 + 填充）。"""
    d = ImageDraw.Draw(base)
    # 槽
    d.rounded_rectangle((x, y, x + w, y + h), radius=h // 2, fill=(40, 30, 20, 200),
                        outline=(180, 140, 80, 255), width=3)
    # 填充
    fw = int((w - 8) * max(0.0, min(1.0, frac)))
    if fw > 0:
        d.rounded_rectangle((x + 4, y + 4, x + 4 + fw, y + h - 4), radius=(h - 8) // 2, fill=color)


def draw_status_box(base: Image.Image, x: int, y: int, w: int, h: int,
                    name: str, crew: int, arms: int, side: str) -> None:
    """顶/底状态框：名牌 + 人手/武器/武装 + 兵力条。side='def'顶部绿 / 'atk'底部红。"""
    d = ImageDraw.Draw(base)
    # 底框（黄铜羊皮面板）
    d.rounded_rectangle((x, y, x + w, y + h), radius=16, fill=(222, 198, 148, 235),
                        outline=(120, 90, 50, 255), width=3)
    d.rounded_rectangle((x + 6, y + 6, x + w - 6, y + h - 6), radius=12,
                        outline=(90, 66, 36, 120), width=2)
    armed, unarmed, power, _ = calc_power(crew, arms)
    # 名牌
    d.text((x + 18, y + 12), name, fill=(60, 40, 20))
    # 兵力条
    bar_x, bar_y = x + 18, y + h - 40
    bar_w = w - 36
    color = (60, 170, 90) if side == "def" else (200, 70, 50)
    frac = crew / 40.0
    draw_power_bar(base, bar_x, bar_y, bar_w, 22, frac, color)
    # 数值
    d.text((x + 18, y + 38), f"人手 {crew} / 武器 {arms} / 武装 {armed}", fill=(70, 50, 26))


def draw_hud(base: Image.Image, atk_crew: int, atk_arms: int,
             def_crew: int, def_arms: int, atk_name: str = "我方舰队",
             def_name: str = "敌方驻军") -> None:
    """顶（守方）/ 底（攻方）状态框 + 中央 VS 徽标。"""
    # 顶部守方框（右对齐）
    draw_status_box(base, W - 470, 60, 440, 130, def_name, def_crew, def_arms, "def")
    # 底部攻方框（左对齐）
    draw_status_box(base, 30, H - 190, 440, 130, atk_name, atk_crew, atk_arms, "atk")
    # 中央 VS 徽标
    d = ImageDraw.Draw(base)
    d.ellipse((W // 2 - 70, H // 2 - 70, W // 2 + 70, H // 2 + 70), fill=(60, 40, 20, 220),
              outline=(200, 160, 90, 255), width=5)
    d.text((W // 2 - 34, H // 2 - 26), "VS", fill=(240, 210, 150))


def draw_battle_frame(base: Image.Image, atk_crew: int, atk_arms: int,
                      def_crew: int, def_arms: int, bg_elem: str = "vs_bg") -> None:
    """通用战斗层：羊皮底框 + 背景（可换战斗类型）+ HUD + 敌我分列站位。"""
    # 羊皮做旧边框
    d = ImageDraw.Draw(base)
    for i, col in enumerate([(158, 122, 66), (182, 146, 88), (200, 168, 110),
                             (214, 186, 130), (224, 200, 150), (230, 212, 166)]):
        inset = 44 - i * 7
        d.rectangle((inset // 2, inset // 2, W - 1 - inset // 2, H - 1 - inset // 2),
                    outline=col, width=max(2, 8))
    d.rectangle((0, 0, W - 1, H - 1), outline=(40, 26, 14, 255), width=9)
    d.rectangle((9, 9, W - 10, H - 10), outline=(66, 44, 24, 255), width=3)
    # 背景（收进框）
    bg = load_elem(bg_elem, remove_bg=False)
    iw, ih = W - 2 * 44, H - 2 * 44
    ow, oh = bg.size
    scale = max(iw / ow, ih / oh)
    bg = bg.resize((int(ow * scale), int(oh * scale)), Image.LANCZOS)
    ox = (bg.width - iw) // 2
    oy = (bg.height - ih) // 2
    bg = bg.crop((ox, oy, ox + iw, oy + ih))
    base.paste(bg, (44, 44))
    draw_hud(base, atk_crew, atk_arms, def_crew, def_arms)


def draw_fight_type_frame(base: Image.Image, bg_elem: str, title: str,
                          atk_crew: int, atk_arms: int, def_crew: int, def_arms: int,
                          hero: Image.Image, enemy: Image.Image, crew: Image.Image) -> None:
    """战斗类型分层帧（攻占/副本/BOSS）：专属背景 + 敌我 + 类型标题。

    背景已烘焙战场部队（船队/登陆艇/炮台守军）＝兵对兵层，不再贴 crew 防重复；
    只贴 hero/enemy 双船长作为焦点角色 + HUD + 类型标题。
    """
    draw_battle_frame(base, atk_crew, atk_arms, def_crew, def_arms, bg_elem)
    d = ImageDraw.Draw(base)
    # 类型标题牌（上中，羊皮底）
    d.rounded_rectangle((W // 2 - 260, 190, W // 2 + 260, 275), radius=16,
                        fill=(222, 198, 148, 235), outline=(120, 90, 50, 255), width=4)
    d.text((W // 2 - 110, 220), title, fill=(60, 40, 20))
    # 敌我双船长（左下攻方 hero / 右上守方 enemy）
    paste_center(base, hero, 200, 1600)
    paste_center(base, enemy, 850, 430)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--outdir", default=OUTDIR)
    ap.add_argument("--seed", type=int, default=20261031)
    ap.add_argument("--atk-crew", type=int, default=30)
    ap.add_argument("--atk-arms", type=int, default=18)
    ap.add_argument("--def-crew", type=int, default=20)
    ap.add_argument("--def-arms", type=int, default=12)
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--fight-type", choices=["all", "siege", "dungeon", "boss"],
                    default="all", help="战斗类型分层帧（默认 all=也出基础 4 帧）")
    args = ap.parse_args()

    need = ["hero", "enemy", "crew", "vs_bg", "banner_win", "banner_lose",
            "bg_siege", "bg_dungeon", "bg_boss"]
    missing = [n for n in need if not os.path.exists(os.path.join(ELEM, f"{n}.png"))]
    if missing:
        print(f"[vs] MISSING elements: {missing}", file=sys.stderr)
        return 2
    if args.check:
        ok = all(os.path.exists(os.path.join(args.outdir, f"vs_v2_{f}.png"))
                 for f in ["clash", "duel", "result_win", "result_lose"])
        print(f"[vs] check: out={'ok' if ok else 'missing'} seed={args.seed}")
        return 0 if ok else 1

    random.seed(args.seed)
    ac, aa, dc, da = args.atk_crew, args.atk_arms, args.def_crew, args.def_arms

    hero = place(load_elem("hero"), 300, 0)
    enemy = place(load_elem("enemy"), 300, 0)
    crew = place(load_elem("crew"), 330, 0)
    hero_small = place(load_elem("hero"), 210, 0)
    enemy_small = place(load_elem("enemy"), 210, 0)

    # ---- clash 兵对兵：左下攻方 crew 冲锋，右上守方 enemy 督战 ----
    base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw_battle_frame(base, ac, aa, dc, da)
    paste_center(base, crew, 360, 1200)          # 左下攻方水手方阵
    paste_center(base, hero_small, 200, 1560)    # 攻方英雄领军
    paste_center(base, enemy, 830, 420)          # 右上守方船长
    # 冲锋刀光
    d = ImageDraw.Draw(base)
    for i in range(6):
        ang = random.uniform(0, 360)
        r1, r2 = random.uniform(320, 420), random.uniform(360, 470)
        x1 = W // 2 + r1 * math.cos(math.radians(ang))
        y1 = H // 2 + r1 * math.sin(math.radians(ang))
        x2 = W // 2 + r2 * math.cos(math.radians(ang))
        y2 = H // 2 + r2 * math.sin(math.radians(ang))
        d.line((x1, y1, x2, y2), fill=(255, 240, 200, 180), width=3)
    base.convert("RGB").save(os.path.join(args.outdir, "vs_v2_clash.png"))

    # ---- duel 将对将：左下 hero vs 右上 enemy 刀光冲突 ----
    base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw_battle_frame(base, ac, aa, dc, da)
    paste_center(base, hero, 230, 1290)
    paste_center(base, enemy, 860, 490)
    # 刀光火花（中央冲突点）
    d = ImageDraw.Draw(base)
    cx, cy = W // 2, H // 2
    for i in range(24):
        ang = random.uniform(0, 360)
        r = random.uniform(60, 220)
        x2 = cx + r * math.cos(math.radians(ang))
        y2 = cy + r * math.sin(math.radians(ang))
        d.line((cx, cy, x2, y2), fill=(255, 245, 210, 210), width=3)
    d.ellipse((cx - 60, cy - 60, cx + 60, cy + 60), outline=(255, 240, 190, 230), width=5)
    base.convert("RGB").save(os.path.join(args.outdir, "vs_v2_duel.png"))

    # ---- result_win 胜利 ----
    base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw_battle_frame(base, ac, aa, dc, da)
    # 胜方 hero 举刀（大，居中偏左下）
    paste_center(base, hero, 460, 960)
    # 书法横幅（上中偏下，不遮顶部状态框，倾斜，加羊皮衬底保可读）
    bw = place(load_elem("banner_win"), 520, 0)
    bw = bw.rotate(-12, expand=True, resample=Image.BICUBIC)
    panel = Image.new("RGBA", (bw.width + 60, bw.height + 40), (0, 0, 0, 0))
    pd = ImageDraw.Draw(panel)
    pd.rounded_rectangle((0, 0, panel.width - 1, panel.height - 1), radius=20,
                         fill=(230, 208, 160, 220), outline=(120, 90, 50, 255), width=4)
    panel.paste(bw, (30, 20), bw)
    paste_center(base, panel, W // 2, 560)
    # 战报框（底部中）
    d = ImageDraw.Draw(base)
    d.rounded_rectangle((240, H - 420, W - 240, H - 120), radius=16, fill=(222, 198, 148, 235),
                        outline=(120, 90, 50, 255), width=3)
    d.text((270, H - 390), "战报：我方胜", fill=(60, 40, 20))
    a_power = calc_power(ac, aa)[2]
    d_power = calc_power(dc, da)[2]
    d.text((270, H - 350), f"我方战力 {a_power}（人手 {ac}/武器 {aa}）", fill=(70, 50, 26))
    d.text((270, H - 310), f"敌方战力 {d_power}（人手 {dc}/武器 {da}）", fill=(70, 50, 26))
    d.text((270, H - 270), "双方必损：各损 15% 人手 + 20% 武器", fill=(90, 60, 30))
    d.text((270, H - 230), "「不堪一击的蝼蚁！」", fill=(60, 40, 20))
    base.convert("RGB").save(os.path.join(args.outdir, "vs_v2_result_win.png"))

    # ---- result_lose 败北 ----
    base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
    draw_battle_frame(base, ac, aa, dc, da)
    # 败方 hero 倒下（小，偏左下，低）
    hero_fall = place(load_elem("hero"), 320, 0)
    hero_fall = hero_fall.rotate(70, expand=True, resample=Image.BICUBIC)
    paste_center(base, hero_fall, 240, 1450)
    # 敌方 enemy 胜（大，右上）
    paste_center(base, enemy, 460, 900)
    # 书法横幅（上中偏下，不遮顶部状态框，倾斜，加羊皮衬底保可读）
    bl = place(load_elem("banner_lose"), 520, 0)
    bl = bl.rotate(-12, expand=True, resample=Image.BICUBIC)
    panel = Image.new("RGBA", (bl.width + 60, bl.height + 40), (0, 0, 0, 0))
    pd = ImageDraw.Draw(panel)
    pd.rounded_rectangle((0, 0, panel.width - 1, panel.height - 1), radius=20,
                         fill=(230, 208, 160, 220), outline=(120, 90, 50, 255), width=4)
    panel.paste(bl, (30, 20), bl)
    paste_center(base, panel, W // 2, 560)
    # 战报框
    d = ImageDraw.Draw(base)
    d.rounded_rectangle((240, H - 420, W - 240, H - 120), radius=16, fill=(222, 198, 148, 235),
                        outline=(120, 90, 50, 255), width=3)
    d.text((270, H - 390), "战报：我方败", fill=(60, 40, 20))
    d.text((270, H - 350), f"我方战力 {calc_power(ac, aa)[2]}（人手 {ac}/武器 {aa}）", fill=(70, 50, 26))
    d.text((270, H - 310), f"敌方战力 {calc_power(dc, da)[2]}（人手 {dc}/武器 {da}）", fill=(70, 50, 26))
    d.text((270, H - 270), "双方必损：各损 15% 人手 + 20% 武器", fill=(90, 60, 30))
    d.text((270, H - 230), "「愚蠢的蝼蚁，大海才是你们的葬身之地！」", fill=(60, 40, 20))
    base.convert("RGB").save(os.path.join(args.outdir, "vs_v2_result_lose.png"))

    # ---- 战斗类型分层（§2.4：攻占/副本/BOSS）----
    if args.fight_type in ("all", "siege", "dungeon", "boss"):
        types = {
            "siege": ("bg_siege", "攻占战：船队攻打建筑", "vs_v2_siege.png"),
            "dungeon": ("bg_dungeon", "副本战：登岛打守军", "vs_v2_dungeon.png"),
            "boss": ("bg_boss", "BOSS 战：英雄 vs 要塞炮台", "vs_v2_boss.png"),
        }
        for key in ("siege", "dungeon", "boss"):
            if args.fight_type != "all" and args.fight_type != key:
                continue
            bg_elem, title, outname = types[key]
            base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
            draw_fight_type_frame(base, bg_elem, title, ac, aa, dc, da,
                                  hero, enemy, crew)
            base.convert("RGB").save(os.path.join(args.outdir, outname))

    print(f"[vs] saved 4 基础帧 + 战斗类型帧（{args.fight_type}）to {args.outdir} "
          f"(atk {ac}/{aa} def {dc}/{da})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
