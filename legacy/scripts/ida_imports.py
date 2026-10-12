# -*- coding: utf-8 -*-
"""S0.4：导出 IDA 的导入表视图（模块/序号/名字/IAT ea + xref 证据）。

用法：
    idat.exe -A -Lref\\imports_ida.log -Sscripts\\ida_imports.py TreasureIsl.exe

产出：
    ref/imports_ida.json   [{dll, entries: [{ea, name, ordinal, xrefs}]}]
    控制台（实际写入 -L 日志）: IDA-IMP modules=.. entries=.. / RESULT: PASS
"""
import json
import os

import ida_nalt
import idaapi
import idautils
import idc


def _xrefs(ea):
    out = []
    for x in idautils.XrefsTo(ea, 0):
        func = idaapi.get_func(x.frm)
        out.append(
            {
                "frm": x.frm,
                "type": int(x.type),
                "func_ea": func.start_ea if func else None,
                "func_name": idc.get_func_name(func.start_ea) if func else None,
            }
        )
    return out


def main():
    idb = idc.get_idb_path()
    root = os.path.dirname(os.path.abspath(idb))

    modules = []
    total = 0
    qty = ida_nalt.get_import_module_qty()
    for i in range(qty):
        dll = ida_nalt.get_import_module_name(i) or ""
        entries = []

        def cb(ea, name, ord_, _bucket=entries):
            _bucket.append(
                {
                    "ea": ea,
                    "name": name if name else None,
                    "ordinal": ord_ if ord_ else None,
                    "xrefs": _xrefs(ea),
                }
            )
            return True

        ida_nalt.enum_import_names(i, cb)
        total += len(entries)
        modules.append({"dll": dll, "entries": entries})

    out = os.path.join(root, "ref", "imports_ida.json")
    with open(out, "w", encoding="utf-8") as fh:
        json.dump(modules, fh, ensure_ascii=False, indent=1)

    named = sum(1 for m in modules for e in m["entries"] if e["name"])
    with_xref = sum(1 for m in modules for e in m["entries"] if e["xrefs"])
    print("IDA-IMP modules=%d entries=%d named=%d xrefs=%d" % (qty, total, named, with_xref))
    ok = total == 116
    print("IDA-IMP RESULT: %s" % ("PASS" if ok else "FAIL"))
    idaapi.qexit(0 if ok else 1)


main()
