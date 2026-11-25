#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""draw_ui_system_map.py — 生成 UI 系统电路板示意图（S85/S86/S87 架构总图）。

产出 `evidence/ui_system_map.png`（证据区，文档相对路径引用）。
画的是**真实接线**（每个框 = 实际存在的文件/函数），非示意图臆造。

用法：
  python3 scripts/draw_ui_system_map.py            # 生成 PNG（幂等：同输入同产物）
  python3 scripts/draw_ui_system_map.py --check    # 只读体检（文件在位 + 非空，不重画）
  python3 scripts/draw_ui_system_map.py --json     # 结构化输出元数据

退出码：0 = OK；1 = FAIL；2 = SKIP（缺字体）。
"""
from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "evidence/ui_system_map.png"
FONT_PATH = ROOT / "assets/fonts/lxgw-wenkai-reg.ttf"

# 配色（项目暗色主题：index.html --panel/--accent 同族）
BG = (13, 15, 23)
PANEL = (22, 26, 38)
BORDER = (44, 52, 74)
ACCENT = (214, 168, 78)      # 黄铜金
TEXT = (232, 228, 214)
MUTED = (150, 146, 132)
GREEN = (63, 185, 80)
BLUE = (108, 158, 232)
RED = (214, 96, 84)


def font(size: int):
    if not FONT_PATH.is_file():
        return None
    return ImageFont.truetype(str(FONT_PATH), size)


def box(d: ImageDraw.ImageDraw, xy, title, lines, accent=ACCENT, tsize=19, lsize=15):
    """画一个电路板模块框：标题条 + 说明行。"""
    x0, y0, x1, y1 = xy
    d.rounded_rectangle(xy, radius=8, fill=PANEL, outline=accent, width=2)
    d.rounded_rectangle((x0, y0, x1, y0 + 30), radius=8, fill=accent)
    d.rectangle((x0, y0 + 22, x1, y0 + 30), fill=accent)
    d.text(((x0 + 12, y0 + 6)), title, font=font(tsize), fill=BG)
    ty = y0 + 38
    for line in lines:
        d.text(((x0 + 12, ty)), line, font=font(lsize), fill=TEXT if line.startswith(" ") else MUTED)
        ty += lsize + 7


def arrow(d: ImageDraw.ImageDraw, p0, p1, color=GREEN, label=None, lf=None):
    """带箭头的连线（电路板走线风格：正交折线 + 箭头）。"""
    x0, y0 = p0
    x1, y1 = p1
    mid = (x0 + x1) // 2
    pts = [(x0, y0), (mid, y0), (mid, y1), (x1, y1)]
    d.line(pts, fill=color, width=3)
    # 箭头
    import math
    ax, ay = x1, y1
    bx, by = mid, y1
    ang = math.atan2(ay - by, ax - bx)
    for s in (+0.45, -0.45):
        d.line([(ax, ay), (ax - 14 * math.cos(ang - s), ay - 14 * math.sin(ang - s))], fill=color, width=3)
    if label:
        d.text(((mid + 8, (y0 + y1) // 2 - 20)), label, font=lf or font(13), fill=color)


def build() -> Image.Image:
    W, H = 1880, 1180
    img = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(img)
    f_title = font(34)
    f_sub = font(17)
    d.text((36, 26), "UI 系统电路板 · 数据 → 加载 → 双后端渲染 → 命中 → 平台编辑器", font=f_title, fill=ACCENT)
    d.text((38, 72), "S85 声明式组件 schema + S86 平台可视化编辑器 + S87 世界层（vs/main_map）｜每个框 = 真实文件/函数", font=f_sub, fill=MUTED)

    # ── 列 1：数据层 ─────────────────────────────────────────────
    box(d, (40, 120, 400, 300), "① 数据层（可编辑）",
        ["assets/data/ui_schema/",
         "  components_v1.json   5 纯 UI 场景",
         "  world_components.json main_map/vs",
         "  asset_roles.json     role→资产路径",
         "assets/data/strings.json  i18n 文案"],
        accent=BLUE)
    box(d, (40, 330, 400, 470), "② 校验门禁（只读）",
        ["scripts/ui_schema_check.py    5 场景",
         "scripts/world_schema_check.py 世界层",
         "  → 非法 type / 裸坐标 /",
         "    未登记 role / 非法 action",
         "    ⇒ FAIL 点名（FAIL 即禁摆）"],
        accent=RED)

    # ── 列 2：加载层 ─────────────────────────────────────────────
    box(d, (500, 120, 900, 300), "③ 加载层 ti-core::data::ui_schema",
        ["components() / world_components()",
         "  asset_roles()",
         "  include_str! 内嵌 + OnceLock",
         "  install_injected() 首访前注入",
         "  （编辑器预览走这条）"],
        accent=ACCENT)

    # ── 列 3：渲染层（双后端）─────────────────────────────────────
    box(d, (1000, 110, 1400, 250), "④A 纯 UI 后端（bevy_ui）",
        ["ti-engine::ui_component",
         "  build_component_chrome()",
         "  taffy 自动布局（% / flex）",
         "  resize 真重排｜零绝对像素"],
        accent=GREEN)
    box(d, (1000, 280, 1400, 420), "④B 世界层后端（CPU 1080）",
        ["ti-app::scenes::world_schema",
         "  render_world_components()",
         "  → Canvas → DrawList",
         "  与世界层同坐标系，不遮挡"],
        accent=GREEN)
    box(d, (1000, 450, 1400, 612), "⑤ 命中层",
        ["bevy_picking（UI 节点）",
         "hit_1080_pick（世界层几何）",
         "  → PickEvent → poll_hit",
         "  → Scene::on_hit",
         "  按 UiAction 分派（非裸 key）"],
        accent=BLUE)

    # ── 列 4：平台层 ─────────────────────────────────────────────
    box(d, (1500, 110, 1840, 290), "⑥ Dev Console 编辑器",
        ["web/ui_editor.html  7 场景 tab",
         "web/index.html      地图编辑器",
         "  拖拽 → 反算百分比 → 写回",
         "  role / action / visible_when",
         "  下拉可选（登记表单一源）"],
        accent=ACCENT)
    box(d, (1500, 320, 1840, 480), "⑦ 预览（可见即可得）",
        ["POST /api/preview",
         "  → preview_shot.py",
         "  → treasure-bevy-native --shot",
         "  TI_PREVIEW_DIR 注入（零写盘）",
         "  PNG 回传 + diff 0 判据"],
        accent=GREEN)
    box(d, (1500, 510, 1840, 662), "⑧ 写回（零 Rust）",
        ["POST /api/ui/components/apply",
         "  → repo/ui_components.rs",
         "  → 规范化写 schema（indent=2）",
         "  → 写点探针 + 校验",
         "导出即生效：导入后画面一致"],
        accent=ACCENT)

    # ── 连线 ─────────────────────────────────────────────────────
    arrow(d, (400, 200), (500, 200), GREEN, "读", font(13))
    arrow(d, (400, 400), (500, 400), RED, None, font(13))
    d.text((410, 376), "校验", font=font(13), fill=RED)
    arrow(d, (900, 180), (1000, 180), GREEN, "解析", font(13))
    arrow(d, (900, 230), (1000, 340), GREEN, "静态元素", font(13))
    arrow(d, (1200, 420), (1200, 450), GREEN, "", font(13))
    arrow(d, (1400, 180), (1500, 180), GREEN, "读", font(13))
    arrow(d, (1670, 290), (1670, 320), GREEN, "预览", font(13))
    arrow(d, (1670, 480), (1670, 510), GREEN, "导出", font(13))
    # 写回 → 数据层（反馈回路）：走底部通道 y=670，避开⑤命中层框，避免压字
    import math
    fb_y = 668
    d.line([(1670, 662), (1670, fb_y)], fill=ACCENT, width=3)
    d.line([(1670, fb_y), (220, fb_y)], fill=ACCENT, width=3)
    d.line([(220, fb_y), (220, 300)], fill=ACCENT, width=3)
    ax, ay = 220, 300
    for s in (+0.5, -0.5):
        d.line([(ax, ay), (ax - 12 * math.cos(-math.pi / 2 + s), ay - 12 * math.sin(-math.pi / 2 + s))], fill=ACCENT, width=3)
    d.text((620, fb_y - 26), "写回 schema（数据驱动：改位置/元素 = 改数据，不改代码）", font=font(15), fill=ACCENT)

    # ── 底部说明 ─────────────────────────────────────────────────
    box(d, (40, 700, 1840, 900), "⑨ 两条数据通路（互斥不叠加）",
        ["纯 UI 5 场景（index / select_role / select_map / message / saveload）",
         "  components_v1.json  →  build_component_chrome  →  bevy_ui 节点（taffy 自动布局）",
         "世界层 2 场景（main_map / vs）",
         "  world_components.json  →  render_world_components  →  CPU Canvas 1080（与世界层同层，不遮挡）",
         "",
         "动画 / 粒子 / 时序 / 运行时数值（HP / 骰点 / 兵力 / toast）始终由 Rust 过程式绘制，画在静态层之后"],
        accent=MUTED, tsize=18, lsize=16)

    box(d, (40, 930, 1840, 1140), "⑩ 边界纪律",
        ["禁绝对像素：组件顶层出现 x/y/w/h/rect ⇒ 校验 FAIL（布局意图 + 百分比锚点）",
         "禁业务语义泄漏：渲染器只认 type/role/binding/action/string，不认 Character / MapSlot / GameApp",
         "两层遮挡契约：世界层元素绝不挂 bevy_ui 节点（S29.3e 坑：UI 相机 order=1 会遮住 1080 动态内容）",
         "parity 口径不变：bevy-cpu-parity 只对拍游戏世界，UI 层 Node=0",
         "主题 / 地图包注入：table_source + ui_schema::install_injected（OnceLock 首访前定型）"],
        accent=RED, tsize=18, lsize=16)

    return img


def main() -> int:
    ap = argparse.ArgumentParser(description="UI 系统电路板示意图生成器")
    ap.add_argument("--check", action="store_true", help="只读体检（不重画）")
    ap.add_argument("--json", action="store_true", help="结构化输出")
    args = ap.parse_args()

    if args.check:
        if not OUT.is_file() or OUT.stat().st_size == 0:
            print("draw_ui_system_map: 图不存在或为空", file=sys.stderr)
            return 1
        print(f"draw_ui_system_map: OK（{OUT} {OUT.stat().st_size} B）")
        return 0

    if font(20) is None:
        print(f"draw_ui_system_map: SKIP（缺字体 {FONT_PATH}）", file=sys.stderr)
        return 2

    OUT.parent.mkdir(parents=True, exist_ok=True)
    img = build()
    img.save(OUT)
    meta = {"out": str(OUT.relative_to(ROOT)), "size": list(img.size),
            "bytes": OUT.stat().st_size, "font": str(FONT_PATH.relative_to(ROOT))}
    if args.json:
        print(json.dumps(meta, ensure_ascii=False))
    else:
        print(f"draw_ui_system_map: 已写 {meta['out']}（{img.size[0]}×{img.size[1]}，{meta['bytes']} B）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
