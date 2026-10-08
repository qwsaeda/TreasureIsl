# S1.11 第十轮：24 个未知键的广义分解
# name = P + D(数字串, 长1..4) + S(后缀, 长1..6)
# k ≡ A*33^(|D|+|S|) + djb2(D)*33^|S| + djb2(S) (mod 2^32)
import struct, zlib, json, string, itertools
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"

def djb2(s):
    if isinstance(s, str):
        s = s.encode("ascii")
    h = 0
    for c in s:
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

CHARS = string.ascii_lowercase + string.digits + "_.- "
prefix_of = {0: ""}
for L in range(1, 6):
    for tup in itertools.product(CHARS, repeat=L):
        s = "".join(tup)
        h = djb2(s)
        if h not in prefix_of:
            prefix_of[h] = s

SUFFIXES = []
for a in [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt",
          ".dat", ".mod", ".rec", ".sav", ".jpg", ".png", ".gl0", ".lpk",
          ".pak", "front", "back", "base", "down", "over", "idle", "walk",
          "hit1", "hit2", "die1", "die2", "sel0", "sel1"]:
    SUFFIXES.append(a)

UNKNOWN = {
  "glpics.pak": ["0xd4ef1be", "0x6171a968"],
  "glmap_m01.pak": ["0x37fbb49", "0x43105ffb", "0x80d06f6a", "0xa23c0229",
                    "0xa566a76a", "0xa578bfeb", "0xa58ad86c", "0xb53654f2",
                    "0xc5561d44", "0xc56835c5", "0xe8d8c62", "0x10d09c82",
                    "0x30fff368", "0x3753dc98"],
  "glmap_m02.pak": ["0x37fbb49", "0x40912d92", "0x8d8b6a53", "0x92de88f1",
                    "0xb95f0b95", "0xe4b250ab", "0xe4c4692c", "0xe6acd3a6"],
}

# 直接读 final.json 的 unknown 列表更准
final = json.loads((ROOT / "evidence" / "S1.11-final.json").read_text(encoding="utf-8"))
targets = {}
for c, rows in final.items():
    u = [r for r in rows if not r["name"]]
    if u:
        targets[c] = u
print("unknown counts:", {c: len(v) for c, v in targets.items()})

def solve(k, tmax=1 << 20):
    """广义：枚举 (|S|, |D|)，模逆求 Q，分解数字位与前缀"""
    hits = []
    for S in SUFFIXES:
        ls = len(S)
        Ts = djb2(S)
        base = pow(33, ls, 1 << 32)
        if base & 1 == 0:
            continue
        inv = pow(base, -1, 1 << 32)
        Q0 = ((k - Ts) & 0xFFFFFFFF) * inv & 0xFFFFFFFF
        for nd in (1, 2, 3, 4):
            P33 = 33 ** nd
            # Q ≡ A*P33 + D' (mod 2^32), D' = djb2(digits str) < 33^nd * 60ish
            # 枚举 t（Q = Q0 + t*2^32），t 上界使 A 不至于过大（A < 33^5 ≈ 4e7 for 5char prefix）
            # A = (Q - D')/P33, D' 由数字位决定: 先取 Q mod P33 的低位作 D' 候选
            maxt = 64
            for t in range(maxt):
                Q = Q0 + t * (1 << 32)
                if Q < 0:
                    continue
                # D' = Q mod P33, 但可能有借位：尝试借 0..2 个 P33
                for borrow in range(3):
                    Dp = (Q - borrow * P33) % P33 if False else (Q % P33)
                    # 直接: D' = Q mod P33（A = Q // P33）
                    A = Q // P33
                    Dp = Q - A * P33
                    # Dp 必须是某个 nd 位数字串的 djb2（含前导零）
                    # 反推数字串：从低位到高位
                    digs = []
                    ok = True
                    tmp = Dp
                    for _ in range(nd):
                        # tmp = prev*33 + d  →  d = tmp % 33 须在 48..57 或借位
                        r = tmp % 33
                        d = r
                        if not (48 <= d <= 57):
                            # 借位: d+33
                            if 48 <= d + 33 <= 57:
                                d = d + 33
                                tmp = tmp // 33 + 1   # 借位进位
                            else:
                                ok = False
                                break
                        else:
                            tmp = tmp // 33
                        digs.append(chr(d))
                    if not ok:
                        continue
                    digs.reverse()
                    D = "".join(digs)
                    if djb2(D) != Dp:
                        continue
                    P = prefix_of.get(A)
                    if P is None:
                        continue
                    name = P + D + S
                    if djb2(name) == k:
                        hits.append(name)
                        break
                if hits:
                    break
            if hits:
                break
        if hits:
            break
    return hits

results = {}
for c, rows in targets.items():
    for r in rows:
        k = int(r["key"], 16)
        h = solve(k)
        r["name"] = h[0] if h else None
        r["kind"] = "broad" if h else "unknown"
        print(f"{c} {r['key']} -> {h[:2] if h else 'FAIL'}")
    results[c] = rows

(ROOT / "evidence" / "S1.11-broad.json").write_text(
    json.dumps(results, indent=1, ensure_ascii=False), encoding="utf-8")
print("wrote evidence/S1.11-broad.json")
