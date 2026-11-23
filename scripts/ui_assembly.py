#!/usr/bin/env python3
"""ui_assembly.py — UI 零件组装装配图（S76 图文配套，S39「拆件组件化 + 多贴图分层渲染」）。

用途
----
把 scenes.json 某场景的 primitives 按 z-order 分层，逐层把真实拆件资产
（assets/ui/*.png + ui_manifest 登记的 icon）叠加，产出「分层组装过程 + 拆件目录」
装配图 PNG。零件 = 项目登记资产 + 实测 rect，非 PIL 裁切完整图、非 AI 概念图。

用法（先 export PATH="$PATH:/root/.cargo/bin" 不需要，纯 Python + Pillow）
---------------------------------------------------------------------------
    python3 scripts/ui_assembly.py --scene index --orientation landscape      # 出图
    python3 scripts/ui_assembly.py --scene main_map --orientation portrait \
        --output evidence/scenes_assembly_main_map.png                        # 指定输出
    python3 scripts/ui_assembly.py --scene index --check                      # 只读体检：拆件在位/数据对账
    python3 scripts/ui_assembly.py --list                                     # 列出可出图场景 + 层数
    python3 scripts/ui_assembly.py --scene index --json                       # 机读分层清单

退出码
------
    0  成功 / --check 全过
    1  参数错 / 场景不存在 / 拆件缺失 / 写入失败
    2  前置缺失（scenes.json / ui_manifest.json / 字体 / Pillow 不可用）

依赖：Pillow（已装）；字体 assets/fonts/lxgw-wenkai-reg.ttf（项目内，非系统字体）。
产物默认落 evidence/scenes_assembly_<scene>_<orientation>.png（gitignore 的 evidence/，
非交付证据，仅文档配图）。可复现：--json / 报告不含时间戳，重复出图内容一致。
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import OrderedDict
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as exc:  # 前置缺失 → exit 2
    print(f"[ui_assembly] Pillow 不可用: {exc}", file=sys.stderr)
    sys.exit(2)

REPO = Path(__file__).resolve().parent.parent
SCENES = REPO / "assets/data/scenes.json"
MANIFEST = REPO / "assets/ui_manifest.json"
FONT_PATH = REPO / "assets/fonts/lxgw-wenkai-reg.ttf"

# 配色（项目暖金羊皮调）
BG = (10, 12, 18)
CARD = (20, 25, 36)
GOLD = (222, 178, 86)
TEXT = (235, 232, 222)
SUB = (150, 158, 175)


def font(sz: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    try:
        return ImageFont.truetype(str(FONT_PATH), sz)
    except OSError:
        return ImageFont.load_default()


def load_json(path: Path) -> dict:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"[ui_assembly] 读取/解析失败 {path}: {exc}", file=sys.stderr)
        sys.exit(2)


def icon_file(icon_name: str, manifest: dict) -> str | None:
    """从 ui_manifest 按 name=icon_<name> 找 file；找不到按约定 file。"""
    for a in manifest.get("assets", []):
        if a.get("name") == f"icon_{icon_name}":
            return a.get("file")
    return None


def layers_of(scene_data: dict, orientation: str) -> tuple[int, int, OrderedDict[int, list]]:
    """返回 (design_w, design_h, {z: [primitive,...]})；z 升序即绘制顺序。"""
    node = scene_data if orientation == "portrait" else scene_data["landscape"]
    prims = node["primitives"]
    w = 1080 if orientation == "portrait" else 1920
    h = 1920 if orientation == "portrait" else 1080
    groups: OrderedDict[int, list] = OrderedDict()
    for p in prims:
        z = int(p.get("z", 0))
        groups.setdefault(z, []).append(p)
    return w, h, OrderedDict(sorted(groups.items()))


def part_image(prim: dict, manifest: dict) -> tuple[str, Path] | None:
    """原语 → (资产相对路径, 绝对路径)；非资产原语（text/rect/icon 无登记）返回 None。"""
    params = prim.get("params", {})
    asset = params.get("asset")
    if asset:
        p = REPO / asset
        return (asset, p) if p.is_file() else (asset, p)
    icon = params.get("icon")
    if icon:
        f = icon_file(icon, manifest)
        if f:
            p = REPO / f
            return (f, p)
    return None


def check_parts(prims: list, manifest: dict) -> list[str]:
    """只读体检：所有带 asset/icon 的原语，其 PNG 必须在位。"""
    problems: list[str] = []
    for p in prims:
        info = part_image(p, manifest)
        if info and not info[1].is_file():
            problems.append(f"缺拆件 {info[0]}（原语 z={p.get('z')} kind={p.get('kind')}）")
    return problems


def compose(groups: OrderedDict[int, list], manifest: dict, w: int, h: int, upto: int) -> Image.Image:
    """叠加前 upto 个 z 层，返回合成图（含真实拆件）。"""
    im = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    for idx, (z, prims) in enumerate(groups.items()):
        if idx >= upto:
            break
        for p in prims:
            info = part_image(p, manifest)
            if not info or not info[1].is_file():
                continue
            rect = p.get("rect", {})
            rw, rh = int(rect.get("w", 0)), int(rect.get("h", 0))
            if rw <= 0 or rh <= 0:
                continue
            part = Image.open(info[1]).convert("RGBA")
            if part.size != (rw, rh):
                part = part.resize((rw, rh), Image.LANCZOS)
            im.alpha_composite(part, (int(rect.get("x", 0)), int(rect.get("y", 0))))
    return im


def build_sheet(scene: str, orientation: str, out: Path) -> dict:
    scenes = load_json(SCENES)
    if scene not in scenes:
        print(f"[ui_assembly] 场景不存在: {scene}（可用: {', '.join(scenes)}）", file=sys.stderr)
        sys.exit(1)
    manifest = load_json(MANIFEST)
    w, h, groups = layers_of(scenes[scene], orientation)
    all_prims = [p for ps in groups.values() for p in ps]
    problems = check_parts(all_prims, manifest)
    if problems:
        for msg in problems:
            print(f"[ui_assembly] ERROR {msg}", file=sys.stderr)
        sys.exit(1)

    n = len(groups)
    tw, th = 600, int(600 * h / w)
    margin, gap = 40, 12
    cols = min(n, 4)
    sheet_w = margin * 2 + cols * tw + (cols - 1) * gap
    stage_block_h = th + 70
    catalog_rows = (sum(len(ps) for ps in groups.values()) + 3) // 4
    sheet_h = 120 + stage_block_h + 60 + catalog_rows * 120 + 40
    canvas = Image.new("RGB", (sheet_w, sheet_h), BG)
    d = ImageDraw.Draw(canvas)

    d.text((margin, 20), f"{scene} · 零件组装装配图（{orientation} {w}×{h}）", font=font(28), fill=GOLD)
    d.text((margin, 58), f"{n} 个 z 层 · {len(all_prims)} 个原语 · 零件=assets/ui 真实拆件 + scenes.json rect", font=font(17), fill=SUB)

    # 上：分层叠加阶段
    for i, (z, prims) in enumerate(groups.items()):
        col, row = i % cols, i // cols
        x = margin + col * (tw + gap)
        y = 100 + row * (stage_block_h + gap)
        d.rounded_rectangle((x, y, x + tw, y + stage_block_h), radius=6, fill=CARD, outline=GOLD, width=2)
        stage = compose(groups, manifest, w, h, i + 1).convert("RGB").resize((tw, th), Image.LANCZOS)
        canvas.paste(stage, (x + 2, y + 2))
        d.text((x + 8, y + th + 8), f"stage {i+1}/{n} · z={z} · {len(prims)} 原语", font=font(16), fill=GOLD)

    # 下：拆件目录（只列带真实资产的原语）
    cy = 100 + ((n + cols - 1) // cols) * (stage_block_h + gap) + 30
    d.text((margin, cy), "拆件目录（真实资产 → 落点 rect / z 层）", font=font(22), fill=GOLD)
    cy += 40
    parts = [(p, part_image(p, manifest)) for ps in groups.values() for p in ps]
    parts = [(p, info) for p, info in parts if info]
    cw = 460
    catalog_cols = max(1, (sheet_w - margin * 2) // cw)
    for i, (p, info) in enumerate(parts):
        col, row = i % catalog_cols, i // catalog_cols
        x = margin + col * cw
        y = cy + row * 120
        rel, path = info
        part = Image.open(path).convert("RGBA")
        osz = part.size
        part.thumbnail((110, 76), Image.LANCZOS)
        canvas.paste(part, (x + 4, y + 4), part)
        r = p.get("rect", {})
        d.text((x + 124, y + 8), rel.rsplit("/", 1)[-1], font=font(15), fill=TEXT)
        d.text((x + 124, y + 32), f"原尺寸 {osz[0]}×{osz[1]}", font=font(12), fill=SUB)
        d.text((x + 124, y + 54), f"落点 ({r.get('x')},{r.get('y')},{r.get('w')},{r.get('h')}) z={p.get('z')}", font=font(12), fill=GOLD)

    out.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(out)
    return {"scene": scene, "orientation": orientation, "design": f"{w}x{h}",
            "z_layers": n, "primitives": len(all_prims), "asset_parts": len(parts),
            "out": str(out.relative_to(REPO))}


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="UI 零件组装装配图（scenes.json 分层 + 真实拆件）")
    ap.add_argument("--scene", help="场景名（index/select_role/.../main_map）")
    ap.add_argument("--orientation", choices=["portrait", "landscape"], default="landscape")
    ap.add_argument("--output", help="输出 PNG 路径（默认 evidence/scenes_assembly_<scene>_<orient>.png）")
    ap.add_argument("--check", action="store_true", help="只读体检：拆件在位/数据对账，不出图")
    ap.add_argument("--json", action="store_true", help="输出机读分层清单")
    ap.add_argument("--list", action="store_true", help="列出可出图场景 + z 层数")
    args = ap.parse_args(argv)

    scenes = load_json(SCENES)
    if args.list:
        for name, data in scenes.items():
            if not isinstance(data, dict) or "primitives" not in data:
                continue  # 跳过 schema_version 等非场景键
            for orient in ("portrait", "landscape"):
                node = data if orient == "portrait" else data.get("landscape", {})
                prims = node.get("primitives", [])
                zs = sorted({int(p.get("z", 0)) for p in prims})
                print(f"{name:12} {orient:10} {len(prims):3} 原语 / {len(zs)} 层 (z={zs})")
        return 0
    if not args.scene or args.scene not in scenes:
        print("[ui_assembly] --scene 缺失或不存在（用 --list 查看）", file=sys.stderr)
        return 1

    manifest = load_json(MANIFEST)
    w, h, groups = layers_of(scenes[args.scene], args.orientation)
    all_prims = [p for ps in groups.values() for p in ps]
    problems = check_parts(all_prims, manifest)
    if problems:
        for msg in problems:
            print(f"[ui_assembly] ERROR {msg}", file=sys.stderr)
        return 1
    summary = {"scene": args.scene, "orientation": args.orientation, "design": f"{w}x{h}",
               "z_layers": len(groups), "primitives": len(all_prims),
               "asset_parts": sum(1 for p in all_prims if part_image(p, manifest))}

    if args.json:
        print(json.dumps({**summary, "layers": [
            {"z": z, "count": len(ps), "parts": [part_image(p, manifest)[0] if part_image(p, manifest) else None for p in ps]}
            for z, ps in groups.items()]}, ensure_ascii=False, indent=1))
        return 0
    if args.check:
        print(f"[ui_assembly] 只读体检 OK: {summary}")
        return 0

    out = Path(args.output) if args.output else REPO / f"evidence/scenes_assembly_{args.scene}_{args.orientation}.png"
    if not out.is_absolute():
        out = REPO / out
    result = build_sheet(args.scene, args.orientation, out)
    print(f"[ui_assembly] 出图 OK: {json.dumps(result, ensure_ascii=False)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
