#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Agnes Video 2.5 Flash 视频脚本（docs/process/04_tooling.md §Agnes 视频）。

用法:
  python scripts/agnes_video.py --prompt "..." --out evidence/tutorial/05_aabb_demo.mp4
  python scripts/agnes_video.py --prompt "..." --out demo.mp4 --seconds 8 --ratio 16:9
  python scripts/agnes_video.py --prompt "..." --out demo.mp4 --reference img1.png img2.png
  python scripts/agnes_video.py --check

环境变量:
  AGNES_API_KEY  — Agnes API 密钥（必填；不写脚本/文档中）

模型: agnes-video-2.5-flash（异步：创建任务 → 轮询 → 下载）
API:  https://apihub.agnes-ai.com/v1/videos
查询: https://apihub.agnes-ai.com/agnesapi?video_id=...&model_name=agnes-video-2.5-flash
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.request
from pathlib import Path

API_BASE = "https://apihub.agnes-ai.com/v1"
MODEL = "agnes-video-2.5-flash"
POLL_INTERVAL_S = 2
POLL_TIMEOUT_S = 600


def _post(url: str, body: dict, api_key: str, timeout: int = 120) -> dict:
    data = json.dumps(body).encode("utf-8")
    req = urllib.request.Request(
        url,
        data=data,
        headers={
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def _get(url: str, api_key: str, timeout: int = 60) -> dict:
    req = urllib.request.Request(
        url,
        headers={"Authorization": f"Bearer {api_key}"},
        method="GET",
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        return json.loads(resp.read().decode("utf-8"))


def _download(url: str, out_path: Path, timeout: int = 300) -> int:
    with urllib.request.urlopen(url, timeout=timeout) as resp:
        data = resp.read()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(data)
    return len(data)


def create_task(
    prompt: str,
    api_key: str,
    seconds: str = "5",
    aspect_ratio: str = "16:9",
    mode: str = "text",
    ref_images: list[str] | None = None,
) -> str:
    """创建视频任务，返回 video_id。"""
    body: dict = {
        "model": MODEL,
        "prompt": prompt,
        "seconds": seconds,
        "mode": mode,
        "size": "720P",
        "aspect_ratio": aspect_ratio,
        "n": 1,
    }
    if ref_images and mode == "reference":
        body["images"] = ref_images

    result = _post(f"{API_BASE}/videos", body, api_key)
    video_id = result.get("video_id") or result.get("id")
    if not video_id:
        print(f"[agnes_video] API 未返回 video_id: {result}", file=sys.stderr)
        sys.exit(1)
    print(f"[agnes_video] 任务已创建: video_id={video_id}")
    return video_id


def poll_and_download(
    video_id: str,
    api_key: str,
    out_path: Path,
    timeout_s: int = POLL_TIMEOUT_S,
) -> Path:
    """轮询任务直至完成，下载视频到 out_path。"""
    url = (
        f"https://apihub.agnes-ai.com/agnesapi"
        f"?video_id={video_id}&model_name={MODEL}"
    )
    t0 = time.time()
    while True:
        elapsed = time.time() - t0
        if elapsed > timeout_s:
            print(f"[agnes_video] 轮询超时（{timeout_s}s）", file=sys.stderr)
            sys.exit(1)
        try:
            result = _get(url, api_key)
        except Exception as e:
            print(f"[agnes_video] 轮询异常: {e}", file=sys.stderr)
            time.sleep(POLL_INTERVAL_S)
            continue
        status = result.get("status", "")
        progress = result.get("progress", 0)
        print(f"[agnes_video] status={status} progress={progress}% ({elapsed:.0f}s)")
        if status == "completed":
            video_url = result.get("url")
            if not video_url:
                print(f"[agnes_video] 完成但无 url: {result}", file=sys.stderr)
                sys.exit(1)
            size = _download(video_url, out_path)
            print(f"[agnes_video] 已保存: {out_path} ({size // 1024}KB)")
            return out_path
        if status == "failed":
            print(f"[agnes_video] 任务失败: {result}", file=sys.stderr)
            sys.exit(1)
        time.sleep(POLL_INTERVAL_S)


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Agnes Video 2.5 Flash 视频脚本（docs/process/04_tooling.md）"
    )
    ap.add_argument("--prompt", required=True, help="视频内容描述")
    ap.add_argument("--out", required=True, help="输出文件路径（.mp4）")
    ap.add_argument("--seconds", default="5", choices=["4", "5", "6", "7", "8", "9", "10", "11", "12"], help="视频时长（秒）")
    ap.add_argument("--ratio", default="16:9", choices=["21:9", "16:9", "4:3", "1:1", "3:4", "9:16"], help="画幅")
    ap.add_argument("--mode", default="text", choices=["text", "keyframe", "reference"], help="生成模式")
    ap.add_argument("--image", action="append", dest="ref_images", default=[], help="参考图片 URL（reference 模式，可多指）")
    ap.add_argument("--check", action="store_true", help="只检查前置条件（AGNES_API_KEY 在位 + 输出目录在位）；不创建任务 / 不写文件。前置缺失 ? 非 0 退出")
    ap.add_argument("--video-id", help="已有 video_id，直接轮询下载（跳过创建）")
    args = ap.parse_args()

    api_key = os.environ.get("AGNES_API_KEY", "")
    out_path = Path(args.out)

    if args.check:
        ok = True
        if not api_key:
            print("[agnes_video --check] FAIL 缺少环境变量 AGNES_API_KEY", file=sys.stderr)
            ok = False
        if not out_path.parent.exists():
            print(f"[agnes_video --check] FAIL 输出目录不存在: {out_path.parent}", file=sys.stderr)
            ok = False
        if ok:
            print("[agnes_video --check] OK（AGNES_API_KEY 在位 + 输出目录在位；未创建任务 / 未写文件）")
        return 0 if ok else 1

    if not api_key:
        print("[agnes_video] 缺少环境变量 AGNES_API_KEY", file=sys.stderr)
        return 1

    if args.video_id:
        video_id = args.video_id
        print(f"[agnes_video] 使用已有 video_id={video_id}")
    else:
        video_id = create_task(
            prompt=args.prompt,
            api_key=api_key,
            seconds=args.seconds,
            aspect_ratio=args.ratio,
            mode=args.mode,
            ref_images=args.ref_images if args.ref_images else None,
        )

    poll_and_download(video_id, api_key, out_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
