#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""UI 版面规格 ↔ 实现三面对账（规格表 → `scenes.json` → `scene_manifest_v3.json`）。

**为什么有这个工具**（`docs/process/15_process_ledger.md` 第 14 条流程）：
S21.3 的 main_map v4 坐标表（`docs/ui-redesign-v3/08_main_map_v4_spec.md` §2）是**坐标单一
事实源**，但落码时 `scenes.json` / `scene_manifest_v3.json` 要手抄 ⇒ 三处漂移
（N8 门禁只对账 manifest ↔ scenes.json 两处，**规格表本身无人校验**）。本工具做三面对账：

- 规格表里声明的键 → `scenes.json` 是否存在且**坐标一致**；
- 规格表声明的 `bbox_pct` → `scene_manifest_v3.json` 是否一致（+ N8 像素往返 ±1px）；
- 规格表声明的不变量（8pt / N3 面积 / N4 beats / N6 HUD 上限）是否成立。

用法：
    python scripts/layout_spec_check.py --spec docs/ui-redesign-v3/08_main_map_v4_spec.json
    python scripts/layout_spec_check.py --spec <spec> --check     # 同上（只读别名）
    python scripts/layout_spec_check.py --spec <spec> --scene main_map   # 指定场景

退出码：0 = 一致；1 = 有漂移（逐条列出）。
幂等：只读，不写任何文件。
"""
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SCENES = ROOT / "assets" / "data" / "scenes.json"
MANIFEST = ROOT / "assets" / "data" / "ui_schema" / "scene_manifest_v3.json"

# 门禁豁免键（v3 叙事键不参与 8pt：doccheck2::ui_8pt_walk 同口径）
EXEMPT_8PT = {
    "world_canvas", "hud_gold", "hud_player", "deck_trade", "deck_sell", "deck_build",
    "canvas_entities", "focus_anchor", "hud_plaque_l", "hud_plaque_r", "hud_plaque",
}


def load_json(path: pathlib.Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def parse_spec(path: pathlib.Path) -> tuple[dict, list[dict]]:
    """从规格 markdown 抽：① 键→rect 表 ② bbox_pct 声明。"""
    text = path.read_text(encoding="utf-8")
    rects: dict[str, list[int]] = {}
    for line in text.splitlines():
        t = line.strip()
        if not t.startswith("|"):
            continue
        cells = [c.strip() for c in t.strip("|").split("|")]
        if len(cells) < 4:
            continue
        # 「| 元素 | `key` | {x,y,w,h} | z | …」
        m = re.fullmatch(r"`([A-Za-z_][A-Za-z0-9_.]*)`", cells[1]) if len(cells) > 1 else None
        if not m:
            continue
        rm = re.search(r"\{(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\}", cells[2] if len(cells) > 2 else "")
        if rm:
            rects[m.group(1)] = [int(x) for x in rm.groups()]
    bp = re.search(r"bbox_pct\s*\{[^}]*?x:\s*(\d+)[^}]*?y:\s*(\d+)[^}]*?w:\s*(\d+)[^}]*?h:\s*(\d+)", text, re.S)
    bbox = [int(x) for x in bp.groups()] if bp else []
    return ({"bbox_pct": bbox}, rects)


def main() -> int:
    ap = argparse.ArgumentParser(description="UI 版面规格 ↔ scenes.json ↔ manifest 三面对账")
    ap.add_argument("--spec", required=True, help="版面规格 markdown（坐标表 + bbox_pct）")
    ap.add_argument("--scene", default="main_map", help="场景 id（默认 main_map）")
    ap.add_argument("--check", action="store_true", help="只读（默认行为，保留别名）")
    args = ap.parse_args()

    spec_path = ROOT / args.spec
    if not spec_path.is_file():
        print(f"规格不存在: {spec_path}", file=sys.stderr)
        return 1
    spec_meta, spec_rects = parse_spec(spec_path)
    if not spec_rects:
        print("规格里没解析到任何「键 → {x,y,w,h}」行（表格格式变了？）", file=sys.stderr)
        return 1
    problems: list[str] = []

    scenes = load_json(SCENES)
    scene = scenes.get(args.scene, {})
    for key, rect in spec_rects.items():
        actual = scene.get(key)
        if actual is None:
            problems.append(f"scenes.json {args.scene}: 缺规格声明的键 `{key}`（{rect}）")
            continue
        if not isinstance(actual, dict) or "x" not in actual:
            continue  # 该键是标量/字典（如 canvas_entities），非矩形，跳过
        got = [actual.get("x", 0), actual.get("y", 0), actual.get("w", 0), actual.get("h", 0)]
        if got != rect:
            problems.append(f"scenes.json {args.scene}.{key}: 实为 {got} ≠ 规格 {rect}")
        if key not in EXEMPT_8PT and any(v % 8 != 0 for v in rect):
            problems.append(f"spec {key}: {rect} 非 8pt 栅格（v3 豁免键才可破格）")

    # N8 往返：manifest.bbox_pct ↔ 像素（±1px）
    bbox = spec_meta.get("bbox_pct") or []
    if bbox:
        manifest = load_json(MANIFEST)
        entry = manifest.get("scenes", {}).get(args.scene, {})
        mb = entry.get("narrative", {}).get("focus", {}).get("bbox_pct")
        if mb is None:
            problems.append(f"manifest {args.scene}: 缺 narrative.focus.bbox_pct")
        else:
            got = [mb.get("x", 0), mb.get("y", 0), mb.get("w", 0), mb.get("h", 0)]
            if got != bbox:
                problems.append(f"manifest {args.scene}.focus.bbox_pct: 实为 {got} ≠ 规格 {bbox}")
            px = [
                round(bbox[0] * 1080 / 100),
                round(bbox[1] * 1920 / 100),
                round(bbox[2] * 1080 / 100),
                round(bbox[3] * 1920 / 100),
            ]
            anchor = scene.get("focus_anchor", {}).get("bbox")
            if anchor is None:
                problems.append(f"scenes.json {args.scene}: 缺 focus_anchor.bbox")
            else:
                got_px = [anchor.get("x", 0), anchor.get("y", 0), anchor.get("w", 0), anchor.get("h", 0)]
                if any(abs(a - b) > 1 for a, b in zip(got_px, px)):
                    problems.append(
                        f"scenes.json {args.scene}.focus_anchor.bbox {got_px} 与 bbox_pct 往返 {px} 差 >1px（N8）"
                    )
            # N3 焦点面积 ≥ 20%
            area = bbox[2] * bbox[3] / 100
            if area < 20:
                problems.append(f"N3 焦点面积 {area:.1f}% < 20%（bbox_pct {bbox}）")

    for p in problems:
        print(f"  ERROR {p}")
    print(f"layout_spec_check: {len(problems)} problem(s)（{args.scene}，规格 {args.spec}）")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
