# S1.11 最后尝试：UTF-16 中文名 / 空格 / 全串组合
import re, struct, zlib, time
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"

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
hits = {}

def check(nm):
    if nm is None or nm in hits:
        return
    if djb2(nm) in TARGET:
        hits[TARGET[djb2(nm)]] = nm
        print("HIT", TARGET[djb2(nm)], repr(nm))

exe = open(ROOT / "TreasureIsl.exe", "rb").read()

# 1) exe 全部 UTF-16 串（含中文/宽字符），及其 + 扩展名
u16 = []
for m in re.finditer(rb"(?:[\x00-\xFF][\x00]){2,}", exe):
    try:
        s = m.group().decode("utf-16-le")
    except Exception:
        continue
    if all(32 <= ord(c) < 127 or ord(c) > 127 for c in s) and s.strip():
        u16.append(s.strip())
print("utf16 strings:", len(u16))
EXTS = [".pcx", ".fnt", ".ani", ".wav", ".bmp", ".dat", ".txt", ".ini", ".doc",
        ".xls", ".log", ".cfg", ".tbl", ".mod", ".sav", ".map", ".htm"]
for s in u16:
    check(s)
    for e in EXTS:
        check(s + e)
        check(s.lower() + e)
        check(s.upper() + e)
print("after u16:", hits)

# 2) ascii 全串（原样、含空格/标点）+ 扩展
for m in re.finditer(rb"[\x20-\x7E]{2,60}", exe):
    s = m.group().decode("ascii")
    check(s)
    for e in EXTS:
        check(s + e)
print("after ascii-full:", hits)

# 3) 空格分隔词组合 w1 + ' ' + w2 + ext / w1 + '.' + w2
words = set()
for m in re.finditer(rb"[\x20-\x7E]{2,30}", exe):
    s = m.group().decode("ascii")
    for w in re.split(r"[^A-Za-z0-9_]+", s):
        if 1 <= len(w) <= 20:
            words.add(w)
words = sorted(words)
t0 = time.time()
for w1 in words:
    for w2 in words:
        for sep in (" ", ".", "+", "#", "&"):
            for e in (".pcx", ".fnt", ".ani", ".wav", ".doc"):
                check(w1 + sep + w2 + e)
    if len(hits) == 4:
        break
print("after sep-combo:", hits, round(time.time()-t0, 1), "s")

# 4) 中文词 + 扩展（exe UTF-16 中文子串）
zh = [s for s in u16 if any(ord(c) > 127 for c in s)]
print("zh strings:", len(zh), zh[:10])
for s in zh:
    for e in EXTS:
        check(s + e)
    # 拆词（中文串内可能拼接多个资源名）
print("after zh:", hits)
print("FINAL:", hits)
