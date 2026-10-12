#!/usr/bin/env python3
"""S7.3 关键图 AI 回炉：B 类角色 2x 增强超分（OpenCV 高质量替代最近邻）。

背景（docs/impl/14_asset_enhancement.md §3.2/§3.3）：
  S7.2 对 B_portrait 用 NEAREST 2x 打底（pixel_upscale.py:47）。本机无独显，
  Real-ESRGAN（PyTorch）/ ComfyUI+SD 跑不动 → 用 OpenCV 4.14 做「高质量 2x 增强」：
  INTER_LANCZOS4 平滑放大 + 双边滤波（保边去噪）+ unsharp mask（细节增强）。
  产出 *_hd_enhanced.png 与 *_hd.png 并存；渲染层优先读 enhanced，缺失时回退。

管线（每张）：
  PCX RLE 解码 → 索引→RGBA（bg slot 0 透明）
  → INTER_LANCZOS4 2x（放大）
  → cv2.bilateralFilter（保边去噪，RGB 分量）
  → unsharp mask（细节增强，仅不透明像素）
  → alpha 通道：Lanczos 2x 后 0/255 二值化（防半透明脏边）
  → 写 assets_hd/<容器>/<name>_hd_enhanced.png（PNG-32）
  → manifest_hd.json 增条目（fnv1a64 对 PNG 字节，与 build_hd_assets 同源）

零新增依赖：只用 opencv-python + numpy + Pillow（本机已有，§5.9）。
"""
import json
import re
import struct
import sys
from pathlib import Path

import cv2
import numpy as np

REPO = Path(__file__).resolve().parent.parent
RAW = REPO / "legacy" / "assets_raw"
HD = REPO / "assets_hd"
CLASS = REPO / "legacy" / "ref" / "asset_classification.json"
MANIFEST = REPO / "assets_hd" / "manifest_hd.json"
STATS = REPO / "assets_hd" / "_stats.json"

SCALE = 2


def fnv1a64(data: bytes) -> str:
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return format(h, "016x")


def pcx_decode(data: bytes):
    """8bpp RLE PCX → (w, h, pixels, palette)。与 legacy/scripts/pixel_upscale.py 同源。"""
    assert len(data) >= 128 and data[0] == 0x0A and data[1] == 5 and data[2] == 1 and data[3] == 8
    x_max, y_max = struct.unpack_from("<HH", data, 8)
    width, height = x_max + 1, y_max + 1

    def rle_decode(need: int):
        out = bytearray()
        i = 128
        while len(out) < need and i < len(data):
            b = data[i]
            i += 1
            if b & 0xC0 == 0xC0:
                count = b & 0x3F
                if count == 0 or i >= len(data):
                    return None
                out.extend([data[i]] * count)
                i += 1
            else:
                out.append(b)
        return out if len(out) == need else None

    cands = sorted({width, struct.unpack_from("<H", data, 0x3E)[0], data[0x42]})
    stride, raw = width, None
    for cand in cands:
        if cand < width:
            continue
        out = rle_decode(cand * height)
        if out is not None:
            raw, stride = out, cand
            break
    if raw is None:
        raise ValueError("PCX RLE 反解失败")
    rows = [raw[y * stride : y * stride + width] for y in range(height)]
    pixels = b"".join(bytes(r) for r in rows)

    palette = None
    if len(data) >= 769 and data[-769] == 0x0C:
        tail = data[-768:]
        palette = [(tail[i * 3], tail[i * 3 + 1], tail[i * 3 + 2]) for i in range(256)]
    if palette is None:
        base = [(0, 0, 0), (0, 0, 170), (0, 170, 0), (0, 170, 170),
                (170, 0, 0), (170, 0, 170), (170, 85, 0), (170, 170, 170),
                (85, 85, 85), (85, 85, 255), (85, 255, 85), (85, 255, 255),
                (255, 85, 85), (255, 85, 255), (255, 255, 85), (255, 255, 255)]
        palette = [base[i] if i < 16 else (i, i, i) for i in range(256)]
    return width, height, pixels, palette


def to_rgba(w: int, h: int, pixels: bytes, palette, bg_slot: int = 0) -> np.ndarray:
    """索引 → HxWx4 RGBA（OpenCV BGRA 转 RGB 顺序统一为 R,G,B,A）。"""
    idx = np.frombuffer(pixels, np.uint8).reshape(h, w)
    pal = np.array(palette, np.uint8).reshape(256, 3)
    rgb = pal[idx]
    alpha = np.where(idx == bg_slot, 0, 255).astype(np.uint8)
    out = np.dstack([rgb, alpha]).astype(np.uint8)  # HxWx4 RGB(A)
    return out


def enhance_bilateral_unsharp(rgba: np.ndarray, scale: int = SCALE) -> np.ndarray:
    """2x 高质量增强：Lanczos4 放大 + 双边滤波保边 + unsharp 细节 + alpha 二值化。"""
    h, w, _ = rgba.shape
    rgb = rgba[:, :, :3]
    a = rgba[:, :, 3]

    # 放大到 2x（透明区填黑，bilateral 后再按原 alpha 扩回来）
    big = cv2.resize(rgb, (w * scale, h * scale), interpolation=cv2.INTER_LANCZOS4)
    a_big = cv2.resize(a, (w * scale, h * scale), interpolation=cv2.INTER_LANCZOS4)

    # 双边滤波：保边去噪（d=9 邻域，sigma 适中，角色立绘细节保留）
    den = cv2.bilateralFilter(big, d=9, sigmaColor=40, sigmaSpace=40)

    # unsharp mask：细节增强（仅轻微，防过锐锯齿）
    blur = cv2.GaussianBlur(den, (0, 0), sigmaX=1.5)
    sharp = cv2.addWeighted(den, 1.4, blur, -0.4, 0)

    # alpha：Lanczos 放大后二值化（>=128 → 255），防半透明脏边
    a_out = np.where(a_big >= 128, 255, 0).astype(np.uint8)

    out = np.dstack([sharp, a_out]).astype(np.uint8)
    return out


def main() -> None:
    assert CLASS.is_file(), f"缺 {CLASS}，先跑 scripts/classify_assets.py"
    cls_doc = json.loads(CLASS.read_text())
    entries = cls_doc["entries"]

    b_items = [(k, m) for k, m in sorted(entries.items()) if m.get("class") == "B_portrait"]
    print(f"[enhance_b] B 类 {len(b_items)} 张")

    manifest = json.loads(MANIFEST.read_text()) if MANIFEST.is_file() else {}
    stats = {"processed": 0, "failed": [], "format": "png"}
    for key, meta in b_items:
        container, name = meta["container"], meta["name"]
        pcx_path = RAW / container / name
        if not pcx_path.is_file():
            pcx_path = RAW / container / f"{key}.pcx"
        if not pcx_path.is_file():
            stats["failed"].append(f"{key}: 源缺失 {name}")
            continue
        try:
            w, h, px, pal = pcx_decode(pcx_path.read_bytes())
            rgba = to_rgba(w, h, px, pal)
            out_img = enhance_bilateral_unsharp(rgba, SCALE)
        except Exception as e:
            stats["failed"].append(f"{key}: {e}")
            continue
        base = re.sub(r"\.pcx$", "", name, flags=re.IGNORECASE)
        out = HD / container / f"{base}_hd_enhanced.png"
        out.parent.mkdir(parents=True, exist_ok=True)
        # OpenCV 写 PNG：BGR(A) 顺序，把 RGB(A) 转 BGR(A)
        out_bgra = cv2.cvtColor(out_img, cv2.COLOR_RGBA2BGRA)
        ok, buf = cv2.imencode(".png", out_bgra)
        assert ok, f"编码失败 {out}"
        out.write_bytes(buf.tobytes())
        data = out.read_bytes()
        ow, oh = out_img.shape[1], out_img.shape[0]
        manifest[f"{container}/{base}_hd_enhanced.png"] = {
            "class": "B_portrait", "container": container, "name": f"{base}_enhanced",
            "width": ow, "height": oh, "fnv1a64": fnv1a64(data),
            "method": "opencv_lanczos4_bilateral_unsharp_2x",
            "base": f"{base}_hd.png",
        }
        stats["processed"] += 1
        print(f"  {name} {w}x{h} → {ow}x{oh}  fnv={manifest[f'{container}/{base}_hd_enhanced.png']['fnv1a64'][:8]}")

    HD.mkdir(exist_ok=True)
    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True))
    stats["manifest"] = str(MANIFEST.relative_to(REPO))
    old = json.loads(STATS.read_text()) if STATS.is_file() else {}
    (STATS).write_text(json.dumps({**old, **stats}, ensure_ascii=False, indent=2))
    print(f"[enhance_b] 处理 {stats['processed']} 张，失败 {len(stats['failed'])}")
    for f in stats["failed"]:
        print(f"  FAIL {f}")
    assert not stats["failed"], "存在失败项"


if __name__ == "__main__":
    main()
