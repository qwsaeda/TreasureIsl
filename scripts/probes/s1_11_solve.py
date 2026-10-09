# S1.11 第六轮：模逆求解
# k ≡ Q*33^4 + T(S) (mod 2^32), 33^4 与 2^32 互素 → Q ≡ (k-T)*inv(33^4) (mod 2^32)
# Q = Q0 + t*2^32, t 小范围枚举；再分解 Q = A*1089 + d1*33 + d0（数字位）
import struct, zlib, json
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

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]
SUFFIX4 = [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt",
           ".dat", ".mod", ".jpg", ".png", ".lpk", ".pak", ".rec", ".sav"]

def solve(k, S, tmax=4096):
    T = djb2(S)
    Q0 = ((k - T) & 0xFFFFFFFF) * INV & 0xFFFFFFFF
    out = []
    for t in range(tmax):
        Q = Q0 + t * (1 << 32)
        # Q = A*1089 + d1*33 + d0, digits 48..57
        # 有效 digit 段: d0 = Q%33 需满足 (Q%33) ∈ {48..57} mod 33 → 15..24 且借位后
        r0 = Q % 33
        for d0 in range(48, 58):
            if d0 % 33 == r0:
                break
        else:
            continue
        Q1 = (Q - d0) // 33
        r1 = Q1 % 33
        for d1 in range(48, 58):
            if d1 % 33 == r1:
                break
        else:
            continue
        A = (Q1 - d1) // 33
        if A < 0:
            continue
        # 校验重组
        if A * 1089 + d1 * 33 + d0 != Q:
            continue
        out.append((Q, A, d1, d0))
        if len(out) >= 4:
            break
    return out

# 阳性对照：带数字的已知名字（若无，用合成名验证算法）
synth = "gl12.pcx"
ks = djb2(synth)
print(f"synth {synth} key={hex(ks)} ->", [(hex(a), chr(d1)+chr(d0)) for _, a, d1, d0 in solve(ks, '.pcx')])

results = {}
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    solved, unsolved = [], []
    for i, k in enumerate(ks):
        hit = None
        for S in SUFFIX4:
            r = solve(k, S)
            if r:
                Q, A, d1, d0 = min(r, key=lambda x: x[1])
                hit = {"idx": i, "key": hex(k), "S": S,
                       "digits": chr(d1)+chr(d0), "A": hex(A)}
                break
        (solved if hit else unsolved).append(hit if hit else {"idx": i, "key": hex(k)})
    results[c] = {"solved": solved, "unsolved": unsolved}
    print(f"{c}: solved={len(solved)}/{len(ks)}")
    if solved:
        print("   e.g.", [(s["digits"], s["S"], s["A"]) for s in solved[:5]])
    if unsolved:
        print("   unsolved:", [u["key"] for u in unsolved[:5]])

Path(r"C:\金银岛\evidence\S1.11-solve.json").write_text(
    json.dumps(results, indent=1), encoding="utf-8")
print("wrote evidence/S1.11-solve.json")
