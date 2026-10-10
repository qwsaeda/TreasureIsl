#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""S7.2 批量超分/矢量化：assets_raw/*.pcx → assets_hd/（PNG-32）+ manifest_hd.json。

策略（docs/impl/14_asset_enhancement.md §3.2，按 ref/asset_classification.json 分类）：
  - C_pixel     最近邻 2x（保像素硬边）
  - D_terrain   Lanczos 2x（大画布平滑）
  - E_animation 5 帧纵向堆叠 → 逐帧 NEAREST 2x + 相邻帧中值去抖（无视频工具时的轻量时序方案）
  - B_portrait  最近邻 2x 兜底（Real-ESRGAN anime 超分可选增强，需外部工具，此处不做）
  - A_ui        九宫格切图 + 最近邻 2x（Inkscape 矢量化可选人工环节，此处不依赖）
  - Skip        不处理

PCX 解码：复用 scripts/pcx_ref.py 的 RLE 反解 + scripts/pack_hd_assets_shared 的调色板读取
（强约束「RLE 恰好消费到文件尾」，与 ti-formats::pcx 一致，F-205/EXP-019）。
索引→真彩：rgb = palette[idx]；背景色 = 调色板 slot 0 → alpha=0（原版 PCX 无 alpha 通道，
blit 透明由 .Map f4 例程模式决定，F-311；HD 管线把背景 slot 像素标透明）。

产物命名：assets_hd/<容器>/<display_name 去后缀>_hd.png
多帧：assets_hd/<容器>/<display_name>_hd_f0.png .. _f4.png（5 帧）
九宫格：assets_hd/<容器>/<display_name>_hd_9s.png + 切分参数写 manifest
manifest_hd.json：FNV-1a64 幂等（无时间戳），复用与 build_assets 一致的哈希（ti_tools::hash 同源）。

用法：python scripts/pixel_upscale.py
零第三方依赖（标准库 + 本机 Pillow 用于 PNG 写盘；若缺 Pillow 则落裸 RGBA PPM 兜底）。
"""
import hashlib
import json
import re
import struct
import sys
import zlib
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
RAW = REPO / "assets_raw"
HD = REPO / "assets_hd"
CLASS = REPO / "ref" / "asset_classification.json"
MANIFEST = REPO / "manifest_hd.json"

SCALE = 2  # 默认整数倍（C/D/E 类；4x 由 S7.4 打包环节生成）

# 各分类策略 → (放大方式, 是否切九宫格, 是否多帧时序)
STRATEGY = {
    "C_pixel": ("nearest", False, False),
    "D_terrain": ("lanczos", False, False),
    "E_animation": ("nearest", False, True),
    "B_portrait": ("nearest", False, False),
    "A_ui": ("nearest", True, False),
}


# ---------- FNV-1a 64（与 ti_tools::hash::fnv1a64 同源，build_assets 幂等基准） ----------
def fnv1a64(data: bytes) -> str:
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return format(h, "016x")


# ---------- PCX 解码（与 scripts/pcx_ref.py 的 RLE 逻辑一致 + 尾部调色板） ----------
def pcx_decode(data: bytes):
    """返回 (width, height, pixels(索引), palette(256×rgb))。"""
    assert len(data) >= 128, "PCX 头不足"
    assert data[0] == 0x0A and data[1] == 5 and data[2] == 1 and data[3] == 8, "仅 8bpp RLE"
    x_max, y_max = struct.unpack_from("<HH", data, 8)
    width = x_max + 1
    height = y_max + 1

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

    # 行跨度候选：width / 0x3E 字段 / 0x42 字节（F-205：头字段不可信，强约束反解）
    cands = sorted({width, struct.unpack_from("<H", data, 0x3E)[0], data[0x42]})
    stride = width
    raw = None
    for cand in cands:
        if cand < width:
            continue
        out = rle_decode(cand * height)
        if out is not None:
            raw, stride = out, cand
            break
    if raw is None:
        raise ValueError(f"PCX RLE 反解失败 {width}x{height}")

    rows = []
    for y in range(height):
        row = raw[y * stride : y * stride + width]
        rows.append(row)
    pixels = b"".join(bytes(r) for r in rows)

    # 尾部调色板（0x0C + 768B RGB），缺失走 EGA 默认
    palette = None
    if len(data) >= 769 and data[-769] == 0x0C:
        tail = data[-768:]
        palette = []
        for i in range(256):
            palette.append((tail[i * 3], tail[i * 3 + 1], tail[i * 3 + 2]))
    if palette is None:
        base = [
            (0, 0, 0), (0, 0, 170), (0, 170, 0), (0, 170, 170),
            (170, 0, 0), (170, 0, 170), (170, 85, 0), (170, 170, 170),
            (85, 85, 85), (85, 85, 255), (85, 255, 85), (85, 255, 255),
            (255, 85, 85), (255, 85, 255), (255, 255, 85), (255, 255, 255),
        ]
        palette = [
            base[i] if i < 16 else (i, i, i) for i in range(256)
        ]
    return width, height, pixels, palette


def idx_to_rgba(width: int, height: int, pixels: bytes, palette, bg_slot: int = 0):
    """索引像素 → RGBA8 行主序。bg_slot 标 alpha=0（透明背景）。"""
    bg = palette[bg_slot % 256]
    out = bytearray(width * height * 4)
    p = bytes(palette[bg_slot % 256])
    for i, idx in enumerate(pixels):
        r, g, b = palette[idx]
        a = 0 if idx == bg_slot else 255
        out[i * 4] = r
        out[i * 4 + 1] = g
        out[i * 4 + 2] = b
        out[i * 4 + 3] = a
    del p
    return bytes(out)


# ---------- 放大 ----------
def scale_nearest(rgba: bytes, w: int, h: int, s: int) -> tuple[bytes, int, int]:
    """最近邻 s 倍（保像素硬边）。"""
    out = bytearray((w * s) * (h * s) * 4)
    for y in range(h):
        for x in range(w):
            src = (y * w + x) * 4
            for dy in range(s):
                for dx in range(s):
                    dst = ((y * s + dy) * (w * s) + (x * s + dx)) * 4
                    out[dst : dst + 4] = rgba[src : src + 4]
    return bytes(out), w * s, h * s


def scale_lanczos(rgba: bytes, w: int, h: int, s: int) -> tuple[bytes, int, int]:
    """Lanczos 平滑放大（地形大画布）。有 Pillow 用 Pillow，否则退回最近邻 + 双线性。"""
    try:
        from PIL import Image  # 本机 12.2.0

        img = Image.frombytes("RGBA", (w, h), rgba)
        img = img.resize((w * s, h * s), Image.LANCZOS)
        return img.tobytes(), w * s, h * s
    except ImportError:
        # 双线性兜底（纯标准库）
        return scale_bilinear(rgba, w, h, s)


def scale_bilinear(rgba: bytes, w: int, h: int, s: int) -> tuple[bytes, int, int]:
    nw, nh = w * s, h * s
    out = bytearray(nw * nh * 4)
    for y in range(nh):
        fy = y / s
        y0 = int(fy)
        y1 = min(y0 + 1, h - 1)
        ty = fy - y0
        for x in range(nw):
            fx = x / s
            x0 = int(fx)
            x1 = min(x0 + 1, w - 1)
            tx = fx - x0
            for c in range(4):
                i = lambda yy, xx: (yy * w + xx) * 4 + c
                v00 = rgba[i(y0, x0)]
                v01 = rgba[i(y0, x1)]
                v10 = rgba[i(y1, x0)]
                v11 = rgba[i(y1, x1)]
                top = v00 + (v01 - v00) * tx
                bot = v10 + (v11 - v10) * tx
                v = top + (bot - top) * ty
                out[(y * nw + x) * 4 + c] = int(round(v))
    return bytes(out), nw, nh


def temporal_median_denoise(frames: list[bytes], w: int, h: int) -> list[bytes]:
    """E 类轻量时序去抖：中间帧像素取 3 帧（上/中/下）RGB 中值，alpha 不变。"""
    n = len(frames)
    out = list(frames)
    px = [frames[i] for i in range(n)]
    for fi in range(1, n - 1):
        prev, cur, nxt = px[fi - 1], px[fi], px[fi + 1]
        new = bytearray(n * 0 + w * h * 4)
        for p in range(w * h):
            for c in range(3):
                vals = sorted(
                    (prev[p * 4 + c], cur[p * 4 + c], nxt[p * 4 + c])
                )
                new[p * 4 + c] = vals[1]
            new[p * 4 + 3] = cur[p * 4 + 3]
        out[fi] = bytes(new)
    return out


# ---------- 九宫格 ----------
def nine_slice(rgba: bytes, w: int, h: int) -> tuple[bytes, int, int, dict]:
    """边缘 1/4 像素带切九宫格（按钮中心可拉伸）。返回 (中心块 RGBA, 中心 w, 中心 h, 切分参数)。"""
    bw = max(1, w // 4)
    bh = max(1, h // 4)
    cx0, cy0 = bw, bh
    cx1, cy1 = w - bw, h - bh
    out = bytearray((cx1 - cx0) * (cy1 - cy0) * 4)
    for y in range(cy0, cy1):
        for x in range(cx0, cx1):
            s = (y * w + x) * 4
            d = ((y - cy0) * (cx1 - cx0) + (x - cx0)) * 4
            out[d : d + 4] = rgba[s : s + 4]
    params = {"border_w": bw, "border_h": bh}
    return bytes(out), cx1 - cx0, cy1 - cy0, params


# ---------- PNG 写盘（有 Pillow 用 Pillow；否则落 PPM 兜底 + 记录） ----------
def write_image(rgba: bytes, w: int, h: int, path: Path) -> str:
    """写 PNG-32（或 PPM 兜底）。返回实际格式。"""
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        from PIL import Image

        img = Image.frombytes("RGBA", (w, h), rgba)
        img.save(path)
        return "png"
    except ImportError:
        fallback = path.with_suffix(".ppm")
        # PPM P6 写 RGB（丢弃 alpha，alpha 信息进 manifest）
        rgb = bytearray(w * h * 3)
        for i in range(w * h):
            rgb[i * 3 : i * 3 + 3] = rgba[i * 4 : i * 4 + 3]
        header = f"P6\n{w} {h}\n255\n".encode()
        fallback.write_bytes(header + bytes(rgb))
        path.write_bytes(b"")  # 占位（Pillow 缺失时 PNG 不可写）
        return "ppm"


# ---------- manifest 构造 ----------
def main() -> None:
    assert CLASS.is_file(), f"缺 {CLASS}，先跑 scripts/classify_assets.py"
    cls_doc = json.loads(CLASS.read_text())
    entries = cls_doc["entries"]

    manifest: dict[str, dict] = {}
    stats = {"processed": 0, "skipped": 0, "failed": [], "format": "png"}
    for key, meta in sorted(entries.items()):
        c = meta["class"]
        if c == "Skip":
            stats["skipped"] += 1
            continue
        strat = STRATEGY.get(c)
        if strat is None:
            stats["skipped"] += 1
            continue
        method, do_9s, do_temporal = strat
        container = meta["container"]
        name = meta["name"]
        raw_dir = RAW / container
        # 找 PCX 文件（display_name 可能是 .bin 后缀的未命名桶，但 Skip 已排除；
        # glpics unknown-*.bin 也是 Skip，不会到这里）
        pcx_path = raw_dir / name
        if not pcx_path.is_file():
            # display_name 与落盘文件名不一致时按 key 兜底
            pcx_path = raw_dir / f"{key}.pcx"
        if not pcx_path.is_file():
            stats["failed"].append(f"{key}: 源文件缺失 {name}")
            continue
        data = pcx_path.read_bytes()
        try:
            w, h, pixels, palette = pcx_decode(data)
        except Exception as e:
            stats["failed"].append(f"{key}: PCX 解码失败 {e}")
            continue

        base = re.sub(r"\.pcx$", "", name, flags=re.IGNORECASE)
        stem = HD / container / base

        # 多帧（E 类）：纵向切帧 → 逐帧 NEAREST 2x + 中值去抖
        if do_temporal:
            nframes = h // 40 if h % 40 == 0 else max(1, h // SCALE)
            # S6.4 实测 4 精灵高 199 = 5 帧 × 39.8 → 取整 40（实现截断多 1 行）
            nframes = 5
            fh = h // nframes
            frames_rgba = []
            for fi in range(nframes):
                sub = pixels[fi * fh * w : (fi + 1) * fh * w]
                frames_rgba.append(idx_to_rgba(w, fh, sub, palette))
            frames_rgba = temporal_median_denoise(frames_rgba, w, fh)
            for fi, fr in enumerate(frames_rgba):
                srgba, sw, sh = scale_nearest(fr, w, fh, SCALE)
                out = stem.with_name(f"{base}_hd_f{fi}.png")
                fmt = write_image(srgba, sw, sh, out)
                stats["format"] = fmt if fmt == "png" else "mixed"
                rel = out.relative_to(REPO).as_posix()
                # FNV 对落盘字节（PNG 压缩态）计算，与 Rust 侧 build_hd_assets 校验同源
                manifest[rel] = {
                    "class": c, "container": container, "name": f"{base}_f{fi}",
                    "width": sw, "height": sh, "fnv1a64": fnv1a64(out.read_bytes()),
                }
                stats["processed"] += 1
            continue

        rgba = idx_to_rgba(w, h, pixels, palette)
        if method == "nearest":
            srgba, sw, sh = scale_nearest(rgba, w, h, SCALE)
        else:
            srgba, sw, sh = scale_lanczos(rgba, w, h, SCALE)
        out = stem.with_name(f"{base}_hd.png")
        fmt = write_image(srgba, sw, sh, out)
        stats["format"] = fmt if fmt == "png" else "mixed"
        rel = out.relative_to(REPO).as_posix()
        manifest[rel] = {
            "class": c, "container": container, "name": base,
            "width": sw, "height": sh, "fnv1a64": fnv1a64(out.read_bytes()),
        }
        stats["processed"] += 1

        if do_9s:
            # 九宫格：中心块（边缘 1/4 像素带）放大 SCALE 倍后写盘，切分参数进 manifest
            center, cw, ch, params = nine_slice(rgba, w, h)
            scenter, scw, sch = scale_nearest(center, cw, ch, SCALE)
            cpath = stem.with_name(f"{base}_hd_9s.png")
            cfmt = write_image(scenter, scw, sch, cpath)
            stats["format"] = cfmt if cfmt == "png" else "mixed"
            crel = cpath.relative_to(REPO).as_posix()
            manifest[crel] = {
                "class": c, "container": container, "name": f"{base}_9s",
                "nine_slice": {
                    "border_w": params["border_w"] * SCALE,
                    "border_h": params["border_h"] * SCALE,
                    "fnv1a64": fnv1a64(cpath.read_bytes()),
                },
                "width": scw, "height": sch,
            }
            stats["processed"] += 1

    HD.mkdir(exist_ok=True)
    MANIFEST.write_text(json.dumps(manifest, ensure_ascii=False, indent=2, sort_keys=True))
    print(f"[pixel_upscale] 处理 {stats['processed']} 条，跳过 {stats['skipped']}，失败 {len(stats['failed'])}")
    print(f"[pixel_upscale] 产物格式: {stats['format']}（无 Pillow 时 PPM 兜底）")
    if stats["failed"]:
        for f in stats["failed"][:20]:
            print(f"[pixel_upscale]   FAIL {f}")
    print(f"[pixel_upscale] manifest → {MANIFEST}")
    # 落盘一份统计供 evidence 引用
    (HD / "_stats.json").write_text(json.dumps(stats, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
