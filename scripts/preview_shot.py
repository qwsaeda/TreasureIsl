#!/usr/bin/env python3
"""preview_shot.py — S86.3 真渲染预览出图包装器（平台 `POST /api/preview` 后台 job 执行体）。

用途
----
把「`treasure-bevy-native --shot`」这条权威渲染命令固化成 python3 包装工具，
使平台能在**不扩程序白名单**（`JOB_PROGRAMS = cargo/python3/pwsh/powershell`）的前提下
后台出图：逐参数 `subprocess.run`（禁 shell 拼接）+ 白名单 scene/viewport/id
+ gitignored 临时目录防穿越 + PNG 内容判据（存在 / 非纯色）。

口径（单一事实源 = docs/plan-s86-visual-editor.md §2）
----------------------------------------------------
- 引擎 env 窄接缝：`TI_PREVIEW_DIR` 指向 `evidence/console/preview/<id>/`，
  引擎在 `inject_map_pack()` 之前经 `ui_schema::install_injected` /
  `table_source::install` 注入临时数据（OnceLock 一次性 ⇒ 每请求一个进程）；
- `TI_SHOT_NAME = bevy_native_preview_<id>`（引擎追加 `_<WxH>.png` 后缀）；
- 出图 = `evidence/bevy_native_preview_<id>_<WxH>.png`（gitignored，前端 `<img src="/evidence/...?t=">`）；
- 无头环境走 `/usr/bin/xvfb-run -a`（Linux 迁移口径，同 theme preview）。

用法
----
    python3 scripts/preview_shot.py --scene index --viewport 1920x1080 \\
        --preview-dir evidence/console/preview/<id> --id <id> [--map-id m01]
    python3 scripts/preview_shot.py --check --scene index --viewport 1920x1080 \\
        --preview-dir evidence/console/preview/<id> --id <id>   # 只读体检，不跑引擎

退出码：成功 = 0；参数非法 / 前置缺失 / 出图判据失败 = 1（`sys.exit(1)`）。
"""
from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PREVIEW_ROOT = (REPO / "evidence" / "console" / "preview").resolve()
SHOT_DIR = REPO / "evidence"

# 与 routes::PREVIEW_SCENES / PREVIEW_VIEWPORTS 同源（doccheck2 三面对账由主线程登记）。
SCENES = ("index", "select_role", "select_map", "saveload", "message", "main_map")
VIEWPORTS = ("1920x1080", "1080x1920", "1280x720", "960x540")
ID_RE = re.compile(r"^[a-z0-9_-]{1,64}$")


def resolve_bin() -> Path:
    """解析 `treasure-bevy-native` bin 路径（CARGO_TARGET_DIR > .cargo/config.toml > repo/target）。"""
    target = os.environ.get("CARGO_TARGET_DIR", "").strip()
    if not target:
        cfg = REPO / ".cargo" / "config.toml"
        if cfg.is_file():
            for line in cfg.read_text(encoding="utf-8", errors="replace").splitlines():
                stripped = line.strip()
                if stripped.startswith("target-dir"):
                    target = (
                        stripped.split("=", 1)[1].strip().strip('"').strip("'")
                        if "=" in stripped
                        else ""
                    )
                    break
    exe = "treasure-bevy-native.exe" if os.name == "nt" else "treasure-bevy-native"
    candidate = Path(target) / "debug" / exe if target else REPO / "target" / "debug" / exe
    return candidate if candidate.is_absolute() else REPO / candidate


def allowed_preview_dir(raw: str) -> Path:
    """预览目录必须是 `evidence/console/preview/` 的子路径（防自由路径 / 穿越）。"""
    path = Path(raw)
    resolved = (path if path.is_absolute() else REPO / path).resolve()
    try:
        under = os.path.commonpath([str(PREVIEW_ROOT), str(resolved)]) == str(PREVIEW_ROOT)
    except ValueError:
        under = False
    if not under or resolved == PREVIEW_ROOT:
        raise ValueError(f"preview-dir 非法（须为 evidence/console/preview/<id>/ 子目录）：{raw}")
    return resolved


def image_ok(path: Path) -> tuple[bool, str]:
    """出图判据（同 ui_wizard::shot_image_ok）：存在 + ≥1KiB + 字节多样性 ≥4（纯色帧判败）。"""
    if not path.is_file():
        return False, f"未找到出图 {path}"
    data = path.read_bytes()
    if len(data) < 1024:
        return False, f"图过小（{len(data)} B），疑纯色/黑帧"
    distinct = len(set(data))
    if distinct < 4:
        return False, f"图字节多样性低（{distinct} 种），疑纯色帧"
    return True, f"出图 {path}（{len(data)} B，字节多样性 {distinct}）"


def shot_path(preview_id: str, viewport: str) -> Path:
    """引擎出图落点（`TI_SHOT_NAME` 基名 + `_<WxH>.png` 后缀）。"""
    return SHOT_DIR / f"bevy_native_preview_{preview_id}_{viewport}.png"


def build_command(bin_path: Path, args: argparse.Namespace) -> list[str]:
    """构造逐参数命令（无 shell；无头环境前置 xvfb-run）。"""
    command: list[str] = []
    xvfb = Path("/usr/bin/xvfb-run")
    if xvfb.is_file() and not os.environ.get("DISPLAY"):
        command.extend([str(xvfb), "-a"])
    command.extend(
        [
            str(bin_path),
            "--shot",
            f"--scene={args.scene}",
            f"--viewport={args.viewport}",
        ]
    )
    return command


def validate(args: argparse.Namespace) -> Path:
    """参数白名单校验（非法直接 sys.exit(1)，不进 spawn）。"""
    if args.scene not in SCENES:
        print(f"scene 非法（白名单 {SCENES}）：{args.scene}", file=sys.stderr)
        sys.exit(1)
    if args.viewport not in VIEWPORTS:
        print(f"viewport 非法（白名单 {VIEWPORTS}）：{args.viewport}", file=sys.stderr)
        sys.exit(1)
    if not ID_RE.fullmatch(args.id):
        print(f"id 非法（须 ^[a-z0-9_-]{{1,64}}$）：{args.id}", file=sys.stderr)
        sys.exit(1)
    if args.map_id and not re.fullmatch(r"[a-z0-9_]{1,64}", args.map_id):
        print(f"map-id 非法（须 ^[a-z0-9_]{{1,64}}$）：{args.map_id}", file=sys.stderr)
        sys.exit(1)
    try:
        return allowed_preview_dir(args.preview_dir)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)


def do_check(args: argparse.Namespace, preview_dir: Path) -> int:
    """只读体检：参数 / bin / 临时目录 / 注入文件清单；不跑引擎、不写盘。"""
    bin_path = resolve_bin()
    files = sorted(str(p.relative_to(preview_dir)) for p in preview_dir.rglob("*.json")) if preview_dir.is_dir() else []
    report = {
        "ok": bin_path.is_file() and preview_dir.is_dir(),
        "scene": args.scene,
        "viewport": args.viewport,
        "id": args.id,
        "bin": str(bin_path),
        "bin_ok": bin_path.is_file(),
        "preview_dir": str(preview_dir),
        "preview_dir_ok": preview_dir.is_dir(),
        "injected_files": files,
        "command": build_command(bin_path, args),
    }
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report["ok"] else 1


def print_output(output: str) -> None:
    """日志回显：头 + 尾（注入日志在开头，ALSA 报错刷屏在末尾，单取尾部会丢注入证据）。"""
    if len(output) <= 8000:
        print(output)
        return
    print(output[:2000])
    print(f"[preview] …中间省略 {len(output) - 8000} 字符…")
    print(output[-6000:])


def do_run(args: argparse.Namespace, preview_dir: Path) -> int:
    """后台出图主路径（逐参数非 shell；env 次序 = 最小集后重设 TI_*）。"""
    bin_path = resolve_bin()
    if not bin_path.is_file():
        print(f"treasure-bevy-native 未构建：{bin_path}", file=sys.stderr)
        sys.exit(1)
    # 清掉上一次同 id 出图，避免引擎冷启失败时误把旧图当本次成功证据。
    out = shot_path(args.id, args.viewport)
    if out.is_file():
        out.unlink()
    env = os.environ.copy()
    # jobs::spawn_program_logged 已 apply_minimal_env（env_clear + 白名单），此处只补 TI_*；
    # 无 TI_PREVIEW_DIR 时引擎 0 行为变化（走内嵌默认包）。
    env["TI_PREVIEW_DIR"] = str(preview_dir)
    env["TI_SHOT_NAME"] = f"bevy_native_preview_{args.id}"
    if args.map_id:
        env["TI_MAP"] = args.map_id
    command = build_command(bin_path, args)
    print(f"[preview] 命令：{' '.join(command)}", flush=True)
    print(f"[preview] 引擎冷启全量资产预载通常 3~6 分钟，日志实时刷新…", flush=True)
    # S87.5：旧实现 `subprocess.run(capture_output=True)` 把引擎输出全缓冲到进程结束才打印
    # ⇒ 用户点预览后日志 0 字节、前端「提交作业…」假死。改为 Popen 逐行转发 + flush，
    # 日志实时可见；超时原样保留（kill 子进程）。
    proc = subprocess.Popen(
        command,
        cwd=str(REPO),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    lines: list[str] = []
    try:
        assert proc.stdout is not None
        for line in proc.stdout:
            print(line, end="", flush=True)
            lines.append(line)
        returncode = proc.wait(timeout=max(0, args.timeout))
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
        print(f"[preview] 出图超时（{args.timeout}s）", file=sys.stderr, flush=True)
        sys.exit(1)
    output = "".join(lines)
    ok, note = image_ok(out)
    shot_ok = "已写" in output
    print(f"[preview] {note}", flush=True)
    print(f"PNG={out.relative_to(REPO).as_posix()}", flush=True)
    print(f"OK={1 if returncode == 0 and shot_ok and ok else 0}", flush=True)
    if returncode != 0 or not shot_ok or not ok:
        print(
            f"[preview] 判据失败：exit={returncode} 已写={shot_ok} 图像={ok}",
            file=sys.stderr,
            flush=True,
        )
        sys.exit(1)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="S86.3 真渲染预览出图（treasure-bevy-native --shot 包装）")
    parser.add_argument("--scene", required=True, help="场景 id（5 UI 场景 | main_map）")
    parser.add_argument("--viewport", default="1920x1080", help="视口 WxH（默认 1920x1080）")
    parser.add_argument("--preview-dir", required=True, help="gitignored 临时注入目录")
    parser.add_argument("--id", required=True, help="作业 id（出图基名后缀）")
    parser.add_argument("--map-id", default="", help="可选地图包 id（注入 TI_MAP / 包段）")
    parser.add_argument("--timeout", type=int, default=900, help="单张出图超时秒数（默认 900）")
    parser.add_argument("--check", action="store_true", help="只读体检（不跑引擎、不写盘）")
    args = parser.parse_args()
    preview_dir = validate(args)
    if args.check:
        return do_check(args, preview_dir)
    return do_run(args, preview_dir)


if __name__ == "__main__":
    sys.exit(main())
