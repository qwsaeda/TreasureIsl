# S1.11 第二轮：ID 型 key 的 djb2 代数分解（模式 P + 2位数字 + 4字符后缀）
# h = A*33^6 + d1*33^5 + d0*33^4 + T(S)
import struct, zlib, json
from pathlib import Path

DATA = Path(r"C:\金银岛\data")
M33_4 = 33**4
M33_6 = 33**6

def keys_of(p):
    b = p.read_bytes()
    n = struct.unpack_from("<I", b, 0x26)[0]
    L = struct.unpack_from("<I", b, 0x2E)[0]
    raw = b[0x36:0x36+L]
    if raw[0] & 0xF == 8 and ((raw[0] << 8) | raw[1]) % 31 == 0:
        raw = zlib.decompress(raw)
    return [struct.unpack_from("<I", raw, i*16)[0] for i in range(n)]

def djb2(s: str) -> int:
    h = 0
    for c in s.encode("ascii"):
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]
SUFFIXES = [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt",
            ".dat", ".mod", ".jpg", ".png", ".lpk", ".pak", ".rec", ".sav"]

report = {}
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    # 已知可解释（名称哈希型）：先排除能由 SUFFIXES 直接反解的
    solved = []
    unsolved = []
    for idx, k in enumerate(ks):
        hit = None
        for S in SUFFIXES:
            T = djb2(S)          # S 作为独立串的 running hash = T(S)
            # h - T(S) 必须能被 33^len(S) 整除；此处 len(S)=4
            rem = (k - T) % M33_4
            if rem != 0:
                continue
            q = (k - T) // M33_4          # = A*1089 + d1*33 + d0
            hit = None
            for carry in range(4):   # d1*33+d0 可达 1938 > 1089，允许进位
                base = q - carry * 1089
                if base < 0:
                    break
                r = base % 1089
                d1, d0 = divmod(r, 33)
                A = (base // 1089) - 0
                if 48 <= d0 <= 57 and 48 <= d1 <= 57:
                    A = base // 1089
                    hit = (S, chr(d1), chr(d0), A)
                    break
            if hit:
                break
        if hit:
            solved.append((idx, k, hit))
        else:
            unsolved.append((idx, k))
    report[c] = {
        "total": len(ks),
        "digits_pattern": len(solved),
        "unsolved": len(unsolved),
        "samples": [(i, hex(k), s[0], s[1]+s[2], hex(s[3])) for i, k, s in solved[:5]],
        "unsolved_sample": [(i, hex(k)) for i, k in unsolved[:6]],
    }

for c, r in report.items():
    print(f"{c}: total={r['total']} digits(P+NN+4ch)={r['digits_pattern']} unsolved={r['unsolved']}")
    if r["samples"]:
        print("   e.g.", r["samples"][:3])
    if r["unsolved_sample"]:
        print("   unsolved:", r["unsolved_sample"])

# 对 digits 模式的 A（前缀 running hash）做前缀候选搜索
all_A = {}
for c, r in report.items():
    pass
# 重新收集所有 A
A_list = []
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    for idx, k in enumerate(ks):
        for S in SUFFIXES:
            T = djb2(S)
            if (k - T) % M33_4: continue
            q = (k - T) // M33_4
            got = None
            for carry in range(4):
                base = q - carry * 1089
                if base < 0: break
                d1, d0 = divmod(base % 1089, 33)
                if 48 <= d0 <= 57 and 48 <= d1 <= 57:
                    got = (chr(d1)+chr(d0), base // 1089)
                    break
            if got:
                A_list.append((c, idx, k, S, got[0], got[1]))
                break
print("\ndistinct A (prefix running-hash):", len(set(a[5] for a in A_list)))
# A 本身 = djb2(P)，P 可短可长。尝试常见前缀
import itertools, string
cand_prefixes = ["", "gl", "ui", "map", "card", "btn", "button", "icon", "role",
                 "item", "npc", "font", "land", "game", "treasure", "gold", "pic",
                 "image", "img", "role", "select", "help", "option", "info",
                 "background", "title", "logo", "talk", "menu", "win", "lose"]
A_set = set(a[5] for a in A_list)
found = {}
for p in cand_prefixes:
    h = djb2(p)
    if h in A_set:
        found[p] = h
print("prefix hits:", found)
# 若 P 为空：A = 0
print("A==0 count:", sum(1 for a in A_list if a[5] == 0))
# 输出全部 (container, idx, digits, A) 供后续比对
Path(r"C:\金银岛\evidence\S1.11-decompose.json").write_text(
    json.dumps([[a[0], a[1], hex(a[2]), a[3], a[4], hex(a[5])] for a in A_list],
               indent=1), encoding="utf-8")
print("wrote evidence/S1.11-decompose.json, rows =", len(A_list))
