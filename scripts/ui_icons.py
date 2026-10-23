#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""UI 图标白蒙版生成器（S21.14 图标图片化，④-04 全量落地）。

**为什么用 Python 而非 Rust**：生 14 张 PNG 是低频脚本活（生成一次即入库），
用仓库现成 PIL 管线最轻、不占编译时间；必须 Rust 的只有**运行时** `blit_image_tint`
跨层 tint 管线（`ti-render/ti-app/ti-engine`），已在渲染层实现。本工具只管「几何 → 白蒙版」。

几何单一事实源 = `assets/ui_batches/icon_v1.json`（14 枚图标的 48 原生盒 ink/carve 矩形，
抄自 `crates/ti-render/src/icons.rs`）。改 icons.rs 几何须同步本 spec 并重跑。

画法：48×48 光栅（ink = 白满 alpha，carve = 刻空透明）→ 最近邻放大到 512 → RGBA8 PNG。
运行时 `blit_image_tint` 按 `tint`（RGB565）multiply 上色；缺资产 = 代码矢量回退（crash-safe）。

用法（仓库根）：
    python scripts/ui_icons.py            # 生成/覆盖 14 张到 assets/ui/
    python scripts/ui_icons.py --check    # 只读体检：重渲染比对字节（幂等 0；漂移/缺失非 0）
退出码：0 = 无漂移（或已生成）；1 = 有漂移/缺失且未 --fix。
幂等：同 spec ⇒ 同字节；--check 连跑两次 = 第二次 0。
"""
from __future__ import annotations

import argparse
import json
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
SPEC = ROOT / "assets/ui_batches/icon_v1.json"
UI_DIR = ROOT / "assets/ui"
UI_ICONS_JSON = ROOT / "assets/data/ui_icons.json"


def render_mask(ink, carve, native, out_size) -> bytes:
    """48 原生盒 → out_size 最近邻白蒙版，返回 4×(out_size*out_size) RGBA8 字节流。"""
    native_px = [[(0, 0, 0, 0) for _ in range(native)] for _ in range(native)]
    for dx, dy, w, h in ink:
        for y in range(dy, min(native, dy + h)):
            for x in range(dx, min(native, dx + w)):
                native_px[y][x] = (255, 255, 255, 255)
    for dx, dy, w, h in carve:
        for y in range(dy, min(native, dy + h)):
            for x in range(dx, min(native, dx + w)):
                native_px[y][x] = (0, 0, 0, 0)  # 刻空（透明）
    buf = bytearray()
    for oy in range(out_size):
        sy = (oy * native) // out_size
        for ox in range(out_size):
            sx = (ox * native) // out_size
            buf.extend(native_px[sy][sx])  # RGBA8
    return bytes(buf)


def png_bytes(rgba: bytes, out_size: int) -> bytes:
    """RGBA8 → PNG 字节（PIL 确定性编码；幂等判据用）。"""
    import io

    from PIL import Image

    im = Image.frombytes("RGBA", (out_size, out_size), rgba)
    bio = io.BytesIO()
    im.save(bio, format="PNG")
    return bio.getvalue()


def write_png(rgba: bytes, out_size: int, path: pathlib.Path) -> None:
    path.write_bytes(png_bytes(rgba, out_size))


def load_spec() -> dict:
    return json.loads(SPEC.read_text(encoding="utf-8"))


def main() -> int:
    ap = argparse.ArgumentParser(description="UI 图标白蒙版生成器（S21.14）")
    ap.add_argument("--check", action="store_true", help="只读体检：重渲染比对 + 资产在位 + 注册表 image→文件（漂移/缺失非 0）")
    ap.add_argument("--spec", default=str(SPEC), help="几何 spec 路径")
    ap.add_argument("--ui-dir", default=str(UI_DIR), help="输出 assets/ui 目录")
    args = ap.parse_args()

    spec = json.loads(pathlib.Path(args.spec).read_text(encoding="utf-8"))
    native = int(spec.get("native", 48))
    out_size = int(spec.get("out_size", 512))
    ui_dir = pathlib.Path(args.ui_dir)

    fails = 0
    for ic in spec["icons"]:
        name = ic["name"]
        rgba = render_mask(ic.get("ink", []), ic.get("carve", []), native, out_size)
        path = ui_dir / f"icon_{name}.png"
        if args.check:
            expected = png_bytes(rgba, out_size)
            try:
                existing = path.read_bytes()
            except FileNotFoundError:
                print(f"MISSING {path.name}：白蒙版缺失（跑无 --check 生成）")
                fails += 1
                continue
            # 幂等：同 spec 重编码字节 == 在盘字节（PIL 确定性 PNG）
            if existing != expected:
                print(f"DRIFT {path.name}：在盘字节 ≠ spec 重渲染（几何 spec 与资产漂移，重跑生成）")
                fails += 1
        else:
            ui_dir.mkdir(parents=True, exist_ok=True)
            write_png(rgba, out_size, path)
            print(f"wrote {path.name}（{out_size}×{out_size} 白蒙版）")

    # 注册表 image → 文件 校验（--check 时）
    if args.check:
        reg = json.loads(UI_ICONS_JSON.read_text(encoding="utf-8"))
        for ic in reg.get("icons", []):
            img = ic.get("image")
            if img and not (ROOT / img).is_file():
                print(f"MISSING {img}：注册表 {ic.get('name')} 的 image 指向缺失资产")
                fails += 1
        if fails == 0:
            print(f"ui_icons --check: PASS（{len(spec['icons'])} 白蒙版 + 注册表一致）")
    return 1 if fails else 0


if __name__ == "__main__":
    raise SystemExit(main())
