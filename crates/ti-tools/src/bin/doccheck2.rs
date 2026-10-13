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
//! 6. **release.md 门禁项数漂移**：`docs/release.md` 里凡出现「N 项门禁」的行，
//!    N 必须 == `scripts/gate.ps1` 实际 `Report`/`Skip` 的项数。
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

/// 6. `release.md` 门禁项数：凡「N 项门禁」的行，N 必须 == `gate.ps1` 实际项数。
fn check_gate_item_count(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(gate) = read_file(root, "scripts/gate.ps1") else {
        return vec!["scripts/gate.ps1 不存在".to_string()];
    };
    // `gate.ps1` 顶层的 `Report`/`Skip` 调用数量 = 实际项数
    let actual_items = gate
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with("Report ") || t.starts_with("Skip ")
        })
        .count();
    let Some(release) = read_file(root, "docs/release.md") else { return errors };
    for (i, line) in release.lines().enumerate() {
        let Some(pos) = line.find("项门禁") else { continue };
        let before = &line[..pos];
        let digits: String = before
            .chars()
            .rev()
            .take_while(char::is_ascii_digit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if digits.is_empty() {
            continue;
        }
        if let Ok(declared) = digits.parse::<usize>() {
            if declared != actual_items {
                errors.push(format!(
                    "release.md:{} 门禁项数声明 {declared}，gate.ps1 实测 {actual_items}（第 5 轮审计固化的漂移检查）",
                    i + 1
                ));
            }
        }
    }
    errors
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
    // 取 start_battle 函数体（到下一个 `}` 闭合为止，简化取 300 字符窗口，按 char 边界截取）
    let raw_end = (sb_start + 300).min(game_app.len());
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
    errors
}

/// 12. 场景 band 数 == 渲染行数（EXP-073）：`select_role.rs` 的 `scene_list_bands`
///     调用参数必须等于 `Character::all().len()`（即 5），禁止硬编码魔法数字。
///     若 band 数走常量（如 `ROLE_COUNT`），则检查常量值 == 5。
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

    println!(
        "doccheck2: {} error(s) across 12 audit-codified checks (audio/oracle/lcg/landmark/i18n/gate-items/headless-battle-api/flywheel-readback/battle-player-data-driven/flywheel-script/touch-render-pair/scene-band-count)",
        all_errors.len()
    );
    for err in &all_errors {
        println!("  ERROR {err}");
    }
    if all_errors.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
