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
//! 9. **战斗玩家数值数据驱动（EXP-066 闭环）**：`game_app/host.rs` 的 `start_battle`
//!    不得写死 `Unit { hp: 1000, atk: 150 }`，须读 `balance().combat.player`。
//!
//! 10. **飞轮开轮必读脚本（EXP-070）**：`scripts/flywheel_readme.ps1` 存在、
//!     `gate.ps1` 含 `flywheel_readme` 关键词、`06_flywheel.md` 含「开轮必读脚本化」、
//!     `docs/experience/index.md` 含 `EXP-070`。
//!
//! 用法：`cargo run -q -p ti-tools -- doccheck2 [repo_root]`（缺省 `repo_root = "."`）。

use std::collections::{BTreeMap, BTreeSet};
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
    let brace = after.find('{').unwrap_or(after.len());
    let close = after[brace..].find('}').unwrap_or(after.len()) + brace;
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

/// 收集单份 Rust 源码里 `sget("key")` / `sgetf("key", …)` 的字面量 key（整文本扫描，
/// 支持多行书写：`(` 与字符串字面量之间允许任意空白/换行；标识符 `sget`/`sgetf` 边界
/// 判定避免 `xsget` 类前缀误配；key 读到 `"` 或换行为止——Rust 普通字符串字面量不跨行）。
fn collect_sget_keys(text: &str, out: &mut BTreeSet<String>) {
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = text[i..].find("sget") {
        let start = i + rel;
        let after = start + 4;
        if start > 0 {
            let prev = bytes[start - 1];
            if prev.is_ascii_alphanumeric() || prev == b'_' {
                i = after;
                continue;
            }
        }
        let mut cursor = after;
        if cursor < bytes.len() && bytes[cursor] == b'f' {
            cursor += 1; // sgetf
        }
        while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t' | b'\n') {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'(' {
            i = after;
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && matches!(bytes[cursor], b' ' | b'\t' | b'\n') {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'"' {
            i = after;
            continue;
        }
        cursor += 1;
        let key_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'"' && bytes[cursor] != b'\n' {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b'"' {
            out.insert(text[key_start..cursor].to_string());
        }
        i = after;
    }
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

    // 收集代码里 `sget`/`sgetf` 的字面量 key。
    // 盲区修复（S26 审计 Q031/Q032）：旧实现按**行**找 `sget("`，多行书写
    //   i18n::sgetf(\n        "key",\n        &[...],\n    )
    // 会漏掉（`sgetf(` 行尾无 `"`）⇒ 假孤儿。改整文本扫描：定位 `sget`/`sgetf` 标识符，
    // 跳过 `(` 与引号间的任意空白/换行再读 key。
    let mut used_keys: BTreeSet<String> = BTreeSet::new();
    for text in read_rs_tree(&root.join("crates/ti-app/src")) {
        collect_sget_keys(&text, &mut used_keys);
        // `MenuItem { key: "…", … }` const 表里直接写死的 i18n key（单行）
        for line in text.lines() {
            if let Some(ki) = line.find("key:") {
                let rest = &line[ki + 4..];
                let Some(q) = rest.find('"') else { continue };
                let inner = &rest[q + 1..];
                let Some(end) = inner.find('"') else { continue };
                used_keys.insert(inner[..end].to_string());
            }
        }
    }
    // 盲区修复（Q032）：`scenes.json` 原语 `text_key` 也是 i18n key 消费方（旧实现只扫
    // ti-app/src ⇒ 仅 JSON 引用的 key 被误报孤儿）。并入 used_keys。
    if let Some(scn) = read_file(root, "assets/data/scenes.json") {
        for line in scn.lines() {
            let t = line.trim();
            let Some(ki) = t.find("\"text_key\"") else { continue };
            let rest = &t[ki + "\"text_key\"".len()..];
            let Some(co) = rest.find(':') else { continue };
            let after = &rest[co + 1..];
            let Some(q) = after.find('"') else { continue };
            let inner = &after[q + 1..];
            let Some(end) = inner.find('"') else { continue };
            used_keys.insert(inner[..end].to_string());
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
///    `docs/git.md` / `docs/README.md` 里所有「gate N 项」类声称值，必须 == 实测项数。
///    实测口径（S24.1 分档感知，FIND-304-B 修复）：按 **Report 调用点的项名去重**计数——
///    分档后 `Report` 调用被缩进进 `if (Tier)` 块，且条件 if/else 分支共享同一项名
///    （`check_data_driven` 的 `Report $false`/`Report $true` 两分支 = 1 项），故取
///    「行内首个引号串 = 项名」提取并去重，与档位无关、与条件分支无关。
///    豁免项编号（「第 N 项」）/ doccheck2 检查数声称（「N 项检查」）/ playbook 命令数
///    声称（「N 项命令」）不在此口径内，上下文区分「gate 项数」vs「doccheck2 项数」。
fn gate_item_actual_count(gate: &str) -> usize {
    use std::collections::BTreeSet;
    let mut names: BTreeSet<String> = BTreeSet::new();
    for line in gate.lines() {
        let t = line.trim_start();
        // 只数 `Report ` 调用（`Skip` / `function Report` 定义不计）
        if !t.starts_with("Report ") {
            continue;
        }
        let rest = &t["Report ".len()..];
        let first = rest.find(|c| ['\'', '"'].contains(&c));
        let last = rest.rfind(|c| ['\'', '"'].contains(&c));
        if let (Some(s), Some(e)) = (first, last) {
            if e > s + 1 {
                names.insert(rest[s + 1..e].to_string());
            }
        }
    }
    names.len()
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
    let expected = 28usize; // 按项名去重口径（S24.1 分档 + S25 check_code_standards 入 28）；门禁结构变化时同步
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
    let game_app = read_file(root, "crates/ti-app/src/game_app/battle.rs");
    let (Some(headless), Some(game_app)) = (headless, game_app) else {
        return vec![
            "treasure_headless.rs 或 game_app.rs 不存在，无法校验 BattleView API 一致性"
                .to_string(),
        ];
    };
    // 从 game_app.rs 抽 `pub struct BattleView` 的真实字段名集合
    let Some(struct_start) = game_app.find("pub struct BattleView") else {
        return vec!["game_app/battle.rs 缺少 `pub struct BattleView` 定义".to_string()];
    };
    let after = &game_app[struct_start..];
    let brace = after.find('{').unwrap_or(after.len());
    let close = after[brace..].find('}').unwrap_or(after.len()) + brace;
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

/// 9. 战斗玩家数值数据驱动（EXP-066 闭环）：`game_app/host.rs` 的 `start_battle`
///    不得写死 `Unit { hp: 1000, atk: 150 }`，须读 `balance().combat.player`。
///    机械判据：`game_app.rs` 里 `start_battle` 函数体须含 `balance().combat.player`
///    或 `combat.player` 引用（P0-1 修复后闭环验证，防再次回归）。
fn check_battle_player_data_driven(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(game_app) = read_file(root, "crates/ti-app/src/game_app/host.rs") else {
        return vec!["game_app/host.rs 不存在，无法校验战斗玩家数值数据驱动".to_string()];
    };
    let Some(sb_start) = game_app.find("fn start_battle") else {
        return vec!["game_app/host.rs 缺少 `fn start_battle`".to_string()];
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

/// 11. 命中/渲染成对（EXP-071 重定基线，S29.2 B1）：「点到哪个元素」的正逆成对已从
///     `touch::layout` 坐标几何（已退役）迁到 **Bevy 拾取层 + 数据驱动**（`ti_engine::pick`
///     `PickTarget` + `scenes.json` `pick_key`/`pick_index` → `HitData` → `on_hit`/`route_hit`）。
///     门禁 = ①命中几何数据化（scenes.json 有 `pick_key` + **逐场景必需键集合**，
///     S27.5 教训：`main_map` `on_hit` 消费 4..=7 但该场景数据面 0 个 `pick_key`
///     ⇒ 主地图四键全不可点，旧门禁只查文件含 `pick_key` 字样即放行）；②场景消费
///     归一命中（`on_hit`，不残留已退役坐标几何 API）；③`touch.rs` 坐标几何退役
///     （仅留手势识别）；④拾取层在位（`pick.rs` + `spawn_pick_layer`）。
fn check_touch_render_pair(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    // ① 命中几何数据化：scenes.json 须有 `pick_key`（`PickTarget` 键源）。
    let scenes = read_file(root, "assets/data/scenes.json").unwrap_or_default();
    if !scenes.contains("pick_key") {
        errors.push(
            "scenes.json 缺 `pick_key`（S29.2 B1：命中几何数据化，`ti_engine::pick` `PickTarget` 键源缺失）"
                .to_string(),
        );
    }
    // ①b 逐场景必需命中键（S27.5 教训固化，独立函数防本函数超长）。
    check_scene_required_pick_keys(&scenes, &mut errors);
    // ② 场景消费归一命中（`on_hit`/`HitData`），已退役坐标几何 API 不得残留。
    for rel in [
        "crates/ti-app/src/scenes/select_role.rs",
        "crates/ti-app/src/scenes/select_map.rs",
        "crates/ti-app/src/scenes/saveload.rs",
        "crates/ti-app/src/scenes/main_map.rs",
    ] {
        let src = read_file(root, rel).unwrap_or_default();
        if src.contains("fn on_touch") {
            errors.push(format!("{rel} 仍残留旧 `on_touch`（S29.2 B1 应改归一命中 `on_hit`）"));
        }
        for old in [
            "scene_list_bands",
            "slot_bands",
            "scene_list_index_at",
            "slot_band_index_at",
            "grid_cell_at",
            "iso_grid_cell_at",
            "menu_medallion_at",
            "menu_row_at",
        ] {
            if src.contains(old) {
                errors.push(format!(
                    "{rel} 残留已退役坐标几何 API `{old}`（EXP-071 S29.2 应数据驱动 `pick_key` + `on_hit`）"
                ));
            }
        }
    }
    // ③ touch.rs 坐标几何退役（EXP-071 重定基线：本模块仅留手势识别 `TouchPad`/`TouchSample`/`Gesture`）。
    let touch_src = read_file(root, "crates/ti-app/src/touch.rs").unwrap_or_default();
    for gone in [
        "LIST_Y0",
        "SLOT_Y0",
        "grid.x0",
        "grid_cell_at",
        "scene_list_bands",
        "slot_bands",
        "mod layout",
    ] {
        if touch_src.contains(gone) {
            errors.push(format!(
                "touch.rs 残留已退役坐标几何 `{gone}`（S29.2 B1：命中几何进 Bevy 拾取层，touch.rs 仅留手势识别）"
            ));
        }
    }
    if !touch_src.contains("TouchSample") {
        errors.push(
            "touch.rs 缺手势识别 `TouchSample`（S29.2：本模块供非 Bevy 壳与 headless 手势测试）"
                .to_string(),
        );
    }
    // ④ 拾取层在位：`pick.rs`（`PickTarget`/`PickEvent`）+ `scene_render`（`spawn_pick_layer`）。
    let pick_src = read_file(root, "crates/ti-engine/src/pick.rs").unwrap_or_default();
    if !pick_src.contains("PickTarget") || !pick_src.contains("PickEvent") {
        errors.push("ti-engine pick.rs 缺 `PickTarget`/`PickEvent`（S29.2 B1 拾取层）".to_string());
    }
    let scene_render = read_file(root, "crates/ti-engine/src/scene_render.rs").unwrap_or_default();
    if !scene_render.contains("spawn_pick_layer") {
        errors.push(
            "scene_render.rs 缺 `spawn_pick_layer`（S29.2：`pick_key` 原语 → Bevy 拾取标记）"
                .to_string(),
        );
    }
    errors
}

/// ①b 逐场景必需命中键（S27.5 教训固化）：场景消费面（`on_hit`）用到的键在数据面
/// 必须存在，缺 = 该键不可点。键语义 = `20_bevy_picking.md` §6 约定
/// （1=index 罗盘环徽章 / 2=列表行 / 3=存档槽 / 4..=7=主地图摇骰·买·卖·建造）。
fn check_scene_required_pick_keys(scenes: &str, errors: &mut Vec<String>) {
    const SCENE_REQUIRED_PICK_KEYS: &[(&str, &[u64])] = &[
        ("index", &[1]),
        ("select_role", &[2]),
        ("select_map", &[2]),
        ("saveload", &[3]),
        ("main_map", &[4, 5, 6, 7]),
    ];
    let Ok(json) = serde_json::from_str::<serde_json::Value>(scenes) else {
        return;
    };
    for (scene_name, required) in SCENE_REQUIRED_PICK_KEYS {
        let mut present: Vec<u64> = Vec::new();
        if let Some(prims) = json
            .get(*scene_name)
            .and_then(|s| s.get("primitives"))
            .and_then(serde_json::Value::as_array)
        {
            for prim in prims {
                if let Some(k) = prim
                    .get("params")
                    .and_then(|q| q.get("pick_key"))
                    .and_then(serde_json::Value::as_u64)
                {
                    present.push(k);
                }
            }
        }
        for key in *required {
            if !present.contains(key) {
                errors.push(format!(
                    "scenes.json `{scene_name}` 缺 `pick_key={key}`（命中数据面缺失 = 该键不可点；S27.5 教训）"
                ));
            }
        }
    }
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
    // select_role: row_h=192（S18.7d v3 中央选项列）；select_map: row_h=320（v3 右列 3 卡）；
    // saveload: row_h=256 gold_x=792（S20.2 v4 面板语言：行高加大 + 内容左移）
    // vs: player_hp 64..512 / boss_hp 576..1024 y=168 w=448 h=48
    // main_map: hud y=152 h=128（不与等距区 grid.y0=520 重叠）
    let anchors: &[(&str, &str, &str)] = &[
        ("\"index\"", "\"h\": 96", "index 标题条高度 = 96（§4.1，S17.2 8pt）"),
        (
            "\"index\"",
            "\"compass_ring\"",
            "index S22 罗盘环焦点锚 type = compass_ring（§4.1 S22，09_index_compass_spec）",
        ),
        (
            "\"index\"",
            "medallion_ring",
            "index S22 罗盘环 6 徽章 = medallion_ring 铜底原语（§4.1 S22）",
        ),
        (
            "\"index\"",
            "chart_full",
            "index S22 满屏羊皮海图底 = chart_full（§4.1 S22，与 main_map 统一）",
        ),
        ("\"index\"", "\"y\": 1840", "index 状态行 y = 1840（§4.1，S17.2 8pt）"),
        (
            "\"select_role\"",
            "\"row_h\": 192",
            "select_role 行高 = 192（§4.2，S18.7d v3 中央选项列，5 角色卡）",
        ),
        (
            "\"select_map\"",
            "\"row_h\": 320",
            "select_map 行高 = 320（§4.2，S18.7d v3 右列 3 地图卡）",
        ),
        (
            "\"saveload\"",
            "\"row_h\": 256",
            "saveload 行高 = 256（§4.6，S20.2 v4 面板语言：行高加大）",
        ),
        (
            "\"saveload\"",
            "\"gold_x\": 792",
            "saveload 金字号列 x = 792（§4.6，S20.2 v4：内容左移居中）",
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
            "main_map HUD y = 152（§4.3，S17.2 8pt，v2 回退键保留；v4 主版面 = S21.6 锚）",
        ),
        ("\"main_map\"", "\"h\": 128", "main_map HUD h = 128（§4.3，v2 回退键保留）"),
        (
            "\"main_map\"",
            "\"y0\": 520",
            "main_map 等距棋盘 grid.y0 = 520（§4.3，v2 回退业务几何非 8pt 版面令牌）",
        ),
        (
            "\"main_map\"",
            "\"hud_plaque_l\"",
            "main_map S21.6 v4 顶左铜牌 = hud_plaque_l（§4.3 S21.6，满屏海图版双铜牌）",
        ),
        (
            "\"main_map\"",
            "btn_spin",
            "main_map S21.6 v4 舵轮 CTA 原语 = btn_spin 贴图（§4.3 S21.6，大号黄铜舵轮）",
        ),
        (
            "\"main_map\"",
            "deck_plaque",
            "main_map S21.6 v4 底部三键铜牌 = deck_plaque 贴图（§4.3 S21.6）",
        ),
        (
            "\"main_map\"",
            "\"y\": 250",
            "main_map S27.4 v5 world_canvas/focus_anchor y = 250（§4.4 S27.4，环外接框，template 生成）",
        ),
        (
            "\"main_map\"",
            "\"w\": 1048",
            "main_map S27.4 v5 world_canvas w = 1048（§4.4 S27.4，环外接框；旧 v4 994 已退役）",
        ),
        (
            "\"main_map\"",
            "\"y\": 1608",
            "main_map S27.4 v5 底部三键铜牌 y = 1608（§4.4 S27.4，让位给环；旧 v4 1440 已退役）",
        ),
        (
            "\"main_map\"",
            "\"ring\"",
            "main_map S27.4 v5 环形几何数据键 main_map.ring（§4.4，数据驱动，map_pack 生成）",
        ),
        ("\"main_map\"", "island_home", "main_map S27.4 v5 中央据点岛键 island_home（§4.4）"),
        (
            "\"message\"",
            "\"h\": 176",
            "message 正文区 h = 176（§4.5，S20.2 v4 面板语言：面板按内容收缩）",
        ),
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
        let close = tail.find('}').unwrap_or(tail.len());
        if !tail[..=close].contains("\"size\":") {
            errors.push(format!(
                "scenes.json main_map `{ent}` 缺 `size` 键（QA133：§4.3 实体边长数据驱动，键缺失 = 渲染静默回退旧公式）"
            ));
        }
    }

    // 视觉回归必须掩背景（EXP-074：HD 背景 ≠ PCX 背景非排版漂移，只锁 UI 像素分布）。
    // S15.4：176×220 逐像素基线机制已暂移除（legacy 像素写帧层删除），`diff_frames_ui`/`is_bg_cur`
    // 不变量随 S15.5 重锚到 1080 代码绘制基线时重新落地；此期间门禁校验文件处于 S15.5 暂存状态。
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
        // S18.7：`primitives` 子树 = 通用原语表（数百个坐标逐值镜像不现实；
        // §4 表镜像其**语义**（kind 集 + 来源），坐标由 `scenes.json` 单一事实源）。
        let stripped = {
            let mut v = val.clone();
            if let Some(o) = v.as_object_mut() {
                o.remove("primitives");
            }
            v
        };
        collect_scalar_pairs(&stripped, &mut pairs);
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
    // 若走常量，验证常量值 == 5（B11 起允许域出口 `Character::count()` 单源派生，
    // 其编译期值仍 = `Character::all().len()` = 5，域层加角色时门禁报错引导同步）
    if last_arg.contains("ROLE_COUNT") {
        let Some(role_const_pos) = src.find("const ROLE_COUNT") else {
            errors.push("select_role.rs 引用 ROLE_COUNT 但未定义该常量".to_string());
            return errors;
        };
        let after_const = &src[role_const_pos..];
        let const_line_end = after_const.find(';').unwrap_or(0);
        let const_decl = &after_const[..=const_line_end];
        if !const_decl.contains("= 5") && !const_decl.contains("= Character::count()") {
            errors.push(format!(
                "select_role.rs ROLE_COUNT 常量值 ≠ 5（声明: `{const_decl}`），必须 == Character::all().len()（= 5，EXP-073）或域出口 Character::count()（B11）"
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
///     `ti-app/scenes/**`、`ti-app/src/touch.rs`、`ti-app/src/game_app/` 与
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
        "crates/ti-app/src/game_app",
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
        // 首列形如 "[`code.md`](code.md)" —— 判据是 `](`（旧写法 `.md](` 永不成立，
        // 该检查自落地起从未生效 → 条数漂移 code +5 / tools +1 长期无人发现，EXP-108）。
        if cols.len() != 3 || !cols[0].contains("](") {
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
    // S17.4 数据/算法分离：数值锚点（图标数/别名/绑定键、动效 4 参数）读 `assets/gate_data.json`，
    // 改数值只改数据文件；引擎（本文件）只留谓词逻辑。
    let gate_data = load_gate_data(root, &mut errors);
    if !gate_data.is_null() {
        errors.extend(ui_icon_registry_check(root, &gate_data));
        errors.extend(ui_motion_params_check(root, &gate_data));
        errors.extend(ui_palette_alias_check(root, &gate_data));
    }
    errors
}

/// 读 `assets/gate_data.json`（门禁数据文件；缺失/解析失败 = 单条 ERROR，不静默跳过）。
fn load_gate_data(root: &Path, errors: &mut Vec<String>) -> serde_json::Value {
    let Some(text) = read_file(root, "assets/gate_data.json") else {
        errors.push("缺 assets/gate_data.json（S17.4 门禁数据文件）".to_string());
        return serde_json::Value::Null;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        errors.push("assets/gate_data.json 解析失败".to_string());
        return serde_json::Value::Null;
    };
    v
}

/// `gate_data.json` 表内 `alias_map`（name → 常量名）；缺表/缺键 = 空表（不报错，走默认命名规则）。
fn gate_alias_map(val: &serde_json::Value) -> std::collections::HashMap<String, String> {
    val.get("alias_map")
        .and_then(serde_json::Value::as_object)
        .map(|m| {
            m.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect()
        })
        .unwrap_or_default()
}

/// `gate_data.json` 表内字符串数组键（缺键 = 空表）。
fn gate_str_list(val: &serde_json::Value, key: &str) -> Vec<String> {
    val.get(key)
        .and_then(serde_json::Value::as_array)
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// S17.3 图标注册表完备性（`docs/process/10_ui_redesign.md` §5；数值锚点数据化到
/// `assets/gate_data.json` 的 `ui_icons` 表，S17.4 数据/算法分离）：
/// ① `assets/data/ui_icons.json` 恰 `expect_count` 枚、id 连续、name 唯一；
/// ② 每枚 id 在 `ti-render/src/icons.rs` 有对应 `ICON_*` 分支（防注册了但没画；
///    非常规命名走 `alias_map`）；
/// ③ bindings（数组键 = `binding_array_keys`，标量键 = `binding_scalar_keys`）引用的
///    图标名都在 `icons` 内（防悬空引用）。
fn ui_icon_registry_check(root: &Path, gate_data: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(ui) = gate_data.get("ui_icons") else {
        errors.push("assets/gate_data.json 缺 ui_icons 表（S17.4 门禁数据）".to_string());
        return errors;
    };
    let Some(n) = ui.get("expect_count").and_then(serde_json::Value::as_u64) else {
        errors.push("gate_data.json ui_icons.expect_count 缺失".to_string());
        return errors;
    };
    let expect_count = u32::try_from(n).unwrap_or(0);
    let alias_map = gate_alias_map(ui);
    let binding_array_keys = gate_str_list(ui, "binding_array_keys");
    let binding_scalar_keys = gate_str_list(ui, "binding_scalar_keys");

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
    if icons.len() != expect_count as usize {
        errors.push(format!(
            "ui_icons.json icons[] = {} 条，应 {expect_count} 枚（10_ui_redesign §5，gate_data.json ui_icons.expect_count）",
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
    for (want, got) in (0..expect_count).zip(&ids) {
        if want != *got {
            errors.push(format!(
                "ui_icons.json id 序列非连续 0..{}：期望 {want} 实际 {got}",
                expect_count - 1
            ));
        }
    }
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    if sorted.len() != names.len() {
        errors.push("ui_icons.json 图标名重复".to_string());
    }
    // ② 框架侧图标分支覆盖（`match kind { ICON_X => … }`；别名走 gate_data alias_map）
    let Some(icons_rs) = read_file(root, "crates/ti-render/src/icons.rs") else {
        return errors;
    };
    for name in &names {
        let const_name =
            alias_map.get(name).cloned().unwrap_or_else(|| format!("ICON_{}", name.to_uppercase()));
        let has_const = icons_rs.contains(&format!("pub const {const_name}"));
        let has_branch = icons_rs.contains(&format!("{const_name} =>"));
        if !(has_const && has_branch) {
            errors.push(format!(
                "ui_icons.json 登记 `{name}` 但 ti_render::icons 缺 {const_name} 常量/绘制分支（S17.3：注册即须可画）"
            ));
        }
    }
    // ③ bindings 引用名在册（键清单 = gate_data.json ui_icons.binding_*_keys）
    if let Some(bindings) = parsed.get("bindings") {
        let refs = collect_binding_refs(bindings, &binding_array_keys, &binding_scalar_keys);
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

/// 收集 `bindings` 里所有引用到的图标名（数组键取每项 `icon`，标量键取 `icon`）。
fn collect_binding_refs(
    bindings: &serde_json::Value,
    array_keys: &[String],
    scalar_keys: &[String],
) -> Vec<String> {
    let mut refs: Vec<String> = Vec::new();
    for key in array_keys {
        if let Some(arr) = bindings.get(key).and_then(serde_json::Value::as_array) {
            refs.extend(
                arr.iter()
                    .filter_map(|b| b.get("icon").and_then(serde_json::Value::as_str))
                    .map(str::to_string),
            );
        }
    }
    for key in scalar_keys {
        if let Some(n) =
            bindings.get(key).and_then(|v| v.get("icon")).and_then(serde_json::Value::as_str)
        {
            refs.push(n.to_string());
        }
    }
    refs
}

/// S17.4 动效参数锁（`docs/process/10_ui_redesign.md` §7；数值锚点数据化到
/// `assets/gate_data.json` 的 `ui_motion` 表，S17.4 数据/算法分离）：
/// ① 代码侧各 `MOTION_*_MS` 常量（`gate_data.ui_motion.const_file`）== 文档 §7 表时长；
/// ② 文档 §7 表行数 == `rows` 条数、每行时长为 `NNNms`。防「改了常量没改文档」或反之。
fn ui_motion_params_check(root: &Path, gate_data: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(mo) = gate_data.get("ui_motion") else {
        errors.push("assets/gate_data.json 缺 ui_motion 表（S17.4 门禁数据）".to_string());
        return errors;
    };
    let get_str = |k: &str| mo.get(k).and_then(serde_json::Value::as_str).map(str::to_string);
    let Some(doc_file) = get_str("doc_file") else {
        errors.push("gate_data.json ui_motion.doc_file 缺失".to_string());
        return errors;
    };
    let Some(doc_section) = get_str("doc_section") else {
        errors.push("gate_data.json ui_motion.doc_section 缺失".to_string());
        return errors;
    };
    let Some(const_file) = get_str("const_file") else {
        errors.push("gate_data.json ui_motion.const_file 缺失".to_string());
        return errors;
    };
    let doc_rows: Vec<(String, String)> = mo
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|r| {
                    let a = r.as_array()?;
                    let name = a.first().and_then(serde_json::Value::as_str)?;
                    let c = a.get(1).and_then(serde_json::Value::as_str)?;
                    Some((name.to_string(), c.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    if doc_rows.is_empty() {
        errors.push("gate_data.json ui_motion.rows 缺失/为空".to_string());
        return errors;
    }

    let Some(doc) = read_file(root, &doc_file) else {
        errors.push(format!("缺 {doc_file}（ui_motion.doc_file）"));
        return errors;
    };
    // 抓 §7 表：从 doc_section 标题起到下一个 "## " 之间的 markdown 表行。
    let Some(sec_start) = doc.find(&doc_section) else {
        errors.push(format!("{doc_file} 缺 §7 动效参数表标题 `{doc_section}`"));
        return errors;
    };
    let after = &doc[sec_start..];
    let sec_end = after.find("\n## ").unwrap_or(after.len());
    let section = &after[..sec_end];
    // 文档 §7 表数据行数（`| 名 | NNNms |`）须与 gate_data rows 条数一致。
    let data_rows = section
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with('|') && t.split('|').nth(2).is_some_and(|c| c.contains("ms"))
        })
        .count();
    if data_rows != doc_rows.len() {
        errors.push(format!(
            "{doc_file} §7 表数据行 = {data_rows}，应恰好 {}（= gate_data.json ui_motion.rows 条数）",
            doc_rows.len()
        ));
    }
    let Some(chrome) = read_file(root, &const_file) else {
        errors.push(format!("缺 {const_file}（ui_motion.const_file）"));
        return errors;
    };
    // 从文档表行里按「时长」列解析 ms 值，与代码常量对账。
    for (name, const_name) in &doc_rows {
        let Some(ms) = parse_doc_ms(section, name) else {
            errors.push(format!("{doc_file} §7 缺 `{name}` 时长（ms）"));
            continue;
        };
        // 代码侧常量（朴素串解析，不引新依赖）：`{const_name}: u32 = NNN`。
        let code_ms = parse_const_ms(&chrome, const_name);
        match code_ms {
            Some(v) if v == ms => {}
            Some(v) => errors.push(format!(
                "{const_name} = {v}，但 {doc_file} §7 `{name}` = {ms}ms（S17.4：动效参数以文档表为准，须同步）"
            )),
            None => errors.push(format!("{const_file} 缺常量 {const_name}（S17.4 动效参数表）")),
        }
    }
    // 飞轮 R1 ①-04/①-06：f32 动效参数 + 帧制时长参数（独立子检查，防主函数超行）。
    errors.extend(ui_motion_f32_check(mo, section, &chrome, &const_file, &doc_file));
    errors.extend(ui_motion_tick_check(root, mo, section, &doc_file));
    errors
}

/// 飞轮 R1 ①-04：f32 动效参数锁（翻页亮度下限/幅值，登记源 =
/// `assets/gate_data.json` `ui_motion.f32_params`，§7 表值与代码常量三方对账）。
fn ui_motion_f32_check(
    mo: &serde_json::Value,
    section: &str,
    chrome: &str,
    const_file: &str,
    doc_file: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(f32_arr) = mo.get("f32_params").and_then(serde_json::Value::as_array) else {
        return errors;
    };
    let f32_doc_row = mo.get("f32_doc_row").and_then(serde_json::Value::as_str).unwrap_or("");
    let row_line: Vec<&str> = section
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with('|') && !f32_doc_row.is_empty() && t.contains(f32_doc_row)
        })
        .collect();
    let mut all_values_registered = true;
    for item in f32_arr {
        let Some(pair) = item.as_array() else { continue };
        let Some(const_name) = pair.first().and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(value_str) = pair.get(1).and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Ok(want) = value_str.parse::<f32>() else {
            errors.push(format!(
                "gate_data.json ui_motion.f32_params `{value_str}` 非合法 f32 字面量"
            ));
            continue;
        };
        let got = parse_const_f32(chrome, const_name);
        match got {
            Some(v) if (v - want).abs() < 1e-6 => {}
            Some(v) => errors.push(format!(
                "{const_name} = {v}，但 gate_data.json ui_motion.f32_params 登记 {want}（飞轮 R1 ①-04：f32 动效参数以登记表为准）"
            )),
            None => errors.push(format!("{const_file} 缺 f32 常量 {const_name}（f32_params 登记）")),
        }
        if row_line.iter().all(|l| !l.contains(value_str)) {
            all_values_registered = false;
        }
    }
    if !f32_arr.is_empty() && !all_values_registered {
        errors.push(format!(
            "{doc_file} §7 `{f32_doc_row}` 行须含 f32_params 全部登记值（逐值对账）"
        ));
    }
    errors
}

/// 飞轮 R1 ①-06：帧制时长参数锁（落格提示条 `TOAST_TICKS`，登记源 =
/// `assets/gate_data.json` `ui_motion.tick_params`，§7 表值与代码常量对账）。
fn ui_motion_tick_check(
    root: &Path,
    mo: &serde_json::Value,
    section: &str,
    doc_file: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(tick_arr) = mo.get("tick_params").and_then(serde_json::Value::as_array) else {
        return errors;
    };
    let tick_doc_row = mo.get("tick_doc_row").and_then(serde_json::Value::as_str).unwrap_or("");
    let Some(tick_file) = mo.get("tick_const_file").and_then(serde_json::Value::as_str) else {
        errors.push(
            "gate_data.json ui_motion.tick_const_file 缺失（tick_params 对账锚点）".to_string(),
        );
        return errors;
    };
    let tick_src = read_file(root, tick_file).unwrap_or_default();
    let tick_lines: Vec<&str> = section
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with('|') && !tick_doc_row.is_empty() && t.contains(tick_doc_row)
        })
        .collect();
    let mut all_tick_registered = true;
    for item in tick_arr {
        let Some(pair) = item.as_array() else { continue };
        let Some(const_name) = pair.first().and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(value_str) = pair.get(1).and_then(serde_json::Value::as_str) else {
            continue;
        };
        let got = parse_const_ms(&tick_src, const_name);
        match (got, value_str.parse::<u32>().ok()) {
            (Some(code), Some(doc)) if code == doc => {}
            (Some(code), Some(doc)) => errors.push(format!(
                "{tick_file} {const_name} = {code} ≠ gate_data.json ui_motion.tick_params 登记 {doc}（飞轮 R1 ①-06 帧制时长参数登记）"
            )),
            (Some(_), None) | (None, Some(_)) => errors.push(format!(
                "gate_data.json ui_motion.tick_params `{value_str}` 与 {tick_file} 常量 {const_name} 对账失败"
            )),
            (None, None) => {
                errors.push(format!("{tick_file} 缺常量 {const_name}（tick_params 登记）"));
            }
        }
        if tick_lines.iter().all(|l| !l.contains(value_str)) {
            all_tick_registered = false;
        }
    }
    if !tick_arr.is_empty() && !all_tick_registered {
        errors.push(format!(
            "{doc_file} §7 `{tick_doc_row}` 行须含 tick_params 登记值（帧制时长参数登记）"
        ));
    }
    errors
}

/// 文档 §7 表内某动效行的「时长」列（形如 `240ms`）→ ms 值；未找到 = None。
fn parse_doc_ms(section: &str, name: &str) -> Option<u32> {
    for line in section.lines() {
        let t = line.trim_start();
        if !t.starts_with('|') || !t.contains(name) {
            continue;
        }
        let cap = t.split('|').nth(2)?;
        let cleaned = cap.trim().strip_suffix("ms").unwrap_or_else(|| cap.trim());
        return cleaned.parse::<u32>().ok();
    }
    None
}

/// 代码文本里 `{const_name}: u32 = NNN` 的常量值（朴素串解析，不引新依赖）；缺失 = None。
fn parse_const_ms(src: &str, const_name: &str) -> Option<u32> {
    let i = src.find(const_name)?;
    let tail = &src[i..];
    let eq = tail.find('=')?;
    if !tail[..eq].contains(':') {
        return None; // 须是 `: u32 =` 常量声明而非赋值
    }
    let digits: String =
        tail[eq + 1..].trim_start().chars().take_while(char::is_ascii_digit).collect();
    digits.parse::<u32>().ok()
}

/// 代码文本里 `{const_name}: f32 = 0.6` 的常量值（朴素串解析，不引新依赖）；缺失 = None。
fn parse_const_f32(src: &str, const_name: &str) -> Option<f32> {
    let i = src.find(const_name)?;
    let tail = &src[i..];
    let eq = tail.find('=')?;
    if !tail[..eq].contains(':') {
        return None; // 须是 `: f32 =` 常量声明而非赋值
    }
    let digits: String = tail[eq + 1..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+'))
        .collect();
    digits.parse::<f32>().ok()
}

/// ① 8pt 栅格：遍历 scenes.json 各场景标量（豁免业务几何、动态落区、屏心锚点 540
/// 与 v3 场景叙事键 `title_art`/`ambient_art`/`focus_anchor`/`battle_arena`/`world_canvas`/
/// `hud_gold`/`hud_player`/`deck_trade`/`deck_sell`/`deck_build`/`canvas_entities`——
/// v3 约束由 `check_scene_narrative` 接管；飞轮 R1 数据化新键 `fx_hit`/`fx_coin`/
/// `body`/`page_dots`/`locked`/`stars`/`summary_dy` 同口径豁免）。
fn ui_8pt_walk(
    obj: &serde_json::Value,
    in_exempt: bool,
    key: &str,
    scene: &str,
    errors: &mut Vec<String>,
) {
    let exempt_by_key = matches!(
        key,
        "x0" | "y0" | "tile_w" | "tile_h" | "size" | "portrait_zones"
            // v3 场景叙事键（UI 重设计 v3.1 双写，docs/ui-redesign-v3/02 §10：v3 坐标不再 8pt 强制，
            // 约束改由 gate 第 21 项 check_scene_narrative DSL 接管；v2 键仍走 8pt 纪律）
            | "title_art" | "ambient_art" | "focus_anchor" | "hero_art"
            | "battle_arena"
            | "world_canvas" | "hud_gold" | "hud_player"
            | "deck_trade" | "deck_sell" | "deck_build" | "canvas_entities"
            // 飞轮 R1 数据化新键（FX 触发点 / 页面正文+页点几何组 / 锁块+难度星几何组 /
            // 摘要行偏移；值照抄原字面量，非 8pt 栅格排版坐标，豁免口径同 v3 叙事键）
            | "fx_hit" | "fx_coin" | "body" | "page_dots"
            | "locked" | "stars" | "summary_dy"
            // S18.7 声明式原语：非坐标语义键（z = 层序；item = 菜单项序号）不参与 8pt 栅格。
            | "z" | "item"
            // S27.4 v5 环形棋盘运行时几何（`main_map.ring.*`：环心/半径/卡宽高/据点尺寸；
            // 非 8pt 栅格排版坐标，口径同 v3 叙事键；`10_main_map_v5_spec.md` §2）
            | "ring.center_y" | "ring.radius"
            // 原语表子树整体豁免：装饰细线（1–4px）与星点尺寸本就非 8pt；
            // 元素**位置**仍按 8pt 排布（数据侧自律）。
            | "primitives"
    );
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
                          业务几何 grid.*/size、portrait_zones、屏心锚点 540 与 v3 叙事键 \
                          （title_art/ambient_art/focus_anchor/battle_arena/world_canvas/hud_gold/hud_player/\
                          deck_trade/deck_sell/deck_build/canvas_entities，ui-redesign-v3/02 §10）\
                          + 飞轮 R1 数据化键（fx_hit/fx_coin/body/page_dots/locked/stars/summary_dy）已豁免）"
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
/// 飞轮 R1 ①-12：`theme.json` 色板**同值令牌组（别名）**须显式声明在 `gate_data.palette_aliases`
/// （有意别名 = `bg` 深底族 / `wood` 木色族 / `gold` 金字族 / `gold_bright` 亮金族，各 3 令牌撞值）；
/// 出现**未声明**的同值组 = 真漂移（新令牌撞色未登记）。
fn ui_palette_alias_check(root: &Path, gate_data: &serde_json::Value) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(text) = read_file(root, "assets/data/theme.json") else {
        return errors;
    };
    let Ok(theme) = serde_json::from_str::<serde_json::Value>(&text) else {
        return errors;
    };
    let Some(palette) = theme.get("palette").and_then(|v| v.as_object()) else {
        return errors;
    };
    // 值 → 令牌名集合（同值组 = 别名族）
    let mut by_val: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for (name, val) in palette {
        by_val.entry(val.to_string()).or_default().push(name.clone());
    }
    // 声明的有意别名副集（每组成员集合化，无序比对）
    let declared: Vec<std::collections::BTreeSet<String>> = gate_data
        .get("palette_aliases")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|g| g.as_array())
                .map(|members| {
                    members.iter().filter_map(|m| m.as_str().map(String::from)).collect()
                })
                .collect()
        })
        .unwrap_or_default();
    for (val, names) in by_val.into_iter().filter(|(_, n)| n.len() > 1) {
        let set: std::collections::BTreeSet<String> = names.iter().cloned().collect();
        if !declared.contains(&set) {
            errors.push(format!(
                "theme.json 色板同值组 [{names:?}]（值 {val}）未在 gate_data.palette_aliases 声明为有意别名（飞轮 R1 ①-12：撞色须显式登记）"
            ));
        }
    }
    errors
}

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
        ("COLOR_WOOD_WARM", "wood_warm"),
        ("COLOR_GOLD_WARM", "gold_warm"),
        ("COLOR_SKY_NAVY", "sky_navy"),
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

/// 原语 `params` 校验（icon / image / grad）。
#[allow(clippy::too_many_arguments)]
fn check_primitive_params(
    root: &Path,
    kind: &str,
    pr: &serde_json::Map<String, serde_json::Value>,
    at: &str,
    tokens: &std::collections::HashSet<String>,
    icons: &std::collections::HashSet<String>,
    registered_assets: &std::collections::HashSet<String>,
) -> Vec<String> {
    let mut errors = Vec::new();
    if kind == "icon" {
        if let Some(ic) = pr.get("icon").and_then(|v| v.as_str()) {
            if !icons.contains(ic) {
                errors.push(format!(
                    "{at}: params.icon='{ic}' 不在 ui_icons.json 注册表（会静默跳过）"
                ));
            }
        }
    }
    if kind == "image" {
        errors.extend(check_image_asset(root, registered_assets, pr, at));
    }
    if kind == "grad" {
        for t in ["from_token", "to_token"] {
            if let Some(tok) = pr.get(t).and_then(|v| v.as_str()) {
                if !tokens.contains(tok) {
                    errors.push(format!("{at}: grad {t}='{tok}' 非 theme 合法令牌"));
                }
            }
        }
        if let Some(st) = pr.get("steps").and_then(serde_json::Value::as_i64) {
            if !(2..=256).contains(&st) {
                errors.push(format!("{at}: grad steps={st} 越界（2..256）"));
            }
        }
    }
    errors
}

/// 原语 `text` 校验（`text`/`text_key` 存在性 + `tier` 合法值）。
fn check_primitive_text(
    obj: &serde_json::Map<String, serde_json::Value>,
    at: &str,
    strings: &serde_json::Value,
) -> Vec<String> {
    let mut errors = Vec::new();
    let tk = obj.get("text_key").and_then(|v| v.as_str()).unwrap_or("");
    let tx = obj.get("text").and_then(|v| v.as_str()).unwrap_or("");
    if tk.is_empty() && tx.is_empty() {
        errors.push(format!("{at}: text 缺 text/text_key（空内容不画）"));
    }
    if !tk.is_empty() && strings.get(tk).is_none() {
        errors.push(format!("{at}: text_key='{tk}' 不在 strings.json（会显示裸 key）"));
    }
    match obj.get("tier").and_then(|v| v.as_str()).unwrap_or("") {
        "title" | "body" | "hint" => {}
        other => {
            errors.push(format!("{at}: tier='{other}' 非法（仅 title/body/hint）"));
        }
    }
    errors
}

/// S18.8b：`image` 原语的资产校验 —— 文件真实存在 **且** 已登记在资产清单
/// （`assets/ui_manifest.json` = 资产单一事实源）。
///
/// 缺资源 = 静默不画（画面缺件却不报错）→ 前置拦截；未登记 = 资产清单与实际用图脱节。
fn check_image_asset(
    root: &Path,
    registered: &std::collections::HashSet<String>,
    pr: &serde_json::Map<String, serde_json::Value>,
    at: &str,
) -> Vec<String> {
    match pr.get("asset").and_then(|v| v.as_str()) {
        None => vec![format!("{at}: image 原语缺 params.asset")],
        Some("") => vec![format!("{at}: image 原语 params.asset 为空")],
        Some(a) => {
            if !root.join(a).is_file() {
                vec![format!("{at}: image 资源 '{a}' 不存在（相对仓库根；缺资源 = 静默不画）")]
            } else if !registered.contains(a) {
                vec![format!(
                    "{at}: image 资源 '{a}' 未在 assets/ui_manifest.json 登记（资产清单 = 单一事实源）"
                )]
            } else {
                Vec::new()
            }
        }
    }
}

/// S18.7g：声明式原语表数据合法性（`scenes.json` `primitives[]`）。
///
/// 通用原语只有 7 种（rect/text/icon/grad/glow/disc/image）；数据侧不变式（`09_render_framework.md` §5.1）：
/// - `color` 必须是 `theme.by_name` 合法令牌名 —— 拼错 = **静默回退** `text_on_parch`；
/// - `params.icon` 必须在 `ui_icons.json` 注册表 —— 拼错 = **静默跳过**；
/// - `text_key` 必须在 `strings.json` —— 缺键 = 界面显示**裸 key**（非静默失败）；
/// - `grad` 的 `from_token`/`to_token` 合法 + `steps` ∈ 2..256；`text` 的 `tier` ∈ title/body/hint。
///
/// 动机：S18.7d~f 七场景 517 条原语落库后，人工自审发现此类错误**不报错**（只改画面不崩），
/// 故固化为门禁，防后续 JSON 编辑引入静默漂移。
fn check_primitives_data(root: &Path) -> Vec<String> {
    const KINDS: [&str; 7] = ["rect", "text", "icon", "grad", "glow", "disc", "image"];
    let mut errors = Vec::new();
    let Some(json_text) = read_file(root, "assets/data/scenes.json") else {
        return vec!["assets/data/scenes.json 不存在，无法校验原语表".to_string()];
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_text) else {
        return vec!["assets/data/scenes.json 解析失败".to_string()];
    };
    let theme_src = read_file(root, "crates/ti-render/src/theme.rs").unwrap_or_default();
    let tokens: std::collections::HashSet<String> = theme_src
        .lines()
        .filter_map(|l| {
            let k = l.split_once("=> self.")?.0.trim();
            let name = k.strip_prefix('"')?;
            let name = name.split_once('"')?.0;
            Some(name.to_string())
        })
        .collect();
    let icons: std::collections::HashSet<String> = read_file(root, "assets/data/ui_icons.json")
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| {
            v.get("icons").and_then(|i| i.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|i| i.get("name").and_then(|n| n.as_str()).map(str::to_string))
                    .collect()
            })
        })
        .unwrap_or_default();
    let strings: serde_json::Value = read_file(root, "assets/data/strings.json")
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(serde_json::Value::Null);
    // S18.8b：资产清单（`assets/ui_manifest.json` = 资产单一事实源）→ 已登记的资产路径集合。
    let registered_assets: std::collections::HashSet<String> =
        read_file(root, "assets/ui_manifest.json")
            .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
            .and_then(|v| {
                v.get("assets").and_then(|a| a.as_array()).map(|arr| {
                    arr.iter()
                        .filter_map(|a| a.get("file").and_then(|f| f.as_str()).map(str::to_string))
                        .collect()
                })
            })
            .unwrap_or_default();
    for (sid, scene) in json.as_object().into_iter().flatten() {
        let Some(prims) = scene.get("primitives").and_then(|p| p.as_array()) else {
            continue;
        };
        for (i, p) in prims.iter().enumerate() {
            let Some(obj) = p.as_object() else { continue };
            let kind = obj.get("kind").and_then(|k| k.as_str()).unwrap_or("");
            let at = format!("{sid}.primitives[{i}]");
            if !KINDS.contains(&kind) {
                errors.push(format!(
                    "{at}: kind='{kind}' 非通用原语（仅 rect/text/icon/grad/glow/disc/image）"
                ));
                continue;
            }
            if let Some(c) = obj.get("color").and_then(|c| c.as_str()) {
                if !c.is_empty() && !tokens.contains(c) {
                    errors.push(format!(
                        "{at}: color='{c}' 非 theme.by_name 合法令牌（会静默回退 text_on_parch）"
                    ));
                }
            }
            if let Some(pr) = obj.get("params").and_then(|v| v.as_object()) {
                errors.extend(check_primitive_params(
                    root,
                    kind,
                    pr,
                    &at,
                    &tokens,
                    &icons,
                    &registered_assets,
                ));
            }
            if kind == "text" {
                errors.extend(check_primitive_text(obj, &at, &strings));
            }
        }
    }
    errors
}

/// S18.9：地形表数据合法性（`assets/data/terrain.json`）。
///
/// - 每图 `rows` 行数 == `balance.json` 该图 `grid_h`，每行长度 == `grid_w`；
/// - 地形码 ⊆ 全局 `legend` 键；
/// - 图例的 `token`/`edge`/`alt` 必须是 `theme.json` palette 里的合法令牌
///   （拼错 = 静默回退 `menu_bg`，画面变单色但不报错）。
fn check_terrain_data(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(terrain_text) = read_file(root, "assets/data/terrain.json") else {
        return vec!["assets/data/terrain.json 不存在，无法校验地形表".to_string()];
    };
    let Ok(terrain) = serde_json::from_str::<serde_json::Value>(&terrain_text) else {
        return vec!["assets/data/terrain.json 解析失败".to_string()];
    };
    let balance = read_file(root, "assets/data/balance.json")
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .unwrap_or(serde_json::Value::Null);
    let theme = read_file(root, "assets/data/theme.json")
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .unwrap_or(serde_json::Value::Null);
    let palette = theme.get("palette").cloned().unwrap_or(serde_json::Value::Null);
    let legend = terrain.get("legend").and_then(|l| l.as_object()).cloned().unwrap_or_default();
    if legend.is_empty() {
        errors.push("terrain.json: legend 不得为空".to_string());
    }
    for (code, lg) in &legend {
        for key in ["token", "edge"] {
            if let Some(tok) = lg.get(key).and_then(|v| v.as_str()) {
                if palette.get(tok).is_none() {
                    errors.push(format!(
                        "terrain.json legend.{code}.{key}='{tok}' 非 theme.json palette 令牌（会静默回退）"
                    ));
                }
            }
        }
        if let Some(alt) = lg.get("alt").and_then(|v| v.as_str()) {
            if palette.get(alt).is_none() {
                errors.push(format!(
                    "terrain.json legend.{code}.alt='{alt}' 非 theme.json palette 令牌（会静默回退）"
                ));
            }
        }
    }
    let maps = terrain.get("maps").and_then(|m| m.as_object()).cloned().unwrap_or_default();
    for (id, rows_v) in &maps {
        let rows: Vec<String> = rows_v
            .get("rows")
            .and_then(|r| r.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
            .unwrap_or_default();
        let spec = balance
            .get("selection")
            .and_then(|s| s.get("maps"))
            .and_then(|m| m.as_array())
            .and_then(|a| {
                a.iter().find(|m| m.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
            });
        let (Some(spec), false) = (spec, rows.is_empty()) else {
            if rows.is_empty() {
                errors.push(format!("terrain.json maps.{id}: rows 不得为空"));
            }
            continue;
        };
        let gw = spec.get("grid_w").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let gh = spec.get("grid_h").and_then(serde_json::Value::as_i64).unwrap_or(0);
        let n_rows = i64::try_from(rows.len()).unwrap_or(-1);
        if n_rows != gh {
            errors
                .push(format!("terrain.json maps.{id}: rows 行数 {n_rows} != balance grid_h {gh}"));
        }
        for (i, row) in rows.iter().enumerate() {
            let n_chars = i64::try_from(row.chars().count()).unwrap_or(-1);
            if n_chars != gw {
                errors.push(format!(
                    "terrain.json maps.{id}.rows[{i}]: 长度 {n_chars} != balance grid_w {gw}"
                ));
            }
            for (c, ch) in row.chars().enumerate() {
                if !legend.contains_key(&ch.to_string()) {
                    errors.push(format!(
                        "terrain.json maps.{id}.rows[{i}][{c}]: 地形码 '{ch}' 不在 legend"
                    ));
                }
            }
        }
    }
    errors
}

/// 26. **工具登记对账**（S20.1，EXP-113）
///
/// `scripts/` 下每个工具（`.py`/`.ps1`）必须在
/// `docs/process/14_tooling_catalog.md`（工具唯一事实源）里出现 —— 新增工具漏登记 ⇒ FAIL。
///
/// 口径：只查 `scripts/` 顶层（`legacy/scripts/` 是取证工具，登记在 14 台账的 §3.4 段）。
fn check_tooling_catalog(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(catalog) = read_file(root, "docs/process/14_tooling_catalog.md") else {
        return vec!["docs/process/14_tooling_catalog.md 不存在，无法校验工具登记".to_string()];
    };
    let scripts_dir = root.join("scripts");
    let Ok(entries) = std::fs::read_dir(&scripts_dir) else {
        return vec!["scripts/ 目录不存在，无法校验工具登记".to_string()];
    };
    let mut tools: Vec<String> = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let ext = std::path::Path::new(&name)
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase);
        if matches!(ext.as_deref(), Some("py" | "ps1")) && !name.starts_with("__") {
            tools.push(name);
        }
    }
    tools.sort();
    for t in &tools {
        if !catalog.contains(t.as_str()) {
            errors.push(format!(
                "scripts/{t} 未登记到 docs/process/14_tooling_catalog.md（工具唯一事实源；\
                 新增工具须：幂等 + --check + 非 0 退出 + 台账登记）"
            ));
        }
    }
    errors
}

/// 第 27 项：**工具契约**（把「脚本纪律」从文档约定变成机器红线）。
///
/// 动机（EXP-113 复发 + S21 实证）：工具缺位 ⇒ 同一脚本被写多遍；一次性脚本写到
/// **仓库外**临时目录 ⇒ 完全绕开 `scripts/` 台账登记，事后无人能查。本检查锁四条：
/// ① **台账登记必须是表格行**且含「幂等/只读」与「退出码」要素（第 26 项只查「出现过」）；
/// ② **新增工具**（不在棘轮基线 `scripts/tooling_contract_baseline.json` 内）必须自带
///    **只读模式** + **非 0 退出码**（存量豁免见基线表，每条带整改任务号）；
/// ③ **一次性脚本落地位置** = `scripts/tmp/`，任务结束必须清空（残留即 FAIL）；
/// ④ 仓库内（`scripts/`、`legacy/`、`.opencode/`、`target/` 之外）**不得散落** `.py`/`.ps1`。
fn check_tooling_contract(root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let Some(catalog) = read_file(root, "docs/process/14_tooling_catalog.md") else {
        return vec!["docs/process/14_tooling_catalog.md 不存在，无法校验工具契约".to_string()];
    };
    let scripts_dir = root.join("scripts");
    let Ok(entries) = std::fs::read_dir(&scripts_dir) else {
        return vec!["scripts/ 目录不存在，无法校验工具契约".to_string()];
    };
    let classes = tooling_catalog_classes(&catalog);
    let waivers = tooling_contract_waivers(root, &mut errors);
    let mut tools: Vec<(String, String)> = Vec::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !is_tool_script(&name) {
            continue;
        }
        let src = std::fs::read_to_string(e.path()).unwrap_or_default();
        tools.push((name, src));
    }
    tools.sort();
    for (name, src) in &tools {
        errors.extend(check_tooling_contract_one(name, src, &catalog, &classes, &waivers));
    }
    errors.extend(check_tooling_tmp_dir(&scripts_dir));
    errors.extend(check_tooling_stray_scripts(root));
    errors
}

/// 台账分类：§3.1 门禁项 / §3.2 长期工具 / §3.3 一次性 / §3.4 取证（分类决定登记要求）。
fn tooling_catalog_classes(catalog: &str) -> BTreeMap<String, String> {
    let mut class_of: BTreeMap<String, String> = BTreeMap::new();
    let mut cur = String::new();
    for line in catalog.lines() {
        let t = line.trim_start();
        if t.starts_with("### 3.1") {
            cur = "gate".to_string();
        } else if t.starts_with("### 3.2") {
            cur = "longterm".to_string();
        } else if t.starts_with("### 3.3") {
            cur = "oneshot".to_string();
        } else if t.starts_with("### 3.4") {
            cur = "forensic".to_string();
        } else if t.starts_with("## 4") {
            cur = String::new();
        }
        if cur.is_empty() {
            continue;
        }
        if let Some(rest) = t.strip_prefix('|') {
            if let Some(cell) = rest.split('|').next() {
                let key = cell.trim().trim_matches('`');
                if !key.is_empty() && key != "工具" && !key.starts_with('-') {
                    class_of.insert(key.to_string(), cur.clone());
                }
            }
        }
    }
    class_of
}

/// 棘轮基线豁免表（存量工具豁免项；新增工具不得入表）。
fn tooling_contract_waivers(
    root: &Path,
    errors: &mut Vec<String>,
) -> BTreeMap<String, Vec<String>> {
    let mut waived: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let Some(baseline) = read_file(root, "scripts/tooling_contract_baseline.json") else {
        errors.push(
            "scripts/tooling_contract_baseline.json 不存在（工具契约棘轮基线；缺它则存量豁免不可查）"
                .to_string(),
        );
        return waived;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&baseline) else {
        errors.push("scripts/tooling_contract_baseline.json 解析失败（须为合法 JSON）".to_string());
        return waived;
    };
    if let Some(obj) = v.get("tools").and_then(|t| t.as_object()) {
        for (name, entry) in obj {
            let list = entry
                .get("waived")
                .and_then(|w| w.as_array())
                .map(|a| {
                    a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect::<Vec<String>>()
                })
                .unwrap_or_default();
            waived.insert(name.clone(), list);
        }
    }
    waived
}

/// 工具脚本判定（`scripts/` 下的 `.py` / `.ps1`；`__` 开头 = 私有模块不计）。
fn is_tool_script(name: &str) -> bool {
    let ext = std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    matches!(ext.as_deref(), Some("py" | "ps1")) && !name.starts_with("__")
}

/// 单个工具的契约判定（台账表格行 / 登记要素 / 只读模式 / 非 0 退出码）。
fn check_tooling_contract_one(
    name: &str,
    src: &str,
    catalog: &str,
    classes: &BTreeMap<String, String>,
    waivers: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let key = format!("| `{name}` |");
    let Some(row) = catalog.lines().find(|l| l.trim_start().starts_with('|') && l.contains(&key))
    else {
        errors.push(format!(
            "scripts/{name}: 未登记为台账表格行（| `{name}` | 形式）——工具唯一事实源 = \
             docs/process/14_tooling_catalog.md §3"
        ));
        return errors;
    };
    let class = classes.get(name).map_or("unknown", String::as_str);
    // 一次性 / 取证工具：不做「幂等 + 退出码」声明要求（§1.1 分类决定登记要求）。
    let strict = matches!(class, "gate" | "longterm");
    let exemptions = waivers.get(name).cloned().unwrap_or_default();
    let exempts = |id: &str| exemptions.iter().any(|w| w == id);
    if strict && !row.contains("幂等") && !row.contains("只读") && !exempts("decl_idempotent") {
        errors.push(format!(
            "scripts/{name}: 台账行未声明「幂等/只读」（§3 表格第 4 列 = 幂等与退出码）"
        ));
    }
    if strict
        && !row.contains("退出")
        && !row.contains("非 0")
        && !row.contains("非零")
        && !exempts("decl_nonzero")
    {
        errors.push(format!(
            "scripts/{name}: 台账行未声明退出码语义（须写明「非 0 退出」或等价表述）"
        ));
    }
    if !strict {
        return errors;
    }
    if !exempts("readonly_mode") {
        const READONLY: [&str; 7] =
            ["--check", "-Check", "--verify", "--report", "--list", "--json", "--dry-run"];
        if !READONLY.iter().any(|pat| src.contains(pat)) {
            errors.push(format!(
                "scripts/{name}: 缺只读模式（须提供 --check/--verify/--report 之一，\
                 见 14_tooling_catalog.md §1.2 工具契约）"
            ));
        }
    }
    if !exempts("nonzero_exit_code") {
        const NONZERO: [&str; 7] = [
            "sys.exit(1",
            "sys.exit(2",
            "raise SystemExit(1",
            "return 1",
            "else 1",
            "exit 1",
            "ExitCode::FAILURE",
        ];
        if !NONZERO.iter().any(|pat| src.contains(pat)) {
            errors.push(format!("scripts/{name}: 缺非 0 退出码（违规时须非 0 退出）"));
        }
    }
    errors
}

/// 一次性脚本落地位置：`scripts/tmp/` 任务结束必须清空。
fn check_tooling_tmp_dir(scripts_dir: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let tmp_dir = scripts_dir.join("tmp");
    if !tmp_dir.is_dir() {
        return errors;
    }
    let Ok(tmp_entries) = std::fs::read_dir(&tmp_dir) else {
        return errors;
    };
    for e in tmp_entries.flatten() {
        let fname = e.file_name().to_string_lossy().to_string();
        if fname != "README.md" && fname != ".gitkeep" {
            errors.push(format!(
                "scripts/tmp/{fname}: 一次性脚本残留（落地位置 = scripts/tmp/，\
                 任务结束必须清理；反复要用的必须提升为 scripts/ 工具并登记台账）"
            ));
        }
    }
    errors
}

/// 仓库内不得散落脚本（`scripts/` 与 `legacy/` 之外）。
fn check_tooling_stray_scripts(root: &Path) -> Vec<String> {
    const SKIP_DIRS: [&str; 7] =
        ["scripts", "legacy", ".opencode", ".git", "target", "node_modules", "web"];
    fn walk(dir: &Path, skip: &[&str], out: &mut Vec<String>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if !skip.contains(&name.as_str()) {
                    walk(&path, skip, out);
                }
            } else {
                let ext = path.extension().and_then(|x| x.to_str()).map(str::to_ascii_lowercase);
                if matches!(ext.as_deref(), Some("py" | "ps1")) {
                    out.push(path.to_string_lossy().to_string());
                }
            }
        }
    }
    let mut stray: Vec<String> = Vec::new();
    walk(root, &SKIP_DIRS, &mut stray);
    stray
        .into_iter()
        .map(|s| {
            format!(
                "{s}: 仓库内散落脚本（工具落 scripts/ 并登记台账；一次性改写脚本落 scripts/tmp/）"
            )
        })
        .collect()
}

/// 飞轮 R1 ⑥-01/③-17/③-18：`ti-engine` 渲染层禁裸 540.0/960.0（屏心半值字面量）——
/// 一律经 `LOGICAL_W_I32`/`LOGICAL_H_I32`（`ti_render::framebuffer` 单一事实源）派生；
/// 双守卫：① 裸字面量 0 命中；② 派生式 `as f32 / 2.0` 在位（防「全删了也没派生」假绿）。
fn check_engine_no_bare_half(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    for file in ["crates/ti-engine/src/canvas.rs", "crates/ti-engine/src/bin/treasure_bevy.rs"] {
        let Some(src) = read_file(root, file) else {
            errors.push(format!("{file} 不存在，无法校验 1080 半值派生"));
            continue;
        };
        for needle in ["540.0", "960.0"] {
            let mut start = 0usize;
            while let Some(rel) = src[start..].find(needle) {
                let abs = start + rel;
                let line_start = src[..abs].rfind('\n').map_or(0, |p| p + 1);
                let in_comment = src[line_start..abs].contains("//");
                if !in_comment {
                    errors.push(format!(
                        "{file}: 裸 {needle} 字面量（1080 单源契约：屏心锚点须 LOGICAL_W_I32/LOGICAL_H_I32 半值派生）"
                    ));
                }
                start = abs + needle.len();
            }
        }
        if !src.contains("as f32 / 2.0") {
            errors.push(format!("{file}: 缺 `as f32 / 2.0` 半值派生（飞轮 R1 ③-17 归一口径）"));
        }
    }
    errors
}

/// 飞轮 R1 ⑥-07：`select_role.rs` 渲染循环须走 `ROLE_COUNT` 命名常量
/// （EXP-073 band 数门禁只锁 `scene_list_bands` 入参，render 循环的字面量 5 是其盲区）。
fn check_select_role_render_loop(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let src = read_file(root, "crates/ti-app/src/scenes/select_role.rs").unwrap_or_default();
    if src.contains("0..5") {
        errors.push(
            "select_role.rs 仍含 `0..5` 字面量循环（EXP-073 盲区：须 0..ROLE_COUNT 单源）"
                .to_string(),
        );
    }
    if !src.contains("0..ROLE_COUNT") {
        errors.push(
            "select_role.rs 缺 `0..ROLE_COUNT`（render 循环角色数须与 band 门禁同单源）"
                .to_string(),
        );
    }
    errors
}

/// 飞轮 R1 ⑥-02/⑥-11 单源对账：
/// ① `ui_icons.json default_size` == `ti_render::icons::ICON_SIZE`（图标缺省边长双端恒等）；
/// ② `tests/scripts/full_run.json seed` == `treasure_bevy.rs` 的 `SELFTEST_SEED` 命名常量
/// （S5.2 同源固定种子：改 seed 忘改另一处 = 出图/自检静默不同源）。
fn check_render_single_source(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    // ① 图标缺省边长
    let icons_json = read_file(root, "assets/data/ui_icons.json").unwrap_or_default();
    let ui_json: serde_json::Value =
        serde_json::from_str(&icons_json).unwrap_or(serde_json::Value::Null);
    if let Some(ds) = ui_json.get("default_size").and_then(serde_json::Value::as_i64) {
        let icons_rs = read_file(root, "crates/ti-render/src/icons.rs").unwrap_or_default();
        if let Some(cap) = icons_rs.find("pub const ICON_SIZE: i32 = ").and_then(|i| {
            icons_rs[i + "pub const ICON_SIZE: i32 = ".len()..].split_whitespace().next()
        }) {
            if let Ok(code_v) = cap.parse::<i64>() {
                if code_v != ds {
                    errors.push(format!(
                        "ui_icons.json default_size = {ds} ≠ ti_render::icons ICON_SIZE = {code_v}（⑥-02 图标缺省边长双端须恒等）"
                    ));
                }
            }
        } else {
            errors.push(
                "ti_render/src/icons.rs 缺 `pub const ICON_SIZE`（⑥-02 框架侧单源）".to_string(),
            );
        }
    } else {
        errors.push(
            "ui_icons.json 缺顶层 `default_size` 键（⑥-02 图标缺省边长数据单源）".to_string(),
        );
    }
    // ② 自检种子
    let full_run = read_file(root, "tests/scripts/full_run.json").unwrap_or_default();
    let fr: serde_json::Value = serde_json::from_str(&full_run).unwrap_or(serde_json::Value::Null);
    if let Some(seed_json) = fr.get("seed").and_then(serde_json::Value::as_i64) {
        let bevy_bin =
            read_file(root, "crates/ti-engine/src/bin/treasure_bevy.rs").unwrap_or_default();
        let code_seed = bevy_bin
            .find("const SELFTEST_SEED: u32 = ")
            .and_then(|i| {
                bevy_bin[i + "const SELFTEST_SEED: u32 = ".len()..]
                    .split(|c: char| !c.is_ascii_digit())
                    .next()
            })
            .and_then(|s| s.parse::<i64>().ok());
        match code_seed {
            Some(v) if v == seed_json => {}
            Some(v) => errors.push(format!(
                "treasure_bevy.rs SELFTEST_SEED = {v} ≠ tests/scripts/full_run.json seed = {seed_json}（⑥-11 同源固定种子对账）"
            )),
            None => errors.push(
                "treasure_bevy.rs 缺 `SELFTEST_SEED` 命名常量（⑥-11：种子须单源命名）".to_string(),
            ),
        }
    } else {
        errors.push("tests/scripts/full_run.json 缺 `seed` 键（⑥-11 对账锚点）".to_string());
    }
    errors
}

/// 第 31 项：**仓库文本完整性**（BOM 策略 / UTF-8 合法 / 无乱码替换字符）。
///
/// 动机（EXP-051/091/111 + S21 飞轮 R1 实证）：文本文件被 cmdlet（`Set-Content`/`Add-Content`/
/// `Out-File`）写入 ⇒ 带 BOM（PS 5.1 `UTF8` 默认）或换行被改写 ⇒ 文档/源码被静默改写。
/// EXP-091 更严重：`doccheck` 读文件失败就静默跳过 ⇒ 坏字节文件整份从检查视野消失。
/// 本项把「写入路径正确性」变成机器红线。
///
/// 规则：① `.ps1` **必须**带 UTF-8 BOM（PS 5.1 解析 CJK 需要，EXP-109）；
/// ② 其余文本文件（`.md/.json/.rs/.toml/.py/.yaml/.yml/.txt`）**禁止**带 BOM；
/// ③ 全部须为合法 UTF-8；④ 不得含 U+FFFD 替换字符（已损坏的痕迹）。
/// 第 31 项：**仓库文本完整性**（BOM 策略 / UTF-8 合法 / 无乱码替换字符）。
///
/// 动机（EXP-051/091/111 + S21 飞轮 R1 实证）：文本文件被 cmdlet（`Set-Content`/`Add-Content`/
/// `Out-File`）写入 ⇒ 带 BOM（PS 5.1 `UTF8` 默认）或换行被改写 ⇒ 文档/源码被静默改写。
/// EXP-091 更严重：`doccheck` 读文件失败就静默跳过 ⇒ 坏字节文件整份从检查视野消失。
/// 本项把「写入路径正确性」变成机器红线。
///
/// 规则：① `.ps1` **必须**带 UTF-8 BOM（PS 5.1 解析 CJK 需要，EXP-109）；
/// ② 其余文本文件（`.md/.json/.rs/.toml/.py/.yaml/.yml/.txt`）**禁止**带 BOM；
/// ③ 全部须为合法 UTF-8；④ 不得含 U+FFFD 替换字符（已损坏的痕迹）。
fn check_repo_text_integrity(root: &Path) -> Vec<String> {
    const TEXT_EXTS: [&str; 8] = ["md", "json", "rs", "toml", "py", "yaml", "yml", "txt"];
    const SKIP_DIRS: [&str; 6] = ["target", "legacy", ".git", ".opencode", "node_modules", "web"];
    const SCAN_ROOTS: [&str; 6] = ["docs", "crates", "scripts", "assets", "plan", "android"];
    let mut errors: Vec<String> = Vec::new();
    for r in SCAN_ROOTS {
        let dir = root.join(r);
        if dir.is_dir() {
            walk_text_files(&dir, &SKIP_DIRS, &TEXT_EXTS, &mut errors);
        }
    }
    if let Ok(rd) = std::fs::read_dir(root) {
        for e in rd.flatten() {
            let path = e.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|x| x.to_str())
                .map(str::to_ascii_lowercase)
                .unwrap_or_default();
            if !TEXT_EXTS.contains(&ext.as_str()) {
                continue;
            }
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let rel = path.to_string_lossy().to_string();
            if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                errors.push(format!("{rel}: 根级文本文件带 UTF-8 BOM（EXP-111）"));
            }
            if let Ok(text) = std::str::from_utf8(&bytes) {
                if text.contains('\u{FFFD}') {
                    errors.push(format!("{rel}: 含 U+FFFD 替换字符（EXP-091）"));
                }
            }
        }
    }
    errors
}

/// 递归扫文本文件并校验编码完整性（`check_repo_text_integrity` 的目录遍历体）。
fn walk_text_files(dir: &Path, skip: &[&str], exts: &[&str], errors: &mut Vec<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let path = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if !skip.contains(&name.as_str()) {
                walk_text_files(&path, skip, exts, errors);
            }
            continue;
        }
        let ext = path
            .extension()
            .and_then(|x| x.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        if !exts.contains(&ext.as_str()) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let rel = path.to_string_lossy().to_string();
        let has_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
        if ext == "ps1" {
            if !has_bom {
                errors.push(format!(
                    "{rel}: .ps1 缺 UTF-8 BOM（PowerShell 5.1 解析 CJK 需要；EXP-109）"
                ));
            }
        } else if has_bom {
            errors.push(format!(
                "{rel}: 文本文件带 UTF-8 BOM（cmdlet 写入痕迹；改用编辑工具或 python 写盘，EXP-111/051）"
            ));
        }
        match std::str::from_utf8(&bytes) {
            Ok(text) => {
                if text.contains('\u{FFFD}') {
                    errors.push(format!(
                        "{rel}: 含 U+FFFD 替换字符（文本已损坏；须修内容而非忽略，EXP-091）"
                    ));
                }
            }
            Err(e) => errors.push(format!(
                "{rel}: 非合法 UTF-8（字节偏移 {}）—— 检查视野会静默丢整份文件（EXP-091）",
                e.valid_up_to()
            )),
        }
    }
}

/// 第 32 项：**改动面声明**（S20+ 任务 dod 必含「改动面=」+ 点名面族）。
///
/// 动机（`00_index.md` §变更面声明纪律 + 硬规则 7「不声明 = 返工」）：纪律过去只写在文档里，
/// 无人判定 ⇒ 台账 200 任务仅 22 处含声明。本项把「声明」变成台账行级机器要求。
fn check_task_change_scope(root: &Path) -> Vec<String> {
    // 声明族（措辞变体：表现/表现层、工具/工具层、业务规则/业务层…）；「规则」族先剔除
    // 「业务规则」以免同一处声明被重复计数。
    const FAMILIES: [&[&str]; 7] =
        [&["业务规则", "业务层"], &["数据"], &["规则"], &["表现"], &["资产"], &["工具"], &["文档"]];
    let mut errors: Vec<String> = Vec::new();
    let Some(ledger) = read_file(root, "plan/ledger.yaml") else {
        return vec!["plan/ledger.yaml 不存在，无法校验改动面声明".to_string()];
    };
    for line in ledger.lines() {
        let Some(rest) = line.trim_start().strip_prefix("- { id: ") else {
            continue;
        };
        let Some((id_raw, _)) = rest.split_once(',') else {
            continue;
        };
        let id = id_raw.trim();
        let major: u32 = id
            .strip_prefix('S')
            .and_then(|r| r.split('.').next())
            .and_then(|m| m.parse().ok())
            .unwrap_or(0);
        // 纪律自 S20 起强制（此前任务无此声明；S19 及更早为存量）
        if major < 20 {
            continue;
        }
        let Some(dod) = line.split("dod: \"").nth(1) else {
            continue;
        };
        let dod = dod.split('"').next().unwrap_or("");
        // 父任务（dod = 「见 S20.0a / …」）不重复要求声明
        if dod.starts_with("见 ") {
            continue;
        }
        if !dod.contains("改动面=") {
            errors.push(format!(
                "ledger.yaml {id}: dod 缺「改动面=」声明（业务规则/数据/规则/表现/资产 + 跨模块原因；\
                 00_index.md §变更面声明，硬规则 7）"
            ));
            continue;
        }
        // 只在声明段内计数（改动面= … ；跨模块原因 / ；判据 之前）
        let seg = dod.split_once("改动面=").map_or("", |(_, rest)| rest);
        let seg = seg.split("；跨模块原因").next().unwrap_or(seg);
        let seg = seg.split("；判据").next().unwrap_or(seg);
        let seg_no_biz = seg.replace("业务规则", "");
        let mut hit = 0usize;
        for fam in FAMILIES {
            let found = fam.iter().any(|pat| {
                if *pat == "规则" { seg_no_biz.contains(pat) } else { seg.contains(pat) }
            });
            if found {
                hit += 1;
            }
        }
        // 注：只要求**声明存在且点名了面族**（≥1）。「是否漏报了别的面」不可机器判定 ⇒ 人工评审。
        // 纯单面任务（如 S20.0b 表现层架构重构）声明 1 个面族是合法的。
        if hit < 1 {
            errors.push(format!(
                "ledger.yaml {id}: 改动面声明未点名任何面族（业务规则/数据/规则/表现/资产/工具/文档，\
                 至少 1 个；纯单面任务声明 1 个面族合法——「是否漏报别的面」不可机器判定，人工评审）"
            ));
        }
    }
    errors
}

/// 第 33 项：**新增工具 3 处登记**（台账表行 + 速查表 + 全库索引），存量走棘轮基线豁免。
///
/// 动机（EXP-113 + S20.1/S21.0 dod 自认「五处/八处同改」）：第 26 项只查「工具名出现在台账里」，
/// 于是「工具存在但速查表/索引没人知道」照旧发生。本项对新工具零豁免（棘轮只收紧）。
fn check_tooling_registration(root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let Some(catalog) = read_file(root, "docs/process/14_tooling_catalog.md") else {
        return vec!["docs/process/14_tooling_catalog.md 不存在，无法校验工具登记".to_string()];
    };
    let quick = read_file(root, "docs/process/04_tooling.md").unwrap_or_default();
    let index = read_file(root, "docs/README.md").unwrap_or_default();
    let mut waived: BTreeMap<String, Vec<String>> = BTreeMap::new();
    if let Some(base) = read_file(root, "scripts/process_ratchet.json") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&base) {
            if let Some(obj) =
                v.get("tooling_registration_grandfathered").and_then(|t| t.as_object())
            {
                for (name, entry) in obj {
                    let list = entry
                        .get("missing")
                        .and_then(|m| m.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(str::to_string))
                                .collect::<Vec<String>>()
                        })
                        .unwrap_or_default();
                    waived.insert(name.clone(), list);
                }
            }
        }
    }
    let scripts_dir = root.join("scripts");
    let Ok(entries) = std::fs::read_dir(&scripts_dir) else {
        return vec!["scripts/ 目录不存在，无法校验工具登记".to_string()];
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        let ext = std::path::Path::new(&name)
            .extension()
            .and_then(|x| x.to_str())
            .map(str::to_ascii_lowercase);
        if !matches!(ext.as_deref(), Some("py" | "ps1")) || name.starts_with("__") {
            continue;
        }
        let missing = waived.get(&name).cloned().unwrap_or_default();
        let row = format!("| `{name}` |");
        if !catalog.contains(&row) {
            errors.push(format!(
                "scripts/{name}: 缺台账表行（14_tooling_catalog §3 须有 {row} 形式）"
            ));
        }
        for (label, hay) in [("04_tooling.md", &quick), ("docs/README.md", &index)] {
            if missing.iter().any(|m| m == label) {
                continue;
            }
            if !hay.contains(&format!("scripts/{name}")) {
                errors.push(format!(
                    "scripts/{name}: 未登记到 {label}（新增工具须 3 处登记：台账表行 + 速查表 + \
                     全库索引；存量豁免见 scripts/process_ratchet.json）"
                ));
            }
        }
    }
    errors
}

/// 第 34 项：**流程固化台账自洽**（状态枚举 / 载体存在 / 待固化带任务号 / 本页已登记索引）。
///
/// 动机（`docs/process/15_process_ledger.md`）：流程台账若只是文档，久了必然与现实脱节
/// （「已自动化」其实没自动化、「待固化」长期无任务号）。本项把它变成机器可判的进度面。
fn check_process_ledger(root: &Path) -> Vec<String> {
    const STATUSES: [&str; 4] = ["已自动化", "半自动", "待固化", "一次性"];
    let mut errors: Vec<String> = Vec::new();
    let Some(ledger) = read_file(root, "docs/process/15_process_ledger.md") else {
        return vec![
            "docs/process/15_process_ledger.md 不存在（流程固化台账 = 纪律执行状态面）".to_string(),
        ];
    };
    if !read_file(root, "docs/README.md").unwrap_or_default().contains("15_process_ledger.md") {
        errors.push("docs/README.md 未登记 15_process_ledger.md（新增文档须登记索引）".to_string());
    }
    for (i, line) in ledger.lines().enumerate() {
        let t = line.trim();
        if !t.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 7 {
            continue;
        }
        let num = cells[0];
        if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let name = cells[1];
        let carrier = cells[5];
        let status = cells[6];
        if !STATUSES.iter().any(|s| status.starts_with(s)) {
            errors.push(format!(
                "15_process_ledger.md:{}: 流程「{name}」状态 `{status}` 非法（须 {}）",
                i + 1,
                STATUSES.join(" / ")
            ));
            continue;
        }
        if status.starts_with("待固化") && !cells.join(" ").contains("S2") {
            errors.push(format!(
                "15_process_ledger.md:{}: 流程「{name}」标待固化但无台账任务号（不许无任务号长期挂账）",
                i + 1
            ));
        }
        for token in carrier.split_whitespace() {
            let tool = token.trim_matches(|c: char| c == '`' || c == ',');
            if let Some(rel) = tool.strip_prefix("scripts/") {
                if !root.join("scripts").join(rel).is_file() {
                    errors.push(format!(
                        "15_process_ledger.md:{}: 流程「{name}」载体 `scripts/{rel}` 不存在",
                        i + 1
                    ));
                }
            }
        }
    }
    errors
}
/// 第 35 项：**UI 视觉改动的设计稿签收留痕**（A 纪律，EXP-114）。
///
/// 动机：EXP-114 —— 视觉改动「先落码后出图」导致返工全堆在收口阶段；`00_index.md` §UI 视觉
/// 改动要求「出图 → 用户确认（留痕）→ 落码」，但留痕只存在于聊天消息，无机器判定。
/// 本项要求：**动了视觉数据面**（`scenes.json` / `theme.json` / `assets/ui`）的 verified 任务，
/// 其 dod 须写明用户视觉确认（过目/验收/确认/签收 之一）—— 否则「没确认就落码」再次复发。
fn check_visual_design_signoff(root: &Path) -> Vec<String> {
    // 只看**版面/配色数据面**（scenes/theme）：单纯产资产（assets/ui）不改画面 ⇒ 不要求签收
    const TOUCH: [&str; 2] = ["scenes.json", "theme.json"];
    // 签收标记写在**证据**里（留痕的载体是证据，不是 dod 的措辞）
    const SIGNOFF: [&str; 5] =
        ["用户视觉验收", "用户视觉过目", "用户视觉确认", "视觉签收", "用户拍板"];
    let mut errors: Vec<String> = Vec::new();
    let Some(ledger) = read_file(root, "plan/ledger.yaml") else {
        return vec!["plan/ledger.yaml 不存在，无法校验视觉签收留痕".to_string()];
    };
    for line in ledger.lines() {
        let Some(rest) = line.trim_start().strip_prefix("- { id: ") else {
            continue;
        };
        let Some((id_raw, _)) = rest.split_once(',') else {
            continue;
        };
        let id = id_raw.trim();
        if !line.contains("status: verified") {
            continue;
        }
        // 纪律自 S20.3 起强制（A 纪律落地点，EXP-114）
        let parts: Vec<&str> = id.trim_start_matches('S').split('.').collect();
        let major: u32 = parts.first().and_then(|m| m.parse().ok()).unwrap_or(0);
        let minor: u32 = parts.get(1).and_then(|m| m.parse().ok()).unwrap_or(0);
        if major < 20 || (major == 20 && minor < 3) {
            continue;
        }
        let Some(dod) = line.split("dod: \"").nth(1) else {
            continue;
        };
        let dod = dod.split('"').next().unwrap_or("");
        if !TOUCH.iter().any(|t| dod.contains(t)) {
            continue;
        }
        // 证据路径 = dod 之后的 evidence: 字段
        let Some(ev) = line.split("evidence: \"").nth(1) else {
            continue;
        };
        let ev = ev.split('"').next().unwrap_or("").trim();
        if ev.is_empty() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(ev)) else {
            continue; // 证据缺失 = check_ledger_reality 的职责
        };
        if !SIGNOFF.iter().any(|s| text.contains(s)) {
            errors.push(format!(
                "ledger.yaml {id}: 动了版面/配色数据面（scenes/theme）但证据 {ev} 无用户视觉签收留痕 \
                 （须含「用户视觉验收通过」等；00_index.md §UI 视觉改动 = A 纪律，EXP-114）"
            ));
        }
    }
    errors
}

/// 第 36 项：**节点 kind ↔ 资产映射唯一事实源**（spec §6 ↔ `ui_manifest.json` ↔ 代码映射）。
///
/// 动机（S21.1 批次 15 资产 / S21.5 落码）：9 类节点牌的 kind→资产映射若只写在规格文档，
/// 落码时手工抄进代码 + manifest ⇒ 抄漏/抄错无断言（渲染找不到图 = 节点画成默认块）。
/// 本项要求：`ui_manifest.json` 里 `node_*` 资产集合 == 规格 §6 声明的 kind 集合（缺一即 FAIL）。
fn check_node_kind_asset_map(root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let Some(manifest_text) = read_file(root, "assets/ui_manifest.json") else {
        return vec!["assets/ui_manifest.json 不存在，无法校验节点资产映射".to_string()];
    };
    let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&manifest_text) else {
        return vec!["assets/ui_manifest.json 解析失败".to_string()];
    };
    let names: Vec<String> = manifest
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let node_assets: Vec<String> =
        names.iter().filter(|n| n.starts_with("node_")).cloned().collect();
    if node_assets.is_empty() {
        return errors; // 尚未引入节点牌资产（v3 之前）⇒ 不判
    }
    // 规格文档声明的 kind→资产名（`node_<kind>.png`）
    let kinds = ["start", "city", "port", "event", "card", "battle", "treasure", "boss", "plain"];
    for kind in kinds {
        let want = format!("node_{kind}");
        if !node_assets.iter().any(|n| n == &want) {
            errors.push(format!(
                "ui_manifest.json: 缺节点牌资产 `{want}`（kind={kind}；9 类 kind 集合 = \
                 规格 08_main_map_v4_spec.md §6 单一事实源）"
            ));
        }
    }
    for name in &node_assets {
        let kind = name.trim_start_matches("node_");
        if !kinds.contains(&kind) {
            errors.push(format!(
                "ui_manifest.json: 节点牌资产 `{name}` 的 kind 不在 9 类集合（{kinds:?}）"
            ));
        }
    }
    errors
}

/// 第 37 项：**`src` 内不得内联 `#[cfg(test)]` 单测**（EXP-052 连踩 2 次 ⇒ 白跑一轮 gate）。
///
/// 动机：内联单测写在 `crates/**/src/**` 里，其中的 `expect`/`panic!` 会被 gate 第 5/6 项
/// （占位 / unwrap 扫描）拦下 ⇒ 写完跑 gate 才被自己的单测拦白跑一轮。规定单测只放
/// `crates/*/tests/`（项目既有约定），`src` 内联单测一律提前拦。
/// 单个检查项实现签名（`fn(仓库根) -> 错误列表`）。
type CheckFn = fn(&Path) -> Vec<String>;

/// `doccheck2` 检查项**实现注册表**（id → 实现）。
///
/// id 与 `assets/gate_data.json` 的 `doccheck2_checks[].id` 一一对应（双向自校验见
/// `check_check_inventory_consistency`）：**清单/计数/打印文案单一事实源 = JSON**，
/// 本表只登记「id → Rust 实现」；新增一项 = JSON 加一行 + 本表加一行（+ 实现 fn）。
const RUST_CHECKS: &[(&str, CheckFn)] = &[
    ("audio", check_audio_state),
    ("oracle", check_shell_oracle),
    ("lcg", check_lcg_hex),
    ("landmark", check_landmark_threshold),
    ("i18n", check_i18n_orphans),
    ("gate-items", check_gate_item_count),
    ("headless-battle-api", check_headless_battle_api),
    ("flywheel-readback", check_flywheel_readback),
    ("battle-player-data-driven", check_battle_player_data_driven),
    ("flywheel-script", check_flywheel_script),
    ("touch-render-pair", check_touch_render_pair),
    ("hd-sprite-transparency", check_hd_sprite_transparency),
    ("scene-layout-fidelity", check_scene_layout_fidelity),
    ("scene-band-count", check_scene_band_count),
    ("save-slot-map-slot-basis", check_save_slot_map_slot_basis),
    ("exhaust-balance-dual-write", check_exhaust_balance_dual_write),
    ("experience-volume-counts", check_experience_volume_counts),
    ("runtime-no-legacy-coords", check_runtime_no_legacy_coords),
    ("layout-doc-table-cross-check", check_layout_doc_table_cross_check),
    ("no-ti-core-fullpath", check_no_ti_core_fullpath),
    ("std-traits-matrix", check_std_traits_matrix),
    ("render-framework-boundary", check_render_framework_boundary),
    ("ui-redesign-tokens", check_ui_redesign_tokens),
    ("primitives-data", check_primitives_data),
    ("terrain-data", check_terrain_data),
    ("tooling-catalog", check_tooling_catalog),
    ("tooling-contract", check_tooling_contract),
    ("engine-no-bare-half", check_engine_no_bare_half),
    ("select-role-render-loop", check_select_role_render_loop),
    ("render-single-source", check_render_single_source),
    ("repo-text-integrity", check_repo_text_integrity),
    ("task-change-scope", check_task_change_scope),
    ("tooling-registration", check_tooling_registration),
    ("process-ledger", check_process_ledger),
    ("visual-signoff", check_visual_design_signoff),
    ("node-asset-map", check_node_kind_asset_map),
    ("no-inline-test-mod", check_no_inline_test_mod),
];

/// 数据驱动 grep 断言（`impl: "grep"` 项）：`require` = 必须出现（**词边界**匹配：
/// 命中处后一个字符不得是字母/数字/下划线）；`forbid` = 禁止出现（普通子串）。
fn doccheck2_grep_checks(root: &Path, gate_data: &serde_json::Value) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    let Some(list) = gate_data.get("doccheck2_checks").and_then(|v| v.as_array()) else {
        errors.push(
            "assets/gate_data.json 缺 `doccheck2_checks` 清单（检查项单一事实源）".to_string(),
        );
        return errors;
    };
    for entry in list {
        if entry.get("impl").and_then(|v| v.as_str()) != Some("grep") {
            continue;
        }
        let id = entry.get("id").and_then(|v| v.as_str()).unwrap_or("?");
        let Some(rules) = entry.get("rules").and_then(|v| v.as_array()) else {
            errors.push(format!("doccheck2_checks[{id}] 缺 `rules`（grep 项须声明断言规则）"));
            continue;
        };
        for rule in rules {
            let file = rule.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let text = read_file(root, file).unwrap_or_default();
            if text.is_empty() {
                errors.push(format!("doccheck2_checks[{id}]: 文件不存在或读失败 {file}"));
                continue;
            }
            // `.rs` 断言只查**代码**（跳过 `//`/`///`/`//!` 注释行）：约定守卫不该被
            // 「记录反例的注释」误伤（如 `bevy_ortho_scale` 的反例留档）。
            let is_rust = std::path::Path::new(file)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("rs"));
            let code = if is_rust {
                text.lines()
                    .filter(|l| {
                        let t = l.trim_start();
                        !(t.starts_with("//") || t.starts_with("/*") || t.starts_with('*'))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                text.clone()
            };
            for needle in rule
                .get("require")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
                .unwrap_or_default()
            {
                // 词边界只对「以标识符字符结尾」的 needle 生效（`o.area` 须排除 `o.area_x`）；
                // 以 `(` 等结尾的 needle 用普通包含（否则 `fn x(` 后面接参数类型即误判）。
                let boundary =
                    needle.chars().next_back().is_some_and(|c| c.is_alphanumeric() || c == '_');
                let hit = if boundary {
                    code.match_indices(needle).any(|(i, _)| {
                        code[i + needle.len()..]
                            .chars()
                            .next()
                            .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
                    })
                } else {
                    code.contains(needle)
                };
                if !hit {
                    errors.push(format!(
                        "doccheck2_checks[{id}]: {file} 缺必需内容 `{needle}`（口径/守卫失位）"
                    ));
                }
            }
            for needle in rule
                .get("forbid")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>())
                .unwrap_or_default()
            {
                if code.contains(needle) {
                    errors.push(format!(
                        "doccheck2_checks[{id}]: {file} 含禁止内容 `{needle}`（违反约定）"
                    ));
                }
            }
        }
    }
    errors
}

/// 清单双向自校验：JSON 清单（`impl: "rust"` 的 id）必须与 Rust 实现注册表**完全一致**
/// （少登记 = 实现白跑；多登记 = 清单漂移）。
fn check_check_inventory_consistency(gate_data: &serde_json::Value, errors: &mut Vec<String>) {
    let Some(list) = gate_data.get("doccheck2_checks").and_then(|v| v.as_array()) else {
        errors.push("assets/gate_data.json 缺 `doccheck2_checks` 清单（单一事实源）".to_string());
        return;
    };
    let data_rust: Vec<&str> = list
        .iter()
        .filter(|e| e.get("impl").and_then(|v| v.as_str()) == Some("rust"))
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()))
        .collect();
    let table: Vec<&str> = RUST_CHECKS.iter().map(|(id, _)| *id).collect();
    for id in &data_rust {
        if !table.contains(id) {
            errors
                .push(format!("doccheck2_checks[{id}] 在 JSON 清单里但无 Rust 实现（登记未落地）"));
        }
    }
    for id in &table {
        if !data_rust.contains(id) {
            errors.push(format!(
                "doccheck2 实现 `{id}` 未登记进 JSON 清单（清单漂移，assets/gate_data.json）"
            ));
        }
    }
}

/// 清单 id 列表（打印用；数据缺失时回落为空）。
fn check_inventory_ids(gate_data: &serde_json::Value) -> Vec<String> {
    gate_data
        .get("doccheck2_checks")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| e.get("id").and_then(|v| v.as_str()))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn check_no_inline_test_mod(root: &Path) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();
    // 棘轮基线：存量内联单测文件豁免（新增文件零豁免）
    let mut waived: Vec<String> = Vec::new();
    if let Some(base) = read_file(root, "scripts/process_ratchet.json") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&base) {
            if let Some(arr) = v
                .get("inline_test_mod_grandfathered")
                .and_then(|g| g.get("files"))
                .and_then(|f| f.as_array())
            {
                waived = arr.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
            }
        }
    }
    for e in
        ["ti-app", "ti-core", "ti-render", "ti-platform", "ti-formats", "ti-tools", "ti-engine"]
    {
        let src = root.join("crates").join(e).join("src");
        if !src.is_dir() {
            continue;
        }
        let mut stack = vec![src];
        while let Some(dir) = stack.pop() {
            let Ok(rd) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in rd.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|x| x.to_str()) != Some("rs") {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                if !text.contains("#[cfg(test)]") {
                    continue;
                }
                let rel =
                    path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
                if waived.iter().any(|w| w == &rel) {
                    continue;
                }
                errors.push(format!(
                    "{rel}: 内联 #[cfg(test)] 单测（单测只放 crates/*/tests/；src 内联会被 gate \
                     第 5/6 项当占位/unwrap 拦 ⇒ 白跑一轮 gate，EXP-052）"
                ));
            }
        }
    }
    errors
}
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let root = args.first().map_or_else(|| std::path::PathBuf::from("."), std::path::PathBuf::from);

    let mut all_errors: Vec<String> = Vec::new();
    // 检查清单（id 列表 / 计数 / 打印文案）单一事实源 = `assets/gate_data.json`；
    // Rust 侧只登记 id → 实现（`RUST_CHECKS`），双向自校验防漂移。
    let mut gate_data_errors: Vec<String> = Vec::new();
    let gate_data = load_gate_data(&root, &mut gate_data_errors);
    all_errors.extend(gate_data_errors);
    check_check_inventory_consistency(&gate_data, &mut all_errors);
    for (_id, f) in RUST_CHECKS {
        all_errors.extend(f(&root));
    }
    all_errors.extend(doccheck2_grep_checks(&root, &gate_data));

    let ids = check_inventory_ids(&gate_data);
    println!(
        "doccheck2: {} error(s) across {} audit-codified checks ({})",
        all_errors.len(),
        ids.len(),
        ids.join("/")
    );
    for err in &all_errors {
        println!("  ERROR {err}");
    }
    if all_errors.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
