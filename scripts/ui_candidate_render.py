#!/usr/bin/env python3
"""视觉候选图合成（飞轮 R21 / S63 T0a）：用现成 UI 资产 + PIL 合成「面板/场景重排」候选看板，
给用户挑风格基准（选定后批量套用 8 面板）。零 Rust、零 scenes.json 改动（候选阶段）。

变体：
  select_role：A 立绘全高左半屏 / B 立绘居中 + 两侧列表 / C 双立绘对置
  vs：         A 双人对峙居中 / B 斜向列阵 / C 当前版放大版
  panel（通用面板风格基准，以贸易为例）：A 羊皮卷轴居中 / B 木质牌匾左列 / C 底部抽屉式

用法：python3 scripts/ui_candidate_render.py --out evidence/ui_candidate
"""
from __future__ import annotations

import argparse
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UI = ROOT / "assets" / "ui"
CJK = "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc"


def _font(sz: int):
    from PIL import ImageFont  # noqa: PLC0415
    return ImageFont.truetype(CJK, sz) if Path(CJK).exists() else ImageFont.load_default()


def _img(name: str):
    from PIL import Image  # noqa: PLC0415
    return Image.open(UI / f"{name}.png").convert("RGBA")


def _parchment_bg(w: int, h: int):
    """羊皮纸底：chart_wide 拉伸铺底 + 暗角。"""
    from PIL import Image, ImageDraw  # noqa: PLC0415
    base = _img("chart_wide").resize((w, h))
    layer = Image.new("RGBA", (w, h), (18, 14, 8, 96))
    base.alpha_composite(layer)
    return base


def _panel_box(draw, x, y, w, h, title, accent="#d4a017"):
    """羊皮面板框（半透明底 + 金边 + 标题牌）。"""
    draw.rounded_rectangle([x, y, x + w, y + h], radius=18, fill=(28, 22, 12, 210), outline=accent, width=3)
    draw.rounded_rectangle([x + 24, y + 16, x + w - 24, y + 66], radius=8, fill=(60, 40, 18, 230), outline=accent, width=2)
    draw.text((x + 40, y + 26), title, fill="#f5d76e", font=_font(26))


def render_select_role(out: Path, variant: str):
    from PIL import Image, ImageDraw  # noqa: PLC0415
    W, H = 1920, 1080
    im = _parchment_bg(W, H)
    dr = ImageDraw.Draw(im)
    portrait = _img("role_drake")
    dr.text((40, 24), f"角色选择重排 · 候选 {variant}（S63 T0a）", fill="#fff", font=_font(30))
    if variant == "A":
        # 立绘占左半屏全高，右侧大字列表
        p = portrait.resize((820, 900))
        im.alpha_composite(p, (40, 120))
        _panel_box(dr, 920, 120, 940, 900, "选择船员")
        for i, n in enumerate(["潘达 · 商人", "泰奇 · 船长", "基德 · 猎手", "德雷克 · 海盗", "凯瑟琳 · 贵妇"]):
            dr.text((970, 200 + i * 120), n, fill="#f5d76e", font=_font(30))
    elif variant == "B":
        # 立绘居中大图，两侧列表
        p = portrait.resize((700, 820))
        im.alpha_composite(p, (610, 180))
        _panel_box(dr, 40, 180, 520, 820, "船员（左）")
        _panel_box(dr, 1360, 180, 520, 820, "船员详情（右）")
        for i, n in enumerate(["潘达", "泰奇", "基德", "德雷克", "凯瑟琳"]):
            dr.text((90, 280 + i * 130), n, fill="#f5d76e", font=_font(30))
        dr.text((1410, 300), "立绘居中", fill="#f5d76e", font=_font(28))
        dr.text((1410, 360), "左右列表", fill="#f5d76e", font=_font(28))
    else:  # C
        # 双立绘对置（当前选中 + 上一角色）
        p1 = portrait.resize((640, 900))
        p2 = _img("role_panda").resize((640, 900))
        im.alpha_composite(p2, (120, 120))
        im.alpha_composite(p1, (1160, 120))
        _panel_box(dr, 660, 300, 600, 480, "当前 / 上一角色")
        dr.text((700, 420), "双立绘对置", fill="#f5d76e", font=_font(32))
        dr.text((700, 480), "PvP 选人感", fill="#f5d76e", font=_font(28))
    im.convert("RGB").save(out)


def render_vs(out: Path, variant: str):
    from PIL import Image, ImageDraw  # noqa: PLC0415
    W, H = 1920, 1080
    im = _parchment_bg(W, H)
    dr = ImageDraw.Draw(im)
    hero = _img("hero_back")
    boss = _img("boss_captain")
    dr.text((40, 24), f"Vs 战斗重排 · 候选 {variant}（S63 T0a）", fill="#fff", font=_font(30))
    if variant == "A":
        # 双人对峙居中
        im.alpha_composite(hero.resize((560, 760)), (240, 240))
        im.alpha_composite(boss.resize((560, 760)), (1120, 240))
        dr.text((860, 500), "VS", fill="#f5d76e", font=_font(80))
        _panel_box(dr, 40, 40, 700, 110, "我方舰队 · 兵力 HUD")
        _panel_box(dr, 1180, 40, 700, 110, "敌方驻军 · 兵力 HUD")
    elif variant == "B":
        # 斜向列阵（我方左下 3 人，敌方右上 BOSS）
        for i in range(3):
            im.alpha_composite(hero.resize((320, 480)), (120 + i * 200, 520 - i * 90))
        im.alpha_composite(boss.resize((640, 760)), (1180, 180))
        dr.text((700, 300), "VS", fill="#f5d76e", font=_font(70))
        _panel_box(dr, 40, 40, 700, 110, "我方舰队（斜列）")
        _panel_box(dr, 1180, 40, 700, 110, "敌方驻军")
    else:  # C
        # 当前版放大（保留炮台 + 双方立绘，仅放大居中）
        im.alpha_composite(_img("deck_battle").resize((1920, 700)), (0, 200))
        im.alpha_composite(hero.resize((460, 620)), (180, 400))
        im.alpha_composite(boss.resize((460, 620)), (1280, 400))
        dr.text((880, 480), "VS", fill="#f5d76e", font=_font(70))
    im.convert("RGB").save(out)


def render_panel(out: Path, variant: str):
    """通用面板风格基准（以贸易面板为例：5 商品列表 + 买/卖 + 金币）。"""
    from PIL import Image, ImageDraw  # noqa: PLC0415
    W, H = 1920, 1080
    im = _parchment_bg(W, H)
    dr = ImageDraw.Draw(im)
    dr.text((40, 24), f"贸易面板（通用面板风格基准）· 候选 {variant}（S63 T0a）", fill="#fff", font=_font(30))
    if variant == "A":
        # 羊皮卷轴居中（圆角羊皮 + 金边）
        _panel_box(dr, 360, 120, 1200, 860, "贸易 · 买卖货物")
        for i, n in enumerate(["布匹", "香料", "朗姆酒", "火药", "木材"]):
            dr.text((460, 240 + i * 110), f"{n}  买/卖  [按钮]", fill="#f5d76e", font=_font(30))
        dr.text((1250, 240), "金 1000", fill="#2ecc71", font=_font(34))
    elif variant == "B":
        # 木质牌匾左列（竖排商品 + 右侧详情）
        _panel_box(dr, 80, 140, 620, 820, "商品")
        for i, n in enumerate(["布匹", "香料", "朗姆酒", "火药", "木材"]):
            dr.text((140, 260 + i * 120), n, fill="#f5d76e", font=_font(32))
        _panel_box(dr, 760, 140, 1080, 820, "交易详情")
        dr.text((840, 280), "选中商品 · 数量 · 价格", fill="#f5d76e", font=_font(28))
        dr.text((840, 380), "买入 / 卖出 按钮", fill="#f5d76e", font=_font(28))
        dr.text((840, 480), "金 1000", fill="#2ecc71", font=_font(34))
    else:  # C
        # 底部抽屉式（半屏面板从底部升起，背景半透主地图）
        dr.rectangle([0, 520, W, H], fill=(24, 18, 10, 235), outline="#d4a017", width=3)
        dr.text((60, 560), "贸易 · 底部抽屉式", fill="#f5d76e", font=_font(32))
        for i, n in enumerate(["布匹", "香料", "朗姆酒", "火药", "木材"]):
            x = 80 + i * 350
            dr.rounded_rectangle([x, 660, x + 300, 900], radius=12, fill=(50, 36, 18, 240), outline="#d4a017", width=2)
            dr.text((x + 30, 710), n, fill="#f5d76e", font=_font(28))
            dr.text((x + 30, 770), "买 / 卖", fill="#2ecc71", font=_font(24))
    im.convert("RGB").save(out)


def render_handoff(out: Path, variant: str):
    """热座交接遮蔽形态候选（territory.md §11 U9）：整屏盖板 A / 角落提示条 B。
    横屏 1920×1080；羊皮底模拟主地图，确认前是否可见棋盘 = 两变体核心差异。"""
    from PIL import Image, ImageDraw  # noqa: PLC0415
    W, H = 1920, 1080
    im = _parchment_bg(W, H)
    dr = ImageDraw.Draw(im, "RGBA")
    if variant == "A":
        # 整屏不透明盖板：确认前完全遮蔽棋盘（符合「不泄露他人情报」最强口径）。
        dr.rectangle([0, 0, W, H], fill=(12, 9, 5, 250))
        rose = _img("compass_rose").resize((360, 360))
        im.alpha_composite(rose, (W // 2 - 180, 220))
        dr.text((W // 2 - 300, 640), "轮到玩家 2", fill="#f5d76e", font=_font(52))
        dr.text((W // 2 - 360, 730), "按确认开始你的回合", fill="#e8d8a0", font=_font(38))
        dr.rounded_rectangle([W // 2 - 220, 820, W // 2 + 220, 910], radius=16,
                             fill=(60, 40, 18, 240), outline="#d4a017", width=3)
        dr.text((W // 2 - 130, 845), "Enter 确认", fill="#2ecc71", font=_font(34))
        dr.text((60, H - 80), "A · 整屏盖板（棋盘完全遮蔽）", fill="#d4a017", font=_font(26))
    else:
        # 角落提示条（当前实现）：底部 56px 提示条 + P# 席位，棋盘在确认前仍可见。
        pw, ph = 620, 72
        px, py = W // 2 - pw // 2, H - 130
        dr.rounded_rectangle([px, py, px + pw, py + ph], radius=12,
                             fill=(28, 22, 12, 230), outline="#d4a017", width=3)
        dr.text((px + 40, py + 18), "轮到玩家 2 · 按确认摇骰开始回合（P2）",
                fill="#f5d76e", font=_font(28))
        dr.text((60, H - 60), "B · 角落提示条（棋盘仍可见）", fill="#d4a017", font=_font(26))
    im.convert("RGB").save(out)


def render_handoff_overall(out: Path, variant: str):
    """U9 整体效果：在**真实主地图引擎截图**上叠加交接层，看整体画面（非孤立盖板）。
    底图 = evidence/bevy_native_main_map_1920x1080.png（引擎真实渲染，U9 纪律）。"""
    from PIL import Image, ImageDraw  # noqa: PLC0415
    base_path = ROOT / "evidence" / "bevy_native_main_map_1920x1080.png"
    if not base_path.exists():
        raise SystemExit(f"缺主地图底图：{base_path}（先跑 --shot main_map）")
    im = Image.open(base_path).convert("RGBA")
    W, H = im.size
    dr = ImageDraw.Draw(im, "RGBA")
    if variant == "A":
        # 整屏盖板：在真实棋盘上盖深色半透明层（保留棋盘轮廓暗示"下面是对局"）。
        # 关键：PIL 在 RGBA 底图上用 ImageDraw 画半透明矩形是「替换」不混合 ⇒ 必须
        # 用独立 layer + alpha_composite 才能真正透出底图（之前全黑就是这个 bug）。
        veil = Image.new("RGBA", (W, H), (10, 8, 5, 205))
        im.alpha_composite(veil)
        dr = ImageDraw.Draw(im, "RGBA")
        rose = _img("compass_rose").resize((300, 300))
        im.alpha_composite(rose, (W // 2 - 150, 180))
        dr.text((W // 2 - 300, 560), "轮到玩家 2", fill="#f5d76e", font=_font(56))
        dr.text((W // 2 - 340, 660), "确认前棋盘完全遮蔽", fill="#c9b888", font=_font(30))
        dr.text((W // 2 - 360, 720), "按确认开始你的回合", fill="#e8d8a0", font=_font(36))
        dr.rounded_rectangle([W // 2 - 200, 820, W // 2 + 200, 900], radius=14,
                             fill=(60, 40, 18, 240), outline="#d4a017", width=3)
        dr.text((W // 2 - 110, 843), "Enter 确认", fill="#2ecc71", font=_font(32))
        dr.text((50, 40), "A · 整屏盖板（真实棋盘上叠加，确认前完全遮蔽）",
                fill="#f5d76e", font=_font(28))
    else:
        # 角落提示条：真实棋盘全可见 + 底部提示条。
        pw, ph = 720, 76
        px, py = W // 2 - pw // 2, H - 150
        dr.rounded_rectangle([px, py, px + pw, py + ph], radius=12,
                             fill=(28, 22, 12, 235), outline="#d4a017", width=3)
        dr.text((px + 40, py + 20), "轮到玩家 2 · 按确认摇骰开始回合（P2）",
                fill="#f5d76e", font=_font(30))
        dr.text((50, 40), "B · 角落提示条（真实棋盘仍全部可见）",
                fill="#f5d76e", font=_font(28))
    im.convert("RGB").save(out)


def main():
    ap = argparse.ArgumentParser(description="视觉候选图合成（S63 T0a / S73 U9）")
    ap.add_argument("--out", default=str(ROOT / "evidence" / "ui_candidate"))
    ap.add_argument("--check", action="store_true", help="只读体检：资产 + CJK 字体在位（0 写盘）")
    args = ap.parse_args()
    if args.check:
        need = ["chart_wide.png", "role_drake.png", "role_panda.png", "hero_back.png", "boss_captain.png", "deck_battle.png"]
        missing = [n for n in need if not (UI / n).exists()]
        if missing:
            print(f"[check] 缺资产：{missing}", file=sys.stderr)
            raise SystemExit(1)
        if not Path(CJK).exists():
            print(f"[check] 缺 CJK 字体：{CJK}", file=sys.stderr)
            raise SystemExit(1)
        print(f"[check] {len(need)} 资产 + CJK 字体在位（0 写盘）：PASS")
        return
    out = Path(args.out)
    if not out.is_absolute():
        out = ROOT / out
    out.mkdir(parents=True, exist_ok=True)
    for scene, fn in [("select_role", render_select_role), ("vs", render_vs), ("panel_trade", render_panel)]:
        for v in ["A", "B", "C"]:
            p = out / f"{scene}_{v}.png"
            fn(p, v)
            print(f"  出图 {p.relative_to(ROOT)}")
    for v in ["A", "B"]:
        p = out / f"handoff_{v}.png"
        render_handoff(p, v)
        print(f"  出图 {p.relative_to(ROOT)}")
        # U9 整体效果（真实棋盘叠加），看整体画面。
        po = out / f"handoff_{v}_overall.png"
        render_handoff_overall(po, v)
        print(f"  出图 {po.relative_to(ROOT)}")
    print(f"ui_candidate: 13 张候选图（含 U9 handoff A/B + 整体效果）→ {out}")


if __name__ == "__main__":
    main()
