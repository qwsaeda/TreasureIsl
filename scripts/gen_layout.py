#!/usr/bin/env python3
"""把 ref/controls.json 的控件记录转成 assets/data/controls.json（控件层数据源）。

用法：
    python scripts/gen_layout.py            # 生成/更新 assets/data/controls.json
    python scripts/gen_layout.py --check    # 只校验当前产物 == 期望产物（不写文件）

幂等：同一输入连续跑两次，产物逐字节一致（docs/process/04_tooling.md §5）。
退出码：0 成功；非 0 失败（gate 直接判 FAIL）。

来源字段语义见 docs/facts/map.md（F-306/F-311：f3 = Y 坐标，f5 = 控件类型）。
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

# ref/controls.json 的 f5 控件类型编码（0 = 无类型，1..=44 为具名类型）。
# kind 分布见 ref/controls.json 的 kind_dist 字段（44 种）。
KIND_NAMES: dict[int, str] = {0: "none"}
for _i in range(1, 45):
    KIND_NAMES[_i] = f"kind{_i}"


def load_controls(src: Path) -> dict:
    with src.open(encoding="utf-8") as fh:
        return json.load(fh)


def build_output(src: dict) -> dict:
    """由 ref/controls.json 派生控件数据表（纯函数，便于测试幂等）。"""
    scenes = []
    total = 0
    for sc in src.get("scenes", []):
        records = []
        for rec in sc.get("records", []):
            kind = rec.get("f5", 0)
            records.append(
                {
                    "kind": kind,
                    "kind_name": KIND_NAMES.get(kind, f"kind{kind}"),
                    "x": rec.get("f0", 0),
                    "y": rec.get("f3", 0),  # f3 = Y 坐标（F-311）
                    "w": rec.get("f1", 0),
                    "h": rec.get("f2", 0),
                    "flags": rec.get("f4", 0),
                    "extra": rec.get("f6", 0),
                }
            )
        total += len(records)
        scenes.append(
            {
                "id": sc["name"],
                "width": sc.get("width", 0),
                "height": sc.get("height", 0),
                "count": len(records),
                "records": records,
            }
        )
    return {
        "schema_version": 1,
        "source": "ref/controls.json",
        "scene_count": len(scenes),
        "record_total": total,
        "scenes": scenes,
    }


def serialize(obj: dict) -> str:
    """稳定序列化：sort_keys + 固定分隔 + 末尾换行，保证幂等逐字节一致。"""
    return json.dumps(obj, ensure_ascii=False, indent=2, sort_keys=True) + "\n"


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--root", type=Path, default=Path("."))
    ap.add_argument("--src", type=Path, default=Path("ref/controls.json"))
    ap.add_argument("--out", type=Path, default=Path("assets/data/controls.json"))
    ap.add_argument("--check", action="store_true", help="只校验不改产物")
    args = ap.parse_args()

    src_path = args.root / args.src
    out_path = args.root / args.out
    if not src_path.is_file():
        print(f"ERROR 缺少来源：{src_path}", file=sys.stderr)
        return 1

    out = build_output(load_controls(src_path))
    text = serialize(out)

    if args.check:
        if not out_path.is_file():
            print(f"ERROR 缺少产物：{out_path}", file=sys.stderr)
            return 1
        cur = out_path.read_text(encoding="utf-8")
        if cur != text:
            print(f"ERROR {out_path} 与 {args.src} 派生的产物不一致（跑不带 --check 生成）", file=sys.stderr)
            return 1
        print(f"OK controls.json 一致（{out['record_total']} 记录 / {out['scene_count']} 场景）")
        return 0

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(text, encoding="utf-8", newline="\n")
    print(f"生成 {out_path}（{out['record_total']} 记录 / {out['scene_count']} 场景）")
    return 0


if __name__ == "__main__":
    sys.exit(main())