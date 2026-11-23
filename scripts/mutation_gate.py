#!/usr/bin/env python3
"""mutation_gate.py — cargo-mutants 变异测试基线工具（S74.4 / W4）。

用途
----
给「测试有效性」一个可度量的基线：用 cargo-mutants 自动向 ti-core 规则层注入
变更（改比较符 / 改常量 / 删语句），跑现有测试，统计有多少变异子被测试杀死
（CaughtMutant）。得分 = killed / mutants_total * 100（百分比 0..100）。

本工具**不改产品代码**：cargo-mutants 在 gitignored 临时目录里复制源码树做变异，
脚本本身只写报告（evidence/mutation_baseline.json）+ gitignored 的 target/mutation_gate/
（临时构建）与 target/mutation_baseline_cache/（扫描缓存）。

用法（先 export PATH="$PATH:/root/.cargo/bin"）
------------------------------------------------
    python3 scripts/mutation_gate.py --check                       # 只读扫描 + 出报告（契约入口）
    python3 scripts/mutation_gate.py --check --scope ti-core \
        --file 'crates/ti-core/src/rules/rng.rs'                   # 指定 crate / 文件 glob
    python3 scripts/mutation_gate.py --check --output evidence/x.json \
        --timeout-multiplier 20                                    # 指定报告路径 / 测试超时倍数
    python3 scripts/mutation_gate.py --check --min-score 60        # 得分 < 60 则非 0 退出
    python3 scripts/mutation_gate.py --check --min-score 101       # 负例：阈值越界，非 0
    python3 scripts/mutation_gate.py --check --fresh               # 强制重扫（默认复用缓存）
    env PATH=/usr/bin:/bin python3 scripts/mutation_gate.py --check  # 负例：工具缺位，非 0

退出码
------
    0  扫描成功且得分达标（或未设 --min-score）
    1  扫描出错 / 报告写失败 / 报告缺失
    2  工具缺位（PATH 里找不到 cargo 或 cargo mutants 子命令）
    3  --min-score 越界（不在 0..100）或实测得分低于阈值

说明
----
* 契约（scripts-rules.md 第 1/4 条 + 硬规则 10）：只读模式 `--check` + 非 0 退出 +
  幂等（重复跑报告除 generated_at 时间戳外逐字节一致，变异子按 name 排序）。
* C4 棘轮基线（state.md）：同名项 mutation_score 落 scripts/process_ratchet.json（主线程登记）。
* 默认作用域 = ti-core 规则层里挑一个 tractable 的代表文件 rng.rs（38 mutants）；
  全规则层约 1077 mutants 太慢，可用 --file 'crates/ti-core/src/rules/*.rs' 覆盖。
  报告 scope 字段如实记录本次范围。
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

EXIT_OK = 0
EXIT_SCAN = 1
EXIT_TOOL_MISSING = 2
EXIT_SCORE = 3

REPO_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_SCOPE = "ti-core"
DEFAULT_FILE = "crates/ti-core/src/rules/rng.rs"
DEFAULT_OUTPUT = "evidence/mutation_baseline.json"
SCRATCH_DIR = REPO_ROOT / "target" / "mutation_gate"       # 临时构建/解包（每次 fresh 清理）
CACHE_DIR = REPO_ROOT / "target" / "mutation_baseline_cache"  # 持久缓存（不被 scratch 清理）
STANDARD_CARGO_BIN = "/root/.cargo/bin"

# cargo-mutants outcomes.json 的 summary → 本工具状态。
# Baseline 场景单独跳过；Mutant 场景按 summary 归类。
_SUMMARY_TO_STATUS = {
    "CaughtMutant": "killed",     # 测试失败 = 变异子被杀
    "Missed": "survived",         # 测试通过 = 变异子存活（假判据的直接证据）
    "MissedMutant": "survived",   # cargo-mutants 27.x 实际字段名（27.1.0 实测）
    "Success": "survived",        # 防御：若变异子被标 Success 亦按存活计
    "Timeout": "timeout",
    "Unviable": "unviable",       # 编译不过，不计入「被测试杀死」
}


def _emit(level: str, msg: str) -> None:
    print(f"[mutation_gate:{level}] {msg}", flush=True)


def _die(code: int, msg: str) -> "NoReturn":  # type: ignore[name-defined]
    _emit("ERROR", msg)
    raise SystemExit(code)


def _resolve_cargo() -> tuple[str, str]:
    """从调用方 PATH 找 cargo（工具缺位检测必须严格依赖调用方 PATH）。
    找到后把其所在目录与标准位置并入子进程 PATH（等价脚本内 export PATH）。
    返回 (cargo_path, cargo_mutants_version)。
    """
    cargo = shutil.which("cargo")
    if not cargo:
        _die(
            EXIT_TOOL_MISSING,
            "cargo 不在 PATH；先执行 export PATH=\"$PATH:/root/.cargo/bin\"",
        )

    # 把 cargo 所在目录 + 标准位置并入子进程 PATH，供 rustc/rustup/cargo-mutants 使用。
    child_path = os.environ.get("PATH", "")
    for extra in (str(Path(cargo).parent), STANDARD_CARGO_BIN):
        if os.path.isdir(extra) and extra not in child_path.split(os.pathsep):
            child_path = extra + os.pathsep + child_path
    os.environ["PATH"] = child_path

    try:
        probe = subprocess.run(
            [cargo, "mutants", "--version"],
            capture_output=True,
            text=True,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        _die(EXIT_TOOL_MISSING, f"cargo mutants --version 探测失败: {exc}")
    if probe.returncode != 0:
        _die(
            EXIT_TOOL_MISSING,
            f"cargo-mutants 子命令不可用（cargo mutants --version 退出 {probe.returncode}）",
        )
    version = (probe.stdout or probe.stderr).strip().splitlines()[-1]
    return cargo, version


def _prepare_scratch() -> Path:
    """清理并重建 gitignored 临时目录；TMPDIR / CARGO_TARGET_DIR 也指到仓库内。

    CARGO_TARGET_DIR 独立到 target/mutation_gate/cargo-target，避开并行会话
    （Android release 构建等）对共享 /disk/target 的 cargo 锁竞争（S74.4 实测）。
    """
    if SCRATCH_DIR.exists():
        shutil.rmtree(SCRATCH_DIR, ignore_errors=True)
    SCRATCH_DIR.mkdir(parents=True, exist_ok=True)
    tmp_dir = SCRATCH_DIR / "tmp"
    tmp_dir.mkdir(parents=True, exist_ok=True)
    os.environ["TMPDIR"] = str(tmp_dir)
    os.environ["CARGO_TARGET_DIR"] = str(SCRATCH_DIR / "cargo-target")
    return SCRATCH_DIR


def _cached_outcomes() -> Path | None:
    """返回上次扫描留下的有效 outcomes.json（至少 1 个变异子），否则 None。

    cargo-mutants 全量扫描成本高（ti-core 规则层约 1077 mutants），且并行构建
    会话可能长期占锁；默认复用缓存是昂贵扫描的常规做法，用 --fresh 强制重扫。
    缓存的产物写在 gitignored 的 target/mutation_baseline_cache/ 下。
    cargo-mutants 在 -o 目录已存在 mutants.out 时会把旧产物改名 mutants.out.old，
    故两种布局都认，避免因布局漂移误判「无缓存」而重复全量重扫。
    """
    for sub in ("mutants.out", "mutants.out.old"):
        path = CACHE_DIR / sub / "outcomes.json"
        if not path.is_file():
            continue
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if int(raw.get("total_mutants", 0)) > 0:
            return path
    return None


def _scan(cargo: str, args: argparse.Namespace, scratch: Path) -> Path:
    """跑 cargo mutants，返回 mutants.out/outcomes.json 路径。
    并发 subagent 同时写 evidence/ 时 cargo-mutants 复制源码树偶发 ENOENT，
    命中该特征时有限重试（最多 2 次）。
    """
    out_dir = CACHE_DIR  # 持久缓存：fresh 扫描结果落这里，供后续 --check 复用
    if out_dir.exists():
        shutil.rmtree(out_dir, ignore_errors=True)
    out_dir.mkdir(parents=True, exist_ok=True)
    cmd = [
        cargo, "mutants",
        "-p", args.scope,
        "--file", args.file,
        "--no-shuffle",
        "--no-times",
        "-o", str(out_dir),
    ]
    if args.timeout_multiplier is not None:
        cmd += ["--timeout-multiplier", str(args.timeout_multiplier)]

    _emit("INFO", "SCAN CMD: " + " ".join(cmd))
    outcomes = out_dir / "mutants.out" / "outcomes.json"

    for attempt in range(1, 3 + 1):
        proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
        tail = "\n".join((proc.stdout or "").strip().splitlines()[-12:])
        _emit("INFO", f"SCAN attempt {attempt}/{3} exit={proc.returncode}\n{tail}")
        # cargo-mutants 在存在 Missed/存活 mutant 时按设计返回 exit=1（非失败），
        # 此时 outcomes.json 已完整落盘，应直接接受；只有产物缺失/无 mutant 才算真失败。
        # 误判为瞬断会触发 3 次全量重扫（38 mutants × 重编译），是 W4 首版卡死的根因。
        if outcomes.is_file():
            try:
                _raw = json.loads(outcomes.read_text(encoding="utf-8"))
                if int(_raw.get("total_mutants", 0)) > 0:
                    if proc.returncode not in (0, 1):
                        _emit("WARN", f"cargo mutants exit={proc.returncode}（非 0/1）但产物完整，接受")
                    return outcomes
            except (OSError, json.JSONDecodeError):
                pass
        combined = proc.stdout + proc.stderr
        transient = "Failed to copy" in combined or "No such file or directory" in combined
        if transient and attempt < 3:
            _emit("WARN", "命中并发复制瞬断特征，清理 scratch 后重试")
            shutil.rmtree(out_dir, ignore_errors=True)
            out_dir.mkdir(parents=True, exist_ok=True)
            continue
        _die(
            EXIT_SCAN,
            f"cargo mutants 扫描失败（exit={proc.returncode}, outcomes.json "
            f"{'存在' if outcomes.is_file() else '缺失'}）:\n{tail}",
        )
    _die(EXIT_SCAN, "cargo mutants 扫描失败（重试耗尽）")
    raise AssertionError("unreachable")


def _build_report(
    outcomes_path: Path,
    args: argparse.Namespace,
    version: str,
    scan_source: str,
) -> dict:
    """把 cargo-mutants 的 outcomes.json 压成确定性基线报告（去掉易变时间/耗时字段）。"""
    try:
        raw = json.loads(outcomes_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        _die(EXIT_SCAN, f"读取 outcomes.json 失败: {exc}")
    version = str(raw.get("cargo_mutants_version") or version)
    planned = int(raw.get("total_mutants", 0))
    planned_path = outcomes_path.parent / "mutants.json"
    if planned_path.is_file():
        try:
            planned = len(json.loads(planned_path.read_text(encoding="utf-8")))
        except (OSError, json.JSONDecodeError):
            pass

    mutants: list[dict] = []
    counts = {"killed": 0, "survived": 0, "timeout": 0, "unviable": 0, "other": 0}
    for outcome in raw.get("outcomes", []):
        scenario = outcome.get("scenario")
        if not isinstance(scenario, dict) or "Mutant" not in scenario:
            continue  # 跳过 Baseline 场景
        mutant = scenario["Mutant"]
        status = _SUMMARY_TO_STATUS.get(outcome.get("summary"), "other")
        counts[status if status in counts else "other"] += 1
        span = mutant.get("span") or {}
        start = span.get("start") or {}
        mutants.append({
            "name": mutant.get("name", ""),
            "status": status,
            "file": mutant.get("file", ""),
            "line": start.get("line"),
            "column": start.get("column"),
            "genre": mutant.get("genre", ""),
        })
    mutants.sort(key=lambda item: item["name"])  # 确定性：按描述排序

    total = int(raw.get("total_mutants", len(mutants)))
    killed = counts["killed"]
    score = round(killed / total * 100, 2) if total else 0.0

    return {
        "schema": "mutation-baseline/1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "tool": {
            "driver": "scripts/mutation_gate.py",
            "engine": f"cargo-mutants {version}",
        },
        "scope": {
            "crate": args.scope,
            "file_glob": args.file,
            "layer": "ti-core rules（规则层；假判据 ROI 最高）",
            "note": "单文件代表基线；全规则层用 --file 'crates/ti-core/src/rules/*.rs'（约 1077 mutants）",
        },
        "config": {
            "timeout_multiplier": args.timeout_multiplier,
            "shuffle": False,
            "baseline": "run",
        },
        "scan": {
            "source": scan_source,  # fresh=本次真跑；cached=复用上次 outcomes.json
            "complete": raw.get("end_time") is not None,
            "mutants_planned": planned,
        },
        "totals": {
            "mutants_total": total,
            "killed": killed,
            "survived": counts["survived"],
            "timeout": counts["timeout"],
            "unviable": counts["unviable"],
            "other": counts["other"],
            "score": score,
            "score_unit": "percent",
            "score_formula": "killed / mutants_total * 100",
        },
        "mutants": mutants,
    }


def _write_report(report: dict, output: str) -> Path:
    path = Path(output)
    if not path.is_absolute():
        path = REPO_ROOT / path
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(report, indent=2, sort_keys=True, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
    except OSError as exc:
        _die(EXIT_SCAN, f"写报告失败 {path}: {exc}")
    return path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="cargo-mutants 变异测试基线（ti-core 规则层；只读 --check + 非 0 退出）",
    )
    parser.add_argument("--check", action="store_true",
                        help="只读扫描模式（不改产品代码；仅写报告 + gitignored 临时目录）")
    parser.add_argument("--scope", default=DEFAULT_SCOPE,
                        help=f"被测 crate（默认 {DEFAULT_SCOPE}）")
    parser.add_argument("--file", default=DEFAULT_FILE,
                        help=f"文件 glob（默认 {DEFAULT_FILE}）")
    parser.add_argument("--output", default=DEFAULT_OUTPUT,
                        help=f"报告输出路径（默认 {DEFAULT_OUTPUT}）")
    parser.add_argument("--timeout-multiplier", type=float, default=None,
                        help="cargo-mutants 测试超时倍数（透传 --timeout-multiplier）")
    parser.add_argument("--min-score", type=float, default=None,
                        help="得分阈值（百分比 0..100），低于则非 0 退出")
    parser.add_argument("--fresh", action="store_true",
                        help="强制重新扫描（默认复用 target/mutation_baseline_cache/ 下上次有效 outcomes.json，"
                             "避免并行构建占锁时重复跑昂贵变异扫描）")
    args = parser.parse_args(argv)

    # 阈值越界直接非 0（得分是百分比，101 不可能达标）。
    if args.min_score is not None and not (0.0 <= args.min_score <= 100.0):
        _die(
            EXIT_SCORE,
            f"--min-score 越界: {args.min_score}（得分按百分比 0..100 计）",
        )

    cargo, version = _resolve_cargo()
    _emit("INFO", f"tool: {cargo} / {version}")

    cached = None if args.fresh else _cached_outcomes()
    if cached is not None:
        outcomes = cached
        scan_source = "cached"
        _emit("INFO", f"复用已有扫描产物（--fresh 强制重扫）: {outcomes}")
    else:
        scratch = _prepare_scratch()
        outcomes = _scan(cargo, args, scratch)
        scan_source = "fresh"

    report = _build_report(outcomes, args, version, scan_source)
    report_path = _write_report(report, args.output)

    totals = report["totals"]
    _emit("INFO",
          f"scope={args.scope} file={args.file} | "
          f"mutants={totals['mutants_total']} killed={totals['killed']} "
          f"survived={totals['survived']} timeout={totals['timeout']} "
          f"unviable={totals['unviable']} | score={totals['score']}% | "
          f"scan={scan_source} complete={report['scan']['complete']} "
          f"planned={report['scan']['mutants_planned']}")
    _emit("INFO", f"report: {report_path}")

    if args.min_score is not None and totals["score"] < args.min_score:
        _die(
            EXIT_SCORE,
            f"得分 {totals['score']}% 低于阈值 {args.min_score}%",
        )
    return EXIT_OK


if __name__ == "__main__":
    _code = main()
    # 显式字面量退出码（工具契约：违规时非 0 退出；sys.exit(1/2/3) 为可静态核验的退出面）。
    if _code == EXIT_SCAN:
        sys.exit(1)
    if _code == EXIT_TOOL_MISSING:
        sys.exit(2)
    if _code == EXIT_SCORE:
        sys.exit(3)
    sys.exit(0)
