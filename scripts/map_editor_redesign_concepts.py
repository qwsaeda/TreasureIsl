#!/usr/bin/env python3
"""map_editor_redesign_concepts.py — 地图编辑器重设计 3 方向概念效果图（A 纪律设计稿先行）。

出 3 张横版概念稿（1600×1000）落 evidence/ui_concept/，每个方向解决「看不懂」的
不同根因（布局/信息密度/视觉层级），用户挑一个再细化：
  - 方向 A「仪表盘」：左控制 / 中大画布 / 右属性+校验，顶部步骤条，一眼定位各区块职责
  - 方向 B「地图为主」：全屏大海图 + 右下角浮层属性 + 顶部工具栏，沉浸感强
  - 方向 C「卡片流」：竖向卡片堆叠（画布卡/内容卡/校验卡/向导卡），自上而下顺序读

工具契约（14 号 §1.2）：--check 只读（产物在位非空，缺非 0）；幂等（PIL 确定性重画）。
"""
import argparse
import os
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "ui_concept"


def font(size, bold=False):
    for p in ([
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ] if bold else [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ]):
        if os.path.exists(p):
            try:
                return ImageFont.truetype(p, size)
            except Exception:
                continue
    return ImageFont.load_default()


# 颜色
BG = (18, 22, 30)
PANEL = (30, 36, 46)
PANEL2 = (40, 48, 62)
BORDER = (64, 74, 92)
INK = (225, 230, 240)
MUTED = (140, 152, 172)
ACCENT = (126, 200, 227)
GREEN = (80, 190, 110)
RED = (220, 90, 80)
GOLD = (200, 170, 90)
PARCH = (58, 50, 38)

# 10 类岛色带（theme.json isle_* 近似）
ISLE = {
    "harbour": (200, 122, 110), "lighthouse": (224, 176, 112), "wreck": (232, 192, 64),
    "pirate_lair": (58, 58, 92), "shipyard": (62, 138, 120), "trading_post": (140, 122, 60),
    "skull_cliff": (232, 228, 208), "volcano": (216, 92, 48), "lightship": (122, 140, 160),
    "boss_fort": (56, 40, 74),
}
GRAY = (90, 96, 110)
KCOL = {"start": (63, 185, 80), "boss": (208, 84, 84), "event": (122, 106, 154),
        "card": (122, 106, 154), "treasure": (122, 106, 154), "territory": None}
KLET = {"start": "起", "boss": "BOSS", "event": "事", "card": "卡", "treasure": "宝"}


def new_canvas(W=1600, H=1000):
    c = Image.new("RGB", (W, H), BG)
    return c, ImageDraw.Draw(c)


def ring_layout(cx, cy, r, n=31):
    import math
    pts = []
    for i in range(n):
        a = -90 + i * (360 / n)
        th = math.radians(a)
        pts.append((cx + r * math.cos(th), cy + r * math.sin(th), i))
    return pts


def kind_isle(i):
    """31 格 kind + isle 序列（与 edDefaultState 同构）。"""
    kindSeq = ['start','territory','territory','event','territory','territory','card','territory','territory','treasure','territory','territory','event','territory','territory','boss','territory','territory','event','territory','territory','treasure','territory','territory','card','territory','territory','territory','territory','event','card']
    isleSeq = [None,'harbour','harbour',None,'lighthouse','lighthouse',None,'wreck','wreck',None,'pirate_lair','pirate_lair',None,'shipyard','shipyard',None,'trading_post','trading_post',None,'skull_cliff','skull_cliff',None,'volcano','volcano',None,'lightship','lightship','boss_fort','boss_fort',None,None]
    return kindSeq[i], isleSeq[i]


def draw_ring(d, cx, cy, r, card_w, card_h, sel_idx=None, drag=None):
    """环形画布（节点方块+色带+岛缩写+格序；功能格蓝底字母；选中描边）。"""
    d.ellipse([cx-r-card_w, cy-r-card_h, cx+r+card_w, cy+r+card_h], outline=(58, 68, 86), width=2)
    d.ellipse([cx-150, cy-150, cx+150, cy+150], fill=PARCH, outline=GOLD, width=2)
    d.text((cx-30, cy-30), "中央据点", font=font(16, True), fill=GOLD)
    d.text((cx-46, cy+8), "island_home_m04", font=font(11), fill=MUTED)
    for x, y, i in ring_layout(cx, cy, r):
        kind, isle = kind_isle(i)
        w, h = card_w, card_h
        x0, y0 = x-w/2, y-h/2
        if kind == "territory":
            col = ISLE[isle] if isle else GRAY
            label = (isle or "无主")[:6].upper()
        else:
            col = KCOL[kind]
            label = KLET.get(kind, kind)
        sel = (i == sel_idx)
        d.rounded_rectangle([x0, y0, x0+w, y0+h], radius=6, fill=col,
                            outline=(ACCENT if sel else (20, 24, 30)), width=3 if sel else 1)
        d.text((x0+4, y0+3), str(i), font=font(10, True), fill=(255, 255, 255))
        d.text((x0+w/2-22, y0+h/2-8), label, font=font(13, True), fill=(255, 255, 255))
        d.text((x0+4, y0+h-13), "格序%d" % i, font=font(9), fill=(230, 230, 230))
    # 拖拽虚线框提示
    if drag:
        d.rounded_rectangle([drag[0]-30, drag[1]-30, drag[0]+30, drag[1]+30], radius=8,
                            outline=ACCENT, width=2)
        d.text((drag[0]-60, drag[1]-46), "拖到这里 = 自由摆放 pos", font=font(11), fill=ACCENT)


def title_bar(d, W, title, sub):
    d.rectangle([0, 0, W, 54], fill=PANEL)
    d.text((20, 14), title, font=font(22, True), fill=INK)
    d.text((20, 34), sub, font=font(12), fill=MUTED)
    d.line([0, 54, W, 54], fill=BORDER, width=1)


def step_strip(d, x, y, w, cur=4):
    """顶部 10 步进度条（第 cur 步高亮）。"""
    names = ["add","import","syncmap","verify","pack","install","check","sync","shot","zero_rust"]
    sw = (w - 9*10) / 10
    for i, n in enumerate(names):
        bx = x + i*(sw+10)
        c = GREEN if i < cur else (GOLD if i == cur else (58, 68, 84))
        d.rounded_rectangle([bx, y, bx+sw, y+34], radius=6, fill=c)
        d.text((bx+sw/2-16, y+4), str(i+1), font=font(13, True), fill=(255,255,255))
        d.text((bx, y+38), n, font=font(10), fill=MUTED if i != cur else GOLD)


def legend(d, x, y, items):
    """岛色带图例（两态 + 10 类）。"""
    d.text((x, y), "图例（节点色带）", font=font(12, True), fill=MUTED)
    yy = y + 24
    for label, col in items:
        d.rectangle([x, yy, x+18, yy+14], fill=col, outline=(20, 24, 30))
        d.text((x+26, yy-1), label, font=font(11), fill=INK)
        yy += 20


def panel_box(d, x, y, w, h, title):
    d.rounded_rectangle([x, y, x+w, y+h], radius=8, fill=PANEL, outline=BORDER, width=1)
    d.text((x+14, y+10), title, font=font(14, True), fill=ACCENT)
    d.line([x+14, y+34, x+w-14, y+34], fill=BORDER, width=1)


def check_bar(d, x, y, w, n_fail=1, fail_items=None):
    d.rounded_rectangle([x, y, x+w, y+90], radius=6, fill=PANEL2, outline=BORDER)
    d.text((x+12, y+8), "I1–I9 + S34.5 实时校验", font=font(12, True), fill=MUTED)
    if n_fail == 0:
        d.text((x+12, y+36), "✓ 全过（前端判定；导入后 map_pack.py verify 终判含 I7 资产哈希）",
               font=font(13, True), fill=GREEN)
    else:
        d.text((x+12, y+34), "✗ 未过 %d 项：" % n_fail, font=font(13, True), fill=RED)
        yy = y + 54
        for it in (fail_items or [])[:3]:
            d.text((x+20, yy), "· " + it, font=font(11), fill=RED)
            yy += 18


# ================================================================== 方向 A 仪表盘
def concept_A():
    W, H = 1600, 1000
    c, d = new_canvas(W, H)
    title_bar(d, W, "地图编辑器 · 仪表盘布局", "左：参数与制作流程　|　中：环形画布（拖拽 + 选岛）　|　右：节点属性 + 内容配置 + 校验")
    # 顶部步骤条
    step_strip(d, 24, 66, W - 48, cur=3)
    # 左列：参数 + 10 步向导
    panel_box(d, 20, 150, 330, 300, "① 环几何 / 包信息")
    lx, ly = 40, 196
    for i, (k, v) in enumerate([("包 id", "m04"), ("环心 x / y", "540 / 784"), ("半径 R", "480"), ("卡宽 / 高", "96 / 120"), ("中央岛", "island_home_m04")]):
        d.text((lx, ly), k, font=font(12), fill=MUTED)
        d.text((lx+130, ly-2), v, font=font(14, True), fill=INK)
        ly += 34
    d.text((lx, ly+6), "改环几何须 8 整除（I6；center_x 屏心 540 豁免）", font=font(10), fill=MUTED)
    panel_box(d, 20, 470, 330, 470, "② 制作流程（10 步 · 不可跳步）")
    ly = 520
    for i, (s, desc) in enumerate([("add 建包", "从 _template 骨架"), ("import 填内容", "编辑器 → 落盘"), ("syncmap 同步", "manifest[map]←map.json"), ("verify 校验", "I1~I9 终判"), ("pack 打包", "dist/ 可分发件"), ("install 安装", "回注 board.json"), ("check 对账", "0 漂移"), ("sync 回注", "全量重算"), ("shot 出图", "真渲染签收"), ("zero_rust", "游戏 crate 0 diff")]):
        mark = "◉" if i < 3 else ("◎" if i == 3 else "○")
        mc = GREEN if i < 3 else (GOLD if i == 3 else MUTED)
        d.text((40, ly), mark, font=font(14, True), fill=mc)
        d.text((64, ly), s, font=font(12, True), fill=INK)
        d.text((64, ly+17), desc, font=font(10), fill=MUTED)
        ly += 40
    # 中列：环形画布（大）
    cx, cy = 20 + 330 + 60 + 300, 150 + 560
    panel_box(d, 370, 150, 760, 820, "③ 环形画布 · 31 格（点领地格选岛 / 拖拽自由摆放）")
    draw_ring(d, cx+180, cy-80, 300, 74, 92, sel_idx=8, drag=(cx+360, cy-360))
    legend(d, 390, 820, [
        ("起点（绿）/ BOSS（红）", KCOL["start"]),
        ("领地格·已占 = 岛色带", ISLE["harbour"]),
        ("领地格·无主（灰）", GRAY),
        ("功能格·不可占（事件/卡/宝）", KCOL["event"]),
    ])
    # 右列：节点属性 + 内容配置 + 校验
    rx = 1150
    panel_box(d, rx, 150, 430, 300, "④ 节点属性（选中 格8 · wreck）")
    ry = 196
    for k, v in [("kind", "territory ▼"), ("isle", "wreck（沉船滩）"), ("resource", "gold（随岛型绑定）"), ("building", "1（上限 3）"), ("group", "3（同岛 2 格）"), ("pos", "（未设 = 按环排布）")]:
        d.text((rx+20, ry), k, font=font(12), fill=MUTED)
        d.text((rx+150, ry), v, font=font(13), fill=INK)
        ry += 30
    d.text((rx+20, ry+4), "点画布领地格 → 弹 10 类岛面板自动配 resource/building", font=font(10), fill=ACCENT)
    panel_box(d, rx, 470, 430, 200, "⑤ 内容配置（I9 + S34.5）")
    ry = 516
    d.text((rx+20, ry), "events[]  (2 条)", font=font(12, True), fill=INK); d.text((rx+20, ry+18), "m04_iceberg·shipwreck / m04_arctic_wind·wind_god", font=font(11), fill=MUTED)
    d.text((rx+20, ry+52), "merchants[]  weapon·slave ✓", font=font(12, True), fill=INK)
    d.text((rx+20, ry+80), "vs_theme  bg=vs_bg_m04 · tint=0x78FF", font=font(12, True), fill=INK)
    check_bar(d, rx, 690, 430, n_fail=1, fail_items=["I3 岛 volcano = 3 格（重分配未成对改）", "（修：改 A→B 两格，再改 B→A 两格）"])
    c.save(OUT / "map_editor_redesign_A.png")


# ================================================================== 方向 B 地图为主
def concept_B():
    W, H = 1600, 1000
    c, d = new_canvas(W, H)
    # 全屏羊皮海图底
    d.rectangle([0, 0, W, H], fill=(30, 34, 44))
    # 顶部工具栏
    d.rectangle([0, 0, W, 64], fill=(22, 26, 34))
    d.text((20, 18), "地图编辑器", font=font(20, True), fill=INK)
    d.text((20, 42), "全屏画布 · 沉浸编辑；右下角浮层属性 · 顶部工具栏操作", font=font(11), fill=MUTED)
    for i, btn in enumerate(["导入", "校验", "出包", "刷新", "撤销/重做"]):
        bx = W - 220 + i*44
        d.rounded_rectangle([bx, 14, bx+40, 48], radius=6, fill=ACCENT if i == 1 else PANEL2, outline=BORDER)
        d.text((bx+4, 22), btn, font=font(11, True), fill=(16, 20, 30) if i == 1 else INK)
    # 大画布（全屏）
    cx, cy = W//2 - 200, 560
    draw_ring(d, cx, cy, 360, 84, 104, sel_idx=14, drag=None)
    # 左上：步骤条（悬浮）
    step_strip(d, 20, 84, 560, cur=3)
    # 右下：浮层属性卡（半透明感）
    fx, fy = W - 400, H - 420
    d.rounded_rectangle([fx, fy, fx+360, fy+380], radius=10, fill=(24, 28, 38), outline=ACCENT, width=2)
    d.text((fx+16, fy+12), "节点属性 · 格14", font=font(14, True), fill=ACCENT)
    ry = fy + 44
    for k, v in [("isle", "volcano（火山岛）"), ("resource", "gold"), ("building", "1 / 3"), ("group", "8"), ("status", "已占 · 色带")]:
        d.text((fx+16, ry), k, font=font(12), fill=MUTED)
        d.text((fx+150, ry), v, font=font(13, True), fill=INK)
        ry += 30
    # 校验角标（右下浮层下方）
    check_bar(d, fx, fy+392, 360, n_fail=0)
    # 左下：10 类岛抽屉（竖排）
    d.rounded_rectangle([20, H-420, 200, H-20], radius=10, fill=(24, 28, 38), outline=BORDER, width=2)
    d.text((34, H-408), "10 类岛（拖到格）", font=font(12, True), fill=MUTED)
    for i, isle in enumerate(list(ISLE.keys())):
        yy = H - 384 + i*38
        d.rectangle([34, yy, 60, yy+26], fill=ISLE[isle], outline=(16, 20, 30))
        d.text((68, yy+3), isle, font=font(11), fill=INK)
    c.save(OUT / "map_editor_redesign_B.png")


# ================================================================== 方向 C 卡片流
def concept_C():
    W, H = 1600, 1000
    c, d = new_canvas(W, H)
    title_bar(d, W, "地图编辑器 · 卡片流（自上而下顺序读）", "每张卡 = 一个职责；顶部只留「校验状态 + 下一步操作」，降低认知负荷")
    # 顶部：一条校验 + 主操作（替代全屏分散）
    check_bar(d, 20, 66, 1100, n_fail=1, fail_items=["I3 岛 volcano = 3 格（期望 2）—— 重分配须成对改", "（当前可导入 = I1~I2/I5/I6/I8/I9/S34.5 全过）"])
    d.rounded_rectangle([1150, 66, 1340, 156], radius=8, fill=ACCENT)
    d.text((1170, 84), "校验 I1~I9", font=font(14, True), fill=(16, 20, 30))
    d.rounded_rectangle([1360, 66, 1580, 156], radius=8, fill=GREEN)
    d.text((1380, 84), "导入落盘", font=font(14, True), fill=(16, 20, 30))
    d.text((1380, 116), "POST /api/map/import", font=font(10), fill=(20, 40, 30))
    # 卡 1：环形画布（横卡，左画布 + 右属性）
    panel_box(d, 20, 176, 1560, 430, "① 环形画布（31 格）· 点领地格选岛 / 拖拽自由摆放 / 右侧属性同步")
    draw_ring(d, 460, 430, 220, 64, 82, sel_idx=8, drag=None)
    legend(d, 240, 620, [("无主灰", GRAY), ("已占岛色带", ISLE["wreck"]), ("功能格", KCOL["event"])])
    # 右侧属性 + 岛选择网格（同卡内）
    ax = 820
    d.text((ax, 200), "节点属性 · 格8（wreck 沉船滩）", font=font(14, True), fill=ACCENT)
    ay = 236
    cols = [("kind","territory"), ("isle","wreck"), ("resource","gold"), ("building","1"), ("group","3"), ("pos","（环）")]
    for i, (k, v) in enumerate(cols):
        bx = ax + (i%3)*160
        by = ay + (i//3)*54
        d.rounded_rectangle([bx, by, bx+150, by+44], radius=6, fill=PANEL2, outline=BORDER)
        d.text((bx+10, by+6), k, font=font(10), fill=MUTED)
        d.text((bx+10, by+22), v, font=font(14, True), fill=INK)
    # 岛选择网格（10 格 5×2）
    d.text((ax, ay+120), "选岛（10 类闭集 · 点选自动配 resource/building）", font=font(12, True), fill=MUTED)
    for i, isle in enumerate(list(ISLE.keys())):
        bx = ax + (i%5)*120
        by = ay + 148 + (i//5)*64
        on = isle == "wreck"
        d.rounded_rectangle([bx, by, bx+110, by+54], radius=6,
                            fill=(PANEL2 if not on else (60, 70, 90)), outline=(ACCENT if on else BORDER), width=2 if on else 1)
        d.rectangle([bx+8, by+8, bx+30, by+32], fill=ISLE[isle])
        d.text((bx+38, by+8), isle[:10], font=font(10, True), fill=INK)
        d.text((bx+8, by+36), ED_RES[isle] if False else isle, font=font(9), fill=MUTED)
    # 卡 2：内容配置（events/merchants/vs_theme）
    panel_box(d, 20, 626, 1560, 180, "② 内容配置（I9 events / merchants + S34.5 vs_theme）· m04 事故根因已修")
    cx = 40
    d.text((cx, 664), "随机事件 events[]", font=font(12, True), fill=INK)
    d.text((cx, 688), "m04_iceberg·shipwreck　m04_arctic_wind·wind_god(+3)", font=font(11), fill=MUTED)
    d.rounded_rectangle([cx+560, 684, cx+640, 706], radius=4, fill=PANEL2, outline=BORDER)
    d.text((cx+572, 688), "＋加事件", font=font(10), fill=INK)
    d.text((cx+700, 664), "商人 merchants[]", font=font(12, True), fill=INK)
    d.text((cx+700, 688), "weapon·cell[15,2] ✓　slave·cell[15,16] ✓", font=font(11), fill=MUTED)
    d.text((cx+1000, 664), "VS 主题 vs_theme", font=font(12, True), fill=INK)
    d.text((cx+1000, 688), "bg=vs_bg_m04 siege·dungeon·boss=共享池 encounter=vs_bg_m04", font=font(11), fill=MUTED)
    d.rounded_rectangle([cx+1000, 712, cx+1200, 734], radius=4, fill=PANEL2, outline=BORDER)
    d.text((cx+1012, 716), "tint = 0x78FF（RGB565）", font=font(11), fill=INK)
    # 卡 3：10 步向导（横排进度）
    panel_box(d, 20, 826, 1560, 154, "③ 制作向导（10 步 · 失败即停 · 写步需 X-TI-Token）")
    step_strip(d, 40, 862, 1520, cur=3)
    d.text((40, 966), "▶ 执行下一步（POST /api/content/map/step）　　当前 = 第 4 步 verify（判据：0 违规）　　失败即停在此步", font=font(12), fill=ACCENT)
    c.save(OUT / "map_editor_redesign_C.png")


ED_RES = {}


def check():
    for f in ["map_editor_redesign_A.png", "map_editor_redesign_B.png", "map_editor_redesign_C.png"]:
        p = OUT / f
        if not p.is_file() or p.stat().st_size < 2048:
            print(f"[FAIL] {f} 缺/过小", file=sys.stderr)
            return 1
    print(f"map_editor_redesign_concepts check: 0 缺（3 方向概念稿在位非空）")
    return 0


def gen():
    OUT.mkdir(parents=True, exist_ok=True)
    concept_A()
    concept_B()
    concept_C()
    print("map_editor_redesign_concepts: 生成 3 方向 →", OUT.relative_to(ROOT))
    for f in ["map_editor_redesign_A.png", "map_editor_redesign_B.png", "map_editor_redesign_C.png"]:
        print(f"  {f} ({(OUT/f).stat().st_size} B)")
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    return check() if a.check else gen()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
