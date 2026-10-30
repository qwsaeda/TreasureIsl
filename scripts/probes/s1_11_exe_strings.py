# S1.11 第三轮：exe 全量 UTF-16 字符串 × 全部容器 key 对碰
import struct, zlib, re, json
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"
EXE = ROOT / "TreasureIsl.exe"

def djb2(s: bytes) -> int:
    h = 0
    for c in s:
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

def keys_of(p):
    b = p.read_bytes()
    n = struct.unpack_from("<I", b, 0x26)[0]
    L = struct.unpack_from("<I", b, 0x2E)[0]
    raw = b[0x36:0x36+L]
    if raw[0] & 0xF == 8 and ((raw[0] << 8) | raw[1]) % 31 == 0:
        raw = zlib.decompress(raw)
    return [struct.unpack_from("<I", raw, i*16)[0] for i in range(n)]

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]
all_keys = {}
for c in CONTAINERS:
    for i, k in enumerate(keys_of(DATA / c)):
        all_keys.setdefault(k, []).append((c, i))

exe = EXE.read_bytes()
# UTF-16LE 可打印串，长度>=2
strings = set()
for m in re.finditer(rb"(?:[\x20-\x7E]\x00){2,}", exe):
    s = m.group().decode("utf-16-le")
    strings.add(s)
    strings.add(s.lower())
# 再补：单字符拼接无意义；同时把字符串拆词（去路径前缀）
extra = set()
for s in list(strings):
    if "\\" in s or "/" in s:
        extra.add(s.split("\\")[-1].split("/")[-1])
    if " " in s:
        for w in s.split():
            extra.add(w)
strings |= extra
print(f"exe utf16 strings: {len(strings)}")

hits = {}
for s in strings:
    k = djb2(s.lower().encode("ascii", "ignore"))
    if k in all_keys:
        hits.setdefault(k, set()).add(s)
print(f"key hits: {len(hits)} / {len(all_keys)}")
# ASCII 窄字符串也扫一遍
for m in re.finditer(rb"[\x20-\x7E]{2,}", exe):
    s = m.group().decode("ascii")
    strings.add(s); strings.add(s.lower())
print(f"after ascii scan: {len(strings)}")
hits2 = {}
for s in strings:
    k = djb2(s.lower().encode("ascii", "ignore"))
    if k in all_keys:
        hits2.setdefault(k, set()).add(s)
hits = hits2
print(f"key hits total: {len(hits)} / {len(all_keys)}")
# 逐容器统计
for c in CONTAINERS:
    ks = [k for k, locs in all_keys.items() if any(l[0] == c for l in locs)]
    h = sum(1 for k in ks if k in hits)
    print(f"  {c}: {h}/{len(ks)} 由 exe 字符串解释")

Path(ROOT / "evidence" / "S1.11-exe-strings.json").write_text(json.dumps(
    {hex(k): sorted(v) for k, v in hits.items()}, indent=1, ensure_ascii=False),
    encoding="utf-8")
print("wrote evidence/S1.11-exe-strings.json")
