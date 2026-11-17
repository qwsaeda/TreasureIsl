#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""gate.ps1 完整描述串去重 1-based 位表回写（`assets/gate_item_posmap.json`）。

**为什么有这个工具**（`docs/process/15_process_ledger.md` 第 12 条流程 + EXP-194 载体）：
gate 项号文字（「第 N 项/位 <gate项名>」）全库残留高频复现（EXP-189 同类第 3 次，R15 实抓 9 处）
⇒ 机读源 = gate.ps1 Report 调用去重保序（与 doccheck2::gate_item_actual_count 同口径），
文档文字「第 N 项」是手工镜像。本工具**从 gate.ps1 实测**后回写 `assets/gate_item_posmap.json`，
供 doccheck2 第 51 项 `gate-item-number-text` 断言消费（声称点 ↔ posmap 对账）。

用法：
    python3 scripts/gate_item_posmap.py            # 只读体检（posmap 与 gate.ps1 实测一致则 0）
    python3 scripts/gate_item_posmap.py --fix      # 实测回写 posmap（gate 加减项后重跑）

退出码：0 = 一致（或 --fix 后已一致）；1 = 漂移且未 --fix。
幂等：--fix 连跑两次 = 第二次 0 改动。
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
GATE = ROOT / "scripts" / "gate.ps1"
POSMAP = ROOT / "assets" / "gate_item_posmap.json"

# Report 调用两种形态：`Report (<expr>) 'name'` 与 `Report $false 'name'`；name 捕获 = 组 1


def gate_report_names() -> list[str]:
    text = GATE.read_text(encoding="utf-8")
    names: list[str] = []
    seen: set[str] = set()
    import re as _re
    for line in text.splitlines():
        m = _re.search(r"Report\s+(?:\([^)]*\)|\$\w+)\s+(['\"])(.+?)\1", line)
        if m:
            n = m.group(2).strip()
            if n and n not in seen:
                seen.add(n)
                names.append(n)
    return names


def main() -> int:
    ap = argparse.ArgumentParser(description="gate.ps1 去重 1-based 位表实测回写（EXP-194 载体）")
    ap.add_argument("--fix", action="store_true", help="实测回写 posmap（gate 加减项后）")
    ap.add_argument("--check", action="store_true", help="只读体检别名（默认即只读；--check 显式声明工具契约 1.2）")
    args = ap.parse_args()

    names = gate_report_names()
    items = [
        {"short": n.split(" (")[0].strip(), "pos": i}
        for i, n in enumerate(names, 1)
    ]
    payload = {
        "_comment": "gate.ps1 完整描述串去重 1-based 位表（机读源；gate 加减项须重跑 scripts/gate_item_posmap.py --fix 回写）",
        "items": items,
    }
    current = json.loads(POSMAP.read_text(encoding="utf-8")) if POSMAP.is_file() else {"items": []}
    drift = current.get("items", []) != items
    print(f"  实测：gate.ps1 去重 {len(names)} 项；posmap 现行 {len(current.get('items', []))} 项")
    if not drift:
        print("  posmap 与 gate.ps1 实测一致（0 漂移）")
        return 0
    if args.fix:
        import json as _j
        item_lines = [
            " " + _j.dumps(it, ensure_ascii=False) for it in items
        ]
        body = (
            "{\n \"_comment\": \"" + payload["_comment"] + "\"\n \"items\": [\n"
            + ",\n".join(item_lines)
            + "\n ]\n}\n"
        )
        POSMAP.write_text(body, encoding="utf-8", newline="")
        print(f"  已回写 posmap（{len(items)} 项）")
        return 0
    print("  posmap 漂移（用 --fix 回写）", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
