#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""原版 PCX 素材 → 透明 PNG（品红键透明），导入 `assets/ui/`。

用法：
  python scripts/pcx_to_png.py legacy/assets_raw/glpics/helm_base.pcx assets/ui/helm.png [--resize 128]

品红键（R&B=255 且 G<16 → alpha=0）；非品红像素 = 不透明。
`--resize` = 最长边缩放到该值（保持宽高比，LANCZOS）。
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

from PIL import Image

import pcx_ref


def convert(src: Path, dst: Path, resize: int | None) -> None:
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
    dst.parent.mkdir(parents=True, exist_ok=True)
    im.save(dst)
    print(f"[pcx_to_png] {src.name} → {dst} ({im.width}x{im.height})")


def main() -> int:
    ap = argparse.ArgumentParser(description="PCX → 透明 PNG（品红键）")
    ap.add_argument("src")
    ap.add_argument("dst")
    ap.add_argument("--resize", type=int, default=None)
    args = ap.parse_args()
    convert(Path(args.src), Path(args.dst), args.resize)
    return 0


if __name__ == "__main__":
    sys.exit(main())
