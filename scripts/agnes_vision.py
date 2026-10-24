#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Agnes 多模态看图脚本（Agnes 2.5 Flash，OpenAI 兼容 Chat Completions）。

与 `scripts/agnes_gen.py`（生图）同一 KEY（环境变量 `AGNES_API_KEY`），
但走 `POST /v1/chat/completions` + `image_url`（本地图片转 Data URI）。

用法:
  python scripts/agnes_vision.py --image a.png --image b.png --prompt "..."
  python scripts/agnes_vision.py --image a.png --prompt "..." --out evidence/x.md
  python scripts/agnes_vision.py --image-dir assets_hd/glpics --glob "Landmark_*" --prompt "..."

模型: agnes-2.5-flash（多模态理解）
API:  https://apihub.agnes-ai.com/v1/chat/completions
"""
from __future__ import annotations

import argparse
import base64
import json
import mimetypes
import os
import sys
import urllib.request
from pathlib import Path

API_ENDPOINT = "https://apihub.agnes-ai.com/v1/chat/completions"
DEFAULT_MODEL = "agnes-2.5-flash"


def to_data_uri(path: Path) -> str:
    """本地图片 → Data URI（image_url 兼容 base64 data URI）。"""
    mime = mimetypes.guess_type(path.name)[0] or "image/png"
    b64 = base64.b64encode(path.read_bytes()).decode("ascii")
    return f"data:{mime};base64,{b64}"


def analyze(
    prompt: str,
    images: list[Path],
    api_key: str,
    model: str = DEFAULT_MODEL,
    max_tokens: int = 2048,
    temperature: float = 0.2,
) -> str:
    """送图 + 提示词，返回模型文本回答。"""
    content: list[dict] = [{"type": "text", "text": prompt}]
    for img in images:
        content.append({"type": "image_url", "image_url": {"url": to_data_uri(img)}})
    body = {
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "max_tokens": max_tokens,
        "temperature": temperature,
    }
    req = urllib.request.Request(
        API_ENDPOINT,
        data=json.dumps(body).encode("utf-8"),
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            result = json.loads(resp.read().decode("utf-8"))
    except Exception as e:  # noqa: BLE001 — 脚本层统一报错退出
        print(f"[agnes-vision] API 请求失败: {e}", file=sys.stderr)
        sys.exit(1)
    try:
        return result["choices"][0]["message"]["content"]
    except (KeyError, IndexError, TypeError):
        print(f"[agnes-vision] 返回格式异常: {json.dumps(result)[:500]}", file=sys.stderr)
        sys.exit(1)


def main() -> int:
    ap = argparse.ArgumentParser(description="Agnes 多模态看图（agnes-2.5-flash）")
    ap.add_argument("--image", action="append", default=[], help="图片路径（可多次）")
    ap.add_argument("--image-dir", default=None, help="目录（配合 --glob 批量）")
    ap.add_argument("--glob", default="*", help="目录下的 glob 过滤（默认 *）")
    ap.add_argument("--prompt", required=True, help="提问/分析要求")
    ap.add_argument("--model", default=DEFAULT_MODEL)
    ap.add_argument("--max-tokens", type=int, default=2048)
    ap.add_argument("--out", default=None, help="结果写入文件（可选）")
    ap.add_argument(
        "--check",
        action="store_true",
        help="只读前置体检（S21.8 工具契约）：AGNES_API_KEY 在位 + 输入图片在位；不调 API / 不写文件，前置缺失 ⇒ 非 0 退出",
    )
    args = ap.parse_args()

    api_key = os.environ.get("AGNES_API_KEY", "")
    images = [Path(p) for p in args.image]
    if args.image_dir:
        images.extend(sorted(Path(args.image_dir).glob(args.glob)))

    if args.check:
        ok = True
        if not api_key:
            print("[agnes-vision --check] FAIL 缺少环境变量 AGNES_API_KEY", file=sys.stderr)
            ok = False
        if not images:
            print("[agnes-vision --check] FAIL 没有待分析图片（--image / --image-dir）", file=sys.stderr)
            ok = False
        missing = [str(p) for p in images if not p.exists()]
        if missing:
            print(f"[agnes-vision --check] FAIL 图片不存在: {missing}", file=sys.stderr)
            ok = False
        if ok:
            print(f"[agnes-vision --check] OK（AGNES_API_KEY 在位 + {len(images)} 张图在位；只读未调 API / 未写文件）")
        return 0 if ok else 1

    if not api_key:
        print("[agnes-vision] 缺少环境变量 AGNES_API_KEY", file=sys.stderr)
        return 1

    if not images:
        print("[agnes-vision] 没有待分析图片", file=sys.stderr)
        return 1
    missing = [str(p) for p in images if not p.exists()]
    if missing:
        print(f"[agnes-vision] 图片不存在: {missing}", file=sys.stderr)
        return 1

    print(f"[agnes-vision] 分析 {len(images)} 张图: {[p.name for p in images]}")
    answer = analyze(
        args.prompt,
        images,
        api_key,
        model=args.model,
        max_tokens=args.max_tokens,
    )
    if args.out:
        out = Path(args.out)
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(answer, encoding="utf-8")
        print(f"[agnes-vision] 已保存: {out}")
    else:
        print(answer)
    return 0


if __name__ == "__main__":
    sys.exit(main())
