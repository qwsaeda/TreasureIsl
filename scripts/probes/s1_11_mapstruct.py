# S1.11: .Map UI 场景记录结构验证 + ui[121] 最后定向
import struct, re, time
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"

def djb2(s):
    h = 0
    for c in s.lower().encode():
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

# ---- A) UI 场景记录结构假设：[4 hdr][4 subhdr][count][count*6 记录] ----
UI_SCENES = ["index", "intro", "main", "selectrole", "selectmap", "build", "castle",
             "trade", "vs", "card", "Bigcard", "help", "information", "option",
             "saveload", "selecttarget", "msgl", "msgm", "msgs", "background"]
stats = {}
for nm in UI_SCENES:
    b = (DATA / f"{nm}.Map").read_bytes()
    ws = struct.unpack_from(f"<{len(b)//4}I", b, 0)
    hdr, body = ws[:4], ws[4:]
    # 假设: body = subhdr(4) + count(1) + count*6
    if len(body) < 5:
        print(nm, "too short")
        continue
    sub = body[:4]
    cnt = body[4]
    need = 4 + 1 + cnt * 6
    ok = need == len(body)
    recs = []
    if ok:
        for i in range(cnt):
            recs.append(body[5 + i*6: 11 + i*6])
    stats[nm] = (ok, cnt, len(body), need)
    print(f"{nm}: sub={sub} count={cnt} words={len(body)} need={need} match={ok}")

# ---- B) 元素 id 取值域 vs ui mapinfo 121 ----
print("\nid-field ranges (field3 of 6-tuple, 0-based):")
allids = []
for nm, (ok, cnt, _, _) in stats.items():
    if not ok:
        continue
    b = (DATA / f"{nm}.Map").read_bytes()
    ws = struct.unpack_from(f"<{len(b)//4}I", b, 0)
    body = ws[4:]
    ids = [body[5 + i*6 + 3] for i in range(cnt)]
    f0 = [body[5 + i*6 + 0] for i in range(cnt)]
    allids.extend(ids)
    print(f"  {nm}: count={cnt} id0={f0[:6]} id3={ids[:8]} max={max(ids) if ids else '-'}")
print("global id3 min/max:", min(allids), max(allids), "within[0,120]:", all(0 <= i <= 120 for i in allids))

# ---- C) ui[121] 定向 ----
TARGET = {0xF6EF0287: "ui121", 0x20769D35: "glpics11", 0x6171A968: "glpics34"}
hits = {}
def check(nm):
    if not nm:
        return
    k = djb2(nm)
    if k in TARGET and TARGET[k] not in hits:
        hits[TARGET[k]] = nm
        print("HIT", TARGET[k], ascii(nm))

exe = open(ROOT / "TreasureIsl.exe", "rb").read()
words = set()
for m in re.finditer(rb"[\x20-\x7E]{2,30}", exe):
    for w in re.split(r"[^A-Za-z0-9_]+", m.group().decode("ascii")):
        if 1 <= len(w) <= 20:
            words.add(w)
words = sorted(words)
EXTS = [".doc", ".xls", ".ppt", ".pcx", ".fnt", ".ani", ".wav", ".bmp", ".dat",
        ".txt", ".ini", ".log", ".cfg", ".tbl", ".mod", ".mdb", ".pub", ".msg"]

t0 = time.time()
# C1: exe 中含路径的串 + 扩展
for m in re.finditer(rb"[\x20-\x7E]{3,80}", exe):
    s = m.group().decode("ascii")
    if "\\" in s or "/" in s or " " in s:
        for e in EXTS:
            check(s + e)
            check(s.split("\\")[-1].split("/")[-1] + e)
print("C1 path/space:", hits, round(time.time()-t0, 1))

# C2: 纯数字名 + 扩展
for n in range(0, 1000):
    for e in EXTS:
        check(str(n) + e)
        check(f"{n:02d}" + e)
        check(f"{n:03d}" + e)
print("C2 numeric:", hits, round(time.time()-t0, 1))

# C3: w1 + 多位数字 + w2 + ext
if len(hits) < 3:
    for w1 in words:
        for w2 in words:
            for nd in (1, 2):
                for n in range(10 ** nd):
                    ds = f"{n:0{nd}d}"
                    for e in (".pcx", ".doc", ".fnt", ".ani", ".wav"):
                        check(w1 + ds + w2 + e)
                        check(w1 + "_" + ds + w2 + e)
            if len(hits) == 3:
                break
        if len(hits) == 3:
            break
print("C3 w1d w2:", hits, round(time.time()-t0, 1))
print("FINAL:", hits)
