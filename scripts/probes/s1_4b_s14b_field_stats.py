import json, struct
from pathlib import Path

data = Path("data").glob("*.Map")
ui = sorted(f for f in data if not f.name.lower().startswith("glmapm"))
print("ui maps:", len(ui))
total_records = 0
f5_dist = {}
f3_max = 0
f1_max = 0
f0_set = set()
f4_set = set()
per_map = {}
for f in ui:
    buf = f.read_bytes()
    words = [struct.unpack_from("<I", buf, i)[0] for i in range(16, len(buf), 4)]
    count = words[4]
    total_records += count
    per_map[f.name] = count
    for i in range(count):
        base = 5 + i * 6
        f0, f1, f2, f3, f4, f5 = words[base:base + 6]
        f0_set.add(f0)
        f4_set.add(f4)
        f5_dist[f5] = f5_dist.get(f5, 0) + 1
        f3_max = max(f3_max, f3)
        f1_max = max(f1_max, f1)
print("total records:", total_records)
print("f0 set:", sorted(f0_set))
print("f4 set:", sorted(f4_set))
print("f3 max:", f3_max, "f1 max:", f1_max)
print("f5 distinct:", len(f5_dist))
print("f5 top:", sorted(f5_dist.items(), key=lambda kv: -kv[1])[:15])
print("per-map:", per_map)
