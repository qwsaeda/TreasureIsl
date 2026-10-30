# S1.8b 分析：地形 .Map 区域记录流切表 + 字段解码
import struct
from pathlib import Path


def load(name):
    b = Path("data") / name
    data = b.read_bytes()
    w = list(struct.unpack("<%dI" % (len(data) // 4), data))
    return w, len(data)


def u16s(v):
    return (v & 0xFFFF, v >> 16)


def scan(name):
    w, total = load(name)
    gw, gh, flags, reg_off = w[0], w[1], w[2], w[3]
    count = w[5]
    print(f"=== {name} gw={gw} gh={gh} flags={flags} reg_off={reg_off} count={count} words={len(w)}")
    # 从 w6 起按 9 字词扫描记录，r4(索引4)==1 或 2 为记录标记；
    # r6(=f6)/32 基址跳变处为切表点。
    i = 6
    tables = []
    cur = []
    cur_base = None
    cur_last = None
    while i + 9 <= len(w):
        r = w[i : i + 9]
        if r[4] not in (1, 2):
            break
        if r[4] == 1:
            base = r[6] // 32
            if cur_base is None:
                cur_base = base
                cur_last = base
            elif base != cur_last + 1:
                # 表内 f6/32 严格 +1；跳变（回跳/跳跃）= 新表
                tables.append((cur_base, cur))
                cur = []
                cur_base = base
                cur_last = base
            else:
                cur_last = base
            cur.append((i, r))
        else:
            # type=2 记录：连续成段
            if not cur or cur[0][1][4] != 2:
                if cur:
                    tables.append((cur_base, cur))
                cur = []
                cur_base = None
                cur_last = None
            cur.append((i, r))
        i += 9
    if cur:
        tables.append((cur_base, cur))
    print(f"  记录流扫出 {len(tables)} 段，尾部剩余词 {len(w) - i}")
    for ti, (base, recs) in enumerate(tables):
        typ = recs[0][1][4]
        f6s = [r[6] for _, r in recs]
        print(f"  段{ti}: type={typ} n={len(recs)} f6base={f6s[0] if f6s else '-'}..{f6s[-1] if f6s else '-'}")
    return w, tables, i


def dump_tables(name):
    w, tables, end_i = scan(name)
    gw = w[0]
    for ti, (base, recs) in enumerate(tables):
        typ = recs[0][1][4]
        print(f"  -- 段{ti} (type={typ}, n={len(recs)}, 起词{recs[0][0]})")
        for idx, (wi, r) in enumerate(recs):
            lo0, hi0 = u16s(r[0])
            lo1, hi1 = u16s(r[1])
            lo8, hi8 = u16s(r[8])
            cell = lo8
            cx, cy = cell % gw, cell // gw if gw else (0, 0)
            extra = ""
            if r[0]:
                extra += f" f0=({lo0},{hi0})"
            if r[1]:
                extra += f" f1=({lo1},{hi1})"
            if hi8:
                extra += f" f8hi=({lo8},{hi8})"
            print(
                f"    [{idx:2d}] w{wi:<5d} f5={r[5]:<3d} f6={r[6]:<5d} f7={r[7]:<4d} "
                f"f8={r[8]:<10d} cell={cell:4d}=({cx:2d},{cy:2d}){extra}"
            )
    print(f"  记录流结束于 w{end_i}（文件字节 {end_i * 4}），其后 {len(w) - end_i} 词：")
    tail = w[end_i : end_i + 40]
    print("   ", tail)


if __name__ == "__main__":
    for nm in ("glmapm01.Map", "glmapm02.Map", "glmapm03.Map"):
        dump_tables(nm)
        print()
