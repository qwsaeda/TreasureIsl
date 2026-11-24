#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""export_parity_check.py — S86.4「导出即生效」像素判据（只读、幂等）。

对比「编辑器预览真渲染 PNG」与「编辑器导出→导入游戏后同 scene/viewport 出图 PNG」：
逐像素 RGBA 比较，差异像素数 = 0 且尺寸一致 ⇒ PASS（可见即可得的机器判据）。

用途：
  python scripts/export_parity_check.py --preview <p1.png> --exported <p2.png>
  python scripts/export_parity_check.py --preview <p1.png> --exported <p2.png> --json
  python scripts/export_parity_check.py --check   # 自检：对同一文件两次应 diff 0

退出码：0 = PASS（diff 0）；1 = FAIL（尺寸不符或存在差异像素）；2 = 参数/文件错误。
"""
from __future__ import annotations
import argparse
import json
import sys
from pathlib import Path

try:
    from PIL import Image, ImageChops
except ImportError:  # pragma: no cover
    print("export_parity_check: PIL 不可用", file=sys.stderr)
    sys.exit(2)


def diff(preview: Path, exported: Path) -> tuple[bool, dict]:
    if not preview.is_file() or not exported.is_file():
        return False, {"error": "文件不存在", "preview": str(preview), "exported": str(exported)}
    a = Image.open(preview).convert("RGBA")
    b = Image.open(exported).convert("RGBA")
    if a.size != b.size:
        return False, {"preview_size": a.size, "exported_size": b.size, "size_mismatch": True}
    d = ImageChops.difference(a, b)
    bbox = d.getbbox()
    # 统计差异像素（RGBA 任一通道不同即算差异）
    differing = sum(1 for px in d.getdata() if px != (0, 0, 0, 0))
    report = {
        "size": list(a.size),
        "differing_pixels": differing,
        "bbox": list(bbox) if bbox else None,
        "exact_equal": differing == 0,
    }
    return differing == 0, report


def main() -> int:
    ap = argparse.ArgumentParser(description="S86.4 导出即生效像素判据（diff 0 = PASS）")
    ap.add_argument("--preview", help="编辑器预览真渲染 PNG")
    ap.add_argument("--exported", help="导出→导入后游戏同 scene 出图 PNG")
    ap.add_argument("--check", action="store_true", help="自检：同一文件 diff 应为 0")
    ap.add_argument("--json", action="store_true", help="结构化输出 JSON")
    args = ap.parse_args()

    if args.check:
        p = Path(__file__).resolve().parent.parent / "evidence/bevy_native_index.png"
        if not p.is_file():
            print("export_parity_check: 自检基准图不存在", file=sys.stderr)
            return 2
        ok, report = diff(p, p)
    elif args.preview and args.exported:
        ok, report = diff(Path(args.preview), Path(args.exported))
    else:
        ap.error("需 --preview + --exported，或 --check")

    if args.json:
        print(json.dumps({"pass": ok, **report}, ensure_ascii=False))
    else:
        if "error" in report:
            print(f"export_parity_check: {report['error']}")
        else:
            print(f"export_parity_check: {'PASS' if ok else 'FAIL'} "
                  f"尺寸={report.get('size')} 差异像素={report.get('differing_pixels')} "
                  f"精确一致={report.get('exact_equal')}")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
