# S1.11 第九轮：AP 串一致性反解（确定性）
# 1) name 型：exe 字符串直接命中（95 条）
# 2) ID 型：连续 key 差 = 33^k → 同前缀同后缀、数字递增；串首用模逆求解 + djb2 校验
import struct, zlib, json, re, string, itertools
from pathlib import Path

ROOT = Path(r"C:\金银岛")
DATA = ROOT / "data"
EXE = ROOT / "TreasureIsl.exe"
M4 = 33**4
INV = pow(M4, -1, 1 << 32)

def djb2(s):
    if isinstance(s, str):
        s = s.encode("ascii")
    h = 0
    for c in s:
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

# ---- 1) exe 字符串 name 型 ----
exe = EXE.read_bytes()
strs = set()
for m in re.finditer(rb"(?:[\x20-\x7E]\x00){2,}", exe):
    s = m.group().decode("utf-16-le")
    strs.add(s); strs.add(s.lower())
for m in re.finditer(rb"[\x20-\x7E]{2,}", exe):
    s = m.group().decode("ascii")
    strs.add(s); strs.add(s.lower())
for s in list(strs):
    if "\\" in s or "/" in s:
        strs.add(s.split("\\")[-1].split("/")[-1])
key2name = {}
for s in strs:
    k = djb2(s.lower().encode("ascii", "ignore"))
    if k not in key2name:
        key2name[k] = s

CONTAINERS = ["ui.pak", "glpics.pak", "glmap_m01.pak", "glmap_m02.pak",
              "cards.lpk", "music.lpk", "goods.lpk"]

SUFFIX4 = [".pcx", ".ani", ".wav", ".ini", ".fnt", ".map", ".bmp", ".txt",
           ".dat", ".mod", ".rec", ".sav", ".jpg", ".png"]
CHARS = string.ascii_lowercase + string.digits + "_.- "
prefix_of = {0: ""}
for L in range(1, 5):
    for tup in itertools.product(CHARS, repeat=L):
        s = "".join(tup)
        h = djb2(s)
        if h not in prefix_of:
            prefix_of[h] = s

def solve_digit2(k, S, tmax=4096):
    """name = P + d1 + d0 + S（|S|=4）→ 返回可校验的名字列表"""
    T = djb2(S)
    Q0 = ((k - T) & 0xFFFFFFFF) * INV & 0xFFFFFFFF
    names = []
    for t in range(tmax):
        Q = Q0 + t * (1 << 32)
        r0 = Q % 33
        d0 = next((d for d in range(48, 58) if d % 33 == r0), None)
        if d0 is None:
            continue
        Q1 = (Q - d0) // 33
        r1 = Q1 % 33
        d1 = next((d for d in range(48, 58) if d % 33 == r1), None)
        if d1 is None:
            continue
        A = (Q1 - d1) // 33
        if A < 0 or A * 1089 + d1 * 33 + d0 != Q:
            continue
        P = prefix_of.get(A)
        if P is None:
            continue
        name = f"{P}{chr(d1)}{chr(d0)}{S}"
        if djb2(name) == k:
            names.append(name)
            if len(names) >= 6:
                break
    return names

def solve_any_digits(k, S, ndig, tmax=4096):
    """推广：P + ndig 位数字 + S。ndig=3 → Q = A*33^3 + d2*33^2 + d1*33 + d0
    先按 2 位解，再单独处理 3 位：直接用前缀表枚举 A 再校验数字位。"""
    # 3 位：k = A*33^(3+4) + D*33^4 + T(S), D = 3 位数字的 running hash
    # 枚举 A（前缀表）不可行（990k×suffixes×keys 太慢），改为：枚举数字组合 000..999
    T = djb2(S)
    names = []
    lo, hi = 0, 10 ** ndig
    for n in range(lo, hi):
        dstr = f"{n:0{ndig}d}"
        # (k - T) ≡ (A*33^ndig + D)*33^4 (mod 2^32)
        D = djb2(dstr)          # P="" 时 running hash of digits alone
        # 但 P 存在时 A 耦合… 此处只处理 P 非空的 2 位情形；
        # 3 位且 P 空：k ≡ D*33^4 + T(S)
        if (D * M4 + T) & 0xFFFFFFFF == k:
            names.append(dstr + S)
            if len(names) >= 6:
                break
    return names

results = {}
for c in CONTAINERS:
    ks = keys_of(DATA / c)
    rows = []
    # 阶段1: exe name 型
    for i, k in enumerate(ks):
        rows.append({"idx": i, "key": hex(k), "name": key2name.get(k), "kind": None})
    # 阶段2: 未命中 → AP 分析
    ids = [i for i, r in enumerate(rows) if r["name"] is None]
    # 组 AP 串（相邻未命中 index 连续且差为 33 的幂）
    runs = []
    cur = []
    for a, b in zip(ids, ids[1:]):
        d = (ks[b] - ks[a]) & 0xFFFFFFFF
        pw = None
        if d and (d & (d - 1)) == 0:
            # d = 33^k? 检查
            for k2 in range(1, 8):
                if 33 ** k2 == d:
                    pw = k2
                    break
        if b == a + 1 and pw:
            cur.append((a, b, pw))
        else:
            if cur:
                runs.append(cur)
            cur = []
    if cur:
        runs.append(cur)
    # 对每个 AP 串首求解
    for run in runs:
        a = run[0][0]
        pw = run[0][2]           # 数字位数（在 suffix 之前的位数变化）
        k = ks[a]
        solved_run = None
        for S in SUFFIX4:
            if pw == 2:
                names = solve_digit2(k, S)
            elif pw == 3:
                names = solve_any_digits(k, S, 3)
            else:
                names = []
            if names:
                # 选与串长一致的：串内数字连续递增
                cand = []
                for base in names:
                    dgs = base[-(4 + pw):-4]
                    if dgs.isdigit():
                        n0 = int(dgs)
                        ok = all(f"{n0 + j:0{pw}d}" == f"{n0 + j:0{pw}d}" for j in range(len(run)))
                        if ok and n0 + len(run) - 1 < 10 ** pw:
                            cand.append(base)
                if cand:
                    solved_run = (S, cand[0])
                    break
        if solved_run:
            S, base = solved_run
            P = base[:-(4 + pw)]
            n0 = int(base[-(4 + pw):-4])
            for j, (a_idx, b_idx, _) in enumerate(run):
                rows[a_idx]["name"] = f"{P}{n0 + j:0{pw}d}{S}"
                rows[a_idx]["kind"] = "id-ap"
            # 串尾最后一个
            last = run[-1][1]
            rows[last]["name"] = f"{P}{n0 + len(run):0{pw}d}{S}"
            rows[last]["kind"] = "id-ap"
    # 阶段3: 剩余孤立键 → 单键求解（2位数字）
    for i, r in enumerate(rows):
        if r["name"] is None:
            k = ks[i]
            got = None
            for S in SUFFIX4:
                for nm in solve_digit2(k, S):
                    got = nm
                    break
                if got:
                    break
            if got is None:
                # P 空 + 2/3 位数字
                for S in SUFFIX4:
                    T = djb2(S)
                    for nd in (2, 3):
                        for n in range(10 ** nd):
                            ds = f"{n:0{nd}d}"
                            if (djb2(ds) * M4 + T) & 0xFFFFFFFF == k:
                                got = ds + S
                                break
                        if got:
                            break
                    if got:
                        break
            r["name"] = got
            if got:
                r["kind"] = "id-solo"
            else:
                r["kind"] = "unknown"
    results[c] = rows
    named = [r for r in rows if r["name"]]
    print(f"{c}: {len(named)}/{len(rows)}  name型={sum(1 for r in rows if r['kind'] is None and r['name'])} "
          f"AP={sum(1 for r in rows if r['kind']=='id-ap')} solo={sum(1 for r in rows if r['kind']=='id-solo')} "
          f"unk={sum(1 for r in rows if r['kind']=='unknown')}")
    unk = [r for r in rows if not r["name"]]
    if unk:
        print("   unknown:", [r["key"] for r in unk[:8]])
    # 全体校验
    bad = [r for r in rows if r["name"] and djb2(r["name"].lower()) != ks[r["idx"]]]
    assert not bad, f"{c} 校验失败: {bad[:2]}"

total = sum(len(v) for v in results.values())
named = sum(1 for v in results.values() for r in v if r["name"])
print(f"\nTOTAL named {named}/{total} (全部 djb2 校验通过)")
Path(ROOT / "evidence" / "S1.11-final.json").write_text(
    json.dumps(results, indent=1, ensure_ascii=False), encoding="utf-8")
print("wrote evidence/S1.11-final.json")
