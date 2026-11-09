#!/usr/bin/env python3
"""map_editor_redesign_v3_minimal.py — 地图编辑器「极简分步」3 变体概念图。

用户 2026-11-09：「设计太复杂了，重新设计地图制作整个交互」——当前一屏全亮
（左色板 + 中画布 + 右 4 面板 + 10 步条 + 状态条）认知过载。
重新设计核心 = 一次只亮当前步工作区，其余折叠，画布只画环 + 中央岛。

3 变体（1280×800，供 docs/map_editor_guide.md 选）：
  A 向导聚焦（推荐）：顶部细步进条（10 圆点，仅当前高亮）+ 主体 = 当前步专属工作区
     （第 2 步 = 左岛色板 + 中画布 + 右属性，第 4 步 = 中画布 + 右校验清单，
      第 5 步 = 中结果卡 + 右下按钮）+ 底部单行。每步只亮相关区，认知聚焦。
  B 上下分屏：上 = 环形画布（固定，始终可见，所见即所得），下 = 步骤卡片流
     （当前步详情 + 前后步灰显 + 下一步按钮）。画布常驻，步骤驱动下方。
  C 纯步骤卡：无画布，10 张竖排步骤卡（当前展开 = 详情 + 操作 + 结果，其余折叠），
     最轻 = 纯向导 checklist，画布只在第 2 步展开。

工具契约（14 号 §1.2）：无参 = 生成 3 图；--check = 只读校验产物。
"""
import argparse
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "ui_concept"

BG      = (20, 24, 33)
PANEL   = (30, 36, 48)
PANEL2  = (40, 48, 62)
BORDER  = (66, 76, 94)
INK     = (226, 230, 242)
MUTED   = (148, 156, 174)
ACCENT  = (120, 196, 224)
GREEN   = (70, 190, 108)
GOLD    = (200, 160, 88)
RED     = (214, 82, 82)
GRAY    = (78, 86, 104)
PARCH   = (54, 46, 34)

ISLE = {"harbour": (200, 122, 110), "lighthouse": (224, 176, 112), "wreck": (232, 192, 64),
        "pirate_lair": (58, 58, 92), "shipyard": (62, 138, 120), "trading_post": (140, 122, 60),
        "skull_cliff": (232, 228, 208), "volcano": (216, 92, 48), "lightship": (122, 140, 160),
        "boss_fort": (56, 40, 74)}
KCOL = {"start": (63, 185, 80), "boss": (208, 84, 84), "event": (122, 106, 154),
        "card": (122, 106, 154), "treasure": (122, 106, 154)}


def font(size, bold=False):
    # 优先 CJK 字体（wqy-zenhei.ttc 系统真实存在；DejaVu 兜底无 CJK）
    for p in (["/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
               "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
               "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"] if bold
              else ["/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
                    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]):
        try:
            return ImageFont.truetype(p, size)
        except Exception:
            continue
    return ImageFont.load_default()


STEPS = ["add", "import", "syncmap", "verify", "pack", "install", "check", "sync", "shot", "zero_rust"]
STEP_ZH = ["新建包", "填内容", "同步", "校验", "打包", "安装", "对账", "回注", "出图", "零Rust"]


def title_bar(d, W, name, sub):
    d.rectangle([0, 0, W, 42], fill=PANEL2)
    d.text((16, 11), name, font=font(17, True), fill=INK)
    d.text((W - 300, 14), sub, font=font(12), fill=MUTED)
    d.text((W - 130, 13), "包 m04 · 极简分步", font=font(12, True), fill=GOLD)


def ring(d, cx, cy, R, n=31, hi=None):
    """画环形赛道（31 格 + 中央岛）。hi = 高亮格序号（None = 不高亮）。"""
    d.ellipse([cx - R, cy - R, cx + R, cy + R], outline=(54, 64, 82), width=2)
    d.ellipse([cx - 78, cy - 78, cx + 78, cy + 78], fill=PARCH, outline=GOLD, width=2)
    d.text((cx - 24, cy - 10), "中央岛", font=font(12, True), fill=GOLD)
    KIND = ['start', 'territory', 'territory', 'event', 'territory', 'territory', 'card',
            'territory', 'territory', 'treasure', 'territory', 'territory', 'event',
            'territory', 'territory', 'boss', 'territory', 'territory', 'event', 'territory',
            'territory', 'treasure', 'territory', 'territory', 'card', 'territory', 'territory',
            'territory', 'territory', 'event', 'card']
    for i in range(n):
        a = math.radians(-90 + i * 360 / n)
        nx = cx + R * math.cos(a)
        ny = cy + R * math.sin(a)
        w, h = 30, 38
        x0, y0 = nx - w / 2, ny - h / 2
        k = KIND[i]
        col = KCOL.get(k, GRAY) if k != "territory" else (110, 130, 160)
        sel = (i == hi)
        d.rounded_rectangle([x0, y0, x0 + w, y0 + h], radius=4,
                            fill=col, outline=(ACCENT if sel else (24, 28, 38)),
                            width=2 if sel else 1)
        d.text((x0 + 3, y0 + 2), str(i), font=font(9, True), fill=(255, 255, 255))


def stepdots(d, x, y, w, cur, done_n=1, fail_n=None):
    """极简步进条：10 圆点横排，已完成绿 / 当前金 / 其余灰。"""
    n = 10
    gap = w / n
    for i in range(n):
        cx = x + gap * i + gap / 2
        col = GREEN if i < done_n else (GOLD if i == cur else GRAY)
        d.ellipse([cx - 11, y - 11, cx + 11, y + 11], fill=col,
                  outline=ACCENT if i == cur else None, width=2 if i == cur else 0)
        d.text((cx - 8, y - 6), str(i + 1), font=font(11, True), fill=(16, 20, 30) if i in (done_n, cur) else MUTED)


# ================================================================= 变体 A：向导聚焦
def variant_a():
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(c)
    title_bar(d, W, "地图编辑器 · Dev Console「地图」页（极简分步 · 变体 A 向导聚焦）",
              "一次只亮当前步工作区")
    # 顶部细步进条（10 圆点）
    d.rounded_rectangle([12, 56, W - 12, 120], radius=8, fill=PANEL, outline=BORDER)
    stepdots(d, 40, 88, W - 80, cur=1, done_n=1)
    d.text((40, 98), "第 2/10 步 · 填内容 + 导入落盘", font=font(14, True), fill=ACCENT)
    d.text((W - 240, 100), "前置✓ 已满足 · 下一步 = 同步", font=font(11), fill=MUTED)

    # 主体 = 当前步（第 2 步）专属工作区：左岛色板（窄）+ 中环 + 右属性/内容（窄）
    my, mh = 136, H - 136 - 64
    lw, rw = 210, 270
    mx = 12 + lw + 12
    mw = W - 24 - lw - rw - 24
    # 左：岛色板（仅 10 类小卡）
    d.rounded_rectangle([12, my, 12 + lw, my + mh], radius=8, fill=PANEL, outline=BORDER)
    d.text((24, my + 10), "岛型（10 · 铺格）", font=font(12, True), fill=ACCENT)
    for i, (k, col) in enumerate(list(ISLE.items())):
        col_i, row_i = i % 2, i // 2
        x = 24 + col_i * 100
        y = my + 38 + row_i * 54
        on = (k == "wreck")
        d.rounded_rectangle([x, y, x + 92, y + 46], radius=5,
                            fill=(56, 66, 88) if on else PANEL2, outline=ACCENT if on else BORDER,
                            width=2 if on else 1)
        d.rectangle([x + 8, y + 8, x + 26, y + 26], fill=col)
        d.text((x + 32, y + 6), k[:8], font=font(9, True), fill=INK)
        d.text((x + 32, y + 24), "→铺格", font=font(8), fill=MUTED)
    # 中：环形画布（仅环 + 中央岛，干净）
    d.rounded_rectangle([mx, my, mx + mw, my + mh], radius=8, fill=(14, 18, 26), outline=BORDER)
    d.text((mx + 14, my + 10), "环形赛道 · 点格铺岛", font=font(13, True), fill=INK)
    ring(d, mx + mw / 2, my + mh / 2 + 8, min(mw, mh) * 0.34, hi=8)
    # 右：属性 + 内容（精简）
    rx = mx + mw + 12
    d.rounded_rectangle([rx, my, rx + rw, my + mh], radius=8, fill=PANEL, outline=BORDER)
    d.text((rx + 14, my + 10), "属性 · 格 8", font=font(12, True), fill=ACCENT)
    d.text((rx + 14, my + 38), "isle = wreck（沉船滩）", font=font(11), fill=INK)
    d.text((rx + 14, my + 58), "resource = gold · bldg = 1", font=font(10), fill=MUTED)
    d.text((rx + 14, my + 84), "内容配置", font=font(11, True), fill=ACCENT)
    d.text((rx + 14, my + 106), "events ✓2 · merchants ✓2", font=font(10), fill=GREEN)
    d.text((rx + 14, my + 124), "vs_theme ✓（bg+2 槽）", font=font(10), fill=GREEN)
    # 当前步操作 + 结果（底部卡）
    by = my + mh + 12
    d.rounded_rectangle([12, by, W - 12, by + 40], radius=8, fill=PANEL, outline=BORDER)
    d.rounded_rectangle([24, by + 8, 130, by + 32], radius=5, fill=ACCENT)
    d.text((40, by + 13), "▶ 执行导入", font=font(12, True), fill=(16, 20, 30))
    d.text((150, by + 14), "✓ 本地校验 8/8 · I3 成对 · 导入将落 4 文件", font=font(11), fill=GREEN)
    d.rounded_rectangle([W - 150, by + 8, W - 24, by + 32], radius=5, fill=PANEL2, outline=BORDER)
    d.text((W - 134, by + 13), "下一步 →", font=font(12, True), fill=INK)
    c.save(OUT / "map_editor_v3_minimal_a.png")
    return OUT / "map_editor_v3_minimal_a.png"


# ================================================================= 变体 B：上下分屏
def variant_b():
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(c)
    title_bar(d, W, "地图编辑器 · Dev Console「地图」页（极简分步 · 变体 B 上下分屏）",
              "画布常驻 · 步骤驱动下方")
    # 上 = 环形画布（固定，始终可见）
    top_h = 430
    d.rounded_rectangle([12, 56, W - 12, 56 + top_h], radius=8, fill=(14, 18, 26), outline=BORDER)
    d.text((28, 70), "环形赛道 · 31 格（所见即所得 · 点格铺岛 · 拖格摆位）", font=font(13, True), fill=INK)
    ring(d, W / 2, 56 + top_h / 2 + 30, 175, hi=8)
    # 下 = 步骤卡片流（当前步展开 + 前后灰）
    sy = 56 + top_h + 14
    sh = H - sy - 14
    d.rounded_rectangle([12, sy, W - 12, sy + sh], radius=8, fill=PANEL, outline=BORDER)
    stepdots(d, 40, sy + 26, W - 80, cur=3, done_n=3)
    d.text((40, sy + 44), "第 4/10 步 · 权威校验（I1~I9 + S34.5 + I7 资产哈希）", font=font(14, True), fill=ACCENT)
    d.text((W - 220, sy + 48), "前置✓ · 上一步 0 违规", font=font(11), fill=MUTED)
    # 当前步操作 + 结果
    ry = sy + 84
    d.rounded_rectangle([28, ry, W - 160, ry + 44], radius=6, fill=(34, 52, 42), outline=GREEN)
    d.text((44, ry + 8), "map_pack.py verify m04 → 0 违规（schema/兼容/I1~I9/S34.5）", font=font(12), fill=GREEN)
    d.rounded_rectangle([W - 150, ry + 4, W - 28, ry + 38], radius=5, fill=ACCENT)
    d.text((W - 134, ry + 9), "执行校验", font=font(12, True), fill=(16, 20, 30))
    d.text((28, ry + 52), "下一步 = ⑤ 打包（dist/maps/m04.map-pack + .zip）", font=font(11), fill=MUTED)
    c.save(OUT / "map_editor_v3_minimal_b.png")
    return OUT / "map_editor_v3_minimal_b.png"


# ================================================================= 变体 C：纯步骤卡
def variant_c():
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(c)
    title_bar(d, W, "地图编辑器 · Dev Console「地图」页（极简分步 · 变体 C 纯步骤卡）",
              "10 卡竖排 · 当前展开")
    my = 56
    colw = (W - 24 - 20) / 2  # 两列 5 卡
    for i in range(10):
        col_i = i // 5
        row_i = i % 5
        x = 12 + col_i * (colw + 20)
        y = my + row_i * 140
        cur = (i == 3)
        done = i < 3
        d.rounded_rectangle([x, y, x + colw, y + 132], radius=8,
                            fill=PANEL, outline=ACCENT if cur else BORDER, width=2 if cur else 1)
        # 头部：序号圆 + 名称
        stcol = GREEN if done else (GOLD if cur else GRAY)
        d.ellipse([x + 12, y + 12, x + 40, y + 40], fill=stcol)
        mark = "✓" if done else str(i + 1)
        d.text((x + 21, y + 16), mark, font=font(15, True), fill=(16, 20, 30) if not done else "white")
        d.text((x + 52, y + 14), STEP_ZH[i], font=font(15, True), fill=INK)
        d.text((x + 52, y + 34), STEPS[i] + " · " + "（当前）" if cur else ("（已完成）" if done else ""), font=font(10), fill=ACCENT if cur else MUTED)
        if cur:
            # 当前卡展开：详情 + 操作 + 结果
            d.text((x + 16, y + 56), "权威校验：map_pack.py verify（含 I7 资产哈希，前端镜像不含）", font=font(11), fill=INK)
            d.rounded_rectangle([x + 16, y + 78, x + colw - 16, y + 98], radius=4, fill=(34, 52, 42), outline=GREEN)
            d.text((x + 26, y + 82), "✓ 0 违规（上一步已 verify 通过）", font=font(11), fill=GREEN)
            d.rounded_rectangle([x + 16, y + 106, x + 130, y + 124], radius=4, fill=ACCENT)
            d.text((x + 30, y + 109), "执行校验", font=font(11, True), fill=(16, 20, 30))
        else:
            d.text((x + 16, y + 58), {"add": "建 4 文件骨架（已装幂等过）", "import": "编辑器填 31 格 + 内容 + 导入",
                "syncmap": "manifest[map]←map.json（0 差）", "pack": "出 dist 可分发件",
                "install": "装 + 接线 board.json", "check": "漂移对账（0 漂移）",
                "sync": "回注构建期默认数据", "shot": "真渲染截图 + 非黑", "zero_rust": "git diff crates/ 空"
            }.get(STEPS[i], ""), font=font(10), fill=MUTED)
    c.save(OUT / "map_editor_v3_minimal_c.png")
    return OUT / "map_editor_v3_minimal_c.png"


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    targets = ["map_editor_v3_minimal_a.png", "map_editor_v3_minimal_b.png", "map_editor_v3_minimal_c.png"]
    if a.check:
        miss = [t for t in targets if not (OUT / t).is_file() or (OUT / t).stat().st_size < 2048]
        if miss:
            print("check: %d 缺" % len(miss))
            for m in miss:
                print("  [FAIL] " + m)
            return 1
        print("check: 0 缺（3 变体在位非空）")
        return 0
    OUT.mkdir(parents=True, exist_ok=True)
    print("  " + str(variant_a().name))
    print("  " + str(variant_b().name))
    print("  " + str(variant_c().name))
    print("map_editor_redesign_v3_minimal: 生成 3 变体 → %s" % OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
