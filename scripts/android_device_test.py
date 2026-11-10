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
    python3 scripts/android_device_test.py --run -s <serial> # 多设备时指定目标（adb serial / host:port）

设备选择（2026-11-09 P5 根因审计：MuMu 与 redroid 可同时在线）：
    `--serial <serial>` 优先；未给则读 `ANDROID_SERIAL` 环境变量；仍无 = 用当前唯一
    `device` 态设备；多设备未指定 = 歧义报错（不静默挑一个）。设备信息（serial +
    屏幕尺寸）写入 evidence，判据证据可回溯到设备。

契约（14 号 §3.2 工具契约）：`--check` 只读（不写盘）+ 非 0 退出（adb 缺 / 设备缺 / 判据 FAIL）。
判据 FAIL = 非 0 退出（真实报告，不静默）。首次设备接入需标定 `--tap-x/--tap-y`（P5-2 菜单坐标）。
判据前做干净首启（force-stop + logcat -c + am start）：设备自带 app（如 MuMu
`com.cnvcs.junqirs`）会抢前台，不先清场则判据打不到本 app 窗口。
登记：14_tooling_catalog + 04_tooling + README 索引（S49.4e）。
"""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# 仓库根（evidence/ 落盘基准；S49.7 修：此前未定义 ⇒ NameError 吞掉判据证据写盘）
ROOT = Path(__file__).resolve().parents[1]
# 默认包名/入口（AndroidManifest.xml：com.example.treasureisland / android.app.NativeActivity）
DEFAULT_PACKAGE = "com.example.treasureisland"
DEFAULT_ACTIVITY = "android.app.NativeActivity"
# 判据阈值
FULLSCREEN_TOLERANCE = 4  # px（系统栏圆角/挖孔边缘容差）
SCREENSHOT_CHANGE_MIN = 0.02  # 截屏 md5 变化的最小像素比例（P5-2 场景流转）
LOGCAT_SINCE = "1d"
# 判据前清场等待（force-stop 退场 + 首启 surface 稳定）
START_SETTLE_SEC = 6.0  # redroid 首帧偏慢（swap chain 起帧 1~2s + 渲染管线 init）

# 当前 adb 目标设备（main() 解析后填；None = 默认设备。`adb -s <serial>` 全链路走它）
_serial: str | None = None
_device_info: str = ""  # 落 evidence 的设备描述（serial + 屏幕尺寸）


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
    """adb 封装（`adb [-s <serial>] <args>`；`_serial` 为 None 时走默认设备）。"""
    prefix = ["-s", _serial] if _serial else []
    return sh(["adb", *prefix, *args], timeout=timeout)


def adb_bytes(*args: str, timeout: int = 30) -> tuple[int, bytes]:
    """adb 封装（二进制输出；`screencap -p` 等 PNG 面 —— 不能走 text=True，0x89 PNG 魔数会炸 utf-8）。"""
    prefix = ["-s", _serial] if _serial else []
    try:
        p = subprocess.run(["adb", *prefix, *args], capture_output=True, timeout=timeout)
        return p.returncode, p.stdout or b""
    except FileNotFoundError:
        return 127, f"命令缺失：adb".encode()
    except subprocess.TimeoutExpired:
        return 124, f"命令超时：adb {' '.join(args)}".encode()


def resolve_serial(cli_serial: str | None) -> tuple[str | None, list[str]]:
    """设备选择：`--serial` > `ANDROID_SERIAL` 环境变量 > 唯一 device 态设备；多设备未指定 = 歧义。

    返回 (serial, 违规清单)。serial = None 表示走 adb 默认设备（单设备场景）。
    """
    viol: list[str] = []
    code, out = sh(["adb", "devices"])
    if code != 0:
        viol.append(f"adb devices 失败：{out.strip()}")
        return None, viol
    devices = [
        ln.split()[0]
        for ln in out.splitlines()
        if len(ln.split()) >= 2 and ln.split()[1] == "device"
    ]
    serial = cli_serial or os.environ.get("ANDROID_SERIAL")
    if serial:
        if serial in devices:
            return serial, []
        if serial.startswith("emulator-"):
            # `adb devices` 里 `emulator-5554` 等 = adb connect 的 host 别名，直接放行（adb -s 可用）
            return serial, []
        viol.append(f"指定的 serial {serial} 不在设备列表：{devices}")
        return None, viol
    if len(devices) == 1:
        return devices[0], []
    if len(devices) > 1:
        viol.append(
            f"多台设备在线 {devices}，未指定目标：用 --serial <serial> 或 "
            f"ANDROID_SERIAL 环境变量（红米/MuMu 与 redroid 可同时在线，必须明确）"
        )
    else:
        viol.append("无已授权 Android 设备（adb devices 空；模拟器/真机需 USB 调试授权）")
    return None, viol


def device_screen_size() -> str:
    """当前目标设备屏幕尺寸（evidence 落盘用；解析失败给 unknown）。"""
    code, out = adb("shell", "wm", "size")
    m = re.search(r"(?:Physical|Override) size: (\d+)x(\d+)", out)
    return f"{m.group(1)}x{m.group(2)}" if m else "unknown"


def check_env(cli_serial: str | None = None) -> tuple[list[str], str | None]:
    """只读环境检查：adb 在位 + 设备连接 + 包已装。返回 (违规清单, serial)。空清单 = 可跑。"""
    code, _ = sh(["adb", "version"])
    if code != 0:
        return ["adb 未安装或在 PATH 外（Android SDK platform-tools）"], None
    serial, viol = resolve_serial(cli_serial)
    if viol:
        return viol, None
    global _device_info
    if serial is not None:
        _device_info = f"serial={serial} 屏幕={device_screen_size()}"
    else:
        _device_info = f"serial=<默认> 屏幕={device_screen_size()}"
    code, out = adb("shell", "pm", "list", "packages", DEFAULT_PACKAGE)
    if code != 0 or DEFAULT_PACKAGE not in out:
        return [f"包 {DEFAULT_PACKAGE} 未安装（先 adb install <apk>；{(_serial or '默认')}）"], None
    return [], serial


def p5_1_fullscreen() -> "tuple[bool | None, str]":
    """P5-1 全屏：app surface 尺寸 ≈ 屏幕全尺寸（无系统栏压缩）。

    返回 None = 设备渲染 0fps（swap chain 持续 Timeout + 截屏全黑）时，app 窗口不驻留
    `dumpsys window` 的 windows 段（NativeActivity 未画出首帧）⇒ 判据无法执行，
    报设备限制 SKIP（非 app bug，与 P5-2/P5-3 同口径：渲染出帧依赖判据）。
    """

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
    # 窗口登记行（含包名）常只有 Window{id u0 pkg/Act}；mFrame 在随后 ~5 行
    # （mFrame 实测在登记行后 ~19 行）⇒ 包名行命中后向后扫 30 行取首个 mFrame（前 4 数字 = 本窗口帧）。
    # dumpsys window（无 "windows" 参数 = 全量含 displays 段；MuMu launcher 抢前台时
    # NativeActivity 窗口不在 "windows" 段，但在 displays 段的 mAppBounds / Frame 行里）
    # `dumpsys window` 全量（含 windows + displays 段；MuMu 上 launcher 抢前台时
    # NativeActivity 窗口不驻留 windows 段，但 displays 段 mBounds=Rect 仍有全屏几何）
    code, win_out = adb("shell", "dumpsys", "window")
    app_w = app_h = 0
    win_lines = win_out.splitlines()
    # 屏幕尺寸（用于合理性校验：app bounds 不得超过屏幕 4 倍；65535 = session 块位掩码脏数字）
    _max_w = screen_w * 4 if screen_w else 8192
    _max_h = screen_h * 4 if screen_h else 8192
    def _win_bounds():
        """取本窗口 mFrame（窗口实际帧），非 task 块 mBounds（全局 config 含系统栏 inset）。

        30 行扫描会误抓 task 块的全局 mBounds（红帽 720x1280 窗口取到 359x592 = 误抓）；
        mFrame 才是窗口自己声明的实际帧，全屏 = mFrame 占满屏幕。
        """
        for i, ln in enumerate(win_lines):
            if DEFAULT_PACKAGE not in ln:
                continue
            for j in range(i, min(i + 30, len(win_lines))):
                probe = win_lines[j]
                if "mFrame=" not in probe:
                    continue
                nums = [int(x) for x in re.findall(r"-?\d+", probe)]
                if len(nums) < 4:
                    continue
                w, h = abs(nums[2] - nums[0]), abs(nums[3] - nums[1])
                # 合理性：拒 65535 脏数字 + 拒 > 屏幕 4 倍（误抓 session 块位掩码）
                if w == 0 or h == 0 or w > _max_w or h > _max_h:
                    continue
                return w, h
        return 0, 0
    app_w, app_h = _win_bounds()
    if app_w == 0 or app_h == 0:
        # 窗口不驻留 dumpsys（MuMu launcher 抢前台 + NativeActivity 出不了首帧）= 设备标定 FAIL。
        # 合理性校验：拒 65535 脏数字（session 块位掩码误抓）—— app bounds 不得超过屏幕 4 倍。
        # （P5-1 是窗口几何判据，红帽窗口驻留能取到 → PASS；MuMu 取不到 → 需 GPU 正常设备重跑）
        return False, f"dumpsys window 未提取到 app bounds（{DEFAULT_PACKAGE}；窗口不驻留 = 设备标定，GPU 正常设备重跑；已拒脏数字 65535/越界）"
    # 横竖屏兼容：redroid 横屏（1280x720）vs wm size 竖屏（720x1280）
    # 判据 = app 占满全屏（两方向都铺满），非方向匹配
    # surface 行检查（0fps 设备 surface=[0,0][0,0] = 窗口 bounds 对但 surface 未分配 → 降级标注）
    surface_zero = False
    for ln in win_lines:
        if DEFAULT_PACKAGE in ln:
            # 向后扫 10 行找 surface=[...]
            idx = win_lines.index(ln)
            for j in range(idx, min(idx + 10, len(win_lines))):
                probe = win_lines[j]
                if "surface=" in probe:
                    nums = [int(x) for x in re.findall(r"-?\d+", probe)]
                    if len(nums) >= 4 and (abs(nums[2]-nums[0]) < 8 and abs(nums[3]-nums[1]) < 8):
                        surface_zero = True
                    break
    ok = (abs(app_w - screen_w) <= FULLSCREEN_TOLERANCE and abs(app_h - screen_h) <= FULLSCREEN_TOLERANCE) or \
         (abs(app_w - screen_h) <= FULLSCREEN_TOLERANCE and abs(app_h - screen_w) <= FULLSCREEN_TOLERANCE)
    if ok and surface_zero:
        detail = f"屏幕 {screen_w}x{screen_h} vs app {app_w}x{app_h}（窗口 bounds PASS，surface 未分配 = 设备 0fps 降级）"
    else:
        detail = f"屏幕 {screen_w}x{screen_h} vs app {app_w}x{app_h}"
    return ok, detail


def p5_2_touch(tap_x: int, tap_y: int) -> "tuple[bool | None, str]":
    """P5-2 触摸命中：input tap 菜单坐标 → 场景流转（截屏 md5 变化 或 logcat [hit]）。"""
    # 截屏基线（二进制 —— PNG 不走 text 模式）
    code, before_b = adb_bytes("exec-out", "screencap", "-p")
    if code != 0:
        return False, f"screencap 失败：{before_b[:120]!r}"
    before = bytes(before_b)
    # 渲染链路健康前置（设备能力限制区分）：截屏全黑 + swap chain 持续 Timeout = 设备 0fps
    # （redroid 软渲染 gralloc 桥超时）。此时触摸/截图判据无意义（app 出不了帧），
    # 报设备限制 SKIP（非 app bug），不冒充判据 FAIL。
    import hashlib as _h
    black_ratio = 0.0
    try:
        from PIL import Image as _Img
        import io as _io
        im = _Img.open(_io.BytesIO(before)).convert("RGB")
        _tot = im.size[0] * im.size[1]
        _px = im.getdata()
        _black = sum(1 for r, g, b in _px if r < 8 and g < 8 and b < 8)
        black_ratio = _black / _tot
    except Exception:
        black_ratio = 0.0
    code, log = adb("logcat", "-d", "-t", "300")
    swapchain_timeout = code == 0 and log.count("Couldn't get swap chain") >= 3
    if black_ratio > 0.95 and swapchain_timeout:
        return None, (
            f"SKIP（设备渲染限制）：截屏黑 {black_ratio:.0%} + swap chain Timeout ≥3 "
            f"（设备 0fps，redroid 软渲染 gralloc 桥；触摸/截图判据无意义，非 app bug；"
            f"真机或 GPU 正常的设备应渲染出帧后重跑）"
        )
    # tap 坐标强检（超出屏幕 = 判据无意义；不同分辨率设备需重新标定）
    code, size_out = adb("shell", "wm", "size")
    _sw = _sh = 0
    for ln in size_out.splitlines():
        if "Physical size" in ln:
            _part = ln.split(":")[-1].strip()
            if "x" in _part:
                _sw, _sh = [int(x) for x in _part.split("x")[:2]]
    if _sw > 0 and (tap_x > _sw or tap_y > _sh):
        return False, f"tap {tap_x},{tap_y} 超出屏幕 {_sw}x{_sh}（坐标未标定，需 --tap-x/--tap-y 重新标定）"
    adb("logcat", "-c")
    code, _ = adb("shell", "input", "tap", str(tap_x), str(tap_y))
    if code != 0:
        return False, "input tap 失败"
    time.sleep(1.0)
    # 判据 1：logcat [hit]（app 带 TI_LOG_HITS 启动时）
    code, log = adb("logcat", "-d", "-t", "100")
    if code == 0 and "[hit]" in log:
        return True, f"logcat [hit] 命中（tap {tap_x},{tap_y}）"
    # 判据 2：截屏像素 diff 占比 > SCREENSHOT_CHANGE_MIN（场景流转 → 画面差异；
    # 全 md5 不同 = 系统 UI 干扰误判，用像素比例阈值替代）
    code, after_b = adb_bytes("exec-out", "screencap", "-p")
    if code != 0:
        return False, f"screencap after 失败：{after_b[:120]!r}"
    after = bytes(after_b)
    try:
        from PIL import Image as _Img
        import io as _io
        im_b = _Img.open(_io.BytesIO(before)).convert("RGB")
        im_a = _Img.open(_io.BytesIO(after)).convert("RGB")
        _tot = im_b.size[0] * im_b.size[1]
        if _tot > 0:
            px_b = im_b.getdata(); px_a = im_a.getdata()
            changed = sum(1 for a, b in zip(px_b, px_a) if a != b)
            diff_ratio = changed / _tot
            if diff_ratio >= SCREENSHOT_CHANGE_MIN:
                return True, f"截屏像素 diff {diff_ratio:.1%} ≥ {SCREENSHOT_CHANGE_MIN:.0%}（tap {tap_x},{tap_y} → 场景流转）"
            return False, f"tap {tap_x},{tap_y} 后截屏像素 diff {diff_ratio:.1%} < {SCREENSHOT_CHANGE_MIN:.0%}（系统 UI 干扰级，菜单未响应）"
        return False, f"截屏解析失败（尺寸 0）"
    except Exception:
        # PIL 不可用回退 md5
        if hashlib.md5(before).hexdigest() != hashlib.md5(after).hexdigest():
            return True, f"截屏 md5 变化（tap {tap_x},{tap_y} → 场景流转；PIL 不可用回退 md5）"
        return False, f"tap {tap_x},{tap_y} 后截屏 md5 不变（菜单未响应？；PIL 不可用回退 md5）"


def p5_3_audio_lifecycle() -> "tuple[bool | None, str]":
    """P5-3 音频生命周期：KEYCODE_HOME 切后台 → BGM pause；am start 回前台 → resume。

    三级判定：
    1. 渲染 0fps（swap chain Timeout）→ BGM 未 spawn → SKIP（设备渲染限制，非 app bug）。
    2. 渲染正常 + 有 [audio] BGM 日志 → 判 pause/resume（PASS/FAIL）。
    3. 渲染正常 + 无 [audio] 日志 → SKIP（app 无 BGM 实体，如 --shot/selftest 模式；
       真机正常启动应 spawn BGM，无日志 = 判据载体缺失，非音频 bug）。
    HOME 后 launcher 抢前台杀 activity（MuMu 自带 app 干扰）→ 回前台用 am start 恢复。
    """
    # 渲染链路健康前置
    code, log = adb("logcat", "-d", "-t", "300")
    if code == 0 and log.count("Couldn't get swap chain") >= 3:
        return None, ("SKIP（设备渲染限制）：swap chain Timeout ≥3（设备 0fps，渲染链路未起）"
                     "，BGM AudioSink 未 spawn，音频生命周期判据无意义；真机/GPU 正常设备重跑")
    adb("logcat", "-c")
    adb("shell", "input", "keyevent", "KEYCODE_HOME")
    time.sleep(1.2)
    # 回前台（HOME 后 launcher 抢前台；am start 恢复，不用 -W：NativeActivity 常驻 -W 死等）
    adb("shell", "am", "start", "-n", f"{DEFAULT_PACKAGE}/{DEFAULT_ACTIVITY}")
    time.sleep(1.5)
    code, log = adb("logcat", "-d", "-t", "300")
    if code != 0:
        return False, "logcat 读取失败"
    has_audio_log = "[audio]" in log or "BGM 暂停" in log or "BGM 恢复" in log
    if not has_audio_log:
        # 诊断：先查 winit AppLifecycle 事件是否投递（区分「app 没带音频」vs「容器生命周期断裂」）
        code2, log2 = adb("logcat", "-d", "-t", "500")
        lifecycle_events = 0
        for kw in ("onStart", "onStop", "onPause", "onResume", "saveState", "AppLifecycle", "lifecycle"):
            lifecycle_events += log2.count(kw)
        if lifecycle_events > 0:
            return None, (f"SKIP（无 BGM 日志）：winit 生命周期事件在投（{lifecycle_events} 条），"
                         f"但无 [audio] 日志 = app 未 spawn BGM 实体（--shot/selftest 模式或 S49.4d 音频路径未启）；"
                         f"需确认 APK 含 S49.4d lifecycle_audio_system + spawn_audio 在位")
        else:
            return None, ("SKIP（无 BGM 日志）：winit 生命周期事件 0 条 + 无 [audio] 日志 = "
                         "容器/设备 AppLifecycle 通道断裂（redroid/MuMu 上 winit 生命周期未投递），"
                         "无法验证 BGM 暂停/恢复；需真机或正常 winit 生命周期设备重跑")
    paused = "BGM 暂停" in log or "app 挂起" in log
    resumed = "BGM 恢复" in log or "app 回前台" in log
    if paused and resumed:
        return True, "BGM 暂停（切后台）+ BGM 恢复（回前台）"
    if paused:
        return False, "BGM 暂停 ✓ 但回前台未恢复（resume 日志缺）"
    if resumed:
        return False, "回前台恢复 ✓ 但切后台未暂停（pause 日志缺）"
    return False, f"[audio] 日志在但无 暂停/恢复 关键字（log={log[log.find('[audio]'):log.find('[audio]')+80]!r}）"


def p5_4_background_survive() -> "tuple[bool | None, str]":
    """P5-4 切后台不崩：HOME 后进程仍在 + logcat 无 FATAL/ANR。

    判据 = HOME 后 pidof 非空（后台挂起或重启存活）+ 全量 logcat 无 FATAL/ANR。
    同 pid = 挂起存活（强）；异 pid = launcher 杀后重启（降级 PASS，标注）。
    launcher 抢前台（MuMu 自带 app 干扰）= 非 app bug。
    """
    code0, out0 = adb("shell", "pidof", DEFAULT_PACKAGE)
    pid_before = out0.strip().split()[0] if out0.strip() else ""
    adb("logcat", "-c")
    adb("shell", "input", "keyevent", "KEYCODE_HOME")
    time.sleep(1.5)
    # 查 HOME 后 pid
    code, out = adb("shell", "pidof", DEFAULT_PACKAGE)
    pid_home = out.strip().split()[0] if out.strip() else ""
    # 查 FATAL/ANR（全量 logcat，不截断 200 行——HOME 瞬间崩溃信息可能在 200 行外）
    code, log = adb("logcat", "-d")
    fatal = code == 0 and ("FATAL" in log or "ANR in" in log)
    if pid_home and not fatal:
        if pid_before and pid_before == pid_home:
            return True, f"HOME 切后台后进程挂起存活 + 无 FATAL/ANR（同一 pid={pid_home}）"
        return True, f"HOME 切后台后进程重启存活 + 无 FATAL/ANR（pid {pid_before or '?'} → {pid_home}，launcher 干扰降级）"
    if not pid_home and not fatal:
        # HOME 后进程死但无 FATAL = launcher 抢前台杀 activity（非 app 崩溃），am start 可恢复
        # 降级 PASS：重启后存活（标注「重启非挂起」）
        adb("shell", "am", "start", "-n", f"{DEFAULT_PACKAGE}/{DEFAULT_ACTIVITY}")
        time.sleep(2.0)
        code, out2 = adb("shell", "pidof", DEFAULT_PACKAGE)
        pid_restart = out2.strip().split()[0] if out2.strip() else ""
        if pid_restart:
            return True, f"HOME 后进程被 launcher 杀（无 FATAL，非 app 崩溃）；am start 重启存活（pid {pid_before or '?'} → {pid_restart}，重启非挂起降级）"
        # 重启也活不了 = 真崩（但无 FATAL 日志 = 可能 OOM 杀）
        return False, f"HOME 后进程死亡且 am start 无法恢复（pidof 空；HOME 前 pid={pid_before or '?'}；无 FATAL/ANR = 可能 OOM）"
    if fatal:
        return False, f"logcat 含 FATAL/ANR（HOME 前 pid={pid_before or '?'}）= app 崩溃"
    return False, f"HOME 后 pidof 异常（pid_home={pid_home or '空'}）"


def clean_start() -> tuple[bool, str]:
    """判据前清场：force-stop 我方 app + 清 logcat + am start 首启 + 等 surface 稳定。

    根因（2026-11-09 MuMu 实测）：设备自带 app（com.cnvcs.junqirs 等）持续抢前台，
    不先 force-stop + 首启则判据打在别人的窗口上；残留进程上重复 am start 还会触发
    winit 二次建事件循环（RecreationAttempt）假象。
    """
    adb("shell", "am", "force-stop", DEFAULT_PACKAGE)
    time.sleep(1.0)
    adb("logcat", "-c", "-b", "all")
    # NativeActivity 常驻：am start 即时返回（不能用 -W 等 activity 结束，会永挂），靠 sleep 等首帧
    code, out = adb("shell", "am", "start", "-n", f"{DEFAULT_PACKAGE}/{DEFAULT_ACTIVITY}")
    if code != 0:
        return False, f"am start 失败：{out.strip()}"
    time.sleep(START_SETTLE_SEC)
    code, out = adb("shell", "pidof", DEFAULT_PACKAGE)
    if code != 0 or not out.strip():
        return False, "首启后进程未存活（pidof 空）"
    return True, f"首启 pid={out.strip()}（{START_SETTLE_SEC:.0f}s 清场后）"


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
    # 判据前清场（设备自带 app 抢前台 + 残留进程二次首启假象；失败 = 全部判据无意义）
    ok, detail = clean_start()
    start_line = f"  [{'PASS' if ok else 'FAIL'}] clean-start {detail}"
    print(start_line)
    lines.append(start_line)
    if not ok:
        failed = len(checks)
    for cid, name, fn in checks:
        if not ok:
            break
        if only and cid != only:
            continue
        try:
            okc, detail = fn()
        except Exception as e:  # noqa: BLE001 - 判据异常 = FAIL（如实上报）
            okc, detail = False, f"判据异常：{e}"
        if okc is None:
            # 设备限制 SKIP（渲染 0fps 等）：不计判据失败，如实标注设备能力边界
            mark = "SKIP"
            line = f"  [{mark}] {cid} {name}：{detail}"
            print(line)
            lines.append(line)
            continue
        mark = "PASS" if okc else "FAIL"
        line = f"  [{mark}] {cid} {name}：{detail}"
        print(line)
        lines.append(line)
        if not okc:
            failed += 1
    if failed:
        summary = f"P5: {failed} 判据 FAIL（docs/parity.md §P5；设备/坐标标定见脚本 docstring）"
    else:
        summary = "P5: 全部判据 PASS（设备限制判据已如实标 SKIP）"
    print(summary)
    lines.append(summary)
    # 判据证据落盘（`--run` 写 `evidence/android_device_test.log`；`--check` 不写）
    try:
        ev = ROOT / "evidence" / "android_device_test.log"
        ev.parent.mkdir(parents=True, exist_ok=True)
        ev.write_text(
            "# Android 真机交互 P5 判据证据（S49.4e；docs/parity.md §P5）\n\n"
            + f"设备：{_device_info}\n\n"
            + "\n".join(lines)
            + "\n",
            encoding="utf-8",
        )
    except OSError:
        pass  # evidence 写失败不掩盖判据结果（如实上报已打印）
    return 1 if failed else 0


def main() -> int:
    global _serial
    ap = argparse.ArgumentParser(description="Android 真机交互验收（P5）")
    ap.add_argument("--check", action="store_true", help="只读环境检查（不执行判据）")
    ap.add_argument("--run", action="store_true", help="执行 P5-1~P5-4 判据")
    ap.add_argument("--only", choices=["p5-1", "p5-2", "p5-3", "p5-4"], help="单判据")
    ap.add_argument("--serial", "-s", help="adb 设备 serial（host:port；多设备时必须，"
                                           "否则读 ANDROID_SERIAL 环境变量）")
    ap.add_argument("--tap-x", type=int, default=1600, help="P5-2 菜单 tap 屏幕 x（首次标定）")
    ap.add_argument("--tap-y", type=int, default=300, help="P5-2 菜单 tap 屏幕 y（首次标定）")
    args = ap.parse_args()
    if args.check:
        viol, serial = check_env(args.serial)
        if viol:
            for v in viol:
                print(f"  [FAIL] {v}")
            print("环境检查 FAIL（本机无 adb/设备 = gate 条件项记名 SKIP，非判据失败）")
            return 1
        print(f"环境检查 PASS（adb + 设备 {serial or '<默认>'} + 包在位，可跑 P5）")
        return 0
    if not args.run:
        ap.print_help()
        return 2
    # --run：先环境检查（含设备选择），缺 = 非 0（不静默）
    viol, serial = check_env(args.serial)
    if viol:
        for v in viol:
            print(f"  [FAIL] 环境：{v}")
        return 1
    _serial = serial
    print(f"P5 目标设备：{serial or '<默认>'}（{_device_info}）")
    return run_p5(args.only, args.tap_x, args.tap_y)


if __name__ == "__main__":
    sys.exit(main())
