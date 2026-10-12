# S7.2b：B 类角色立绘 waifu2x 超分（CPU 可跑，动漫模型）
# 用法：python scripts\waifu2x_upscale.py [--scale 2] [--model cunet]
#
# 读 ref/asset_classification.json 中 class == B_portrait 的条目，
# 从 assets_hd/<容器>/<name>_hd.png（S7.2 已 NEAREST 2x 兜底）取原图，
# 调 waifu2x-ncnn-vulkan 可执行做 AI 超分，覆盖回 _hd.png。
#
# 前置：下载 waifu2x-ncnn-vulkan（CPU 或 Vulkan 版），可执行名 waifu2x-ncnn-vulkan(.bat)，
# 放到 scripts/waifu2x/ 目录。无此工具时脚本打印指引并以 0 退出（B 类保持 NEAREST 兜底产物）。
#
# 零新增依赖（标准库 subprocess + json + pathlib）。

import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
W2X_DIR = REPO / "scripts" / "waifu2x"
CLASS = REPO / "ref" / "asset_classification.json"
HD = REPO / "assets_hd"

MODELS = {
    "cunet": "cunet",        # 去噪 0（轻，保细节，推荐）
    "cunet2": "cunet2",      # 去噪 1（中）
    "cunet3": "cunet3",      # 去噪 2（重）
    "cunet4": "cunet4",      # 去噪 3（最强）
}


def find_waifu2x() -> Path | None:
    """找 waifu2x-ncnn-vulkan 可执行（Windows .bat 或 Unix 脚本）。"""
    if not W2X_DIR.is_dir():
        return None
    for p in sorted(W2X_DIR.rglob("*")):
        if p.is_file() and p.name.lower().startswith("waifu2x-ncnn-vulkan"):
            return p
    return None


def pick_waifu2x_bin() -> str:
    """挑 CPU / Vulkan 可执行（本机 Windows 无独显 → 优先 CPU 版 _vulkan 不可用时用默认）。"""
    exe = find_waifu2x()
    if exe is None:
        return ""
    return str(exe)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--scale", type=int, default=2, choices=[1, 2, 4])
    ap.add_argument("--model", default="cunet", choices=list(MODELS))
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    w2x = pick_waifu2x_bin()
    if not w2x:
        print(
            "[waifu2x_upscale] 未找到 waifu2x-ncnn-vulkan 可执行。\n"
            f"[waifu2x_upscale] 请下载 CPU 或 Vulkan 版到 {W2X_DIR}（github xdarklen/waifu2x-ncnn-vulkan releases），\n"
            "[waifu2x_upscale] 再跑本脚本。当前 B 类保持 S7.2 的 NEAREST 2x 兜底产物，不阻塞 S7.4。"
        )
        return 0

    cls = json.loads(CLASS.read_text())
    entries = {k: v for k, v in cls["entries"].items() if v["class"] == "B_portrait"}
    n = 0
    for key, meta in sorted(entries.items()):
        container = meta["container"]
        name = meta["name"]
        base = re.sub(r"\.pcx$", "", name, flags=re.IGNORECASE)
        # S7.2 已落的 NEAREST 2x 产物（作为 waifu2x 输入的底图）
        src = HD / container / f"{base}_hd.png"
        if not src.is_file():
            print(f"[waifu2x_upscale] SKIP {src}（无 S7.2 兜底产物）")
            continue
        out = src.with_name(f"{base}_hd_w2x.png")
        cmd = [
            w2x,
            "-i",
            str(src),
            "-o",
            str(out),
            "-n",
            MODELS[args.model],
            "-s",
            str(args.scale),
            "-f",
            "png",
            "-t",
            "cpu",  # 本机无独显 → 强制 CPU（有 GPU 可改 vulkan）
        ]
        if args.dry_run:
            print(" ".join(cmd))
            continue
        r = subprocess.run(cmd, capture_output=True, text=True)
        if r.returncode != 0:
            print(f"[waifu2x_upscale] FAIL {base}: {r.stderr.strip()[:200]}")
            continue
        # 成功后覆盖兜底产物（保持 _hd.png 命名，S7.4 打包统一引用）
        if out.is_file():
            shutil.move(str(out), str(src))
            n += 1
    print(f"[waifu2x_upscale] waifu2x 超分 {n}/{len(entries)} 张 B 类立绘（model={args.model}, scale={args.scale}x）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
