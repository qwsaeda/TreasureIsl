# S1.11 最终收口：全部 305 键的权威命名
import struct, zlib, json, re, itertools
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"
TMP = Path(r"C:\Users\Administrator\AppData\Local\Temp\ini_chk")

def djb2(s):
    h = 0
    for c in s.lower().encode():
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

# ---- 权威名单 ----
exe = open(ROOT / "TreasureIsl.exe", "rb").read()
auth = set()
for m in re.finditer(rb"(?:[\x20-\x7E]\x00){3,}", exe):
    s = m.group().decode("utf-16-le")
    auth.add(s); auth.add(s.lower())
    for p in re.split(r"[\\/]", s):
        auth.add(p)
for m in re.finditer(rb"[\x20-\x7E]{3,60}", exe):
    s = m.group().decode("ascii")
    auth.add(s); auth.add(s.lower())
    for p in re.split(r"[\\/]", s):
        auth.add(p)
for mi in TMP.rglob("3753dc98.bin"):
    b = mi.read_bytes()
    for enc in ("utf-16-le", "utf-8", "latin-1"):
        try:
            t = b.decode(enc)
        except Exception:
            continue
        if "filenum" in t:
            for n in re.findall(r"filename=(\S+)", t):
                auth.add(n)
            break
final = json.loads((ROOT / "evidence" / "S1.11-final.json").read_text(encoding="utf-8"))
for rows in final.values():
    for r in rows:
        if r["name"]:
            auth.add(r["name"])
auth = {a for a in auth if a and a.isascii()}
auth_keys = {}
for a in auth:
    k = djb2(a)
    if k not in auth_keys or (len(a) < len(auth_keys[k])):
        auth_keys[k] = a
print("authority names:", len(auth))

# ---- 递推扩展：对已知名中「倒数第5位是数字」的名字，变数字找新键 ----
def extend_digit(name):
    # name = head + d + tail4 形态（tail=最后4字符）
    if len(name) < 6:
        return []
    out = []
    pos = len(name) - 5
    if name[pos].isdigit():
        base = name[:pos]
        tail = name[pos + 1:]
        for d in "0123456789":
            out.append(base + d + tail)
    return out

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]

# ---- 词表 ----
words = set()
for m in re.finditer(rb"[\x20-\x7E]{2,40}", exe):
    s = m.group().decode("ascii")
    for w in re.split(r"[^A-Za-z0-9_]+", s):
        if 2 <= len(w) <= 24:
            words.add(w)
        for v in w.split("_"):
            if 2 <= len(v) <= 24:
                words.add(v)
for a in auth:
    if re.match(r"^[A-Za-z0-9_]+\.(pcx|fnt|ani|ini|wav)$", a, re.I):
        stem = a.rsplit(".", 1)[0]
        m = re.match(r"^([A-Za-z_]+?)(\d+)$", stem)
        if m:
            words.add(m.group(1))
exts = [".pcx", ".fnt", ".ani", ".ini", ".wav", ".bmp", ".txt", ".dat", ".mod", ".mdu"]

# 第一轮：权威 + 递推
resolved = {}
pending = []
for c in CONTAINERS:
    for i, k in enumerate(keys_of(DATA / c)):
        if k in auth_keys:
            resolved[(c, i)] = auth_keys[k]
        else:
            pending.append((c, i, k))
print("round1 resolved:", len(resolved), "pending:", len(pending))

# 第二轮：数字递推（从权威名生成邻居名）
by_key = {djb2(n): n for n in auth}
new_names = set()
for n in list(auth):
    for g in extend_digit(n):
        new_names.add(g)
for g in new_names:
    k = djb2(g)
    if k not in by_key:
        by_key[k] = g
still = []
for c, i, k in pending:
    if k in by_key:
        resolved[(c, i)] = by_key[k]
    else:
        still.append((c, i, k))
print("round2 resolved:", len(resolved), "still:", len(still))

# 第三轮：词表组合 word + 数字 + 扩展 / word_word + 扩展
for c, i, k in still:
    got = None
    for w in words:
        for nd in (1, 2, 3):
            for n in range(10 ** nd):
                for e in (".pcx", ".fnt", ".ani"):
                    if djb2(w + f"{n:0{nd}d}" + e) == k:
                        got = w + f"{n:0{nd}d}" + e
                        break
                if got:
                    break
            if got:
                break
        if got:
            break
    if not got:
        # word + word + ext（受限 word 集：auth 的 stem）
        stems = sorted({w for w in words if 2 <= len(w) <= 16})
        for w1 in stems:
            for w2 in stems:
                for e in (".pcx", ".fnt"):
                    if djb2(w1 + "_" + w2 + e) == k:
                        got = w1 + "_" + w2 + e
                        break
                if got:
                    break
            if got:
                break
    if got:
        resolved[(c, i)] = got
    else:
        print("UNRESOLVED", c, i, hex(k))
print("final resolved:", len(resolved), "/ 305")

# ---- 校验 ----
bad = [(c, i) for (c, i), n in resolved.items() if djb2(n) != keys_of(DATA / c)[i]]
assert not bad, bad

out = {}
for (c, i), n in sorted(resolved.items()):
    out.setdefault(c, []).append({"idx": i, "name": n})
(ROOT / "evidence" / "S1.11-resolved.json").write_text(
    json.dumps(out, indent=1, ensure_ascii=False), encoding="utf-8")
print("wrote evidence/S1.11-resolved.json")
