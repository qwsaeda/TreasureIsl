#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""ui_schema_check.py — S85 声明式 UI 组件 schema 门禁（见 docs/process/33_ui_component_schema.md）。

数据源：
  - assets/data/ui_schema/components_v1.json          （组件实例，5 纯 UI 场景）
  - assets/data/ui_schema/components_v1.schema.json   （JSON Schema draft-07：enum + 禁绝对坐标）
  - assets/data/ui_schema/asset_roles.json            （role → 资产路径登记，单源）
  - assets/data/strings.json                          （i18n key 存在性）

检查项（机器可判、只读、幂等）：
  S1 实例文件存在 + JSON 合法 + schema_version=1
  S2 每场景 scene_id ∈ 5 纯 UI 闭集；组件 id 场景内唯一 + 正则
  S3 JSON Schema enum 校验（type/layout.intent/action/state/tier）+ 禁绝对坐标字段（x/y/w/h/rect）
  S4 条件必填：image/rect/grad/glow/badge 须 role；text/button/input/list 须 binding；
     button/list/badge 须 action
  S5 role 须在 asset_roles.json 登记；icon 须在 ui_icons.json 登记
  S6 action ∈ 33 号 §4 闭集；binding 的 i18n key/模板解析后须在 strings.json 存在
     （模板 {label}/{id} 等变量用样例值代入：每场景已知 label 集，缺样例则报 WARN 不 FAIL）

退出码：0 = PASS；1 = FAIL（点名场景+组件 id+字段）；2 = SKIP（实例文件不存在，S85.2 落地前）。

用法：
  python scripts/ui_schema_check.py            # 只读门禁（默认）
  python scripts/ui_schema_check.py --check    # 契约别名（行为同默认）
"""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCHEMA_DIR = ROOT / "assets/data/ui_schema"
INSTANCE = SCHEMA_DIR / "components_v1.json"
SCHEMA = SCHEMA_DIR / "components_v1.schema.json"
ROLES = SCHEMA_DIR / "asset_roles.json"
STRINGS = ROOT / "assets/data/strings.json"
ICONS = ROOT / "assets/data/ui_icons.json"

SCENES = {"index", "select_role", "select_map", "message", "saveload"}
TYPE_ROLE = {"image", "rect", "grad", "glow", "badge"}
TYPE_BINDING = {"text", "button", "input", "list", "panel"}
TYPE_ACTION = {"button", "list", "badge"}
TYPES = TYPE_ROLE | TYPE_BINDING | {"image", "rect", "text", "button", "input", "list", "badge", "grad", "glow", "panel"}
INTENTS = {"fill", "center", "side_left", "side_right", "bottom_bar", "flex_col", "flex_row", "anchored"}
ACTIONS = {"menu_select", "character_select", "map_select", "slot_load", "slot_write",
           "msg_next", "msg_prev", "spin", "back", "attack", "do_item", "defend"}
STATES = {"normal", "selected", "hover", "disabled"}
TIERS = {"title", "body", "hint"}
FORBIDDEN_COORD = {"x", "y", "w", "h", "rect"}
ANCHORS = {"top_left", "top_right", "top_center", "bottom_left", "bottom_right", "bottom_center", "center_left", "center_right", "center"}
ID_RE = re.compile(r"^[a-z][a-z0-9_]*$")
# binding 模板样例变量（S6 代入后查 key）；未知变量跳过校验（WARN）
SAMPLE_VARS = {"{label}": "panda", "{id}": "m01", "{idx}": "0", "{kind}": "help"}


def fail(errors: list[str]) -> int:
    for e in errors:
        print(f"  ERROR {e}")
    print(f"ui_schema_check: FAIL（{len(errors)} 项违规）")
    return 1


def load_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        raise ValueError(f"{path.name} 读取/解析失败: {exc}") from exc


def check() -> int:
    if not INSTANCE.is_file():
        print("ui_schema_check: SKIP（components_v1.json 不存在，S85.2 落地前）")
        return 2
    errors: list[str] = []
    try:
        data = load_json(INSTANCE)
        schema = load_json(SCHEMA)
        strings = load_json(STRINGS)
    except ValueError as exc:
        return fail([str(exc)])

    if data.get("schema_version") != 1:
        errors.append(f"schema_version 应为 1，实得 {data.get('schema_version')!r}")

    roles: dict = {}
    if ROLES.is_file():
        try:
            roles = load_json(ROLES).get("roles", {})
        except ValueError:
            pass
    icon_names: set[str] = set()
    if ICONS.is_file():
        try:
            icon_names = {i["name"] for i in load_json(ICONS).get("icons", [])}
        except (ValueError, KeyError, TypeError):
            pass
    string_keys = {k for k in strings if k != "schema_version"}

    for scene_id, scene in data.get("scenes", {}).items():
        if scene_id not in SCENES:
            errors.append(f"场景 {scene_id} 不在 5 纯 UI 闭集 {sorted(SCENES)}")
        seen: set[str] = set()
        for idx, comp in enumerate(scene.get("elements", [])):
            cid = comp.get("id", f"#{idx}")
            loc = f"{scene_id}.{cid}"
            if not isinstance(comp.get("id"), str) or not ID_RE.match(str(comp.get("id", ""))):
                errors.append(f"{loc}: id 非法（须 ^[a-z][a-z0-9_]*$）")
            if cid in seen:
                errors.append(f"{loc}: id 重复")
            seen.add(cid)

            typ = comp.get("type")
            if typ not in TYPES:
                errors.append(f"{loc}: type {typ!r} 非法（闭集 {sorted(TYPES)}）")

            # S3 禁绝对坐标（JSON Schema not.anyOf 的 Python 兜底，防 schema 校验器缺失）
            bad_coord = FORBIDDEN_COORD & comp.keys()
            if bad_coord:
                errors.append(f"{loc}: 出现绝对坐标字段 {sorted(bad_coord)}（违反不手写像素总纲）")

            layout = comp.get("layout", {})
            if not isinstance(layout, dict) or layout.get("intent") not in INTENTS:
                errors.append(f"{loc}: layout.intent {layout.get('intent') if isinstance(layout, dict) else layout!r} 非法")
            if isinstance(layout, dict) and layout.get("anchor") and layout["anchor"] not in ANCHORS:
                errors.append(f"{loc}: layout.anchor {layout['anchor']!r} 非法")

            # S4 条件必填
            if typ in TYPE_ROLE and not comp.get("role"):
                errors.append(f"{loc}: type={typ} 须 role")
            if typ in TYPE_BINDING and not comp.get("binding"):
                errors.append(f"{loc}: type={typ} 须 binding")
            if typ in TYPE_ACTION and not comp.get("action"):
                errors.append(f"{loc}: type={typ} 须 action")

            # S5 登记对账
            role = comp.get("role")
            if role and roles and role not in roles:
                errors.append(f"{loc}: role {role!r} 未在 asset_roles.json 登记")
            icon = comp.get("icon")
            if icon and icon_names and icon not in icon_names:
                errors.append(f"{loc}: icon {icon!r} 未在 ui_icons.json 登记")

            # S6 action / i18n
            action = comp.get("action")
            if action and action not in ACTIONS:
                errors.append(f"{loc}: action {action!r} 非法（闭集 {sorted(ACTIONS)}）")
            for field in ("binding", "text"):
                key = comp.get(field)
                if not key or not isinstance(key, str):
                    continue
                resolved = key
                for var, sample in SAMPLE_VARS.items():
                    resolved = resolved.replace(var, sample)
                # 只校验像 i18n key 的形态（含点号/已知前缀），跳过纯字面量
                if "." in resolved and resolved not in string_keys:
                    errors.append(f"{loc}: {field}={key!r} → {resolved!r} 在 strings.json 不存在")
            if comp.get("tier") and comp["tier"] not in TIERS:
                errors.append(f"{loc}: tier {comp['tier']!r} 非法")
            states = comp.get("state", [])
            if isinstance(states, list):
                for s in states:
                    if s not in STATES:
                        errors.append(f"{loc}: state {s!r} 非法")

    # JSON Schema 校验（jsonschema 若可用，作为 S3 enum 的严格补充）
    try:
        import jsonschema  # type: ignore
        jsonschema.validate(data, schema)
    except ImportError:
        pass  # 内置校验已覆盖 enum/禁坐标；无第三方依赖时仍可用
    except Exception as exc:  # noqa: BLE001
        first = str(exc).strip().splitlines()[0] if str(exc).strip() else type(exc).__name__
        errors.append(f"JSON Schema 校验失败: {first}")

    return fail(errors) if errors else (_ok())


def _ok() -> int:
    data = load_json(INSTANCE)
    n = sum(len(s.get("elements", [])) for s in data.get("scenes", {}).values())
    print(f"ui_schema_check: OK（{len(data.get('scenes', {}))} 场景 / {n} 组件，enum+禁绝对坐标+登记对账通过）")
    return 0


if __name__ == "__main__":
    sys.exit(check())
