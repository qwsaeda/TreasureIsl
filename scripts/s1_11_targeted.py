# S1.11 定向解 7 条非权威键
import struct, zlib, re, time
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"
TMP = Path(r"C:\Users\Administrator\AppData\Local\Temp\ini_chk")

def djb2(s):
    h = 0
    for c in s.lower().encode():
        h = (h * 33 + c) & 0xFFFFFFFF
    return h

def entries(p):
    b = p.read_bytes()
    n = struct.unpack_from("<I", b, 0x26)[0]
    L = struct.unpack_from("<I", b, 0x2E)[0]
    ds = struct.unpack_from("<I", b, 0x32)[0]
    raw = b[0x36:0x36 + L]
    if raw[0] & 0xF == 8 and ((raw[0] << 8) | raw[1]) % 31 == 0:
        raw = zlib.decompress(raw)
    out = []
    for i in range(n):
        k, off, ln, cl = struct.unpack_from("<4I", raw, i * 16)
        out.append((k, off, ln, cl))
    return b, ds, out

# 1) 嗅探未知条目内容
for cont, idx in [("ui.pak", 121), ("cards.lpk", 1), ("cards.lpk", 2), ("glpics.pak", 3), ("glpics.pak", 11), ("glpics.pak", 34)]:
    b, ds, ents = entries(DATA / cont)
    k, off, ln, cl = ents[idx]
    blob = b[ds + off: ds + off + cl]
    if blob[0] == 0x78:
        blob = zlib.decompress(blob)
    print(f"{cont}[{idx}] key={k:#x} usize={ln} head={blob[:16].hex(' ')}")
    # PCX 判别
    if len(blob) > 128 and blob[0] == 0x0A:
        xmin, ymin, xmax, ymax = struct.unpack_from("<4H", blob, 4)
        print(f"   -> PCX {xmax-xmin+1}x{ymax-ymin+1} depth={blob[3]}")
    if blob[:4] == b"RIFF":
        print("   -> WAV", struct.unpack_from("<I", blob, 24)[0], "Hz")
print()

# 2) m02 4 连 AP（gap=33^4，名 = A + D + '.pcx'，D 逐字符 +1）
TARGET4 = {0xAF495A0A, 0xAF5B728B, 0xAF6D8B0C, 0xAF7FA38D}

# 前缀候选：exe 词表 + mapinfo 词根 + 场景名
exe = open(ROOT / "TreasureIsl.exe", "rb").read()
words = set()
for m in re.finditer(rb"[\x20-\x7E]{2,30}", exe):
    s = m.group().decode("ascii")
    for w in re.split(r"[^A-Za-z0-9_]+", s):
        if 1 <= len(w) <= 24:
            words.add(w)
for mi in TMP.rglob("3753dc98.bin"):
    t = mi.read_bytes().decode("utf-16-le", "ignore")
    for nm in re.findall(r"filename=(\S+)", t):
        stem = nm.rsplit(".", 1)[0]
        words.add(stem)
        for part in re.split(r"[_\-]", stem):
            words.add(part)
        # 全 stem 也作前缀（去掉尾数字再拼）
        m2 = re.match(r"^([A-Za-z_]+?)(\d+)$", stem)
        if m2:
            words.add(m2.group(1))
words |= {"", "b", "B", "g", "s", "diji", "shui", "huo", "zhuzi", "bd", "gr", "card", "event", "war", "appo"}
print("prefix words:", len(words))

exts = [".pcx", ".fnt", ".ani"]
t0 = time.time()
found = {}
for w in words:
    for e in exts:
        # 连续 4 键（支持 '9'+1=':' 越界也检查，但只要合法 ASCII）
        ks = []
        for d in range(4):
            nm = w + chr(ord("0") + d) + e  # 从 '0' 起步
            ks.append((nm, djb2(nm)))
        # 试任意起始数字 D0
        for d0 in range(10):
            run = [w + chr(ord("0") + d0 + d) + e for d in range(4) if ord("0") + d0 + d <= ord("9")]
            if len(run) < 4:
                continue
            if all(djb2(n) in TARGET4 for n in run):
                found = {n: djb2(n) for n in run}
                print("AP HIT:", run)
    if found:
        break
# 也试 '任意字符递增'（不限数字）：A 长度 1..8 穷举太贵，仅数字
print("AP found:", found)
print("elapsed", round(time.time() - t0, 1))

# 3) 通用：word + digit + ext + word2? 及 word + digit(1-2位) + ext（覆盖 m02 4 键可能形态）
if not found:
    hits = {}
    for w in sorted(words):
        for nd in (1, 2):
            for n in range(10 ** nd):
                ds = f"{n:0{nd}d}"
                for e in exts:
                    k = djb2(w + ds + e)
                    if k in TARGET4:
                        hits[k] = w + ds + e
    print("digit-suffix hits:", {hex(k): v for k, v in hits.items()})

# 4) ui[121] 与 cards fnt 专用
targets = {0xF6EF0287: "ui121", 0xC5561D44: "cards1", 0xC56835C5: "cards2"}
if True:
    hits2 = {}
    for w in sorted(words):
        for nd in (1, 2, 3):
            for n in range(10 ** nd):
                ds = f"{n:0{nd}d}"
                for e in (".wav", ".pcx", ".fnt", ".ani", ".bmp", ".dat", ".txt", ".ini", ".mod"):
                    k = djb2(w + ds + e)
                    if k in targets:
                        hits2[targets[k]] = w + ds + e
    print("ui/cards digit hits:", hits2)
