#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""UI 资产管线（清单驱动）：AI 生图 → 抠透明 → 规范化 → 入库 `assets/ui/`。

**可复用**的资产生产入口：以后任何界面/任何游戏都只改「资产清单」，
本脚本负责生成、抠图、规范化、校验；界面数据（`scenes.json`）只引用资产路径。

资产清单（单一事实源）：`assets/ui_manifest.json`
```json
{
  "version": 1,
  "roles": {
    "background": {"ratio": "9:16", "target": [1080, 1920], "key": "none",   "prompt": "..."},
    "medallion":  {"ratio": "1:1",  "target": [512, 512],  "key": "luma",   "prompt": "...{what}..."},
    "board":      {"ratio": "21:9", "target": [1024, 439], "key": "magenta", "prompt": "..."}
  },
  "assets": [
    {"name": "bg_night_sea", "role": "background", "file": "assets/ui/bg_night_sea.png",
     "params": {}, "generated_at": "2026-10-21"}
  ]
}
```

角色（role）= 资产规格预设：生图比例 + 目标尺寸 + 抠图模式 + 提示词模板。
抠图模式：`none`（不抠）/ `luma`（纯黑底 → 亮度抠 alpha）/ `magenta`（品红底 → 品红度抠 alpha）。

用法：
    python scripts/ui_assets.py build              # 补齐清单里缺失的资产（幂等）
    python scripts/ui_assets.py build --force      # 全部重生成
    python scripts/ui_assets.py build --only icon_compass
    python scripts/ui_assets.py add --name icon_x --role medallion --what "a ship anchor"
    python scripts/ui_assets.py rekey --name board          # 重抠图（改抠图参数后）
    python scripts/ui_assets.py normalize --name board      # 重规范化（尺寸/格式）
    python scripts/ui_assets.py verify                     # 校验（清单 ↔ 文件 ↔ scenes.json 引用）

环境变量：`AGNES_API_KEY`（Agnes API 密钥；仅 `build/add` 需要）。

原图（未抠图）落在 `evidence/ui_concept/raw_<name>.png`（不入库）。
"""
from __future__ import annotations

import argparse
import datetime as _dt
import json
import os
import pathlib
import subprocess
import sys

from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parent.parent
UI_DIR = ROOT / "assets" / "ui"
CONCEPT_DIR = ROOT / "evidence" / "ui_concept"
MANIFEST = ROOT / "assets" / "ui_manifest.json"
SCENES = ROOT / "assets" / "data" / "scenes.json"
API_KEY_ENV = "AGNES_API_KEY"

# ── 角色预设（新增角色只加这里；提示词里的 {what} 由 `add --what` 填充）──────────
DEFAULT_ROLES: dict[str, dict] = {
    "background": {
        "ratio": "9:16",
        "target": [1080, 1920],
        "key": "none",
        "prompt": (
            "A vertical 9:16 game background, night sky over a calm sea horizon, deep navy blue gradient "
            "sky with scattered small warm stars, soft moonlight glow behind thin drifting clouds on the "
            "upper right, a subtle dark sea with faint wave highlights at the bottom, warm gold accent "
            "glints near the horizon, painterly nautical adventure style, EMPTY center area, absolutely no "
            "text, no letters, no logo, no icons, no characters, no UI frames"
        ),
    },
    "medallion": {
        "ratio": "1:1",
        "target": [512, 512],
        "key": "luma",
        "prompt": (
            "a single circular gold medallion emblem, ornate thin gold line art, {what} in the center of "
            "the medallion, antique brass and gold filigree, glowing gold on a PURE SOLID BLACK "
            "background, no text, no letters, no scenery, no texture in the background, high contrast, "
            "centered"
        ),
    },
    "board": {
        "ratio": "21:9",
        "target": [1024, 439],
        "key": "magenta",
        "prompt": (
            "an ornate carved wooden signboard plaque, dark warm walnut wood with gold filigree carved "
            "border and curled corners, a completely EMPTY smooth wooden center panel with no carving "
            "inside, horizontal banner shape, the plaque is the ONLY object in the frame, isolated on a "
            "flat solid chroma-key MAGENTA #FF00FF background, no floor, no shadow, no backdrop, no text, "
            "no letters"
        ),
    },
    "portrait": {
        "ratio": "2:3",
        "target": [512, 768],
        "key": "luma",
        "prompt": (
            "a character portrait, {what}, heroic fantasy adventure game character art, warm gold rim "
            "light, painterly nautical treasure-hunting style, full upper body, glowing gold on a PURE "
            "SOLID BLACK background, no text, no letters, no scenery, high contrast, centered"
        ),
    },
}


# ────────────────────────────── 清单读写 ──────────────────────────────
def load_manifest() -> dict:
    if MANIFEST.exists():
        data = json.loads(MANIFEST.read_text(encoding="utf-8"))
    else:
        data = {"version": 1, "roles": {}, "assets": []}
    for name, role in DEFAULT_ROLES.items():
        data.setdefault("roles", {}).setdefault(name, role)
    return data


def save_manifest(data: dict) -> None:
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(
        json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    print(f"  清单已写: {MANIFEST.relative_to(ROOT)}")


def find_asset(data: dict, name: str) -> dict | None:
    return next((a for a in data.get("assets", []) if a.get("name") == name), None)


# ────────────────────────────── 生图 / 抠图 ──────────────────────────────
def api_key() -> str:
    key = os.environ.get(API_KEY_ENV, "")
    if not key:
        print(f"缺少环境变量 {API_KEY_ENV}（Agnes API 密钥）", file=sys.stderr)
        sys.exit(2)
    return key


def generate(prompt: str, out: pathlib.Path, ratio: str, size: str = "2K") -> pathlib.Path:
    cmd = [
        sys.executable, str(ROOT / "scripts" / "agnes_gen.py"),
        "--prompt", prompt, "--out", str(out), "--size", size, "--ratio", ratio,
    ]
    env = {**os.environ, API_KEY_ENV: api_key()}
    result = subprocess.run(cmd, env=env, capture_output=True, text=True,
                            encoding="utf-8", errors="replace")
    if result.returncode != 0 or not out.exists():
        print(result.stdout or "", result.stderr or "", file=sys.stderr)
        sys.exit(1)
    return out


def key_luma(src: pathlib.Path, dst: pathlib.Path, target: tuple[int, int]) -> None:
    """纯黑底 → alpha = 亮度；RGB 按 alpha 预乘（消掉暗底残留的蓝紫）。"""
    im = Image.open(src).convert("RGB")
    w, h = im.size
    out = Image.new("RGBA", (w, h))
    op, sp = out.load(), im.load()
    for y in range(h):
        for x in range(w):
            r, g, b = sp[x, y]
            lum = (r * 30 + g * 59 + b * 11) // 100
            a = 0 if lum < 58 else (255 if lum > 125 else (lum - 58) * 255 // 67)
            op[x, y] = (r * a // 255, g * a // 255, b * a // 255, a)
    _save_normalized(out, dst, target)


def key_magenta(src: pathlib.Path, dst: pathlib.Path, target: tuple[int, int]) -> None:
    """品红底 → alpha=0（边缘按品红度软过渡）。"""
    im = Image.open(src).convert("RGB")
    w, h = im.size
    out = Image.new("RGBA", (w, h))
    op, sp = out.load(), im.load()
    for y in range(h):
        for x in range(w):
            r, g, b = sp[x, y]
            mag = min(r, b) - g
            a = 0 if mag > 60 else (255 if mag < 20 else (60 - mag) * 255 // 40)
            op[x, y] = (0, 0, 0, 0) if a < 255 else (r, g, b, 255)
    _save_normalized(out, dst, target)


def _save_normalized(im: Image.Image, dst: pathlib.Path, target: tuple[int, int]) -> None:
    """规范化：缩放到目标尺寸（等比裁切/填充由缩略图近似）+ RGBA8 + 存盘。"""
    tw, th = target
    im = im.convert("RGBA")
    im.thumbnail((tw, th), Image.LANCZOS)
    dst.parent.mkdir(parents=True, exist_ok=True)
    im.save(dst, optimize=True)
    print(f"  规范化 -> {dst.relative_to(ROOT)} {im.size}")


def key_none(src: pathlib.Path, dst: pathlib.Path, target: tuple[int, int]) -> None:
    """不抠图：整图缩放到目标尺寸（背景用）。"""
    im = Image.open(src).convert("RGB").resize(target, Image.LANCZOS)
    _save_normalized(im, dst, target)


KEYERS = {"none": key_none, "luma": key_luma, "magenta": key_magenta}


# ────────────────────────────── 命令 ──────────────────────────────
def build_asset(data: dict, asset: dict, force: bool) -> bool:
    name = asset["name"]
    role_name = asset.get("role", "")
    role = data.get("roles", {}).get(role_name)
    if role is None:
        print(f"  FAIL  资产 {name}: 未知角色 {role_name!r}", file=sys.stderr)
        return False
    dst = ROOT / asset["file"]
    raw = CONCEPT_DIR / f"raw_{name}.png"
    if dst.exists() and not force:
        print(f"  SKIP  {name}（已存在；--force 可重生成）")
        return True
    prompt = role["prompt"]
    for k, v in (asset.get("params") or {}).items():
        prompt = prompt.replace("{" + k + "}", str(v))
    print(f"  GEN   {name}（role={role_name} ratio={role['ratio']}）")
    generate(prompt, raw, role.get("ratio", "1:1"), role.get("size", "2K"))
    target = tuple(role.get("target", [512, 512]))
    KEYERS[role.get("key", "none")](raw, dst, target)  # type: ignore[arg-type]
    asset["generated_at"] = _dt.date.today().isoformat()
    return True


def cmd_build(args: argparse.Namespace) -> int:
    data = load_manifest()
    ok = True
    for asset in data.get("assets", []):
        if args.only and asset["name"] != args.only:
            continue
        ok &= build_asset(data, asset, args.force)
    save_manifest(data)
    return 0 if ok else 1


def cmd_add(args: argparse.Namespace) -> int:
    data = load_manifest()
    if find_asset(data, args.name):
        print(f"资产已存在: {args.name}", file=sys.stderr)
        return 1
    role = data.get("roles", {}).get(args.role)
    if role is None:
        print(f"未知角色: {args.role}（可用: {sorted(data.get('roles', {}))}）", file=sys.stderr)
        return 1
    params = dict(args.params or {})
    if args.what:
        params["what"] = args.what
    asset = {
        "name": args.name,
        "role": args.role,
        "file": f"assets/ui/{args.name}.png",
        "params": params,
        "generated_at": _dt.date.today().isoformat(),
    }
    data.setdefault("assets", []).append(asset)
    build_asset(data, asset, force=False)
    save_manifest(data)
    return 0


def cmd_rekey(args: argparse.Namespace) -> int:
    data = load_manifest()
    asset = find_asset(data, args.name)
    if asset is None:
        print(f"清单里没有资产: {args.name}", file=sys.stderr)
        return 1
    role = data["roles"][asset["role"]]
    raw = CONCEPT_DIR / f"raw_{args.name}.png"
    if not raw.exists():
        print(f"原图不存在（先生成）: {raw}", file=sys.stderr)
        return 1
    KEYERS[role.get("key", "none")](raw, ROOT / asset["file"], tuple(role.get("target", [512, 512])))
    return 0


def cmd_normalize(args: argparse.Namespace) -> int:
    data = load_manifest()
    asset = find_asset(data, args.name)
    if asset is None:
        print(f"清单里没有资产: {args.name}", file=sys.stderr)
        return 1
    role = data["roles"][asset["role"]]
    src = ROOT / asset["file"]
    im = Image.open(src).convert("RGBA")
    _save_normalized(im, src, tuple(role.get("target", [512, 512])))
    return 0


def cmd_verify(args: argparse.Namespace) -> int:
    """校验：清单 ↔ 文件 ↔ `scenes.json` 引用（三方一致）。"""
    data = load_manifest()
    errors: list[str] = []
    registered: set[str] = set()
    for asset in data.get("assets", []):
        path = ROOT / asset.get("file", "")
        if not path.is_file():
            errors.append(f"清单资产 {asset.get('name')} 的文件不存在: {asset.get('file')}")
        registered.add(asset.get("file", ""))
    # scenes.json 的 image 原语引用必须在清单里登记（防「用了没登记的资产」）
    if SCENES.exists():
        scenes = json.loads(SCENES.read_text(encoding="utf-8"))
        for sid, scene in scenes.items():
            if not isinstance(scene, dict):
                continue
            for i, p in enumerate(scene.get("primitives", []) or []):
                if p.get("kind") != "image":
                    continue
                asset_path = (p.get("params") or {}).get("asset", "")
                if asset_path and asset_path not in registered:
                    errors.append(
                        f'scenes.json "{sid}".primitives[{i}]: 资产 {asset_path!r} 未在 ui_manifest.json 登记'
                    )
    # 孤儿文件：assets/ui 下存在但未登记
    if UI_DIR.exists():
        for png in sorted(UI_DIR.glob("*.png")):
            rel = f"assets/ui/{png.name}"
            if rel not in registered:
                errors.append(f"孤儿资产文件（未登记）: {rel}")
    if errors:
        for e in errors:
            print(f"  ERROR {e}")
        print(f"ui_assets verify: {len(errors)} error(s)")
        return 1
    print(f"ui_assets verify: OK（{len(data.get('assets', []))} 个资产，登记/文件/引用三方一致）")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description="UI 资产管线（清单驱动）")
    sub = ap.add_subparsers(dest="cmd", required=True)

    b = sub.add_parser("build", help="补齐清单里缺失的资产（幂等）")
    b.add_argument("--force", action="store_true", help="已存在也重生成")
    b.add_argument("--only", help="只处理某个资产名")
    b.set_defaults(func=cmd_build)

    a = sub.add_parser("add", help="登记新资产并生成")
    a.add_argument("--name", required=True)
    a.add_argument("--role", required=True)
    a.add_argument("--what", help="图案描述（填入提示词的 {what}）")
    a.add_argument("--params", nargs="*", help="额外参数 k=v（填入提示词的占位符）")
    a.set_defaults(func=cmd_add)

    r = sub.add_parser("rekey", help="重抠图")
    r.add_argument("--name", required=True)
    r.set_defaults(func=cmd_rekey)

    n = sub.add_parser("normalize", help="重规范化（尺寸/格式）")
    n.add_argument("--name", required=True)
    n.set_defaults(func=cmd_normalize)

    v = sub.add_parser("verify", help="校验（清单 ↔ 文件 ↔ scenes.json 引用）")
    v.set_defaults(func=cmd_verify)

    args = ap.parse_args()
    if getattr(args, "params", None):
        args.params = dict(p.split("=", 1) for p in args.params if "=" in p)
    UI_DIR.mkdir(parents=True, exist_ok=True)
    CONCEPT_DIR.mkdir(parents=True, exist_ok=True)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
