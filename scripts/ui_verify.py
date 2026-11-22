#!/usr/bin/env python3
"""ui_verify.py — 设备矩阵出图验收（S67.0，LGUI verify 借鉴）。

职责：一条命令对「场景 × 设备视口」矩阵批量出图，逐图做非黑像素 + bevy_ui
探针断言，汇总 JSON 证据；空帧 / 探针缺失 / 命令失败 => 非 0 退出。
只读模式 `--check` 只校验既有证据（不重新出图、不写盘）。

出图复用 ti-engine 的 `--shot --scene=<id> --viewport=WxH`，产物独立命名
（`TI_SHOT_NAME=ui_verify_<scene>`）以免覆盖其它验收截图。
安全区叠加（S67.1）由 `--safe-area` 读 `ui_schema/safe_area.json` 后画参考线，
默认配置全 0 = 不叠加 = 零视觉变化。
"""

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
EVIDENCE = REPO / "evidence"
REPORT = EVIDENCE / "ui_verify_report.json"
SAFE_AREA = REPO / "assets" / "data" / "ui_schema" / "safe_area.json"

# LGUI verify 口径的设备矩阵（横竖各档；本项目以横屏为主，竖屏档仍可出图）。
DEVICES = {
    "desktop": (1920, 1080),
    "ultrawide": (2560, 1080),
    "phone_landscape": (844, 390),
    "tablet": (1024, 768),
    "phone_portrait": (390, 844),
    "tablet_portrait": (768, 1024),
}
QUICK_DEVICES = ["desktop", "phone_landscape", "tablet"]
DEFAULT_DEVICES = ["desktop", "phone_landscape"]  # 默认 2 档快速矩阵（大视口 PNG 编码重）
DEFAULT_SCENES = ["index"]
PROBE_MARK = "[ui] bevy_ui probe:"
SIZE_MARK = "max_node="


def cargo_bin():
    """解析 cargo 可执行文件（PATH / rustup 默认位）。"""
    found = shutil.which("cargo")
    if found:
        return found
    fallback = Path.home() / ".cargo" / "bin" / "cargo"
    if fallback.is_file():
        return str(fallback)
    return "cargo"


def shot_command(scene, width, height, spin):
    """组装单张出图命令（逐参数、非 shell，遵循工具契约）。"""
    command = [
        cargo_bin(), "run", "-q", "-p", "ti-engine",
        "--bin", "treasure-bevy-native", "--",
        f"--scene={scene}", "--shot", f"--viewport={width}x{height}",
    ]
    if spin:
        command.append(f"--spin={spin}")
    if sys.platform.startswith("linux") and not os.environ.get("DISPLAY"):
        if shutil.which("xvfb-run"):
            command = ["xvfb-run", "-a"] + command
    return command


def shot_path(scene, width, height):
    """出图落盘路径（与 TI_SHOT_NAME 命名口径一致）。"""
    return EVIDENCE / f"ui_verify_{scene}_{width}x{height}.png"


def nonblack_count(path):
    """非黑像素数（任一通道 > 8 即算非黑；与引擎 ui_preview 同口径）。"""
    try:
        from PIL import Image
    except ImportError:
        return -1
    with Image.open(path) as image:
        data = image.convert("RGB").tobytes()
    return sum(1 for offset in range(0, len(data), 3)
               if data[offset] > 8 or data[offset + 1] > 8 or data[offset + 2] > 8)


def probe_present(stderr):
    """bevy_ui 探针是否在位（node/computed_layout + 布局尺寸两行）。"""
    return PROBE_MARK in stderr and SIZE_MARK in stderr


def load_safe_area(path):
    """读安全区配置；默认全 0（文件缺失 = 零 inset，不视为错误）。"""
    if not path.is_file():
        return {"landscape": [0, 0, 0, 0], "portrait": [0, 0, 0, 0]}
    with open(path, "r", encoding="utf-8") as handle:
        data = json.load(handle)
    for orientation in ("landscape", "portrait"):
        if orientation not in data:
            raise ValueError(f"safe_area.{orientation} 缺字段（上右下左四元组）")
        values = data[orientation]
        if len(values) != 4 or any((not isinstance(item, int)) or item < 0 for item in values):
            raise ValueError(f"safe_area.{orientation} 须为 4 个非负数（上右下左）")
    return data


def annotate_safe_area(image_path, values):
    """在非 0 inset 时叠加半透明遮罩 + 参考线，产出 *_safe.png。"""
    from PIL import Image, ImageDraw
    with Image.open(image_path) as image:
        base = image.convert("RGBA")
    overlay = Image.new("RGBA", base.size, (0, 0, 0, 0))
    draw = ImageDraw.Draw(overlay)
    top, right, bottom, left = values
    width, height = base.size
    bands = [(0, 0, width, top), (width - right, 0, width, height),
             (0, height - bottom, width, height), (0, 0, left, height)]
    for box in bands:
        draw.rectangle(box, fill=(0, 0, 0, 90))
    draw.rectangle([left, top, width - right - 1, height - bottom - 1],
                   outline=(220, 175, 60, 220), width=2)
    out = image_path.with_name(image_path.stem + "_safe.png")
    Image.alpha_composite(base, overlay).save(out)
    return out


def run_matrix(scenes, device_names, spin, safe_area_path):
    """跑矩阵并汇总结论；返回 report 字典。"""
    config = load_safe_area(safe_area_path)
    results = []
    for scene in scenes:
        for name in device_names:
            width, height = DEVICES[name]
            command = shot_command(scene, width, height, spin)
            child_env = dict(os.environ)
            child_env["TI_SHOT_NAME"] = f"ui_verify_{scene}"
            process = subprocess.run(
                command, cwd=str(REPO), env=child_env, capture_output=True, text=True,
                timeout=600, check=False,
            )
            stderr = (process.stderr or "") + (process.stdout or "")
            path = shot_path(scene, width, height)
            exists = path.is_file()
            pixels = nonblack_count(path) if exists else 0
            probe = probe_present(stderr)
            orientation = "landscape" if width > height else "portrait"
            inset = config.get(orientation, [0, 0, 0, 0])
            safe_path = None
            if exists and any(inset):
                safe_path = annotate_safe_area(path, inset)
            entry = {
                "scene": scene,
                "device": name,
                "viewport": f"{width}x{height}",
                "orientation": orientation,
                "shot": str(path.relative_to(REPO)),
                "nonblack": pixels,
                "probe_ok": probe,
                "safe_area": str(safe_path.relative_to(REPO)) if safe_path else None,
                "safe_inset": inset,
                "ok": process.returncode == 0 and exists and pixels > 0 and probe,
            }
            results.append(entry)
            print(f"  [{scene}/{name}] nonblack={pixels} probe={probe} "
                  f"safe={inset} ok={entry['ok']}")
    return {
        "version": 1,
        "task": "S67.0",
        "scenes": scenes,
        "devices": device_names,
        "results": results,
        "pass": all(item["ok"] for item in results) if results else False,
    }


def check_report():
    """只读校验既有 report 与截图；不写盘、不出图。"""
    if not REPORT.is_file():
        print(f"[ui_verify] 缺证据 {REPORT}（先跑非 --check 生成）")
        return False
    with open(REPORT, "r", encoding="utf-8") as handle:
        report = json.load(handle)
    if not report.get("pass"):
        print("[ui_verify] report pass=false")
        return False
    try:
        config = load_safe_area(SAFE_AREA)
    except (ValueError, OSError) as error:
        print(f"[ui_verify] 安全区配置非法：{error}")
        return False
    for entry in report.get("results", []):
        path = REPO / entry["shot"]
        if not path.is_file() or nonblack_count(path) <= 0:
            print(f"[ui_verify] 截图缺失或空帧：{entry['shot']}")
            return False
        expected_inset = config.get(entry["orientation"], [0, 0, 0, 0])
        has_overlay = entry.get("safe_area") is not None
        if has_overlay != any(expected_inset):
            print(f"[ui_verify] 安全区叠加态与配置不一致：{entry['device']}")
            return False
    print(f"[ui_verify] --check PASS（{len(report['results'])} 张截图非黑 + 安全区配置一致）")
    return True


def main():
    """入口：生成证据或只读校验；违规非 0 退出。"""
    parser = argparse.ArgumentParser(description="设备矩阵出图验收（S67.0）")
    parser.add_argument("--check", action="store_true", help="只读校验既有证据（不出图不写盘）")
    parser.add_argument("--json", action="store_true", help="结构化输出 report JSON 到 stdout")
    parser.add_argument("--scenes", default=",".join(DEFAULT_SCENES), help="逗号分隔场景清单")
    parser.add_argument("--devices", default=None,
                        help="逗号分隔设备名（缺省 desktop+phone_landscape 2 档；quick=3 档；all=全 6 档）")
    parser.add_argument("--spin", type=int, default=5, help="main_map 摇骰步数（固定 seed 42）")
    parser.add_argument("--safe-area", default=str(SAFE_AREA), help="安全区配置路径")
    args = parser.parse_args()

    if args.check:
        return 0 if check_report() else 1

    scenes = [item.strip() for item in args.scenes.split(",") if item.strip()]
    if args.devices == "quick":
        devices = QUICK_DEVICES
    elif args.devices == "all":
        devices = list(DEVICES)
    elif args.devices:
        devices = [item.strip() for item in args.devices.split(",") if item.strip()]
    else:
        devices = DEFAULT_DEVICES
    unknown = [name for name in devices if name not in DEVICES]
    if unknown or not scenes:
        print(f"[ui_verify] 非法设备/场景：devices={unknown} scenes={scenes}")
        return 2

    try:
        report = run_matrix(scenes, devices, args.spin, Path(args.safe_area))
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(f"[ui_verify] 出图失败：{error}")
        return 1
    EVIDENCE.mkdir(parents=True, exist_ok=True)
    with open(REPORT, "w", encoding="utf-8") as handle:
        json.dump(report, handle, ensure_ascii=False, indent=2)
    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2))
    else:
        print(f"[ui_verify] report: {REPORT.relative_to(REPO)} pass={report['pass']}")
    return 0 if report["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
