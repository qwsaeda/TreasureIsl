# S1.11 第四轮：ID 型 key 结构分解
# 模型: name = P + d1 + d0 + S, |S|=4
# h = A*33^6 + d1*33^5 + d0*33^4 + T(S)   (A = djb2(P), T(S) = djb2(S))
import struct, zlib, json
from pathlib import Path

DATA = Path(r"C:\金银岛\data")
M4 = 33**4

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
SUFFIX4 = [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt",
           ".dat", ".mod", ".jpg", ".png", ".lpk", ".pak", ".rec", ".sav",
           ".gl00", "back", "base"]

# 基础检验：每个 key 对每个 4 字符后缀，(k - T(S)) % 33^4 是否为 0
stats = {}
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    row = {}
    for S in SUFFIX4:
        T = djb2(S)
        row[S] = sum(1 for k in ks if (k - T) % M4 == 0)
    stats[c] = row

print("=== 可除性命中数（key 总数见括号）===")
for c, row in stats.items():
    n = len(keys_of(DATA / c))
    best = sorted(((v, s) for s, v in row.items()), reverse=True)[:3]
    print(f"{c} ({n}): " + ", ".join(f"{s}={v}" for v, s in best))

# 已知 4 个 name 型 key 做阳性对照（ui.pak index 0 应为 ???）
# 用 glpics 的已知名字反算 T(S) 结构
test_names = ["dialogue.pcx", "number.pcx", "landinfo.pcx"]
for nm in test_names:
    k = djb2(nm)
    S = nm[-4:]
    T = djb2(S)
    print(f"ctrl {nm}: key={hex(k)} divisible_by_33^4={(k - T) % M4 == 0}")
