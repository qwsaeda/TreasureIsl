# -*- coding: utf-8 -*-
"""S7.2 辅助：A 类九宫格切图（scripts/pixel_upscale.py 已内置 nine_slice，本脚本独立可跑）。

读 ref/asset_classification.json 中 class == A_ui 的条目，
从 assets_raw 解 PCX → RGBA → 2x 最近邻 → 切九宫格（边缘 1/4 像素带）→
assets_hd/<容器>/<name>_hd_9s.png + 更新 manifest_hd.json 的 nine_slice 参数。

Inkscape 矢量化（可选人工环节）不在本脚本范围。
用法：python scripts/vectorize_ui.py
零第三方依赖（标准库 + Pillow 写 PNG；缺 Pillow 时 PPM 兜底）。
"""
import json
import re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
HD = REPO / "assets_hd"
MANIFEST = REPO / "manifest_hd.json"
import sys

sys.path.insert(0, str(Path(__file__).parent))
from pixel_upscale import fnv1a64, idx_to_rgba, nine_slice, pcx_decode, scale_nearest, write_image  # noqa: E402

SCALE = 2


def main() -> None:
    cls = json.loads((REPO / "ref" / "asset_classification.json").read_text())
    entries = {k: v for k, v in cls["entries"].items() if v["class"] == "A_ui"}
    manifest = json.loads(MANIFEST.read_text()) if MANIFEST.is_file() else {}

    n = 0
    for key, meta in sorted(entries.items()):
        name = meta["name"]
        container = meta["container"]
        pcx_path = REPO / "assets_raw" / container / name
        if not pcx_path.is_file():
            continue
        w, h, pixels, palette = pcx_decode(pcx_path.read_bytes())
        rgba = idx_to_rgba(w, h, pixels, palette)
        srgba, sw, sh = scale_nearest(rgba, w, h, SCALE)
        base = re.sub(r"\.pcx$", "", name, flags=re.IGNORECASE)
        out = (HD / container / f"{base}_hd_9s.png")
        fmt = write_image(srgba, sw, sh, out)
        rel = out.relative_to(REPO).as_posix()
        manifest[rel] = {
            "class": "A_ui",
            "container": container,
            "name": base + "_9s",
            "width": sw,
            "height": sh,
            "fnv1a64": fnv1a64(out.read_bytes()),
        }
        # 九宫格切分参数（边缘 1/4 像素带，放大后）
        bw = max(1, w // 4) * SCALE
        bh = max(1, h // 4) * SCALE
        manifest[rel]["nine_slice"] = {"border_w": bw, "border_h": bh}
        n += 1

    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True))
    print(f"[vectorize_ui] {n} 个 A 类九宫格切图 → manifest_hd.json（格式: {fmt}）")


if __name__ == "__main__":
    main()
