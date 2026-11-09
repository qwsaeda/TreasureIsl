#!/usr/bin/env python3
"""map_editor_tile_concepts.py — 地图格子「创意形状」概念图（对齐真渲染 v6 不规则多边形）。

用户 2026-11-09：「格子为什么用卡片？去创意生成不同格子」——当前编辑器/概念图用
统一矩形卡片（S27.1 旧定案 96×120 卡），但**真渲染 v6 已是 seed 随机 6 顶点不规则多边形**
（`main_map.rs::v6_tile_polygon`：每格形状差异大 + 色带圆环 + 中央图标 + 名牌）。
核心 = 编辑器画布/设计渲染图与真渲染脱节 ⇒ 概念图应显示「真不规则格」，所见即所得。

本脚本生成 4 张 1280×800 概念图（创意格子形状方案）：
  A 真渲染对齐（推荐）：seed 随机 6 顶点多边形 + 色带圆环 + 中央岛图标 + 名牌
     （逐格 seed 差异 ⇒ 每格形状唯一；与 main_map.rs v6_tile_polygon 同算法）
  B 拼图块（四边凸舌/凹槽）：随机边加半圆凸舌或凹槽，格间咬合，棋盘拼图感
  C 圆形徽章（同心圆 + 外环刻度）：纯圆 + 色带外环 + 刻度齿，简洁
  D 菱形/六边形蜂窝：六边形平铺 + 色带边 + 中央图标，棋盘格规整

工具契约（14 号 §1.2）：无参 = 生成 4 图；--check = 只读校验产物在位非空。
"""
import argparse
import math
import random
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "ui_concept"

BG = (20, 24, 33)
PANEL = (30, 36, 48)
PANEL2 = (40, 48, 62)
BORDER = (66, 76, 94)
INK = (226, 230, 242)
MUTED = (148, 156, 174)
ACCENT = (120, 196, 224)
GOLD = (200, 160, 88)
GREEN = (70, 190, 108)
GRAY = (78, 86, 104)
SEA = (26, 52, 66)
PARCH = (222, 210, 176)
PARCH_DK = (190, 176, 142)

# 岛色带（theme.json isle_* 近似）
ISLE = {"harbour": (200, 122, 110), "lighthouse": (224, 176, 112), "wreck": (232, 192, 64),
        "pirate_lair": (58, 58, 92), "shipyard": (62, 138, 120), "trading_post": (140, 122, 60),
        "skull_cliff": (232, 228, 208), "volcano": (216, 92, 48), "lightship": (122, 140, 160),
        "boss_fort": (56, 40, 74)}
ISLE_ZH = {"harbour": "港口城", "lighthouse": "灯塔礁", "wreck": "沉船滩", "pirate_lair": "海盗老巢",
           "shipyard": "帆船坞", "trading_post": "军需商岛", "skull_cliff": "骷髅崖",
           "volcano": "火山岛", "lightship": "灯船站", "boss_fort": "无畏号要塞"}
KCOL = {"start": (63, 185, 80), "boss": (208, 84, 84), "event": (122, 106, 154),
        "card": (122, 106, 154), "treasure": (122, 106, 154)}
KLET = {"start": "S", "boss": "B", "event": "E", "card": "C", "treasure": "T"}

KIND = ['start', 'territory', 'territory', 'event', 'territory', 'territory', 'card',
        'territory', 'territory', 'treasure', 'territory', 'territory', 'event',
        'territory', 'territory', 'boss', 'territory', 'territory', 'event', 'territory',
        'territory', 'treasure', 'territory', 'territory', 'card', 'territory', 'territory',
        'territory', 'territory', 'event', 'card']
ISLE_SEQ = [None, 'harbour', 'harbour', None, 'lighthouse', 'lighthouse', None, 'wreck', 'wreck', None,
            'pirate_lair', 'pirate_lair', None, 'shipyard', 'shipyard', None, 'trading_post', 'trading_post',
            None, 'skull_cliff', 'skull_cliff', None, 'volcano', 'volcano', None, 'lightship', 'lightship',
            'boss_fort', 'boss_fort', None, None]


def font(size, bold=False):
    for p in (["/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
               "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"] if bold
              else ["/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
                    "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"]):
        try:
            return ImageFont.truetype(p, size)
        except Exception:
            continue
    return ImageFont.load_default()


def title_bar(d, W, name, sub):
    d.rectangle([0, 0, W, 42], fill=PANEL2)
    d.text((16, 11), name, font=font(16, True), fill=INK)
    d.text((W - 320, 14), sub, font=font(11), fill=MUTED)


def ring_positions(W, H, n=31, R=None, cx=None, cy=None):
    cx = cx if cx is not None else W / 2
    cy = cy if cy is not None else H / 2 + 20
    R = R if R is not None else min(W, H) * 0.36
    pts = []
    for i in range(n):
        a = math.radians(-90 + i * 360 / n)
        pts.append((cx + R * math.cos(a), cy + R * math.sin(a)))
    return pts, (cx, cy), R


def tile_color(i):
    k = KIND[i]
    isle = ISLE_SEQ[i]
    if k == "territory":
        return ISLE.get(isle, GRAY) if isle else GRAY
    return KCOL.get(k, GRAY)


# ── A 真渲染对齐：seed 随机 6 顶点多边形 ──
def v6_polygon(index, cx, cy, base_r):
    seed = (index * 0x9E3779B9 + 0x12345678) & 0xFFFFFFFF
    n = 6
    pts = []
    for k in range(n):
        seed = (seed * 1664525 + 1013904223) & 0xFFFFFFFF
        ang = -90 + k * 360 / n
        rr = base_r * (0.72 + (seed % 28) / 100.0)
        rad = math.radians(ang)
        pts.append((cx + rr * math.cos(rad), cy + rr * math.sin(rad)))
    return pts


def draw_icon(d, cx, cy, size, isle, color):
    # 简化中央图标（色块 + 文字标签）
    d.ellipse([cx - size / 2, cy - size / 2, cx + size / 2, cy + size / 2], fill=color,
              outline=(255, 255, 255), width=2)
    d.text((cx - size / 2 + 4, cy - 6), (ISLE_ZH.get(isle, isle or "?") or "?")[:2], font=font(9, True), fill=(255, 255, 255))


def variant_a():
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), SEA)
    d = ImageDraw.Draw(c)
    title_bar(d, W, "地图格子 · 创意形状 变体 A（真渲染对齐 · seed 随机 6 顶点不规则多边形）",
              "编辑器/概念图从矩形卡 → 真渲染 v6 不规则格 · 所见即所得")
    pts, (cx, cy), R = ring_positions(W, H, R=245)
    d.ellipse([cx - R - 60, cy - R - 60, cx + R + 60, cy + R + 60], outline=(40, 70, 88), width=2)
    d.ellipse([cx - 95, cy - 95, cx + 95, cy + 95], fill=(54, 46, 34), outline=GOLD, width=2)
    d.text((cx - 24, cy - 8), "中央岛", font=font(13, True), fill=GOLD)
    for i in range(31):
        x, y = pts[i]
        poly = v6_polygon(i, x, y, 32)
        col = tile_color(i)
        # 投影
        sh = [(px, py + 3) for px, py in poly]
        d.polygon(sh, fill=(12, 20, 26))
        # 主体羊皮卡面
        d.polygon(poly, fill=PARCH, outline=(30, 36, 46))
        # 色带圆环
        d.ellipse([x - 20, y - 20, x + 20, y + 20], fill=col)
        d.ellipse([x - 15, y - 15, x + 15, y + 15], fill=PARCH)
        # 中央图标
        k = KIND[i]
        if k == "territory":
            draw_icon(d, x, y, 22, ISLE_SEQ[i], col)
        else:
            d.text((x - 5, y - 7), KLET.get(k, "?"), font=font(12, True), fill=col)
        # 名牌
        nm = ISLE_ZH.get(ISLE_SEQ[i], ISLE_SEQ[i]) if k == "territory" else k
        d.text((x - 18, y + 36), (nm or "")[:4], font=font(8), fill=(200, 210, 220))
    d.text((30, H - 26), "每格 seed 差异 ⇒ 形状唯一 · 色带圆环 = 岛/功能语义色 · 中央图标 = 岛型识别",
           font=font(11), fill=MUTED)
    c.save(OUT / "map_editor_tile_concept_a.png")
    return OUT / "map_editor_tile_concept_a.png"


# ── B 拼图块（四边凸舌/凹槽）──
def jigsaw_polygon(index, cx, cy, base_r):
    seed = (index * 0x85EBCA6B + 0xC2B2AE35) & 0xFFFFFFFF
    pts = []
    n = 4
    for k in range(n):
        seed = (seed * 1664525 + 1013904223) & 0xFFFFFFFF
        ang = -90 + k * 360 / n
        rad = math.radians(ang)
        # 每边中点：凸舌或凹槽（seed 决定）
        seed2 = (seed * 1103515245 + 12345) & 0xFFFFFFFF
        mid_r = base_r * (1.25 if seed2 % 2 == 0 else 0.95)
        mid_ang = ang + 180 / n
        midrad = math.radians(mid_ang)
        pts.append((cx + base_r * math.cos(rad), cy + base_r * math.sin(rad)))
        pts.append((cx + mid_r * math.cos(midrad), cy + mid_r * math.sin(midrad)))
    return pts


def variant_b():
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), SEA)
    d = ImageDraw.Draw(c)
    title_bar(d, W, "地图格子 · 创意形状 变体 B（拼图块 · 四边凸舌/凹槽咬合）",
              "格间咬合棋盘拼图感 · 每边随机凸/凹")
    pts, (cx, cy), R = ring_positions(W, H, R=245)
    d.ellipse([cx - R - 60, cy - R - 60, cx + R + 60, cy + R + 60], outline=(40, 70, 88), width=2)
    d.ellipse([cx - 95, cy - 95, cx + 95, cy + 95], fill=(54, 46, 34), outline=GOLD, width=2)
    d.text((cx - 24, cy - 8), "中央岛", font=font(13, True), fill=GOLD)
    for i in range(31):
        x, y = pts[i]
        poly = jigsaw_polygon(i, x, y, 28)
        col = tile_color(i)
        d.polygon(poly, fill=PARCH, outline=(30, 36, 46))
        d.ellipse([x - 19, y - 19, x + 19, y + 19], fill=col)
        d.ellipse([x - 14, y - 14, x + 14, y + 14], fill=PARCH)
        k = KIND[i]
        if k == "territory":
            draw_icon(d, x, y, 20, ISLE_SEQ[i], col)
        else:
            d.text((x - 5, y - 7), KLET.get(k, "?"), font=font(11, True), fill=col)
        nm = ISLE_ZH.get(ISLE_SEQ[i], ISLE_SEQ[i]) if k == "territory" else k
        d.text((x - 16, y + 34), (nm or "")[:4], font=font(8), fill=(200, 210, 220))
    d.text((30, H - 26), "四边随机凸舌（1.25R）/凹槽（0.95R）· 咬合拼图感 · 色带环 + 中央图标",
           font=font(11), fill=MUTED)
    c.save(OUT / "map_editor_tile_concept_b.png")
    return OUT / "map_editor_tile_concept_b.png"


# ── C 圆形徽章 ──
def variant_c():
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), SEA)
    d = ImageDraw.Draw(c)
    title_bar(d, W, "地图格子 · 创意形状 变体 C（圆形徽章 · 同心圆 + 外环刻度）",
              "简洁圆徽章 · 外环色带 + 刻度齿")
    pts, (cx, cy), R = ring_positions(W, H, R=245)
    d.ellipse([cx - R - 60, cy - R - 60, cx + R + 60, cy + R + 60], outline=(40, 70, 88), width=2)
    d.ellipse([cx - 95, cy - 95, cx + 95, cy + 95], fill=(54, 46, 34), outline=GOLD, width=2)
    d.text((cx - 24, cy - 8), "中央岛", font=font(13, True), fill=GOLD)
    for i in range(31):
        x, y = pts[i]
        col = tile_color(i)
        r0 = 32
        # 刻度齿（外环 8 齿）
        for t in range(8):
            a = math.radians(t * 45)
            x1 = x + (r0 + 4) * math.cos(a)
            y1 = y + (r0 + 4) * math.sin(a)
            d.ellipse([x1 - 3, y1 - 3, x1 + 3, y1 + 3], fill=col)
        d.ellipse([x - r0, y - r0, x + r0, y + r0], fill=col, outline=(255, 255, 255), width=2)
        d.ellipse([x - 22, y - 22, x + 22, y + 22], fill=PARCH)
        k = KIND[i]
        if k == "territory":
            draw_icon(d, x, y, 24, ISLE_SEQ[i], col)
        else:
            d.text((x - 5, y - 7), KLET.get(k, "?"), font=font(12, True), fill=col)
        nm = ISLE_ZH.get(ISLE_SEQ[i], ISLE_SEQ[i]) if k == "territory" else k
        d.text((x - 16, y + 38), (nm or "")[:4], font=font(8), fill=(200, 210, 220))
    d.text((30, H - 26), "纯圆徽章 + 8 刻度齿外环 + 色带 · 简洁识别强 · 名牌下移",
           font=font(11), fill=MUTED)
    c.save(OUT / "map_editor_tile_concept_c.png")
    return OUT / "map_editor_tile_concept_c.png"


# ── D 六边形蜂窝 ──
def hexagon(cx, cy, r):
    return [(cx + r * math.cos(math.radians(-90 + k * 60)), cy + r * math.sin(math.radians(-90 + k * 60)))
            for k in range(6)]


def variant_d():
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), SEA)
    d = ImageDraw.Draw(c)
    title_bar(d, W, "地图格子 · 创意形状 变体 D（六边形蜂窝 · 棋盘格规整）",
              "六边形平铺 · 色带边 + 中央图标 · 规整棋盘")
    pts, (cx, cy), R = ring_positions(W, H, R=245)
    d.ellipse([cx - R - 60, cy - R - 60, cx + R + 60, cy + R + 60], outline=(40, 70, 88), width=2)
    d.ellipse([cx - 95, cy - 95, cx + 95, cy + 95], fill=(54, 46, 34), outline=GOLD, width=2)
    d.text((cx - 24, cy - 8), "中央岛", font=font(13, True), fill=GOLD)
    for i in range(31):
        x, y = pts[i]
        col = tile_color(i)
        poly = hexagon(x, y, 30)
        d.polygon(poly, fill=PARCH, outline=col, width=3)
        d.ellipse([x - 18, y - 18, x + 18, y + 18], fill=col)
        d.ellipse([x - 13, y - 13, x + 13, y + 13], fill=PARCH)
        k = KIND[i]
        if k == "territory":
            draw_icon(d, x, y, 20, ISLE_SEQ[i], col)
        else:
            d.text((x - 5, y - 7), KLET.get(k, "?"), font=font(11, True), fill=col)
        nm = ISLE_ZH.get(ISLE_SEQ[i], ISLE_SEQ[i]) if k == "territory" else k
        d.text((x - 16, y + 36), (nm or "")[:4], font=font(8), fill=(200, 210, 220))
    d.text((30, H - 26), "六边形蜂窝平铺 + 色带描边 + 中央图标 · 规整棋盘格 · 相邻咬合",
           font=font(11), fill=MUTED)
    c.save(OUT / "map_editor_tile_concept_d.png")
    return OUT / "map_editor_tile_concept_d.png"


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true")
    a = ap.parse_args(argv)
    targets = ["map_editor_tile_concept_a.png", "map_editor_tile_concept_b.png",
               "map_editor_tile_concept_c.png", "map_editor_tile_concept_d.png"]
    if a.check:
        miss = [t for t in targets if not (OUT / t).is_file() or (OUT / t).stat().st_size < 2048]
        if miss:
            print("check: %d 缺" % len(miss))
            for m in miss:
                print("  [FAIL] " + m)
            return 1
        print("check: 0 缺（4 变体在位非空）")
        return 0
    OUT.mkdir(parents=True, exist_ok=True)
    for fn in (variant_a, variant_b, variant_c, variant_d):
        p = fn()
        print("  %s (%d B)" % (p.name, p.stat().st_size))
    print("map_editor_tile_concepts: 生成 4 变体 → %s" % OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
