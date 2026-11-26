#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""world_schema_check.py — S87.1 世界层（vs/main_map）声明式 schema 门禁。

单一事实源：
  - assets/data/ui_schema/world_components.json          （世界层静态元素声明，2 场景）
  - assets/data/ui_schema/world_components.schema.json   （JSON Schema draft-07：type 闭集 + 禁裸坐标）
  - assets/data/ui_schema/asset_roles.json               （role → 资产路径登记，world role 扩登记）
  - assets/data/strings.json                             （i18n key 存在性）
  - assets/data/main_map_v6_layout_landscape.json        （geometry.ref 源①：v6 布局几何）
  - assets/data/scenes.json                              （geometry.ref 源②：场景矩形）

检查项（机器可判、只读、幂等；无需 jsonschema 也能跑，装了则作严格补充）：
  W1 实例/Schema 文件存在 + JSON 合法 + schema_version=1
  W2 scene_id ∈ {main_map, vs}；元素 id 场景内唯一 + ^[a-z][a-z0-9_]*$
  W3 type ∈ {Rect,Text,Image}（CPU DrawOp 三型）；禁 grad/glow/panel 与散落裸坐标
     （x/y/w/h/rect/layout/intent 直接出现在元素顶层 = FAIL；world_rect / geometry 内合法）；
     world_rect 四键齐全且落在 1920×1080 屏内
  W4 role 须在 asset_roles.json 登记且资产文件实存；z 为整数；每个元素须有 geometry 或 world_rect
  W5 pick.action ∈ UiAction 15 项闭集 + pick.index 非负整数；binding i18n key 实存或 host path 白名单；
     state/tier/visible_when(path+op)/visible_in_phase 形状合法
  W6 geometry.source ∈ 两源闭集 + ref 指向的源键真实存在（支持 dot 路径 + [n]/[*] 下标）；
     geometry 与 world_rect 不得同时缺失

退出码：0 = PASS；1 = FAIL（点名场景 + 元素 id + 字段）；2 = 实例文件不存在（未落地时 SKIP）。

用法：
  python3 scripts/world_schema_check.py [--check]
  python3 scripts/world_schema_check.py --check --instance <path>   # 负例注入自证（只读）
"""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCHEMA_DIR = ROOT / "assets/data/ui_schema"
INSTANCE = SCHEMA_DIR / "world_components.json"
SCHEMA = SCHEMA_DIR / "world_components.schema.json"
ROLES = SCHEMA_DIR / "asset_roles.json"
STRINGS = ROOT / "assets/data/strings.json"
V6_LAYOUT = ROOT / "assets/data/main_map_v6_layout_landscape.json"
SCENES = ROOT / "assets/data/scenes.json"

SCENES_IDS = {"main_map", "vs"}
TYPES = {"Rect", "Text", "Image"}
FORBIDDEN_TYPE = {"grad", "glow", "panel", "image", "rect", "text", "button", "input", "list", "badge"}
FORBIDDEN_COORD = {"x", "y", "w", "h", "rect", "layout", "intent"}
# UiAction 15 项闭集（33 号规范 §4 + S87.1 vs_skip/confirm/reset）
ACTIONS = {
    "menu_select", "character_select", "map_select", "slot_load", "slot_write",
    "msg_next", "msg_prev", "spin", "back", "attack", "do_item", "defend",
    "vs_skip", "vs_confirm", "vs_reset",
}
STATES = {"normal", "selected", "hover", "disabled"}
TIERS = {"title", "body", "hint"}
SOURCES = {"main_map_v6_layout_landscape.json", "scenes.json"}
OPS = {"is_none", "is_some", "eq", "ne"}
# &dyn SceneHost 白名单 path（禁任意 Rust 方法名 binding，S87.2 渲染器按此实现访问器）
HOST_PATHS = {
    "EconomyHost.gold", "EconomyHost.cargo",
    "MapHost.dice", "MapHost.seat", "MapHost.panel", "MapHost.handoff_pending",
    "BattleHost.hp_player", "BattleHost.hp_enemy", "BattleHost.crew",
    "BattleHost.round", "BattleHost.report", "BattleHost.phase",
    "ToastHost.text", "OutcomeHost.text",
}
VISIBLE_WHEN_PATHS = HOST_PATHS
SCREEN_W, SCREEN_H = 1920, 1080
ID_RE = re.compile(r"^[a-z][a-z0-9_]*$")


def fail(errors: list[str]) -> int:
    for e in errors:
        print(f"  ERROR {e}")
    print(f"world_schema_check: FAIL（{len(errors)} 项违规）")
    return 1


def load_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        raise ValueError(f"{path.name} 读取/解析失败: {exc}") from exc


def resolve_ref(data, ref: str) -> tuple[bool, str]:
    """解析 geometry.ref：dot 路径 + [n]/[*] 下标；返回 (是否命中, 原因)。"""
    if not isinstance(data, dict):
        return False, "源不是 JSON object"
    seg_re = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)((?:\[(?:\d+|\*)\])*)$")
    node = data
    for seg in ref.split("."):
        m = seg_re.match(seg)
        if not m:
            return False, f"路径段 `{seg}` 非法"
        key, suffix = m.group(1), m.group(2)
        if not isinstance(node, dict) or key not in node:
            return False, f"键 `{key}` 不存在"
        node = node[key]
        for idx in re.findall(r"\[(\d+|\*)\]", suffix):
            if not isinstance(node, list):
                return False, f"键 `{key}` 不是数组，不能下标"
            if idx == "*":
                if not node:
                    return False, f"键 `{key}` 是空数组，通配无意义"
                continue
            n = int(idx)
            if n >= len(node):
                return False, f"下标 [{n}] 越界（长度 {len(node)}）"
            node = node[n]
    return True, "ok"


def check_instance(path: Path) -> int:
    errors: list[str] = []
    try:
        data = load_json(path)
        schema = load_json(SCHEMA)
        strings = load_json(STRINGS)
        v6 = load_json(V6_LAYOUT)
        scenes_data = load_json(SCENES)
    except ValueError as exc:
        return fail([str(exc)])
    roles: dict = {}
    if ROLES.is_file():
        try:
            roles = load_json(ROLES).get("roles", {})
        except ValueError:
            pass
    string_keys = {k for k in strings if k != "schema_version"}
    sources = {
        "main_map_v6_layout_landscape.json": v6,
        "scenes.json": scenes_data,
    }

    # W1 版本号
    if data.get("schema_version") != 1:
        errors.append(f"schema_version 应为 1，实得 {data.get('schema_version')!r}")

    for scene_id, scene in data.get("scenes", {}).items():
        # W2 场景闭集
        if scene_id not in SCENES_IDS:
            errors.append(f"场景 {scene_id} 不在世界层闭集 {sorted(SCENES_IDS)}")
        sid = scene.get("scene_id", scene_id)
        if sid != scene_id:
            errors.append(f"场景 {scene_id}: scene_id={sid!r} 与键名不一致")
        seen: set[str] = set()
        for idx, elem in enumerate(scene.get("elements", [])):
            eid = elem.get("id", f"#{idx}")
            loc = f"{scene_id}.{eid}"
            if not isinstance(elem.get("id"), str) or not ID_RE.match(str(eid)):
                errors.append(f"{loc}: id 非法（须 ^[a-z][a-z0-9_]*$）")
            if eid in seen:
                errors.append(f"{loc}: id 重复")
            seen.add(eid)

            # W3 type 闭集 + 禁裸坐标
            typ = elem.get("type")
            if typ not in TYPES:
                errors.append(
                    f"{loc}: type {typ!r} 非法（闭集 {sorted(TYPES)}；grad/glow/panel 非 CPU DrawOp，禁）"
                )
            if typ in FORBIDDEN_TYPE:
                errors.append(f"{loc}: type {typ!r} 属被禁类型（小写 UI 型 / grad/glow/panel 不得入世界层）")
            bad = FORBIDDEN_COORD & elem.keys()
            if bad:
                errors.append(f"{loc}: 元素顶层出现 {sorted(bad)}（裸坐标/布局字段非法，只能写在 world_rect / geometry.ref）")

            # W3 world_rect 四键 + 屏内约束
            wr = elem.get("world_rect")
            if wr is not None:
                if not isinstance(wr, dict) or set(wr) != {"x", "y", "w", "h"}:
                    errors.append(f"{loc}: world_rect 须恰含 x/y/w/h 四键")
                else:
                    for k, lim in (("x", SCREEN_W), ("y", SCREEN_H), ("w", SCREEN_W), ("h", SCREEN_H)):
                        v = wr[k]
                        if not isinstance(v, int) or isinstance(v, bool) or not 0 <= v <= lim:
                            errors.append(f"{loc}: world_rect.{k}={v!r} 非法（整数且 0..{lim}）")

            # W4 z 整数 + 锚点存在（geometry 或 world_rect 二选一）
            if "z" in elem and (not isinstance(elem["z"], int) or isinstance(elem["z"], bool)):
                errors.append(f"{loc}: z 非整数")
            if elem.get("geometry") is None and wr is None:
                errors.append(f"{loc}: 缺 geometry / world_rect（世界元素须有锚点）")

            # W4 role 登记 + 资产文件实存
            role = elem.get("role")
            if role:
                if not isinstance(role, str) or not ID_RE.match(role):
                    errors.append(f"{loc}: role {role!r} 非法（须 ^[a-z][a-z0-9_]*$）")
                elif role not in roles:
                    errors.append(f"{loc}: role {role!r} 未在 asset_roles.json 登记")
                else:
                    asset = roles[role]
                    if not isinstance(asset, str) or not (ROOT / asset).is_file():
                        errors.append(f"{loc}: role {role!r} 资产 '{asset}' 文件不存在")

            # W5 pick.action 闭集 + index
            pick = elem.get("pick")
            if pick is not None:
                if not isinstance(pick, dict) or set(pick) != {"action", "index"}:
                    errors.append(f"{loc}: pick 须恰含 action/index 两键")
                else:
                    if pick["action"] not in ACTIONS:
                        errors.append(
                            f"{loc}: pick.action {pick['action']!r} 非法（UiAction 15 项闭集）"
                        )
                    idx = pick["index"]
                    if not isinstance(idx, int) or isinstance(idx, bool) or idx < 0:
                        errors.append(f"{loc}: pick.index {idx!r} 非法（非负整数）")

            # W5 binding：i18n key 字符串 或 host path 白名单对象
            binding = elem.get("binding")
            if isinstance(binding, str):
                if "." in binding and binding not in string_keys:
                    errors.append(f"{loc}: binding '{binding}' 在 strings.json 不存在（裸 i18n key）")
            elif isinstance(binding, dict):
                if binding.get("kind") != "host":
                    errors.append(f"{loc}: binding.kind {binding.get('kind')!r} 非法（只支持 host）")
                if binding.get("path") not in HOST_PATHS:
                    errors.append(
                        f"{loc}: binding.path {binding.get('path')!r} 不在 SceneHost 白名单"
                    )
                if "fmt" in binding and not isinstance(binding["fmt"], str):
                    errors.append(f"{loc}: binding.fmt 须为字符串")
            elif binding is not None:
                errors.append(f"{loc}: binding 须为字符串或 {{kind:host,path,fmt}}")

            # W5 state / tier / anim_ref / visible_in_phase
            for s in elem.get("state", []) or []:
                if s not in STATES:
                    errors.append(f"{loc}: state {s!r} 非法")
            if elem.get("tier") and elem["tier"] not in TIERS:
                errors.append(f"{loc}: tier {elem['tier']!r} 非法")
            if elem.get("anim_ref") is not None and not isinstance(elem["anim_ref"], str):
                errors.append(f"{loc}: anim_ref 须为字符串（代码时间轴名）")
            phases = elem.get("visible_in_phase", []) or []
            if not isinstance(phases, list) or any(not isinstance(p, str) for p in phases):
                errors.append(f"{loc}: visible_in_phase 须为字符串数组")

            # W5 visible_when path/op 白名单
            vw = elem.get("visible_when")
            if vw is not None:
                if not isinstance(vw, dict):
                    errors.append(f"{loc}: visible_when 须为对象")
                else:
                    if vw.get("path") not in VISIBLE_WHEN_PATHS:
                        errors.append(f"{loc}: visible_when.path {vw.get('path')!r} 不在白名单")
                    if vw.get("op") not in OPS:
                        errors.append(f"{loc}: visible_when.op {vw.get('op')!r} 非法")
                    if vw.get("op") in {"eq", "ne"} and "value" not in vw:
                        errors.append(f"{loc}: visible_when.op={vw['op']} 须带 value")

            # W6 geometry 源/ref 解析
            geo = elem.get("geometry")
            if geo is not None:
                if not isinstance(geo, dict) or set(geo) != {"source", "ref"}:
                    errors.append(f"{loc}: geometry 须恰含 source/ref 两键")
                else:
                    if geo["source"] not in SOURCES:
                        errors.append(f"{loc}: geometry.source {geo['source']!r} 非法（只许两源）")
                    else:
                        ok, why = resolve_ref(sources[geo["source"]], geo["ref"])
                        if not ok:
                            errors.append(f"{loc}: geometry.ref '{geo['ref']}' 在 {geo['source']} 不存在（{why}）")

    # JSON Schema 严格补充（jsonschema 可用时）：与内置 enum/禁坐标互为正交双保险。
    try:
        import jsonschema  # type: ignore
        jsonschema.validate(data, schema)
    except ImportError:
        pass
    except Exception as exc:  # noqa: BLE001
        first = str(exc).strip().splitlines()[0] if str(exc).strip() else type(exc).__name__
        errors.append(f"JSON Schema 校验失败: {first}")

    return fail(errors) if errors else _ok(path)


def _ok(path: Path) -> int:
    data = load_json(path)
    n = sum(len(s.get("elements", [])) for s in data.get("scenes", {}).values())
    per = ", ".join(f"{sid}={len(s.get('elements', []))}" for sid, s in data.get("scenes", {}).items())
    print(
        f"world_schema_check: OK（{len(data.get('scenes', {}))} 场景 / {n} 元素 [{per}]；"
        "type 三型闭集 + 禁裸坐标 + 登记对账 + geometry.ref 解析通过）"
    )
    return 0


def main() -> int:
    args = sys.argv[1:]
    path = INSTANCE
    if "--instance" in args:
        i = args.index("--instance")
        if i + 1 >= len(args):
            print("world_schema_check: --instance 缺路径参数")
            return 2
        path = Path(args[i + 1])
        if not path.is_absolute():
            path = ROOT / path
    if not path.is_file():
        print(f"world_schema_check: SKIP（{path.name} 不存在，S87.1 落地前）")
        return 2
    return check_instance(path)


if __name__ == "__main__":
    sys.exit(main())
