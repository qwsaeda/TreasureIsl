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
    """加载元素。

    S34.6 固化：优先用引擎透明资产（assets/ui/vs_v2/，RGBA 抠净）
    ——设计稿与引擎渲染同源，避免 evidence 元素 RGB 未抠净
    （hero 脚部白带 83.2% 亮像素，flood 未连通人物轮廓包围区）。
    回退 evidence 元素 + flood_white_bg（背景/横幅仍用）。
    """
    ap = os.path.join(ROOT, "assets", "ui", "vs_v2", f"{name}.png")
    if os.path.exists(ap):
        im = Image.open(ap)
        return im.convert("RGBA") if im.mode != "RGBA" else im
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


def ground_shadow(d: ImageDraw.ImageDraw, cx: int, foot_y: int, w: int, a: int = 96) -> None:
    """人物脚下椭圆落地阴影（贴地感，S34.6 固化；`foot_y` = 人物脚底 y）。

    用户 2026-11-03「还TM飞天，你咋不入地呢」：人物通用资产叠加格类插画背景 =
    贴纸感（无落地阴影/无空间融合）。本函数在人物脚底画椭圆阴影（贴地），
    clash/duel/格类帧人物贴前调用（人物脚压阴影上沿 = 落地）。幂等（无随机）。
    """
    d.ellipse(
        (cx - w // 2, foot_y - 8, cx + w // 2, foot_y + 22),
        fill=(0, 0, 0, a),
    )


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
    """顶/底状态框：名牌 + 底框（黄铜羊皮面板）。

    S34.6 分离（用户「地图和游戏逻辑分离」）：本函数只画**静态框**（名牌/底/黄铜框）——
    兵力条填充/数字由**引擎动态画**（运行态兵力，非烘焙演示值）。引擎按同坐标叠画。
    """
    d = ImageDraw.Draw(base)
    # 底框（黄铜羊皮面板）
    d.rounded_rectangle((x, y, x + w, y + h), radius=16, fill=(222, 198, 148, 235),
                        outline=(120, 90, 50, 255), width=3)
    d.rounded_rectangle((x + 6, y + 6, x + w - 6, y + h - 6), radius=12,
                        outline=(90, 66, 36, 120), width=2)
    # 名牌（静态标签）
    d.text((x + 18, y + 12), name, fill=(60, 40, 20))
    # 兵力条槽（黄铜空槽；填充由引擎动态画）
    bar_x, bar_y = x + 18, y + h - 40
    bar_w = w - 36
    draw_power_bar(base, bar_x, bar_y, bar_w, 22, 0.0, (60, 170, 90))


def draw_hud(base: Image.Image, atk_crew: int, atk_arms: int,
             def_crew: int, def_arms: int, atk_name: str = "我方舰队",
             def_name: str = "敌方驻军") -> None:
    """顶（守方）/ 底（攻方）状态框 + 中央 VS 徽标。"""
    # 顶部守方框（右对齐）
    draw_status_box(base, W - 470, 60, 440, 130, def_name, def_crew, def_arms, "def")
    # 底部攻方框（左对齐）
    draw_status_box(base, 30, H - 190, 440, 130, atk_name, atk_crew, atk_arms, "atk")
    # 中央 VS 徽标（S34.6 固化：加下方投影融合，非悬浮纯色块——用户「飞天/不入地」）
    d = ImageDraw.Draw(base)
    d.ellipse((W // 2 - 70, H // 2 - 48, W // 2 + 70, H // 2 + 92),
              fill=(0, 0, 0, 70))
    d.ellipse((W // 2 - 70, H // 2 - 70, W // 2 + 70, H // 2 + 70), fill=(60, 40, 20, 220),
              outline=(200, 160, 90, 255), width=5)
    d.text((W // 2 - 34, H // 2 - 26), "VS", fill=(240, 210, 150))


def draw_battle_frame(base: Image.Image, atk_crew: int, atk_arms: int,
                      def_crew: int, def_arms: int, bg_elem: str = "vs_bg",
                      with_hud: bool = True) -> None:
    """通用战斗层：羊皮底框 + 背景（可换战斗类型）+ 可选 HUD（状态框底 + VS 徽标）。"""
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
    if with_hud:
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
    # S34.6 固化：人物落地阴影（hero/enemy focus，背景已烘焙部队不再贴 crew）
    ground_shadow(d, 200, 1700, 190)   # hero 脚下
    ground_shadow(d, 850, 530, 210)    # enemy 脚下
    # 敌我双船长（左下攻方 hero / 右上守方 enemy）
    paste_center(base, hero, 200, 1600)
    paste_center(base, enemy, 850, 430)


def draw_gameui_design(base: Image.Image, bg_elem: str, title: str,
                       atk_crew: int, atk_arms: int, def_crew: int, def_arms: int,
                       hero: Image.Image, enemy: Image.Image) -> None:
    """游戏化战斗界面设计稿（S34.6，参照宝可梦/FGO/FF 回合制战斗 HUD）。

    用户 2026-11-03「这是游戏，不是动画片」：全屏场景插画当背景 = 动画片海报，非游戏。
    成熟游戏（宝可梦/FGO/FF）战斗界面共同规律：
      ① 背景 = 简洁战斗场地（色块地面 + 弱化远景剪影），**不抢戏**，非全屏插画；
      ② 信息优先：顶部敌方面板 + 底部我方面板（名字/兵力条/兵力数字）；
      ③ 底部指令区（攻击/防御/道具/技能）——战斗交互核心；
      ④ 人物敌上我下，站场地地面线（落地阴影），是战斗角色非插画人物。
    本函数出**设计稿**（1 张，encounter 代表），用户看效果再拍板参照哪类。
    """
    d = ImageDraw.Draw(base)
    # ---- ① 简洁战斗场地（非全屏插画）----
    # 天空/远景（上 55%）：深色渐变夜空，弱化场景图当剪影远景（低透明度，不抢戏）
    sky_top, sky_bot = (26, 22, 18), (58, 46, 32)
    for i in range(0, 960):
        t = i / 960
        r = int(sky_top[0] * (1 - t) + sky_bot[0] * t)
        g = int(sky_top[1] * (1 - t) + sky_bot[1] * t)
        b = int(sky_top[2] * (1 - t) + sky_bot[2] * t)
        d.line((0, i, W, i), fill=(r, g, b))
    # 弱化场景图当远景剪影（上 55%，低透明度，去色）
    try:
        bg = load_elem(bg_elem, remove_bg=False).convert("RGB")
        bg = bg.resize((W, 1056), Image.LANCZOS)
        # 去色 + 压暗
        g = ImageOps.grayscale(bg)
        g = ImageOps.autocontrast(g)
        g = g.point(lambda p: int(p * 0.28))
        tinted = Image.new("RGB", (W, 1056), (52, 40, 28))
        silhou = Image.composite(g, tinted, ImageOps.invert(g).point(lambda p: int(p * 0.5)))
        base.paste(silhou, (0, 0))
    except Exception:
        pass
    # 地面（下 45%，色块，分近/中/远三带，简洁）
    ground_bot = [(120, 88, 48), (98, 70, 40), (76, 54, 32)]
    for i, (col, y0, y1) in enumerate([
            (ground_bot[0], 1056, 1440),
            (ground_bot[1], 1440, 1720),
            (ground_bot[2], 1720, 1920)]):
        d.rectangle((0, y0, W, y1), fill=col)
    # 地面线（人物站地基准 + 高光）
    d.line((0, 1440, W, 1440), fill=(160, 120, 70), width=4)
    d.line((0, 1720, W, 1720), fill=(60, 44, 26), width=6)
    # 远处战场剪影（敌营火把/旗帜点，点缀不抢戏）
    for fx in (140, 340, 620, 900):
        d.rectangle((fx, 1000, fx + 4, 1056), fill=(200, 150, 80))
        d.ellipse((fx - 6, 992, fx + 10, 1008), fill=(240, 180, 90))
    # ---- ④ 人物敌上我下，站场地地面线 ----
    ground_shadow(d, 270, 1560, 230)   # hero 脚下（近处地面）
    ground_shadow(d, 810, 1180, 240)   # enemy 脚下（远处地面）
    paste_center(base, hero, 270, 1470)
    paste_center(base, enemy, 810, 1090)
    # 中央 VS 徽标（黄铜圆 + 投影）
    d.ellipse((W // 2 - 70, 900 - 48, W // 2 + 70, 900 + 92), fill=(0, 0, 0, 80))
    d.ellipse((W // 2 - 70, 900 - 70, W // 2 + 70, 900 + 70), fill=(60, 40, 20, 225),
              outline=(200, 160, 90, 255), width=5)
    d.text((W // 2 - 34, 900 - 26), "VS", fill=(240, 210, 150))
    # ---- ② 顶部敌方面板（右上，名字 + 兵力条 + 兵力数字）----
    d.rounded_rectangle((W - 470, 60, W - 30, 250), radius=16,
                        fill=(40, 32, 24, 235), outline=(200, 160, 90, 255), width=3)
    d.text((W - 450, 78), "敌方驻军", fill=(240, 210, 150))
    d.text((W - 450, 108), f"人手 {def_crew}  武器 {def_arms}  武装 {min(def_crew, def_arms)}",
           fill=(230, 200, 150))
    draw_power_bar(base, W - 450, 160, 390, 26, 0.6, (200, 80, 60))
    d.text((W - 450, 196), "兵力", fill=(180, 150, 110))
    # ---- ③ 底部指令区（攻击/防御/道具/技能 4 按钮）----
    cmd_names = ["攻击", "防御", "道具", "技能"]
    cmd_cols = [(180, 60, 50), (70, 120, 170), (90, 140, 80), (140, 90, 160)]
    bx0, by0 = 44, 1760
    btn_w, btn_h, gap = 230, 120, 16
    for i, (cn, cc) in enumerate(zip(cmd_names, cmd_cols)):
        bx = bx0 + i * (btn_w + gap)
        d.rounded_rectangle((bx, by0, bx + btn_w, by0 + btn_h), radius=12,
                            fill=cc, outline=(240, 210, 150), width=3)
        d.text((bx + btn_w // 2 - 28, by0 + btn_h // 2 - 14), cn, fill=(255, 245, 220))
    # 底部我方面板（左下，名字 + 兵力条 + 兵力数字）——指令区上方
    d.rounded_rectangle((44, 1560, 44 + 440, 1560 + 150), radius=16,
                        fill=(40, 32, 24, 235), outline=(200, 160, 90, 255), width=3)
    d.text((62, 1578), "我方舰队", fill=(240, 210, 150))
    d.text((62, 1608), f"人手 {atk_crew}  武器 {atk_arms}  武装 {min(atk_crew, atk_arms)}",
           fill=(230, 200, 150))
    draw_power_bar(base, 62, 1660, 390, 26, 0.8, (90, 160, 80))
    # 标题牌（上中）
    if title:
        d.rounded_rectangle((W // 2 - 260, 300, W // 2 + 260, 385), radius=16,
                            fill=(222, 198, 148, 235), outline=(120, 90, 50, 255), width=4)
        d.text((W // 2 - 110, 330), title, fill=(60, 40, 20))


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
    ap.add_argument("--gameui", action="store_true",
                    help="游戏化战斗界面设计稿（参照宝可梦/FGO/FF：简洁场地 + 敌我面板 + 指令区）")
    args = ap.parse_args()

    need = ["hero", "enemy", "crew", "vs_bg", "banner_win", "banner_lose",
            "bg_siege", "bg_dungeon", "bg_boss"]
    # S34.6 固化：主源 = 引擎透明资产（assets/ui/vs_v2/），缺则回退 evidence 元素
    src_dir = os.path.join(ROOT, "assets", "ui", "vs_v2")
    missing = [n for n in need
               if not os.path.exists(os.path.join(src_dir, f"{n}.png"))
               and not os.path.exists(os.path.join(ELEM, f"{n}.png"))]
    if missing:
        print(f"[vs] MISSING elements: {missing}", file=sys.stderr)
        return 2
    if args.check:
        ok = all(os.path.exists(os.path.join(args.outdir, f"vs_v2_{f}.png"))
                 for f in ["clash", "duel", "result_win", "result_lose"])
        # S34.6 分离：场景层资产（引擎 blit 读）在 assets/ui/vs_v2/
        sa = os.path.join(ROOT, "assets", "ui", "vs_v2")
        ok = ok and all(os.path.exists(os.path.join(sa, f"scene_{c}.png"))
                        for c in ["encounter", "raid", "boss", "attack"])
        # S34.6 result 帧（无人物/无 HUD/无标题，引擎叠站地 hero + 横幅 + 战报）
        ok = ok and all(os.path.exists(os.path.join(sa, f"scene_result_{c}.png"))
                        for c in ["encounter", "raid", "boss", "attack"])
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
    d = ImageDraw.Draw(base)
    # S34.6 固化：人物落地阴影（贴地，非飞天；人物脚压阴影上沿）
    ground_shadow(d, 360, 1310, 270)   # crew 方阵脚下
    ground_shadow(d, 200, 1660, 190)   # hero 领军脚下
    ground_shadow(d, 830, 520, 210)    # enemy 督战脚下（高处）
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
    d = ImageDraw.Draw(base)
    # S34.6 固化：人物落地阴影（贴地）
    ground_shadow(d, 230, 1370, 200)   # hero 脚下
    ground_shadow(d, 860, 570, 200)    # enemy 脚下
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

    # ---- 场景层资产（S34.6 分离：compose 生成资产，引擎 blit + 动态叠 HUD 数据）----
    # 用户 2026-11-03「地图和游戏逻辑分离」：VS 场景静态视觉（背景+人物+落地阴影+
    # VS徽标+标题牌+状态框底）由本脚本固化成资产（assets/ui/vs_v2/scene_<class>.png），
    # 引擎只 blit 资产 + 动态画兵力数字/条/战报（运行态数据，非烘焙演示值）。
    scene_assets = os.path.join(ROOT, "assets", "ui", "vs_v2")
    scenes = {
        # class: (bg_elem, 标题牌 or None, 输出文件名, 贴crew, enemy_foot_y, hero_foot_y, crew_foot_y)
        # S34.6 修复（用户「一个人飞到天上」）：enemy/hero 站各场景**地面线**。
        # 坐标系：compose W,H = 1080,1920（1920 高）；地面线 = 场景图亮面（python 1080 高
        # 分析 ×1.78 转 compose 坐标）——vs_bg 甲板面 1460-1690 / bg_dungeon 洞窟 1345-1690 /
        # bg_siege 城墙 1400 / bg_boss 塔门平台 900 + 石阶下 1400-1700。
        "encounter": ("vs_bg", None, "scene_encounter.png", True, 1500, 1680, 1560),
        "raid": ("bg_dungeon", "副本战 · 登岛夺宝", "scene_raid.png", False, 1400, 1600, 0),
        "boss": ("bg_boss", "BOSS 战 · 强攻要塞", "scene_boss.png", False, 900, 1550, 0),
        "attack": ("bg_siege", "攻占战 · 船队攻城", "scene_attack.png", False, 1450, 1600, 0),
    }
    os.makedirs(scene_assets, exist_ok=True)
    for cls, (bg_elem, title, outname, with_crew, enemy_foot, hero_foot, crew_foot) in scenes.items():
        base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        draw_battle_frame(base, ac, aa, dc, da, bg_elem)
        d = ImageDraw.Draw(base)
        if title:
            # 格类标题牌（上中，羊皮底；对齐 draw_fight_type_frame）
            d.rounded_rectangle((W // 2 - 260, 190, W // 2 + 260, 275), radius=16,
                                fill=(222, 198, 148, 235), outline=(120, 90, 50, 255), width=4)
            d.text((W // 2 - 110, 220), title, fill=(60, 40, 20))
        if with_crew:
            # 遭遇战甲板列阵：crew+hero+enemy + 落地阴影（站各场景地面线）
            ground_shadow(d, 360, crew_foot, 270)    # crew 方阵脚下
            ground_shadow(d, 200, hero_foot, 190)    # hero 领军脚下
            ground_shadow(d, 830, enemy_foot, 210)   # enemy 督战脚下
            paste_center(base, crew, 360, crew_foot - 90)
            paste_center(base, hero_small, 200, hero_foot - 80)
            paste_center(base, enemy, 830, enemy_foot - 90)
        else:
            # 格类帧：背景已烘焙部队，只贴 hero/enemy focus（不贴 crew 防重复）+ 落地阴影
            # （enemy 站偏上地面 / hero 站近处地面，一上一下都贴地）
            ground_shadow(d, 200, hero_foot, 190)    # hero 脚下
            ground_shadow(d, 850, enemy_foot, 210)   # enemy 脚下
            paste_center(base, hero, 200, hero_foot - 90)
            paste_center(base, enemy, 850, enemy_foot - 90)
        base.convert("RGB").save(os.path.join(scene_assets, outname))
        print(f"[vs] scene asset: assets/ui/vs_v2/{outname}")

    # ---- result 帧（S34.6 修复 result 双影/状态框底重叠）----
    # result 帧 = 仅边框+背景（无人物/无 HUD/无标题牌），引擎 draw_result_scene 叠
    # 站地 hero/enemy + 书法横幅 + 战报框。分离 = 场景帧人物与 result 人物不再双影，
    # 状态框底（clash 专属）不再透出 result 帧。
    for cls, (bg_elem, _title, _outname, _crew, _ef, _hf, _cf) in scenes.items():
        result_out = f"scene_result_{cls}.png"
        base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        draw_battle_frame(base, ac, aa, dc, da, bg_elem, with_hud=False)
        base.convert("RGB").save(os.path.join(scene_assets, result_out))
        print(f"[vs] result asset: assets/ui/vs_v2/{result_out}")

    # ---- gameui 设计稿（S34.6，参照宝可梦/FGO/FF）----
    if args.gameui:
        base = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        draw_gameui_design(base, "vs_bg", "遭遇战 · 甲板对峙",
                           ac, aa, dc, da, hero, enemy)
        out = os.path.join(args.outdir, "game_ui_design.png")
        base.convert("RGB").save(out)
        print(f"[vs] gameui design: {out}")

    print(f"[vs] saved 4 基础帧 + 战斗类型帧（{args.fight_type}）to {args.outdir} "
          f"(atk {ac}/{aa} def {dc}/{da})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
