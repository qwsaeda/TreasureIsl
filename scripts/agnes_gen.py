#!/usr/bin/env python3
"""Agnes Image 2.5 Flash 出图脚本（POST /v1/images/generations）。

用法：
    python scripts/agnes_gen.py --prompt "..." --out path.png [--size 512x512] [--quality low]

env AGNES_API_KEY 提供密钥（不落盘、不打印）。
仅调 Agnes 官方文档列的模型（agnes-image-25-flash / agnes-image-25），零第三方依赖（urllib）。
"""

import argparse
import base64
import json
import os
import sys
import urllib.request


def call_agnes(prompt: str, size: str, n: int, api_key: str) -> bytes:
    """调 Agnes 生成接口（model=agnes-image-2.5-flash，size 档位 2K，response_format 走 extra_body），
    返回 PNG 字节（url 下载 或 b64 解码）。"""
    url = "https://apihub.agnes-ai.com/v1/images/generations"
    payload = {
        "model": "agnes-image-2.5-flash",
        "prompt": prompt,
        "size": size,
        "ratio": "9:16",
        "extra_body": {"response_format": "url"},
    }
    body = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {api_key}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=180) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except Exception as e:  # noqa: BLE001
        try:
            body_err = e.read().decode()
        except Exception:
            body_err = str(e)
        print(f"[agnes] HTTP/网络错误: {e}\n响应: {body_err[:300]}", file=sys.stderr)
        return b""
    # 响应：data[0].b64_json（b64_json）或 data[0].url（url）
    item = data.get("data", [{}])[0]
    b64 = item.get("b64_json")
    if b64:
        return base64.b64decode(b64)
    url = item.get("url")
    if url:
        with urllib.request.urlopen(url, timeout=60) as r2:
            return r2.read()
    print(f"[agnes] 响应无 b64_json/url: {json.dumps(data)[:300]}", file=sys.stderr)
    return b""


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--prompt", required=True, help="界面描述（中英皆可，建议英文更稳）")
    ap.add_argument("--out", required=True, help="输出 PNG 路径")
    ap.add_argument("--size", default="2K", help="尺寸档位（1K/2K/4K 等，Agnes 侧接受）")
    ap.add_argument("--ratio", default="9:16", help="宽高比（默认 9:16 竖屏移动端）")
    ap.add_argument("--n", type=int, default=1)
    args = ap.parse_args()

    api_key = os.environ.get("AGNES_API_KEY")
    if not api_key:
        print("[agnes] 缺 AGNES_API_KEY 环境变量", file=sys.stderr)
        sys.exit(1)

    for i in range(args.n):
        png = call_agnes(args.prompt, args.size, args.n, api_key)
        if not png:
            print(f"[agnes] 第 {i+1} 张生成失败", file=sys.stderr)
            sys.exit(1)
        suffix = "" if args.n == 1 else f"_{i}"
        out = args.out if args.n == 1 else args.out.replace(".png", f"{suffix}.png")
        with open(out, "wb") as f:
            f.write(png)
        print(f"[agnes] 已生成 {out}（{len(png)} bytes，{args.size}）")


if __name__ == "__main__":
    main()
