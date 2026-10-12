# -*- coding: utf-8 -*-
"""S0.4 第一步：手写 PE32 导入表解析（无 pefile，见 EXP-006）。

产出：
    ref/imports_raw.json   每个导入的 dll/ordinal/hint/name/IAT_VA/thunk_rva（未定名前的原始事实）

用法：
    python scripts/s0_4_imports.py [TreasureIsl.exe]
"""
import json
import os
import struct
import sys

IMAGE_ORDINAL_FLAG32 = 0x80000000


def u16(b, off):
    return struct.unpack_from("<H", b, off)[0]


def u32(b, off):
    return struct.unpack_from("<I", b, off)[0]


def read_cstr(data, off, limit=260):
    end = data.find(b"\x00", off, off + limit)
    if end < 0:
        raise ValueError("unterminated cstring at 0x%X" % off)
    return data[off:end].decode("ascii", "replace")


def parse_imports(path):
    with open(path, "rb") as fh:
        data = fh.read()

    if data[:2] != b"MZ":
        raise ValueError("not MZ")
    e_lfanew = u32(data, 0x3C)
    if data[e_lfanew:e_lfanew + 4] != b"PE\x00\x00":
        raise ValueError("not PE")
    coff = e_lfanew + 4
    nsec = u16(data, coff + 2)
    opt_size = u16(data, coff + 16)
    opt = coff + 20
    magic = u16(data, opt)
    if magic != 0x10B:
        raise ValueError("not PE32 (magic=0x%X)" % magic)
    # 数据目录：PE32 的 OptionalHeader 里 image base 之后是 16 个 data dir
    # NumberOfRvaAndSizes @ opt+92, dirs @ opt+96
    nrva = u32(data, opt + 92)
    dirs = opt + 96
    if nrva < 2:
        raise ValueError("no import directory")
    imp_rva = u32(data, dirs + 1 * 8)
    imp_size = u32(data, dirs + 1 * 8 + 4)

    # 节表：rva → file offset
    sect = []
    sec_off = opt + opt_size
    for i in range(nsec):
        s = sec_off + i * 40
        vsize = u32(data, s + 8)
        va = u32(data, s + 12)
        raw_size = u32(data, s + 16)
        raw_ptr = u32(data, s + 20)
        sect.append((va, vsize, raw_ptr, raw_size))

    def rva2off(rva):
        for va, vsize, raw_ptr, raw_size in sect:
            if va <= rva < va + max(vsize, raw_size):
                return raw_ptr + (rva - va)
        raise ValueError("rva 0x%X not in any section" % rva)

    if imp_rva == 0:
        return []

    entries = []
    desc_off = rva2off(imp_rva)
    desc_size = 20
    idx = 0
    while True:
        d = desc_off + idx * desc_size
        oft = u32(data, d + 0)
        name_rva = u32(data, d + 12)
        ft = u32(data, d + 16)
        if oft == 0 and name_rva == 0 and ft == 0:
            break
        dll = read_cstr(data, rva2off(name_rva))
        int_rva = oft if oft else ft
        int_off = rva2off(int_rva)
        iat_rva = ft
        j = 0
        while True:
            thunk = u32(data, int_off + j * 4)
            if thunk == 0:
                break
            iat_va = 0x10000000 + 0  # placeholder, fill below (image base handled outside)
            if thunk & IMAGE_ORDINAL_FLAG32:
                entries.append(
                    {
                        "dll": dll,
                        "ordinal": thunk & 0xFFFF,
                        "hint": None,
                        "name": None,
                        "iat_rva": iat_rva + j * 4,
                        "int_index": j,
                    }
                )
            else:
                ho = rva2off(thunk)
                hint = u16(data, ho)
                nm = read_cstr(data, ho + 2)
                entries.append(
                    {
                        "dll": dll,
                        "ordinal": None,
                        "hint": hint,
                        "name": nm,
                        "iat_rva": iat_rva + j * 4,
                        "int_index": j,
                    }
                )
            j += 1
        idx += 1

    image_base = u32(data, opt + 28)
    for e in entries:
        e["iat_va"] = image_base + e["iat_rva"]
        e.pop("int_index", None)
        e.pop("iat_rva", None)
    return entries


def main():
    exe = sys.argv[1] if len(sys.argv) > 1 else "TreasureIsl.exe"
    entries = parse_imports(exe)

    # 汇总
    by_dll = {}
    for e in entries:
        by_dll.setdefault(e["dll"], []).append(e)

    print("imports total = %d" % len(entries))
    for dll, es in by_dll.items():
        named = sum(1 for e in es if e["name"])
        print(
            "  %-12s count=%-4d named=%-4d ordinal_only=%d"
            % (dll, len(es), named, len(es) - named)
        )

    out = os.path.join("ref", "imports_raw.json")
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(entries, fh, ensure_ascii=False, indent=1)
    print("written: %s" % out)


if __name__ == "__main__":
    main()
