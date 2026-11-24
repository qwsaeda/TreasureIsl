#!/usr/bin/env python3
"""ui_theme：内容运营全主题创新·主题应用工具（S48.0，S85.2 写点迁移）。

把「主题包」`assets/themes/<tid>/compose.json` 应用到游戏数据（零 Rust，S39 换首页 UI = 改数据 + 换资产）：
1. **构图覆盖**（S85.2 起）：`compose.json` 的组件覆盖列表（旧键 `landscape_primitives`，
   语义 = 组件覆盖）按 role 导出 `role → 资产路径`，回注
   `assets/data/ui_schema/components_v1.json` 的 `scenes.index.overrides`（新 schema 唯一写点；
   S84 后 `scenes.json` 已无 `index.landscape` 段，不再回注 scenes.json）。
   幂等：同 compose 同结果；其余场景/元素原样保留（read-modify-write）。
2. **资产**：`compose.json.assets` 的 `<name>` → `<source_path>` 部署到 `assets/ui/<name>.png`
   （source 可 = 主题包内文件或既有 assets/ui 文件）。
3. **登记**：部署资产补登 `ui_manifest.json`（disk↔registry 一致）。
4. **校验**：`validate_data --check`（scenes.json 规范形 + 资产双向同步 0 违规）。

role 解析（33 号单一事实源 `asset_roles.json`）：优先 prim/params 显式 `role`；否则按资产
basename 反查注册表（去 `<tid>_` 前缀 + `index_` 变体）；仍未命中则用去前缀 stem 作覆盖键
（schema `overrides.additionalProperties` 允许任意字符串键）。

用法：
    python3 scripts/ui_theme.py apply --theme <tid> [--register] [--check]
    python3 scripts/ui_theme.py list

契约（14 号 §3.2 工具契约）：`--check` 只读（不写盘，0 合法 / 1 非法）+ 非 0 退出（主题缺失 /
compose 非法 / 资产缺失 / schema 写点异常 / validate_data 违规）。幂等（同 compose 同结果）。
登记：14_tooling_catalog + 04_tooling + README 索引（S48.0）。
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCENES = ROOT / "assets" / "data" / "scenes.json"
COMPONENTS = ROOT / "assets" / "data" / "ui_schema" / "components_v1.json"
ROLES_FILE = ROOT / "assets" / "data" / "ui_schema" / "asset_roles.json"
THEMES_DIR = ROOT / "assets" / "themes"
UI_DIR = ROOT / "assets" / "ui"
MANIFEST = ROOT / "assets" / "ui_manifest.json"


def load_json(p: Path) -> dict:
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def save_json(p: Path, obj: dict, indent: int = 1) -> None:
    # indent=1 规范形（check_scenes_canonical 判据）；components_v1.json 用 2（同文件现状）。
    p.write_text(json.dumps(obj, ensure_ascii=False, indent=indent), encoding="utf-8")


def theme_dir(tid: str) -> Path:
    if not tid or not tid.isalnum() and "_" not in tid:
        raise ValueError(f"theme id `{tid}` 非法")
    return THEMES_DIR / tid


def compose_path(tid: str) -> Path:
    return theme_dir(tid) / "compose.json"


def load_compose(tid: str) -> dict:
    p = compose_path(tid)
    if not p.is_file():
        raise FileNotFoundError(f"主题包缺 compose.json：{p.relative_to(ROOT)}")
    c = load_json(p)
    # 新 compose 可用 `overrides`（role→path 字典）或保留旧键 `landscape_primitives`
    # （语义改为「组件覆盖列表」）；两者皆无 = 非法。
    if "landscape_primitives" not in c and "overrides" not in c:
        raise ValueError(f"{p.relative_to(ROOT)} 缺 `landscape_primitives`/`overrides`")
    return c


def load_roles() -> dict[str, str]:
    """资产角色注册表（33 号单一事实源）：role → 资产路径。缺位 = 空表（不阻塞 apply）。"""
    if not ROLES_FILE.is_file():
        return {}
    data = load_json(ROLES_FILE)
    roles = data.get("roles", {})
    return roles if isinstance(roles, dict) else {}


def role_for_asset(asset: str, tid: str, roles: dict[str, str]) -> str:
    """资产路径 → 覆盖角色键：显式 role > 注册表反查 > 去 `<tid>_` 前缀 stem。"""
    if not isinstance(asset, str) or not asset.startswith("assets/"):
        return ""
    stem = Path(asset).stem
    candidates = [stem]
    if tid and stem.startswith(f"{tid}_"):
        stripped = stem[len(tid) + 1:]
        candidates += [stripped, f"index_{stripped}"]
    if stem.startswith("index_"):
        candidates.append(stem[len("index_"):])
    basename_to_role = {Path(p).name: role for role, p in roles.items()}
    for cand in candidates:
        role = basename_to_role.get(f"{cand}.png")
        if role:
            return role
    role = next((r for r, p in roles.items() if p == asset), "")
    if role:
        return role
    return candidates[1] if len(candidates) > 1 else stem


def derive_overrides(compose: dict, tid: str, roles: dict[str, str]) -> dict[str, str]:
    """从主题 compose 导出 components_v1.json 写点：role → 资产路径。"""
    overrides: dict[str, str] = {}
    direct = compose.get("overrides")
    if isinstance(direct, dict):
        for role, path in direct.items():
            if isinstance(role, str) and isinstance(path, str) and path.startswith("assets/"):
                overrides[role] = path
        return overrides
    prims = compose.get("landscape_primitives", [])
    if not isinstance(prims, list):
        return overrides
    for prim in prims:
        if not isinstance(prim, dict):
            continue
        params = prim.get("params", {}) if isinstance(prim.get("params"), dict) else {}
        asset = (
            params.get("asset") or params.get("image") or params.get("src")
            or params.get("file") or prim.get("asset")
        )
        if not isinstance(asset, str) or not asset.startswith("assets/"):
            continue
        explicit = prim.get("role") or params.get("role")
        role = explicit if isinstance(explicit, str) and explicit else role_for_asset(asset, tid, roles)
        if role:
            overrides[role] = asset
    return overrides


def validate_theme_target(root: Path = ROOT) -> list[str]:
    """只读校验主题写点：components_v1.json 存在 + scenes.index.elements 非空 + overrides 为对象。"""
    problems: list[str] = []
    p = root / "assets" / "data" / "ui_schema" / "components_v1.json"
    if not p.is_file():
        problems.append(f"缺主题写点 {p.relative_to(root)}")
        return problems
    try:
        data = load_json(p)
    except (OSError, json.JSONDecodeError) as e:
        problems.append(f"components_v1.json 解析失败：{e}")
        return problems
    scenes = data.get("scenes")
    if not isinstance(scenes, dict) or not isinstance(scenes.get("index"), dict):
        problems.append("components_v1.json 缺 scenes.index")
        return problems
    idx = scenes["index"]
    if not isinstance(idx.get("elements"), list) or not idx["elements"]:
        problems.append("components_v1.json scenes.index.elements 空或非数组")
    if not isinstance(idx.get("overrides"), dict):
        problems.append("components_v1.json scenes.index.overrides 非对象（主题写点异常）")
    return problems


def load_manifest() -> dict:
    if not MANIFEST.is_file():
        return {}
    return load_json(MANIFEST)


def save_manifest(m: dict) -> None:
    json.dump(m, open(MANIFEST, "w", encoding="utf-8"), ensure_ascii=False, indent=1)


def find_asset(m: dict, name: str) -> dict | None:
    for a in m.get("assets", []):
        if a.get("name") == name:
            return a
    return None


def apply_theme(tid: str, register: bool) -> int:
    """应用主题：回注 components_v1.json overrides + 部署资产 + 登记 + 校验。"""
    c = load_compose(tid)
    roles = load_roles()
    overrides = derive_overrides(c, tid, roles)
    if not overrides:
        print(f"ui_theme: {tid} 组件覆盖列表为空/无有效资产路径", file=sys.stderr)
        return 1

    # ① 回注 components_v1.json scenes.index.overrides（S85.2 唯一写点，0 其余场景 churn）
    if not COMPONENTS.is_file():
        print(f"ui_theme: 缺主题写点 {COMPONENTS.relative_to(ROOT)}", file=sys.stderr)
        return 1
    components = load_json(COMPONENTS)
    scenes = components.setdefault("scenes", {})
    idx = scenes.setdefault("index", {})
    idx["overrides"] = overrides
    save_json(COMPONENTS, components, indent=2)
    print(f"ui_theme: {tid} 回注 components_v1.json scenes.index.overrides（{len(overrides)} role→路径）")

    # ② 部署资产（compose.assets：name → source_path）
    assets_map = c.get("assets", {})
    deployed = []
    for name, src_rel in assets_map.items():
        src = ROOT / src_rel
        if not src.is_file():
            print(f"ui_theme: 资产源缺失 {src_rel}（{tid} 的 {name}）", file=sys.stderr)
            return 1
        out = UI_DIR / f"{name}.png"
        shutil.copyfile(src, out)
        deployed.append((name, src_rel, out.relative_to(ROOT)))

    # ③ 登记 ui_manifest（继承既有 role 或默认 asset）
    manifest = load_manifest()
    if register:
        for name, src_rel, out_rel in deployed:
            entry = find_asset(manifest, name)
            if entry is None:
                entry = {
                    "name": name,
                    "role": "asset",
                    "file": str(out_rel),
                    "params": {"source": "ui_theme"},
                    "generated_at": str(date.today()),
                }
                manifest["assets"].append(entry)
            entry.setdefault("params", {})["source"] = "ui_theme"
            entry["params"]["note"] = f"S48.0 主题 {tid}（{src_rel}）"
        save_manifest(manifest)
        print(f"ui_theme: ui_manifest 已更新 {len(deployed)} 条（source=ui_theme）")

    for name, src_rel, out_rel in deployed:
        print(f"ui_theme: 部署 {out_rel}（自 {src_rel}）")

    # ④ validate_data --check（scenes.json 规范形 + 资产双向同步）
    proc = subprocess.run(
        [sys.executable, str(ROOT / "scripts" / "validate_data.py"), "--check"],
        cwd=ROOT, capture_output=True, text=True,
    )
    if proc.returncode != 0:
        print(f"ui_theme: validate_data --check 失败（{proc.returncode}）", file=sys.stderr)
        sys.stderr.write(proc.stdout[-2000:] + proc.stderr[-2000:])
        return 1
    print("ui_theme: validate_data --check OK（scenes.json 规范形 + disk↔registry 0 违规）")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="内容运营全主题创新·主题应用工具（S48.0，S85.2 写点迁移）")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("apply", help="应用主题包（回注 schema overrides + 部署资产 + 登记 + 校验）")
    p.add_argument("--theme", required=True, help="主题包 id（assets/themes/<tid>/）")
    p.add_argument("--register", action="store_true", help="更新/新增 ui_manifest 条目")
    p.add_argument("--check", action="store_true", help="只读：0 合法 / 1 非法（不写盘）")
    sub.add_parser("list", help="列出已装主题包")
    args = parser.parse_args(argv[1:])

    if args.cmd == "list":
        if not THEMES_DIR.is_dir():
            print("ui_theme: 无主题包目录")
            return 0
        for d in sorted(THEMES_DIR.iterdir()):
            if d.is_dir() and (d / "compose.json").is_file():
                print(f"  {d.name}")
        return 0

    if args.cmd == "apply":
        if args.check:
            # 只读预检：compose 合法 + 写点（components_v1.json）结构合法 + 覆盖列表非空
            try:
                c = load_compose(args.theme)
                overrides = derive_overrides(c, args.theme, load_roles())
                problems = validate_theme_target()
                if problems:
                    for msg in problems:
                        print(f"ui_theme apply --check ERROR: {msg}", file=sys.stderr)
                    return 1
                if not overrides:
                    print(f"ui_theme apply --check ERROR: {args.theme} 无有效组件覆盖", file=sys.stderr)
                    return 1
                print(f"ui_theme apply --check: {args.theme} compose 合法"
                      f"（导出 {len(overrides)} role→路径，{len(c.get('assets', {}))} 资产）"
                      "；components_v1.json 写点结构合法")
                return 0
            except (FileNotFoundError, ValueError) as e:
                print(f"ui_theme --check 失败：{e}", file=sys.stderr)
                return 1
        try:
            return apply_theme(args.theme, args.register)
        except (FileNotFoundError, ValueError) as e:
            print(f"ui_theme: {e}", file=sys.stderr)
            return 1

    parser.print_help()
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
