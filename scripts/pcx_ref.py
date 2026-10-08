# -*- coding: utf-8 -*-
"""PCX 参考解码器（S1.3 独立实现，用于交叉验证 Rust 解码器 P0-4）。

不依赖任何第三方库；输出纯 ASCII：`WIDTH HEIGHT FNV1a64(pixels)`。
用法：python scripts/pcx_ref.py <file.pcx>
"""
import struct
import sys


def fnv1a64(data):
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h


def decode(data):
    assert len(data) >= 128, "PCX 头不足 128 字节"
    manufacturer = data[0]
    version = data[1]
    encoding = data[2]
    bpp = data[3]
    x_min, y_min, x_max, y_max = struct.unpack_from("<4H", data, 4)
    assert manufacturer == 0x0A, f"manufacturer={manufacturer:#x}"
    assert version == 5, f"version={version}"
    assert encoding == 1, f"encoding={encoding} (仅支持 RLE)"
    assert bpp == 8, f"bpp={bpp} (仅支持 8bpp)"

    width = x_max - x_min + 1
    height = y_max - y_min + 1

    # 行跨度不靠字段猜：用「RLE 恰好消费到文件尾」这一强约束在候选值中反解
    def rle_decode(need):
        out = bytearray()
        i = 128
        while len(out) < need and i < len(data):
            b = data[i]
            i += 1
            if b & 0xC0 == 0xC0:
                count = b & 0x3F
                if count == 0 or i >= len(data):
                    return None, i
                out.extend([data[i]] * count)
                i += 1
            else:
                out.append(b)
        return out, i

    cands = sorted(set(c for c in [width, struct.unpack_from("<H", data, 0x3E)[0], data[0x42]] if c >= width))
    raw = None
    stride = width
    for cand in cands:
        out, consumed = rle_decode(cand * height)
        if out is not None and len(out) == cand * height and consumed == len(data):
            raw, stride = out, cand
            break
    if raw is None:
        out, _ = rle_decode(width * height)
        raw, stride = out, width

    # 每行截断到 width
    rows = []
    for y in range(height):
        row = out[y * stride : y * stride + width]
        if len(row) < width:
            row = row + b"\x00" * (width - len(row))
        rows.append(bytes(row))
    pixels = b"".join(rows)
    return width, height, pixels


def main():
    path = sys.argv[1]
    with open(path, "rb") as f:
        data = f.read()
    w, h, px = decode(data)
    print(f"{w} {h} {fnv1a64(px):016x}")


if __name__ == "__main__":
    main()