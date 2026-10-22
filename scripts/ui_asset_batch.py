#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""UI 资产**批量**工具（清单 + 批次规格双源，AI 生图 → 抠透明 → 规范化 → 审阅拼版）。

`ui_assets.py` 是**单资产**入口（add/build/rekey/normalize/verify）。当一次要产出
**一组**资产（换肤 / 新场景整组贴图 / 多形状图标）时，单资产入口会退化成「写一遍
临时脚本、prompt 抄一遍、抠图失败无人发现」——本工具补上这段缺口（EXP-113）。

设计要点（复用不重复实现）：
- **抠图 / 规范化 / 生图全部委托 `ui_assets.py`**（`import ui_assets`），不复制 keyer 逻辑；
- **批次规格 = 仓库内 JSON**（`assets/ui_batches/<batch>.json`）：角色预设 + 资产条目，
  可评审、可 diff、可复现；`assets/ui_manifest.json` 仍是**资产唯一事实源**（本工具只做
  upsert，不另立门户）；
- **抠图可行性预检**（`check`）：`luma` 角色要求纯黑底、`magenta` 角色要求品红底，
  底色不对时 keyer 会把整张图留成不透明 —— 预检在 install 之前就报出来。

用法：
    python scripts/ui_asset_batch.py register  --spec assets/ui_batches/main_map_v4.json
    python scripts/ui_asset_batch.py generate  --spec assets/ui_batches/main_map_v4.json [--workers 4] [--only a,b]
    python scripts/ui_asset_batch.py install   --spec assets/ui_batches/main_map_v4.json [--only a,b]
    python scripts/ui_asset_batch.py build     --spec assets/ui_batches/main_map_v4.json   # register+generate+install
    python scripts/ui_asset_batch.py check     --spec assets/ui_batches/main_map_v4.json   # 只读；有问题非 0
    python scripts/ui_asset_batch.py --check    --spec assets/ui_batches/main_map_v4.json   # 同上（工具契约别名）
    python scripts/ui_asset_batch.py sheet     --spec assets/ui_batches/main_map_v4.json --out evidence/ui_concept/sheet.png

退出码：0 = 全绿；1 = 有问题（check 发现漂移 / 生成失败）。
幂等：register 可重复执行（upsert）；generate 已有原图则跳过（`--force` 强制重生）；
install 可重复执行（重新抠图 + 规范化，产物确定）。
"""
from __future__ import annotations

import argparse
import concurrent.futures
import json
import pathlib
import sys

from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import ui_assets  # noqa: E402  (复用单资产管线的 keyer / generate，不重复实现)

MANIFEST = ROOT / "assets" / "ui_manifest.json"
CONCEPT_DIR = ROOT / "evidence" / "ui_concept"

# 抠图可行性阈值（与 ui_assets.key_luma / key_magenta 的判定点同源）。
LUMA_CUTOFF = 58
MAGENTA_CUTOFF = 20


# ────────────────────────────── 规格读取 ──────────────────────────────


def load_spec(path: pathlib.Path) -> dict:
    if not path.is_file():
        print(f"批次规格不存在: {path}", file=sys.stderr)
        raise SystemExit(2)
    spec = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(spec.get("assets"), list) or not spec["assets"]:
        print(f"批次规格无 assets 条目: {path}", file=sys.stderr)
        raise SystemExit(2)
    return spec


def resolve_prompt(spec: dict, asset: dict, roles: dict) -> str:
    role = roles.get(asset.get("role", ""), {})
    prompt = asset.get("prompt") or role.get("prompt", "")
    for k, v in (asset.get("params") or {}).items():
        prompt = prompt.replace("{" + k + "}", str(v))
    return prompt


def raw_path(name: str) -> pathlib.Path:
    return CONCEPT_DIR / f"raw_{name}.png"


def asset_path(asset: dict) -> pathlib.Path:
    return ROOT / asset.get("file", f"assets/ui/{asset.get('name', '')}.png")


# ────────────────────────────── register ──────────────────────────────


def cmd_register(spec: dict, _args: argparse.Namespace) -> int:
    data = ui_assets.load_manifest()
    roles = data.setdefault("roles", {})
    for name, role in (spec.get("roles") or {}).items():
        roles[name] = role
        print(f"  role  {name}（ratio={role.get('ratio')} key={role.get('key')}）")
    assets = data.setdefault("assets", [])
    by_name = {a.get("name"): a for a in assets}
    for asset in spec["assets"]:
        name = asset["name"]
        entry = {
            "name": name,
            "role": asset["role"],
            "file": asset.get("file", f"assets/ui/{name}.png"),
            "params": asset.get("params") or {},
        }
        if asset.get("prompt"):
            entry["prompt"] = asset["prompt"]
        old = by_name.get(name)
        if old is None:
            entry["generated_at"] = ui_assets._dt.date.today().isoformat()
            assets.append(entry)
            print(f"  ADD   {name}")
        else:
            old.update({k: v for k, v in entry.items() if k != "generated_at"})
            print(f"  UPSERT {name}")
    ui_assets.save_manifest(data)
    return 0


# ────────────────────────────── generate ──────────────────────────────


def _generate_one(name: str, prompt: str, ratio: str, size: str, force: bool) -> tuple[str, str]:
    out = raw_path(name)
    if out.exists() and out.stat().st_size > 10_000 and not force:
        return name, "SKIP"
    try:
        ui_assets.generate(prompt, out, ratio, size)
    except SystemExit as exc:  # ui_assets.generate 失败即退出
        return name, f"FAIL({exc.code})"
    if not out.exists():
        return name, "FAIL(无产物)"
    return name, f"OK({out.stat().st_size // 1024}KB)"


def cmd_generate(spec: dict, args: argparse.Namespace) -> int:
    data = ui_assets.load_manifest()
    roles = data["roles"]
    only = set(args.only.split(",")) if args.only else None
    jobs = []
    for asset in spec["assets"]:
        name = asset["name"]
        if only and name not in only:
            continue
        role = roles.get(asset["role"])
        if role is None:
            print(f"  FAIL  {name}: 未知角色 {asset['role']!r}", file=sys.stderr)
            return 1
        jobs.append(
            (
                name,
                resolve_prompt(spec, asset, roles),
                role.get("ratio", "1:1"),
                role.get("size", "2K"),
                args.force,
            )
        )
    rc = 0
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, args.workers)) as pool:
        for name, status in pool.map(lambda j: _generate_one(*j), jobs):
            print(f"  GEN   {name}: {status}", flush=True)
            if status.startswith("FAIL"):
                rc = 1
    return rc


# ────────────────────────────── install ──────────────────────────────


def cmd_install(spec: dict, args: argparse.Namespace) -> int:
    data = ui_assets.load_manifest()
    roles = data["roles"]
    only = set(args.only.split(",")) if args.only else None
    rc = 0
    for asset in spec["assets"]:
        name = asset["name"]
        if only and name not in only:
            continue
        role = roles.get(asset["role"])
        if role is None:
            print(f"  FAIL  {name}: 未知角色 {asset['role']!r}", file=sys.stderr)
            rc = 1
            continue
        raw = raw_path(name)
        if not raw.exists():
            print(f"  FAIL  {name}: 原图不存在（先 generate）: {raw}")
            rc = 1
            continue
        dst = asset_path(asset)
        try:
            ui_assets.KEYERS[role.get("key", "none")](raw, dst, tuple(role.get("target", [512, 512])))
            print(f"  OK    {name} -> {dst.relative_to(ROOT)}")
        except Exception as exc:  # noqa: BLE001 - 逐资产失败不影响其余
            print(f"  FAIL  {name}: {exc}")
            rc = 1
    return rc


# ────────────────────────────── check（只读） ──────────────────────────────


def _luma_stats(path: pathlib.Path) -> tuple[float, float]:
    """返回 (四角平均亮度, 暗像素占比)。"""
    import statistics

    im = Image.open(path).convert("L")
    w, h = im.size
    px = im.load()

    def corner_mean(x0: int, y0: int) -> float:
        s = 0
        n = 0
        for y in range(max(0, y0), min(h, y0 + 20)):
            for x in range(max(0, x0), min(w, x0 + 20)):
                s += px[x, y]
                n += 1
        return s / max(1, n)

    means = [
        corner_mean(0, 0),
        corner_mean(w - 20, 0),
        corner_mean(0, h - 20),
        corner_mean(w - 20, h - 20),
    ]
    hist = im.histogram()
    dark = sum(hist[:LUMA_CUTOFF]) / max(1, w * h)
    return statistics.fmean(means), dark


def _magenta_stats(path: pathlib.Path) -> float:
    im = Image.open(path).convert("RGB")
    w, h = im.size
    px = im.load()
    hit = 0
    total = 0
    for y in range(0, h, 8):
        for x in range(0, w, 8):
            r, g, b = px[x, y]
            total += 1
            if min(r, b) - g > MAGENTA_CUTOFF:
                hit += 1
    return hit / max(1, total)


def cmd_check(spec: dict, _args: argparse.Namespace) -> int:
    data = ui_assets.load_manifest()
    roles = data["roles"]
    manifest_assets = {a.get("name"): a for a in data.get("assets", [])}
    errors: list[str] = []
    warns: list[str] = []

    for name, role in (spec.get("roles") or {}).items():
        if roles.get(name) != role:
            errors.append(f"角色 {name} 与清单不一致（清单未同步）")
    for asset in spec["assets"]:
        name = asset["name"]
        m = manifest_assets.get(name)
        if m is None:
            errors.append(f"{name}: 未登记到 ui_manifest.json（先 register）")
            continue
        if m.get("role") != asset["role"]:
            errors.append(f"{name}: 清单 role={m.get('role')} ≠ 规格 role={asset['role']}")
        if m.get("file") != asset.get("file", f"assets/ui/{name}.png"):
            errors.append(f"{name}: 清单 file 与规格不一致")
        if m.get("prompt") != asset.get("prompt"):
            errors.append(f"{name}: 清单 prompt 与规格不一致（提示词漂移）")
        dst = asset_path(asset)
        if not dst.is_file():
            errors.append(f"{name}: 产物缺失 {dst.relative_to(ROOT)}（先 generate+install）")
            continue
        key = roles.get(asset["role"], {}).get("key", "none")
        if key != "none":
            im = Image.open(dst).convert("RGBA")
            bbox = im.split()[3].getbbox()
            if bbox and bbox[0] == 0 and bbox[1] == 0 and bbox[2] == im.size[0] and bbox[3] == im.size[1]:
                errors.append(f"{name}: 抠图疑似失败（alpha 覆盖整幅 = 底色没被抠掉）")
        raw = raw_path(name)
        if raw.is_file():
            if key == "luma":
                corner, dark = _luma_stats(raw)
                if corner > LUMA_CUTOFF:
                    warns.append(
                        f"{name}: 原图底色偏亮（四角均亮 {corner:.0f} > {LUMA_CUTOFF}）"
                        f" ⇒ luma 抠图可能失效，建议提示词要求纯黑底并重生"
                    )
            elif key == "magenta":
                frac = _magenta_stats(raw)
                if frac < 0.05:
                    warns.append(
                        f"{name}: 原图品红底占比仅 {frac:.1%} ⇒ magenta 抠图可能失效，建议重生"
                    )
    for w in warns:
        print(f"  WARN  {w}")
    for e in errors:
        print(f"  ERROR {e}")
    print(f"ui_asset_batch check: {len(errors)} error(s), {len(warns)} warn(s)")
    return 1 if errors else 0


# ────────────────────────────── sheet（审阅拼版） ──────────────────────────────


def cmd_sheet(spec: dict, args: argparse.Namespace) -> int:
    from PIL import ImageDraw, ImageFont

    cell = args.cell
    names = [a["name"] for a in spec["assets"] if not args.only or a["name"] in args.only.split(",")]
    if not names:
        print("拼版无条目", file=sys.stderr)
        return 1
    cols = min(args.cols, len(names))
    rows = (len(names) + cols - 1) // cols
    try:
        font = ImageFont.truetype(r"C:\Windows\Fonts\msyh.ttc", 18)
    except OSError:
        font = ImageFont.load_default()
    sheet = Image.new("RGB", (cols * (cell + 12) + 12, rows * (cell + 34) + 12), (40, 36, 32))
    draw = ImageDraw.Draw(sheet)
    for i, name in enumerate(names):
        path = ROOT / "assets" / "ui" / f"{name}.png"
        if not path.is_file():
            print(f"  MISS {name}（先 install）")
            continue
        im = Image.open(path).convert("RGBA")
        im.thumbnail((cell, cell), Image.LANCZOS)
        bg = Image.new("RGBA", (cell, cell), (28, 26, 24, 255))
        bg.alpha_composite(im, ((cell - im.size[0]) // 2, (cell - im.size[1]) // 2))
        x = 12 + (i % cols) * (cell + 12)
        y = 12 + (i // cols) * (cell + 34)
        sheet.paste(bg.convert("RGB"), (x, y))
        draw.text((x + 4, y + cell + 4), name, fill=(235, 200, 110), font=font)
    out = pathlib.Path(args.out)
    if not out.is_absolute():
        out = ROOT / out
    out.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    print(f"  拼版已存: {out}（{len(names)} 项）")
    return 0


# ────────────────────────────── CLI ──────────────────────────────


def main() -> int:
    argv = sys.argv[1:]
    check_alias = "--check" in argv
    if check_alias:
        argv = [a for a in argv if a != "--check"]
    ap = argparse.ArgumentParser(description="UI 资产批量工具（规格驱动：登记/生图/抠图/校验/拼版）")
    ap.add_argument(
        "cmd",
        choices=["register", "generate", "install", "build", "check", "sheet"],
        help="子命令（--check = check 的只读别名）",
    )
    ap.add_argument("--spec", required=True, help="批次规格 JSON 路径")
    ap.add_argument("--workers", type=int, default=4, help="并行生图并发数（默认 4）")
    ap.add_argument("--only", help="只处理指定资产（逗号分隔）")
    ap.add_argument("--force", action="store_true", help="强制重生原图")
    ap.add_argument("--out", default="evidence/ui_concept/batch_sheet.png", help="拼版输出路径")
    ap.add_argument("--cell", type=int, default=220, help="拼版单格边长")
    ap.add_argument("--cols", type=int, default=6, help="拼版列数")
    args = ap.parse_args(argv)

    spec = load_spec(pathlib.Path(args.spec))
    if check_alias:
        args.cmd = "check"
    print(f"[ui_asset_batch] {args.cmd} spec={args.spec} batch={spec.get('batch', '?')}")
    if args.cmd == "register":
        return cmd_register(spec, args)
    if args.cmd == "generate":
        return cmd_generate(spec, args)
    if args.cmd == "install":
        return cmd_install(spec, args)
    if args.cmd == "build":
        rc = cmd_register(spec, args)
        rc |= cmd_generate(spec, args)
        rc |= cmd_install(spec, args)
        return rc
    if args.cmd == "check":
        return cmd_check(spec, args)
    if args.cmd == "sheet":
        return cmd_sheet(spec, args)
    return 2


if __name__ == "__main__":
    sys.exit(main())
