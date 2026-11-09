#!/usr/bin/env python3
"""android_device_test：Android 真机交互验收（P5，docs/parity.md §P5）。

背景（2026-11-09 真机反馈「全屏/菜单无响应/后台音乐不停」根因审计）：
测试体系从设计起无「Android 真机交互」判据（P0-P4 全桌面/解析器，Android 仅「构建」条件项
且本机 SKIP），真机专属行为（全屏/触摸/音频生命周期）结构性测不到。本工具把 P5 判据落地为
adb 驱动的机器判据（设备接入即能跑；本机无设备 = gate 条件项记名 SKIP，不阻塞但文档化）。

判据（docs/parity.md §P5）：
- P5-1 全屏：app surface 尺寸 == 屏幕全尺寸（无系统栏压缩；adb `wm size` vs `dumpsys window`）。
- P5-2 触摸命中：`input tap` 菜单坐标 → 场景流转（截屏 md5 变化 或 logcat `[hit]`）。
- P5-3 音频生命周期：`KEYCODE_HOME` 切后台 → BGM pause；`am start` 回前台 → BGM resume
  （logcat `[audio] BGM 暂停/恢复`，S49.4d lifecycle_audio_system 日志）。
- P5-4 切后台不崩：`pidof` 仍在 + logcat 无 FATAL/ANR。

用法：
    python3 scripts/android_device_test.py --check           # 只读：adb/设备/包名在位检查（0 可跑 / 1 缺环境）
    python3 scripts/android_device_test.py --run             # 执行 P5-1~P5-4（需设备 + app 已装）
    python3 scripts/android_device_test.py --run --only p5-2 # 单判据

契约（14 号 §3.2 工具契约）：`--check` 只读（不写盘）+ 非 0 退出（adb 缺 / 设备缺 / 判据 FAIL）。
判据 FAIL = 非 0 退出（真实报告，不静默）。首次设备接入需标定 `--tap-x/--tap-y`（P5-2 菜单坐标）。
登记：14_tooling_catalog + 04_tooling + README 索引（S49.4e）。
"""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# 默认包名/入口（AndroidManifest.xml：com.example.treasureisland / android.app.NativeActivity）
DEFAULT_PACKAGE = "com.example.treasureisland"
DEFAULT_ACTIVITY = "android.app.NativeActivity"
# 判据阈值
FULLSCREEN_TOLERANCE = 4  # px（系统栏圆角/挖孔边缘容差）
SCREENSHOT_CHANGE_MIN = 0.02  # 截屏 md5 变化的最小像素比例（P5-2 场景流转）
LOGCAT_SINCE = "1d"


def sh(cmd: list[str], timeout: int = 30) -> tuple[int, str]:
    """跑一条命令，返回 (exit_code, 合并输出)。"""
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout or "") + (p.stderr or "")
    except FileNotFoundError:
        return 127, f"命令缺失：{cmd[0]}"
    except subprocess.TimeoutExpired:
        return 124, f"命令超时：{' '.join(cmd)}"


def adb(*args: str, timeout: int = 30) -> tuple[int, str]:
    """adb 封装（`adb <args>`）。"""
    return sh(["adb", *args], timeout=timeout)


def check_env() -> list[str]:
    """只读环境检查：adb 在位 + 设备连接 + 包已装。返回违规清单（空 = 可跑）。"""
    viol: list[str] = []
    code, out = sh(["adb", "version"])
    if code != 0:
        return ["adb 未安装或在 PATH 外（Android SDK platform-tools）"]
    code, out = adb("devices")
    if code != 0:
        viol.append(f"adb devices 失败：{out.strip()}")
        return viol
    devices = [ln for ln in out.splitlines() if "\tdevice" in ln]
    if not devices:
        viol.append("无已授权 Android 设备（adb devices 空；模拟器/真机需 USB 调试授权）")
    else:
        code, out = adb("shell", "pm", "list", "packages", DEFAULT_PACKAGE)
        if code != 0 or DEFAULT_PACKAGE not in out:
            viol.append(f"包 {DEFAULT_PACKAGE} 未安装（先 adb install <apk>）")
    return viol


def p5_1_fullscreen() -> tuple[bool, str]:
    """P5-1 全屏：app surface 尺寸 ≈ 屏幕全尺寸（无系统栏压缩）。"""
    code, size_out = adb("shell", "wm", "size")
    if code != 0:
        return False, f"wm size 失败：{size_out.strip()}"
    # Physical size: 1920x1080（或 Override size）
    screen_w = screen_h = 0
    for ln in size_out.splitlines():
        if "Physical size" in ln or "Override size" in ln:
            part = ln.split(":")[-1].strip()
            if "x" in part:
                w, h = part.split("x")[:2]
                try:
                    screen_w, screen_h = int(w), int(h)
                except ValueError:
                    pass
    if screen_w == 0 or screen_h == 0:
        return False, f"wm size 解析失败：{size_out.strip()}"
    # app 窗口 bounds（dumpsys window；不同版本键名：mFrame / appBounds / Frames）
    code, win_out = adb("shell", "dumpsys", "window", "windows")
    app_w = app_h = 0
    for ln in win_out.splitlines():
        if DEFAULT_PACKAGE not in ln:
            continue
        for key in ("mFrame=", "appBounds=", "Frames:"):
            if key in ln:
                # 提取 [0,0][1920,1080] 或 Rect(0, 0 - 1920, 1080)
                nums = [int(x) for x in __import__("re").findall(r"-?\d+", ln)]
                if len(nums) >= 4:
                    app_w = abs(nums[2] - nums[0])
                    app_h = abs(nums[3] - nums[1])
                    break
        if app_w:
            break
    if app_w == 0 or app_h == 0:
        return False, f"dumpsys window 未提取到 app bounds（{DEFAULT_PACKAGE}；格式兼容需设备标定）"
    ok = abs(app_w - screen_w) <= FULLSCREEN_TOLERANCE and abs(app_h - screen_h) <= FULLSCREEN_TOLERANCE
    detail = f"屏幕 {screen_w}x{screen_h} vs app {app_w}x{app_h}"
    return ok, detail


def p5_2_touch(tap_x: int, tap_y: int) -> tuple[bool, str]:
    """P5-2 触摸命中：input tap 菜单坐标 → 场景流转（截屏 md5 变化 或 logcat [hit]）。"""
    # 截屏基线
    code, before = adb("exec-out", "screencap", "-p")
    if code != 0:
        return False, f"screencap 失败：{before[:120]}"
    adb("logcat", "-c")
    code, _ = adb("shell", "input", "tap", str(tap_x), str(tap_y))
    if code != 0:
        return False, "input tap 失败"
    time.sleep(1.0)
    # 判据 1：logcat [hit]（app 带 TI_LOG_HITS 启动时）
    code, log = adb("logcat", "-d", "-t", "100")
    if code == 0 and "[hit]" in log:
        return True, f"logcat [hit] 命中（tap {tap_x},{tap_y}）"
    # 判据 2：截屏 md5 变化（场景流转 → 画面差异）
    code, after = adb("exec-out", "screencap", "-p")
    if code != 0:
        return False, f"screencap after 失败：{after[:120]}"
    if hashlib.md5(before).hexdigest() != hashlib.md5(after).hexdigest():
        return True, f"截屏 md5 变化（tap {tap_x},{tap_y} → 场景流转）"
    return False, f"tap {tap_x},{tap_y} 后无 [hit] 日志且截屏 md5 不变（菜单未响应？）"


def p5_3_audio_lifecycle() -> tuple[bool, str]:
    """P5-3 音频生命周期：KEYCODE_HOME 切后台 → BGM pause；am start 回前台 → resume。"""
    adb("logcat", "-c")
    adb("shell", "input", "keyevent", "KEYCODE_HOME")
    time.sleep(0.8)
    code, log = adb("logcat", "-d", "-t", "100")
    paused = code == 0 and "BGM 暂停" in log
    # 回前台
    code, _ = adb("shell", "am", "start", "-n", f"{DEFAULT_PACKAGE}/{DEFAULT_ACTIVITY}")
    time.sleep(1.0)
    adb("logcat", "-c")
    code, log = adb("logcat", "-d", "-t", "100")
    resumed = code == 0 and "BGM 恢复" in log
    if paused and resumed:
        return True, "BGM 暂停（切后台）+ BGM 恢复（回前台）"
    if paused:
        return False, "BGM 暂停 ✓ 但回前台未恢复（resume 日志缺）"
    if resumed:
        return False, "回前台恢复 ✓ 但切后台未暂停（pause 日志缺）"
    return False, "无 BGM 暂停/恢复日志（app 未带 S49.4d lifecycle_audio_system？）"


def p5_4_background_survive() -> tuple[bool, str]:
    """P5-4 切后台不崩：pidof 仍在 + logcat 无 FATAL/ANR。"""
    adb("shell", "input", "keyevent", "KEYCODE_HOME")
    time.sleep(0.8)
    adb("shell", "am", "start", "-n", f"{DEFAULT_PACKAGE}/{DEFAULT_ACTIVITY}")
    time.sleep(1.0)
    code, out = adb("shell", "pidof", DEFAULT_PACKAGE)
    alive = code == 0 and DEFAULT_PACKAGE in out
    code, log = adb("logcat", "-d", "-t", "200")
    fatal = code == 0 and ("FATAL" in log or "ANR in" in log)
    if alive and not fatal:
        return True, "切后台回前台进程存活 + 无 FATAL/ANR"
    if not alive:
        return False, "切后台后进程死亡（pidof 空）"
    return False, "logcat 含 FATAL/ANR"


def run_p5(only: str | None, tap_x: int, tap_y: int) -> int:
    """执行 P5 判据（--run）。返回 0 = 全过。同时写 `evidence/android_device_test.log`（判据证据）。"""
    checks = [
        ("p5-1", "全屏（surface=屏幕尺寸）", p5_1_fullscreen),
        ("p5-2", f"触摸命中（tap {tap_x},{tap_y}→场景流转）", lambda: p5_2_touch(tap_x, tap_y)),
        ("p5-3", "音频生命周期（切后台暂停/回前台恢复）", p5_3_audio_lifecycle),
        ("p5-4", "切后台不崩（进程存活+无 FATAL）", p5_4_background_survive),
    ]
    failed = 0
    lines: list[str] = []
    for cid, name, fn in checks:
        if only and cid != only:
            continue
        try:
            ok, detail = fn()
        except Exception as e:  # noqa: BLE001 - 判据异常 = FAIL（如实上报）
            ok, detail = False, f"判据异常：{e}"
        mark = "PASS" if ok else "FAIL"
        line = f"  [{mark}] {cid} {name}：{detail}"
        print(line)
        lines.append(line)
        if not ok:
            failed += 1
    if failed:
        summary = f"P5: {failed} 判据 FAIL（docs/parity.md §P5；设备/坐标标定见脚本 docstring）"
    else:
        summary = "P5: 全部判据 PASS"
    print(summary)
    lines.append(summary)
    # 判据证据落盘（`--run` 写 `evidence/android_device_test.log`；`--check` 不写）
    try:
        ev = ROOT / "evidence" / "android_device_test.log"
        ev.parent.mkdir(parents=True, exist_ok=True)
        ev.write_text(
            "# Android 真机交互 P5 判据证据（S49.4e；docs/parity.md §P5）\n\n"
            + "\n".join(lines)
            + "\n",
            encoding="utf-8",
        )
    except OSError:
        pass  # evidence 写失败不掩盖判据结果（如实上报已打印）
    return 1 if failed else 0


def main() -> int:
    ap = argparse.ArgumentParser(description="Android 真机交互验收（P5）")
    ap.add_argument("--check", action="store_true", help="只读环境检查（不执行判据）")
    ap.add_argument("--run", action="store_true", help="执行 P5-1~P5-4 判据")
    ap.add_argument("--only", choices=["p5-1", "p5-2", "p5-3", "p5-4"], help="单判据")
    ap.add_argument("--tap-x", type=int, default=1600, help="P5-2 菜单 tap 屏幕 x（首次标定）")
    ap.add_argument("--tap-y", type=int, default=300, help="P5-2 菜单 tap 屏幕 y（首次标定）")
    args = ap.parse_args()
    if args.check:
        viol = check_env()
        if viol:
            for v in viol:
                print(f"  [FAIL] {v}")
            print("环境检查 FAIL（本机无 adb/设备 = gate 条件项记名 SKIP，非判据失败）")
            return 1
        print("环境检查 PASS（adb + 设备 + 包在位，可跑 P5）")
        return 0
    if not args.run:
        ap.print_help()
        return 2
    # --run：先环境检查，缺 = 非 0（不静默）
    viol = check_env()
    if viol:
        for v in viol:
            print(f"  [FAIL] 环境：{v}")
        return 1
    return run_p5(args.only, args.tap_x, args.tap_y)


if __name__ == "__main__":
    sys.exit(main())
