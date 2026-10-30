import re, glob, json
from pathlib import Path

def djb2(s):
    h = 0
    for c in s.lower().encode():
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

targets = {0x37fbb49, 0xd4ef1be, 0x6171a968}
cands = ["0.pcx", "1.pcx", "b21.pcx", "b24.pcx", "b24_2.pcx", "boss.pcx",
         "war.pcx", "card0.pcx", "event0.pcx", "ice.pcx", "pool.pcx",
         "mapinfo.ini", "event.ini", "cards.ini", "goods.ini", "role.ini",
         "land.ini", "music.ini", "sound.ini", "glmap.ini", "gl.ini",
         "background.pcx", "back.pcx", "front.pcx", "title.pcx", "logo.pcx",
         "dialogue.pcx", "bigcard.pcx", "select.pcx", "selectmap.pcx"]
for c in cands:
    k = djb2(c)
    if k in targets:
        print("CAND", hex(k), c)

# mapinfo.ini 全部名字
base = Path(r"C:\Users\Administrator\AppData\Local\Temp\ini_chk")
for mi in base.rglob("3753dc98.bin"):
    b = mi.read_bytes()
    for enc in ("utf-16-le", "utf-8", "latin-1"):
        try:
            t = b.decode(enc)
        except Exception:
            continue
        if "filenum" in t:
            names = re.findall(r"filename=(\S+)", t)
            print(mi.parent.name, "names:", names)
            for n in names:
                k = djb2(n)
                if k in targets:
                    print("  HIT", hex(k), n)
            break

# glpics 的 solo 解 + 未知内容特征
f = json.loads(Path(r"C:\金银岛\evidence\S1.11-final.json").read_text(encoding="utf-8"))
for r in f["glpics.pak"]:
    if not r["name"] or r["kind"] == "id-solo":
        print("glpics row:", r)
