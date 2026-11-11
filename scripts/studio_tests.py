#!/usr/bin/env python3
"""studio_tests.py — 元测试面（S50.2，CCGS studio-tests 模式借鉴，决策依据见 docs/process/32_ccgs_borrow.md）。

对「门禁断言 / 测试登记 / 工具台账」本身做 QA（测「测代码的测试」），3 类断言：

  1. doccheck2 触发面双向对账
     crates/ti-tools/src/bin/doccheck2.rs 的注册表 `RUST_CHECKS` 引用的每个 (id, fn)
     必须在 `assets/gate_data.json` 的 `doccheck2_checks[]` 有条目（id ↔ 注册表一一对应，
     少登记 = 实现白跑；多登记 = 清单漂移，与 doccheck2 `check_check_inventory_consistency` 同口径）。
     反向：`gate_data.json` 里 `impl: "rust"` 的每个 id 必须在注册表有实现（登记未落地 = FAIL）。

  2. test_manifest ↔ console_commands 映射
     `assets/data/test_manifest.json` 每条登记的 command 首词 ∈ 程序白名单
     （cargo/python3/python/pwsh/powershell，与 31 号 §command 契约同口径；**空 command = 人工验收/
     无命令类测试，合法，跳过**（31 号「人工验收/无命令 = 按钮禁用」））；且该 command 在
     `crates/ti-console/console_commands.json` 有对应 id 映射（缺映射 = 平台「测试」页无法经白名单
     执行该测试 = 平台面未覆盖 = 漂移 FAIL）。存量未映射 = 进 `scripts/studio_tests_waiver.json`
     棘轮基线（豁免，只降不升）；新增测试必映射。

  3. 工具台账三处登记核验
     `scripts/*.py` / `scripts/*.ps1` 每个文件在 ①`docs/process/14_tooling_catalog.md` 表格行
     + ②`docs/process/04_tooling.md` 速查表 + ③`docs/README.md` 索引 三处均 grep 命中（文件名
     出现）。缺任一即 FAIL；存量缺登记 = 进 `scripts/studio_tests_waiver.json` 棘轮基线；
     `scripts/tmp/` 前缀天然豁免（一次性脚本，工具契约 §14 第 1 条）。

工具契约（14 号 §1.2 / doccheck2 第 27 项）：落地 `scripts/` + 14 号 §3.1 表格行 + 04 号速查表
+ README 索引三处登记 + `--check` 只读 + 非 0 退出（3 类断言任一 FAIL = exit 1；前置源文件缺失 = exit 2）。
"""
from __future__ import annotations

import json
import os
import re
import sys

# 仓库根（本脚本在 scripts/ 下，父目录 = 仓库根；兼容从任意 cwd 跑）。
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# test_manifest 每条 command 首词白名单（31 号 §command 契约，与 check_test_manifest 同口径）。
CMD_PROGRAM_WHITELIST = {"cargo", "python3", "python", "pwsh", "powershell"}

# 工具台账三处登记面（14 号表格 / 04 速查 / README 索引）。
REGISTRATION_FACES = [
    ("14_tooling_catalog", "docs/process/14_tooling_catalog.md"),
    ("04_tooling_quick", "docs/process/04_tooling.md"),
    ("README_index", "docs/README.md"),
]

WAIVER_PATH = "scripts/studio_tests_waiver.json"


def _read_json(rel: str):
    with open(os.path.join(ROOT, rel), encoding="utf-8") as f:
        return json.load(f)


def _load_waiver() -> dict:
    """棘轮基线：存量豁免清单（test_manifest 未映射 id / 工具台账缺登记文件名）。
    文件缺失 = 全空基线（新增即 FAIL，存量 0）。"""
    if not os.path.exists(os.path.join(ROOT, WAIVER_PATH)):
        return {"unmapped_test_manifest": [], "unregistered_scripts": []}
    try:
        data = _read_json(WAIVER_PATH)
    except (json.JSONDecodeError, OSError) as e:  # noqa: BLE001 - 读失败不阻断，按全空基线
        print(f"[studio_tests] waiver 文件读失败 {WAIVER_PATH}: {e}", file=sys.stderr)
        return {"unmapped_test_manifest": [], "unregistered_scripts": []}
    return {
        "unmapped_test_manifest": list(data.get("unmapped_test_manifest", [])),
        "unregistered_scripts": list(data.get("unregistered_scripts", [])),
    }


# ---- 断言 1：doccheck2 触发面双向对账 -----------------------------------------

def _rust_check_pairs(src: str) -> dict:
    """从 doccheck2.rs 抽 RUST_CHECKS 注册表的 (id → fn_name) 映射。
    注册表实际形态 = `const RUST_CHECKS: &[(&str, CheckFn)] = &[ ("id", fn), ... ];`
    （`&[` 前带 `&`，须容错）。"""
    m = re.search(
        r"RUST_CHECKS\s*:\s*&\[\(&str,\s*CheckFn\)\]\s*=\s*&?\[\s*(.*?)\s*\];", src, re.DOTALL
    )
    if not m:
        return {}
    return {
        id_str: fn_name
        for id_str, fn_name in re.findall(
            r"\(\s*\"([^\"]+)\"\s*,\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)", m.group(1)
        )
    }


def check_doccheck2_trigger_surface() -> list:
    """RUST_CHECKS 注册表引用 ↔ gate_data.json doccheck2_checks 条目 双向对账。"""
    errors: list[str] = []
    dc2_path = "crates/ti-tools/src/bin/doccheck2.rs"
    if not os.path.exists(os.path.join(ROOT, dc2_path)):
        errors.append(f"前置缺失：{dc2_path} 不存在（exit 2 口径）")
        return errors
    with open(os.path.join(ROOT, dc2_path), encoding="utf-8") as f:
        dc2_src = f.read()

    reg_pairs = _rust_check_pairs(dc2_src)
    if not reg_pairs:
        errors.append("doccheck2.rs RUST_CHECKS 注册表解析为空（提取失败，exit 2 口径）")
        return errors

    gate_data = _read_json("assets/gate_data.json")
    checks = gate_data.get("doccheck2_checks", [])
    if not isinstance(checks, list):
        errors.append("assets/gate_data.json 缺 `doccheck2_checks` 数组（单一事实源缺失）")
        return errors

    by_id = {e.get("id"): e for e in checks if isinstance(e, dict)}

    # 正向：注册表每个 (id,fn) 必须在 gate_data.json 有条目 + 该条 impl = "rust"。
    for id_str, fn_name in reg_pairs.items():
        entry = by_id.get(id_str)
        if entry is None:
            errors.append(
                f"doccheck2 注册表 `{id_str}`（fn {fn_name}）未在 gate_data.json 登记（清单漂移）"
            )
        elif entry.get("impl") != "rust":
            errors.append(
                f"doccheck2 注册表 `{id_str}`（fn {fn_name}）在 gate_data.json "
                f"impl={entry.get('impl')}（注册表是 rust 实现，期望 impl=rust）"
            )

    # 反向：gate_data.json 每个 `impl: rust` 条目必须在注册表有函数实现。
    for entry in checks:
        if not isinstance(entry, dict) or entry.get("impl") != "rust":
            continue
        id_str = entry.get("id")
        if id_str not in reg_pairs:
            errors.append(
                f"gate_data.json doccheck2_checks[{id_str}] impl=rust 但无 Rust 实现（登记未落地）"
            )

    return errors


# ---- 断言 2：test_manifest ↔ console_commands 映射 ---------------------------

def _test_manifest_items() -> list:
    data = _read_json("assets/data/test_manifest.json")
    if isinstance(data, dict):
        for key in ("tests", "items"):
            if key in data and isinstance(data[key], list):
                return data[key]
        if "command" in data:
            return [data]
        return []
    if isinstance(data, list):
        return data
    return []


def _command_mapped_in_console(command: str, cmd_list: list) -> bool:
    """command 是否在 console_commands 任一 entry 有映射。
    匹配口径 = entry 的 `command`/`cmd`/`argv`/(`program`+`args`) 字段含该 command 全文（规范化空格）。
    无法解析映射字段 = 保守判未映射（漂移 FAIL，由 waiver 兜底存量）。"""
    norm = re.sub(r"\s+", " ", command.strip())
    for entry in cmd_list:
        if not isinstance(entry, dict):
            continue
        candidates: list[str] = []
        for field in ("command", "cmd", "argv"):
            v = entry.get(field)
            if isinstance(v, str):
                candidates.append(re.sub(r"\s+", " ", v.strip()))
            elif isinstance(v, list):
                candidates.append(
                    re.sub(r"\s+", " ", " ".join(x for x in v if isinstance(x, str)))
                )
        prog = entry.get("program")
        args = entry.get("args")
        if prog and isinstance(args, list):
            candidates.append(
                re.sub(r"\s+", " ", prog + " " + " ".join(a for a in args if isinstance(a, str)))
            )
        for cand in candidates:
            if cand and (cand == norm or cand in norm or norm in cand):
                return True
    return False


def check_test_manifest_mapping(waiver: dict) -> list:
    errors: list[str] = []
    manifest_path = "assets/data/test_manifest.json"
    if not os.path.exists(os.path.join(ROOT, manifest_path)):
        errors.append(f"前置缺失：{manifest_path} 不存在（exit 2 口径）")
        return errors

    console_path = "crates/ti-console/console_commands.json"
    if not os.path.exists(os.path.join(ROOT, console_path)):
        errors.append(f"前置缺失：{console_path} 不存在（exit 2 口径）")
        return errors

    manifest = _test_manifest_items()
    console = _read_json(console_path)
    cmd_list = console.get("commands", console) if isinstance(console, dict) else console
    if not isinstance(cmd_list, list):
        errors.append("console_commands.json 结构异常（无 commands 数组 / 非 list）")
        return errors

    waiver_unmapped = set(waiver.get("unmapped_test_manifest", []))
    for item in manifest:
        if not isinstance(item, dict):
            continue
        tid = item.get("id", "?")
        command = (item.get("command") or "").strip()
        # 空 command = 人工验收 / 无命令类测试（31 号「人工验收/无命令 = 按钮禁用」），合法，跳过。
        if not command:
            continue
        first = command.split()[0]
        if first not in CMD_PROGRAM_WHITELIST:
            errors.append(
                f"test_manifest[{tid}] command 首词 `{first}` 不在程序白名单 "
                f"{sorted(CMD_PROGRAM_WHITELIST)}"
            )
            continue
        if not _command_mapped_in_console(command, cmd_list):
            if tid in waiver_unmapped:
                continue  # 存量豁免（棘轮基线内，只降不升）
            errors.append(
                f"test_manifest[{tid}] command `{command}` 未在 console_commands.json 映射"
                "（平台「测试」页无法经白名单执行 = 平台面未覆盖；"
                "存量豁免须登记 studio_tests_waiver.json）"
            )
    return errors


# ---- 断言 3：工具台账三处登记核验 ---------------------------------------------

def _scripts() -> list:
    out: list[str] = []
    scripts_dir = os.path.join(ROOT, "scripts")
    if not os.path.isdir(scripts_dir):
        return out
    for name in sorted(os.listdir(scripts_dir)):
        if name.startswith("."):
            continue
        if name == "tmp" or name.startswith("tmp/"):
            continue
        full = os.path.join(scripts_dir, name)
        if os.path.isfile(full) and (name.endswith(".py") or name.endswith(".ps1")):
            out.append(name)
    return out


def check_tooling_registration(waiver: dict) -> list:
    errors: list[str] = []
    face_texts: dict[str, str] = {}
    for label, rel in REGISTRATION_FACES:
        p = os.path.join(ROOT, rel)
        if not os.path.exists(p):
            errors.append(f"前置缺失：登记面 {rel} 不存在（exit 2 口径）")
            face_texts[label] = ""
            continue
        with open(p, encoding="utf-8") as f:
            face_texts[label] = f.read()

    waiver_unreg = set(waiver.get("unregistered_scripts", []))
    for name in _scripts():
        missing = [label for label, _rel in REGISTRATION_FACES if name not in face_texts.get(label, "")]
        if not missing:
            continue
        if name in waiver_unreg:
            continue
        errors.append(
            f"scripts/{name} 缺登记面：{','.join(missing)}"
            "（须三处全登记，存量豁免须入 studio_tests_waiver.json）"
        )
    return errors


def main() -> int:
    waiver = _load_waiver()
    e1 = check_doccheck2_trigger_surface()
    e2 = check_test_manifest_mapping(waiver)
    e3 = check_tooling_registration(waiver)
    all_errors: list[str] = []
    all_errors.extend(e1)
    all_errors.extend(e2)
    all_errors.extend(e3)

    print("[studio_tests] 元测试面 3 类断言（S50.2）")
    print(f"  1. doccheck2 触发面双向对账            : {'OK' if not e1 else str(len(e1)) + ' error(s)'}")
    print(f"  2. test_manifest↔console_commands 映射 : {'OK' if not e2 else str(len(e2)) + ' error(s)'}")
    print(f"  3. 工具台账三处登记核验                : {'OK' if not e3 else str(len(e3)) + ' error(s)'}")

    has_missing = any("前置缺失" in e for e in all_errors)
    if has_missing:
        for e in all_errors:
            print(f"    MISSING: {e}")
        print("[studio_tests] 前置源文件缺失（exit 2）")
        return 2

    if all_errors:
        for e in all_errors:
            print(f"    FAIL: {e}")
        print(f"[studio_tests] {len(all_errors)} error(s)（非 0 退出，exit 1）")
        return 1

    print("[studio_tests] 3 类断言全 OK（exit 0）")
    return 0


if __name__ == "__main__":
    # 工具契约：`--check` 只读别名（本脚本全量即只读，`--check` 无副作用同义）。
    if "--help" in sys.argv:
        print(__doc__)
        sys.exit(0)
    sys.exit(main())
