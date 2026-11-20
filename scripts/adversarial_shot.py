#!/usr/bin/env python3
"""AI 对抗对局快照出图（飞轮 R21 / S61）：规则层对局全程 → PNG。

把 `adversarial_common.rs` 驱动器跑出的「N 个 AI 席自动对抗完整对局」dump 成
**各环节 PNG**：31 格环赛道（BOSS@15 + 各席当前格棋子）+ 席位状态表（金/兵力/存活/终局胜者）。
零游戏依赖（纯 Python + PIL，读 JSON 快照），是「AI 自动对抗真实 E2E 玩」的**规则层过程图**
载体（表现层动态帧截不动，见 `evidence/ai_adversarial_e2e_options.md`）。

子命令（工具契约：落地 scripts/ + 只读模式 + 非 0 退出 + 三处登记）：
  python3 scripts/adversarial_shot.py --seeds 2026,7 --seats 4 --rounds 400 --out <dir>
    经 `cargo test --test adversarial_common` 导出 JSON 快照，PIL 绘制每 seed 一张 PNG。
  --check  只读体检：PIL 在位 + board.json 可读（0 写盘）。
"""
from __future__ import annotations

import argparse
import json
import math
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SNAPSHOT = ROOT / "evidence" / "adv_snapshot.json"
BOARD = ROOT / "assets" / "data" / "board.json"


def load_track_kinds(map_id: str = "m01") -> list[str]:
    db = json.loads(BOARD.read_text(encoding="utf-8"))
    spaces = db["maps"][map_id]["spaces"]
    return [s.get("kind", "plain") for s in spaces]


def _load_pil():
    try:
        from PIL import Image, ImageDraw  # noqa: PLC0415
    except ImportError:
        print("FAIL: PIL 缺位（pip install pillow），无法出图", file=sys.stderr)
        sys.exit(1)
    return Image, ImageDraw


def export_snapshot(seeds: list[int], seats: int, rounds: int) -> None:
    """跑 headless 对局导出 JSON 快照（逐参数 Command::new，非 shell 拼接）。"""
    cmd = [
        "cargo", "test", "-q", "-p", "ti-core", "--test", "adversarial_snapshot",
        "adversarial_snapshot_explicit_export",
    ]
    import os  # noqa: PLC0415
    print("[export] " + " ".join(cmd))
    env = dict(
        os.environ,
        TI_ADV_SEEDS=",".join(map(str, seeds)),
        TI_ADV_SEATS=str(seats),
        TI_ADV_ROUNDS=str(rounds),
    )
    r = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, env=env)
    if r.returncode != 0:
        print((r.stdout or "")[-400:] + (r.stderr or "")[-400:], file=sys.stderr)
        raise SystemExit("cargo 导出快照失败（非 0 退出）")
    if not SNAPSHOT.exists():
        raise SystemExit(f"快照未生成：{SNAPSHOT}")


def draw_shot(record: dict, track_kinds: list[str], out_png: Path) -> None:
    """单 seed 对局快照 → PNG（31 格环赛道 + 席位状态表 + 终局标注）。"""
    Image, _ = _load_pil()
    from PIL import ImageDraw  # noqa: PLC0415
    W, H = 900, 640
    img = Image.new("RGB", (W, H), "#1a1a2e")
    dr = ImageDraw.Draw(img)
    cx, cy, R = W // 2, H // 2 - 40, 220
    n = len(track_kinds)
    kind_color = {
        "start": "#f5d76e", "boss": "#e74c3c", "treasure": "#f1c40f",
        "event": "#3498db", "card": "#9b59b6", "territory": "#2c3e50", "plain": "#34495e",
    }
    owner_color = {0: "#2ecc71", 1: "#e67e22", 2: "#1abc9c", 3: "#e84393", 4: "#fd79a8"}
    for i, kind in enumerate(track_kinds):
        a = -math.pi / 2 + i * 2 * math.pi / n
        x, y = cx + R * math.cos(a), cy + R * math.sin(a)
        r = 22 if kind == "boss" else 16
        dr.ellipse([x - r, y - r, x + r, y + r], fill=kind_color.get(kind, "#34495e"), outline="#000")
        if kind in ("boss", "start"):
            dr.text((x - 8, y - 6), "BOSS" if kind == "boss" else "起", fill="#fff")
    for s, pos in enumerate(record.get("positions", [])):
        if pos is None or pos >= n:
            continue
        a = -math.pi / 2 + pos * 2 * math.pi / n
        x, y = cx + (R - 34) * math.cos(a), cy + (R - 34) * math.sin(a)
        dr.ellipse([x - 9, y - 9, x + 9, y + 9], fill=owner_color.get(s, "#fff"), outline="#000")
        dr.text((x - 4, y - 6), str(s), fill="#000")
    winner = record.get("winner")
    alive = record.get("alive", [])
    dr.text((cx - 60, cy - 30), f"seed={record.get('seed')}  rounds={record.get('rounds')}", fill="#f5d76e")
    if winner is not None:
        sub = f"终局 = 胜者 席{winner}" + ("（BOSS 通关）" if record.get("boss_final") else "（唯一存活）")
        dr.text((cx - 60, cy - 8), sub, fill="#2ecc71")
    else:
        sub = "终局 = 全破产（平局）" if all(a is False for a in alive) else f"未收敛（存活 {sum(1 for a in alive if a)} 席）"
        dr.text((cx - 60, cy - 8), sub, fill="#e74c3c")
    y0 = H - 150
    dr.text((20, y0 - 16), "席位状态（金 / 兵力 / 存活）", fill="#ccc")
    golds = record.get("gold", [])
    crews = record.get("crew", [])
    for s in range(len(alive)):
        yy = y0 + s * 26
        dr.text((20, yy), f"席{s}", fill=owner_color.get(s, "#fff"))
        dr.text((70, yy), f"gold={golds[s] if s < len(golds) else '?'}", fill="#f5d76e")
        dr.text((280, yy), f"crew={crews[s] if s < len(crews) else '?'}", fill="#3498db")
        dr.text((460, yy), "存活" if alive[s] else "破产", fill="#2ecc71" if alive[s] else "#e74c3c")
    img.save(out_png)


def main() -> None:
    ap = argparse.ArgumentParser(description="AI 对抗对局快照出图（飞轮 R21/S61）")
    ap.add_argument("--seeds", default="2026,7")
    ap.add_argument("--seats", type=int, default=4)
    ap.add_argument("--rounds", type=int, default=400)
    ap.add_argument("--out", default=str(ROOT / "evidence"))
    ap.add_argument("--check", action="store_true", help="只读体检（0 写盘）")
    args = ap.parse_args()
    if args.check:
        _load_pil()
        load_track_kinds()
        print("[check] PIL 在位 + board.json 可读（0 写盘）：PASS")
        return
    seeds = [int(x) for x in args.seeds.split(",") if x.strip()]
    export_snapshot(seeds, args.seats, args.rounds)
    track = load_track_kinds()
    snap = json.loads(SNAPSHOT.read_text(encoding="utf-8"))
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    for rec in snap:
        png = out_dir / f"adv_seed{rec.get('seed')}_r{rec.get('rounds')}.png"
        draw_shot(rec, track, png)
        print(f"  出图 {png.name}（seed={rec.get('seed')} rounds={rec.get('rounds')} winner={rec.get('winner')}）")
    print(f"adversarial_shot: {len(snap)} 张对局快照 PNG → {out_dir}")


if __name__ == "__main__":
    main()
