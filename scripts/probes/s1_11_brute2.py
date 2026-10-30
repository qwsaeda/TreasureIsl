# S1.11 最后 2 键暴力：w1 + digit + w2 + ext（增量 djb2）
import re, time
from pathlib import Path

ROOT = Path(r"C:\金银岛")

def djb2(s):
    h = 0
    for c in s.lower().encode():
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

TARGET = {0xD4EF1BE: "glpics3", 0x6171A968: "glpics34"}

exe = open(ROOT / "TreasureIsl.exe", "rb").read()
words = set()
for m in re.finditer(rb"[\x20-\x7E]{2,30}", exe):
    s = m.group().decode("ascii")
    for w in re.split(r"[^A-Za-z0-9]+", s):
        if 2 <= len(w) <= 16 and w.isascii():
            words.add(w)
words = sorted(words)
print("words:", len(words))

exts = [".pcx", "", ".ani", ".bmp", ".fnt", ".png"]
t0 = time.time()
hits = {}

# 预计算: prefix = w1 + digit 的哈希
prefixes = []
for w1 in words:
    b1 = djb2(w1)
    for d in b"0123456789":
        prefixes.append((w1, d, (b1 * 33 + d) & 0xFFFFFFFF))
print("prefixes:", len(prefixes))

# 后缀: w2 和 w2+ext 的增量
suffixes = []
for w2 in words:
    raw = w2.encode()
    for e in exts:
        suffixes.append((w2, e, raw + e.encode()))
print("suffixes:", len(suffixes))

cnt = 0
for w1, d, h in prefixes:
    for w2, e, tail in suffixes:
        k = h
        for c in tail:
            k = (k * 33 + c) & 0xFFFFFFFF
        cnt += 1
        if k in TARGET and TARGET[k] not in hits:
            name = w1 + chr(d) + w2 + e
            hits[TARGET[k]] = name
            print("HIT", TARGET[k], name)
    if len(hits) == 2:
        break
print("checked", cnt, "elapsed", round(time.time() - t0, 1), "s")
print("hits:", hits)
