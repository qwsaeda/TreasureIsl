#!/usr/bin/env python3
"""构建产物编排器（S35.1，24 号标准 §4.1 执行面；平台「构建」页 / `console_commands.json` 消费）。

用途
----
把「LLM 不确定、但可程序化确定」的构建动作固化为单一入口，消除 LLM 每次手拼命令
（bash 后台任务 / mtime 锚定 / 写面登记 / 构建后复检）的重复错误面：

- `pre <kind>` / `post <kind>`：构建前后 mtime 锚定（防手动改过 .rs 的 mtime 误触发 15min 重编，
  口径 = 24 号 §1「.so 复用判据」）；
- `run <kind>`：执行构建本体（调既有脚本，工具契约；构建后自动 `post` + 体积复检）；
- `list`：列出已构建产物（体积 / 修改时间 / 判据状态）；
- `check`：只读体检（python3 在位 + 各 kind 前置依赖在位；非 0 退出）。

口径（单一事实源 = docs/process/24_apk_size_std.md）
----------------------------------------------------
- kind = `apk`（build_apk.sh，复用 .so 时秒级）/ `wasm`（build_wasm.ps1，含 cargo 编译）/
  `native`（cargo build treasure-bevy-native，冒烟）；
- 复检 = 构建后自动跑 `apk_audit.py --apk`（apk）/ 产物在位断言（wasm/native）；
- 写面 = `dist/`（apk）+ `web/`（wasm）+ `target/`（native）；平台侧登记 22 号 §1 写点表。

退出码：成功 = 0；前置缺失 / 构建失败 / 复检 FAIL = 非 0。
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
STATE_DIR = REPO / "evidence" / "console"
MTIME_FILE = STATE_DIR / "build_mtimes.json"
LOG_DIR = STATE_DIR / "logs"

# 各 kind 的构建配置（runner = 绝对命令；body = 是否写 dist/web；复检 = 构建后动作）
KINDS: dict[str, dict] = {
    "apk": {
        "runner": "bash",
        "cmd": ["bash", str(REPO / "scripts/build_apk.sh"), "aarch64"],
        # S46.3：build_apk.sh 产物名 = `TreasureIsl-<x.y.z>-<arch>.apk`（S25 版本规范，arch=aarch64；
        # 注释/文档曾写 arm64，脚本实际用 aarch64 CLI 参）⇒ glob 匹配（旧 `TreasureIsl-arm64.apk`
        # 与新版 `TreasureIsl-0.2.0-aarch64.apk` 均命中），不再硬编码精确名导致「已构建仍报未构建」。
        "artifact": "dist/TreasureIsl-*.apk",
        "post": "apk_audit",
        "note": "复用 .so 时秒级；.so 比壳源新才重编（>15min）",
    },
    "wasm": {
        "runner": "pwsh",
        "cmd": ["pwsh", "-NoProfile", "-File", str(REPO / "scripts/build_wasm.ps1")],
        # S25 版本规范（25_versioning.md §2）：wasm 产物名带版本 `treasure_bevy_wasm-<x.y.z>.wasm`；
        # artifact_present 复检 = 按 glob 找最新带版本文件（旧 raw 副本已退役）。
        "artifact": "web/treasure_bevy_wasm-*.wasm",
        "post": "artifact_present",
        "note": "含 cargo 编译（release wasm 目标）；按引用集裁剪部署；产物名带版本（S25）",
    },
    "native": {
        "runner": "cargo",
        "cmd": ["cargo", "build", "--release", "-p", "ti-engine", "--bin", "treasure-bevy-native"],
        "artifact": "",  # native 产物在 config target-dir（/disk/target）/release/，不部署；复检 = 构建 0 退出码
        "post": "exit_code",
        "note": "桌面冒烟构建；F4 切画质",
    },
}


def _mtimes() -> dict[str, int]:
    if MTIME_FILE.is_file():
        try:
            return json.loads(MTIME_FILE.read_text(encoding="utf-8"))
        except (json.JSONDecodeError, OSError):
            return {}
    return {}


def _save_mtimes(data: dict[str, int]) -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    MTIME_FILE.write_text(json.dumps(data, indent=1, sort_keys=True) + "\n", encoding="utf-8")


def _source_mtime(kind: str) -> int:
    """该 kind 构建入口脚本的 mtime（apk = build_apk.sh；wasm/native = 各自入口）。"""
    p = REPO / "scripts/build_apk.sh" if kind == "apk" else (
        REPO / "scripts/build_wasm.ps1" if kind == "wasm" else REPO / "crates/ti-engine/Cargo.toml"
    )
    return int(p.stat().st_mtime) if p.exists() else 0


def cmd_pre(kind: str) -> int:
    if kind not in KINDS:
        print(f"未知 kind: {kind}（可选 {list(KINDS)}）", file=sys.stderr)
        return 1
    if kind == "apk":
        # .so 复用判据锚定：build_apk.sh 比 `.so -nt treasure_bevy.rs` 决定复用/重编。
        # 手动改过 .rs（IDE 保存 / git 操作）会使 .rs mtime > .so ⇒ 误触发 15min 重编。
        # 此处把 .rs mtime 回退到 .so 前 10s（仅 mtime，不改内容；构建后 post 复原原值）。
        so = Path("/disk/target/aarch64-linux-android/release/libandroid_activity.so")
        shell_src = REPO / "crates/ti-engine/src/bin/treasure_bevy.rs"
        mt = _mtimes()
        if so.is_file() and shell_src.is_file():
            orig = int(shell_src.stat().st_mtime)
            if orig > int(so.stat().st_mtime):
                import os as _os
                _os.utime(shell_src, (int(so.stat().st_mtime - 10), int(so.stat().st_mtime - 10)))
                mt["apk_shell_src_orig"] = orig
                _save_mtimes(mt)
                print(f"pre: 回退 {shell_src.name} mtime → .so 前 10s（复用判据，避免误重编）")
            else:
                print("pre: 无需锚定（.rs 未比 .so 新）")
    else:
        print(f"pre: {kind} 无 mtime 锚定需求（cargo 按 hash 增量）")
    return 0


def _restore_mtime() -> None:
    mt = _mtimes()
    orig = mt.pop("apk_shell_src_orig", None)
    if orig is not None:
        shell_src = REPO / "crates/ti-engine/src/bin/treasure_bevy.rs"
        import os as _os
        _os.utime(shell_src, (orig, orig))
        _save_mtimes(mt)
        print(f"post: 复原 {shell_src.name} mtime → {orig}")


def _resolve_artifact(kind: str) -> Path | None:
    """解析 kind 的产物路径（S46.3：artifact 可为 glob 模式，取最新匹配；字面路径 is_file 直判）。"""
    art = KINDS.get(kind, {}).get("artifact", "")
    if not art:
        return None
    p = REPO / art
    if "*" in art:
        matches = sorted(p.parent.glob(p.name))
        return matches[-1] if matches else None
    return p if p.is_file() else None


def cmd_post(kind: str) -> int:
    if kind not in KINDS:
        print(f"未知 kind: {kind}", file=sys.stderr)
        return 1
    post = KINDS[kind]["post"]
    if post == "apk_audit":
        apk = _resolve_artifact(kind)
        if apk is None:
            print("post: APK 未生成", file=sys.stderr)
            return 2
        r = subprocess.run([sys.executable, str(REPO / "scripts/apk_audit.py"), "--apk", str(apk)],
                           capture_output=True, text=True)
        print(r.stdout.strip().splitlines()[-1] if r.stdout.strip() else "(无输出)")
        if r.returncode != 0:
            print(f"post: 体积复检 FAIL（exit {r.returncode}）", file=sys.stderr)
            for ln in (r.stderr or "").splitlines():
                print(ln, file=sys.stderr)
            return 3
        return 0
    if post == "artifact_present":
        art = REPO / KINDS[kind]["artifact"]
        # S25 版本规范：wasm/apk 产物名带版本（`*.<x.y.z>.*` glob 模式）——用 glob 找最新文件；
        # 旧版精确名（无 `*`）走 is_file 直判（兼容）。
        if "*" in str(art):
            matches = sorted(art.parent.glob(art.name))
            if not matches:
                print(f"post: 产物缺失（glob 无匹配）{art}", file=sys.stderr)
                return 2
            art = matches[-1]
        if not art.is_file():
            print(f"post: 产物缺失 {art}", file=sys.stderr)
            return 2
        print(f"post: 产物在位 {art}（{art.stat().st_size//1024}KB）")
        return 0
    # post == "exit_code"：native 由 run 的返回码直接判定
    return 0


def cmd_run(kind: str, log_name: str | None = None) -> int:
    if kind not in KINDS:
        print(f"未知 kind: {kind}", file=sys.stderr)
        return 1
    cfg = KINDS[kind]
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log = LOG_DIR / (log_name or f"build-{kind}-{int(time.time())}.log")
    # pre（apk 锚定 mtime）
    pre_r = cmd_pre(kind)
    if pre_r != 0:
        return pre_r
    with log.open("w", encoding="utf-8") as lf:
        lf.write(f"$ {cfg['cmd'][0]}\n")
        # 子进程补 ~/.cargo/bin 到 PATH（cargo/rustup 常在此；构建走 cargo 子命令时 shell profile 不一定生效）
        env = os.environ.copy()
        env["PATH"] = str(Path.home() / ".cargo/bin") + os.pathsep + env.get("PATH", "")
        r = subprocess.run(cfg["cmd"], cwd=str(REPO), stdout=lf, stderr=subprocess.STDOUT, text=True, env=env)
    print(f"构建日志: {log}")
    _restore_mtime()  # 无论成败都复原 .rs mtime（build_apk.sh 构建后 .so 若重编已更新，复原无副作用）
    if r.returncode != 0:
        print(f"构建 FAIL（exit {r.returncode}；见 {log.name}）", file=sys.stderr)
        return r.returncode
    return cmd_post(kind)


def cmd_list() -> int:
    for kind in ("apk", "wasm", "native"):
        cfg = KINDS[kind]
        art = _resolve_artifact(kind)
        if art and art.is_file():
            st = art.stat()
            print(f"{kind:8} {art}  {st.st_size/1048576:.1f}MB  mtime={time.strftime('%m-%d %H:%M', time.localtime(st.st_mtime))}")
        else:
            print(f"{kind:8} 未构建")
    return 0


def cmd_check() -> int:
    import shutil

    def _which(name: str) -> bool:
        if shutil.which(name):
            return True
        # cargo 常在 ~/.cargo/bin（shell profile 才进 PATH；构建走 cargo 子命令时由 shell 提供）
        if name == "cargo":
            return (Path.home() / ".cargo/bin/cargo").exists()
        return False

    ok = True
    for kind, cfg in KINDS.items():
        runner = cfg["cmd"][0]
        if not _which(runner):
            print(f"FAIL {kind}: {runner} 不可用（PATH/~/.cargo 均缺位）")
            ok = False
        elif cfg["artifact"] and not (REPO / cfg["artifact"]).parent.is_dir():
            print(f"WARN {kind}: 产物目录 {cfg['artifact']} 父级不存在（将自动创建）")
    print("OK  build_artifacts 前置体检通过" if ok else "FAIL 前置体检")
    return 0 if ok else 1


def main() -> int:
    args = [a for a in sys.argv[1:] if a != "--json"]
    if not args:
        print(__doc__.splitlines()[0])
        return 1
    op, kind = args[0], (args[1] if len(args) > 1 else None)
    if op == "pre":
        return cmd_pre(kind or "")
    if op == "post":
        return cmd_post(kind or "")
    if op == "run":
        return cmd_run(kind or "")
    if op == "list":
        return cmd_list()
    if op == "check":
        return cmd_check()
    print(f"未知操作: {op}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
