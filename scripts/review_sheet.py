#!/usr/bin/env python3
"""review_sheet.py — 迭看图合成一张压缩 contact sheet（用户确认图一次看全）。

背景：用户 2026-10-31「让我确认的图片，都合成一张图压缩大小让我看」→ 迭看图
（main_map v6 / VS v2 等引擎渲染 + 合成稿）逐张迭看太散、单张 8MB 传输重，
本工具把多张迭看图缩略拼成一张 contact sheet（PNG-8 压缩，~1MB 内），供用户一次确认。

用法（幂等，`--check` 只读）：
  python3 scripts/review_sheet.py --out evidence/ui_concept/review_sheet.png \\
      evidence/ui_concept/main_map_v6_engine.png \\
      evidence/ui_concept/vs_v2_clash_engine.png \\
      evidence/ui_concept/vs_v2_duel_engine.png \\
      evidence/ui_concept/vs_v2_result_win_engine.png \\
      evidence/ui_concept/vs_v2_result_lose_engine.png
  python3 scripts/review_sheet.py --check --out evidence/ui_concept/review_sheet.png

工具契约（14_tooling_catalog §1.2）：幂等 + `--check` 只读（缺输入/缺 PIL 非 0）+ 非 0 退出。
"""
from __future__ import annotations
import argparse
import os
import sys

from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _resolve(p: str) -> str:
    return p if os.path.isabs(p) else os.path.join(ROOT, p)


def _font(size: int):
    for cand in (
        r"C:\Windows\Fonts\msyh.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ):
        try:
            return ImageFont.truetype(cand, size)
        except OSError:
            continue
    return ImageFont.load_default()


def build(images: list[str], out: str, cell: int = 540, cols: int = 2) -> int:
    """把 images 缩略拼成 contact sheet（cell 每格、cols 列）→ out（PNG-8 压缩）。"""
    paths = [_resolve(p) for p in images]
    missing = [p for p in paths if not os.path.exists(p)]
    if missing:
        print(f"[review_sheet] 缺输入: {missing}", file=sys.stderr)
        return 1
    loads = []
    for p in paths:
        im = Image.open(p).convert("RGBA")
        # 缩略到 cell（长边），保持比例；竖图缩略高 = cell
        im.thumbnail((cell, cell), Image.LANCZOS)
        loads.append((os.path.basename(p), im))
    n = len(loads)
    cols = max(1, min(cols, n))
    rows = (n + cols - 1) // cols
    label_h = 30
    pad = 10
    sheet_w = cols * cell + (cols + 1) * pad
    sheet_h = rows * (cell + label_h) + (rows + 1) * pad
    sheet = Image.new("RGB", (sheet_w, sheet_h), (42, 38, 34))
    draw = ImageDraw.Draw(sheet)
    font = _font(17)
    for i, (name, im) in enumerate(loads):
        cx = pad + (i % cols) * (cell + pad)
        cy = pad + (i // cols) * (cell + label_h + pad)
        # 居中贴格（灰底 + 白描边）
        bg = Image.new("RGBA", (cell, cell), (30, 28, 26, 255))
        bg.alpha_composite(im, ((cell - im.size[0]) // 2, (cell - im.size[1]) // 2))
        sheet.paste(bg.convert("RGB"), (cx, cy))
        draw.rectangle((cx, cy, cx + cell - 1, cy + cell - 1), outline=(180, 150, 100), width=1)
        draw.text((cx + 6, cy + cell + 6), name, fill=(235, 205, 120), font=font)
    out_abs = _resolve(out)
    os.makedirs(os.path.dirname(out_abs), exist_ok=True)
    sheet.save(out_abs, "PNG", optimize=True)
    size_kb = os.path.getsize(out_abs) // 1024
    print(f"[review_sheet] 已存 {out_abs}（{n} 张 → {sheet_w}x{sheet_h}，{size_kb}KB）")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("images", nargs="*", help="迭看图路径（相对仓库根或绝对）")
    ap.add_argument("--out", required=True, help="输出 contact sheet 路径")
    ap.add_argument("--cell", type=int, default=540, help="每格长边像素（默认 540）")
    ap.add_argument("--cols", type=int, default=2, help="列数（默认 2）")
    ap.add_argument("--check", action="store_true", help="只读：输入全在位 + PIL 可用，不写文件")
    args = ap.parse_args()
    if not args.images:
        print("[review_sheet] 需 ≥1 张迭看图（--out 必填）", file=sys.stderr)
        return 1
    paths = [_resolve(p) for p in args.images]
    missing = [p for p in paths if not os.path.exists(p)]
    if args.check:
        if missing:
            print(f"[review_sheet --check] 缺输入: {missing}", file=sys.stderr)
            return 1
        print(f"[review_sheet --check] ok（{len(paths)} 张输入全在位 + PIL 可用）")
        return 0
    return build(args.images, args.out, args.cell, args.cols)


if __name__ == "__main__":
    sys.exit(main())
