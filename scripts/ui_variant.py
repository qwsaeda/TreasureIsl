#!/usr/bin/env python3
"""ui_variant：UI 资产视觉变体器（S42.0 同路径替换固化 → S44 平台 UI 向导）。

固化 S42.0 的 PIL `tint_cool`（Color/Brightness 增强 + R×/G×/B× 冷移）为正式工具：
选一组 `assets/ui/` 资产 + 着色参数 → 输出**同路径替换**（默认覆盖原资产，29 号 §3 同路径替换，
资产名/坐标/scenes.json 全不动 ⇒ 零 Rust + 零 scenes.json diff）或 `--suffix` 变体路径（设计稿迭看）。

用法：
    python3 scripts/ui_variant.py tint --assets a,b,c \\
        --color 0.32 --brightness 1.10 --rmul 0.88 --gmult 1.00 --bmult 1.20 \\
        [--suffix _v2] [--register] [--check]

契约（14 号 §3.2 工具契约）：幂等（同输入同产物，PIL 确定性）+ `--check` 只读（不写盘，0 合法 / 1 非法）
+ 非 0 退出（参数非法 / 资产缺失 / 写失败）。登记：14_tooling_catalog §3.2 + 04_tooling + README 索引。
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import date
from pathlib import Path

from PIL import Image, ImageEnhance

ROOT = Path(__file__).resolve().parent.parent
UI_DIR = ROOT / "assets" / "ui"
MANIFEST = ROOT / "assets" / "ui_manifest.json"


def tint_cool(img, color_enhance: float, brightness: float, r_mul: float, g_mul: float, b_mul: float):
    """S42.0 冷调风化：降饱和 + 调亮度 + 通道冷移（R×/G×/B×）。"""
    img = ImageEnhance.Color(img).enhance(color_enhance)
    img = ImageEnhance.Brightness(img).enhance(brightness)
    r, g, b, a = img.split()
    r = r.point(lambda v: max(0, min(255, int(v * r_mul))))
    g = g.point(lambda v: max(0, min(255, int(v * g_mul))))
    b = b.point(lambda v: max(0, min(255, int(v * b_mul))))
    return Image.merge("RGBA", (r, g, b, a))


def load_manifest() -> dict:
    if not MANIFEST.is_file():
        print(f"ui_variant: 缺 {MANIFEST.relative_to(ROOT)}", file=sys.stderr)
        return {}
    return json.load(open(MANIFEST, encoding="utf-8"))


def save_manifest(m: dict) -> None:
    json.dump(m, open(MANIFEST, "w", encoding="utf-8"), ensure_ascii=False, indent=1)


def find_asset(m: dict, name: str) -> dict | None:
    for a in m.get("assets", []):
        if a.get("name") == name:
            return a
    return None


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="UI 资产视觉变体器（S42.0 同路径替换固化）")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("tint", help="冷调风化着色（tint_cool）")
    p.add_argument("--assets", required=True, help="逗号分隔资产名（去 .png，如 index_plank_0,index_plank_1）")
    p.add_argument("--color", type=float, default=1.0, help="Color 增强（<1 降饱和，S42.0 v2b=0.32/0.42）")
    p.add_argument("--brightness", type=float, default=1.0, help="Brightness 增强（v2b=1.10/0.94）")
    p.add_argument("--rmul", type=float, default=1.0, help="R 通道乘数（冷移，v2b=0.88/0.92）")
    p.add_argument("--gmult", type=float, default=1.0, help="G 通道乘数（v2b=1.00）")
    p.add_argument("--bmult", type=float, default=1.0, help="B 通道乘数（冷移，v2b=1.20/1.12）")
    p.add_argument("--suffix", default="", help="输出文件名后缀（空=同路径替换；非空=变体路径如 _v2）")
    p.add_argument("--register", action="store_true", help="更新/新增 ui_manifest 条目（disk↔registry 一致）")
    p.add_argument("--check", action="store_true", help="只读：0 合法 / 1 非法（不写盘）")
    args = parser.parse_args(argv[1:])

    if args.cmd != "tint":
        parser.print_help()
        return 2

    names = [n.strip() for n in args.assets.split(",") if n.strip()]
    if not names:
        print("ui_variant: --assets 空", file=sys.stderr)
        return 1
    # 参数范围预检（防误传：通道乘数 ∈ [0,3]，Color/Brightness ∈ [0,3]）
    for label, v in (("--color", args.color), ("--brightness", args.brightness),
                     ("--rmul", args.rmul), ("--gmult", args.gmult), ("--bmult", args.bmult)):
        if not (0.0 <= v <= 3.0):
            print(f"ui_variant: {label}={v} 超范围 [0,3]", file=sys.stderr)
            return 1

    manifest = load_manifest()
    entries = []
    for name in names:
        src = UI_DIR / f"{name}.png"
        if not src.is_file():
            print(f"ui_variant: 资产缺失 {src.relative_to(ROOT)}", file=sys.stderr)
            return 1
        out_name = f"{name}{args.suffix}"
        out = UI_DIR / f"{out_name}.png"
        entries.append((name, src, out_name, out))

    if args.check:
        print(f"ui_variant tint --check: {len(entries)} 资产合法，将输出：")
        for name, src, out_name, out in entries:
            print(f"  {src.relative_to(ROOT)} → {out.relative_to(ROOT)}")
        return 0

    for name, src, out_name, out in entries:
        img = tint_cool(Image.open(src).convert("RGBA"),
                        args.color, args.brightness, args.rmul, args.gmult, args.bmult)
        img.save(out)
        print(f"ui_variant tint: {out.relative_to(ROOT)}（自 {name}，color={args.color} b={args.brightness} "
              f"R={args.rmul} G={args.gmult} B={args.bmult}）")

    if args.register:
        if not manifest:
            return 1
        changed = 0
        for name, src, out_name, out in entries:
            entry = find_asset(manifest, out_name)
            if entry is None:
                src_entry = find_asset(manifest, name)
                if src_entry is None:
                    print(f"ui_variant: 原资产 {name} 未登记 ui_manifest（无法继承 role）", file=sys.stderr)
                    return 1
                entry = json.loads(json.dumps(src_entry))  # 深拷贝（浅拷贝共享 params dict，会污染原条目）
                entry["name"] = out_name
                entry["file"] = f"assets/ui/{out_name}.png"
                entry["generated_at"] = str(date.today())
                manifest["assets"].append(entry)
            entry.setdefault("params", {})["source"] = "ui_variant"
            entry["params"]["note"] = (
                f"S44 视觉变体（tint_cool color={args.color} b={args.brightness} "
                f"R={args.rmul} G={args.gmult} B={args.bmult}）"
            )
            changed += 1
        save_manifest(manifest)
        print(f"ui_variant: ui_manifest 已更新 {changed} 条（source=ui_variant）")
        # F5 修复（S44.3 双审计 r1 B-P2）：register 后跑 validate_data --check（disk↔registry 一致性），
        # 违规非 0 退出（gate 第 10a 项统一入口口径；向导 variant 步「0 违规」判据由退出码传达）。
        proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "validate_data.py"), "--check"],
            cwd=ROOT, capture_output=True, text=True,
        )
        if proc.returncode != 0:
            print(f"ui_variant: validate_data --check 失败（{proc.returncode}）—— ui_manifest 与磁盘不一致，"
                  f"变体登记回滚不成立", file=sys.stderr)
            sys.stderr.write(proc.stdout[-2000:] + proc.stderr[-2000:])
            return 1
        print("ui_variant: validate_data --check OK（ui_manifest ↔ disk 一致，0 违规）")

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
