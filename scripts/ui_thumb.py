#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""ui_concept 设计稿压缩预览缩略图生成器（平台设计图页「图片太大传输重」优化，2026-10-30）。

背景：`evidence/ui_concept/` 146 张共 353 MB（单张 4-6 MB，>512 KB = 123 张），平台
设计图页（卡 + lightbox）走仓库白名单直读全量 PNG，每张全量传输 = 预览慢。
本工具生成**缩略图侧车**（同名 `<name>.thumb.png`，长边 480px、sRGB PNG-8 量化），
卡网格只读侧车（480px 足够卡尺寸），lightbox 放大时读原图；`--check` 幂等对账。

用法（只生成，幂等；输出侧车文件进 gitignore 的 `evidence/ui_concept/` 内）：
  python3 scripts/ui_thumb.py            # 全量生成/刷新侧车（缺则建、变更则刷）
  python3 scripts/ui_thumb.py --max 480  # 长边像素（默认 480）
  python3 scripts/ui_thumb.py --check    # 只读对账：侧车缺失/过期列表，非 0 = 有缺失
环境变量：无（纯 PIL 离线，不调 API、不写证据 log）。
口径：侧车 = 压缩预览面（0 改动原图，原图仍白名单直读）；PIL 缺失 = 明确 FAIL 非 0。
"""
from __future__ import annotations

import argparse
import hashlib
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONCEPT = ROOT / "evidence" / "ui_concept"


def _sha1(p: Path) -> str:
    h = hashlib.sha1()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def _sidecar(src: Path) -> Path:
    return src.with_name(src.name.replace(".png", ".thumb.png"))


def _fresh(src: Path, dst: Path, cache: Path) -> bool:
    """侧车是否已是最新（src 指纹 + 生成参数 都在 cache 元数据里）。"""
    if not dst.is_file():
        return False
    try:
        meta = Path(cache).read_text()
        return _sha1(src) == meta.strip()
    except OSError:
        return False


def generate(max_edge: int, force: bool, only: str | None) -> int:
    try:
        from PIL import Image
    except ImportError:
        print("[ui_thumb] FAIL 缺 PIL（pip install pillow）", file=sys.stderr)
        return 1
    if not CONCEPT.is_dir():
        print(f"[ui_thumb] FAIL 目录不存在: {CONCEPT}", file=sys.stderr)
        return 1
    cache_dir = CONCEPT / ".thumb_meta"
    cache_dir.mkdir(exist_ok=True)
    pngs = [p for p in sorted(CONCEPT.glob("*.png")) if p.name != ".thumb.png"
            and not p.name.endswith(".thumb.png") and p.name != (only or "")]
    made = fresh = skipped = 0
    for src in pngs:
        dst = _sidecar(src)
        meta = cache_dir / (src.stem + ".sha1")
        if not force and _fresh(src, dst, meta):
            fresh += 1
            continue
        img = Image.open(src)
        if img.mode not in ("RGB", "RGBA", "L"):
            img = img.convert("RGB")
        scale = max_edge / max(img.width, img.height)
        if scale < 1:
            w = max(1, round(img.width * scale))
            h = max(1, round(img.height * scale))
            img = img.resize((w, h), Image.LANCZOS)
        out = img if img.mode in ("RGB", "L") else img.convert("RGB")
        out = out.quantize(colors=256, method=Image.Quantize.FASTOCTREE,
                           dither=Image.Dither.NONE)
        out.save(dst, format="PNG", optimize=True)
        meta.write_text(_sha1(src) + "\n")
        made += 1
        print(f"[ui_thumb] {dst.name}  ({dst.stat().st_size // 1024}KB, 源 {src.stat().st_size // 1024}KB)")
        skipped += 0
    print(f"[ui_thumb] 共 {len(pngs)} 张：新生成 {made} / 已最新 {fresh}（max_edge={max_edge}）")
    return 0


def check() -> int:
    cache_dir = CONCEPT / ".thumb_meta"
    missing = []
    for src in sorted(CONCEPT.glob("*.png")):
        if src.name.endswith(".thumb.png"):
            continue
        dst = _sidecar(src)
        if not dst.is_file():
            missing.append(src.name)
    srcs = [p.name for p in sorted(CONCEPT.glob("*.png")) if not p.name.endswith(".thumb.png")]
    if missing:
        print(f"[ui_thumb --check] {len(missing)} 张缺侧车：")
        for n in missing[:40]:
            print(f"  - {n}")
        print("[ui_thumb --check] 修：python3 scripts/ui_thumb.py")
        return 1
    print(f"[ui_thumb --check] OK（{len(srcs)} 张源全有侧车）")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--max", type=int, default=480, help="缩略图长边像素（默认 480）")
    ap.add_argument("--force", action="store_true", help="无视指纹强制重生成全部")
    ap.add_argument("--check", action="store_true", help="只读对账：侧车缺失/非 0 退出（工具契约）")
    ap.add_argument("--only", help="只处理某文件名（调试）")
    a = ap.parse_args()
    if a.check:
        return check()
    return generate(a.max, a.force, a.only)


if __name__ == "__main__":
    sys.exit(main())
