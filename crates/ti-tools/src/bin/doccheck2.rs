//! `gate.ps1` 第 13 项：文档↔代码 / 文档↔数据 一致性（双 agent 审计固化断言）。
//!
//! 这是第 1～5 轮对抗审计（FIND→AUDIT 逐条 verify）里反复确认、易在后续回归中漂移的
//! 高价值检查，全部固化成可重跑断言，替代「每轮靠子代理现场 grep」：
//!
//! 1. **文档↔代码 API 一致性**：`docs/impl/12_audio.md` 描述的 `ChannelState`
//!    必须与 `crates/ti-platform/src/audio.rs` 的真实 `pub struct ChannelState` 字段集合一致
//!    （`active`/`volume`/`position`/`looping`）。文档里若出现与真实代码冲突的
//!    `AudioChannel`/`enum ChannelState`（Idle/Playing/Paused/Stopped）旧占位描述，报错。
//! 2. **oracle 边界残留**：`crates/ti-app/src/**`（shell 层）不得出现 `oracle_*` 函数调用
//!    （与 `scripts/check_oracle_boundary.py` 同一红线，这里用 Rust 再做一道独立校验，
//!    双保险，防某一侧脚本被误改）。
//! 3. **LCG 乘数 hex 一致性**：`docs/impl/10_rng.md` 与 `docs/impl/06_cards_events.md`
//!    是 LCG hex 权威记录处，若出现误算旧值 `0x3443D`（应为 `0x343FD`）即报错
//!    （第 3～4 轮审计反复踩坑：214013 的 hex 曾被误写 `0x3443D`）。
//! 4. **据点触发阈值一致性**：`docs/` 里凡出现「据点」判定阈值语义的行，
//!    出现 `<= 40`/`40% 命中` 的相反断言即报错（权威 `design/constants.md` §6.6 是
//!    `> 40`，60% 命中分支）。
//! 5. **i18n 孤儿 key**：`crates/ti-app/src/**` 里所有 `sget("…")`/`sgetf("…")` 用到的
//!    key 必须都存在于 `assets/data/strings.json`；反向 `strings.json` 里未被任何
//!    `sget`/`sgetf`/`MenuItem.key` 引用的 key 以 WARN 打印（不阻塞门禁，S8.x 数据化
//!    迁移前清理；`schema_*` 与旧命名空间 key 豁免）。
//! 6. **gate 项数声称一致性**：`scripts/gate.ps1` 头注释/横幅 + `docs/parity.md` /
//!    `docs/git.md` / `docs/README.md` 里所有「gate N 项」声称值，必须 == 实测固定项数
//!    （`gate.ps1` 顶层不缩进 `Report` 调用数 = 20，条件 `Report`/`Skip` 分支不计）。
//!
//! 7. **P3-3 headless bin API 一致性**：`treasure_headless.rs` 对 `BattleView` 的
//!    访问方式必须与 `game_app.rs` 中 `BattleView` 真实定义一致（field vs method）。
//! 8. **飞轮回读（EXP-068）**：开轮必读清单 + 规范回执的文档固化检查：
//!    `05_module_parallel.md` 须含「开轮必读」并引用 `06_flywheel`；
//!    `03_templates.md` §7 任务书模版须含「开轮必读 + 回执」段；
//!    `06_flywheel.md` 须存在且含「开轮必读清单」「飞轮回写」「停机条件」三关键词；
//!    `docs/experience/index.md` 须登记 EXP-068。
//!
//! 9. **战斗玩家数值数据驱动（EXP-066 闭环）**：`game_app.rs` 的 `start_battle`
//!    不得写死 `Unit { hp: 1000, atk: 150 }`，须读 `balance().combat.player`。
//!
//! 10. **飞轮开轮必读脚本（EXP-070）**：`scripts/flywheel_readme.ps1` 存在、
//!     `gate.ps1` 含 `flywheel_readme` 关键词、`06_flywheel.md` 含「开轮必读脚本化」、
//!     `docs/experience/index.md` 含 `EXP-070`。
//!
//! 用法：`cargo run -q -p ti-tools -- doccheck2 [repo_root]`（缺省 `repo_root = "."`）。

use std::collections::BTreeSet;
use std::path::Path;
use std::process::ExitCode;

/// 文档根（相对仓库根），与 doccheck 一致；据点阈值检查只扫 `docs/`。
const DOC_ROOTS: &[&str] = &["docs"];

fn read_file(root: &Path, rel: &str) -> Option<String> {
    std::fs::read_to_string(root.join(rel)).ok()
}

fn collect_lines(root: &Path, roots: &[&str]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for entry in roots {
        collect_recursive(&root.join(entry), &mut out);
    }
    out
}

fn collect_recursive(dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_recursive(&p, out);
        } else if matches!(
            p.extension().and_then(|x| x.to_str()),
            Some("md" | "yaml" | "json" | "rs" | "ps1")
        ) {
            if let Ok(text) = std::fs::read_to_string(&p) {
                out.insert(text);
            }
        }
    }
}

/// 1. 文档↔代码：`impl/12_audio.md` 的 `ChannelState` 必须与 `audio.rs` 真实字段一致。
fn check_audio_state(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(audio_rs) = read_file(root, "crates/ti-platform/src/audio.rs") else {
        return vec!["audio.rs 不存在，无法校验 ChannelState 一致性".to_string()];
    };
    // 从 audio.rs 里抽 `pub struct ChannelState { … }` 的真实字段名
    let Some(struct_start) = audio_rs.find("pub struct ChannelState") else {
        errors.push("audio.rs 缺少 `pub struct ChannelState` 定义".to_string());
        return errors;
    };
    let after = &audio_rs[struct_start..];
    let brace = after.find('{').map_or(after.len(), |i| i);
    let close = after[brace..].find('}').map_or(after.len(), |i| i) + brace;
    let body = &after[..=close];
    // 解析 `pub <field>: <type>` 行（跳过 /// 文档注释行）
    let known_fields: BTreeSet<String> = body
        .lines()
        .filter(|l| l.trim().starts_with("pub ") && l.contains(':'))
        .filter_map(|l| {
            let name = l.trim().strip_prefix("pub ")?.split(':').next()?.trim().to_string();
            (!name.is_empty()).then_some(name)
        })
        .collect();
    if known_fields.is_empty() {
        errors.push("audio.rs 的 ChannelState 未解析出任何 `pub <field>` 字段".to_string());
        return errors;
    }
    // 校验：真实字段集合必须正好是 {active, volume, position, looping}
    let expected: BTreeSet<String> =
        ["active", "volume", "position", "looping"].iter().map(ToString::to_string).collect();
    if known_fields != expected {
        errors.push(format!(
            "audio.rs ChannelState 真实字段 {known_fields:?} ≠ 期望 {expected:?}（文档/代码已漂移）"
        ));
    }
    // 校验：`impl/12_audio.md` 里若出现 `pub enum ChannelState`，必须是带「设计占位」
    // 标注的（文档 §2.2 已有 `> 实际 ti-platform::audio … 为 struct 非 enum` 说明块）。
    // 豁免条件：该 enum 块所在上下文 50 行内有「设计占位」字样。
    let Some(doc12) = read_file(root, "docs/impl/12_audio.md") else {
        return errors;
    };
    let lines: Vec<&str> = doc12.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with("pub enum ChannelState") {
            continue;
        }
        let window_start = idx.saturating_sub(49);
        let in_design_zone = lines[window_start..=idx].iter().any(|l| l.contains("设计占位"));
        if !in_design_zone {
            errors.push(format!(
                "impl/12_audio.md:{} 描述 `pub enum ChannelState` 但无「设计占位」标注（真实 audio.rs 是 struct，非 enum）",
                idx + 1
            ));
        }
    }
    errors
}

/// 2. shell 层（`crates/ti-app/src`）不得出现 `oracle_*` 函数调用（oracle 仅 rules 内部参照物）。
fn check_shell_oracle(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let base = root.join("crates/ti-app/src");
    if !base.is_dir() {
        return errors;
    }
    for text in read_rs_tree(&base) {
        for (i, line) in text.lines().enumerate() {
            let Some(pos) = line.find("oracle_") else { continue };
            let after = &line[pos + "oracle_".len()..];
            let name_end = after
                .char_indices()
                .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
                .map_or(after.len(), |(b, _)| b);
            let fname = &after[..name_end];
            let is_call = after[name_end..].trim_start().starts_with('(');
            if !fname.is_empty() && is_call {
                errors.push(format!(
                    "ti-app/src shell 层第 {i} 行出现 oracle_{fname} 函数调用（oracle 仅 rules 内部参照物，shell 层禁用；见 S8.8 oracle 边界）"
                ));
            }
        }
    }
    errors
}

fn read_rs_tree(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return out };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            out.extend(read_rs_tree(&p));
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            if let Ok(t) = std::fs::read_to_string(&p) {
                out.push(t);
            }
        }
    }
    out
}

/// 3. LCG 乘数 hex 误算检查：`impl/10_rng.md` 与 `impl/06_cards_events.md` 是 LCG hex
///    权威记录处，若出现误算旧值 `0x3443D`（应为 `0x343FD`）即报错。
fn check_lcg_hex(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    for rel in ["docs/impl/10_rng.md", "docs/impl/06_cards_events.md"] {
        let Some(text) = read_file(root, rel) else { continue };
        for (i, line) in text.lines().enumerate() {
            if line.contains("0x3443D") {
                errors.push(format!(
                    "{rel}:{} 出现 LCG 乘数误算 hex 0x3443D（应为 0x343FD）：{}",
                    i + 1,
                    line.trim()
                ));
            }
        }
    }
    errors
}

/// 4. 据点触发阈值：`docs/` 里凡出现「据点」判定阈值语义的行，
///    出现 `<= 40` 或 `40% 命中` 的相反断言即报错（权威 `design/constants.md` §6.6 是
///    `> 40`，60% 命中分支，其余 40% 走区域匹配失败路径）。
fn check_landmark_threshold(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    // 豁免：docs/parity.md 里描述 doccheck2 检查项本身的行（自引用，非业务规则误用）
    let parity = read_file(root, "docs/parity.md").unwrap_or_default();
    let parity_exempt_lines: BTreeSet<String> = parity
        .lines()
        .filter(|l| l.contains("doccheck2") || l.contains("反向断言"))
        .map(|l| l.trim().to_string())
        .collect();
    for text in &collect_lines(root, DOC_ROOTS) {
        for (i, line) in text.lines().enumerate() {
            if !line.contains("据点") {
                continue;
            }
            // 豁免：该行是描述 doccheck2 检查项本身的（自引用）
            if parity_exempt_lines.contains(line.trim()) {
                continue;
            }
            // 阈值判定：不得写成 `<= 40` 命中（权威是 `> 40`）
            if line.contains("<= 40") || line.contains("≤ 40") {
                errors.push(format!(
                    "docs 第 {i} 行：据点阈值误用 `<= 40`（权威 design/constants.md §6.6 是 `> 40`，60% 命中）：{}",
                    line.trim()
                ));
            }
            // 命中比例误标：60% 命中分支不得写成 40% 命中（「40% 走」「其余 40%」是正确表述，豁免）
            let wrong_pct = line.contains("40%")
                && (line.contains("命中") || line.contains("据点"))
                && !line.contains("40% 走")
                && !line.contains("其余 40%");
            if wrong_pct {
                errors.push(format!(
                    "docs 第 {i} 行：据点命中比例误标 40%（权威值：60% 命中分支，其余 40% 走区域匹配失败）：{}",
                    line.trim()
                ));
            }
        }
    }
    errors
}

/// 5. i18n 一致性：代码里 `sget`/`sgetf`/`MenuItem.key` 用的 key 必须都存在于
///    `strings.json`（缺失 key 会让界面显示裸 key）；反向 `strings.json` 里未被引用
///    的 key 以 WARN 打印（不阻塞门禁，S8.x 数据化迁移前清理；`schema_*` 与旧命名空间豁免）。
fn check_i18n_orphans(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(strings_json) = read_file(root, "assets/data/strings.json") else {
        return vec!["assets/data/strings.json 不存在".to_string()];
    };
    // 解析 `strings.json` 的顶层 key（形如 `  "key": "value"`）
    let json_keys: BTreeSet<String> = strings_json
        .lines()
        .filter_map(|l| {
            let rest = l.trim().strip_prefix('"')?;
            let (k, _) = rest.split_once('"')?;
            Some(k.to_string())
        })
        .collect();

    // 收集代码里 `sget(`/`sgetf(`/`MenuItem.key` 的字面量 key
    let mut used_keys: BTreeSet<String> = BTreeSet::new();
    for text in read_rs_tree(&root.join("crates/ti-app/src")) {
        for line in text.lines() {
            for call in ["sget(", "sgetf("] {
                let Some(start) = line.find(call) else { continue };
                let after = &line[start + call.len()..];
                let Some(q) = after.find('"') else { continue };
                let inner = &after[q + 1..];
                let Some(end) = inner.find('"') else { continue };
                used_keys.insert(inner[..end].to_string());
            }
            // `MenuItem { key: "…", … }` const 表里直接写死的 i18n key
            if line.contains("key:") {
                let rest = &line[line.find("key:").unwrap_or(0)..];
                let Some(q) = rest.find('"') else { continue };
                let inner = &rest[q + 1..];
                let Some(end) = inner.find('"') else { continue };
                used_keys.insert(inner[..end].to_string());
            }
        }
    }
    // 代码用了但 JSON 没有的 key → 缺失（界面会显示裸 key，可见异常）—— 失败级
    for k in &used_keys {
        if !json_keys.contains(k) {
            errors.push(format!(
                "i18n key `{k}` 被 sget/sgetf 使用但不存在于 strings.json（界面将显示裸 key）"
            ));
        }
    }
    // JSON 里有但代码从未引用的 key → 孤儿（文案表漂移）。
    // 豁免：`schema_*` / 旧命名空间（battle/build/cards/goods/hint/hud/slot/msg）/
    // `scene.*` 下已知被 `*_fmt` 长 key 取代的短后缀变体（设计占位，S8.x 数据化迁移前清理）
    let exempt_prefixes =
        ["schema_", "battle.", "build.", "cards.", "goods.", "hint.", "hud.", "slot.", "msg."];
    let exempt_suffixed = [
        ".hint",
        ".empty",
        ".has",
        ".slot",
        ".slot_rwx",
        ".page",
        ".page_hint",
        ".boss_hp",
        ".player_hp",
        ".rounds",
    ];
    for k in &json_keys {
        if exempt_prefixes.iter().any(|p| k.starts_with(p)) {
            continue;
        }
        let is_suffixed_variant =
            k.starts_with("scene.") && exempt_suffixed.iter().any(|s| k.ends_with(s));
        if !used_keys.contains(k) && !is_suffixed_variant {
            eprintln!(
                "  WARN i18n 孤儿 key `{k}`（strings.json 有但代码未引用，S8.x 数据化迁移前清理）"
            );
        }
    }
    errors
}

/// 6. **gate 项数声称一致性**：`scripts/gate.ps1` 头注释/横幅 + `docs/parity.md` /
///    `docs/git.md` / `docs/README.md` 里所有「gate N 项」类声称值，必须 == 实测固定项数
///    （`gate.ps1` 顶层不缩进 `Report` 调用数；条件 `Report`/`Skip` 分支不计）。
///    豁免项编号（「第 N 项」）/ doccheck2 检查数声称（「N 项检查」）/ playbook 命令数
///    声称（「N 项命令」），上下文区分「gate 项数」vs「doccheck2 项数」。
fn gate_item_actual_count(gate: &str) -> usize {
    // 固定项 = 顶层（不缩进）`Report` 调用数；条件分支（if 块内缩进的
    // `Report`/`Skip`）不计。实测 gate.ps1 = 22 固定 + 3 条件 Skip。
    gate.lines().filter(|l| l.starts_with("Report ")).count()
}

/// 抽「紧邻 `项` 前的数字 + 数字左边是非数字字符（行首/空白/全角括号等）」的声称值。
/// 豁免：「第 N 项」项编号（数字前是「第」）/ 行含「项检查」（doccheck2 检查数声称）/
/// 行含「项命令」（playbook 命令数声称）。
fn extract_count_claim(line: &str, keyword: char) -> Option<usize> {
    // char 边界安全扫描：「项」是 3 字节 UTF-8，byte find 结果不能直接当 char 偏移用。
    let mut positions: Vec<usize> = Vec::new();
    for (ci, ch) in line.char_indices() {
        if ch == keyword {
            positions.push(ci);
        }
    }
    let bytes = line.as_bytes();
    for &pos in &positions {
        // 数字串（紧邻 `项` 左边，中间只允许空白）
        let before = &bytes[..pos];
        let mut i = before.len();
        while i > 0 && (before[i - 1] == b' ' || before[i - 1] == b'\t') {
            i -= 1;
        }
        let mut j = i;
        while j > 0 && before[j - 1].is_ascii_digit() {
            j -= 1;
        }
        let digit_run_len = i - j;
        if digit_run_len == 0 {
            continue;
        }
        let digit_start = j;
        let left_ok = digit_start == 0 || !bytes[digit_start - 1].is_ascii_digit();
        if !left_ok {
            continue;
        }
        // 豁免项编号/非 gate 项数声称（「第 N 项」里 N 前是「第」→ 项编号非项数）
        let is_item_number = line[..digit_start].chars().any(|c| c == '第');
        let is_check_count = line.contains("项检查");
        let is_cmd_count = line.contains("项命令");
        if is_item_number || is_check_count || is_cmd_count {
            continue;
        }
        let digits: String = bytes[digit_start..i].iter().map(|&b| b as char).collect();
        return digits.parse::<usize>().ok();
    }
    None
}

fn check_gate_item_count(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(gate) = read_file(root, "scripts/gate.ps1") else {
        return vec!["scripts/gate.ps1 不存在".to_string()];
    };
    let actual = gate_item_actual_count(&gate);
    let expected = 22usize;
    if actual != expected {
        errors.push(format!(
            "gate.ps1 实测固定项 {actual}，期望 {expected}（门禁结构变化时同步更新 doccheck2 阈值）"
        ));
    }
    // 声称点：gate.ps1 头注释 + 横幅（`N 项` 由 extract_count_claim 提取，`N items` 由
    // extract_items_claim 提取；同一行同时命中两者时按 (N 项) 优先去重，避免双报）
    let mut claimed: Vec<(String, usize)> = Vec::new();
    for (i, line) in gate.lines().enumerate() {
        let trimmed = line.trim_start();
        if !(trimmed.starts_with('#') || trimmed.starts_with("Write-Host")) {
            continue;
        }
        let v_items = extract_items_claim(line);
        let v_xiang = extract_count_claim(line, '项');
        if let Some(v) = v_xiang {
            claimed.push((format!("gate.ps1:{} {}", i + 1, line.trim()), v));
        } else if let Some(v) = v_items {
            claimed.push((format!("gate.ps1:{} {}", i + 1, line.trim()), v));
        }
    }
    // docs 里的「gate N 项」/「gate.ps1 N 项」声称（含 doccheck2 上下文豁免）
    for rel in ["docs/parity.md", "docs/git.md", "docs/README.md"] {
        let Some(text) = read_file(root, rel) else { continue };
        for (i, line) in text.lines().enumerate() {
            if line.contains("doccheck2") {
                continue; // doccheck2 项数声称，非 gate 项数
            }
            if !line.contains("gate") {
                continue;
            }
            if let Some(v) = extract_count_claim(line, '项') {
                claimed.push((format!("{rel}:{} {}", i + 1, line.trim()), v));
            }
        }
    }
    for (where_, declared) in &claimed {
        if *declared != actual {
            errors.push(format!(
                "{where_} 声称 gate 项数 {declared}，实测 {actual}（FIND-304-B：门禁项数声称漂移）"
            ));
        }
    }
    errors
}

/// 仅匹配 `N items`（横幅声称，`items` 前是 `N` 或 `N ` 形式，N 左边非数字）
fn extract_items_claim(line: &str) -> Option<usize> {
    let pos = line.find(" items")?;
    let bytes = line.as_bytes();
    let before = &bytes[..pos];
    let mut i = before.len();
    while i > 0 && (before[i - 1] == b' ' || before[i - 1] == b'\t') {
        i -= 1;
    }
    let mut j = i;
    while j > 0 && before[j - 1].is_ascii_digit() {
        j -= 1;
    }
    if i == j {
        return None;
    }
    let left_ok = j == 0 || !bytes[j - 1].is_ascii_digit();
    if !left_ok {
        return None;
    }
    let digits: String = bytes[j..i].iter().map(|&b| b as char).collect();
    digits.parse::<usize>().ok()
}

/// 7. P3-3 headless bin API 一致性：`treasure_headless.rs` 对 `BattleView` 的
///    访问方式必须与 `game_app.rs` 中 `BattleView` 真实定义一致（field vs method）。
///    若 `BattleView` 里某成员是 pub field，headless bin 里不得调用同名 method；
///    反之亦然（防 P3-3 因 API 漂移编译失败，S10.5 第 6 轮审计发现）。
fn check_headless_battle_api(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let headless = read_file(root, "crates/ti-app/src/bin/treasure_headless.rs");
    let game_app = read_file(root, "crates/ti-app/src/game_app.rs");
    let (Some(headless), Some(game_app)) = (headless, game_app) else {
        return vec![
            "treasure_headless.rs 或 game_app.rs 不存在，无法校验 BattleView API 一致性"
                .to_string(),
        ];
    };
    // 从 game_app.rs 抽 `pub struct BattleView` 的真实字段名集合
    let Some(struct_start) = game_app.find("pub struct BattleView") else {
        return vec!["game_app.rs 缺少 `pub struct BattleView` 定义".to_string()];
    };
    let after = &game_app[struct_start..];
    let brace = after.find('{').map_or(after.len(), |i| i);
    let close = after[brace..].find('}').map_or(after.len(), |i| i) + brace;
    let body = &after[..=close];
    let fields: BTreeSet<String> = body
        .lines()
        .filter(|l| l.trim().starts_with("pub ") && l.contains(':'))
        .filter_map(|l| {
            let name = l.trim().strip_prefix("pub ")?.split(':').next()?.trim().to_string();
            (!name.is_empty()).then_some(name)
        })
        .collect();
    if fields.is_empty() {
        return vec!["game_app.rs BattleView 未解析出任何 `pub <field>` 字段".to_string()];
    }
    // 抽 `impl BattleView` 里定义的 method 名集合（这些字段同时有 accessor method，
    // 方法调用合法；只有「纯 field 无 method」的成员，方法调用才是错的）
    let methods: BTreeSet<String> = game_app
        .lines()
        .filter(|l| l.trim().starts_with("pub fn "))
        .filter_map(|l| {
            let name = l.trim().strip_prefix("pub fn ")?.split('(').next()?.trim().to_string();
            (!name.is_empty()).then_some(name)
        })
        .collect();
    // 只对「纯 pub field 且无同名 method」的成员检查方法调用（编译会失败）
    let field_only: Vec<&String> = fields.iter().filter(|f| !methods.contains(*f)).collect();
    // 检查 headless bin 里对这些纯 field 成员的方法调用 `b.<name>()`：
    // 若 <name> 是纯 pub field（无同名 method），则不得写成 method 调用（编译会失败）
    for (i, line) in headless.lines().enumerate() {
        for field in &field_only {
            let pattern = format!(".{field}()");
            if line.contains(&pattern) {
                errors.push(format!(
                    "treasure_headless.rs:{} 调用 `.{field}()` 但 BattleView.{field} 是纯 pub field（无同名 method），应写字段访问 `.{field}` 而非方法调用",
                    i + 1
                ));
            }
        }
    }
    errors
}

/// 8. 飞轮回读（EXP-068）：开轮必读清单 + 规范回执的文档固化检查。
///    - `05_module_parallel.md` 须含「开轮必读」关键词 + 引用 `06_flywheel`；
///    - `03_templates.md` §7 任务书模版须含「开轮必读 + 回执」段；
///    - `06_flywheel.md` 须存在且含「开轮必读清单」「飞轮回写」「停机条件」；
///    - `docs/experience/index.md` 须登记 EXP-068。
fn check_flywheel_readback(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let parallel = read_file(root, "docs/process/05_module_parallel.md").unwrap_or_default();
    if !parallel.contains("开轮必读") {
        errors.push(
            "05_module_parallel.md 缺少「开轮必读」关键词（EXP-068 飞轮增强，见 06_flywheel §开轮必读）"
                .to_string(),
        );
    }
    if !parallel.contains("06_flywheel") {
        errors.push("05_module_parallel.md 未引用 06_flywheel（开轮必读单一事实源）".to_string());
    }
    let templates = read_file(root, "docs/process/03_templates.md").unwrap_or_default();
    if !templates.contains("开轮必读") || !templates.contains("规范回执") {
        errors.push(
            "03_templates.md §7 任务书模版缺少「开轮必读 + 回执」段（EXP-068 飞轮增强）"
                .to_string(),
        );
    }
    match read_file(root, "docs/process/06_flywheel.md") {
        None => {
            errors
                .push("docs/process/06_flywheel.md 不存在（飞轮单一事实源，EXP-068）".to_string());
        }
        Some(flywheel) => {
            for kw in ["开轮必读清单", "飞轮回写", "停机条件"] {
                if !flywheel.contains(kw) {
                    errors.push(format!("06_flywheel.md 缺少关键词「{kw}」（EXP-068 飞轮增强）"));
                }
            }
        }
    }
    let exp_index = read_file(root, "docs/experience/index.md").unwrap_or_default();
    if !exp_index.contains("EXP-068") {
        errors.push("docs/experience/index.md 未登记 EXP-068（飞轮回读）".to_string());
    }
    errors
}

/// 9. 战斗玩家数值数据驱动（EXP-066 闭环）：`game_app.rs` 的 `start_battle`
///    不得写死 `Unit { hp: 1000, atk: 150 }`，须读 `balance().combat.player`。
///    机械判据：`game_app.rs` 里 `start_battle` 函数体须含 `balance().combat.player`
///    或 `combat.player` 引用（P0-1 修复后闭环验证，防再次回归）。
fn check_battle_player_data_driven(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(game_app) = read_file(root, "crates/ti-app/src/game_app.rs") else {
        return vec!["game_app.rs 不存在，无法校验战斗玩家数值数据驱动".to_string()];
    };
    let Some(sb_start) = game_app.find("fn start_battle") else {
        return vec!["game_app.rs 缺少 `fn start_battle`".to_string()];
    };
    // 取 start_battle 函数体（到下一个 `}` 闭合为止，简化取 800 字符窗口，按 char 边界截取；
    // 800 足以覆盖到 `b.combat.player` 引用行（FIND-013 改后引用在函数体中部）
    let raw_end = (sb_start + 800).min(game_app.len());
    // 回退到合法 char 边界（floor_char_boundary 需 Rust 1.91，MSRV 1.85 故手动回退）
    let mut end = raw_end;
    while end > sb_start && !game_app.is_char_boundary(end) {
        end -= 1;
    }
    let body = &game_app[sb_start..end];
    if !body.contains("combat.player") {
        errors.push(
            "game_app.rs start_battle 未读 balance.combat.player（EXP-066 未闭环，战斗玩家数值写死）"
                .to_string(),
        );
    }
    errors
}

/// 10. 飞轮开轮必读脚本（EXP-070）：`scripts/flywheel_readme.ps1` 存在、
///     `gate.ps1` 含 `flywheel_readme` 关键词、`06_flywheel.md` 含「开轮必读脚本化」、
///     `docs/experience/index.md` 含 `EXP-070`。
fn check_flywheel_script(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let script_path = root.join("scripts/flywheel_readme.ps1");
    if !script_path.exists() {
        errors.push("scripts/flywheel_readme.ps1 不存在（EXP-070 开轮必读脚本化）".to_string());
    }
    let gate = read_file(root, "scripts/gate.ps1").unwrap_or_default();
    if !gate.contains("flywheel_readme") {
        errors
            .push("gate.ps1 未含 flywheel_readme 关键词（EXP-070 gate 第 17 项缺失）".to_string());
    }
    let flywheel = read_file(root, "docs/process/06_flywheel.md").unwrap_or_default();
    if !flywheel.contains("开轮必读脚本化") {
        errors.push("06_flywheel.md 缺少「开轮必读脚本化」节（EXP-070）".to_string());
    }
    let exp_index = read_file(root, "docs/experience/index.md").unwrap_or_default();
    if !exp_index.contains("EXP-070") {
        errors.push("docs/experience/index.md 未登记 EXP-070（开轮必读脚本化）".to_string());
    }
    errors
}

/// 11. 触摸/渲染正逆成对（EXP-071）：列表场景的 `on_touch` 必须用
///     `scene_list_bands` / `slot_bands`（从 `ListLayout` 推导），
///     不得残留旧的 `list_index_at` / `slot_at` 调用。
fn check_touch_render_pair(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    for rel in [
        "crates/ti-app/src/scenes/select_role.rs",
        "crates/ti-app/src/scenes/select_map.rs",
        "crates/ti-app/src/scenes/saveload.rs",
    ] {
        let src = read_file(root, rel).unwrap_or_default();
        let has_old_api = src.contains("list_index_at") && !src.contains("scene_list_index_at")
            || src.contains("slot_at(") && !src.contains("slot_band_index_at");
        if has_old_api {
            errors.push(format!("{rel} 仍残留旧触摸 API list_index_at/slot_at（EXP-071 应改用 scene_list_bands/slot_bands）"));
        }
        let has_bands = src.contains("scene_list_bands") || src.contains("slot_bands");
        if !has_bands {
            errors
                .push(format!("{rel} 未使用 scene_list_bands/slot_bands（EXP-071 正逆成对缺失）"));
        }
    }
    let touch_src = read_file(root, "crates/ti-app/src/touch.rs").unwrap_or_default();
    if touch_src.contains("LIST_Y0") || touch_src.contains("SLOT_Y0") {
        errors.push("touch.rs 仍残留旧常量 LIST_Y0/SLOT_Y0（EXP-071 应已移除）".to_string());
    }
    // S15.6 QA025：主地图触摸↔渲染正逆成对锚点（EXP-067）——触摸逆变换须读
    // scenes.json `grid.*`（与渲染 `chrome::iso_cell` 同数据源，无 legacy 32/16 + camera 分裂）。
    if !touch_src.contains("grid.x0") {
        errors.push(
            "touch.rs 等距逆变换未读 scenes.json `grid.*`（QA022/QA023：触摸与渲染须共用 grid 数据源，EXP-067 正逆成对）"
                .to_string(),
        );
    }
    let main_map_src = read_file(root, "crates/ti-app/src/scenes/main_map.rs").unwrap_or_default();
    if !main_map_src.contains("grid_cell_at") {
        errors.push(
            "main_map.rs 触摸命中未走 scenes.json 派生的 grid_cell_at（QA025：主地图触摸↔渲染正逆成对锚点缺失）"
                .to_string(),
        );
    }
    errors
}

/// 13. HD 精灵透明门禁（EXP-075 洋红框）。S15.4 重锚：legacy 逐像素 CPU blit 层
///     （index 0 跳过 + RGBA alpha 阈值）已随 legacy 像素写帧层移除，运行时只剩代码
///     绘制 + GPU 纹理路。S16 渲染线再收敛：`render_pipeline.rs` 整文件删除（死码，
///     生产零消费），PCX index 0 哨兵透明位的锚点转到 `pack_hd_assets.py` 侧
///     `sanitize_alpha`（alpha 二值化）+ 运行时无 CPU 逐像素 RGBA blit 两条不变量。
fn check_hd_sprite_transparency(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    // ① 死码不得复活：`render_pipeline.rs` 已在 S16 渲染线删除（契约 1/2 收口），
    //    若重新出现则说明 legacy 坐标裁剪层被带回运行时。
    if root.join("crates/ti-app/src/render_pipeline.rs").exists() {
        errors.push(
            "render_pipeline.rs 必须保持删除（S16 渲染线：legacy 越界裁剪层生产零消费，F18 死码清理；契约 1/2 收口）"
                .to_string(),
        );
    }

    // ② HD 资产管线 alpha 二值化（EXP-075②：Lanczos 重采样软 alpha 1..254 → 洋红框）。
    //    `blit_scene_rgba` 已删（S15.4），但 HD PNG 注入仍走 `load_hd_pngs`/GPU 路，
    //    资产侧二值化约定不变。
    let packer = read_file(root, "scripts/pack_hd_assets.py").unwrap_or_default();
    if !packer.contains("def sanitize_alpha") || !packer.contains("a >= 128") {
        errors.push(
            "scripts/pack_hd_assets.py 必须含 `sanitize_alpha`（alpha 二值化 + 透明区 RGB 清零），否则 Lanczos 缩采样产生软 alpha（EXP-075②）"
                .to_string(),
        );
    }
    if !packer.contains("img1 = sanitize_alpha(img1)") {
        errors.push(
            "pack_hd_assets.py 1x 档必须对缩采样结果再 `sanitize_alpha`（EXP-075②：插值仍可能产生 1..254 软 alpha）"
                .to_string(),
        );
    }

    errors
}

/// S14 全量重设计：scenes.json 坐标以 `docs/business/08_layout_design.md` §4 为单一事实源。
/// 旧 EXP-074 锚定（=ab0e5b5 逆向坐标）已作废——S14 重设计是正向开发的新版面，
/// 不再要求 == 旧逆向视觉，改锚定 `08_layout_design.md` §4 的新坐标。
#[allow(clippy::too_many_lines)] // S15.6 QA133 实体 size 键在位锚并入后 > 100 行
fn check_scene_layout_fidelity(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(json) = read_file(root, "assets/data/scenes.json") else {
        return vec!["assets/data/scenes.json 不存在，无法校验布局（S14）".to_string()];
    };

    // 逐场景锚定 **1080 现代版面**（`08_layout_design.md` §4 / `scenes.json`，S16 渲染线；
    // S17.2 起锚点 = 航海夜·精修 v2 8pt 值，`docs/process/10_ui_redesign.md` §4 增量）：
    // 公共：标题条 h=96、提示/状态行 y=1840、列表 x=96 w=896
    // index: list y=392 row_h=224 item_h=168 highlight_offset=40，status y=1840
    // select_role: row_h=304；select_map: row_h=336；saveload: row_h=304 gold_x=816
    // vs: player_hp 64..512 / boss_hp 576..1024 y=168 w=448 h=48
    // main_map: hud y=152 h=128（不与等距区 grid.y0=520 重叠）
    let anchors: &[(&str, &str, &str)] = &[
        ("\"index\"", "\"h\": 96", "index 标题条高度 = 96（§4.1，S17.2 8pt）"),
        ("\"index\"", "\"y\": 392", "index 列表基线 y = 392（§4.1，S17.2 8pt）"),
        ("\"index\"", "\"row_h\": 224", "index 行高 = 224（§4.1，S17.2 8pt）"),
        ("\"index\"", "\"highlight_offset\": 40", "index 高亮上移 = 40（§4.1，S17.2 8pt）"),
        ("\"index\"", "\"y\": 1840", "index 状态行 y = 1840（§4.1，S17.2 8pt）"),
        (
            "\"select_role\"",
            "\"row_h\": 304",
            "select_role 行高 = 304（§4.2，5 角色卡，S17.2 8pt）",
        ),
        ("\"select_map\"", "\"row_h\": 336", "select_map 行高 = 336（§4.2，3 地图卡，S17.2 8pt）"),
        ("\"saveload\"", "\"row_h\": 304", "saveload 行高 = 304（§4.6，S17.2 8pt）"),
        (
            "\"saveload\"",
            "\"gold_x\": 816",
            "saveload 金字号列 x = 816（§4.6，S17.2 8pt，QA105/QB122 左移防越屏）",
        ),
        (
            "\"vs\"",
            "\"w\": 448",
            "vs 血条宽 = 448（§4.4，S17.2 8pt，QB120 解重叠 64..512/576..1024）",
        ),
        ("\"vs\"", "\"h\": 48", "vs 血条高 = 48（§4.4，8pt）"),
        (
            "\"main_map\"",
            "\"y\": 152",
            "main_map HUD y = 152（§4.3，S17.2 8pt，与等距区 520 不重叠）",
        ),
        ("\"main_map\"", "\"h\": 128", "main_map HUD h = 128（§4.3，S17.2 8pt）"),
        (
            "\"main_map\"",
            "\"y0\": 520",
            "main_map 等距棋盘 grid.y0 = 520（§4.3，业务几何非 8pt 版面令牌）",
        ),
        ("\"message\"", "\"h\": 1480", "message 正文区 h = 1480（§4.5，FIND-026 区间约束）"),
    ];
    // 每个场景段单独切片，避免跨场景误匹配（如 select_role 与 select_map 同名 list）
    for &(scene_key, needle, msg) in anchors {
        let scene_block = scene_section(&json, scene_key);
        if !scene_block.contains(needle) {
            errors.push(format!("scenes.json {scene_key}: {msg}（须含 `{needle}`）"));
        }
    }

    // 标量字段语义：未解锁标签 x。QA033 更新：1080 版面数据驱动键 `locked.text_x` = 920
    // （scenes.json `select_map.locked` 块），legacy 176 时代休眠回退值 130 已删
    // （键缺失/非正 = 不画锁块，契约 1/4）。锚定**数据侧键存在 + 代码侧读取、
    // 且无 130 回退**。
    let select_map_rs =
        read_file(root, "crates/ti-app/src/scenes/select_map.rs").unwrap_or_default();
    let select_map_json = read_file(root, "assets/data/scenes.json").unwrap_or_default();
    if !select_map_rs.contains("locked.text_x") {
        errors.push(
            "select_map.rs 必须读 `extra_scalar[\"locked.text_x\"]`（QA033：§4.2 未解锁标签 x，1080 数据驱动，契约 4）"
                .to_string(),
        );
    }
    if select_map_rs.contains("unwrap_or(130)") {
        errors.push(
            "select_map.rs 不得有 legacy 176 时代回退值 130（QA033：130 = 176 坐标系值，契约 1）"
                .to_string(),
        );
    }
    if !select_map_json.contains("locked") || !select_map_json.contains("\"text_x\": 920") {
        errors.push(
            "scenes.json `select_map` 必须声明 `locked.text_x` = 920（QA033：锁块 x 走 1080 数据驱动）"
                .to_string(),
        );
    }

    // 负向锚定：废弃的 `-item_h/2` 居中性不得残留（EXP-074 回归「基线上移 offset」语义）
    let layout_src = read_file(root, "crates/ti-core/src/data/mod.rs").unwrap_or_default();
    if layout_src.contains("item_h / 2") {
        errors.push(
            "ListLayout::item_rect 残留 `-item_h/2` 居中性，必须回归「基线上移 highlight_offset」语义（EXP-074）"
                .to_string(),
        );
    }
    if !layout_src.contains("highlight_offset") {
        errors.push(
            "ListLayout 缺少 `highlight_offset` 字段（EXP-074 高亮上移像素，旧代码 y-10/y-12）"
                .to_string(),
        );
    }

    // 标量解析器：extra_scalar 抽取 `*_x`/`*_y`（EXP-074 修复 extra 静默置 0）
    let layout_parser = read_file(root, "crates/ti-core/src/data/layout.rs").unwrap_or_default();
    if !layout_parser.contains("extra_scalar") {
        errors.push(
            "scenes 解析器缺少 `extra_scalar`（EXP-074：locked.text_x 等标量字段不得被 Rect 反序列化静默置 0）"
                .to_string(),
        );
    }

    // S15.6 QA133：实体 size 键（player.size / landmark.size / boss.size，§4.3 数据驱动）
    // 存在性锚——键缺失时 `main_map.rs` 静默回退 16×8/12×6/18×9 旧公式，
    // gate/doccheck2 全绿（QA030 修了「键被消费」，本锚补「键本身被守护」）。
    let main_map_section = scene_section(&json, "\"main_map\"");
    for ent in ["player", "landmark", "boss"] {
        let pat = format!("\"{ent}\":");
        let Some(pos) = main_map_section.find(&pat) else {
            errors.push(format!(
                "scenes.json main_map 缺实体对象 `{ent}`（QA133：§4.3 实体边长数据驱动键，缺失 = 代码静默回退旧公式）"
            ));
            continue;
        };
        let tail = &main_map_section[pos..];
        let close = tail.find('}').map_or(tail.len(), |i| i);
        if !tail[..=close].contains("\"size\":") {
            errors.push(format!(
                "scenes.json main_map `{ent}` 缺 `size` 键（QA133：§4.3 实体边长数据驱动，键缺失 = 渲染静默回退旧公式）"
            ));
        }
    }

    // 视觉回归必须掩背景（EXP-074：HD 背景 ≠ PCX 背景非排版漂移，只锁 UI 像素分布）。
    // S15.4：176×220 逐像素基线机制已暂移除（legacy 像素写帧层删除），`diff_frames_ui`/`is_bg_cur`
    // 不变量随 S15.5 重锚到 1080 代码绘制基线时重新落地；此期间门禁校验文件处于 S15.5 暂存状态。
    let vr = read_file(root, "crates/ti-app/tests/visual_regression.rs").unwrap_or_default();
    if !vr.contains("diff_frames_ui") && !vr.contains("S15.5") {
        errors.push(
            "visual_regression.rs 必须用 `diff_frames_ui` 掩背景色只比 UI 像素分布（EXP-074）；S15.4 起逐像素基线暂移除，文件应含 S15.5 重锚标记"
                .to_string(),
        );
    }
    errors
}

/// 取 `scene_key`（如 `"index"`）在 JSON 中的对象段（到配对的 `}` 为止）。
fn scene_section(json: &str, scene_key: &str) -> String {
    let Some(start) = json.find(scene_key) else {
        return String::new();
    };
    // 从 scene_key 之后找 `:` 后的 `{`，截取到配对的 `}`
    let after = &json[start..];
    let Some(colon) = after.find(':') else {
        return after.to_string();
    };
    let rest = &after[colon + 1..];
    let Some(open) = rest.find('{') else {
        return String::new();
    };
    let mut depth = 0usize;
    let mut end = open;
    for (i, c) in rest[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = open + i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    rest[open..end].to_string()
}

/// 20. 版面表 ↔ scenes.json 逐项对账（飞轮 F1 根治）：
///     `08_layout_design.md` §4 表是 `scenes.json` 的**人读镜像**（机器权威 = JSON）。
///     历史上 md 长期停留旧值（`row_h` 270/250、`text_x` 140、`hint` y 1860…）而无人发现——
///     `render_intent` / doccheck2 / 三套基线四者锁 JSON，md 是唯一漂移方。
///     断言：JSON 每个场景的每个整数标量键值对，必须以「键:值,」或「键:值}」形式
///     （即 md 表格 `{x:90, y:400, …}` 写法）出现在 md 文本中。
///     只查「存在性 + 数值字面」；漂移即改 md（scenes.json 不许改，改了破基线）。
fn check_layout_doc_table_cross_check(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(json_text) = read_file(root, "assets/data/scenes.json") else {
        return vec!["assets/data/scenes.json 不存在".to_string()];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) else {
        return vec!["assets/data/scenes.json 解析失败".to_string()];
    };
    let Some(md) = read_file(root, "docs/business/08_layout_design.md") else {
        return vec!["docs/business/08_layout_design.md 不存在".to_string()];
    };
    let Some(scenes) = json.as_object() else {
        return vec!["scenes.json 顶层必须是对象".to_string()];
    };
    let mut missing: Vec<String> = Vec::new();
    for (scene, val) in scenes {
        if scene == "schema_version" {
            continue;
        }
        let mut pairs: Vec<(String, i64)> = Vec::new();
        collect_scalar_pairs(val, &mut pairs);
        for (key, v) in pairs {
            let comma = format!("{key}:{v},");
            let braced = format!("{key}:{v}}}");
            if !md.contains(&comma) && !md.contains(&braced) {
                missing.push(format!("{scene}.{key}:{v}"));
            }
        }
        // `portrait_zones` 在 md 里两种写法混用（index 用 `[{x:0,…}]`，其余用位置
        // 写法 `[{0,…}]`），任一命中即通过；数值以 scenes.json 为准
        if let Some(serde_json::Value::Array(zones)) = val.get("portrait_zones") {
            for z in zones {
                let Some(obj) = z.as_object() else { continue };
                let get = |k: &str| obj.get(k).and_then(serde_json::Value::as_i64).unwrap_or(-1);
                let (x, y, w, h) = (get("x"), get("y"), get("w"), get("h"));
                let pos = format!("portrait_zones [{{{x},{y},{w},{h}}}]");
                let kv = [("x", x), ("y", y), ("w", w), ("h", h)].iter().all(|(k, v)| {
                    md.contains(&format!("{k}:{v},")) || md.contains(&format!("{k}:{v}}}"))
                });
                if !md.contains(&pos) && !kv {
                    missing.push(format!("{scene}.portrait_zones[{x},{y},{w},{h}]"));
                }
            }
        }
    }
    if !missing.is_empty() {
        errors.push(format!(
            "08_layout_design.md §4 表与 scenes.json 漂移（md 是 JSON 的人读镜像，改 md 不改 JSON）：{}",
            missing.join(", ")
        ));
    }
    errors
}

/// 递归收集对象/数组中的整数标量键值对（`portrait_zones` 数组一并纳入，
/// md 动态区行用 `portrait_zones [{x:0,…}]` 同式写法）。
/// 跳过 `size` 键（实体边长由 QA133 锚 + 渲染测试锁，md 以「≤26」散文描述）。
fn collect_scalar_pairs(val: &serde_json::Value, out: &mut Vec<(String, i64)>) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if k == "size" || k == "portrait_zones" {
                    continue;
                }
                collect_scalar_pairs(v, out);
                if let serde_json::Value::Number(n) = v
                    && let Some(i) = n.as_i64()
                {
                    out.push((k.clone(), i));
                }
            }
        }
        serde_json::Value::Array(items) => {
            for v in items {
                collect_scalar_pairs(v, out);
            }
        }
        _ => {}
    }
}

/// 21. 非 `use` 行禁止 `ti_core::` 全路径（EXP-061 收敛）：
///     跨 crate 引用必须收敛到文件顶部 `use`，非 `use` 行的 `ti_core::…`
///     全路径视为漂移隐患（同一符号两种写法，grep/门禁抓不到）。S16 渲染线
///     全库实测已归零（43 处 → 0），本断言固化防回潮。
fn check_no_ti_core_fullpath(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    for rel in [
        "crates/ti-app/src",
        "crates/ti-core/src",
        "crates/ti-render/src",
        "crates/ti-formats/src",
        "crates/ti-platform/src",
        "crates/ti-tools/src",
    ] {
        let dir = root.join(rel);
        if !dir.is_dir() {
            continue;
        }
        let mut hits: Vec<String> = Vec::new();
        let mut stack = vec![dir];
        while let Some(d) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().and_then(|x| x.to_str()) != Some("rs") {
                    continue;
                }
                // 断言自身实现含 `ti_core::` 字符串字面量，不自我指认
                if p.ends_with("doccheck2.rs") {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&p) else { continue };
                for (i, line) in text.lines().enumerate() {
                    let ls = line.trim_start();
                    if ls.starts_with("use ") || ls.starts_with("//") || ls.starts_with('*') {
                        continue;
                    }
                    if line.contains("ti_core::") {
                        hits.push(format!("{}:{}", p.display(), i + 1));
                    }
                }
            }
        }
        if !hits.is_empty() {
            errors.push(format!(
                "{rel} 非 use 行仍有 `ti_core::` 全路径（EXP-061：必须收敛到顶部 use）：{}",
                hits.join(", ")
            ));
        }
    }
    errors
}

fn check_scene_band_count(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(src) = read_file(root, "crates/ti-app/src/scenes/select_role.rs") else {
        return vec!["select_role.rs 不存在，无法校验 band 数".to_string()];
    };
    // 找到 `scene_list_bands(` 调用
    let Some(call_pos) = src.find("scene_list_bands(") else {
        return errors; // 已由 check_touch_render_pair 报错
    };
    // 提取调用块（平衡括号）
    let after_call = &src[call_pos + "scene_list_bands(".len()..];
    let mut depth = 1usize;
    let mut end_pos = 0usize;
    for (i, c) in after_call.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end_pos = i;
                    break;
                }
            }
            _ => {}
        }
    }
    let args_block = after_call[..end_pos].trim();
    // 按逗号取最后一个非空参数（band 数）
    let last_arg = args_block.rsplit(',').map(str::trim).find(|s| !s.is_empty()).unwrap_or("");
    // 判断 band 数是否为字面量 5 或常量 ROLE_COUNT
    let band_ok = last_arg == "5"
        || last_arg == "5usize"
        || last_arg == "ROLE_COUNT as usize"
        || last_arg == "ROLE_COUNT";
    if !band_ok {
        errors.push(format!(
            "select_role.rs scene_list_bands band 数参数为 `{last_arg}`，必须 == Character::all().len()（= 5）或 ROLE_COUNT（EXP-073）"
        ));
    }
    // 若走常量，验证常量值 == 5
    if last_arg.contains("ROLE_COUNT") {
        let Some(role_const_pos) = src.find("const ROLE_COUNT") else {
            errors.push("select_role.rs 引用 ROLE_COUNT 但未定义该常量".to_string());
            return errors;
        };
        let after_const = &src[role_const_pos..];
        let const_line_end = after_const.find(';').unwrap_or(0);
        let const_decl = &after_const[..=const_line_end];
        if !const_decl.contains("= 5") {
            errors.push(format!(
                "select_role.rs ROLE_COUNT 常量值 ≠ 5（声明: `{const_decl}`），必须 == Character::all().len()（EXP-073）"
            ));
        }
    }
    errors
}

/// FIND-203 固化：存读档 `map_slot` 字段写档/读档基准统一为 0 起 `zero_index`。
/// 机械判据：`crates/ti-app/src/scenes/saveload.rs` 与 `crates/ti-app/tests/fuzz_playtest.rs`
/// 中所有 `SaveSlot` 字面量构造处的 `map_slot` 赋值不得使用 `MapSlot::index`（1 起），
/// 须使用 `MapSlot::zero_index`（0 起）或字面量 `0`（无地图时回退值）。
fn check_save_slot_map_slot_basis(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let files = ["crates/ti-app/src/scenes/saveload.rs", "crates/ti-app/tests/fuzz_playtest.rs"];
    for rel in files {
        let src = read_file(root, rel).unwrap_or_default();
        for (i, line) in src.lines().enumerate() {
            if line.contains("MapSlot::index") && !line.contains("zero_index") {
                errors.push(format!(
                    "{rel}:{} `map_slot` 写档基准用了 `MapSlot::index`（1 起），应统一为 `MapSlot::zero_index`（0 起，FIND-203）：{}",
                    i + 1,
                    line.trim()
                ));
            }
        }
    }
    errors
}

/// 15. 穷举场景双写对账（S15.1 第 6 域补全，第 11 轮对抗审计 Q062/Q063/Q085 固化）：
///     `assets/data/exhaust_scenarios.json` 与 `assets/data/balance.json` 双写的数值常量
///     必须一致，防 balance 改值后 exhaust JSON 静默失配（exhaust 测试读 JSON，balance 读
///     真实表，两者各自「自洽」但互相矛盾时全部假绿）：
///     - ① `exhaust.combat.max_rounds_one_case_exact.player_atk` == `balance.combat.boss_tiers[tier=0].hp`
///       （exhaust 的「精确边界」用例假设 atk 恰好等于档 0 BOSS hp）；
///     - ② `exhaust.progression.card_event_id_max` == `25`（`cards.rs::all_event_ids` 返回
///       `(1..=25)` 硬编码上界，exhaust 卡牌穷举遍历 `0..=card_event_id_max`，两者必须一致）。
fn parse_json_file(root: &Path, rel: &str) -> Option<serde_json::Value> {
    let text = read_file(root, rel)?;
    serde_json::from_str(&text).ok()
}

fn check_exhaust_balance_dual_write(root: &Path) -> Vec<String> {
    use serde_json::Value;
    let mut errors = Vec::new();
    let exhaust = parse_json_file(root, "assets/data/exhaust_scenarios.json");
    if exhaust.is_none() {
        errors.push("exhaust_scenarios.json 不存在或解析失败".to_string());
        return errors;
    }
    let balance = parse_json_file(root, "assets/data/balance.json");
    if balance.is_none() {
        errors.push("balance.json 不存在或解析失败".to_string());
        return errors;
    }
    let (Some(exhaust), Some(balance)) = (exhaust, balance) else {
        return errors;
    };
    // ① player_atk 须 == 档 0 BOSS hp
    let player_atk =
        exhaust.pointer("/combat/max_rounds_one_case_exact/player_atk").and_then(Value::as_u64);
    let tier0_hp = balance.pointer("/combat/boss_tiers").and_then(|t| {
        t.as_array().and_then(|arr| {
            arr.iter()
                .find(|tier| tier.get("tier").and_then(Value::as_u64) == Some(0))
                .and_then(|tier| tier.get("hp"))
                .and_then(Value::as_u64)
        })
    });
    match (player_atk, tier0_hp) {
        (Some(a), Some(h)) if a != h => errors.push(format!(
            "exhaust_scenarios.json combat.max_rounds_one_case_exact.player_atk={a} \
             != balance.json combat.boss_tiers[0].hp={h}（双写失配，exhaust 精确边界用例假绿）"
        )),
        (None, _) => errors.push(
            "exhaust_scenarios.json 缺 combat.max_rounds_one_case_exact.player_atk".to_string(),
        ),
        (_, None) => errors.push("balance.json 缺 combat.boss_tiers[tier=0].hp".to_string()),
        _ => {}
    }
    // ② card_event_id_max 须 == 25（cards.rs all_event_ids 硬编码上界）
    let card_max = exhaust.pointer("/progression/card_event_id_max").and_then(Value::as_u64);
    if card_max != Some(25) {
        errors.push(format!(
            "exhaust_scenarios.json progression.card_event_id_max={card_max:?} != 25 \
             （cards.rs all_event_ids 返回 1..=25，exhaust 卡牌穷举遍历 0..=card_event_id_max 须与之上界一致）"
        ));
    }
    errors
}

/// 18. 运行时禁 legacy 坐标（S16 渲染线契约 1，`docs/process/08_render_arch.md`）：
///     呈现坐标系唯一 = 1080×1920。`ti-platform/gfx.rs`、`ti-app/menu_shell.rs`、
///     `ti-app/scenes/**`、`ti-app/src/touch.rs`、`ti-app/src/game_app.rs` 与
///     `ti-core/tests/**` 的**代码**（剔除 `//` / `///` / `//!` 注释与 `#[cfg(test)]`
///     模块）不得出现 legacy 尺寸字面量 `176` / `220` —— S15.2 事故根因就是
///     legacy 坐标泄漏进运行时（`select_map.rs` 曾写 `176 - text_x - 6` 钳进 176 宽屏）。
///     S15.6 QA047：目标集扩面（旧网恰好漏掉全部 176 尺度活件 `touch.rs`/`game_app.rs`
///     与 ti-core 测试面）。
///     同时锁呈现常量同值（`menu_shell` 的 `LOGICAL_W_I32`/`LOGICAL_H_I32` == `gfx` 的 `CANVAS_W`/`CANVAS_H`）。
fn check_runtime_no_legacy_coords(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let mut targets: Vec<std::path::PathBuf> = Vec::new();
    for rel in [
        "crates/ti-platform/src/gfx.rs",
        "crates/ti-app/src/menu_shell.rs",
        "crates/ti-app/src/touch.rs",
        "crates/ti-app/src/game_app.rs",
    ] {
        targets.push(root.join(rel));
    }
    {
        let mut scene_rs: BTreeSet<String> = BTreeSet::new();
        collect_recursive(&root.join("crates/ti-app/src/scenes"), &mut scene_rs);
        targets.extend(scene_rs.iter().map(|rel| root.join(rel)));
        // QA047：ti-core 集成测试面（`unwrap_or(176/220)` 休眠回退值类泄漏，data_tables.rs）。
        let mut core_tests: BTreeSet<String> = BTreeSet::new();
        collect_recursive(&root.join("crates/ti-core/tests"), &mut core_tests);
        targets.extend(core_tests.iter().map(|rel| root.join(rel)));
    }

    for path in targets {
        let Ok(text) = std::fs::read_to_string(&path) else { continue };
        let rel = path.strip_prefix(root).unwrap_or(path.as_path()).display().to_string();
        let mut in_test_mod = false;
        for (i, line) in text.lines().enumerate() {
            let t = line.trim();
            // 跳过 `#[cfg(test)]` 单测模块：测试夹具坐标（如 Viewport 变换的
            // 176×220 采样点）是合法输入数据，不算运行时坐标泄漏。
            if !in_test_mod && (t.starts_with("#[cfg(test)]") || t == "mod tests {") {
                in_test_mod = true;
                continue;
            }
            if in_test_mod {
                // 只认**列 0** 的 `}` 作模块结束（缩进的 `}` 是函数/块闭合，不算）
                if line == "}" {
                    in_test_mod = false;
                }
                continue;
            }
            // 剔除注释：整行注释 + 行尾注释（注释里的历史尺寸引用不算泄漏）
            let code = match line.find("//") {
                Some(p) => &line[..p],
                None => line,
            };
            if code.trim().is_empty() {
                continue;
            }
            for lit in ["176", "220"] {
                if contains_word(code, lit) {
                    errors.push(format!(
                        "{rel}:{} 运行时出现 legacy 坐标字面量 `{lit}`（契约 1：呈现坐标系唯一 1080×1920，见 08_render_arch.md §3/§5；改用 gfx::CANVAS_W/H 或 menu_shell::LOGICAL_*_I32）",
                        i + 1
                    ));
                }
            }
        }
    }

    // 呈现常量同值：权威坐标系单一源 = `ti_render::framebuffer`（S15.7 框架提取后，
    // LOGICAL_*_I32 由菜单层上移到渲染框架），平台层 gfx CANVAS_* 须与之同值。
    // menu_shell 现为 `pub use` 转发（非本地定义），故不再 grep 其 `pub const`。
    let gfx = read_file(root, "crates/ti-platform/src/gfx.rs").unwrap_or_default();
    let render = read_file(root, "crates/ti-render/src/framebuffer.rs").unwrap_or_default();
    for (a, b, label) in [
        ("pub const LOGICAL_W_I32: i32 = 1080;", "pub const CANVAS_W: u32 = 1080;", "宽度"),
        ("pub const LOGICAL_H_I32: i32 = 1920;", "pub const CANVAS_H: u32 = 1920;", "高度"),
    ] {
        if !render.contains(a) {
            errors.push(format!(
                "ti_render/framebuffer.rs 缺呈现常量 `{a}`（契约 1：框架权威坐标系单一源）"
            ));
        }
        if !gfx.contains(b) {
            errors.push(format!("gfx.rs 缺呈现常量 `{b}`（契约 1：{label}须与 ti_render 同值）"));
        }
    }
    errors
}

/// 行内是否出现独立词 `needle`（避免 `1176` / `2200` 之类误判）。
fn contains_word(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let n = needle.len();
    let mut from = 0usize;
    while let Some(rel) = haystack[from..].find(needle) {
        let at = from + rel;
        let before_ok = at == 0 || !bytes[at - 1].is_ascii_digit();
        let after = at + n;
        let after_ok = after >= bytes.len() || !bytes[after].is_ascii_digit();
        if before_ok && after_ok {
            return true;
        }
        from = at + n;
    }
    false
}

/// 17. 经验库分卷条数对账（第 12 轮对抗审计 Q016 类复发固化）：
///     `docs/experience/index.md` 分卷表「条数」列须 == 各卷文件实际 `### EXP-` heading 数
///     （历史 2 轮漂移：code/process 各差 5、tools/build 反向差 3/1，均靠人肉发现）。
fn check_experience_volume_counts(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(index) = read_file(root, "docs/experience/index.md") else { return errors };
    for line in index.lines() {
        let trimmed = line.trim();
        // 仅解析分卷表行：`| [`file.md`](file.md) | 内容 | 条数 |`（3 列且首列含 `](…md)`）
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }
        let body = &trimmed[1..trimmed.len() - 1];
        let cols: Vec<&str> = body.split('|').map(str::trim).collect();
        if cols.len() != 3 || !cols[0].contains(".md](") {
            continue;
        }
        let Some(p) = cols[0].find("](") else { continue };
        let seg = &cols[0][p + 2..];
        let Some(close) = seg.find(')') else { continue };
        let fname = &seg[..close];
        let Ok(declared) = cols[2].parse::<usize>() else { continue };
        let actual = read_file(root, &format!("docs/experience/{fname}"))
            .map_or(0, |t| t.lines().filter(|l| l.starts_with("### EXP-")).count());
        if actual != declared {
            errors.push(format!(
                "docs/experience/index.md 分卷表条数列：{fname} 声明 {declared}，实际 `### EXP-` heading {actual}（Q016 类漂移，改条目时须同步分卷表）"
            ));
        }
    }
    errors
}

/// 19. 基线镜像恒等（S15.6 QA014 / B1 裁决③「三套基线镜像恒等」）：
///     `assets/render_baseline` 为唯一权威基线，`forward_baseline` / `hd_baseline`
///     是同一代码渲染器的**镜像**（S15.4 删 legacy 写帧层后两态像素恒等）。
///     对应场景文件须**逐字节恒等**（比 SHA256 恒等更强：字节同 ⇒ 哈希同），
///     防未来某套基线被单独改动导致三套门禁静默分叉。
///     场景名映射：render（legacy 名）↔ forward（scenes.json 键）/ hd（PascalCase）。
fn check_baseline_mirror_identity(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    // (render legacy 名, forward scenes.json 键) 7 对
    let maps: [(&str, &str); 7] = [
        ("index", "index"),
        ("selectrole", "select_role"),
        ("selectmap", "select_map"),
        ("main", "main_map"),
        ("vs", "vs"),
        ("msgs", "message"),
        ("saveload", "saveload"),
    ];
    for (render, forward) in maps {
        let rp = root.join("assets/render_baseline").join(format!("{render}.png"));
        let fp = root.join("assets/forward_baseline").join(format!("{forward}.png"));
        let (rb, fb) = (std::fs::read(&rp), std::fs::read(&fp));
        match (rb, fb) {
            (Ok(a), Ok(b)) => {
                if a != b {
                    errors.push(format!(
                        "基线镜像分叉：render_baseline/{render}.png（{} bytes）≠ forward_baseline/{forward}.png（{} bytes）（B1 裁决③：forward 为 render 的镜像，同代码生成应逐字节恒等；某套被单独改动）",
                        a.len(),
                        b.len()
                    ));
                }
            }
            (Err(e1), Err(e2)) => {
                errors.push(format!(
                    "基线镜像缺失：render_baseline/{render}.png（{e1}）与 forward_baseline/{forward}.png（{e2}）均不存在（跑 `cargo test -p ti-app --test render_snapshot --test render_snapshot_forward` 重生成）"
                ));
            }
            _ => {
                errors.push(format!(
                    "基线镜像缺失：render_baseline/{render}.png 与 forward_baseline/{forward}.png 至少一个不存在（跑快照测试重生成）"
                ));
            }
        }
    }
    // hd_baseline 3 文件（Index/SelectRole/MainMap）与 render 对应文件恒等
    let hd: [(&str, &str); 3] =
        [("index", "Index"), ("selectrole", "SelectRole"), ("main", "MainMap")];
    for (render, hd_name) in hd {
        let rp = root.join("assets/render_baseline").join(format!("{render}.png"));
        let hp = root.join("assets/hd_baseline").join(format!("{hd_name}.png"));
        match (std::fs::read(&rp), std::fs::read(&hp)) {
            (Ok(a), Ok(b)) => {
                if a != b {
                    errors.push(format!(
                        "基线镜像分叉：render_baseline/{render}.png ≠ hd_baseline/{hd_name}.png（B1 裁决③：hd 为 render 的镜像，同代码生成应逐字节恒等）"
                    ));
                }
            }
            (Err(e1), Err(e2)) => {
                errors.push(format!(
                    "基线镜像缺失：render_baseline/{render}.png（{e1}）与 hd_baseline/{hd_name}.png（{e2}）均不存在（跑 `cargo test -p ti-app --test visual_regression` 重生成）"
                ));
            }
            _ => {
                errors.push(format!(
                    "基线镜像缺失：render_baseline/{render}.png 与 hd_baseline/{hd_name}.png 至少一个不存在（跑快照测试重生成）"
                ));
            }
        }
    }
    errors
}

/// 标准库 trait 选用矩阵（`docs/style.md` §5.6.6）的落地断言：
/// 文档里点名的「先例」impl 必须真实存在于对应源文件，防止被静默删除
/// （§5.6.6 选用矩阵与代码双向锚定，同 EXP-058 防虚报口径）。
fn check_std_traits_matrix(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    // (文件, 必须存在的 impl 签名, 说明)
    let expectations: [(&str, &str, &str); 5] = [
        (
            "crates/ti-app/src/commands.rs",
            "impl From<GameCommand> for GameInput",
            "GameCommand → GameInput 零损失转换（§5.6.6 先例）",
        ),
        (
            "crates/ti-platform/src/settings.rs",
            "impl From<FilterMode> for macroquad::miniquad::FilterMode",
            "FilterMode → miniquad 零损失转换（§5.6.6 先例）",
        ),
        (
            "crates/ti-render/src/framebuffer.rs",
            "impl AsRef<[u16]> for Framebuf",
            "Framebuf 借用视图（§5.6.6 缓冲/容器 → AsRef/AsMut）",
        ),
        (
            "crates/ti-platform/src/gfx.rs",
            "pub struct FramePresenter",
            "呈现器（脏行跟踪归属方，S15.10 起脏行状态在 presenter 而非帧缓冲）",
        ),
        (
            "crates/ti-formats/src/ids.rs",
            "impl From<u32> for ControlKind",
            "ControlKind ↔ u32 newtype ID（§5.6.6 先例 / §5.4 define_id! 参考实现）",
        ),
    ];
    for (rel, needle, why) in expectations {
        let Some(text) = read_file(root, rel) else {
            errors.push(format!("缺文件 {rel}（{why}）"));
            continue;
        };
        if !text.contains(needle) {
            errors.push(format!(
                "{rel} 缺 `{needle}`（{why}；§5.6.6 选用矩阵已登记为先例，删除须先改文档）"
            ));
        }
    }
    errors
}

/// UI 重设计令牌锁（S17.2，`docs/process/10_ui_redesign.md` §3/§4/§8.1）：
/// ① `scenes.json` 全部版面坐标 8pt 整除（业务几何 `grid.*`/棋子 `size`/`portrait_zones` 整棵豁免）；
/// ② 色板三方恒等：`assets/data/theme.json`（运行时数据源）== `ti-render/src/theme.rs`
///    `Theme::DEFAULT`（代码锚点）== `docs/process/10_ui_redesign.md` §3 令牌表（人读镜像）。
fn check_ui_redesign_tokens(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    errors.extend(ui_8pt_check(root));
    errors.extend(ui_palette_triple_check(root));
    errors.extend(ui_icon_registry_check(root));
    errors
}

/// S17.3 图标注册表完备性（`docs/process/10_ui_redesign.md` §5）：
/// ① `assets/data/ui_icons.json` 恰好 14 枚、id 连续 0..13、name 唯一；
/// ② 每枚 id 在 `ti-render/src/icons.rs` 有对应 `ICON_*` 分支（防注册了但没画）；
/// ③ bindings 引用的图标名都在 `icons` 内（防悬空引用）。
fn ui_icon_registry_check(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(text) = read_file(root, "assets/data/ui_icons.json") else {
        errors.push("缺 assets/data/ui_icons.json（S17.3 图标注册表）".to_string());
        return errors;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) else {
        errors.push("assets/data/ui_icons.json 解析失败".to_string());
        return errors;
    };
    let Some(icons) = parsed.get("icons").and_then(serde_json::Value::as_array) else {
        errors.push("ui_icons.json 缺 icons[]".to_string());
        return errors;
    };
    if icons.len() != 14 {
        errors.push(format!(
            "ui_icons.json icons[] = {} 条，应 14 枚（10_ui_redesign §5）",
            icons.len()
        ));
    }
    let mut ids: Vec<u32> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    for it in icons {
        let (Some(name), Some(id)) = (
            it.get("name").and_then(serde_json::Value::as_str),
            it.get("id").and_then(serde_json::Value::as_u64),
        ) else {
            errors.push("ui_icons.json icons[] 缺 name/id 字段".to_string());
            continue;
        };
        if let Ok(id_u32) = u32::try_from(id) {
            ids.push(id_u32);
        }
        names.push(name.to_string());
    }
    for (want, got) in (0..14).zip(&ids) {
        if want != *got {
            errors.push(format!("ui_icons.json id 序列非连续 0..13：期望 {want} 实际 {got}"));
        }
    }
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    if sorted.len() != names.len() {
        errors.push("ui_icons.json 图标名重复".to_string());
    }
    // ② 框架侧图标分支覆盖（`match kind { ICON_X => … }`）
    let Some(icons_rs) = read_file(root, "crates/ti-render/src/icons.rs") else {
        return errors;
    };
    for name in &names {
        let const_name = if name == "monogram" {
            "ICON_MONOGRAM_RING".to_string()
        } else {
            format!("ICON_{}", name.to_uppercase())
        };
        let has_const = icons_rs.contains(&format!("pub const {const_name}"));
        let has_branch = icons_rs.contains(&format!("{const_name} =>"));
        if !(has_const && has_branch) {
            errors.push(format!(
                "ui_icons.json 登记 `{name}` 但 ti_render::icons 缺 {const_name} 常量/绘制分支（S17.3：注册即须可画）"
            ));
        }
    }
    // ③ bindings 引用名在册
    if let Some(bindings) = parsed.get("bindings") {
        let mut refs: Vec<String> = Vec::new();
        if let Some(arr) = bindings.get("index_menu").and_then(serde_json::Value::as_array) {
            refs.extend(
                arr.iter()
                    .filter_map(|b| b.get("icon").and_then(serde_json::Value::as_str))
                    .map(str::to_string),
            );
        }
        if let Some(arr) = bindings.get("main_map_hud").and_then(serde_json::Value::as_array) {
            refs.extend(
                arr.iter()
                    .filter_map(|b| b.get("icon").and_then(serde_json::Value::as_str))
                    .map(str::to_string),
            );
        }
        for key in ["select_map_row", "select_role_row", "main_map_landmark", "vs_boss_cap"] {
            if let Some(n) =
                bindings.get(key).and_then(|v| v.get("icon")).and_then(serde_json::Value::as_str)
            {
                refs.push(n.to_string());
            }
        }
        for r in refs {
            if !names.contains(&r) {
                errors.push(format!(
                    "ui_icons.json bindings 引用 `{r}`，但 icons[] 无此名（S17.3：禁悬空引用）"
                ));
            }
        }
    }
    errors
}

/// ① 8pt 栅格：遍历 scenes.json 各场景标量（豁免业务几何、动态落区与屏心锚点 540）。
fn ui_8pt_walk(
    obj: &serde_json::Value,
    in_exempt: bool,
    key: &str,
    scene: &str,
    errors: &mut Vec<String>,
) {
    let exempt_by_key =
        matches!(key, "x0" | "y0" | "tile_w" | "tile_h" | "size" | "portrait_zones");
    match obj {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                let next_exempt = in_exempt || exempt_by_key || k == "portrait_zones";
                ui_8pt_walk(v, next_exempt, k, scene, errors);
            }
        }
        serde_json::Value::Array(items) => {
            for it in items {
                ui_8pt_walk(it, in_exempt || exempt_by_key, key, scene, errors);
            }
        }
        serde_json::Value::Number(n) => {
            if in_exempt || exempt_by_key {
                return;
            }
            if let Some(v) = n.as_i64() {
                if v % 8 != 0 && v != 540 {
                    errors.push(format!(
                        "scenes.json {scene}: `{key}` = {v} 非 8pt 栅格（10_ui_redesign §4：版面坐标 8 整除；\
                         业务几何 grid.*/size、portrait_zones 与屏心锚点 540 已豁免）"
                    ));
                }
            }
        }
        _ => {}
    }
}

/// ① 8pt 栅格入口（scenes.json 各场景）。
fn ui_8pt_check(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(json_text) = read_file(root, "assets/data/scenes.json") else {
        return errors;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) else {
        return errors;
    };
    let Some(scenes) = json.as_object() else {
        return errors;
    };
    for (key, val) in scenes {
        if key == "schema_version" {
            continue;
        }
        ui_8pt_walk(val, false, key, key, &mut errors);
    }
    errors
}

/// ② 色板三方恒等：`theme.json` == `Theme::DEFAULT` == `10_ui_redesign.md` §3 表。
fn ui_palette_triple_check(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(theme_json_text) = read_file(root, "assets/data/theme.json") else {
        return errors;
    };
    let Some(theme_rs) = read_file(root, "crates/ti-render/src/theme.rs") else {
        return errors;
    };
    // theme.rs DEFAULT 块：`field: 0xNNNN,`
    let mut code_map: Vec<(String, u16)> = Vec::new();
    if let Some(start) = theme_rs.find("pub const DEFAULT") {
        let end =
            theme_rs[start..].find("black: 0x0000").map_or(theme_rs.len(), |o| start + o + 16);
        for line in theme_rs[start..end].lines() {
            let Some((name, val)) = line.trim().trim_end_matches(',').split_once("0x") else {
                continue;
            };
            let field = name.trim().trim_end_matches(':').to_string();
            if val.len() == 4 && field.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                if let Ok(v) = u16::from_str_radix(val, 16) {
                    code_map.push((field, v));
                }
            }
        }
    }
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&theme_json_text) else {
        return errors;
    };
    let Some(palette) = parsed.get("palette").and_then(serde_json::Value::as_object) else {
        return errors;
    };
    for (field, val) in palette {
        let Some(json_v) = val
            .as_str()
            .and_then(|s| s.strip_prefix("0x"))
            .and_then(|h| u16::from_str_radix(h, 16).ok())
        else {
            continue;
        };
        match code_map.iter().find(|(f, _)| f == field) {
            Some((_, code_v)) if *code_v != json_v => {
                errors.push(format!(
                    "theme.json `{field}` = 0x{json_v:04X} 但 Theme::DEFAULT = 0x{code_v:04X}（S17.2：theme.json 运行时数据源须与代码锚点恒等）"
                ));
            }
            None => errors.push(format!(
                "theme.json palette 字段 `{field}` 在 Theme::DEFAULT 无对应（S17.2 三方对账）"
            )),
            _ => {}
        }
    }
    // 文档 §3 表：`| `token` | `#hex` | `0xNNNN` | `COLOR_X` / … |` → RGB565 列 vs theme.json
    // （经 COLOR_ 常量名映射到 theme 字段；反引号精确匹配防子串碰撞）
    let Some(doc) = read_file(root, "docs/process/10_ui_redesign.md") else {
        return errors;
    };
    let mut doc_errors = ui_doc_table_check(&doc, palette);
    errors.append(&mut doc_errors);
    errors
}

/// ②-3 文档 §3 令牌表（`10_ui_redesign.md`）RGB565 列 vs `theme.json`（人读镜像漂移检测）。
fn ui_doc_table_check(
    doc: &str,
    palette: &serde_json::Map<String, serde_json::Value>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let color_to_field: &[(&str, &str)] = &[
        ("COLOR_PARCH_DARK", "parch_dark"),
        ("COLOR_BG", "bg"),
        ("COLOR_WOOD", "wood"),
        ("COLOR_MENU_BORDER", "menu_border"),
        ("COLOR_TITLE_BAR", "title_bar"),
        ("COLOR_GOLD", "gold"),
        ("COLOR_LANDMARK", "landmark"),
        ("COLOR_TEXT_ON_WOOD", "text_on_wood"),
        ("COLOR_GOLD_GLOW", "gold_glow"),
        ("COLOR_GOLD_BRIGHT", "gold_bright"),
        ("COLOR_SELECTED", "selected"),
        ("COLOR_MENU_BG", "menu_bg"),
        ("COLOR_BEVEL_HI", "bevel_hi"),
        ("COLOR_HILITE", "hilite"),
        ("COLOR_SECONDARY", "secondary"),
        ("COLOR_PLAYER", "player"),
        ("COLOR_DANGER", "danger"),
        ("COLOR_BEVEL_SHADOW", "bevel_shadow"),
        ("COLOR_TEXT_ON_PARCH", "text_on_parch"),
    ];
    for line in doc.lines() {
        let parts: Vec<&str> = line.split('`').collect();
        // §3 表行：parts[1]=token, parts[3]=8-bit hex, parts[5]=RGB565
        let Some(rgb) = parts.get(5) else {
            continue;
        };
        if !rgb.starts_with("0x") || rgb.len() != 6 {
            continue;
        }
        let Ok(rgb565) = u16::from_str_radix(&rgb[2..], 16) else {
            continue;
        };
        let Some((_, field)) =
            color_to_field.iter().find(|(c, _)| line.contains(&format!("`{c}`")))
        else {
            continue;
        };
        let Some(expected) = palette
            .get(&field.to_string())
            .and_then(|v| v.as_str())
            .and_then(|s| s.strip_prefix("0x"))
            .and_then(|h| u16::from_str_radix(h, 16).ok())
        else {
            continue;
        };
        if expected != rgb565 {
            errors.push(format!(
                "10_ui_redesign.md §3 表 `{field}` = 0x{rgb565:04X} 但 theme.json = 0x{expected:04X}（文档表 = 数值镜像，须同步）"
            ));
        }
    }
    errors
}

/// 渲染框架分离收口防复发（S15.10，`docs/process/08_render_arch.md` §2 契约）：
/// ① `MenuShell::fill_rect` 不得自带像素循环（须转发 `ti_render` 帧缓冲）；
/// ② `ti-render` 不得依赖 `ti-platform`（框架可复用到其他游戏 = 依赖单向且不反向）；
/// ③ `ti-render` 不得依赖 `ti-formats`（不认本游戏资产载体格式，字体走中性 `FontSet`）。
fn check_render_framework_boundary(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();

    // ① MenuShell::fill_rect 内不得出现像素索引运算（禁手写重复实现）
    let Some(shell) = read_file(root, "crates/ti-app/src/menu_shell.rs") else {
        errors.push("缺 crates/ti-app/src/menu_shell.rs（无法校验 fill_rect 单源）".to_string());
        return errors;
    };
    if let Some(idx) = shell.find("pub fn fill_rect") {
        let body: String = shell[idx..].chars().take(600).collect();
        // 转发实现应命中 `fb.fill_rect(`；若出现 `as_mut_slice` 或 `* LOGICAL_W_USIZE`
        // 说明像素循环又回到了 ti-app（重复实现）
        if body.contains("as_mut_slice") || body.contains("LOGICAL_W_USIZE") {
            errors.push(
                "menu_shell.rs::fill_rect 又出现手写像素循环（S15.10：像素单源 = ti_render 帧缓冲，\
                 本方法只应转发 + presenter.mark_dirty）"
                    .to_string(),
            );
        }
        if !body.contains("presenter.mark_dirty") {
            errors.push(
                "menu_shell.rs::fill_rect 未标脏（应调 presenter.mark_dirty —— S15.10 脏行跟踪在呈现层）"
                    .to_string(),
            );
        }
    }

    // ② ti-render 不得依赖 ti-platform；③ 不得依赖 ti-formats
    let Some(cargo) = read_file(root, "crates/ti-render/Cargo.toml") else {
        errors.push("缺 crates/ti-render/Cargo.toml（无法校验依赖方向）".to_string());
        return errors;
    };
    for forbidden in ["ti-platform", "ti-formats", "ti-core", "ti-app"] {
        if cargo.contains(forbidden) {
            errors.push(format!(
                "ti-render/Cargo.toml 依赖了 `{forbidden}`（S15.10：渲染框架须可复用，\
                 只依赖 ab_glyph/serde/serde_json；游戏资产载体格式由 ti-app 在转换层解耦）"
            ));
        }
    }

    errors
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = args.first().map_or_else(|| std::path::PathBuf::from("."), std::path::PathBuf::from);

    let mut all_errors: Vec<String> = Vec::new();
    all_errors.extend(check_audio_state(&root));
    all_errors.extend(check_shell_oracle(&root));
    all_errors.extend(check_lcg_hex(&root));
    all_errors.extend(check_landmark_threshold(&root));
    all_errors.extend(check_i18n_orphans(&root));
    all_errors.extend(check_gate_item_count(&root));
    all_errors.extend(check_headless_battle_api(&root));
    all_errors.extend(check_flywheel_readback(&root));
    all_errors.extend(check_battle_player_data_driven(&root));
    all_errors.extend(check_flywheel_script(&root));
    all_errors.extend(check_touch_render_pair(&root));
    all_errors.extend(check_scene_band_count(&root));
    all_errors.extend(check_scene_layout_fidelity(&root));
    all_errors.extend(check_hd_sprite_transparency(&root));
    all_errors.extend(check_save_slot_map_slot_basis(&root));
    all_errors.extend(check_exhaust_balance_dual_write(&root));
    all_errors.extend(check_experience_volume_counts(&root));
    all_errors.extend(check_runtime_no_legacy_coords(&root));
    all_errors.extend(check_baseline_mirror_identity(&root));
    all_errors.extend(check_layout_doc_table_cross_check(&root));
    all_errors.extend(check_no_ti_core_fullpath(&root));
    all_errors.extend(check_std_traits_matrix(&root));
    all_errors.extend(check_render_framework_boundary(&root));
    all_errors.extend(check_ui_redesign_tokens(&root));

    println!(
        "doccheck2: {} error(s) across 24 audit-codified checks (audio/oracle/lcg/landmark/i18n/gate-items/headless-battle-api/flywheel-readback/battle-player-data-driven/flywheel-script/touch-render-pair/hd-sprite-transparency/scene-layout-fidelity/scene-band-count/save-slot-map-slot-basis/exhaust-balance-dual-write/experience-volume-counts/runtime-no-legacy-coords/baseline-mirror-identity/layout-doc-table-cross-check/no-ti-core-fullpath/std-traits-matrix/render-framework-boundary/ui-redesign-tokens)",
        all_errors.len()
    );
    for err in &all_errors {
        println!("  ERROR {err}");
    }
    if all_errors.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
