#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""UI 版面线图（规格驱动：1080 坐标表叠加到美术图上 ⇒ 版面评审/出图验收的像素证据）。

把**声明式规格**（`assets/ui_batches/*_layout.json`：底图 + 逐元素矩形 + 可选环岛示意）
合成一张「坐标表线图」供用户过目（设计稿先行，A 纪律 §UI 视觉改动）。

定位（`docs/process/14_tooling_catalog.md` §1.2 工具契约 + §1.5 组合优先）：
- **不是**一次性脚本（EXP-115 同类缺口第 3 次发作的补位：此前该能力用 `python -c` 内联手搓）；
- 与 `ui_asset_batch.py sheet`（批次**资产**拼版，1 资产 1 格）分工：本工具画**版面矩形叠层**；
- 与 `ui_preview.ps1`（真实场景渲染出图）分工：本工具 = 规格定案期的**设计稿线图**，
  `ui_preview` = 落码后的**真实渲染**；S21.9 出图验收两边对照。

用法：
    python scripts/ui_layout_sheet.py --spec assets/ui_batches/main_map_v4_layout.json        # 渲染
    python scripts/ui_layout_sheet.py --spec <spec> --out <evidence/...png> --scale 0.3      # 覆盖输出/缩放
    python scripts/ui_layout_sheet.py --spec <spec> --check                                  # 只读校验（非 0 = 违规）

幂等：同规格 + 同底图 ⇒ 输出逐像素一致（无随机、无时间戳入图）。
退出码：0 = 成功 / 1 = 规格或产物问题（`--check` 发现问题即 1）。
"""
from __future__ import annotations

import argparse
import json
import pathlib
import sys

from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parent.parent

_CJK_FONTS = [
    r"C:\Windows\Fonts\msyh.ttc",
    r"C:\Windows\Fonts\simhei.ttf",
    r"C:\Windows\Fonts\arial.ttf",
]


def _font(size: int) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    for cand in _CJK_FONTS:
        try:
            return ImageFont.truetype(cand, size)
        except OSError:
            continue
    return ImageFont.load_default()


def load_spec(path: pathlib.Path) -> dict:
    if not path.is_file():
        print(f"规格不存在: {path}", file=sys.stderr)
        raise SystemExit(1)
    try:
        spec = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        print(f"规格 JSON 解析失败: {exc}", file=sys.stderr)
        raise SystemExit(1)
    if not isinstance(spec, dict):
        print("规格须为 JSON 对象", file=sys.stderr)
        raise SystemExit(1)
    return spec


def _check_color(name: str, color: object, problems: list[str]) -> str | None:
    if not isinstance(color, str) or len(color) != 7 or color[0] != "#":
        problems.append(f"{name}: color 须 #rrggbb，实为 {color!r}")
        return None
    try:
        int(color[1:], 16)
    except ValueError:
        problems.append(f"{name}: color 非十六进制 {color!r}")
        return None
    return color


def check_spec(spec: dict, art_size: tuple[int, int] | None) -> list[str]:
    problems: list[str] = []
    if not spec.get("art"):
        problems.append("缺 art（底图，相对仓库根）")
    if not spec.get("out"):
        problems.append("缺 out（线图输出，须 evidence/ 下）")
    elif not str(spec["out"]).lstrip("./").startswith("evidence/"):
        problems.append(f"out 须落在 evidence/（线图 = 评审证据，勿散放）：{spec['out']}")
    if not isinstance(spec.get("scale"), (int, float)) or not (0.05 <= spec["scale"] <= 1.0):
        problems.append(f"scale 须 0.05~1.0，实为 {spec.get('scale')!r}")
    aw, ah = art_size or (0, 0)
    for i, box in enumerate(spec.get("boxes", [])):
        if not isinstance(box, dict):
            problems.append(f"boxes[{i}] 须为对象")
            continue
        if not str(box.get("name", "")).strip():
            problems.append(f"boxes[{i}] name 为空")
        r = box.get("rect")
        if not (isinstance(r, list) and len(r) == 4 and all(isinstance(v, int) for v in r)):
            problems.append(f"boxes[{i}] rect 须 [x,y,w,h] 整数，实为 {r!r}")
            continue
        x, y, w, h = r
        if w <= 0 or h <= 0:
            problems.append(f"boxes[{i}] w/h 须 > 0")
        if aw and ah and (x < 0 or y < 0 or x + w > aw or y + h > ah):
            problems.append(f"boxes[{i}] {box.get('name')} 越出底图 {aw}×{ah}: {r}")
        _check_color(f"boxes[{i}]", box.get("color"), problems)
    ring = spec.get("ring")
    if ring is not None:
        for k in ("cx", "cy", "rx", "ry", "nodes"):
            if not isinstance(ring.get(k), int) or ring[k] <= 0:
                problems.append(f"ring.{k} 须正整数，实为 {ring.get(k)!r}")
        _check_color("ring", ring.get("color"), problems)
    return problems


def render(spec: dict, out: pathlib.Path, scale: float) -> pathlib.Path:
    art_path = ROOT / spec["art"]
    if not art_path.is_file():
        print(f"底图不存在: {art_path}", file=sys.stderr)
        raise SystemExit(1)
    art = Image.open(art_path).convert("RGB")
    aw, ah = art.size
    problems = check_spec(spec, (aw, ah))
    if problems:
        for p in problems:
            print(f"  ERROR {p}", file=sys.stderr)
        raise SystemExit(1)

    im = art.resize((int(aw * scale), int(ah * scale)), Image.LANCZOS)
    d = ImageDraw.Draw(im)
    font = _font(14)
    for box in spec["boxes"]:
        x, y, w, h = box["rect"]
        color = box["color"].upper()
        X, Y = int(x * scale), int(y * scale)
        d.rectangle(
            [X, Y, X + int(w * scale), Y + int(h * scale)],
            outline=color,
            width=2,
        )
        d.text((X + 2, Y + 2), box["name"], fill=color, font=font)
    ring = spec.get("ring")
    if ring:
        color = ring["color"].upper()
        cx, cy = int(ring["cx"] * scale), int(ring["cy"] * scale)
        rx, ry = int(ring["rx"] * scale), int(ring["ry"] * scale)
        d.ellipse([cx - rx, cy - ry, cx + rx, cy + ry], outline=color, width=2)
        import math

        n = ring["nodes"]
        for k in range(n):
            a = k * 2 * math.pi / n
            nx, ny = cx + int(rx * math.cos(a)), cy + int(ry * math.sin(a))
            d.ellipse([nx - 7, ny - 7, nx + 7, ny + 7], fill=color, outline="#5a3c0a")
        d.text((cx - 12, cy - 6), ring.get("label", ""), fill=color, font=font)
    out.parent.mkdir(parents=True, exist_ok=True)
    im.save(out)
    print(f"  版面线图已存: {out}（{im.size[0]}×{im.size[1]}，boxes={len(spec['boxes'])}" +
          (" +ring" if ring else "") + "）")
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description="UI 版面线图（规格 → 坐标表叠加美术图）")
    ap.add_argument("--spec", required=True, help="版面线图规格 JSON（assets/ui_batches/*_layout.json）")
    ap.add_argument("--out", help="覆盖规格里的 out")
    ap.add_argument("--scale", type=float, help="覆盖规格里的 scale（0.05~1.0）")
    ap.add_argument("--check", action="store_true", help="只读校验规格（不写图；发现问题非 0 退出）")
    args = ap.parse_args()

    spec = load_spec(pathlib.Path(args.spec))
    art = ROOT / spec.get("art", "")
    art_size = None
    if art.is_file():
        with Image.open(art) as im:
            art_size = im.size

    problems = check_spec(spec, art_size)
    if args.check:
        for p in problems:
            print(f"  ERROR {p}")
        print(f"ui_layout_sheet check: {len(problems)} problem(s)")
        return 1 if problems else 0

    out = pathlib.Path(args.out) if args.out else ROOT / spec["out"]
    scale = args.scale if args.scale is not None else spec["scale"]
    render(spec, out, scale)
    return 0


if __name__ == "__main__":
    sys.exit(main())
