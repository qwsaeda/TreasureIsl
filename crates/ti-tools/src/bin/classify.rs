//! S0.3 函数分类打标 → `ref/classify.json`（S3.12 增强：pending 清零）。
//!
//! 用法：`cargo run -q -p ti-tools --bin classify`
//!
//! 读 `ref/functions.csv` + `ref/imports.json` + `ref/pseudocode/*.c`，
//! 按「名字 → callee API → 字符串 → 高频 callee → 伪代码特征 → 结构兜底」
//! 优先级给 1827 个函数打类目；写 `ref/classify.json` 并打印摘要。
//!
//! S3.12 子步 6：原 pending 兜底类目清零，改判到
//! cpp-method / utility / game-logic / stub / framework 之一
//! （`ref/classify.json` 中 `category=="pending"` 计数 = 0，见 `docs/spec/rng.md` §e）。
#![allow(
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::manual_range_contains,
    clippy::many_single_char_names
)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn repo_root() -> PathBuf {
    let p = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir).join("..").join(".."),
        Err(_) => PathBuf::from("."),
    };
    p.canonicalize().unwrap_or(p)
}

#[derive(Debug, Clone)]
struct Row {
    addr: String,
    size: u32,
    name: String,
    callers: String,
    callees: String,
    str_xrefs: String,
}

/// 逐函数归类结果。
#[derive(Debug, Clone, serde::Serialize)]
struct Entry {
    name: String,
    category: String,
    evidence: Vec<String>,
}

/// 结构兜底判定的输入特征（S3.12 子步 6，pending 清零用）。
#[derive(Debug, Clone, Copy)]
struct Shape {
    size: u32,
    caller_ct: usize,
}

/// imports.json 条目（只取需要的两个可选字段）。
#[derive(serde::Deserialize)]
struct ImportEntry {
    #[serde(default)]
    thunk: Option<String>,
    #[serde(default)]
    api: Option<String>,
}

/// 输出 JSON 形态（`by_category`/`entries` 用 BTreeMap 保证确定性）。
#[derive(serde::Serialize)]
struct OutJson {
    total: usize,
    coverage: f64,
    by_category: BTreeMap<String, usize>,
    hot_sub_callees: Vec<String>,
    entries: BTreeMap<String, Entry>,
}

/// 字符串特征表（与 scripts/classify.py STR_FEATURES 对齐，大小写不敏感）。
const STR_FEATURES: &[(&str, &str)] = &[
    ("need dictionary", "zlib"),
    ("unknown compression method", "zlib"),
    ("invalid window size", "zlib"),
    ("incorrect header check", "zlib"),
    ("incorrect data check", "zlib"),
    ("invalid bit length", "zlib"),
    ("too many length", "zlib"),
    ("invalid stored block", "zlib"),
    ("invalid block type", "zlib"),
    ("invalid distance code", "zlib"),
    ("invalid literal/length", "zlib"),
    ("incomplete dynamic bit", "zlib"),
    ("oversubscribed dynamic", "zlib"),
    ("incomplete literal", "zlib"),
    ("oversubscribed literal", "zlib"),
    ("empty distance tree", "zlib"),
    ("incomplete distance", "zlib"),
    ("oversubscribed distance", "zlib"),
    ("LucencyPackage", "lucency"),
    ("Hekkus Sound", "hmss"),
    ("HMSS", "hmss"),
    (".pcx", "pcx"),
    ("pcx", "pcx"),
    (".map", "game_resource"),
    (".pak", "game_resource"),
    (".lpk", "game_resource"),
    (".fnt", "game_resource"),
    (".sav", "game_resource"),
    (".rec", "game_resource"),
    (".mod", "game_resource"),
    (".wav", "game_resource"),
    (".ini", "game_resource"),
    ("cards", "game_resource"),
    ("goods", "game_resource"),
    ("music", "game_resource"),
    ("glmapm", "game_resource"),
    ("event", "ui_event"),
    ("click", "ui_event"),
    ("press", "ui_event"),
    ("button", "ui_event"),
];

/// API 名表 → 类目的对照表（与 `scripts/classify.py` 的 `API_CATEGORY`/`GX_APIS` 对齐）。
const API_TABLES: &[(&[&str], &str)] = &[
    (CRT_NAMES, "crt"),
    (IO_NAMES, "io"),
    (WIN_CE_NAMES, "win_ce"),
    (GFX_NAMES, "gfx"),
    (AUDIO_NAMES, "audio"),
    (THREAD_NAMES, "thread"),
];

const CRT_NAMES: &[&str] = &[
    "??2@YAPAXI@Z",
    "??3@YAXPAX@Z",
    "memset",
    "memcpy",
    "memmove",
    "realloc",
    "malloc",
    "free",
    "strncmp",
    "strlen",
    "LocalAlloc",
    "LocalFree",
    "LocalReAlloc",
    "LocalSize",
    "XcptFilter",
    "_C_specific_handler",
    "purecall",
    "_rt_sdiv",
    "_rt_udiv",
    "_rt_sdiv64by64",
    "_rt_udiv64by64",
    "_muls",
    "_stoi",
];

const IO_NAMES: &[&str] = &[
    "ReadFile",
    "WriteFile",
    "CreateFileW",
    "CreateFileForMappingW",
    "DeleteFileW",
    "CloseHandle",
    "GetFileSize",
    "SetFilePointer",
    "MapViewOfFile",
    "UnmapViewOfFile",
    "CreateFileMappingW",
];

const WIN_CE_NAMES: &[&str] = &[
    "CreateWindowExW",
    "DestroyWindow",
    "ShowWindow",
    "MoveWindow",
    "UpdateWindow",
    "PeekMessageW",
    "GetMessageW",
    "DispatchMessageW",
    "PostQuitMessage",
    "SetForegroundWindow",
    "FindWindowW",
    "RegisterClassW",
    "UnregisterClassW",
    "DefWindowProcW",
    "CallWindowProcW",
    "SetWindowLongW",
    "GetWindowLongW",
    "IsWindow",
    "GetClientRect",
    "LoadCursorW",
    "GetClassInfoW",
    "wsprintfW",
    "swprintf",
    "itow",
    "itoa",
    "wtol",
    "tolower",
    "SHFullScreen",
    "SHIdleTimerReset",
    "SystemIdleTimerReset",
    "GetTickCount",
    "Sleep",
    "MessageBoxW",
    "GetModuleFileNameW",
    "SetTimer",
    "KillTimer",
    "CeZeroPointer",
    "TerminateProcess",
    "FlushInstructionCache",
    "GetLocalTime",
];

const GFX_NAMES: &[&str] = &[
    "BitBlt",
    "SelectObject",
    "CreateCompatibleDC",
    "GetDC",
    "ReleaseDC",
    "DeleteDC",
    "DeleteObject",
    "CreateDIBSection",
    "GetObjectW",
    "?GXBeginDraw@@YAPAXXZ",
    "?GXEndDraw@@YAHXZ",
    "?GXOpenDisplay@@YAHPAUHWND__@@K@Z",
    "?GXCloseDisplay@@YAHXZ",
    "?GXGetDisplayProperties@@YA?AUGXDisplayProperties@@XZ",
    "?GXOpenInput@@YAHXZ",
    "?GXCloseInput@@YAHXZ",
    "?GXSuspend@@YAHXZ",
    "?GXResume@@YAHXZ",
];

const AUDIO_NAMES: &[&str] = &[
    "waveOutOpen",
    "waveOutClose",
    "waveOutWrite",
    "waveOutPause",
    "waveOutRestart",
    "waveOutReset",
    "waveOutPrepareHeader",
    "waveOutUnprepareHeader",
    "waveOutGetNumDevs",
];

const THREAD_NAMES: &[&str] = &[
    "CreateThread",
    "SuspendThread",
    "ResumeThread",
    "ExitThread",
    "SetThreadPriority",
    "EnterCriticalSection",
    "LeaveCriticalSection",
    "InitializeCriticalSection",
    "DeleteCriticalSection",
    "WaitForSingleObject",
    "CreateEventW",
    "EventModify",
    "HeapDestroy",
];

/// 已知 API 名 → 类目（与 scripts/classify.py 的 API_CATEGORY/GX_APIS 对齐）。
fn api_category(api: &str) -> Option<&'static str> {
    for (names, category) in API_TABLES {
        if names.contains(&api) {
            return Some(category);
        }
    }
    if api == "rand" || api == "srand" {
        return Some("game_logic");
    }
    None
}

/// callee 列 → 去重 API 名集合（imports.json thunk 映射 + 已知 API/GX 名）。
fn callee_apis(callees: &str, thunk_map: &HashMap<String, String>) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for part in callees.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (addr, name) = if let Some(i) = part.find(':') {
            (part[..i].to_uppercase(), part[i + 1..].trim().to_string())
        } else {
            (String::new(), part.to_string())
        };
        if let Some(api) = thunk_map.get(&addr) {
            found.push(api.clone());
        } else if api_category(&name).is_some() {
            found.push(name);
        }
    }
    found.sort();
    found.dedup();
    found
}

/// 字符串特征 → 类目（STR_FEATURES，大小写不敏感）。
fn str_hits(str_xrefs: &str) -> Vec<(String, &'static str)> {
    let mut out: Vec<(String, &'static str)> = Vec::new();
    for part in str_xrefs.split(';') {
        let text = if let Some(i) = part.find(':') {
            part[i + 1..].trim().to_string()
        } else {
            part.trim().to_string()
        };
        if text.is_empty() {
            continue;
        }
        let low = text.to_lowercase();
        for (pat, cat) in STR_FEATURES {
            if low.contains(pat) {
                out.push((text, cat));
                break;
            }
        }
    }
    out
}

/// 伪代码全文特征扫描（vtable ctor / 虚调用 / new-delete / CMemFile）。
fn pc_scan(
    pc_dir: &Path,
    addr: &str,
    cache: &mut HashMap<String, Option<(String, String)>>,
) -> Option<(String, String)> {
    if let Some(hit) = cache.get(addr) {
        return hit.clone();
    }
    let target = addr.to_lowercase();
    let prefix = format!("{target}_");
    let fname = fs::read_dir(pc_dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .map(|d| d.file_name())
        .find(|f| f.to_string_lossy().starts_with(&prefix) && f.to_string_lossy().ends_with(".c"));
    let found = if let Some(fname) = fname {
        let text = fs::read_to_string(pc_dir.join(fname.clone())).unwrap_or_default();
        scan_pc(&text)
    } else {
        None
    };
    cache.insert(addr.to_string(), found.clone());
    found
}

fn scan_pc(text: &str) -> Option<(String, String)> {
    if text.contains("(&off_") || text.contains("=&off_") {
        return Some(("crt_data".to_string(), "pc:vtable_ctor".to_string()));
    }
    if text.contains("__fastcall")
        && (text.contains("(*(void") || text.contains("(*(_DWORD") || text.contains("(*(_BYTE"))
    {
        return Some(("cpp_virt".to_string(), "pc:virt_call".to_string()));
    }
    if text.contains("operator new") {
        return Some(("crt".to_string(), "pc:new_call".to_string()));
    }
    if text.contains("operator delete") {
        return Some(("crt".to_string(), "pc:delete_call".to_string()));
    }
    if text.contains("Free@CMemFile") {
        return Some(("crt".to_string(), "pc:cmemfile".to_string()));
    }
    None
}

/// callee API 判类：强特征按票数判（弱特征 crt/game_logic 单独出现不判类）。
fn classify_apis(apis: &[String]) -> Option<(String, Vec<String>)> {
    if apis.is_empty() {
        return None;
    }
    let mut votes: HashMap<String, usize> = HashMap::new();
    for a in apis {
        let cat = api_category(a).unwrap_or("game_logic").to_string();
        *votes.entry(cat).or_insert(0) += 1;
    }
    let strong: Vec<(String, usize)> = votes
        .iter()
        .filter(|(c, _)| c.as_str() != "crt" && c.as_str() != "game_logic")
        .map(|(c, n)| (c.clone(), *n))
        .collect();
    if !strong.is_empty() {
        let top_votes = strong.iter().map(|(_, n)| *n).max().unwrap_or(0);
        let tied: Vec<String> =
            strong.iter().filter(|(_, n)| *n == top_votes).map(|(c, _)| c.clone()).collect();
        let top = tied.into_iter().min_by_key(|c| priority_rank(c)).unwrap_or_default();
        let shown = apis.iter().take(6).cloned().collect::<Vec<_>>().join(",");
        return Some((top, vec![format!("call:{shown}")]));
    }
    if votes.get("crt").copied().unwrap_or(0) >= 2 {
        let shown = apis.iter().take(6).cloned().collect::<Vec<_>>().join(",");
        return Some(("crt".into(), vec![format!("call:{shown}")]));
    }
    None
}

/// S3.12 (a)-(e) 判类：把原 `pending` 兜底改判到具体类目。
///
/// 优先级（命中即停，见 `docs/spec/rng.md` §e）：
/// (a) 伪代码特征 → `cpp-method`；(b) 被 ≥3 个函数调用 → `utility`；
/// (c) 尺寸 ≥300 → `game-logic`；(d) 尺寸 ≤32 且无 caller → `stub`；
/// (e) 其余 → `framework`（LucencyApp 框架杂项兜底，不再叫 pending）。
fn classify_fallback(shape: Shape, pc: Option<&(String, String)>) -> (String, String) {
    if let Some((_, tag)) = pc {
        if tag.contains("virt") || tag.contains("vtable") {
            return ("cpp-method".into(), format!("pc:{tag}"));
        }
    }
    if shape.caller_ct >= 3 {
        return ("utility".into(), format!("in-degree={}", shape.caller_ct));
    }
    if shape.size >= 300 {
        return ("game_logic".into(), format!("size={}", shape.size));
    }
    if shape.size <= 32 && shape.caller_ct == 0 {
        return ("stub".into(), format!("size={}, unreferenced, small", shape.size));
    }
    ("framework".into(), format!("framework-misc, size={}", shape.size))
}

fn classify_row(
    row: &Row,
    thunk_map: &HashMap<String, String>,
    hot_sub: &HashSet<String>,
    pc_cache: &mut HashMap<String, Option<(String, String)>>,
) -> (String, Vec<String>) {
    let addr = &row.addr;
    let name = &row.name;
    let size = row.size;
    let caller_ct = row.callers.split(';').filter(|s| !s.trim().is_empty()).count();
    let has_callees = !row.callees.trim().is_empty();

    // —— 名字类（O(1)，不读伪代码）——
    if name.starts_with("nullsub") {
        return ("stub".into(), vec![name.clone()]);
    }
    if CRT_NAMES.contains(&name.as_str()) {
        return ("crt".into(), vec![format!("name={name}")]);
    }
    if name.starts_with('?') {
        if let Some(cat) = api_category(name) {
            return (cat.to_string(), vec![format!("name={name} (mangled)")]);
        }
    }

    // —— callee API 类（imports.json 映射）——
    let apis = callee_apis(&row.callees, thunk_map);
    if let Some(cat_ev) = classify_apis(&apis) {
        return cat_ev;
    }

    // —— 字符串类 ——
    let hits = str_hits(&row.str_xrefs);
    if let Some((text, cat)) = hits.first() {
        if *cat != "game_logic" {
            return (
                cat.to_string(),
                vec![format!("str:'{}' [{cat}]", &text[..text.len().min(24)])],
            );
        }
    }

    // —— 高频 sub callee（≥10 callers 的 sub = 基础设施）——
    let mut sub_hits = HashSet::new();
    for part in row.callees.split(';') {
        let n = if let Some(i) = part.find(':') { part[i + 1..].trim() } else { part.trim() };
        if hot_sub.contains(n) {
            sub_hits.insert(n.to_string());
        }
    }
    if !sub_hits.is_empty() {
        let shown = sub_hits.iter().take(4).cloned().collect::<Vec<_>>().join(",");
        return ("utility".into(), vec![format!("hotcallee:{shown}")]);
    }

    // —— 伪代码全文特征 ——
    let pc = pc_scan(&pc_dir_cache(), addr, pc_cache);
    if let Some((cat, tag)) = &pc {
        return (cat.clone(), vec![format!("pc:{tag}")]);
    }

    // —— 结构兜底（S3.12 子步 6：pending 清零，归 (a)-(e) 具体类目）——
    let shape = Shape { size, caller_ct };
    let addr_int = u32::from_str_radix(&row.addr, 16).unwrap_or(0);
    if size <= 16 && caller_ct == 0 && !has_callees {
        return ("stub".into(), vec![format!("size={size}, unreferenced")]);
    }
    if addr_int >= 0x6D000 && addr_int < 0x72000 && caller_ct == 0 && !has_callees {
        return ("crt_data".into(), vec![format!("data-seg ctor (size={size})")]);
    }
    if caller_ct >= 15 {
        return ("game_logic".into(), vec![format!("in-degree={caller_ct}")]);
    }
    if size >= 800 {
        return ("game_logic".into(), vec![format!("size={size}")]);
    }
    if has_callees || caller_ct > 0 {
        return (
            "utility".into(),
            vec![format!("has refs, no distinctive features (size={size})")],
        );
    }
    // 无引用、无 API/串特征：按 S3.12 (d)/(e) 结构判类（不再落 pending）
    let (cat, ev) = classify_fallback(shape, pc.as_ref());
    (cat, vec![ev])
}

fn pc_dir_cache() -> PathBuf {
    repo_root().join("ref").join("pseudocode")
}

fn priority_rank(cat: &str) -> usize {
    const PRIORITY: &[&str] = &[
        "zlib",
        "lucency",
        "hmss",
        "pcx",
        "game_resource",
        "ui_event",
        "gfx",
        "audio",
        "io",
        "thread",
        "win_ce",
        "game_logic",
    ];
    PRIORITY.iter().position(|p| *p == cat).unwrap_or(PRIORITY.len())
}

fn parse_csv(path: &Path) -> Vec<Row> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(err) => {
            eprintln!("classify: cannot read {}: {err}", path.display());
            std::process::exit(1);
        }
    };
    let mut rows = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        if idx == 0 && line.trim_start().to_lowercase().starts_with("addr,") {
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split(',').collect();
        if f.is_empty() || f[0].trim().is_empty() {
            continue;
        }
        let addr = f[0].trim().to_uppercase();
        let size = f.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        let name = f.get(2).map(|s| s.trim().to_string()).unwrap_or_default();
        let callers = f.get(3).map(ToString::to_string).unwrap_or_default();
        let callee_list = f.get(4).map(ToString::to_string).unwrap_or_default();
        let str_refs = f.get(5).map(ToString::to_string).unwrap_or_default();
        rows.push(Row { addr, size, name, callers, callees: callee_list, str_xrefs: str_refs });
    }
    rows
}

fn main() -> ExitCode {
    let root = repo_root();
    let csv_path = root.join("ref").join("functions.csv");
    let imports_path = root.join("ref").join("imports.json");
    let out_path = root.join("ref").join("classify.json");

    let rows = parse_csv(&csv_path);
    if rows.is_empty() {
        eprintln!("FATAL: functions.csv 0 rows");
        return ExitCode::FAILURE;
    }

    // imports.json → {THUNK_UPPER: api}
    let mut thunk_map: HashMap<String, String> = HashMap::new();
    if let Ok(json_text) = fs::read_to_string(&imports_path) {
        if let Ok(data) = serde_json::from_str::<Vec<ImportEntry>>(&json_text) {
            for e in &data {
                if let (Some(t), Some(a)) = (&e.thunk, &e.api) {
                    thunk_map.insert(t.to_uppercase(), a.clone());
                }
            }
        }
    }

    // 高频 sub：callee 出现次数 ≥10
    let mut callee_ct: HashMap<String, usize> = HashMap::new();
    for row in &rows {
        for part in row.callees.split(';') {
            let n = if let Some(i) = part.find(':') { part[i + 1..].trim() } else { part.trim() };
            if n.starts_with("sub_") {
                *callee_ct.entry(n.to_string()).or_insert(0) += 1;
            }
        }
    }
    let hot_sub: HashSet<String> =
        callee_ct.iter().filter(|(_, c)| **c >= 10).map(|(n, _)| n.clone()).collect();

    let mut total = 0usize;
    let mut pending = 0usize;
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
    let mut pc_cache: HashMap<String, Option<(String, String)>> = HashMap::new();
    for row in &rows {
        let (cat, ev) = classify_row(row, &thunk_map, &hot_sub, &mut pc_cache);
        total += 1;
        if cat == "pending" {
            pending += 1;
        }
        *by_category.entry(cat.clone()).or_insert(0) += 1;
        entries.insert(
            row.addr.clone(),
            Entry { name: row.name.clone(), category: cat, evidence: ev },
        );
    }

    let coverage = if total == 0 {
        0.0
    } else {
        f64::from(u32::try_from(total - pending).unwrap_or(u32::MAX))
            / f64::from(u32::try_from(total).unwrap_or(u32::MAX))
            * 100.0
    };
    let mut hot_sorted: Vec<String> = hot_sub.iter().cloned().collect();
    hot_sorted.sort();

    let out = OutJson {
        total,
        coverage: (coverage * 100.0).round() / 100.0,
        by_category: by_category.clone(),
        hot_sub_callees: hot_sorted.clone(),
        entries,
    };
    match serde_json::to_string_pretty(&out) {
        Ok(json_text) => {
            if let Err(err) = fs::write(&out_path, json_text) {
                eprintln!("classify: cannot write {}: {err}", out_path.display());
                return ExitCode::FAILURE;
            }
        }
        Err(err) => {
            eprintln!("classify: serialize failed: {err}");
            return ExitCode::FAILURE;
        }
    }

    println!("=== S0.3 classify (S3.12 enhanced) ===");
    println!("total={total}  coverage={coverage:.2}%  pending={pending}");
    println!("by_category:");
    let mut cats: Vec<(&String, &usize)> = by_category.iter().collect();
    cats.sort_by(|a, b| b.1.cmp(a.1));
    for (cat, n) in cats {
        println!("  {cat:<14} {n:>5}");
    }
    println!("hot_sub_callees={}  output={}", hot_sorted.len(), out_path.display());
    if pending > 0 {
        println!("pending={pending}（S3.12 要求为 0；当前非 0 说明判类规则未覆盖全）");
    }
    ExitCode::SUCCESS
}
