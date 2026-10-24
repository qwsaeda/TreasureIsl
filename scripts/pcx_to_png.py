#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""原版 PCX 素材 → 透明 PNG（品红键透明），导入 `assets/ui/`。

用法：
  python scripts/pcx_to_png.py legacy/assets_raw/glpics/helm_base.pcx assets/ui/helm.png [--resize 128]
  python scripts/pcx_to_png.py <in.pcx> <out.png> --check   # 只读校验：out 在位且与 in 重导出一致（不写文件）

品红键（R&B=255 且 G<16 → alpha=0）；非品红像素 = 不透明。
`--resize` = 最长边缩放到该值（保持宽高比，LANCZOS）。
退出码：0 = 成功/校验通过；1 = 源缺失 / `--check` 校验失败（缺失或漂移）。
"""
from __future__ import annotations

import argparse
import io
import sys
from pathlib import Path

from PIL import Image

import pcx_ref


def derive_png_bytes(src: Path, resize: int | None) -> tuple[bytes, int, int]:
    """PCX → PNG 字节（内存态，不落盘；`--check` 与写盘共用的确定性推导）。

    返回 (png_bytes, width, height)。
    """
    data = src.read_bytes()
    w, h, px = pcx_ref.decode(data)
    palette = data[0x40 : 0x40 + 768]
    assert len(palette) == 768, "PCX 调色板缺失（仅支持 256 色 8bpp）"
    # 透明键 = 亮绿(0,255,0) / 品红(255,0,255)（MS-DOS PCX 惯例）+ 角落色（该素材实际键色）
    corner = px[0]
    cr, cg, cb = palette[corner * 3], palette[corner * 3 + 1], palette[corner * 3 + 2]
    keys = {(0, 255, 0), (255, 0, 255), (cr, cg, cb)}
    rgba = bytearray()
    for idx in px:
        r = palette[idx * 3]
        g = palette[idx * 3 + 1]
        b = palette[idx * 3 + 2]
        a = 0 if (r, g, b) in keys else 255
        rgba.extend((r, g, b, a))
    im = Image.frombytes("RGBA", (w, h), bytes(rgba))
    if resize and (im.width > resize or im.height > resize):
        im.thumbnail((resize, resize), Image.LANCZOS)
    buf = io.BytesIO()
    im.save(buf, format="PNG")
    return buf.getvalue(), im.width, im.height


def convert(src: Path, dst: Path, resize: int | None) -> None:
    blob, w, h = derive_png_bytes(src, resize)
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_bytes(blob)
    print(f"[pcx_to_png] {src.name} → {dst} ({w}x{h})")


def main() -> int:
    ap = argparse.ArgumentParser(description="PCX → 透明 PNG（品红键）")
    ap.add_argument("src")
    ap.add_argument("dst")
    ap.add_argument("--resize", type=int, default=None)
    ap.add_argument(
        "--check",
        action="store_true",
        help="只读校验：dst 在位且与 src 重导出字节一致（不写文件；缺失/漂移 ⇒ 非 0 退出）",
    )
    args = ap.parse_args()
    src, dst = Path(args.src), Path(args.dst)
    if not src.is_file():
        print(f"[pcx_to_png] ERROR 源文件不存在: {src}")
        return 1
    if args.check:
        expected, _w, _h = derive_png_bytes(src, args.resize)
        if not dst.is_file():
            print(f"[pcx_to_png] --check FAIL: {dst} 不存在（先跑默认模式生成）")
            return 1
        if dst.read_bytes() != expected:
            print(f"[pcx_to_png] --check FAIL: {dst} 与 {src} 重导出不一致（漂移，需重新生成）")
            return 1
        print(f"[pcx_to_png] --check OK: {dst} 与 {src} 一致（只读未写文件）")
        return 0
    convert(src, dst, args.resize)
    return 0


if __name__ == "__main__":
    sys.exit(main())
