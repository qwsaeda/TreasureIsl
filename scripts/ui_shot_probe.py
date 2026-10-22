#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""出图取样：PNG 区域主色分布 / 最亮像素 / 裁剪放大 —— UI 出图验收的「看清某区域画了什么」。

用途（`docs/process/13_ui_asset_pipeline.md` §出图验收 + S19.6/S19.7 验收实操）：
出图后「肉眼缩略图看不清」是常见盲区（提示条底板与深海底同色、标记过小等），
本工具给出**可复核的像素证据**（区域主色 / 最亮像素 / 裁剪放大图）。

用法：
  python scripts/ui_shot_probe.py evidence/bevy_native_main_map.png --region 400,1420,280,100
      # 区域主色分布（top N）+ 最亮像素
  python scripts/ui_shot_probe.py evidence/bevy_native_main_map.png --crop 300,1000,780,580 --out /tmp/crop.png
      # 裁剪放大（×2）存盘，便于人工看
  python scripts/ui_shot_probe.py evidence/bevy_native_main_map.png --check
      # 非空帧校验（non-black 像素占比 < 98% ⇒ 空帧 ⇒ 非 0 退出）

约定：幂等 + 只读（仅 --crop 写出文件）+ `--check` 非 0 退出（`docs/process/04_tooling.md` §1）。
"""
from __future__ import annotations

import argparse
import os
import sys
from collections import Counter

try:
    from PIL import Image
except ImportError:  # pragma: no cover - 环境缺 PIL 时给出明确指引
    print('需要 Pillow：pip install pillow（或用 python -m pip install --user pillow）', file=sys.stderr)
    raise SystemExit(2) from None


def nonblack_ratio(im) -> float:
    rgb = im.convert('RGB')
    data = rgb.tobytes()
    nz = sum(1 for i in range(0, len(data), 3)
             if data[i] > 8 or data[i + 1] > 8 or data[i + 2] > 8)
    return nz / float(im.width * im.height)


def main() -> int:
    ap = argparse.ArgumentParser(description='出图取样（UI 出图验收）')
    ap.add_argument('png', help='PNG 路径')
    ap.add_argument('--region', help='区域 x,y,w,h（取样）')
    ap.add_argument('--crop', help='裁剪区 x,y,w,h')
    ap.add_argument('--out', help='裁剪输出路径（配 --crop）')
    ap.add_argument('--top', type=int, default=8, help='主色显示条数（默认 8）')
    ap.add_argument('--check', action='store_true', help='非空帧校验（空帧 ⇒ 非 0 退出）')
    args = ap.parse_args()

    if not os.path.exists(args.png):
        print('文件不存在：%s' % args.png, file=sys.stderr)
        return 2
    im = Image.open(args.png)
    ratio = nonblack_ratio(im)
    print('文件 %s  尺寸 %dx%d  non-black = %.4f' % (args.png, im.width, im.height, ratio))

    if args.region:
        x, y, w, h = (int(v) for v in args.region.split(','))
        box = im.crop((x, y, x + w, y + h)).convert('RGB')
        c = Counter(box.tobytes()[i:i + 3] for i in range(0, len(box.tobytes()), 3))
        print('区域 (%d,%d,%d,%d) 主色 top%d：' % (x, y, w, h, args.top))
        for col, n in c.most_common(args.top):
            print('   %-16s ×%d' % ('(%d, %d, %d)' % col, n))
        px = box.load()
        best = max(((px[px_x, px_y], px_x, px_y)
                    for px_y in range(box.height) for px_x in range(box.width)),
                   key=lambda t: sum(t[0]))
        print('   最亮像素 = %s @ 区域内 (%d, %d)' % (str(best[0]), best[1], best[2]))

    if args.crop:
        x, y, w, h = (int(v) for v in args.crop.split(','))
        out = args.out or 'crop.png'
        crop = im.crop((x, y, x + w, y + h))
        crop = crop.resize((crop.width * 2, crop.height * 2), Image.LANCZOS)
        crop.save(out)
        print('裁剪 (%d,%d,%d,%d) ×2 → %s（%dx%d）' % (x, y, w, h, out, crop.width, crop.height))

    if args.check:
        if ratio < 0.98:
            print('[ui_shot_probe] FAIL：空帧/近空帧（non-black %.4f < 0.98）' % ratio, file=sys.stderr)
            return 1
        print('[ui_shot_probe] PASS：非空帧（non-black %.4f）' % ratio)
    return 0


if __name__ == '__main__':
    sys.exit(main())
