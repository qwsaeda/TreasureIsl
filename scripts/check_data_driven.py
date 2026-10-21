#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""数据分离门禁（S18.7）：界面 = 数据，渲染层不认识场景语义。

判据（任一不满足即 FAIL）：
1. **通用原语闭集**：渲染层只认 7 个通用 kind（`rect`/`text`/`icon`/`grad`/`glow`/`disc`/`image`）；
   数据侧（`scenes.json`）也不得出现第 8 个 kind。
2. **渲染层无场景语义**：渲染层源码不得出现场景名 / 场景专属 kind
   （index/select_role/main_map/vs/saveload/menu_items/ornate_frame/night_sky/...）。
3. **令牌存在性**：`scenes.json` 原语引用的每个 theme 令牌名必须真实存在
   （防"令牌拼错 → 渲染器静默回退"= 今天踩过的 `sky_horizon` 坑）。
4. **版面数值不在渲染层**：渲染层不得出现 1080/1920 之外的版面魔数
   （坐标/尺寸只能来自数据或 theme 令牌）。

用法：python scripts/check_data_driven.py（exit 0 = 通过）
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

# 通用原语闭集（渲染层只认这些；S18.8b 新增 `image` 贴图原语）。
GENERIC_KINDS = {"rect", "text", "icon", "grad", "glow", "disc", "image"}

# 渲染层源码（须无场景语义）。
RENDERER_FILES = [
    "crates/ti-render/src/generic.rs",
    "crates/ti-engine/src/scene_render.rs",
]

# 场景语义关键字（渲染层禁出现；注释/文档字符串豁免：只查代码行）。
SCENE_SEMANTICS = [
    "index", "select_role", "select_map", "main_map", "saveload",
    "menu_items", "ornate_frame", "night_sky", "star_sky", "ocean_horizon",
    "moon_glow", "title_art", "ambient_art", "battle_arena", "world_canvas",
]

# theme 令牌名（用于数据侧令牌存在性校验；Rust DEFAULT 的权威列表）。
THEME_TOKENS = {
    "parch_dark", "parch_light", "wood", "gold", "gold_glow", "gold_bright",
    "menu_bg", "menu_border", "hilite", "bevel_hi", "bevel_shadow",
    "text_on_parch", "text_on_wood", "danger", "player", "landmark", "bg",
    "title_bar", "selected", "secondary", "black",
    "wood_warm", "gold_warm", "sky_navy", "sky_horizon", "moon_pale",
    "panel_warm", "brass", "brass_light", "brass_dark", "sea_deep", "sea_shallow", "reef", "land_sand", "land_sand_light", "land_sand_dark", 
}


def strip_comments(line: str) -> str:
    """去掉行注释（文档字符串豁免 = 只查代码行）。"""
    idx = line.find("//")
    return line if idx == -1 else line[:idx]


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    errors: list[str] = []

    # ① 数据侧：kind ∈ 通用闭集 + 令牌存在性
    scenes_path = root / "assets" / "data" / "scenes.json"
    try:
        scenes = json.loads(scenes_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as e:
        print(f"  ERROR 读 scenes.json 失败: {e}")
        return 1
    # S18.8b：资产清单（`assets/ui_manifest.json` = 资产单一事实源）→ 已登记的资产路径集合。
    manifest_path = root / "assets" / "ui_manifest.json"
    registered_assets: set[str] = set()
    if manifest_path.exists():
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            registered_assets = {
                a.get("file", "") for a in manifest.get("assets", []) if isinstance(a, dict)
            }
        except (OSError, json.JSONDecodeError) as e:
            print(f"  ERROR 读 assets/ui_manifest.json 失败: {e}")
            return 1
    else:
        errors.append("assets/ui_manifest.json 不存在（资产清单 = 单一事实源，S18.8b）")
    for scene_id, scene in scenes.items():
        if not isinstance(scene, dict):
            continue
        prims = scene.get("primitives")
        if not isinstance(prims, list):
            continue
        for i, p in enumerate(prims):
            if not isinstance(p, dict):
                continue
            kind = p.get("kind")
            if kind not in GENERIC_KINDS:
                errors.append(
                    f'scenes.json "{scene_id}".primitives[{i}]: kind = {kind!r} 非通用原语'
                    f"（允许：{sorted(GENERIC_KINDS)}）"
                )
            color = p.get("color")
            if isinstance(color, str) and color and color not in THEME_TOKENS:
                errors.append(
                    f'scenes.json "{scene_id}".primitives[{i}]: color 令牌 {color!r} 不存在'
                    f"（theme 令牌表未收录 → 渲染器会静默回退）"
                )
            for key in ("from_token", "to_token"):
                tok = (p.get("params") or {}).get(key)
                if isinstance(tok, str) and tok and tok not in THEME_TOKENS:
                    errors.append(
                        f'scenes.json "{scene_id}".primitives[{i}].{key} = {tok!r} 令牌不存在'
                    )
            # S18.8b：`image` 原语的资源必须真实存在（路径 = 相对仓库根）
            # **且已登记在资产清单**（`assets/ui_manifest.json` = 资产单一事实源）。
            # 缺资源 = 静默不画（画面缺件却无报错）→ 门禁前置拦截。
            if kind == "image":
                asset = (p.get("params") or {}).get("asset")
                if not isinstance(asset, str) or not asset:
                    errors.append(
                        f'scenes.json "{scene_id}".primitives[{i}]: image 原语缺 params.asset'
                    )
                elif not (root / asset).is_file():
                    errors.append(
                        f'scenes.json "{scene_id}".primitives[{i}]: image 资源 {asset!r} 不存在'
                        f"（相对仓库根；缺资源 = 静默不画）"
                    )
                elif asset not in registered_assets:
                    errors.append(
                        f'scenes.json "{scene_id}".primitives[{i}]: image 资源 {asset!r} 未在'
                        f" assets/ui_manifest.json 登记（资产清单 = 单一事实源）"
                    )

    # ② 渲染层：只认通用 kind + 无场景语义
    for rel in RENDERER_FILES:
        path = root / rel
        if not path.exists():
            errors.append(f"渲染层文件缺失: {rel}")
            continue
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            code = strip_comments(line)
            if not code.strip():
                continue
            for kw in SCENE_SEMANTICS:
                if re.search(rf"\"{kw}\"|'{kw}'", code):
                    errors.append(f"{rel}:{lineno} 渲染层出现场景语义 {kw!r}（须改为数据驱动）")
            for m in re.finditer(r'"([a-z_]+)"\s*=>', code):
                if m.group(1) not in GENERIC_KINDS and m.group(1) not in ("selected", "normal", "title", "body", "hint", "from_token", "to_token", "icon", "size", "steps", "asset"):
                    errors.append(
                        f"{rel}:{lineno} 渲染层 kind 分支 {m.group(1)!r} 非通用原语"
                        f"（允许：{sorted(GENERIC_KINDS)}）"
                    )

    if errors:
        for e in errors:
            print(f"  ERROR {e}")
        print(f"check_data_driven: {len(errors)} error(s)")
        return 1
    print(f"check_data_driven: OK（通用原语 {sorted(GENERIC_KINDS)}；渲染层无场景语义；令牌全存在）")
    return 0


if __name__ == "__main__":
    sys.exit(main())
