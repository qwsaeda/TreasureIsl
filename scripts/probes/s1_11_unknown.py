import re, json
from pathlib import Path

def djb2(s):
    h = 0
    for c in s.lower().encode():
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

b = open(r"C:\金银岛\TreasureIsl.exe", "rb").read()
pats = {
    "L.pcx": rb"\.\x00p\x00c\x00x\x00",
    "L%d": rb"%\x00d\x00",
    "Limage": rb"i\x00m\x00a\x00g\x00e\x00",
    "Lcard": rb"c\x00a\x00r\x00d\x00",
}
for n, p in pats.items():
    print(n, len(list(re.finditer(p, b))))

f = json.loads(Path(r"C:\金银岛\evidence\S1.11-final.json").read_text(encoding="utf-8"))
unk = [(c, r["key"]) for c, rows in f.items() for r in rows if not r["name"]]
print("unknown:", len(unk))

strs = set()
for m in re.finditer(rb"(?:[\x20-\x7E]\x00){2,}", b):
    s = m.group().decode("utf-16-le").lower()
    strs.add(s)
    strs.add(s.replace("\\", "/").split("/")[-1])
    strs.add(s.split(" ")[-1])
for m in re.finditer(rb"[\x20-\x7E]{2,}", b):
    s = m.group().decode("ascii").lower()
    strs.add(s)
    strs.add(s.replace("\\", "/").split("/")[-1])

for c, k in unk:
    ki = int(k, 16)
    hit = [s for s in strs if djb2(s) == ki]
    print(k, c, "->", hit[:3])
