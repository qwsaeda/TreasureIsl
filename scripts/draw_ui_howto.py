#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""draw_ui_howto.py — 生成「怎么在平台改 UI 元素」使用说明图（S86/S87 操作流程）。

产出 `evidence/ui_howto.png`（证据区，文档相对路径引用）。

用法：
  python3 scripts/draw_ui_howto.py            # 生成 PNG（幂等）
  python3 scripts/draw_ui_howto.py --check    # 只读体检
  python3 scripts/draw_ui_howto.py --json     # 结构化输出
"""
from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "evidence/ui_howto.png"
FONT_PATH = ROOT / "assets/fonts/lxgw-wenkai-reg.ttf"

BG = (13, 15, 23)
PANEL = (22, 26, 38)
BORDER = (44, 52, 74)
ACCENT = (214, 168, 78)
TEXT = (232, 228, 214)
MUTED = (150, 146, 132)
GREEN = (63, 185, 80)
BLUE = (108, 158, 232)
RED = (214, 96, 84)


def font(size: int):
    if not FONT_PATH.is_file():
        return None
    return ImageFont.truetype(str(FONT_PATH), size)


def wrap(d: ImageDraw.ImageDraw, text: str, f, max_w: int) -> list[str]:
    """按像素宽度折行（中文逐字）。"""
    lines, cur = [], ""
    for ch in text:
        trial = cur + ch
        if d.textlength(trial, font=f) > max_w and cur:
            lines.append(cur)
            cur = ch
        else:
            cur = trial
    if cur:
        lines.append(cur)
    return lines


def step(d: ImageDraw.ImageDraw, xy, num, title, body, color=ACCENT):
    """一步操作：序号圆 + 标题 + 正文。"""
    x0, y0, x1, y1 = xy
    d.rounded_rectangle(xy, radius=8, fill=PANEL, outline=color, width=2)
    d.ellipse((x0 + 14, y0 + 14, x0 + 52, y0 + 52), fill=color)
    d.text((x0 + 26, y0 + 21), str(num), font=font(24), fill=BG)
    d.text((x0 + 66, y0 + 18), title, font=font(21), fill=color)
    f = font(16)
    ty = y0 + 64
    for line in wrap(d, body, f, x1 - x0 - 32):
        d.text((x0 + 18, ty), line, font=f, fill=TEXT)
        ty += 24
    return ty


def build() -> Image.Image:
    W, H = 1500, 1180
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    d.text((34, 24), "使用说明 · 在平台改 UI 元素（零代码）", font=font(34), fill=ACCENT)
    d.text((36, 70), "Dev Console → 地图页 / UI 组件页 → 拖拽 → 校验 → 真渲染预览 → 导出即生效",
           font=font(18), fill=MUTED)

    # ── 启动 ────────────────────────────────────────────────
    d.rounded_rectangle((34, 112, 1466, 196), radius=8, fill=PANEL, outline=BLUE, width=2)
    d.text((50, 126), "0. 启动平台", font=font(21), fill=BLUE)
    d.text((50, 158),
           "cargo run -p ti-console     →  浏览器打开 http://127.0.0.1:8080     （设置页粘贴 token，用于写操作）",
           font=font(16), fill=TEXT)

    # ── 地图页 ──────────────────────────────────────────────
    d.text((34, 224), "路线 A · 地图制作（31 格环 / 领地 / 岛屿布局）", font=font(23), fill=GREEN)
    step(d, (34, 262, 740, 452), 1, "打开「地图」页 → 「新建 / 编辑地图」",
         "左侧 Tile Palette 选地块，中间画布拖格子。横屏 1920×1080 坐标系，格位可自由摆放——"
         "但必须落在 world_canvas 内，且 I1~I9 不变量全过（31 格构成 / BOSS 位 / 8pt 栅格 / 资产齐）。",
         GREEN)
    step(d, (760, 262, 1466, 452), 2, "拖拽即校验，越界即拒",
         "拖到越界位置时直接拒绝落位并红字提示（格 N pos=x,y 超出 world_canvas）；"
         "其它不变量不合时校验条列出问题，此时不能导入。", GREEN)

    # ── UI 组件页 ───────────────────────────────────────────
    d.text((34, 482), "路线 B · UI 组件（7 场景：index / select_role / select_map / message / saveload / main_map / vs）",
            font=font(23), fill=ACCENT)
    step(d, (34, 520, 740, 760), 3, "打开「UI 组件 ↗」→ 选场景 → 拖元素",
         "画布上每个框 = 一个组件（背景 / 木牌 / 列表 / 按钮 / 文本 / 面板）。拖拽位移会自动"
         "反算成百分比写回 layout（offset_pct / size_pct），不写绝对像素。", ACCENT)
    step(d, (760, 520, 1466, 760), 4, "属性面板：role / action / visible_when",
         "role 从资产角色表下拉选（role→路径单源）；action 从 UiAction 闭集选（决定点击行为）；"
         "visible_when 决定运行时可见性（如「热座交接中才显示盖板」）。", ACCENT)

    # ── 校验 + 预览 + 导出 ──────────────────────────────────
    d.text((34, 790), "路线 A/B 共通 · 校验 → 预览 → 导出", font=font(23), fill=BLUE)
    step(d, (34, 828, 740, 1030), 5, "校验（FAIL 即禁摆）",
         "每次拖拽/改属性都先跑校验器：非法 type、出现裸坐标 x/y/w/h、未登记 role、非法 action、"
         "i18n 裸 key 一律 FAIL 并点名是哪个组件——不合规拖不进去，也导不出去。", RED)
    step(d, (760, 828, 1466, 1030), 6, "预览 = 游戏真渲染（不是示意图）",
         "「预览真渲染」按钮 → 后台跑 Bevy --shot → 返回 PNG。编辑器里看到的就是游戏引擎"
         "实际渲染的画面（当前画面 + 未落盘改动注入），不是 SVG 近似。", BLUE)

    # ── 底部条 ──────────────────────────────────────────────
    d.rounded_rectangle((34, 1056, 1466, 1150), radius=8, fill=PANEL, outline=GREEN, width=2)
    d.text((50, 1070), "7. 导出即生效", font=font(21), fill=GREEN)
    d.text((50, 1102),
           "「应用」写回 schema（indent=2 保序，零 churn）→ 游戏下次加载即用新数据。"
           "画面一致性判据：导出后同 scene 出图与预览 PNG 逐像素 diff = 0。",
           font=font(16), fill=TEXT)

    return img


def main() -> int:
    ap = argparse.ArgumentParser(description="UI 元素编辑使用说明图生成器")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()

    if args.check:
        if not OUT.is_file() or OUT.stat().st_size == 0:
            print("draw_ui_howto: 图不存在或为空", file=sys.stderr)
            return 1
        print(f"draw_ui_howto: OK（{OUT} {OUT.stat().st_size} B）")
        return 0
    if font(20) is None:
        print(f"draw_ui_howto: SKIP（缺字体 {FONT_PATH}）", file=sys.stderr)
        return 2

    OUT.parent.mkdir(parents=True, exist_ok=True)
    img = build()
    img.save(OUT)
    meta = {"out": str(OUT.relative_to(ROOT)), "size": list(img.size), "bytes": OUT.stat().st_size}
    if args.json:
        print(json.dumps(meta, ensure_ascii=False))
    else:
        print(f"draw_ui_howto: 已写 {meta['out']}（{img.size[0]}×{img.size[1]}，{meta['bytes']} B）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
