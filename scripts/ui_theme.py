#!/usr/bin/env python3
"""ui_theme：内容运营全主题创新·主题应用工具（S48.0）。

把「主题包」`assets/themes/<tid>/compose.json` 应用到游戏数据（零 Rust，S39 换首页 UI = 改数据 + 换资产）：
1. **构图**：`compose.json.landscape_primitives` 定点回注 `scenes.json` 的 `index.landscape.primitives`
   （read-modify-write + json.dumps(indent=1) 规范形；scenes.json 其余键原样保留 ⇒ 0 churn）。
2. **资产**：`compose.json.assets` 的 `<name>` → `<source_path>` 部署到 `assets/ui/<name>.png`
   （source 可 = 主题包内文件或既有 assets/ui 文件）。
3. **登记**：部署资产补登 `ui_manifest.json`（disk↔registry 一致）。
4. **校验**：`validate_data --check`（scenes.json 规范形 + 资产双向同步 0 违规）。

用法：
    python3 scripts/ui_theme.py apply --theme <tid> [--register] [--check]
    python3 scripts/ui_theme.py list

契约（14 号 §3.2 工具契约）：`--check` 只读（不写盘，0 合法 / 1 非法）+ 非 0 退出（主题缺失 /
compose 非法 / 资产缺失 / scenes.json 回注失败 / validate_data 违规）。幂等（同 compose 同结果）。
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
THEMES_DIR = ROOT / "assets" / "themes"
UI_DIR = ROOT / "assets" / "ui"
MANIFEST = ROOT / "assets" / "ui_manifest.json"


def load_json(p: Path) -> dict:
    with open(p, encoding="utf-8") as f:
        return json.load(f)


def save_json(p: Path, obj: dict) -> None:
    # indent=1 规范形（check_scenes_canonical 判据）
    p.write_text(json.dumps(obj, ensure_ascii=False, indent=1), encoding="utf-8")


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
    if "landscape_primitives" not in c:
        raise ValueError(f"{p.relative_to(ROOT)} 缺 `landscape_primitives`")
    return c


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
    """应用主题：回注构图 + 部署资产 + 登记 + 校验。"""
    c = load_compose(tid)
    prims = c["landscape_primitives"]
    if not isinstance(prims, list) or not prims:
        print(f"ui_theme: {tid} landscape_primitives 空或非数组", file=sys.stderr)
        return 1

    # ① 回注 scenes.json index.landscape.primitives（read-modify-write，0 churn 其余键）
    scenes = load_json(SCENES)
    idx = scenes.setdefault("index", {})
    land = idx.setdefault("landscape", {})
    land["primitives"] = prims
    save_json(SCENES, scenes)
    print(f"ui_theme: {tid} 回注 scenes.json index.landscape.primitives（{len(prims)} 原语）")

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
    parser = argparse.ArgumentParser(description="内容运营全主题创新·主题应用工具（S48.0）")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("apply", help="应用主题包（回注构图 + 部署资产 + 登记 + 校验）")
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
            # 只读预检
            try:
                c = load_compose(args.theme)
                print(f"ui_theme apply --check: {args.theme} compose 合法"
                      f"（{len(c['landscape_primitives'])} 原语，{len(c.get('assets', {}))} 资产）")
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
