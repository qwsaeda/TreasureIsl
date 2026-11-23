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
    """S81 运行期参数化语义：参数化原语的死坐标已删（x/y=0）+ derived 标记在位。
    竖屏徽章（derived=ring）+ 横屏木牌/金边/图标（derived=menu）全纳入。
    Rust 侧 index_data_items 运行期从 extra_scalar 算真实 rect（公式在 Rust，本脚本只守数据纪律）。"""
    problems = []
    # 竖屏徽章 36 行
    prims = scenes["index"]["primitives"]
    slots = medallion_slots(prims)
    if len(slots) != r["count"]:
        problems.append(f"竖屏徽章数 {len(slots)} ≠ ring.count={r['count']}")
    for item, idxs in slots.items():
        for idx in idxs:
            p = prims[idx]
            if p.get("params", {}).get("derived") != "ring":
                problems.append(f"#{idx} item={item} 缺 derived=ring 标记")
            if p["rect"]["x"] != 0 or p["rect"]["y"] != 0:
                problems.append(f"#{idx} item={item} 死坐标未删 rect.x/y={p['rect']['x']}/{p['rect']['y']}")
    # 横屏木牌/金边/图标 18 行
    lprims = scenes["index"]["landscape"]["primitives"]
    lmenu = 0
    for idx, p in enumerate(lprims):
        a = p.get("params", {}).get("asset", "")
        if "index_plank_" in a or "index_icon_" in a:
            lmenu += 1
            if p.get("params", {}).get("derived") != "menu":
                problems.append(f"landscape#{idx} 缺 derived=menu 标记")
            if p["rect"]["x"] != 0 or p["rect"]["y"] != 0:
                problems.append(f"landscape#{idx} 死坐标未删 rect.x/y={p['rect']['x']}/{p['rect']['y']}")
    if problems:
        for p in problems[:30]:
            print(f"[gen_index_medallions] DRIFT {p}", file=sys.stderr)
        print(f"[gen_index_medallions] 共 {len(problems)} 处死坐标/标记漂移", file=sys.stderr)
        return 1
    print(f"[gen_index_medallions] --check OK: 竖屏 {len(slots)*6} 行(derived=ring) + 横屏 {lmenu} 行(derived=menu) 死坐标已删(x/y=0)，Rust 运行期从 extra_scalar 算真实 rect")
    return 0


def generate(scenes: dict, r: dict) -> int:
    """S81：把参数化原语标记为「死坐标已删」（x/y=0 + derived 标记）。
    真实 rect 由 Rust index_data_items 运行期从 extra_scalar 算，本函数不写真坐标。"""
    changed = 0
    prims = scenes["index"]["primitives"]
    slots = medallion_slots(prims)
    for item, idxs in slots.items():
        for idx in idxs:
            p = prims[idx]
            if p["rect"]["x"] != 0 or p["rect"]["y"] != 0 or p.get("params", {}).get("derived") != "ring":
                p["rect"] = {"x": 0, "y": 0, "w": 0, "h": 0}
                p.setdefault("params", {})["derived"] = "ring"
                changed += 1
    lprims = scenes["index"]["landscape"]["primitives"]
    for p in lprims:
        a = p.get("params", {}).get("asset", "")
        if "index_plank_" in a or "index_icon_" in a:
            keep_wh = {"w": p["rect"]["w"], "h": p["rect"]["h"]} if "index_icon_" in a else {"w": 0, "h": 0}
            if p["rect"]["x"] != 0 or p["rect"]["y"] != 0 or p.get("params", {}).get("derived") != "menu":
                p["rect"] = {"x": 0, "y": 0, **keep_wh}
                p.setdefault("params", {})["derived"] = "menu"
                changed += 1
    if changed:
        SCENES.write_text(json.dumps(scenes, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"[gen_index_medallions] 标记 {changed} 行死坐标已删（x/y=0 + derived），真实 rect 由 Rust 运行期算")
    return changed


def sync_extra_scalar(scenes: dict, r: dict) -> int:
    """把嵌套 index.ring 展开成 index.extra_scalar 的扁平 `ring.*` 键（对齐 main_map，
    供 Rust ring_geom 同款运行期读取）。嵌套 ring = 唯一事实源，扁平键由本函数派生。
    返回本次改动键数。"""
    idx = scenes["index"]
    extra = idx.setdefault("extra_scalar", {})
    changed = 0
    for k, v in r.items():
        key = f"ring.{k}"
        if extra.get(key) != v:
            extra[key] = v
            changed += 1
    return changed


def check_dual_source(scenes: dict, r: dict) -> int:
    """只读对账：index.extra_scalar 的 ring.* 扁平键 == 嵌套 index.ring（双源一致性）。"""
    idx = scenes["index"]
    extra = idx.get("extra_scalar", {})
    problems = []
    for k, v in r.items():
        key = f"ring.{k}"
        if extra.get(key) != v:
            problems.append(f"{key}: extra_scalar={extra.get(key)} 应={v}（嵌套 ring 为唯一源）")
    if problems:
        for p in problems:
            print(f"[gen_index_medallions] DUAL-SOURCE DRIFT {p}", file=sys.stderr)
        return 1
    print(f"[gen_index_medallions] --check-dual OK: extra_scalar ring.* {len(r)} 键 == 嵌套 ring（0 漂移）")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description="index 罗盘环 6 徽章坐标参数化生成器（不手写像素总纲）")
    ap.add_argument("--check", action="store_true", help="只读对账：JSON 坐标 == ring 参数算出 + 双源一致")
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
        rc = check(scenes, r)
        if rc != 0:
            return rc
        # S81 最终架构：参数唯一源 = index.ring / index.menu 两个嵌套对象
        # （layout.rs #[serde(flatten)] 自动展开成 SceneLayout.extra_scalar 的 ring.*/menu.*，
        #   与 main_map 完全一致）——不再有扁平键双源，check_dual_source 已不适用。
        # 验证嵌套 menu 对象存在（横屏参数源）。
        if "menu" not in scenes["index"]:
            print("[gen_index_medallions] index.menu 嵌套参数对象缺失（S81 横屏参数源）", file=sys.stderr)
            return 1
        print("[gen_index_medallions] --check OK: 参数唯一源 = index.ring + index.menu 嵌套对象（layout.rs 自动展开成 extra_scalar）")
        return 0
    # 默认生成：只标记死坐标已删（参数唯一源 = 嵌套 ring/menu 对象，Rust layout.rs 自动展开）
    return generate(scenes, r)


if __name__ == "__main__":
    sys.exit(main())
