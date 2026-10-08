# S1.11 交叉分析：容器 ID 型 key 与 .Map u32 字流的匹配关系
import struct, zlib, sys, io, os, json
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]

def read_pkg(p):
    b = p.read_bytes()
    assert b[:16] == b"LucencyPackage\x00\x00", p
    count = struct.unpack_from("<I", b, 0x26)[0]
    idx_len = struct.unpack_from("<I", b, 0x2E)[0]
    data_start = struct.unpack_from("<I", b, 0x32)[0]
    raw = b[0x36:0x36+idx_len]
    if raw[0] & 0x0F == 8 and ((raw[0]<<8)|raw[1]) % 31 == 0:
        raw = zlib.decompress(raw)
    keys = []
    for i in range(count):
        k, off, ln, cl = struct.unpack_from("<IIII", raw, i*16)
        keys.append((k, off, ln, cl))
    return keys, data_start, b

# 1) 收集全部 key
all_keys = {}   # key -> (container, idx)
for c in CONTAINERS:
    ks, ds, raw = read_pkg(DATA / c)
    for i, (k, off, ln, cl) in enumerate(ks):
        all_keys.setdefault(k, []).append((c, i))

# 2) 收集全部 .Map 字流
maps = {}
for f in sorted(DATA.glob("*.Map")) + sorted(DATA.glob("*.map")):
    b = f.read_bytes()
    if len(b) % 4: continue
    words = list(struct.unpack_from("<%dI" % (len(b)//4), b, 0))
    maps[f.name] = words

# 3) 交叉匹配
matches = {}
word_hits = {}
for name, ws in maps.items():
    hits = []
    for pos, w in enumerate(ws):
        if w in all_keys:
            hits.append((pos, w, all_keys[w]))
    matches[name] = hits
    for h in hits:
        word_hits.setdefault(h[1], []).append((name, h[0]))

total_matched = sum(len(v) for v in matches.values())
distinct_matched = len(word_hits)
print(f"containers keys={len(all_keys)} maps={len(maps)}")
print(f"word occurrences matching a key: {total_matched}, distinct keys matched: {distinct_matched}")

# 4) 哪些 key 从未出现在任何 .Map（候选：非 .Map 绑定）
never = sorted(k for k in all_keys if k not in word_hits)
print(f"keys never appearing in any .Map: {len(never)}")

# 5) 等差分析（ui.pak 前 90 条）
ui_ks = [k for k, i, o, l, c in [(k,)+t for t in []] ] if False else None
ks_ui = [k for (k, off, ln, cl) in read_pkg(DATA/"ui.pak")[0]]
print("ui.pak first 5 keys:", [hex(k) for k in ks_ui[:5]])
diffs = sorted({ks_ui[i+1]-ks_ui[i] for i in range(min(89, len(ks_ui)-1))})
print("ui.pak consecutive diffs (first 90):", [hex(d) for d in diffs[:8]])

# 6) 每个 .Map 命中数 + 序号模式
print("\nper-map hits:")
for name in sorted(matches):
    h = matches[name]
    if h:
        ks = [x[1] for x in h]
        print(f"  {name}: n={len(h)} keys={[hex(k) for k in ks[:6]]}{' ...' if len(ks)>6 else ''}")

# 7) 未被 .Map 解释、且不属于等差串的 key —— 分布
print("\nnever-in-map keys by container:")
byc = {}
for k in never:
    for (c, i) in all_keys[k]:
        byc.setdefault(c, []).append((i, k))
for c in CONTAINERS:
    if c in byc:
        lst = sorted(byc[c])
        print(f"  {c}: n={len(lst)} first5={[hex(k) for i,k in lst[:5]]}")

# 8) 输出 ID 型清单到 evidence（供 Rust 侧常量表）
out = {
  "matched_keys": {str(k): [[n, p] for n, p in v] for k, v in sorted(word_hits.items())},
  "never_keys": {c: [[i, hex(k)] for i, k in sorted(v)] for c, v in byc.items()},
}
Path(r"C:\金银岛\evidence").mkdir(exist_ok=True)
Path(r"C:\金银岛\evidence\S1.11-analysis.json").write_text(
    json.dumps(out, indent=1, ensure_ascii=False), encoding="utf-8")
print("\nwrote evidence/S1.11-analysis.json")
