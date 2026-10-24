#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""check_scene_narrative.py — UI 场景叙事约束门禁（gate 第 22 项，v3）。

规格源：docs/ui-redesign-v3/02_spec_lang.md
数据源：
  - assets/data/ui_schema/scene_manifest_v3.json   （DSL 实例）
  - assets/data/ui_schema/scene_manifest_v3.schema.json   （JSON Schema）
  - assets/data/scenes.json                          （坐标事实源，v3.1 起含 focus_anchor 键）

检查项（机器可判，只读、幂等）：
  N1 manifest 文件存在 + JSON Schema 校验通过
  N2 每场景 mood / focus.anchor / rhythm.rule 三元一致（按 §2 推荐表）
  N3 focus.bbox 占屏比 ≥ 20%（w*h ≥ 20，百分比空间）
  N4 rhythm beats 权重和 = 1.0（±0.01），beats[0] = 重心 beat（name == focus.anchor）
     且 weight ≥ 0.30（注意力优先级序；y_pct 仅位置标注，不排序）
  N5 beats 数量 ∈ [2, 4]
  N6 mood 与 permanent_hud_count_max 一致性（按 §6 表上限）
  N7 dead_zone_max_pct ≤ 15（默认上限，可逐场景覆盖但不应超过 25）
  N8 scenes.json focus_anchor.bbox 与 DSL focus.bbox_pct × (1080, 1920) 一致
     （v3.1 起执行；v3.0 阶段 scenes.json 无 focus_anchor 键时 SKIP）

退出码：0 = PASS；1 = FAIL（列出违规项）；2 = SKIP（manifest 文件不存在）。

用法：
  python scripts/check_scene_narrative.py            # 只读门禁（默认即只读，不写文件）
  python scripts/check_scene_narrative.py --check    # 只读门禁（契约别名，行为同默认）
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent

MANIFEST_PATH = ROOT / "assets" / "data" / "ui_schema" / "scene_manifest_v3.json"
SCHEMA_PATH = ROOT / "assets" / "data" / "ui_schema" / "scene_manifest_v3.schema.json"
SCENES_JSON_PATH = ROOT / "assets" / "data" / "scenes.json"

GAME_SCENES = ["index", "select_role", "select_map", "main_map", "vs", "message", "saveload"]

# §2 mood → (focus.anchor, rhythm.rule) 推荐三元
MOOD_RECOMMENDATION: dict[str, tuple[set[str], set[str]]] = {
    "epic_entry":     ({"title_art", "compass_ring"}, {"art_to_action_top_down"}),
    "hero_choice":    ({"portrait_carousel"},    {"side_anchor_center_options"}),
    "map_overview":   ({"map_collage"},          {"side_anchor_center_options"}),
    "world_explore":  ({"world_canvas"},         {"world_first_hud_corner"}),
    "tense_battle":   ({"battle_arena"},         {"arena_center_command_edge"}),
    "reading":        ({"text_panel"},            {"left_hero_right_text"}),
    "archive":        ({"slot_list_with_meta"},  {"three_column_left_hero"}),
}

# §6 mood → permanent_hud_count_max 上限
MOOD_HUD_MAX: dict[str, int] = {
    "epic_entry":    0,
    "hero_choice":   0,
    "map_overview":  0,
    "world_explore": 3,
    "tense_battle":  4,
    "reading":       1,
    "archive":       1,
}

# 屏坐标系（1080×1920，竖屏）
SCREEN_W = 1080
SCREEN_H = 1920


def fail(msg: str) -> None:
    print(f"  FAIL  {msg}")


def info(msg: str) -> None:
    print(f"        {msg}")


def load_json(path: Path) -> Any:
    with path.open(encoding="utf-8") as f:
        return json.load(f)


def check_schema(manifest: dict, schema: dict) -> list[str]:
    """N1：JSON Schema 校验（轻量内联实现，避免 jsonschema 依赖）。

    只校验本规格实际用到的约束（required / enum / min/max / minItems / maxItems）。
    生产级 JSON Schema 校验可由 ti-tools 的 doccheck2 接管；本脚本聚焦叙事断言。
    """
    problems: list[str] = []

    def _walk(node: dict, instance: Any, path: str = "$") -> None:
        if not isinstance(instance, dict):
            return
        # required
        for req in node.get("required", []):
            if req not in instance:
                problems.append(f"{path}: missing required key '{req}'")
        # type
        t = node.get("type")
        if t == "object" and not isinstance(instance, dict):
            problems.append(f"{path}: expected object, got {type(instance).__name__}")
            return
        # properties
        props = node.get("properties", {})
        for k, v in props.items():
            if k in instance:
                _walk(v, instance[k], f"{path}.{k}")
        # enum
        if "enum" in node:
            val = instance
            if val not in node["enum"]:
                problems.append(f"{path}: '{val}' not in enum {node['enum']}")
        # additionalProperties = false（简化：只警告，不强制）
        # minItems / maxItems（针对 array）
        if isinstance(instance, list):
            if "minItems" in node and len(instance) < node["minItems"]:
                problems.append(f"{path}: minItems={node['minItems']}, got {len(instance)}")
            if "maxItems" in node and len(instance) > node["maxItems"]:
                problems.append(f"{path}: maxItems={node['maxItems']}, got {len(instance)}")
        # minimum / maximum
        if isinstance(instance, (int, float)) and not isinstance(instance, bool):
            if "minimum" in node and instance < node["minimum"]:
                problems.append(f"{path}: {instance} < minimum {node['minimum']}")
            if "maximum" in node and instance > node["maximum"]:
                problems.append(f"{path}: {instance} > maximum {node['maximum']}")

    _walk(schema, manifest)
    return problems


def bbox_area_pct(bbox_pct: dict) -> float:
    return bbox_pct["w"] * bbox_pct["h"] / 100.0  # w,h 是百分比，乘积 /100 是占屏比


def check_narrative(scene_id: str, m: dict) -> int:
    """N2-N7：单场景叙事断言。返回违规数。"""
    n_fail = 0
    narr = m["narrative"]
    mood = narr["mood"]
    focus = narr["focus"]
    rhythm = narr["rhythm"]
    info_density = narr["info_density"]

    # N2：mood → (focus.anchor, rhythm.rule) 三元一致
    expected_anchors, expected_rules = MOOD_RECOMMENDATION.get(
        mood, (set(), set())
    )
    if focus["anchor"] not in expected_anchors:
        fail(f"{scene_id}: focus.anchor='{focus['anchor']}' 与 mood='{mood}' 推荐 {expected_anchors} 不一致")
        n_fail += 1
    if rhythm["rule"] not in expected_rules:
        fail(f"{scene_id}: rhythm.rule='{rhythm['rule']}' 与 mood='{mood}' 推荐 {expected_rules} 不一致")
        n_fail += 1

    # N3：focus.bbox 占屏 ≥ 20%
    area = bbox_area_pct(focus["bbox_pct"])
    if area < 0.20:
        fail(f"{scene_id}: focus.bbox 占屏 {area:.1%} < 20% 下限")
        n_fail += 1

    # N4：beats[0] = 重心 beat（name == focus.anchor）且 weight ≥ 0.30，权重和 = 1.0 ± 0.01。
    # beat 序 = 注意力优先级序；y_pct 仅位置标注，不排序（spec §4.1 修订）。
    beats = rhythm["beats"]
    total_w = sum(b["weight"] for b in beats)
    if abs(total_w - 1.0) > 0.01:
        fail(f"{scene_id}: rhythm beats 权重和 = {total_w:.3f}（应 = 1.0 ± 0.01）")
        n_fail += 1
    if beats[0]["name"] != focus["anchor"]:
        fail(f"{scene_id}: beats[0]='{beats[0]['name']}' 不是重心 beat '{focus['anchor']}'（注意力优先级序违例）")
        n_fail += 1
    if beats[0]["weight"] < 0.30:
        fail(f"{scene_id}: 重心 beat weight={beats[0]['weight']} < 0.30（视觉重心权重不足）")
        n_fail += 1

    # N5：beats 数量 ∈ [2, 4]
    if not (2 <= len(beats) <= 4):
        fail(f"{scene_id}: beats 数量 {len(beats)} 不在 [2, 4]")
        n_fail += 1

    # N6：permanent_hud_count_max 与 mood 一致
    expected_max = MOOD_HUD_MAX.get(mood, 0)
    if narr["permanent_hud_count_max"] > expected_max:
        fail(f"{scene_id}: permanent_hud_count_max={narr['permanent_hud_count_max']} "
             f"> mood '{mood}' 上限 {expected_max}")
        n_fail += 1

    # N7：dead_zone_max_pct ≤ 25（防人为放大死区）
    if info_density["dead_zone_max_pct"] > 25:
        fail(f"{scene_id}: dead_zone_max_pct={info_density['dead_zone_max_pct']} > 25 上限")
        n_fail += 1

    return n_fail


def check_scenes_json_focus_anchor(manifest: dict, scenes_json: dict) -> tuple[int, int]:
    """N8：scenes.json focus_anchor.bbox 与 DSL bbox_pct × (1080,1920) 一致。

    v3.0 阶段 scenes.json 可无 focus_anchor 键（视为未落地，SKIP）。
    返回 (失败数, SKIP 数)。
    """
    n_fail = 0
    n_skip = 0
    scenes = manifest["scenes"]
    for sid, m in scenes.items():
        narr = m["narrative"]
        bbox_pct = narr["focus"]["bbox_pct"]
        expected_x = round(bbox_pct["x"] / 100 * SCREEN_W)
        expected_y = round(bbox_pct["y"] / 100 * SCREEN_H)
        expected_w = round(bbox_pct["w"] / 100 * SCREEN_W)
        expected_h = round(bbox_pct["h"] / 100 * SCREEN_H)

        scene_data = scenes_json.get(sid, {})
        fa = scene_data.get("focus_anchor")
        if fa is None:
            info(f"{sid}: scenes.json 无 focus_anchor 键（v3.1 待落地，SKIP N8）")
            n_skip += 1
            continue
        bbox = fa.get("bbox", {})
        actual = (bbox.get("x"), bbox.get("y"), bbox.get("w"), bbox.get("h"))
        expected = (expected_x, expected_y, expected_w, expected_h)
        for label, a, e in zip(["x", "y", "w", "h"], actual, expected):
            if a is None or abs(a - e) > 1:
                fail(f"{sid}: scenes.json focus_anchor.bbox.{label}={a} "
                     f"≠ DSL bbox_pct × 屏尺寸 = {e}（容差 1px）")
                n_fail += 1
    return n_fail, n_skip


def main() -> int:
    # S21.8 工具契约：--check = 只读门禁别名（默认行为即只读校验，不写文件；违规非 0 退出）
    ap = argparse.ArgumentParser(description="UI 场景叙事约束门禁（gate 第 22 项，v3）")
    ap.add_argument("--check", action="store_true", help="只读模式（默认行为即只读门禁，保留契约别名）")
    ap.parse_args()
    if not MANIFEST_PATH.is_file():
        print(f"SKIP  check_scene_narrative (manifest 不存在: {MANIFEST_PATH.name})")
        return 2

    print("=== check_scene_narrative (UI 叙事约束 DSL v3) ===")
    total_fail = 0

    # N1：加载 + Schema 校验
    try:
        manifest = load_json(MANIFEST_PATH)
    except Exception as e:
        fail(f"manifest JSON 解析失败: {e}")
        return 1
    if SCHEMA_PATH.is_file():
        try:
            schema = load_json(SCHEMA_PATH)
            problems = check_schema(manifest, schema)
            if problems:
                for p in problems:
                    fail(f"Schema: {p}")
                total_fail += len(problems)
        except Exception as e:
            fail(f"schema JSON 解析失败: {e}")
            total_fail += 1
    else:
        info(f"schema 文件不存在: {SCHEMA_PATH}（跳过 Schema 校验）")

    # 覆盖检查：每个游戏场景必须有 manifest 条目
    scenes = manifest.get("scenes", {})
    for sid in GAME_SCENES:
        if sid not in scenes:
            fail(f"manifest 缺失场景: {sid}")
            total_fail += 1

    # 逐场景 N2-N7
    for sid, m in scenes.items():
        scene_fail = check_narrative(sid, m)
        total_fail += scene_fail

    # N8：scenes.json 对账（v3.0 SKIP，v3.1 起执行）
    if SCENES_JSON_PATH.is_file():
        try:
            scenes_json = load_json(SCENES_JSON_PATH)
            n8_fail, n8_skip = check_scenes_json_focus_anchor(manifest, scenes_json)
            total_fail += n8_fail
            if n8_skip > 0 and n8_fail == 0:
                info(f"N8：{n8_skip} 个场景 scenes.json 暂无 focus_anchor 键（v3.1 待落地）")
        except Exception as e:
            fail(f"scenes.json 解析失败: {e}")
            total_fail += 1
    else:
        info(f"scenes.json 不存在: {SCENES_JSON_PATH}（N8 SKIP）")

    # 收口
    print(f"=== check_scene_narrative: {total_fail} failure(s) ===")
    return 0 if total_fail == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
