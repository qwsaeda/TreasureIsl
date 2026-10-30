# -*- coding: utf-8 -*-
"""S1.4b 辅助：按 IDA 地址读 PE 内的 dword 表（不依赖 pefile）。

F-312（实测）：TreasureIsl.exe 的 ImageBase = 0x10000，因此
**IDA 地址 = RVA + 0x10000**。用 IDA 地址查表必须先减 0x10000，
直接当 RVA 用会读到 .rdata 里的相邻字符串（曾误读成 "tro.map"）。
"""
import struct
import sys

PE = r"C:\金银岛\TreasureIsl.exe"
IMAGEBASE = 0x10000


def sections(data):
    pe_off = struct.unpack_from("<I", data, 0x3C)[0]
    assert data[pe_off : pe_off + 4] == b"PE\0\0", "not a PE"
    nsec = struct.unpack_from("<H", data, pe_off + 6)[0]
    opt_size = struct.unpack_from("<H", data, pe_off + 20)[0]
    opt_off = pe_off + 24
    sec_off = opt_off + opt_size
    out = []
    for i in range(nsec):
        base = sec_off + i * 40
        name = data[base : base + 8].rstrip(b"\0").decode("ascii", "replace")
        vsize, va, rawsize, rawoff = struct.unpack_from("<IIII", data, base + 8)
        out.append((name, va, vsize, rawoff, rawsize))
    return out


def rva_to_off(data, rva):
    for name, va, vsize, rawoff, rawsize in sections(data):
        if va <= rva < va + max(vsize, rawsize):
            return rawoff + (rva - va), name
    return None, None


def dwords(data, rva, count):
    off, sec = rva_to_off(data, rva)
    if off is None:
        raise SystemExit("RVA 0x%X 不在任何 section 内" % rva)
    vals = struct.unpack_from("<%dI" % count, data, off)
    return sec, off, vals


def main():
    with open(PE, "rb") as fh:
        data = fh.read()
    for name, va, vsize, rawoff, rawsize in sections(data):
        print("section %-8s va=0x%06X vsize=0x%06X rawoff=0x%06X rawsize=0x%06X"
              % (name, va, vsize, rawoff, rawsize))
    ida = int(sys.argv[1], 16) if len(sys.argv) > 1 else 0x6F518
    count = int(sys.argv[2]) if len(sys.argv) > 2 else 16
    rva = ida - IMAGEBASE
    sec, off, vals = dwords(data, rva, count)
    print("\nIDA 0x%X -> RVA 0x%X -> %s file off 0x%X" % (ida, rva, sec, off))
    for i, v in enumerate(vals):
        print("  [%2d] +%-3d = 0x%08X%s" % (i, i * 4, v, "  <-" if False else ""))


main()
