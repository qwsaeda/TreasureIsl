#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Agnes Image 2.5 Flash 生图脚本（docs/process/04_tooling.md §Agnes 生图）。

用法:
  python scripts/agnes_gen.py --prompt "..." --out evidence/ui_concept/index_v3.png
  python scripts/agnes_gen.py --prompt "..." --out evidence/ui_concept/index_v3.png \
      --image https://example.com/ref.png --image https://example.com/ref2.png
  python scripts/agnes_gen.py --prompt "..." --out mockup.png --size 2K --ratio 9:16

环境变量:
  AGNES_API_KEY  — Agnes API 密钥（必填；不写入脚本/不入库）

模型: agnes-image-2.5-flash（当前全部输出分辨率免费）
API:  https://apihub.agnes-ai.com/v1/images/generations
"""
from __future__ import annotations

import argparse
import base64
import json
import os
import sys
import urllib.request
from pathlib import Path

API_ENDPOINT = "https://apihub.agnes-ai.com/v1/images/generations"
MODEL = "agnes-image-2.5-flash"


def generate(
    prompt: str,
    out_path: Path,
    api_key: str,
    size: str = "2K",
    ratio: str = "9:16",
    ref_images: list[str] | None = None,
) -> Path:
    """调用 Agnes API 生图并保存到 out_path。

    Args:
        prompt: 生图提示词。
        out_path: 输出文件路径（.png/.jpg）。
        api_key: Agnes API 密钥。
        size: 尺寸档位（1K/2K/3K/4K）。
        ratio: 宽高比（1:1/3:4/4:3/16:9/9:16/2:3/3:2/21:9）。
        ref_images: 图生图输入图像 URL 或 Data URI 列表。

    Returns:
        输出文件路径。
    """
    body: dict = {
        "model": MODEL,
        "prompt": prompt,
        "size": size,
        "ratio": ratio,
        "extra_body": {"response_format": "url"},
    }
    if ref_images:
        body["extra_body"]["image"] = ref_images

    data = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        API_ENDPOINT,
        data=data,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )

    print(f"[agnes] 生图: size={size} ratio={ratio} prompt={prompt[:60]}...")
    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            result = json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        print(f"[agnes] API 请求失败: {e}", file=sys.stderr)
        sys.exit(1)

    url = result.get("data", [{}])[0].get("url")
    if not url:
        print(f"[agnes] API 未返回图像 URL: {result}", file=sys.stderr)
        sys.exit(1)

    # 下载图像
    try:
        with urllib.request.urlopen(url, timeout=120) as resp:
            img_bytes = resp.read()
    except Exception as e:
        print(f"[agnes] 下载失败: {e}", file=sys.stderr)
        sys.exit(1)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(img_bytes)
    print(f"[agnes] 已保存: {out_path} ({len(img_bytes) // 1024}KB)")
    return out_path


def generate_base64(
    prompt: str,
    api_key: str,
    size: str = "2K",
    ratio: str = "9:16",
) -> tuple[str, str | None]:
    """文生图 Base64 输出（data[0].b64_json）。

    Returns:
        (base64 图像数据, 临时 URL 或 None)。
    """
    body: dict = {
        "model": MODEL,
        "prompt": prompt,
        "size": size,
        "ratio": ratio,
        "return_base64": True,
    }
    data = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        API_ENDPOINT,
        data=data,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=300) as resp:
        result = json.loads(resp.read().decode("utf-8"))
    item = result.get("data", [{}])[0]
    return item.get("b64_json", ""), item.get("url")


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Agnes Image 2.5 Flash 生图（docs/process/04_tooling.md §Agnes 生图）",
    )
    ap.add_argument("--prompt", required=True, help="生图提示词")
    ap.add_argument("--out", required=True, help="输出文件路径")
    ap.add_argument("--size", default="2K", choices=["1K", "2K", "3K", "4K"], help="尺寸档位")
    ap.add_argument(
        "--ratio",
        default="9:16",
        choices=["1:1", "3:4", "4:3", "16:9", "9:16", "2:3", "3:2", "21:9"],
        help="宽高比",
    )
    ap.add_argument(
        "--image",
        action="append",
        dest="ref_images",
        default=[],
        help="图生图输入图像 URL（可多次指定）",
    )
    ap.add_argument(
        "--check",
        action="store_true",
        help="只读前置体检（S21.8 工具契约）：AGNES_API_KEY 在位 + 输出目录在位；不生图 / 不写文件，前置缺失 ⇒ 非 0 退出",
    )
    args = ap.parse_args()

    api_key = os.environ.get("AGNES_API_KEY", "")
    out_path = Path(args.out)
    if args.check:
        ok = True
        if not api_key:
            print("[agnes --check] FAIL 缺少环境变量 AGNES_API_KEY", file=sys.stderr)
            ok = False
        if not out_path.parent.exists():
            print(f"[agnes --check] FAIL 输出目录不存在: {out_path.parent}", file=sys.stderr)
            ok = False
        if ok:
            print("[agnes --check] OK（AGNES_API_KEY 在位 + 输出目录在位；只读未生图 / 未写文件）")
        return 0 if ok else 1
    if not api_key:
        print("[agnes] 缺少环境变量 AGNES_API_KEY", file=sys.stderr)
        return 1

    generate(
        prompt=args.prompt,
        out_path=out_path,
        api_key=api_key,
        size=args.size,
        ratio=args.ratio,
        ref_images=args.ref_images if args.ref_images else None,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
