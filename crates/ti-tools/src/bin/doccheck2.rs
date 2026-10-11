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

    println!(
        "doccheck2: {} error(s) across 6 audit-codified checks (audio/oracle/lcg/landmark/i18n/gate-items)",
        all_errors.len()
    );
    for err in &all_errors {
        println!("  ERROR {err}");
    }
    if all_errors.is_empty() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
