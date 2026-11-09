#!/usr/bin/env python3
"""map_editor_step_render.py — 地图编辑器「步进方案」逐步设计渲染图 + 多图合成总览。

设计目标（用户 2026-11-09：一步一步设计，每步都要设计渲染图，可多次生成后合成）：
  每一步 = 一张 1280×800 渲染图（完整编辑器界面 + 该步高亮 + 该步结果面板），
  最后一张 = 总览图（10 步缩略拼图 + 流程说明），供 `docs/map_editor_guide.md` 图文引用。

图编号（与 map_wizard.rs 10 步 1:1，与 WZ_STEPS 同步）：
  01_add → 02_fill → 03_syncmap → 04_verify → 05_pack → 06_install
  → 07_check → 08_sync → 09_shot → 10_zero_rust
  总览：map_editor_steps_overview.png（10 步缩略 5×2 拼图 + 流转标注）

工具契约（14 号 §1.2）：
  - 无参 = 生成 11 张（10 步 + 总览）→ evidence/ui_concept/（幂等，重生成覆盖同路径）
  - `--check` = 只读（产物在位 + 非空，缺则非 0）
  - `--only <slug>` = 只重出指定步（合成时缺图 = 非 0，须先补齐）
"""
import argparse
import math
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "evidence" / "ui_concept"

# ---------------------------------------------------------------- 配色
BG      = (22, 26, 36)
PANEL   = (34, 40, 54)
PANEL2  = (44, 52, 68)
BORDER  = (72, 82, 102)
INK     = (228, 232, 244)
MUTED   = (150, 158, 178)
ACCENT  = (126, 200, 227)
GREEN   = (72, 196, 110)
GOLD    = (201, 161, 90)
RED     = (216, 84, 84)
GRAY    = (88, 94, 110)
PARCH   = (56, 48, 36)

# 10 类岛色带（theme.json isle_* 近似 888）
ISLE_COLORS = {
    "harbour": (200, 122, 110), "lighthouse": (224, 176, 112), "wreck": (232, 192, 64),
    "pirate_lair": (58, 58, 92), "shipyard": (62, 138, 120), "trading_post": (140, 122, 60),
    "skull_cliff": (232, 228, 208), "volcano": (216, 92, 48), "lightship": (122, 140, 160),
    "boss_fort": (56, 40, 74),
}
ISLE_ZH = {"harbour": "港口城", "lighthouse": "灯塔礁", "wreck": "沉船滩", "pirate_lair": "海盗老巢",
           "shipyard": "帆船坞", "trading_post": "军需商岛", "skull_cliff": "骷髅崖",
           "volcano": "火山岛", "lightship": "灯船站", "boss_fort": "无畏号要塞"}
ISLE_RES = {"harbour": "fishing", "lighthouse": "reef", "wreck": "gold", "pirate_lair": "loot",
            "shipyard": "ship", "trading_post": "trade", "skull_cliff": "loot", "volcano": "gold",
            "lightship": "dice", "boss_fort": "boss"}

KCOL = {"start": (63, 185, 80), "boss": (208, 84, 84), "event": (122, 106, 154),
        "card": (122, 106, 154), "treasure": (122, 106, 154)}
KLET = {"start": "S", "boss": "B", "event": "E", "card": "C", "treasure": "T"}

KIND_SEQ = ['start','territory','territory','event','territory','territory','card','territory',
            'territory','treasure','territory','territory','event','territory','territory','boss',
            'territory','territory','event','territory','territory','treasure','territory',
            'territory','card','territory','territory','territory','territory','event','card']
ISLE_SEQ = [None,'harbour','harbour',None,'lighthouse','lighthouse',None,'wreck','wreck',None,
            'pirate_lair','pirate_lair',None,'shipyard','shipyard',None,'trading_post','trading_post',
            None,'skull_cliff','skull_cliff',None,'volcano','volcano',None,'lightship','lightship',
            'boss_fort','boss_fort',None,None]


def font(size, bold=False):
    # 优先 CJK 字体（wqy-zenhei.ttc 系统真实存在；DejaVu 兜底无 CJK）
    for p in (["/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
               "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
               "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"] if bold else
              ["/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
               "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
               "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]):
        try:
            f = ImageFont.truetype(p, size)
            return f
        except Exception:
            continue
    return ImageFont.load_default()


# ---------------------------------------------------------------- 步骤定义
STEPS = [
    # slug, 标签, 结果面板, 画布演示
    ("01_add", "① 新建包",
     "✓ 内容就绪（31 格 + 2 事件 + 2 商人 + vs_theme）\n"
     "  包 m04 已安装 → add 幂等跳过（4 文件在位即过）",
     lambda i: None),
    ("02_fill", "② 填内容 + 导入落盘",
     "✓ 已落盘 assets/maps/m04/ 四文件\n"
     "  manifest / map / layout / strings  + sync + verify",
     lambda i: 8),
    ("03_syncmap", "③ 同步 manifest[map] ← map.json",
     "✓ 0 差（幂等；两文件一致）\n"
     "  判据：map_pack.py syncmap 输出「0 差」",
     lambda i: None),
    ("04_verify", "④ 权威校验",
     "✓ 0 违规（schema/兼容/I1~I9/S34.5 全过）\n"
     "  判据：map_pack.py verify m04 → 0 违规",
     lambda i: None),
    ("05_pack", "⑤ 出可分发件",
     "✓ dist/maps/m04.map-pack/ + .zip（sha256 清单 + 池资产副本）\n"
     "  判据：pack 成功 + dist 产物存在",
     lambda i: None),
    ("06_install", "⑥ 安装 + 回注",
     "✓ 装到 assets/maps/m04/ + 接线 board.json maps.m04 + index.json\n"
     "  判据：install 成功",
     lambda i: None),
    ("07_check", "⑦ 漂移对账",
     "✓ 0 漂移（已装态 vs board.json 全量对账）\n"
     "  判据：check 输出「0 漂移」",
     lambda i: None),
    ("08_sync", "⑧ 回注构建期默认数据",
     "✓ 全量重算 board/scenes/manifest/strings（幂等）\n"
     "  判据：sync 成功",
     lambda i: None),
    ("09_shot", "⑨ 出图验收",
     "✓ 真渲染 main_map（TI_MAP=m04）+ 出图非黑\n"
     "  判据：exit 0 + 像素多样性 ≥ 4 种",
     lambda i: None),
    ("10_zero_rust", "⑩ 零 Rust 判据",
     "✓ git diff crates/ti-core…ti-formats 空（6 crate 0 diff）\n"
     "  判据：内容运营 = 只动数据不碰游戏代码",
     lambda i: None),
]
WIZARD_ORDER = ["add", "fill", "syncmap", "verify", "pack", "install", "check", "sync", "shot", "zero_rust"]


# ---------------------------------------------------------------- 编辑器整屏（每步共用底）
def draw_editor_screen(step_no, highlight_idx, status=""):
    """画完整编辑器界面（1280×800）：工具条 + 步进条 + 左岛型 + 中画布 + 右面板。

    step_no = 0~9（当前步）；highlight_idx = 画布上要高亮的格（如第 2 步铺岛高亮格 8）。"""
    W, H = 1280, 800
    c = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(c)

    # ── 标题 ──
    d.rectangle([0, 0, W, 40], fill=PANEL2)
    d.text((16, 10), "地图编辑器 · Dev Console「地图」页（极简分步 · 向导聚焦）", font=font(15, True), fill=INK)
    d.text((W - 260, 13), "包 m04 · 一次只亮当前步工作区", font=font(11), fill=MUTED)

    # ── 顶部细步进条（10 圆点，仅当前高亮）──
    by = 52
    d.rounded_rectangle([12, by, W - 12, by + 96], radius=8, fill=PANEL, outline=BORDER)
    dot_y = by + 26
    gap = (W - 90) / 10
    for i, name in enumerate(WIZARD_ORDER):
        cx2 = 40 + gap * i + gap / 2
        if i < step_no:
            col, mark, fg = GREEN, "✓", (255, 255, 255)
        elif i == step_no:
            col, mark, fg = GOLD, str(i + 1), (16, 20, 30)
        else:
            col, mark, fg = (58, 68, 84), str(i + 1), MUTED
        d.ellipse([cx2 - 12, dot_y - 12, cx2 + 12, dot_y + 12], fill=col,
                  outline=ACCENT if i == step_no else None,
                  width=2 if i == step_no else 1)
        d.text((cx2 - 8, dot_y - 7), mark, font=font(11, i == step_no), fill=fg)
        d.text((cx2 - 24, dot_y + 16), name, font=font(9), fill=INK if i == step_no else MUTED)
    d.text((40, by + 62), "第 %d/10 步 · %s" % (step_no + 1, STEPS[step_no][1].replace("① ", "").replace("② ", "").replace("③ ","").replace("④ ","").replace("⑤ ","").replace("⑥ ","").replace("⑦ ","").replace("⑧ ","").replace("⑨ ","").replace("⑩ ","")), font=font(13, True), fill=ACCENT)

    # ── 当前步工作区（按步显隐）──
    my = by + 108
    mh = H - my - 96
    step_name = WIZARD_ORDER[step_no]
    edit_steps = ("add", "import")
    if step_name in edit_steps:
        # 编辑区：左岛色板（窄）+ 中环画布 + 右属性/内容
        lw = 220
        rw = 270
        mx = 12 + lw + 10
        mw = W - 24 - lw - rw - 20
        d.rounded_rectangle([12, my, 12 + lw, my + mh], radius=8, fill=PANEL, outline=BORDER)
        d.text((24, my + 10), "岛型（10 · 铺格）", font=font(12, True), fill=ACCENT)
        isles = list(ISLE_COLORS.keys())
        for i, isle in enumerate(isles):
            col_i, row_i = i % 2, i // 2
            x = 24 + col_i * 104
            y = my + 38 + row_i * 58
            on = (isle == "wreck")
            d.rounded_rectangle([x, y, x + 96, y + 50], radius=6,
                                fill=(60, 70, 92) if on else PANEL2,
                                outline=ACCENT if on else BORDER, width=2 if on else 1)
            d.rectangle([x + 8, y + 8, x + 28, y + 28], fill=ISLE_COLORS[isle])
            d.text((x + 34, y + 6), isle[:8], font=font(9, True), fill=INK)
            d.text((x + 34, y + 22), ISLE_ZH[isle], font=font(8), fill=MUTED)
            d.text((x + 8, y + 34), "→" + ISLE_RES[isle], font=font(8), fill=MUTED)
        # 中画布（环 + 中央岛 + 选中格）
        d.rounded_rectangle([mx, my, mx + mw, my + mh], radius=8, fill=(16, 20, 28), outline=BORDER)
        d.text((mx + 14, my + 10), "环形赛道 · 点格铺岛", font=font(12, True), fill=INK)
        cx, cy = mx + mw / 2, my + mh / 2 + 8
        R = min(mw, mh) * 0.34
        d.ellipse([cx - R, cy - R, cx + R, cy + R], outline=(54, 64, 82), width=2)
        d.ellipse([cx - 78, cy - 78, cx + 78, cy + 78], fill=PARCH, outline=GOLD, width=2)
        d.text((cx - 24, cy - 10), "中央岛", font=font(12, True), fill=GOLD)
        for i in range(31):
            a = math.radians(-90 + i * 360 / 31)
            nx, ny = cx + R * math.cos(a), cy + R * math.sin(a)
            kind, isle = KIND_SEQ[i], ISLE_SEQ[i]
            w2, h2 = 34, 42
            x0, y0 = nx - w2 / 2, ny - h2 / 2
            col = KCOL.get(kind, GRAY) if kind != "territory" else (ISLE_COLORS.get(isle, GRAY) if isle else (110, 130, 160))
            sel = (i == highlight_idx)
            d.rounded_rectangle([x0, y0, x0 + w2, y0 + h2], radius=4,
                                fill=col, outline=(ACCENT if sel else (24, 28, 38)),
                                width=3 if sel else 1)
            d.text((x0 + 3, y0 + 2), str(i), font=font(9, True), fill=(255, 255, 255))
        # 右属性/内容
        rx = mx + mw + 10
        d.rounded_rectangle([rx, my, rx + rw, my + mh], radius=8, fill=PANEL, outline=BORDER)
        d.text((rx + 14, my + 10), "属性 · 格 %s" % (str(highlight_idx) if highlight_idx is not None else "—"), font=font(12, True), fill=ACCENT)
        d.text((rx + 14, my + 36), "isle = wreck（沉船滩）", font=font(10), fill=INK)
        d.text((rx + 14, my + 54), "resource=gold · bldg=1", font=font(9), fill=MUTED)
        d.text((rx + 14, my + 78), "内容配置", font=font(10, True), fill=ACCENT)
        d.text((rx + 14, my + 96), "events ✓2 · merchants ✓2", font=font(9), fill=GREEN)
        d.text((rx + 14, my + 112), "vs_theme ✓（bg+2 槽）", font=font(9), fill=GREEN)
    else:
        # 结果卡（syncmap~zero_rust：本步命令 + 结果回显 + 产出路径）
        d.rounded_rectangle([12, my, W - 12, my + mh], radius=8, fill=PANEL, outline=BORDER)
        cmd_map = {
            "syncmap": "map_pack.py syncmap m04", "verify": "map_pack.py verify m04",
            "pack": "map_pack.py pack m04", "install": "map_pack.py install m04",
            "check": "map_pack.py check", "sync": "map_pack.py sync",
            "shot": "treasure-bevy-native --shot --scene=main_map（TI_MAP=m04）",
            "zero_rust": "git diff --quiet -- crates/ti-core…ti-formats"}
        d.text((28, my + 14), "本步命令：" + cmd_map.get(step_name, ""), font=font(11, True), fill=ACCENT)
        res_y = my + 48
        d.rounded_rectangle([28, res_y, W - 28, res_y + 150], radius=6, fill=(38, 52, 42), outline=GREEN)
        d.text((40, res_y + 10), "第 %d 步 · %s" % (step_no + 1, STEPS[step_no][1]), font=font(12, True), fill=GREEN)
        for k, ln in enumerate(STEPS[step_no][2].split("\n")):
            d.text((40, res_y + 34 + k * 22), ln, font=font(11), fill=(190, 220, 200))

    # ── 操作条（执行 + 结果 + 下一步）──
    oy = my + mh + 12
    d.rounded_rectangle([12, oy, 150, oy + 38], radius=6, fill=ACCENT)
    d.text((30, oy + 10), "▶ 执行 " + STEPS[step_no][1], font=font(12, True), fill=(16, 20, 30))
    d.rounded_rectangle([160, oy, W - 130, oy + 38], radius=6, fill=(38, 52, 42), outline=GREEN)
    d.text((176, oy + 10), "✓ " + STEPS[step_no][2].split("\n")[0], font=font(11), fill=GREEN)
    d.rounded_rectangle([W - 118, oy, W - 12, oy + 38], radius=6, fill=PANEL2, outline=BORDER)
    d.text((W - 100, oy + 10), "下一步 →", font=font(12, True), fill=INK)

    # ── 底部状态条（极简单行）──
    sy = H - 34
    d.rectangle([0, sy, W, H], fill=PANEL2)
    d.line([0, sy, W, sy], fill=BORDER)
    d.text((16, sy + 10), "当前第 %d/10 步 · %s" % (step_no + 1, STEPS[step_no][1]), font=font(11, True), fill=ACCENT)
    d.text((W - 220, sy + 10), "校验 8/8 · 不可跳步 · 失败即停", font=font(10), fill=INK)
    return c


# ---------------------------------------------------------------- 总览拼图
def draw_overview():
    """10 步缩略 5×2 拼图 + 流程说明。"""
    W, H = 1400, 1000
    c = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(c)
    d.text((20, 16), "地图编辑器 · 10 步进方案总览（每步 = 一张设计渲染图）", font=font(20, True), fill=INK)
    d.text((20, 52), "操作链路：左选岛 → 中铺格 → 上步进条引导 → 失败即停 · 不可跳步（map_wizard.rs 状态机）",
           font=font(12), fill=MUTED)
    # 5×2 缩略
    tw, th = 256, 160
    gx, gy = 24, 84
    gap = 16
    for idx, (slug, label, _res, hl) in enumerate(STEPS):
        r, col_i = idx // 5, idx % 5
        x = gx + col_i * (tw + gap)
        y = gy + r * (th + gap + 30)
        # 缩略 = 该步渲染图缩小
        step_img = Image.open(OUT / ("map_editor_step_%s.png" % slug)).convert("RGB")
        step_img = step_img.resize((tw, int(step_img.height * tw / step_img.width)))
        c.paste(step_img, (x, y))
        d.rectangle([x - 2, y - 2, x + tw + 2, y + step_img.height + 2], outline=BORDER, width=1)
        d.text((x, y + step_img.height + 6), "%s" % label, font=font(11, True), fill=ACCENT)
    # 流转说明
    note_y = gy + 2 * (th + gap + 30) + 20
    d.line([24, note_y, W - 24, note_y], fill=BORDER)
    d.text((24, note_y + 10), "步序流转（不可跳步）：", font=font(13, True), fill=INK)
    flow = "① 新建包 → ② 填内容+导入 → ③ 同步 → ④ 校验 → ⑤ 打包 → ⑥ 安装 → ⑦ 对账 → ⑧ 回注 → ⑨ 出图 → ⑩ 零 Rust"
    d.text((24, note_y + 36), flow, font=font(13), fill=ACCENT)
    d.text((24, note_y + 64), "每步机器判据（map_wizard.rs 白名单动作）：PASS → 自动推进下一步；FAIL → 失败即停（halted），只能重跑该步",
           font=font(11), fill=MUTED)
    d.text((24, note_y + 84), "内容运营核心判据 = 第 ⑩ 步「git diff crates/ 空」（游戏 6 crate 零改动，只动数据）",
           font=font(11), fill=GOLD)
    return c


# ---------------------------------------------------------------- 主流程
def gen_one(step_no):
    slug, label, res, hl_fn = STEPS[step_no]
    hl = hl_fn(step_no)
    img = draw_editor_screen(step_no, hl)
    out = OUT / ("map_editor_step_%s.png" % slug)
    OUT.mkdir(parents=True, exist_ok=True)
    img.save(out)
    return out


def gen_all():
    OUT.mkdir(parents=True, exist_ok=True)
    for i in range(10):
        p = gen_one(i)
        print("  %s (%d B)" % (p.name, p.stat().st_size))
    ov = draw_overview()
    ov.save(OUT / "map_editor_steps_overview.png")
    print("  map_editor_steps_overview.png (%d B)" % (OUT / "map_editor_steps_overview.png").stat().st_size)
    print("map_editor_step_render: 生成 10 步 + 总览 → %s" % OUT.relative_to(ROOT))
    return 0


def check():
    missing = []
    for i in range(10):
        p = OUT / ("map_editor_step_%s.png" % STEPS[i][0])
        if not p.is_file() or p.stat().st_size < 2048:
            missing.append(p.name)
    ov = OUT / "map_editor_steps_overview.png"
    if not ov.is_file() or ov.stat().st_size < 2048:
        missing.append("map_editor_steps_overview.png")
    if missing:
        print("map_editor_step_render check: %d 张缺/过小" % len(missing))
        for m in missing:
            print("  [FAIL] %s" % m)
        return 1
    print("map_editor_step_render check: 0 缺（10 步 + 总览全在位非空）")
    return 0


def main(argv):
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="只读：产物在位非空，缺则非 0")
    ap.add_argument("--only", help="只重出指定 slug（如 04_verify；总览需全齐后才合成）")
    a = ap.parse_args(argv)
    if a.check:
        return check()
    if a.only:
        for i, s in enumerate(STEPS):
            if s[0] == a.only:
                p = gen_one(i)
                print("map_editor_step_render: 重出 %s (%d B)" % (p.name, p.stat().st_size))
                return 0
        print("map_editor_step_render: 未知 slug %s（可选：%s）" % (a.only, "/".join(s[0] for s in STEPS)), file=sys.stderr)
        return 1
    return gen_all()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
