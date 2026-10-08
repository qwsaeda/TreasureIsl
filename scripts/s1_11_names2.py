# S1.11 第八轮：miss 键 → (A, digits, S) → A 反查短前缀
import struct, zlib, json, string, itertools
from pathlib import Path

DATA = Path(r"C:\金银岛\data")
M4 = 33**4
INV = pow(M4, -1, 1 << 32)

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

SUFFIX4 = [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt",
           ".dat", ".mod", ".rec", ".sav"]

def solve(k, S, tmax=8192):
    T = djb2(S)
    Q0 = ((k - T) & 0xFFFFFFFF) * INV & 0xFFFFFFFF
    out = []
    for t in range(tmax):
        Q = Q0 + t * (1 << 32)
        r0 = Q % 33
        cands_d0 = [d for d in range(48, 58) if d % 33 == r0]
        if not cands_d0:
            continue
        d0 = cands_d0[0]
        Q1 = (Q - d0) // 33
        r1 = Q1 % 33
        cands_d1 = [d for d in range(48, 58) if d % 33 == r1]
        if not cands_d1:
            continue
        d1 = cands_d1[0]
        A = (Q1 - d1) // 33
        if A >= 0 and A * 1089 + d1 * 33 + d0 == Q:
            out.append((A, d1, d0))
            if len(out) >= 8:
                break
    return out

# A → 短前缀反查表（0..4 字符，可打印 ASCII）
CHARS = string.ascii_lowercase + string.digits + "_.-"
prefix_of = {0: ""}
for L in range(1, 5):
    for tup in itertools.product(CHARS, repeat=L):
        s = "".join(tup)
        h = djb2(s)
        if h not in prefix_of:
            prefix_of[h] = s
print(f"prefix table: {len(prefix_of)}")

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]

out = {}
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    rows = []
    for i, k in enumerate(ks):
        best = None
        for S in SUFFIX4:
            for (A, d1, d0) in solve(k, S):
                P = prefix_of.get(A)
                if P is not None:
                    name = f"{P}{chr(d1)}{chr(d0)}{S}"
                    if djb2(name) == k:      # 确定性校验
                        best = name
                        break
            if best:
                break
        rows.append({"idx": i, "key": hex(k), "name": best})
    out[c] = rows
    hit = [r for r in rows if r["name"]]
    print(f"{c}: {len(hit)}/{len(rows)}")
    if hit:
        print("   e.g.", [r["name"] for r in hit[:8]])
    miss = [r for r in rows if not r["name"]]
    if miss:
        print("   miss:", [r["key"] for r in miss[:6]])

Path(r"C:\金银岛\evidence\S1.11-names2.json").write_text(
    json.dumps(out, indent=1, ensure_ascii=False), encoding="utf-8")
print("wrote evidence/S1.11-names2.json")
