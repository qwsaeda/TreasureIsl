# S1.11 第十一轮：词表驱动反解剩余 24 键
# k = A*33^(d+s) + Dd*33^s + T(s)  (mod 2^32)
# W ∈ 词表(exe串+已知名), D ∈ 数字串(1..4位), S ∈ 已知名后缀/扩展名
import struct, zlib, json, re, string
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

# ---- 词表 ----
exe = open(ROOT / "TreasureIsl.exe", "rb").read()
words = set()
for m in re.finditer(rb"(?:[\x20-\x7E]\x00){2,}", exe):
    s = m.group().decode("utf-16-le").lower()
    for part in re.split(r"[\\/ \-:.]", s):
        if 1 <= len(part) <= 20 and part.isascii():
            words.add(part)
for m in re.finditer(rb"[\x20-\x7E]{2,}", exe):
    s = m.group().decode("ascii").lower()
    for part in re.split(r"[\\/ \-:.]", s):
        if 1 <= len(part) <= 20 and part.isascii():
            words.add(part)

final = json.loads((ROOT / "evidence" / "S1.11-final.json").read_text(encoding="utf-8"))
known = []
for c, rows in final.items():
    for r in rows:
        if r["name"]:
            known.append(r["name"].lower())
            words.add(r["name"].lower())
# 已知名去数字得 stem
for n in known:
    stem = re.sub(r"\d+", "", n)
    words.add(stem)
    words.add(stem.rstrip("._-"))
words = {w for w in words if w and w.isascii() and not w.startswith("%")}

# ---- 后缀集：已知名的所有 3..6 字符尾 + 常用扩展 ----
suffixes = set()
for n in known:
    for L in range(2, 7):
        if len(n) >= L:
            suffixes.add(n[-L:])
exts = [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt", ".dat",
        ".mod", ".rec", ".sav", ".jpg", ".png", ".mdu", ".gl0", ".lpk", ".pak"]
suffixes.update(exts)
suffixes = {s for s in suffixes if len(s) >= 2}
print(f"words={len(words)} suffixes={len(suffixes)}")

# ---- 数字串 djb2 表 ----
digmap = {}
for d in (1, 2, 3, 4):
    m = {}
    for n in range(10 ** d):
        ds = f"{n:0{d}d}"
        m[djb2(ds)] = ds
    digmap[d] = m

def solve_key(k, tmax_words=None):
    hits = []
    for S in suffixes:
        s = len(S)
        T = djb2(S)
        base = pow(33, s, 1 << 32)
        if base & 1 == 0:
            continue
        inv = pow(base, -1, 1 << 32)
        for w in words:
            A = djb2(w)
            for d in (1, 2, 3, 4):
                pw = 33 ** (d + s)
                # need = Dd ≡ (k - A*pw - T) * inv(33^s)   -- 注意 A*pw 已含 33^s? 不：
                # k = A*33^(d+s) + Dd*33^s + T  →  (k - A*33^(d+s) - T) ≡ Dd*33^s
                need = ((k - A * pw - T) & 0xFFFFFFFF) * inv & 0xFFFFFFFF
                ds = digmap[d].get(need)
                if ds is not None:
                    name = w + ds + S
                    if djb2(name) == k:
                        hits.append(name)
                        if len(hits) >= 5:
                            return hits
    return hits

# 全量 305 键重解（验证词表法覆盖）
allkeys = {}
for c in ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
          "cards.lpk", "music.lpk", "goods.lpk"]:
    b = (DATA / c).read_bytes()
    n = struct.unpack_from("<I", b, 0x26)[0]
    L = struct.unpack_from("<I", b, 0x2E)[0]
    raw = b[0x36:0x36 + L]
    if raw[0] & 0xF == 8 and ((raw[0] << 8) | raw[1]) % 31 == 0:
        raw = zlib.decompress(raw)
    ks = [struct.unpack_from("<I", raw, i * 16)[0] for i in range(n)]
    allkeys[c] = ks

targets = []
for c, rows in final.items():
    for r in rows:
        if not r["name"]:
            targets.append((c, r["idx"], int(r["key"], 16)))

out = {}
for c, idx, k in targets:
    h = solve_key(k)
    out.setdefault(c, []).append({"idx": idx, "key": hex(k), "cands": h})
    print(f"{c} {hex(k)} -> {h[:3] if h else 'FAIL'}")

(ROOT / "evidence" / "S1.11-vocab.json").write_text(
    json.dumps(out, indent=1, ensure_ascii=False), encoding="utf-8")
print("wrote evidence/S1.11-vocab.json")
