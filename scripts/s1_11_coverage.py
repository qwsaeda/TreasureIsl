# S1.11 收口：mapinfo.ini 权威名单 × 各容器 key 覆盖率
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

# 1) 收集所有 mapinfo.ini 的 filename 列表（按容器）
mapinfo_names = {}
for mi in TMP.rglob("3753dc98.bin"):
    b = mi.read_bytes()
    for enc in ("utf-16-le", "utf-8", "latin-1"):
        try:
            t = b.decode(enc)
        except Exception:
            continue
        if "filenum" in t:
            mapinfo_names[mi.parent.name] = re.findall(r"filename=(\S+)", t)
            break

# 2) mapinfo 是从容器里提取的，谁含 mapinfo 谁就有名单
print("mapinfo containers:", {k: len(v) for k, v in mapinfo_names.items()})

# 3) 覆盖率：容器 key 有多少能被『exe 资源名 ∪ mapinfo 名单』解释
exe = open(ROOT / "TreasureIsl.exe", "rb").read()
exe_names = set()
for m in re.finditer(rb"(?:[\x20-\x7E]\x00){3,}", exe):
    s = m.group().decode("utf-16-le")
    if re.search(r"\.(pcx|ani|fnt|ini|bmp|wav|png|txt|dat|mod|mdu)$", s, re.I):
        exe_names.add(s)
for m in re.finditer(rb"[\x20-\x7E]{3,40}", exe):
    s = m.group().decode("ascii")
    if re.search(r"\.(pcx|ani|fnt|ini|bmp|wav|png|txt|dat|mod|mdu)$", s, re.I):
        exe_names.add(s)

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]
report = {}
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    stem = c.rsplit(".", 1)[0]
    # 本容器的 mapinfo 名单 + 全局 mapinfo 名单（跨容器引用也常见）
    authority = set(exe_names)
    for names in mapinfo_names.values():
        authority.update(names)
    auth_keys = {djb2(n) for n in authority}
    # 也允许把 mapinfo 名单当纯文件名（去掉路径）
    hit = sum(1 for k in ks if k in auth_keys)
    report[c] = (hit, len(ks))
    print(f"{c}: {hit}/{len(ks)} 由 exe∪mapinfo 解释")
    missing = [hex(k) for k in ks if k not in auth_keys]
    if missing:
        print("   missing:", missing)

# 4) ui.pak 与 mapinfo(ui) 的一一对应验证
ui_keys = set(keys_of(DATA / "ui.pak"))
ui_names = mapinfo_names.get("ui", [])
ui_hit = sum(1 for n in ui_names if djb2(n) in ui_keys)
print(f"ui mapinfo names -> ui.pak keys: {ui_hit}/{len(ui_names)}")

# 5) glmap 容器 mapinfo 覆盖（m01 名单 → m01 keys）
for c, mi_name in [("glmap_m01.pak", "glmap_m01"), ("glmap_m02.pak", "glmap_m02")]:
    ks = set(keys_of(DATA / c))
    names = mapinfo_names.get(mi_name, [])
    hit = sum(1 for n in names if djb2(n) in ks)
    print(f"{mi_name} mapinfo -> {c}: {hit}/{len(names)}")

(ROOT / "evidence" / "S1.11-coverage.json").write_text(
    json.dumps({"report": report, "mapinfo_counts": {k: len(v) for k, v in mapinfo_names.items()}},
               indent=1), encoding="utf-8")
print("wrote evidence/S1.11-coverage.json")
