# S1.11 末 4 键：ui[121] OLE + glpics 3/11/34
import re, time, itertools
from pathlib import Path

ROOT = Path(r"C:\金银岛")

def djb2(s):
    h = 0
    for c in s.lower().encode():
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

TARGET = {
    0xF6EF0287: "ui121",
    0xD4EF1BE: "glpics3",
    0x20769D35: "glpics11",
    0x6171A968: "glpics34",
}

exe = open(ROOT / "TreasureIsl.exe", "rb").read()
words = set()
for m in re.finditer(rb"[\x20-\x7E]{2,30}", exe):
    s = m.group().decode("ascii")
    for w in re.split(r"[^A-Za-z0-9_]+", s):
        if 1 <= len(w) <= 24:
            words.add(w)
# 常见扩展独立词
words |= {"help", "info", "about", "logo", "title", "menu", "back", "none", "null",
          "ole", "doc", "sheet", "book", "table", "data", "cfg", "conf", "config",
          "image", "picture", "photo", "bitmap", "icon", "cursor", "select", "role",
          "player", "enemy", "guard", "king", "queen", "prince", "girl", "boy",
          "katharina", "anna", "maria", "laura", "sophia", "narrator", "oldman",
          "captain", "sailor", "merchant", "blacksmith", "priest", "judge"}
words = sorted(words)
print("words:", len(words))

EXTS = [".pcx", ".fnt", ".ani", ".wav", ".bmp", ".dat", ".txt", ".ini", ".doc",
        ".xls", ".ppt", ".log", ".cfg", ".tbl", ".tab", ".csv", ".htm", ".html",
        ".rtf", ".wri", ".mod", ".mdu", ".sav", ".rec", ".map", ".pak", ".lpk"]

hits = {}
t0 = time.time()

def check(nm):
    if not nm:
        return
    k = djb2(nm)
    if k in TARGET and TARGET[k] not in hits:
        hits[TARGET[k]] = nm
        print("HIT", TARGET[k], nm)

# 形态1: word + ext（无数字）
for w in words:
    for e in EXTS:
        check(w + e)
print("after word+ext:", hits, round(time.time()-t0, 1), "s")

# 形态2: word + digit* + ext
if len(hits) < 4:
    for w in words:
        for nd in (1, 2, 3):
            for n in range(10 ** nd):
                ds = f"{n:0{nd}d}"
                for e in EXTS:
                    check(w + ds + e)
    print("after word+num+ext:", hits, round(time.time()-t0, 1), "s")

# 形态3: word_word + ext / word_word数字 + ext
if len(hits) < 4:
    for w1 in words:
        for w2 in words:
            for e in (".pcx", ".fnt", ".ani", ".wav", ".doc", ".bmp"):
                check(w1 + "_" + w2 + e)
        if len(hits) == 4:
            break
    print("after w1_w2+ext:", hits, round(time.time()-t0, 1), "s")

# 形态4: w1 + sep + digit + sep + w2 + ext
if len(hits) < 4:
    for w1 in words:
        for w2 in words:
            for sep in ("", "_", "-"):
                for d in "0123456789":
                    for e in (".pcx", ".fnt", ".ani", ".wav"):
                        check(w1 + sep + d + sep + w2 + e)
            if len(hits) == 4:
                break
        if len(hits) == 4:
            break
    print("after w1+d+w2:", hits, round(time.time()-t0, 1), "s")

print("FINAL hits:", hits)
