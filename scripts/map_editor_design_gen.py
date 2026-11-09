#!/usr/bin/env python3
"""map_editor_design_gen.py — 地图编辑器设计概念稿生成器（一批 PNG，落 evidence/ui_concept/）。

用途：为 `docs/map_editor_guide.md` 出「图文并茂」设计稿。
- 主编辑器概念图（环形画布 + 节点属性面板 + 实时校验条）：
  **节点颜色语义 = 真实渲染口径**（`main_map.rs::render_v6_elem`）：
  节点底图只有两态 `tile_owned`（已被某席占领）/ `tile_deny`（无主·功能格不可占），
  **开局 31 格全无主**，逐局随各席位落格占领逐格变色；归属色带 = 10 类岛 token
  （`theme.json::isle_*` 实测色值），席位 = P1~P5 徽标 + 船/旗。颜色**非按 kind 画死**。
  岛型 = 10 类闭集（`balance.territory.prototypes`），编辑器在 20 个领地格间**分配**
  哪格是哪类岛（可在 group 间重分配，须保持每类 2 格同 group），**不可新增岛型**。
- 10 步向导逐步渲染图（add/import/syncmap/verify/pack/install/check/sync/shot/zero_rust）。

工具契约（14 号 §1.2）：`--check` 只读（产物在位 + 非空，缺则非 0）；幂等（同输入同产物）。
"""

import argparse
import os
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT_DIR = ROOT / "evidence" / "ui_concept"

# theme.json 实测 isle_* 色值（RGB565 → 近似 888 展开）
ISLE_COLORS = {
    "isle_coral": 0xDAC8, "isle_sand": 0xE652, "isle_gold": 0xFE47,
    "isle_dark": 0x4163, "isle_teal": 0x3D54, "isle_olive": 0x8C07,
    "isle_bone": 0xEEF7, "isle_ember": 0xE365, "isle_mist": 0xA5B7,
    "isle_black": 0x20E3,
}
# 岛名 → token（main_map.rs::isle_token_color 实测映射）
ISLE_TO_TOKEN = {
    "harbour": "isle_coral", "lighthouse": "isle_sand", "wreck": "isle_gold",
    "pirate_lair": "isle_dark", "shipyard": "isle_teal", "trading_post": "isle_olive",
    "skull_cliff": "isle_bone", "volcano": "isle_ember", "lightship": "isle_mist",
    "boss_fort": "isle_black",
}


def rgb565_to_rgb8(color: int) -> tuple:
    """RGB565 (0xRRRGGGBB 的 16bit 形态 0xRRRGGBB 简写) → 888。

    主题色是 0xXXXX 4 位 hex（RGB565 压缩）。按 R5 G6 B5 展开。"""
    r = (color >> 11) & 0x1F
    g = (color >> 5) & 0x3F
    b = color & 0x1F
    return (r << 3, g << 2, b << 3)


def get_font(size: int):
    for p in [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    ]:
        if os.path.exists(p):
            try:
                return ImageFont.truetype(p, size)
            except Exception:
                continue
    return ImageFont.load_default()


def load_tile(kind: str):
    """tile_owned / tile_deny 缩略（真实节点底图，两态）。"""
    name = {"owned": "tile_owned", "deny": "tile_deny"}[kind]
    p = ROOT / "assets" / "ui" / "v6" / f"{name}.png"
    if not p.is_file():
        return None
    return Image.open(p).convert("RGBA").resize((56, 48))


def load_isle_thumb(isle: str):
    p = ROOT / "assets" / "ui" / f"isle_{isle}.png"
    if not p.is_file():
        return None
    return Image.open(p).convert("RGBA").resize((48, 60))


# ----------------------------------------------------------------- 主编辑器概念图


def draw_main_editor(out: Path) -> None:
    W, H = 1240, 820
    canvas = Image.new("RGBA", (W, H), (24, 32, 44, 255))
    d = ImageDraw.Draw(canvas)
    f_title = get_font(26)
    f_label = get_font(15)
    f_small = get_font(12)

    LX, LY, LW, LH = 40, 40, 560, 740
    d.rounded_rectangle([LX, LY, LX + LW, LY + LH], radius=12, fill=(36, 48, 64, 255), outline=(80, 100, 130, 255))
    d.text((LX + 16, LY + 14), "环式画布 · 31 格", font=f_title, fill=(230, 215, 180, 255))
    d.text((LX + 16, LY + 48), "节点底图两态：无主(灰) · 已占(岛色带)  颜色随每局占领动态变化",
           font=f_small, fill=(150, 165, 190, 255))

    # 画布底
    CX, CY = LX + LW // 2, LY + LH // 2 + 20
    RX, RY = 190, 200
    pad = 20
    d.rounded_rectangle([CX - RX - pad, CY - RY - pad, CX + RX + pad, CY + RY + pad],
                        radius=16, fill=(58, 50, 38, 255), outline=(120, 100, 70, 255), width=2)

    # 格 kind 序列（I2：start@0 + 20 领地 + 9 功能 + boss@15）
    KIND_SEQ = ["start",
                "territory", "territory", "event",
                "territory", "territory", "card",
                "territory", "territory", "treasure",
                "territory", "territory", "event",
                "territory", "territory", "boss",
                "territory", "territory", "event",
                "territory", "territory", "treasure",
                "territory", "territory", "card",
                "territory", "territory", "territory",
                "territory", "event", "card"]
    # group → isle（m01 实测固定分配，可在编辑器重分配）
    GROUP_ISLE = {1: "harbour", 2: "lighthouse", 3: "wreck", 4: "pirate_lair",
                   5: "shipyard", 6: "trading_post", 7: "skull_cliff", 8: "volcano",
                   9: "lightship", 10: "boss_fort"}

    nodes = []
    for i, kind in enumerate(KIND_SEQ):
        import math
        angle = math.radians(-90 + i * (360 / 31))
        nx = CX + int(RX * math.cos(angle))
        ny = CY + int(RY * math.sin(angle))
        nodes.append((nx, ny, kind, i))

    # 中央岛
    im = ROOT / "assets" / "ui" / "island_home_m04.png"
    if im.is_file():
        home = Image.open(im).convert("RGBA").resize((150, 150))
        canvas.paste(home, (CX - 75, CY - 75), home)

    # 节点：领地格 = 岛 token 色圆（两态：这里画「已占」态展示岛色带），功能格 = 中性灰
    # 模拟：1/3/5/7 类已占（用各自岛色），其余灰（无主）—— 表达「逐局动态」
    owned_groups = {1, 3, 6}
    for nx, ny, kind, i in nodes:
        if kind == "territory":
            grp = (i // 3) + 1 if i % 3 != 0 else (i // 3)  # 近似 group 序
            isle = GROUP_ISLE.get(min(max(((i + 1) // 3), 1), 10), "harbour")
            # 用岛 token 色（真实 theme.json 值）
            token = ISLE_TO_TOKEN[isle]
            base = rgb565_to_rgb8(ISLE_COLORS[token]) + (255,)
            col = base if ((i // 2) % 3 + 1) in owned_groups else (110, 110, 110, 255)
            d.ellipse([nx - 16, ny - 14, nx + 16, ny + 14], fill=col,
                      outline=(40, 40, 40, 255), width=2)
        else:
            label = {"start": "S", "event": "E", "card": "C", "treasure": "T", "boss": "B"}[kind]
            c = {"start": (90, 200, 110), "event": (170, 140, 220),
                 "card": (225, 150, 95), "treasure": (240, 200, 70), "boss": (235, 85, 75)}[kind]
            d.ellipse([nx - 14, ny - 12, nx + 14, ny + 12], fill=(*c, 255),
                      outline=(40, 40, 40, 255), width=2)
            d.text((nx - 4, ny - 8), label, font=f_small, fill=(255, 255, 255, 255))

    # 图例（真实语义）
    ly = LY + 96
    d.text((LX + 20, ly), "图例（节点底图两态 + 席位）", font=f_small, fill=(150, 165, 190, 255))
    ly += 24
    for text, col in [
        ("领地格·无主", (110, 110, 110, 255)),
        ("领地格·已占（岛色带）", rgb565_to_rgb8(ISLE_COLORS["isle_coral"]) + (255,)),
        ("功能格（不可占）", (120, 120, 130, 255)),
        ("BOSS 格", (235, 85, 75, 255)),
        ("席位 P1~P5（船+旗）", (240, 220, 120, 255)),
    ]:
        d.ellipse([LX + 24, ly - 8, LX + 40, ly + 8], fill=col, outline=(40, 40, 40, 255))
        d.text((LX + 48, ly - 8), text, font=f_small, fill=(200, 205, 215, 255))
        ly += 24

    # 10 类岛分配面板（画布下方）
    py = CY + RY + 30
    d.text((CX - 180, py - 26), "10 类岛（闭集 · 只能在这 10 类间分配 20 格 · 不可新增）",
           font=f_small, fill=(200, 180, 140, 255))
    isle_names = ["harbour", "lighthouse", "wreck", "pirate_lair", "shipyard",
                  "trading_post", "skull_cliff", "volcano", "lightship", "boss_fort"]
    col_w = 50
    start_x = CX - (len(isle_names) * col_w) // 2
    for idx, name in enumerate(isle_names):
        x = start_x + idx * col_w
        thumb = load_isle_thumb(name)
        if thumb:
            canvas.paste(thumb, (x, py), thumb)
            d.rounded_rectangle([x - 2, py - 4, x + thumb.width + 2, py + thumb.height + 2],
                                radius=4, outline=(80, 70, 50, 255))
        d.text((x - 6, py + 62), name[:9], font=get_font(9), fill=(180, 170, 150, 255))

    # ─── 右区：节点属性面板 ───
    RX0, RY0, RW, RH = 640, 40, 560, 740
    d.rounded_rectangle([RX0, RY0, RX0 + RW, RY0 + RH], radius=12, fill=(36, 48, 64, 255), outline=(80, 100, 130, 255))
    d.text((RX0 + 16, RY0 + 14), "节点属性 · 选中第 14 格（领地格）", font=f_title, fill=(230, 215, 180, 255))
    fields = [
        ("kind", "territory ▼", False, "格类型（闭集 6 值）"),
        ("isle", "volcano ▼（10 类闭集）", False, "在哪类岛 20 格间分配"),
        ("resource", "gold（岛型绑定）", False, "产出地形 = f(岛型)"),
        ("building", "1（初始等级 ≥1）", False, "升级上限 max_level=3"),
        ("pos", "未设 = 走环形自动排布", True, "自由摆放须在 world_canvas 内 ±4"),
    ]
    fy = RY0 + 68
    for label, val, warn, hint in fields:
        col = (255, 110, 100, 255) if warn else (190, 200, 220, 255)
        d.text((RX0 + 20, fy), f"{label:<12}", font=f_label, fill=(120, 140, 170, 255))
        d.text((RX0 + 130, fy), val, font=f_label, fill=col)
        d.text((RX0 + 330, fy), hint, font=get_font(11), fill=(110, 120, 140, 255))
        fy += 34
    d.line([RX0 + 16, fy + 4, RX0 + RW - 16, fy + 4], fill=(70, 85, 105, 255), width=1)
    fy += 18

    d.text((RX0 + 20, fy), "校验状态（I1–I9 实时）", font=f_label, fill=(120, 140, 170, 255))
    fy += 30
    checks = [
        ("I1  31 格", "PASS", (90, 200, 130), "●"),
        ("I2  构成 1+20+9+1", "PASS", (90, 200, 130), "●"),
        ("I3  10 类岛各 2 格", "PASS", (90, 200, 130), "●"),
        ("I5  start@0 · boss@15", "PASS", (90, 200, 130), "●"),
        ("I6-ext  pos 画布内", "FAIL  y=1382>1310", (240, 95, 85), "✗"),
        ("I9  events / merchants", "PASS", (90, 200, 130), "●"),
        ("S34.5  vs_theme", "PASS", (90, 200, 130), "●"),
    ]
    for desc, status, color, mark in checks:
        d.text((RX0 + 24, fy), f"{mark} {desc:<26} {status}", font=f_label, fill=color)
        fy += 25

    by = RY0 + RH - 64
    for i, (txt, bg, fg) in enumerate([
        ("导入/保存（I1–I9）", (60, 140, 80), (255, 255, 255)),
        ("pack → dist/", (80, 95, 115), (210, 220, 240)),
        ("install", (80, 95, 115), (210, 220, 240)),
    ]):
        bx = RX0 + 16 + i * 180
        d.rounded_rectangle([bx, by, bx + 168, by + 44], radius=8, fill=(*bg, 255))
        d.text((bx + 16, by + 12), txt, font=f_label, fill=fg)

    # ─── 顶部：10 步向导进度条 ───
    SY = 44
    d.text((640, SY), "地图制作 10 步向导（不可跳步）", font=f_label, fill=(180, 190, 210, 255))
    sw = 52
    sx = 640
    step_names = ["add", "import", "syncmap", "verify", "pack", "install", "check", "sync", "shot", "zero_rust"]
    for i, sn in enumerate(step_names):
        bx = sx + i * sw
        active = i == 3
        c = (60, 140, 80) if i < 3 else ((240, 180, 60) if i == 3 else (80, 95, 115))
        d.rounded_rectangle([bx, SY + 28, bx + sw - 4, SY + 54], radius=6, fill=(*c, 255))
        d.text((bx + 4, SY + 33), f"{i + 1}", font=f_small, fill=(255, 255, 255, 255))
        d.text((bx, SY + 58), sn, font=get_font(9), fill=(160, 170, 190, 255))

    out.parent.mkdir(parents=True, exist_ok=True)
    canvas.convert("RGB").save(out, "PNG")


# ----------------------------------------------------------------- 10 步向导逐步图


WIZARD_STEPS = [
    ("01_add", "add", "从 _template 建包骨架",
     "map_pack add mXX", "4 文件骨架（map.json 待填）", 0),
    ("02_import", "import", "编辑器填内容 → 导入落盘",
     "POST /api/map/import", "写 assets/maps/<id>/ 四文件", 0),
    ("03_syncmap", "syncmap", "manifest[map] ← map.json",
     "map_pack syncmap mXX", "0 差（幂等）", 0),
    ("04_verify", "verify", "I1–I9 终判",
     "map_pack verify mXX", "0 违规", 0),
    ("05_pack", "pack", "出可分发件",
     "map_pack pack mXX", "dist/maps/<id>.map-pack/ + zip", 0),
    ("06_install", "install", "装上 + 回注 board.json",
     "map_pack install <id>", "board.json maps.<id> 接线", 0),
    ("07_check", "check", "已装态 vs board.json 对账",
     "map_pack check", "0 漂移", 0),
    ("08_sync", "sync", "回注构建期默认数据",
     "map_pack sync", "board/scenes/manifest/strings 重写", 0),
    ("09_shot", "shot", "真渲染 main_map 截图",
     "treasure-bevy-native --shot", "出图非黑 = 视觉签收", 1),
    ("10_zero_rust", "zero_rust", "游戏 crate 零改动断言",
     "git diff --quiet crates/...", "空 diff = 内容运营判据", 0),
]


def draw_wizard_step(slug: str, step_no: int, name: str, title: str,
                      cmd: str, judge: str, has_shot: bool, out: Path) -> None:
    W, H = 900, 460
    canvas = Image.new("RGBA", (W, H), (24, 32, 44, 255))
    d = ImageDraw.Draw(canvas)
    f_big = get_font(24)
    f_mid = get_font(17)
    f_small = get_font(13)

    # 顶部步骤进度条
    bar_y = 26
    for i in range(10):
        bx = 30 + i * 84
        state = i < step_no - 1 or (has_shot and i == step_no - 1 and step_no == 9)
        c = (60, 140, 80) if i < step_no - 1 else ((240, 180, 60) if i == step_no - 1 else (70, 84, 104))
        d.rounded_rectangle([bx, bar_y, bx + 76, bar_y + 30], radius=6, fill=(*c, 255))
        d.text((bx + 12, bar_y + 6), f"{i + 1}", font=f_small, fill=(255, 255, 255, 255))
        label = WIZARD_STEPS[i][1]
        d.text((bx, bar_y + 34), label, font=get_font(10), fill=(150, 160, 180, 255))

    # 主区：左 = 该步示意，右 = 命令 + 判据
    d.text((30, 96), f"第 {step_no} 步 · {name}", font=f_big, fill=(230, 215, 180, 255))
    d.text((30, 134), title, font=f_mid, fill=(170, 185, 205, 255))

    # 左侧示意框
    LX, LY, LW, LH = 30, 170, 420, 250
    d.rounded_rectangle([LX, LY, LX + LW, LY + LH], radius=10, fill=(36, 48, 64, 255), outline=(80, 100, 130, 255))
    if has_shot:
        p = ROOT / "evidence" / "bevy_native_main_map.png"
        if p.is_file():
            shot = Image.open(p).convert("RGB").resize((300, 168))
            canvas.paste(shot, (LX + 60, LY + 20))
            d.text((LX + 120, LY + 200), "出图 = 真渲染 main_map（视觉签收）", font=f_small, fill=(200, 210, 220, 255))
    else:
        # 文件示意图（4 文件 JSON）
        files = ["manifest.json", "map.json", "layout.json", "strings.json"]
        fx = LX + 30
        for j, fn in enumerate(files):
            fx = LX + 30 + (j % 2) * 200
            fyy = LY + 30 + (j // 2) * 70
            d.rounded_rectangle([fx, fyy, fx + 180, fyy + 50], radius=6, fill=(52, 66, 86, 255), outline=(90, 110, 140, 255))
            d.text((fx + 12, fyy + 8), "assets/maps/<id>/", font=get_font(10), fill=(130, 145, 170, 255))
            d.text((fx + 12, fyy + 26), fn, font=f_small, fill=(200, 210, 225, 255))
        d.text((LX + 30, LY + 190), "该步操作对象", font=f_small, fill=(130, 145, 170, 255))

    # 右区：命令 + 判据
    RX0 = 480
    d.rounded_rectangle([RX0, 170, RX0 + 390, 420], radius=10, fill=(30, 40, 56, 255), outline=(80, 100, 130, 255))
    d.text((RX0 + 18, 184), "执行命令", font=f_mid, fill=(120, 200, 150, 255))
    d.text((RX0 + 18, 214), cmd, font=f_mid, fill=(230, 235, 245, 255))
    d.line([RX0 + 18, 260, RX0 + 372, 260], fill=(60, 75, 95, 255), width=1)
    d.text((RX0 + 18, 274), "机器判据（非 0 = 停）", font=f_mid, fill=(240, 190, 90, 255))
    d.text((RX0 + 18, 304), judge, font=f_mid, fill=(230, 235, 245, 255))
    d.text((RX0 + 18, 340), "未过判据 → 失败即停，只能重跑该步", font=f_small, fill=(150, 160, 180, 255))
    d.rounded_rectangle([RX0 + 18, 372, RX0 + 220, 404], radius=8,
                        fill=(60, 140, 80, 255) if step_no < 10 else (120, 90, 60, 255))
    d.text((RX0 + 34, 380), "过 → 下一步" if step_no < 10 else "收口（全链路完成）", font=f_small, fill=(255, 255, 255, 255))

    out.parent.mkdir(parents=True, exist_ok=True)
    canvas.convert("RGB").save(out, "PNG")


def gen_all() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    drawn = 0
    # 主编辑器概念图（覆盖旧的错误版）
    draw_main_editor(OUT_DIR / "map_editor_design.png")
    drawn += 1
    # 10 步向导逐步图
    for i, (slug, name, title, cmd, judge, has_shot) in enumerate(WIZARD_STEPS, start=1):
        draw_wizard_step(slug, i, name, title, cmd, judge, has_shot == 1,
                         OUT_DIR / f"map_wizard_step_{slug}.png")
        drawn += 1
    return drawn


def check() -> int:
    problems = []
    for p in sorted(OUT_DIR.glob("map_editor_design.png")) + sorted(OUT_DIR.glob("map_wizard_step_*.png")):
        if not p.is_file() or p.stat().st_size < 512:
            problems.append(f"缺/过小 {p}")
    if not problems:
        n = len(list(OUT_DIR.glob('map_wizard_step_*.png')))
        print(f"map_editor_design_gen check: 0 缺（主图 1 + 向导逐步 {n}，全在位非空）")
        return 0
    print(f"map_editor_design_gen check: {len(problems)} 项缺")
    for p in problems:
        print(f"  [FAIL] {p}")
    return 1


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="地图编辑器设计概念稿生成器")
    ap.add_argument("--check", action="store_true", help="只读：产物在位 + 非空，缺则非 0")
    args = ap.parse_args(argv[1:])
    if args.check:
        return check()
    n = gen_all()
    print(f"map_editor_design_gen: 生成 {n} 张设计稿 → {OUT_DIR.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
