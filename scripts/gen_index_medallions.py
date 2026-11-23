#!/usr/bin/env python3
"""gen_index_medallions.py — index 罗盘环 6 徽章坐标参数化生成器（S80 / EXP-210）。

用途
----
落实总纲「不手写像素：声明意图/数据，引擎算坐标」（docs/process/18_asset_content_pipeline.md §15）：
index 竖屏 6 枚徽章的 36 条原语坐标不再手抄，而是由 `scenes.json` 的
`index.ring` 参数（环心/半径/数量/尺寸）经正六边形公式算出并回写。

公式（与 chrome::ring_card_center 同构，实测 0 误差还原原 36 行）：
    中心_i = (cx + round(R·cos(θ)), cy + round(R·sin(θ))),  θ = -90° + i·360°/count
    徽章底 rect = (cx_i - badge_w/2, cy_i - badge_h/2, badge_w, badge_h)
    图标 rect   = (cx_i - icon_w/2,  cy_i - icon_w/2,  icon_w, icon_w)
    标签 rect   = (cx_i - label_w/2, cy_i + badge_h/2 + label_dy, label_w, label_h)
    其中 label_dy = label 顶相对徽章底边的额外偏移（原数据 498+144+12=654 ⇒ dy=12）

用法
----
    python3 scripts/gen_index_medallions.py            # 生成（按 ring 参数回写 36 行坐标）
    python3 scripts/gen_index_medallions.py --check    # 只读对账：JSON 坐标 == 参数算出的，不符非 0
    python3 scripts/gen_index_medallions.py --json     # 机读：打印 36 行期望 rect
    python3 scripts/gen_index_medallions.py --ring R   # 仅试算半径（不写）

退出码：0 成功/--check 0 漂移；1 参数缺失/坐标漂移/写入失败；2 前置缺失（scenes.json/Pillow 无关，纯 stdlib）。

幂等：同参数连续跑两次，scenes.json 逐字节一致（只改 36 行 rect 的 x/y，其余不动）。
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SCENES = REPO / "assets/data/scenes.json"


def load_scenes() -> dict:
    try:
        return json.loads(SCENES.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"[gen_index_medallions] 读 scenes.json 失败: {exc}", file=sys.stderr)
        sys.exit(2)


def ring_params(scenes: dict) -> dict:
    ring = scenes.get("index", {}).get("ring")
    if not ring:
        print("[gen_index_medallions] index.ring 参数缺失（S80 参数化前置）", file=sys.stderr)
        sys.exit(1)
    need = ("center_x", "center_y", "radius", "badge_w", "badge_h", "count", "icon_w", "label_w", "label_h", "label_dy")
    for k in need:
        if k not in ring:
            print(f"[gen_index_medallions] index.ring 缺字段 {k}", file=sys.stderr)
            sys.exit(1)
    return ring


def expected_rects(r: dict) -> dict[int, dict[int, dict]]:
    """返回 {item: {kind_slot: rect}}；kind_slot 对应每枚 6 原语的顺序索引。"""
    cx, cy, R = r["center_x"], r["center_y"], r["radius"]
    n = r["count"]
    bw, bh, iw = r["badge_w"], r["badge_h"], r["icon_w"]
    lw, lh, ldy = r["label_w"], r["label_h"], r["label_dy"]
    out: dict[int, dict[int, dict]] = {}
    for i in range(n):
        theta = math.radians(-90.0 + i * 360.0 / n)
        mx = cx + round(R * math.cos(theta))
        my = cy + round(R * math.sin(theta))
        badge = {"x": mx - bw // 2, "y": my - bh // 2, "w": bw, "h": bh}
        icon = {"x": mx - iw // 2, "y": my - iw // 2, "w": iw, "h": iw}
        label = {"x": mx - lw // 2, "y": my + bh // 2 + ldy, "w": lw, "h": lh}
        # 6 原语顺序：image(badge) / icon×2 / rect / text×2
        out[i] = {0: badge, 1: icon, 2: icon, 3: label, 4: label, 5: label}
    return out


def medallion_slots(prims: list[dict]) -> dict[int, list[int]]:
    """从 primitives 找每枚徽章的 6 原语下标：带 badge=true 的底图所在 item，
    该 item 的全部原语（image/icon/rect/text）= 6 个，按原顺序即 kind_slot 0..5。"""
    badge_items = {p.get("item") for p in prims if p.get("params", {}).get("badge") is True}
    slots: dict[int, list[int]] = {}
    for idx, p in enumerate(prims):
        item = p.get("item")
        if item in badge_items:
            slots.setdefault(item, []).append(idx)
    for item, idxs in slots.items():
        if len(idxs) != 6:
            print(f"[gen_index_medallions] item={item} 徽章原语数 {len(idxs)}≠6", file=sys.stderr)
            sys.exit(1)
    return slots


def check(scenes: dict, r: dict) -> int:
    prims = scenes["index"]["primitives"]
    slots = medallion_slots(prims)
    exp = expected_rects(r)
    problems = []
    for item, idxs in slots.items():
        if item not in exp:
            problems.append(f"item={item} 超出 ring.count={r['count']}")
            continue
        for slot, idx in enumerate(idxs):
            got = prims[idx]["rect"]
            want = exp[item][slot]
            if got != want:
                problems.append(f"#{idx} item={item} slot={slot}: rect={got} 应={want}")
    if problems:
        for p in problems[:30]:
            print(f"[gen_index_medallions] DRIFT {p}", file=sys.stderr)
        print(f"[gen_index_medallions] 共 {len(problems)} 处手写坐标 vs 参数算出漂移", file=sys.stderr)
        return 1
    print(f"[gen_index_medallions] --check OK: {len(slots)} 枚徽章 × 6 原语 = {len(slots)*6} 行坐标 == ring 参数算出（0 漂移）")
    return 0


def generate(scenes: dict, r: dict) -> int:
    prims = scenes["index"]["primitives"]
    slots = medallion_slots(prims)
    exp = expected_rects(r)
    changed = 0
    for item, idxs in slots.items():
        for slot, idx in enumerate(idxs):
            want = exp[item][slot]
            if prims[idx]["rect"] != want:
                prims[idx]["rect"] = want
                changed += 1
    text = json.dumps(scenes, ensure_ascii=False, indent=1) + "\n"
    SCENES.write_text(text, encoding="utf-8")
    print(f"[gen_index_medallions] 回写 {changed} 行坐标（{len(slots)} 枚徽章 × 6 原语），ring={json.dumps(r, ensure_ascii=False)}")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="index 罗盘环 6 徽章坐标参数化生成器（不手写像素总纲）")
    ap.add_argument("--check", action="store_true", help="只读对账：JSON 坐标 == ring 参数算出")
    ap.add_argument("--json", action="store_true", help="机读：打印每枚徽章 6 原语期望 rect")
    ap.add_argument("--ring", type=int, dest="ring_override", help="仅试算：临时覆盖 radius（不写文件）")
    args = ap.parse_args(argv)

    scenes = load_scenes()
    r = ring_params(scenes)
    if args.ring_override is not None:
        r = {**r, "radius": args.ring_override}

    if args.json:
        print(json.dumps(expected_rects(r), ensure_ascii=False, indent=1))
        return 0
    if args.ring_override is not None:
        print(f"[gen_index_medallions] 试算 R={args.ring_override}: {json.dumps(expected_rects(r)[0], ensure_ascii=False)}")
        return 0
    if args.check:
        return check(scenes, r)
    return generate(scenes, r)


if __name__ == "__main__":
    sys.exit(main())
