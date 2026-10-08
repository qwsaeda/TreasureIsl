# S1.11 第七轮：确定性暴力反解（前缀词表 × 数字 × 后缀）
import struct, zlib, json, itertools
from pathlib import Path

DATA = Path(r"C:\金银岛\data")

def djb2(s):
    h = 0
    for c in s.encode("ascii"):
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

def keys_of(p):
    b = p.read_bytes()
    n = struct.unpack_from("<I", b, 0x26)[0]
    L = struct.unpack_from("<I", b, 0x2E)[0]
    raw = b[0x36:0x36 + L]
    if raw[0] & 0xF == 8 and ((raw[0] << 8) | raw[1]) % 31 == 0:
        raw = zlib.decompress(raw)
    return [struct.unpack_from("<I", raw, i * 16)[0] for i in range(n)]

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]

# 候选前缀：空 + 常见 stem
PREFIXES = ["", "gl", "ui", "map", "card", "role", "item", "npc", "btn",
            "select", "help", "option", "info", "land", "game", "pic",
            "back", "base", "talk", "menu", "win", "lose", "title", "logo",
            "font", "image", "img", "icon", "num", "number", "target",
            "event", "good", "goods", "music", "sound", "role", "person",
            "build", "castle", "trade", "vs", "msg", "index", "intro",
            "main", "background", "Bigcard", "saveload", "selectmap",
            "selectrole", "selecttarget", "information", "dialogue", "dialog"]
SUFFIXES = [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt",
            ".dat", ".mod", ".rec", ".sav"]

# 每容器候选数字位数
DIGITS = [2, 3]

# 先建反查表：key -> name（对所有 前缀+数字+后缀 组合）
table = {}
for P in PREFIXES:
    for n in range(0, 1000):
        for ds in DIGITS:
            for S in SUFFIXES:
                name = f"{P}{n:0{ds}d}{S}"
                k = djb2(name)
                if k not in table:
                    table[k] = name

print(f"reverse table size: {len(table)}")

results = {}
total_hit = 0
total = 0
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    hit, miss = [], []
    for i, k in enumerate(ks):
        n = table.get(k)
        (hit if n else miss).append({"idx": i, "key": hex(k), "name": n} if n
                                    else {"idx": i, "key": hex(k)})
    results[c] = {"hit": hit, "miss": miss}
    total_hit += len(hit)
    total += len(ks)
    print(f"{c}: {len(hit)}/{len(ks)}")
    if hit:
        print("   e.g.", [h["name"] for h in hit[:6]])
    if miss:
        print("   miss:", [m["key"] for m in miss[:6]])

print(f"\nTOTAL: {total_hit}/{total}")
Path(r"C:\金银岛\evidence\S1.11-names.json").write_text(
    json.dumps(results, indent=1, ensure_ascii=False), encoding="utf-8")
print("wrote evidence/S1.11-names.json")
