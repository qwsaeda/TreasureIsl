# S1.11 权威 vs 推断命名统计（诚实口径）
import struct, zlib, json, re
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

# 权威源 1：exe 资源名（取 basename，全部小写敏感由 djb2 处理）
exe = open(ROOT / "TreasureIsl.exe", "rb").read()
auth = set()
EXT_RE = re.compile(r"\.(pcx|ani|fnt|ini|bmp|wav|png|txt|dat|mod|mdu)$", re.I)
for m in re.finditer(rb"(?:[\x20-\x7E]\x00){3,}", exe):
    s = m.group().decode("utf-16-le")
    if EXT_RE.search(s):
        auth.add(s)
        auth.add(s.replace("\\", "/").split("/")[-1])
for m in re.finditer(rb"[\x20-\x7E]{3,60}", exe):
    s = m.group().decode("ascii")
    if EXT_RE.search(s):
        auth.add(s)
        auth.add(s.replace("\\", "/").split("/")[-1])

# 权威源 2：mapinfo.ini 文件名列表
mapinfo = {}
for mi in TMP.rglob("3753dc98.bin"):
    b = mi.read_bytes()
    for enc in ("utf-16-le", "utf-8", "latin-1"):
        try:
            t = b.decode(enc)
        except Exception:
            continue
        if "filenum" in t:
            mapinfo[mi.parent.name] = re.findall(r"filename=(\S+)", t)
            break
auth_keys = {}
for a in auth:
    auth_keys.setdefault(djb2(a), a)
map_keys = {}
for cont, names in mapinfo.items():
    for n in names:
        map_keys.setdefault(djb2(n), n)

# 推断源：S1.11-resolved.json（含暴力 preimage）
resolved = json.loads((ROOT / "evidence" / "S1.11-resolved.json").read_text(encoding="utf-8"))

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]
summary = {}
all_rows = {}
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    rows = []
    n_auth = n_map = n_infer = n_none = 0
    for i, k in enumerate(ks):
        if k in auth_keys:
            rows.append({"idx": i, "key": hex(k), "name": auth_keys[k], "src": "exe"})
            n_auth += 1
        elif k in map_keys:
            rows.append({"idx": i, "key": hex(k), "name": map_keys[k], "src": "mapinfo"})
            n_map += 1
        else:
            # 从 resolved.json 找推断名
            cand = next((r for r in resolved.get(c, []) if r["idx"] == i), None)
            if cand and djb2(cand["name"]) == k:
                rows.append({"idx": i, "key": hex(k), "name": cand["name"], "src": "inferred"})
                n_infer += 1
            else:
                rows.append({"idx": i, "key": hex(k), "name": None, "src": "unknown"})
                n_none += 1
    summary[c] = {"total": len(ks), "exe": n_auth, "mapinfo": n_map,
                  "inferred": n_infer, "unknown": n_none}
    all_rows[c] = rows
    print(f"{c}: total={len(ks)} exe={n_auth} mapinfo={n_map} inferred={n_infer} unknown={n_none}")

tot = {k: sum(s[k] for s in summary.values()) for k in ("total", "exe", "mapinfo", "inferred", "unknown")}
print("\nTOTAL:", tot)
auth_total = tot["exe"] + tot["mapinfo"]
print(f"权威(exe+mapinfo) = {auth_total}/{tot['total']} = {auth_total*100//tot['total']}%")
print(f"含推断 = {tot['total']-tot['unknown']}/{tot['total']}")

(ROOT / "evidence" / "S1.11-audit.json").write_text(
    json.dumps({"summary": summary, "rows": all_rows}, indent=1, ensure_ascii=False),
    encoding="utf-8")
print("wrote evidence/S1.11-audit.json")
