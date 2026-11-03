#!/usr/bin/env python3
"""tmp：VS 方案 A（纯 HUD）角色形态对比预演（4 联，同构图只换角色摆位）。

用法：python3 scripts/tmp/vs_a_compare.py
输出：evidence/ui_concept/vs_v4_role_compare.png（含 .thumb.png 侧车，手动跑 ui_thumb）

背景 = 羊皮纹理（chart_wide 拉伸）+ 暗角（用户拍板保留羊皮）；
HUD = 顶部敌方面板 + 底部我方面板 + 中央 VS 徽标 + 指令区（跳过/确认/重打）。
角色 4 种摆位：
  A 小精灵  敌上我下，小尺寸（立绘缩 ~30%），站地面线 + 椭圆投影
  B 立绘中  敌上我下，中尺寸（立绘 ~55%），站地面线 + 投影（当前引擎接近此形态）
  C 立绘大  敌上我下，大尺寸（立绘 ~75%），背景退更暗（角色仍主角）
  D 卡牌棋子 角色退为面板内头像圆块 + 战场只留兵种棋子（crew 缩 20%），最纯 HUD
"""
import math
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parents[2]  # scripts/tmp/ → 仓库根
OUT = ROOT / "evidence" / "ui_concept" / "vs_v4_role_compare.png"
W, H = 1080, 1920
CELL = 540  # 4 联 = 2x2，每格 540x960 再缩到 540x960

HERO = ROOT / "assets" / "ui" / "vs_v2" / "hero.png"
ENEMY = ROOT / "assets" / "ui" / "vs_v2" / "enemy.png"
CREW = ROOT / "assets" / "ui" / "vs_v2" / "crew.png"
CHART = ROOT / "assets" / "ui" / "chart_wide.png"

INK = (60, 45, 25)
BRASS = (147, 101, 64)
GOLD = (246, 146, 30)
PARCH_BG = (232, 217, 168)
PLAYER = (70, 150, 220)
DANGER = (200, 70, 50)
CREW_COLOR = (120, 90, 60)
CREW_DANGER = (110, 60, 50)

FONT_PATH = "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc"


def _font(px: int):
    from PIL import ImageFont

    return ImageFont.truetype(FONT_PATH, px)


def _load(name: Path) -> Image.Image:
    im = Image.open(name).convert("RGBA")
    bb = im.getbbox()
    if bb:
        im = im.crop(bb)
    return im


HERO_IM = _load(HERO)
ENEMY_IM = _load(ENEMY)
CREW_IM = _load(CREW)
CHART_IM = _load(CHART)


def _fit(im: Image.Image, h: int) -> Image.Image:
    """按目标高度等比缩放。"""
    scale = h / im.height
    w = max(1, round(im.width * scale))
    return im.resize((w, h), Image.LANCZOS)


def _shadow(d: ImageDraw.ImageDraw, cx: int, foot_y: int, w: int) -> None:
    """角色脚下椭圆投影。"""
    d.ellipse([cx - w // 2, foot_y - 8, cx + w // 2, foot_y + 8], fill=(40, 30, 15, 160))


def bg_parchment(dim: tuple[int, int], vignette: float) -> Image.Image:
    """羊皮底（chart_wide 拉伸）+ 暗角（vignette 0..1）。"""
    w, h = dim
    im = CHART_IM.resize((w, h), Image.LANCZOS).convert("RGBA")
    if vignette > 0:
        vig = Image.new("RGBA", (w, h), (0, 0, 0, 0))
        vd = ImageDraw.Draw(vig)
        step = max(1, h // 40)
        for i in range(40):
            a = round(vignette * 255 * (i / 40) ** 2)
            y0 = h - (i + 1) * step
            y1 = h - i * step
            if y1 > y0:
                vd.rectangle([0, y0, w, y1], fill=(20, 15, 8, a // 3))
            x0 = w - (i + 1) * step
            x1 = w - i * step
            if x1 > x0:
                vd.rectangle([x0, 0, x1, h], fill=(20, 15, 8, a // 3))
        im = Image.alpha_composite(im, vig)
    return im


def panel(d: ImageDraw.ImageDraw, x: int, y: int, w: int, h: int,
          title: str, crew: int, arms: int, color: tuple) -> None:
    """敌/我状态面板（羊皮底 + 黄铜框 + 标题 + 兵力条 + 人手/武器/武装 三数）。"""
    armed = min(crew, arms)
    d.rounded_rectangle([x, y, x + w, y + h], radius=10, fill=(214, 196, 148, 235),
                        outline=BRASS, width=4)
    d.rounded_rectangle([x + 8, y + 8, x + w - 8, y + h - 8], radius=6,
                        outline=(90, 70, 40), width=1)
    d.text((x + 24, y + 14), title, fill=INK, font="12px")
    # 兵力条（人手/武器 双条，敌=红 我=蓝）
    bx, by, bw, bh = x + 24, y + 56, w - 48, 16
    maxv = max(crew, arms, 1)
    d.rounded_rectangle([bx, by, bx + bw, by + bh], radius=6, fill=(90, 70, 40),
                        outline=(60, 45, 25), width=2)
    d.rounded_rectangle([bx, by, bx + max(1, round(bw * crew / maxv)), by + bh],
                        radius=6, fill=color)
    by2 = by + bh + 8
    d.rounded_rectangle([bx, by2, bx + bw, by2 + bh], radius=6, fill=(90, 70, 40),
                        outline=(60, 45, 25), width=2)
    d.rounded_rectangle([bx, by2, bx + max(1, round(bw * arms / maxv)), by2 + bh],
                        radius=6, fill=GOLD)
    d.text((x + 24, y + h - 40),
           f"人手 {crew}  武器 {arms}  武装 {armed}", fill=INK, font="11px")


def command_zone(d: ImageDraw.ImageDraw, x: int, y: int, w: int, h: int) -> None:
    """指令区：跳过 / 确认 / 重打 三按钮（底部）。"""
    bw = (w - 40) // 3
    labels = ("跳过", "确认", "重打")
    for i in range(3):
        bx = x + i * (bw + 20)
        sel = i == 1
        d.rounded_rectangle([bx, y, bx + bw, y + h], radius=8,
                            fill=(236, 222, 180) if sel else (206, 188, 142),
                            outline=GOLD if sel else BRASS,
                            width=3 if sel else 2)
        d.text((bx + bw // 2 - 14, y + h // 2 - 14), labels[i], fill=INK, font="13px")


def place_char(base: Image.Image, im: Image.Image, cx: int, foot_y: int,
               h: int, shadow: bool = True) -> None:
    """按目标高度把角色底边贴到 foot_y 地线上。"""
    fitted = _fit(im, h)
    if shadow:
        _shadow(ImageDraw.Draw(base), cx, foot_y, fitted.width)
    base.alpha_composite(fitted, (cx - fitted.width // 2, foot_y - fitted.height))


def panel_frame(base: Image.Image, kind: str) -> tuple[int, int, int, int]:
    """面板区域坐标（与引擎 render_v2_elem 顶/底框同值：顶框 (610,60,440,130) / 底框 (30,1730,440,130)
    在 540 格内按比例折半）。"""
    if kind == "enemy":
        return (610 // 2, 60, 440, 130)
    return (30, 1730 - 0, 440, 130)  # 我方 = 战报区让位，底框上移


def build(variant: str) -> Image.Image:
    dim = (W // 2, H // 2)  # 540x960 格
    vw, vh = dim
    base = bg_parchment(dim, vignette=0.5 if variant in ("A", "D") else 0.25)
    d = ImageDraw.Draw(base)
    # 格内几何（引擎坐标 ÷2 折到 540x960 格）
    ew, eh = vw, 220  # 敌方区（上 1/4）
    # 敌方面板（右上，对称布局）
    panel(d, vw - 250, 30, 220, 150, "敌方 海盗旗舰", 30, 20, DANGER)
    # 我方面板（左上，对角对称）
    panel(d, 30, 30, 220, 150, "我方 木帆船", 30, 18, PLAYER)
    # 中央 VS 徽标
    d.ellipse([vw // 2 - 34, vh // 2 - 34, vw // 2 + 34, vh // 2 + 34],
              fill=(147, 101, 64, 220), outline=GOLD, width=3)
    d.text((vw // 2 - 12, vh // 2 - 12), "VS", fill=GOLD, font="14px")
    # 指令区（底部）
    command_zone(d, 60, vh - 120, vw - 120, 70)

    # 角色摆位（地面线：敌方 y=430（中上），我方 y=760（中下））
    enemy_gy = 470
    player_gy = 760
    if variant == "A":  # 小精灵：立绘缩 ~30%（h≈260 格内）
        place_char(base, ENEMY_IM, vw - 150, enemy_gy, 250)
        place_char(base, HERO_IM, 150, player_gy, 250)
    elif variant == "B":  # 立绘中：~55%（h≈430）
        place_char(base, ENEMY_IM, vw - 140, enemy_gy, 420)
        place_char(base, HERO_IM, 150, player_gy, 420)
    elif variant == "C":  # 立绘大：~75%（h≈560），背景更暗
        base = bg_parchment(dim, vignette=0.6)
        d = ImageDraw.Draw(base)
        panel(d, vw - 250, 30, 220, 150, "敌方 海盗旗舰", 30, 20, DANGER)
        panel(d, 30, 30, 220, 150, "我方 木帆船", 30, 18, PLAYER)
        d.ellipse([vw // 2 - 34, vh // 2 - 34, vw // 2 + 34, vh // 2 + 34],
                  fill=(147, 101, 64, 220), outline=GOLD, width=3)
        d.text((vw // 2 - 12, vh // 2 - 12), "VS", fill=GOLD, font="14px")
        command_zone(d, 60, vh - 120, vw - 120, 70)
        place_char(base, ENEMY_IM, vw - 120, enemy_gy + 60, 560, shadow=False)
        place_char(base, HERO_IM, 150, player_gy, 560, shadow=False)
    elif variant == "D":  # 卡牌棋子：面板内头像圆块 + 战场只留兵种棋子（crew 缩 20%）
        # 敌方头像圆块（面板右上角内嵌，预演用圆裁示意；引擎无圆裁能力，落地用方形 + 圆角描边近似）
        for im0, px, py in ((ENEMY_IM, vw - 130, 34), (HERO_IM, 24, 34)):
            head = _fit(im0, 150)
            cm = Image.new("RGBA", (150, 150), (0, 0, 0, 0))
            cm.alpha_composite(head, ((150 - head.width) // 2, 150 - head.height))
            mask = Image.new("L", (150, 150), 0)
            ImageDraw.Draw(mask).ellipse([0, 0, 149, 149], fill=255)
            alpha = cm.getchannel("A").point(lambda v, m=mask: min(v, m.getpixel((0, 0))))
            cm.putalpha(mask)
            cm.putalpha(Image.frombytes("L", (150, 150),
                                         bytes(min(cm.getchannel("A").getpixel(xy), mask.getpixel(xy))
                                               for xy in [(x, y) for y in range(150) for x in range(150)])))
            canvas_m = Image.new("RGBA", cm.size, (0, 0, 0, 0))
            canvas_m.paste(cm, (0, 0), mask)
            base.alpha_composite(canvas_m, (px, py))
        # 战场只留兵种棋子（crew 小图，敌上我下站地面线）
        crew_e = _fit(CREW_IM, 170)
        _shadow(d, vw - 150, enemy_gy + 170, crew_e.width)
        base.alpha_composite(crew_e, (vw - 150 - crew_e.width // 2, enemy_gy))
        crew_p = _fit(CREW_IM, 170)
        _shadow(d, 150, player_gy + 170, crew_p.width)
        base.alpha_composite(crew_p, (150 - crew_p.width // 2, player_gy - 10))
    else:
        raise ValueError(variant)
    return base


def main() -> None:
    cells = [build(v) for v in ("A", "B", "C", "D")]
    canvas = Image.new("RGB", (W, H), (30, 24, 16))
    dd = ImageDraw.Draw(canvas)
    pos = [(0, 0), (540, 0), (0, 960), (540, 960)]
    names = ("A 小精灵（立绘缩 30% + 地面线投影）",
             "B 立绘中（立绘 55% + 投影，接近现状）",
             "C 立绘大（立绘 75%，背景加暗）",
             "D 纯卡牌棋子（头像圆块 + 兵种棋子）")
    for cell, p, name in zip(cells, pos, names):
        canvas.paste(cell, p)
        dd.rectangle([p[0] + 2, p[1] + 2, p[0] + 538, p[1] + 958], outline=GOLD, width=3)
        dd.text((p[0] + 16, p[1] + 930), name, fill=GOLD, font="13px")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(OUT)
    print(f"[OK] {OUT}")
    print("尺寸 1080x1920（2x2 各 540x960）；请生成 .thumb.png 侧车或平台直读")


if __name__ == "__main__":
    main()
