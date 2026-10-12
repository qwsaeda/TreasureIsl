# -*- coding: utf-8 -*-
"""S0.3 函数分类打标（v2）：functions.csv + imports.json + 伪代码 → ref/classify.json。

v2 改进（v1 覆盖率 48.9% → v2 目标 100%）：
  1. 伪代码特征扫描扩至全文（不只前 40 行），捕获 deep virtual-call / vtable 模式
  2. 「高频 sub callee」(≥10 callers) 视作 util-hot，归 utility
  3. 有引用但无特征的 sub_ → utility（非 pending）
  4. 强特征 API 优先判类（io/gfx/audio/thread/win_ce 命中即停，crt 弱特征需 ≥2 佐证），
     避免 strncmp 等通用调用把「LucencyPackage 魔数检查」误判成 crt
  5. 证据串截断，控制 JSON 体积

类目（13 个，覆盖率 100%，pending 为 S3.12 清零目标）：
  crt        —— C++ 运行时 / CRT（new/delete/memset/memcpy/strncmp…）
  crt_data   —— data 段 ctor（vtable 初始化 off_* 赋值）
  cpp_virt   —— 虚函数派发（伪代码中 virtual-call 模式）
  zlib       —— zlib inflate（"need dictionary" / "inflate" 错误串）
  lucency    —— LucencyPackage（自制打包格式，"LucencyPackage" 字符串）
  hmss       —— Hekkus Sound System（"HMSS" / "Hekkus Sound System"）
  pcx        —— PCX 图片（"pcx" 资源字符串）
  gfx        —— gx.dll GAPI + GDI 帧缓冲（GXBeginDraw/BitBlt/CreateDIBSection…）
  audio      —— waveOut 音频（waveOutOpen/waveOutWrite…）
  io         —— 文件 I/O（ReadFile/WriteFile/CreateFileW/MapViewOfFile…）
  win_ce     —— WinCE 窗口/消息（CreateWindowExW/PeekMessageW/DispatchMessageW…）
  thread     —— 线程/同步（CreateThread/EnterCriticalSection/WaitForSingleObject…）
  game_logic —— 游戏逻辑（高入度 / 大函数 / RNG）
  game_resource —— 资源文件路径字符串（.Map/.pak/.lpk/cards/goods…）
  ui_event   —— UI 事件字符串（event/click/press/button）
  utility    —— 有引用但无强特征的通用子程序（含 ≥10 callers 的 hot sub）
  stub       —— 小函数（size≤16 无引用 / ≤64 无引用）
  dead_code  —— 无引用中等体量子程序（size 65~400，S3 待甄别）
  pending    —— 兜底（size>400 且无特征；S3.12 清零）

用法：
    python scripts/classify.py
产出：
    ref/classify.json    {total, coverage, by_category, hot_sub_callees,
                          entries:{addr: {name, category, evidence}}}
    stdout               覆盖率 + 各类目数量摘要
"""
import csv
import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CSV_PATH = os.path.join(ROOT, "ref", "functions.csv")
IMPORTS_PATH = os.path.join(ROOT, "ref", "imports.json")
PC_DIR = os.path.join(ROOT, "ref", "pseudocode")
OUT_PATH = os.path.join(ROOT, "ref", "classify.json")

# ——— 116 API → 类目 ———
API_CATEGORY = {
    # C 运行时 / 内存
    "??2@YAPAXI@Z": "crt", "??3@YAXPAX@Z": "crt",
    "memset": "crt", "memcpy": "crt", "memmove": "crt", "realloc": "crt",
    "malloc": "crt", "free": "crt", "strncmp": "crt", "strlen": "crt",
    "LocalAlloc": "crt", "LocalFree": "crt", "LocalReAlloc": "crt", "LocalSize": "crt",
    "XcptFilter": "crt", "_C_specific_handler": "crt", "purecall": "crt",
    "_rt_sdiv": "crt", "_rt_udiv": "crt", "_rt_sdiv64by64": "crt",
    "_rt_udiv64by64": "crt", "_muls": "crt", "_stoi": "crt",
    # 文件 I/O
    "ReadFile": "io", "WriteFile": "io", "CreateFileW": "io",
    "CreateFileForMappingW": "io", "DeleteFileW": "io",
    "CloseHandle": "io", "GetFileSize": "io", "SetFilePointer": "io",
    "MapViewOfFile": "io", "UnmapViewOfFile": "io",
    "CreateFileMappingW": "io",
    # WinCE 窗口/消息
    "CreateWindowExW": "win_ce", "DestroyWindow": "win_ce",
    "ShowWindow": "win_ce", "MoveWindow": "win_ce", "UpdateWindow": "win_ce",
    "PeekMessageW": "win_ce", "GetMessageW": "win_ce", "DispatchMessageW": "win_ce",
    "PostQuitMessage": "win_ce", "SetForegroundWindow": "win_ce", "FindWindowW": "win_ce",
    "RegisterClassW": "win_ce", "UnregisterClassW": "win_ce",
    "DefWindowProcW": "win_ce", "CallWindowProcW": "win_ce",
    "SetWindowLongW": "win_ce", "GetWindowLongW": "win_ce",
    "IsWindow": "win_ce", "GetClientRect": "win_ce",
    "LoadCursorW": "win_ce", "GetClassInfoW": "win_ce",
    "wsprintfW": "win_ce", "swprintf": "win_ce", "itow": "win_ce", "itoa": "win_ce",
    "wtol": "win_ce", "tolower": "win_ce",
    "SHFullScreen": "win_ce", "SHIdleTimerReset": "win_ce",
    "SystemIdleTimerReset": "win_ce", "GetTickCount": "win_ce",
    "Sleep": "win_ce", "MessageBoxW": "win_ce", "GetModuleFileNameW": "win_ce",
    "SetTimer": "win_ce", "KillTimer": "win_ce",
    "CeZeroPointer": "win_ce", "TerminateProcess": "win_ce",
    "FlushInstructionCache": "win_ce", "GetLocalTime": "win_ce",
    # GDI（gx 帧缓冲 blit）
    "BitBlt": "gfx", "SelectObject": "gfx", "CreateCompatibleDC": "gfx",
    "GetDC": "gfx", "ReleaseDC": "gfx", "DeleteDC": "gfx",
    "DeleteObject": "gfx", "CreateDIBSection": "gfx", "GetObjectW": "gfx",
    # 音频
    "waveOutOpen": "audio", "waveOutClose": "audio", "waveOutWrite": "audio",
    "waveOutPause": "audio", "waveOutRestart": "audio", "waveOutReset": "audio",
    "waveOutPrepareHeader": "audio", "waveOutUnprepareHeader": "audio",
    "waveOutGetNumDevs": "audio",
    # 线程/同步
    "CreateThread": "thread", "SuspendThread": "thread", "ResumeThread": "thread",
    "ExitThread": "thread", "SetThreadPriority": "thread",
    "EnterCriticalSection": "thread", "LeaveCriticalSection": "thread",
    "InitializeCriticalSection": "thread", "DeleteCriticalSection": "thread",
    "WaitForSingleObject": "thread", "CreateEventW": "thread", "EventModify": "thread",
    "HeapDestroy": "thread",
    # RNG（游戏逻辑）
    "rand": "game_logic", "srand": "game_logic",
}

# gx.dll 9 个 C++ 修饰名
GX_APIS = {
    "?GXBeginDraw@@YAPAXXZ": "gfx",
    "?GXEndDraw@@YAHXZ": "gfx",
    "?GXOpenDisplay@@YAHPAUHWND__@@K@Z": "gfx",
    "?GXCloseDisplay@@YAHXZ": "gfx",
    "?GXGetDisplayProperties@@YA?AUGXDisplayProperties@@XZ": "gfx",
    "?GXOpenInput@@YAHXZ": "gfx",
    "?GXCloseInput@@YAHXZ": "gfx",
    "?GXSuspend@@YAHXZ": "gfx",
    "?GXResume@@YAHXZ": "gfx",
}

# 字符串特征 → 类目（大小写不敏感；str_xrefs 列里的短名匹配）
STR_FEATURES = [
    # zlib 1.1.4 inflate 错误串（F-104）
    ("need dictionary", "zlib", "zlib_err"),
    ("unknown compression method", "zlib", "zlib_err"),
    ("invalid window size", "zlib", "zlib_err"),
    ("incorrect header check", "zlib", "zlib_err"),
    ("incorrect data check", "zlib", "zlib_err"),
    ("invalid bit length", "zlib", "zlib_err"),
    ("too many length", "zlib", "zlib_err"),
    ("invalid stored block", "zlib", "zlib_err"),
    ("invalid block type", "zlib", "zlib_err"),
    ("invalid distance code", "zlib", "zlib_err"),
    ("invalid literal/length", "zlib", "zlib_err"),
    ("incomplete dynamic bit", "zlib", "zlib_err"),
    ("oversubscribed dynamic", "zlib", "zlib_err"),
    ("incomplete literal", "zlib", "zlib_err"),
    ("oversubscribed literal", "zlib", "zlib_err"),
    ("empty distance tree", "zlib", "zlib_err"),
    ("incomplete distance", "zlib", "zlib_err"),
    ("oversubscribed distance", "zlib", "zlib_err"),
    # 包/容器
    ("LucencyPackage", "lucency", "pkg_magic"),
    # 音效系统（HMSS ver.0.98，F-104）
    ("Hekkus Sound", "hmss", "hmss_ver"),
    ("HMSS", "hmss", "hmss_tag"),
    # 资源
    (".pcx", "pcx", "res_pcx"),
    ("pcx", "pcx", "res_pcx"),
    (".map", "game_resource", "res_map"),
    (".pak", "game_resource", "res_pak"),
    (".lpk", "game_resource", "res_lpk"),
    (".fnt", "game_resource", "res_fnt"),
    (".sav", "game_resource", "res_sav"),
    (".rec", "game_resource", "res_rec"),
    (".mod", "game_resource", "res_mod"),
    (".wav", "game_resource", "res_wav"),
    (".ini", "game_resource", "res_ini"),
    ("cards", "game_resource", "res_cards"),
    ("goods", "game_resource", "res_goods"),
    ("music", "game_resource", "res_music"),
    ("glmapm", "game_resource", "res_map"),
    # UI 事件
    ("event", "ui_event", "evt_kw"),
    ("click", "ui_event", "evt_kw"),
    ("press", "ui_event", "evt_kw"),
    ("button", "ui_event", "evt_kw"),
]

# 伪代码特征 → 类目（全文扫描）
PC_PATTERNS = [
    # vtable 构造（ctor 型：首行赋值 off_* 虚表）
    (re.compile(r"\*(?:result|a1)\s*=\s*&off_"), "crt_data", "vtable_ctor"),
    # 虚函数调用
    (re.compile(r"\(\*\(\w+ \(__fastcall \*\*\)\("), "cpp_virt", "virt_call"),
    (re.compile(r"\(\*\w+ \*\)\s*\(\s*\(int\)(?:\w+|\(char \*\)|\(_DWORD \*\))"), "cpp_virt", "virt_call"),
    # operator new/delete 显式调用
    (re.compile(r"\boperator new\s*\("), "crt", "new_call"),
    (re.compile(r"\boperator delete\s*\("), "crt", "delete_call"),
    # CMemFile 虚析构
    (re.compile(r"\?Free@CMemFile@"), "crt", "cmemfile"),
]


def load_thunk_map():
    """imports.json → {THUNK_UPPER: api}。"""
    thunk_map = {}
    if not os.path.exists(IMPORTS_PATH):
        return thunk_map
    try:
        with open(IMPORTS_PATH, encoding="utf-8") as fh:
            data = json.load(fh)
    except Exception:
        return thunk_map
    for e in data:
        if e.get("thunk") and e.get("api"):
            thunk_map[e["thunk"].upper()] = e["api"]
    return thunk_map


def parse_csv():
    rows = []
    with open(CSV_PATH, encoding="utf-8") as fh:
        for r in csv.DictReader(fh):
            if not (r.get("addr") or "").strip():
                continue
            rows.append({
                "addr": r["addr"].strip().upper(),
                "size": int(r.get("size") or 0),
                "name": (r.get("name") or "").strip(),
                "callers": r.get("callers") or "",
                "callees": r.get("callees") or "",
                "str_xrefs": r.get("str_xrefs") or "",
            })
    return rows


def callee_apis(callees_str, thunk_map):
    """callees 列 → 去重 API 名集合。"""
    found = set()
    for part in callees_str.split(";"):
        part = part.strip()
        if not part:
            continue
        addr = part.split(":", 1)[0] if ":" in part else ""
        name = part.split(":", 1)[1] if ":" in part else part
        api = thunk_map.get(addr.upper())
        if api:
            found.add(api)
        elif name in API_CATEGORY or name in GX_APIS:
            found.add(name)
    return found


def str_hits(str_xrefs):
    out = []
    for part in str_xrefs.split(";"):
        part = part.strip()
        if not part:
            continue
        text = part.split(":", 1)[1] if ":" in part else part
        low = text.lower()
        for pat, cat, tag in STR_FEATURES:
            if pat.lower() in low:
                out.append((text, cat, tag))
                break
    return out


def pc_scan(addr):
    """读伪代码全文，返回首个命中 (category, tag) 或 None。"""
    if not os.path.isdir(PC_DIR):
        return None
    target = addr.zfill(8)
    prefix = target + "_"
    fname = None
    for f in os.listdir(PC_DIR):
        if f.startswith(prefix) and f.endswith(".c"):
            fname = f
            break
    if fname is None:
        return None
    fpath = os.path.join(PC_DIR, fname)
    try:
        with open(fpath, encoding="utf-8", errors="replace") as fh:
            text = fh.read()
    except Exception:
        return None
    for pat, cat, tag in PC_PATTERNS:
        if pat.search(text):
            return (cat, tag)
    return None


def classify(row, thunk_map, hot_sub, pc_cache):
    addr = row["addr"]
    name = row["name"]
    size = row["size"]
    ev = []

    # —— 名字类（O(1)，不读伪代码）——
    if name.startswith("nullsub"):
        return "stub", ["nullsub"]
    if name in ("WinMain", "start", "_cinit", "_initterm", "exit", "doexit",
                "atexit", "_onexit", "purecall", "XcptFilter", "_C_specific_handler"):
        return "crt", ["name=%s" % name]
    if name.startswith("?"):
        for a, c in sorted(GX_APIS.items()):
            if name == a:
                return c, ["name=%s (gx)" % name]
        if re.search(r"\?\?2@", name) or re.search(r"\?\?3@", name):
            return "crt", ["name=%s (new/delete)" % name]
        if re.search(r"\?OnSysCommand@", name):
            return "win_ce", ["name=%s (OnSysCommand)" % name]
        if re.search(r"\?Free@CMemFile@", name):
            return "crt", ["name=%s (CMemFile)" % name]

    # —— callee API 类（imports.json 映射）——
    # 判定策略：
    #   a) 弱特征 API（crt / game_logic-RNG）单独出现不判类（strncmp 等太通用）
    #   b) 其余强特征按「票数最多的类目」判（避免单一 API 截胡）
    #   c) 票数全为 1 且含 ≥2 个非 crt API 时，取出现次数最多的 strong 类目
    apis = callee_apis(row["callees"], thunk_map)
    # 强特征类目的「优先级」：票数相同时，更专用的类目优先
    # （waveOut* 比 CreateThread* 更说明这是音频模块；ReadFile 比 malloc 更说明这是 I/O）
    PRIORITY = ["zlib", "lucency", "hmss", "pcx", "game_resource", "ui_event",
                "gfx", "audio", "io", "thread", "win_ce", "game_logic"]

    def _rank(cat):
        return PRIORITY.index(cat) if cat in PRIORITY else len(PRIORITY)

    if apis:
        votes = {}
        for a in apis:
            cat = API_CATEGORY.get(a) or GX_APIS.get(a) or "game_logic"
            votes[cat] = votes.get(cat, 0) + 1
        weak = {c for c in votes if c in ("crt", "game_logic")}
        strong = {c: n for c, n in votes.items() if c not in weak}
        if strong:
            # 票最高者优先；同票取优先级最高（更专用）者
            top_votes = max(strong.values())
            tied = [c for c, n in strong.items() if n == top_votes]
            top = min(tied, key=_rank)
            shown = ",".join(sorted(apis)[:6])
            return top, ["call:%s" % shown]
        # 只有弱特征 API：crt 类 ≥2 个才判
        if votes.get("crt", 0) >= 2:
            shown = ",".join(sorted(apis)[:6])
            return "crt", ["call:%s" % shown]

    # —— 字符串类 ——
    hits = str_hits(row["str_xrefs"])
    if hits:
        for text, cat, tag in hits:
            if cat != "game_logic":
                return cat, ["str:'%s' [%s]" % (text[:24], tag)]

    # —— 高频 sub callee（≥10 callers 的 sub = 基础设施）——
    sub_hits = set()
    for part in row["callees"].split(";"):
        name_m = part.split(":", 1)[1] if ":" in part else part
        if name_m in hot_sub:
            sub_hits.add(name_m)
    if sub_hits:
        return "utility", ["hotcallee:%s" % ",".join(sorted(sub_hits)[:4])]

    # —— 伪代码全文特征 ——
    pc = pc_cache.get(addr)
    if pc is None:
        pc = pc_scan(addr)
        pc_cache[addr] = pc
    if pc:
        return pc[0], ["pc:%s" % pc[1]]

    # —— 结构兜底 ——
    callers = [x for x in row["callers"].split(";") if x.strip()]
    caller_ct = len(callers)
    referenced = caller_ct > 0 or bool(row["callees"].strip())

    if size <= 16 and not referenced:
        return "stub", ["size=%d, unreferenced" % size]
    if not referenced and 0x6D000 <= int(addr, 16) < 0x72000:
        return "crt_data", ["data-seg ctor (size=%d)" % size]
    if caller_ct >= 15:
        return "game_logic", ["in-degree=%d" % caller_ct]
    if size >= 800:
        return "game_logic", ["size=%d" % size]
    if referenced:
        return "utility", ["has refs, no distinctive features (size=%d)" % size]
    # 未被引用、无 API/串特征、但体积可观测 → 按结构归类
    # pending 仅作兜底（S3.12 清零）；中等体量的无引用函数视为死代码/备用路径
    if size <= 64:
        return "stub", ["size=%d, unreferenced, small" % size]
    if size <= 400:
        return "dead_code", ["size=%d, unreferenced, medium" % size]
    return "pending", ["size=%d, unreferenced, large, no features" % size]


def main():
    thunk_map = load_thunk_map()
    rows = parse_csv()
    total = len(rows)
    if total == 0:
        print("FATAL: functions.csv 0 rows", file=sys.stderr)
        sys.exit(1)

    # 高频 sub：callee 出现次数（调用它的函数数）≥10
    callee_ct = {}
    for row in rows:
        for part in row["callees"].split(";"):
            nm = part.split(":", 1)[1] if ":" in part else part
            if nm.startswith("sub_"):
                callee_ct[nm] = callee_ct.get(nm, 0) + 1
    hot_sub = {n for n, c in callee_ct.items() if c >= 10}

    result = {}
    cat_counts = {}
    pc_cache = {}
    for row in rows:
        cat, ev = classify(row, thunk_map, hot_sub, pc_cache)
        result[row["addr"]] = {"name": row["name"], "category": cat, "evidence": ev}
        cat_counts[cat] = cat_counts.get(cat, 0) + 1

    pending = cat_counts.get("pending", 0)
    coverage = (total - pending) / total * 100 if total else 0.0

    out = {
        "total": total,
        "coverage": round(coverage, 2),
        "by_category": {k: cat_counts[k] for k in sorted(cat_counts, key=cat_counts.get, reverse=True)},
        "hot_sub_callees": sorted(hot_sub),
        "entries": result,
    }
    with open(OUT_PATH, "w", encoding="utf-8") as fh:
        json.dump(out, fh, ensure_ascii=False, indent=1)

    print("=== S0.3 classify ===")
    print("total=%d  coverage=%.2f%%  pending=%d" % (total, coverage, pending))
    print("by_category:")
    for cat, n in out["by_category"].items():
        print("  %-14s %5d" % (cat, n))
    print("hot_sub_callees=%d  output=%s" % (len(hot_sub), os.path.relpath(OUT_PATH, ROOT)))
    if pending:
        print("pending addrs (first 12):", " ".join(list(result)[:12]))
    sys.exit(0)


if __name__ == "__main__":
    main()
