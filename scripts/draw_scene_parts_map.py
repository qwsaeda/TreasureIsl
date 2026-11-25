#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""draw_scene_parts_map.py — 每个场景一张「UI 元器件说明图」（电路板装配图范式）。

每张图 = 一个场景的 UI 元件清单：
  · 画布区：按 1920×1080 真实布局摆元件，每个元件画成封装块 + 位号(id) + 型号(type/role)
  · 引线标注：元件 → 引出线 → 位号/规格标签（电路板装配图范式）
  · 清单区：位号 / 型号 / 角色 / 位置 / 可替换项 / 可点性
  · 角标：元件总数 / 可拖动数 / 可交互数

数据源：assets/data/ui_schema/components_v1.json（5 纯 UI 场景）
        + assets/data/ui_schema/world_components.json（main_map / vs 世界层）
        + assets/data/ui_schema/asset_roles.json（role → 资产路径）

用法：
  python3 scripts/draw_scene_parts_map.py                     # 全部场景
  python3 scripts/draw_scene_parts_map.py --scene index       # 单场景
  python3 scripts/draw_scene_parts_map.py --check             # 只读体检
  python3 scripts/draw_scene_parts_map.py --json              # 结构化输出

产出：evidence/scene_parts/<scene>.png（每个场景一张）
退出码：0 = OK；1 = FAIL；2 = SKIP（缺字体/缺数据）。
"""
from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "evidence/scene_parts"
FONT_PATH = ROOT / "assets/fonts/lxgw-wenkai-reg.ttf"
UI_SCHEMA = ROOT / "assets/data/ui_schema/components_v1.json"
WORLD_SCHEMA = ROOT / "assets/data/ui_schema/world_components.json"
ROLES = ROOT / "assets/data/ui_schema/asset_roles.json"

SCENE_W, SCENE_H = 1920, 1080
BG = (13, 15, 23)
PANEL = (22, 26, 38)
BORDER = (44, 52, 74)
ACCENT = (214, 168, 78)
TEXT = (232, 228, 214)
MUTED = (150, 146, 132)
GREEN = (63, 185, 80)
BLUE = (108, 158, 232)
RED = (214, 96, 84)
TYPE_COLOR = {"image": BLUE, "text": GREEN, "rect": ACCENT, "panel": ACCENT}


def font(size: int):
    if not FONT_PATH.is_file():
        return None
    return ImageFont.truetype(str(FONT_PATH), size)


def load_scenes() -> dict:
    out = {}
    for path, kind in ((UI_SCHEMA, "ui"), (WORLD_SCHEMA, "world")):
        if not path.is_file():
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        for sid, scene in data.get("scenes", {}).items():
            out[sid] = {"kind": kind, "scene": scene}
    return out


def element_rect(elem: dict, resolved: dict | None) -> tuple[int, int, int, int] | None:
    """取元件在世界坐标下的矩形（world_rect 优先；geometry 用已解析值）。"""
    wr = elem.get("world_rect")
    if isinstance(wr, dict) and all(k in wr for k in ("x", "y", "w", "h")):
        return int(wr["x"]), int(wr["y"]), int(wr["w"]), int(wr["h"])
    if isinstance(resolved, dict) and all(k in resolved for k in ("x", "y", "w", "h")):
        return int(resolved["x"]), int(resolved["y"]), int(resolved["w"]), int(resolved["h"])
    return None


def build_scene(sid: str, info: dict, roles: dict) -> Image.Image:
    scene = info["scene"]
    elements = scene.get("elements", [])
    W, H = 1960, 1180
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)

    # 画布区（缩放到 1000×562）
    CX0, CY0, CW, CH = 216, 108, 700, 394
    scale = CW / SCENE_W
    d.rounded_rectangle((CX0, CY0, CX0 + CW, CY0 + CH), radius=8, fill=(16, 19, 29), outline=BORDER, width=2)
    # 网格（8pt 语义缩放）
    step = 100 * scale
    gx = CX0
    while gx <= CX0 + CW:
        d.line([(gx, CY0), (gx, CY0 + CH)], fill=(28, 33, 48), width=1)
        gx += step
    gy = CY0
    while gy <= CY0 + CH:
        d.line([(CX0, gy), (CX0 + CW, gy)], fill=(28, 33, 48), width=1)
        gy += step

    f_title = font(30)
    f_sub = font(16)
    f_small = font(14)
    f_tiny = font(12)
    d.text((36, 26), f"{sid} · UI 元器件说明图", font=f_title, fill=ACCENT)
    kind_txt = "纯 UI 场景（bevy_ui 层）" if info["kind"] == "ui" else "世界层场景（CPU 1080 层）"
    d.text((38, 66), f"{kind_txt}　1920×1080 真实布局　·　每块 = 一个可拖动/可替换元件　·　"
                    f"{len(elements)} 元件", font=f_sub, fill=MUTED)

    # 元件 → 画布矩形
    placed = []
    for idx, e in enumerate(elements):
        r = element_rect(e, None)
        if r is None:
            continue
        x, y, w, h = r
        px = CX0 + x * scale
        py = CY0 + y * scale
        pw = max(w * scale, 3)
        ph = max(h * scale, 3)
        col = TYPE_COLOR.get(e.get("type", ""), MUTED)
        placed.append((idx, e, r, (px, py, pw, ph), col))

    # 引线标注（左右分列到画布外侧的引线区；只画位号，规格在清单区）
    label_h = 21
    left_n = (len(placed) + 1) // 2
    right_n = len(placed) - left_n
    for n, (idx, e, r, (px, py, pw, ph), col) in enumerate(placed):
        label = str(e.get("id", "?"))
        if n % 2 == 0:
            ax, ay = px, py + ph / 2
            tx = CX0 - 10
            ty = CY0 + 96 + (n // 2) * label_h
            d.line([(ax, ay), (tx - 20, ay), (tx - 20, ty), (tx, ty)], fill=col, width=1)
            d.ellipse((tx - 3, ty - 3, tx + 3, ty + 3), fill=col)
            d.text((tx - 18 - d.textlength(label, font=f_small), ty - 8), label, font=f_small, fill=col)
        else:
            ax, ay = px + pw, py + ph / 2
            tx = CX0 + CW + 10
            ty = CY0 + 96 + ((n - 1) // 2) * label_h
            d.line([(ax, ay), (tx + 20, ay), (tx + 20, ty), (tx, ty)], fill=col, width=1)
            d.ellipse((tx - 3, ty - 3, tx + 3, ty + 3), fill=col)
            d.text((tx + 6, ty - 8), label, font=f_small, fill=col)

    # 元件块（画在引线之上）
    for idx, e, r, (px, py, pw, ph), col in placed:
        d.rectangle((px, py, px + pw, py + ph), outline=col, width=2, fill=None)
        if pw > 60 and ph > 18:
            d.text((px + 4, py + 2), e.get("id", "")[:14], font=f_tiny, fill=col)

    # 清单区
    LX0 = 1112
    d.rounded_rectangle((LX0, 108, W - 36, 620), radius=8, fill=PANEL, outline=BORDER, width=2)
    d.text((LX0 + 16, 122), "元件清单（位号 / 型号 / 位置 / 可替换 / 可点）", font=font(17), fill=ACCENT)
    cols = [(LX0 + 16, "位号"), (LX0 + 190, "型号"), (LX0 + 330, "位置"), (LX0 + 470, "可替换"), (LX0 + 620, "可点")]
    for cx, name in cols:
        d.text((cx, 152), name, font=f_small, fill=MUTED)
    d.line([(LX0 + 12, 176), (W - 48, 176)], fill=BORDER, width=1)
    ty = 184
    for idx, e, r, _b, col in placed:
        if ty > CY0 + CH - 24:
            d.text((LX0 + 16, ty), f"… 其余 {len(placed) - (ty - 184) // 22} 元件见 JSON", font=f_tiny, fill=MUTED)
            break
        role = e.get("role") or "-"
        act = e.get("pick", {}).get("action") if isinstance(e.get("pick"), dict) else None
        d.text((cols[0][0], ty), e.get("id", "?")[:18], font=f_small, fill=col)
        d.text((cols[1][0], ty), e.get("type", "?")[:14], font=f_small, fill=TEXT)
        d.text((cols[2][0], ty), f"{r[0]},{r[1]} {r[2]}×{r[3]}", font=f_tiny, fill=MUTED)
        d.text((cols[3][0], ty), role[:16], font=f_tiny, fill=MUTED)
        d.text((cols[4][0], ty), act or "-", font=f_tiny, fill=GREEN if act else MUTED)
        ty += 20

    # 底部说明
    d.rounded_rectangle((36, 646, W - 36, 1000), radius=8, fill=PANEL, outline=BORDER, width=2)
    d.text((52, 660), "怎么用这张图", font=font(20), fill=ACCENT)
    lines = [
        ("拖动", "在平台「UI 组件 ↗」页的同名画布上直接拖元件块 —— 位号即组件 id，位移自动反算为百分比写回 layout。", GREEN),
        ("替换", "选中元件 → 右栏属性面板换 role（资产角色表下拉）/ action（UiAction 闭集）/ visible_when（运行时可见性）。", BLUE),
        ("禁摆", "黄框 = geometry 引用元素（坐标受源数据约束，拖拽即 FAIL 禁摆，可在属性面板「解除约束 → world_rect」）。", RED),
        ("即时", "每次落点自动跑校验器（非法 type / 裸坐标 / 未登记 role / 非法 action / i18n 裸 key ⇒ FAIL 点名 + 自动回滚）。", MUTED),
        ("可见即可得", "「预览真渲染」跑 Bevy --shot 出图 —— 看到的就是游戏引擎实际渲染，不是示意图。", ACCENT),
        ("导出", "「应用」写回 schema（indent=2 保序）→ 游戏下次加载即生效；一致性判据 = 导出后同 scene 出图与预览 PNG 逐像素 diff 0。", ACCENT),
    ]
    ly = 700
    for tag, desc, col in lines:
        d.rounded_rectangle((52, ly, 52 + 74, ly + 26), radius=5, fill=col)
        d.text((62, ly + 4), tag, font=font(15), fill=BG)
        d.text((140, ly + 4), desc, font=font(15), fill=TEXT)
        ly += 42
    return img


def main() -> int:
    ap = argparse.ArgumentParser(description="场景 UI 元器件说明图生成器")
    ap.add_argument("--scene", help="只生成指定场景")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    scenes = load_scenes()
    if not scenes:
        print("draw_scene_parts_map: SKIP（无 schema 数据）", file=sys.stderr)
        return 2
    roles = json.loads(ROLES.read_text(encoding="utf-8")).get("roles", {}) if ROLES.is_file() else {}

    if args.check:
        want = [args.scene] if args.scene else sorted(scenes)
        missing = [s for s in want if not (OUT_DIR / f"{s}.png").is_file()]
        if missing:
            print(f"draw_scene_parts_map: 图缺失 {missing}", file=sys.stderr)
            return 1
        print(f"draw_scene_parts_map: OK（{len(want)} 张图在位）")
        return 0

    if font(20) is None:
        print(f"draw_scene_parts_map: SKIP（缺字体 {FONT_PATH}）", file=sys.stderr)
        return 2

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    targets = [args.scene] if args.scene else sorted(scenes)
    out = []
    for sid in targets:
        if sid not in scenes:
            print(f"draw_scene_parts_map: 未知场景 {sid}", file=sys.stderr)
            return 1
        img = build_scene(sid, scenes[sid], roles)
        path = OUT_DIR / f"{sid}.png"
        img.save(path)
        out.append({"scene": sid, "out": str(path.relative_to(ROOT)),
                    "size": list(img.size), "bytes": path.stat().st_size,
                    "elements": len(scenes[sid]["scene"].get("elements", []))})
    if args.json:
        print(json.dumps({"out": out}, ensure_ascii=False))
    else:
        for o in out:
            print(f"draw_scene_parts_map: 已写 {o['out']}（{o['elements']} 元件，{o['bytes']} B）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
