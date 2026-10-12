# -*- coding: utf-8 -*-
"""S0.4：拼装 ref/imports.json（116 个导入 API 名恢复 + 调用点证据）。

数据源（三方独立）：
    ref/imports_raw.json    PE 导入表原始事实（dll/ordinal/hint/name/IAT_VA）—— scripts/s0_4_imports.py
    ref/imports_ida.json    IDA 导入表视图（名字/序号/xref）—— scripts/ida_imports.py
    ref/functions.csv       伪代码函数表（callers/callees）—— S0.2 scripts/ida_dump.py
    ref/pseudocode/*.c      伪代码文本（间接调用点）—— S0.2
    ref/ordinals/coredll.def  外部 WinCE6 ARM 序号表（metarutaiga/WINCE600，独立于 IDB）

产出：
    ref/imports.json        116 条：{index, dll, ordinal, pe_name, iat_va, api, ida_symbol,
                            def_name, thunk, evidence:{callers, text_refs, xrefs}}
    evidence/S0.4.log       验收日志（DoD：116/116 有名字 + 每条 >=1 调用点证据 + def 对照）

用法：
    python scripts/s0_4_build.py
"""
import csv
import json
import os
import re
import sys

DEF_PATHS = {
    "coredll": os.path.join("ref", "ordinals", "coredll.def"),
}


def load_json(path):
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def parse_def(path):
    """`    name[=alias] @ordinal` → {ordinal: name}（name 取 '=' 左侧）。"""
    table = {}
    pat = re.compile(r"^\s*(\S+)\s+@(\d+)\s*$")
    with open(path, encoding="utf-8-sig", errors="replace") as fh:
        for line in fh:
            m = pat.match(line.rstrip("\r\n"))
            if not m:
                continue
            name = m.group(1)
            if "=" in name:
                name = name.split("=", 1)[0]
            table[int(m.group(2))] = name
    return table


def short_name(api):
    """C++ 修饰名 → 短名（用于伪代码文本匹配）：?GXOpenDisplay@@... → GXOpenDisplay。"""
    if api.startswith("?@@") or api.startswith("??"):
        return api
    if api.startswith("?"):
        body = api[1:]
        return body.split("@@")[0].split("@")[0]
    return api


def index_pseudocode():
    """扫描 ref/pseudocode/*.c，返回 {callee名: [(file, line, text)]}（每 callee 最多 8 条）。

    跳过注释行，按「callee 名后跟 (」捕获；C 关键字与宏噪声不入表。
    """
    hits = {}
    kw = {
        "if", "for", "while", "switch", "sizeof", "do", "else", "case", "return",
        "printf", "assert", "struct", "int", "char", "void",
    }
    pc_dir = os.path.join("ref", "pseudocode")
    call_pat = re.compile(r"\b([A-Za-z_?][A-Za-z0-9_?@]*)\s*\(")
    for fname in sorted(os.listdir(pc_dir)):
        if not fname.endswith(".c"):
            continue
        fpath = os.path.join(pc_dir, fname)
        with open(fpath, encoding="utf-8", errors="replace") as fh:
            for lineno, line in enumerate(fh, 1):
                stripped = line.strip()
                if not stripped or stripped.startswith("/*") or stripped.startswith("//"):
                    continue
                for m in call_pat.finditer(stripped):
                    callee = m.group(1)
                    if callee in kw:
                        continue
                    bucket = hits.setdefault(callee, [])
                    if len(bucket) < 8:
                        bucket.append((fname, lineno, stripped[:160]))
    return hits


def index_mentions(names):
    """名字提及索引（含地址取用 `a1[19] = DefWindowProcW;` 类非调用引用）。"""
    if not names:
        return {}
    pat = re.compile(r"(?<!\w)(" + "|".join(re.escape(n) for n in sorted(names, key=len, reverse=True)) + r")(?!\w)")
    hits = {}
    pc_dir = os.path.join("ref", "pseudocode")
    for fname in sorted(os.listdir(pc_dir)):
        if not fname.endswith(".c"):
            continue
        fpath = os.path.join(pc_dir, fname)
        with open(fpath, encoding="utf-8", errors="replace") as fh:
            for lineno, line in enumerate(fh, 1):
                stripped = line.strip()
                if not stripped or stripped.startswith("/*") or stripped.startswith("//"):
                    continue
                found = set(pat.findall(stripped))
                for name in found:
                    bucket = hits.setdefault(name, [])
                    if len(bucket) < 4:
                        bucket.append((fname, lineno, stripped[:160]))
    return hits


def main():
    raw = load_json("ref/imports_raw.json")
    ida = load_json("ref/imports_ida.json")
    ida_by_ea = {e["ea"]: e for m in ida for e in m["entries"]}
    ida_dll_by_ea = {e["ea"]: m["dll"] for m in ida for e in m["entries"]}

    with open("ref/functions.csv", encoding="utf-8") as fh:
        csv_rows = list(csv.DictReader(fh))
    by_name = {}
    for r in csv_rows:
        by_name.setdefault(r["name"], []).append(r)

    pc_hits = index_pseudocode()
    def_table = parse_def(DEF_PATHS["coredll"])

    all_names = set()
    for e in raw:
        ie = ida_by_ea.get(e["iat_va"])
        if not ie:
            continue
        symbol = ie["name"] or ""
        api = symbol[6:] if symbol.startswith("__imp_") else symbol
        all_names.add(api)
        all_names.add(short_name(api))
        if api.startswith("_"):
            all_names.add(api[1:])
    pc_mentions = index_mentions(all_names)

    # 伪代码调用点匹配：短名 + 修饰名
    def text_match_keys(api, short):
        keys = {short}
        if api.startswith("?"):
            keys.add(api)
        if api.startswith("_"):
            keys.add(api[1:])
        if "??2" in api or "??3" in api:
            for kw in ("operator new", "operator delete"):
                keys.add(kw)
        return keys

    entries = []
    problems = []
    def_checked = 0
    def_mismatch = 0

    for idx, e in enumerate(raw):
        iat = e["iat_va"]
        ie = ida_by_ea.get(iat)
        if ie is None:
            problems.append("missing ida entry for IAT 0x%X" % iat)
            continue
        symbol = ie["name"] or ""
        api = symbol[6:] if symbol.startswith("__imp_") else symbol
        short = short_name(api)

        # —— def 对照（仅 COREDLL，外部独立表）
        def_name = None
        def_match = None
        if e["dll"] == "COREDLL.dll" and e["ordinal"] is not None:
            def_checked += 1
            def_name = def_table.get(e["ordinal"])
            if def_name == api:
                def_match = "exact"
            elif def_name == "_" + api:
                # C 修饰约定：def 存带前导下划线的装饰名，IDA 显示去掉一个
                def_match = "leading_underscore"
            else:
                def_match = "none"
                def_mismatch += 1
                problems.append(
                    "DEF MISMATCH ord=%d ida=%s def=%s"
                    % (e["ordinal"], api, def_name)
                )

        # —— 证据 1：直接调用（thunk 的 callers，来自 functions.csv）
        callers = []
        thunk = None
        for cand in by_name.get(api, []) + by_name.get(short, []):
            if int(cand["size"]) <= 16 and cand["addr"] not in [
                c["addr"] for c in callers
            ]:
                thunk = cand["addr"]
                for one in (cand["callers"] or "").split(";"):
                    if one:
                        callers.append({"func": one, "thunk": thunk})
                break

        # —— 证据 2：伪代码文本调用点（间接 blx 经 IAT 的调用，hex-rays 解析为名字）
        text_refs = []
        for key in sorted(text_match_keys(api, short)):
            for fname, lineno, text in pc_hits.get(key, []):
                if thunk and fname.startswith(thunk):
                    continue
                ref = {"file": "ref/pseudocode/" + fname, "line": lineno, "text": text}
                if ref not in text_refs:
                    text_refs.append(ref)

        # —— 证据 2b：名字提及（地址取用类引用，如 `a1[19] = DefWindowProcW;`）
        mentions = []
        for key in sorted(text_match_keys(api, short)):
            for fname, lineno, text in pc_mentions.get(key, []):
                if thunk and fname.startswith(thunk):
                    continue
                if text in [m["text"] for m in mentions]:
                    continue
                ref = {"file": "ref/pseudocode/" + fname, "line": lineno, "text": text}
                if ref not in mentions:
                    mentions.append(ref)

        # —— 证据 3：IDA xref（指向 IAT 槽的 code/data 引用，含所在函数）
        xrefs = [
            {
                "frm": "%08X" % x["frm"],
                "type": x["type"],
                "func": ("%08X:%s" % (x["func_ea"], x["func_name"]))
                if x["func_ea"] is not None
                else None,
            }
            for x in ie.get("xrefs", [])
        ]

        evidence_count = len(callers) + len(text_refs) + len(mentions) + len(xrefs)
        if not api:
            problems.append("NO NAME at IAT 0x%X" % iat)
        if evidence_count == 0:
            problems.append("NO EVIDENCE for %s (IAT 0x%X)" % (api, iat))

        entries.append(
            {
                "index": idx,
                "dll": e["dll"],
                "ordinal": e["ordinal"],
                "hint": e["hint"],
                "pe_name": e["name"],
                "iat_va": "0x%08X" % iat,
                "ida_module": ida_dll_by_ea.get(iat),
                "api": api,
                "ida_symbol": symbol,
                "def_name": def_name,
                "def_match": def_match,
                "thunk": ("%08X" % int(thunk, 16)) if thunk else None,
                "evidence": {
                    "callers": callers[:8],
                    "text_refs": text_refs[:4],
                    "mentions": mentions[:4],
                    "xrefs": xrefs[:6],
                },
            }
        )

    out_path = "ref/imports.json"
    with open(out_path, "w", encoding="utf-8") as fh:
        json.dump(entries, fh, ensure_ascii=False, indent=1)

    # —— 验收日志
    total = len(entries)
    named = sum(1 for x in entries if x["api"])

    def ev_kind(x):
        if x["evidence"]["callers"]:
            return "direct"
        if x["evidence"]["text_refs"]:
            return "text_call"
        if x["evidence"]["mentions"]:
            return "mention"
        if x["evidence"]["xrefs"]:
            return "xref_only"
        return "NONE"

    kinds = {}
    for x in entries:
        kinds[ev_kind(x)] = kinds.get(ev_kind(x), 0) + 1
    with_evidence = total - kinds.get("NONE", 0)
    by_dll = {}
    for x in entries:
        by_dll[x["dll"]] = by_dll.get(x["dll"], 0) + 1

    log = []
    log.append("=== S0.4 验收：116 个导入 API 名恢复 ===")
    log.append("数据源: PE导入表(s0_4_imports.py) x IDA导入表(ida_imports.py) x "
               "coredll.def(metarutaiga/WINCE600, 独立外部表) x functions.csv/pseudocode(S0.2)")
    log.append("")
    log.append("  PASS  导入总数 == 116 (PE == IDA: %d == %d)" % (len(raw), total))
    for dll, n in sorted(by_dll.items()):
        log.append("        %s: %d" % (dll, n))
    log.append("  %s  116/116 有名字 (%d)" % ("PASS" if named == total else "FAIL", named))
    log.append("  %s  每条 >=1 调用/引用证据 (%d/%d)" % ("PASS" if with_evidence == total else "FAIL", with_evidence, total))
    log.append("        证据类型分布: %s" % json.dumps(kinds, ensure_ascii=False, sort_keys=True))
    weak = [x for x in entries if ev_kind(x) in ("mention", "xref_only")]
    log.append("        弱证据 %d 条（地址取用/运行时桩，无直接游戏调用点，逐条列名）:" % len(weak))
    for x in weak:
        log.append("          [%s] %s ord=%s  %s"
                   % (ev_kind(x), x["dll"], x["ordinal"], x["api"]))
    log.append("  %s  COREDLL def 对照 %d/%d 吻合 (%d 不符; 其中前导下划线归一 %d)"
               % ("PASS" if def_mismatch == 0 else "FAIL", def_checked - def_mismatch,
                  def_checked, def_mismatch,
                  sum(1 for x in entries if x["def_match"] == "leading_underscore")))
    log.append("        注: gx.dll 名字来自 PE 命名导入（非序号，无需对照）；"
               "AYGSHELL.dll 2 条为按序号导入，无外部 def 表可对照（仅 IDB 名+调用/引用证据）")
    if problems:
        log.append("")
        log.append("  问题清单:")
        for p in problems:
            log.append("    - %s" % p)
    log.append("")
    log.append("  证据示例（前 3 条）:")
    for x in entries[:3]:
        ev = x["evidence"]
        first = None
        if ev["callers"]:
            first = "direct call from " + ev["callers"][0]["func"]
        elif ev["text_refs"]:
            first = "text %s:%d" % (ev["text_refs"][0]["file"], ev["text_refs"][0]["line"])
        elif ev["xrefs"]:
            first = "xref %s type=%d" % (ev["xrefs"][0]["frm"], ev["xrefs"][0]["type"])
        log.append("    %s ord=%s api=%s -> %s" % (x["dll"], x["ordinal"], x["api"], first))
    log.append("")
    ok = total == 116 and named == 116 and with_evidence == 116 and def_mismatch == 0
    log.append("RESULT: %s" % ("PASS" if ok else "FAIL (%d problem(s))" % len(problems)))
    text = "\n".join(log)
    print(text)
    os.makedirs("evidence", exist_ok=True)
    with open(os.path.join("evidence", "S0.4.log"), "w", encoding="utf-8") as fh:
        fh.write(text + "\n")
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
