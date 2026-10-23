#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""任务收口一条命令：跑判据 → 写 evidence → 置 verified → 同步台账计数 → 三方对账复验。

**为什么有这个工具**（`docs/process/15_process_ledger.md` 第 1 条流程）：
硬规则 2 要求「实跑输出写入 `evidence/<task>.log` 后，才可在台账置 `verified`」，但这三步
过去全是手工 ⇒ 必然复发「先 verified 后补 evidence」「改了状态忘同步 `release.md` 计数」
「手改整份台账把别人的行冲掉」。本工具把它们收成**一条命令**，且：

- **行级改写**：只重写目标行（`status` / `evidence` / `rounds`），不整份重写台账（EXP-065）；
- **证据先于状态**：判据命令非 0 ⇒ **不写台账、不改状态**（虚报防线在工具内）；
- **计数同步**：`release.md` 的「N 任务 / M verified」从台账**实测**回写（不是手抄）；
- **收尾复验**：跑 `scripts/check_ledger_reality.py`（verified ↔ evidence ↔ 代码符号 三方对账）；
- **UTF-8 写盘**（`encoding='utf-8', newline=''`）：禁 `Set-Content`/`Out-File` 写 CJK（EXP-111）。

用法：
    python scripts/task_close.py --task S21.4 --cmd "cargo test --workspace"
    python scripts/task_close.py --task S21.4 --cmd "cargo test --workspace" --cmd "python scripts/validate_data.py"
    python scripts/task_close.py --task S21.4 --no-run          # 证据已在，只翻台账 + 同步计数
    python scripts/task_close.py --task S21.4 --check            # 只读体检（不改任何文件）

退出码：0 = 成功；1 = 判据失败 / 台账无此任务 / 对账失败。
幂等：同一 `--task` 重复跑 = 重写证据 + 幂等改状态（`verified` 已置则不重复加 rounds）。
"""
from __future__ import annotations

import argparse
import datetime as _dt
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
LEDGER = ROOT / "plan" / "ledger.yaml"
RELEASE = ROOT / "docs" / "release.md"
EVIDENCE_DIR = ROOT / "evidence"

TASK_RE = re.compile(r"^\s*-\s*\{\s*id:\s*(?P<id>[A-Za-z0-9_.]+)\s*,")


def read_ledger_tasks() -> list[tuple[int, str, str]]:
    """返回 (行号, 任务 id, 整行)。台账是行式 flow-mapping，故按行解析（零第三方依赖）。"""
    if not LEDGER.is_file():
        print(f"台账不存在: {LEDGER}", file=sys.stderr)
        raise SystemExit(1)
    out: list[tuple[int, str, str]] = []
    for i, line in enumerate(LEDGER.read_text(encoding="utf-8").splitlines()):
        m = TASK_RE.match(line)
        if m:
            out.append((i, m.group("id"), line))
    return out


def task_counts() -> tuple[int, int]:
    tasks = read_ledger_tasks()
    verified = sum(1 for _, _, line in tasks if "status: verified" in line)
    return len(tasks), verified


def find_task(task_id: str) -> tuple[int, str] | None:
    for i, tid, line in read_ledger_tasks():
        if tid == task_id:
            return i, line
    return None


def write_text(path: pathlib.Path, text: str) -> None:
    """UTF-8 无 BOM + 原换行（禁 cmdlet 写仓库文件，EXP-111）。"""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8", newline="")


def run_command(cmd: str) -> tuple[int, str]:
    """跑一条判据命令，捕获合并输出（UTF-8，容错解码）。"""
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        shell=True,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
    )
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def build_evidence(task_id: str, line: str, cmds: list[str], outputs: list[tuple[str, int, str]]) -> str:
    title = ""
    m = re.search(r'title:\s*"(.*?)"\s*,', line)
    if m:
        title = m.group(1)
    dod = ""
    m = re.search(r"dod:\s*\"(.*)\"\s*,", line)
    if m:
        dod = m.group(1)
    parts = [
        f"# {task_id} 验收证据（task_close.py 自动生成 { _dt.date.today().isoformat() }）",
        "",
        f"- 任务标题：{title}",
        f"- 判据（台账 dod）：{dod}",
        "",
        "## 判据实跑",
        "",
    ]
    for cmd, code, out in outputs:
        parts += [f"### `$ {cmd}`", "", f"退出码：{code}", "", "```", out.rstrip(), "```", ""]
    parts += [
        "---",
        "",
        "（本文件由 `scripts/task_close.py` 生成：证据先于状态，判据非 0 不改台账）",
    ]
    return "\n".join(parts) + "\n"


def flip_status(line: str, evidence: str) -> tuple[str, bool]:
    """行级改写：status → verified、evidence → 路径、rounds += 1。返回 (新行, 是否变化)。"""
    new = line
    changed = False
    if not re.search(r"status:\s*verified", new):
        new = re.sub(r"status:\s*(pending|in_progress|blocked)", "status: verified", new, count=1)
        changed = True
    if f'evidence: "{evidence}"' not in new:
        new = re.sub(r"evidence:\s*(null|\"[^\"]*\")", f'evidence: "{evidence}"', new, count=1)
        changed = True
    m = re.search(r"rounds:\s*(\d+)", new)
    if m and changed:
        new = new[: m.start()] + f"rounds: {int(m.group(1)) + 1}" + new[m.end() :]
    return new, changed


def sync_release_counts() -> bool:
    """把 release.md 的「N 任务（M verified；…」从台账实测回写。"""
    if not RELEASE.is_file():
        return False
    total, verified = task_counts()
    text = RELEASE.read_text(encoding="utf-8")
    pat = re.compile(r"(`plan/ledger\.yaml`\s*)(\d+)\s*任务\s*（(\d+)\s*verified")
    m = pat.search(text)
    if not m:
        print("  WARN release.md 未找到台账计数字样（未同步）", file=sys.stderr)
        return False
    if m.group(2) == str(total) and m.group(3) == str(verified):
        return False
    text = pat.sub(lambda mm: f"{mm.group(1)}{total} 任务（{verified} verified", text, count=1)
    write_text(RELEASE, text)
    print(f"  release.md 计数已同步: {total} 任务 / {verified} verified")
    return True


def main() -> int:
    ap = argparse.ArgumentParser(description="任务收口：跑判据 → 写证据 → 置 verified → 同步计数 → 对账")
    ap.add_argument("--task", required=True, help="台账任务 id（如 S21.4）")
    ap.add_argument("--cmd", action="append", default=[], help="判据命令（可重复，按序实跑）")
    ap.add_argument("--no-run", action="store_true", help="不跑判据（证据已存在时用）")
    ap.add_argument("--check", action="store_true", help="只读体检：查任务存在 + 证据状态，不改文件")
    ap.add_argument("--skip-counter", action="store_true", help="不跑 check_ledger_reality.py")
    args = ap.parse_args()

    found = find_task(args.task)
    if found is None:
        print(f"台账无此任务: {args.task}", file=sys.stderr)
        return 1
    line_no, line = found
    ev_path = f"evidence/{args.task}.log"
    ev_abs = ROOT / ev_path

    if args.check:
        total, verified = task_counts()
        status = "verified" if "status: verified" in line else "pending"
        has_ev = ev_abs.is_file() and ev_abs.stat().st_size > 0
        print(f"  任务 {args.task}：status={status} 证据={ev_path}{'存在' if has_ev else '缺失'}")
        print(f"  台账计数：{total} 任务 / {verified} verified")
        return 0

    outputs: list[tuple[str, int, str]] = []
    if not args.no_run:
        if not args.cmd:
            print("未给 --cmd 且未给 --no-run：要么给判据命令，要么确认证据已在", file=sys.stderr)
            return 1
        for cmd in args.cmd:
            print(f"  跑判据: {cmd}")
            code, out = run_command(cmd)
            outputs.append((cmd, code, out))
            print(f"    退出码 {code}")
        failed = [c for c, code, _ in outputs if code != 0]
        if failed:
            print(f"判据失败（{failed}）⇒ 不写证据、不改台账", file=sys.stderr)
            for cmd, code, out in outputs:
                print(f"--- $ {cmd} (exit {code}) ---\n{out[-2000:]}")
            return 1
        write_text(ev_abs, build_evidence(args.task, line, args.cmd, outputs))
        print(f"  证据已写: {ev_path}")
    elif not ev_abs.is_file():
        print(f"--no-run 但证据不存在: {ev_path}", file=sys.stderr)
        return 1

    # 行级改写台账（只动目标行）
    lines = LEDGER.read_text(encoding="utf-8").splitlines()
    new_line, changed = flip_status(lines[line_no], ev_path)
    if changed:
        lines[line_no] = new_line
        write_text(LEDGER, "\n".join(lines) + "\n")
        print(f"  台账已更新: {args.task} → verified（行 {line_no + 1}，evidence={ev_path}）")
    else:
        print(f"  台账已是 verified，未重复改写: {args.task}")

    sync_release_counts()

    if not args.skip_counter:
        proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "check_ledger_reality.py")],
            cwd=ROOT,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
        )
        print(f"  三方对账: {proc.stdout.strip() or proc.stderr.strip()}")
        if proc.returncode != 0:
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
