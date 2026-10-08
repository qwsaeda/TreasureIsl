# -*- coding: utf-8 -*-
"""IDA headless 导出（S0.2）。

用法：
    idat64.exe -A -Sscripts\\ida_dump.py TreasureIsl.exe

产出（相对 exe/i64 所在目录）：
    ref/pseudocode/<start>_<name>.c   每函数一个文件（Hex-Rays 失败则存反汇编）
    ref/functions.csv                 addr/size/name/callers/callees/str_xrefs

自检（DoD）：伪代码文件数 == CSV 数据行数 == IDA 函数数，否则 qexit(1)。
"""
import csv
import os
import re

import ida_hexrays
import idaapi
import idautils
import idc

COLUMNS = ("addr", "size", "name", "callers", "callees", "str_xrefs")
SAFE_NAME = re.compile(r"[^0-9A-Za-z_]+")


def _label(ea):
    """地址 → `AAAAAAAA:name`，名字优先取函数名。"""
    name = idc.get_func_name(ea) or idc.get_name(ea) or ""
    return "%08X:%s" % (ea, name)


def _func_name(ea):
    return idc.get_func_name(ea) or ("sub_%X" % ea)


def _safe_name(name):
    safe = SAFE_NAME.sub("_", name).strip("_")
    return safe or "unnamed"


def _disasm(func):
    """Hex-Rays 反编译失败时的兜底：逐行反汇编。"""
    lines = ["/* HEX-RAYS DECOMPILE FAILED -> disasm fallback */"]
    ea = func.start_ea
    while ea != idc.BADADDR and ea < func.end_ea:
        lines.append(idc.generate_disasm_line(ea, idc.GENDSM_FORCE_CODE))
        ea = idc.next_head(ea, func.end_ea)
    return "\n".join(lines)


def _str_xrefs_index():
    """string ea → 引用该字符串的函数 start_ea 集合。"""
    index = {}
    for s in idautils.Strings():
        sea = s.ea
        text = str(s)
        for xref in idautils.XrefsTo(sea, 0):
            func = idaapi.get_func(xref.frm)
            if func is not None:
                index.setdefault(func.start_ea, {}).setdefault(sea, text)
    return index


def _callers(start_ea):
    out = set()
    for ref in idautils.CodeRefsTo(start_ea, 0):
        func = idaapi.get_func(ref)
        if func is not None and func.start_ea != start_ea:
            out.add(func.start_ea)
    return sorted(out)


def _callees(func):
    out = set()
    ea = func.start_ea
    while ea != idc.BADADDR and ea < func.end_ea:
        for ref in idautils.CodeRefsFrom(ea, 0):
            if func.start_ea <= ref < func.end_ea:
                continue
            out.add(ref)
        ea = idc.next_head(ea, func.end_ea)
    return sorted(out)


def main():
    if not ida_hexrays.init_hexrays_plugin():
        print("IDA-DUMP RESULT: FAIL reason=hexrays-unavailable")
        idaapi.qexit(1)

    idb = idc.get_idb_path()
    root = os.path.dirname(os.path.abspath(idb))
    ref_dir = os.path.join(root, "ref")
    pc_dir = os.path.join(ref_dir, "pseudocode")
    os.makedirs(pc_dir, exist_ok=True)

    for stale in os.listdir(pc_dir):
        if stale.endswith(".c"):
            os.remove(os.path.join(pc_dir, stale))

    functions = list(idautils.Functions())
    str_index = _str_xrefs_index()
    ok = 0
    failed = []
    rows = []

    for start_ea in functions:
        func = idaapi.get_func(start_ea)
        if func is None:
            print("IDA-DUMP RESULT: FAIL reason=missing-func ea=0x%X" % start_ea)
            idaapi.qexit(1)

        name = _func_name(start_ea)
        header = [
            "/* ea:   0x%08X */" % start_ea,
            "/* name: %s */" % name,
            "/* size: %d */" % (func.end_ea - start_ea),
            "",
        ]
        try:
            cfunc = ida_hexrays.decompile(start_ea)
            body = str(cfunc) if cfunc is not None else _disasm(func)
            if cfunc is None:
                failed.append(name)
            else:
                ok += 1
        except Exception as exc:  # noqa: BLE001 - 单函数失败不应中断整体导出
            body = "%s\n/* error: %s */" % (_disasm(func), exc)
            failed.append(name)

        fname = "%08X_%s.c" % (start_ea, _safe_name(name))
        with open(os.path.join(pc_dir, fname), "w", encoding="utf-8", errors="replace") as fh:
            fh.write("\n".join(header) + body + "\n")

        str_eas = sorted(str_index.get(start_ea, {}))
        rows.append(
            {
                "addr": "%08X" % start_ea,
                "size": func.end_ea - start_ea,
                "name": name,
                "callers": ";".join(_label(ea) for ea in _callers(start_ea)),
                "callees": ";".join(_label(ea) for ea in _callees(func)),
                "str_xrefs": ";".join(
                    "%08X:%s" % (ea, str_index[start_ea][ea]) for ea in str_eas
                ),
            }
        )

    csv_path = os.path.join(ref_dir, "functions.csv")
    with open(csv_path, "w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=COLUMNS)
        writer.writeheader()
        writer.writerows(rows)

    files = [f for f in os.listdir(pc_dir) if f.endswith(".c")]
    print(
        "IDA-DUMP functions=%d files=%d csv_rows=%d hexrays_ok=%d hexrays_failed=%d"
        % (len(functions), len(files), len(rows), ok, len(failed))
    )
    if failed:
        print("IDA-DUMP failed_functions: %s" % ";".join(failed))
    passed = len(functions) == len(files) == len(rows)
    print("IDA-DUMP RESULT: %s" % ("PASS" if passed else "FAIL"))
    idaapi.qexit(0 if passed else 1)


main()
