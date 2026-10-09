"""S1.4b 验收脚本：复核 ref/controls.json + ref/event_ids.json + 数据分布。

用法: python scripts/s14b_verify.py
输出: 各节通过/失败行，末尾 PASS/FAIL。
"""

import json
import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "data"
REF = ROOT / "ref"

failures = []


def check(cond: bool, label: str, detail: str = "") -> None:
    tag = "PASS" if cond else "FAIL"
    line = f"[{tag}] {label}"
    if detail:
        line += f" ({detail})"
    print(line)
    if not cond:
        failures.append(label)


# ---- 0) 数据分布（python 直接解析 .Map，独立于 Rust 解析器）----
ui_maps = sorted(f for f in DATA.glob("*.Map") if not f.name.lower().startswith("glmapm"))
total_records = 0
f0_set: set[int] = set()
f1_values: set[int] = set()
f4_set: set[int] = set()
f5_dist: dict[int, int] = {}
scene_counts: dict[str, int] = {}
f3_sorted: list[int] = []
f3_p95 = 0
f3_sentinel_count = 0
f3_sorted: list[int] = []
for f in ui_maps:
    buf = f.read_bytes()
    words = [struct.unpack_from("<I", buf, i)[0] for i in range(16, len(buf), 4)]
    assert len(words) == 5 + 6 * words[4], f"{f.name}: 字流不整除"
    count = words[4]
    scene_counts[f.name] = count
    total_records += count
    for i in range(count):
        b = 5 + i * 6
        f0, f1, _f2, f3, f4, f5 = words[b : b + 6]
        f0_set.add(f0)
        f1_values.add(f1)
        f4_set.add(f4)
        f5_dist[f5] = f5_dist.get(f5, 0) + 1
        f3_sorted.append(f3)
        if f3 == 0xFFFFFFFF:
            f3_sentinel_count += 1
f3_sorted.sort()
f3_p95 = f3_sorted[int(len(f3_sorted) * 0.95)] if f3_sorted else 0

check(len(ui_maps) == 20, "0.1 非地形 UI 场景数 == 20", str(len(ui_maps)))
check(total_records == 2041, "0.2 记录总数 == 2041", str(total_records))
check(f0_set == {1}, "0.3 f0 恒为 1", str(sorted(f0_set)))
check(f4_set == {1, 2, 3}, "0.4 f4 ∈ {1,2,3}", str(sorted(f4_set)))
check(
    f3_p95 <= 219,
    "0.5 f3 p95 ≤ 219（Y 坐标，非事件 id；F-311）",
    f"p95={f3_p95}, 哨兵 0xFFFFFFFF × {f3_sentinel_count}",
)
check(max(f1_values) <= 119, "0.6 f1 最大 ≤ 119（精灵索引）", str(max(f1_values)))
check(len(f5_dist) == 44, "0.7 f5 去重种类 == 44", str(len(f5_dist)))

# ---- 1) ref/controls.json 与数据一致 ----
cj = json.loads((REF / "controls.json").read_text(encoding="utf-8"))
check(cj.get("scene_count") == 20, "1.1 controls.json scene_count == 20", str(cj.get("scene_count")))
check(cj.get("record_total") == 2041, "1.2 controls.json record_total == 2041", str(cj.get("record_total")))
cs_scenes = {s["name"]: s for s in cj.get("scenes", [])}
check(len(cs_scenes) == 20, "1.3 controls.json scenes 数组长度 == 20", str(len(cs_scenes)))
mismatch = [
    n for n, c in scene_counts.items() if cs_scenes.get(n, {}).get("count") != c
]
check(not mismatch, "1.4 controls.json 逐场景 count 与数据一致", str(mismatch[:3]))
kd = dict((e["kind"], e["count"]) for e in cj.get("kind_dist", []))
check(all(kd.get(k, 0) == c for k, c in f5_dist.items()), "1.5 controls.json kind_dist 与数据 f5 分布一致")

# ---- 2) ref/event_ids.json 语义 ----
ej = json.loads((REF / "event_ids.json").read_text(encoding="utf-8"))
check(
    "f3" in ej.get("fields", {}) and "Y" in ej["fields"]["f3"],
    "2.1 event_ids.json 记载 f3 = 屏幕 Y 坐标",
)
check(
    "f5" in ej.get("fields", {}) and "类型" in ej["fields"]["f5"],
    "2.2 event_ids.json 记载 f5 = 控件类型码",
)
check(
    "sub_31424" in ej.get("event_id_formula", {}).get("function", ""),
    "2.3 event_ids.json 记载事件 id 公式 sub_31424",
)
check(
    len(ej.get("consumers", [])) == 4,
    "2.4 event_ids.json consumers 覆盖 sub_2101C 四处调用（含行 2303 直接读）",
    str(len(ej.get("consumers", []))),
)

# ---- 3) write_sites（事件 id 回填点全量）----
ws = ej.get("write_sites", [])
check(len(ws) == 41, "3.1 write_sites 共 41 条（sub_2101C 32 + sub_29110 9）", str(len(ws)))
check(
    all(w["id_value"] in range(1, 28) for w in ws),
    "3.2 write_sites 每条 id_value ∈ 1..27",
    str(sorted(set(w["id_value"] for w in ws))),
)
funcs = set(w["func"] for w in ws)
check(funcs == {"sub_2101C", "sub_29110"}, "3.3 write_sites 仅涉及 sub_2101C/sub_29110", str(funcs))
writers = set(w["writer"] for w in ws)
check(
    writers == {"sub_33E20", "sub_33E3C", "sub_2F6A8"},
    "3.4 write_sites writer ∈ {sub_33E20, sub_33E3C, sub_2F6A8}",
    str(writers),
)
check(
    all(w["target_slot"].startswith(("v1+88", "v43+34", "v814+88", "a1+100")) for w in ws),
    "3.5 write_sites target_slot 全部在事件队列槽（+88 队列2 / +34 队列1 / +100 队列1+28）",
)

# ---- 4) event_id_consumption + map mapping 结论 ----
ec = ej.get("event_id_consumption", {})
check("dispatch_1_8" in ec and "dispatch_9_27" in ec, "4.1 事件消费分派表存在")
mm = ej.get("event_id_to_map_mapping", {})
check(
    "不可静态映射" in mm.get("conclusion", ""),
    "4.2 事件 id ↔ .Map 映射结论 = 不可静态映射",
)
check(len(mm.get("evidence", [])) >= 3, "4.3 不可静态映射证据 ≥ 3 条", str(len(mm.get("evidence", []))))

# ---- 5) object_layout 新增结构 ----
ol = ej.get("object_layout", {})
check("ctrl_0x34" in ol, "5.1 object_layout.ctrl_0x34 存在")
check("event_descriptor_+22" in ol, "5.2 object_layout.event_descriptor_+22 存在")
check("app_event_structs" in ol, "5.3 object_layout.app_event_structs 存在")
check(len(ej.get("unresolved", [])) >= 1, "5.4 unresolved 仍有遗留项", str(len(ej.get("unresolved", []))))

# ---- 结果 ----
if failures:
    print(f"\nFAIL: {len(failures)} 项未通过: {failures}")
    sys.exit(1)
print("\nPASS: S1.4b 全部验收项通过")
