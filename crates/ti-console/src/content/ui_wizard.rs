//! UI 工作台向导（S44.3 / 30 号 §2.2）：固定 6 步，每步调白名单动作/平台自身 + 机器判据回显，
//! 不可跳步（未过 N 步判据 → N+1 步 409），失败即停。
//!
//! 状态机持久化到 `evidence/console/content-ui-<id>.json`（gitignored；22 号 §3.1 平台自身写点）。
//! 写面纪律：写动作（第 1/2/3/5 步 = design/variant/layout/shot）须 `enable_write`（30 号 §3.1 默认关）；
//! 第 4/6 `步（build/zero_rust）为只读面（build` 写 `target/` gitignored `不计；zero_rust` 只读断言）。
//! 北极星写点探针（`write_probe`）按动作登记写点集判定（30 号 §3.2）。
//! 变体器同路径替换（29 号 §3）：`ui_variant tint` 默认覆盖原资产（资产名/坐标/scenes.json 不动 ⇒ 零 Rust + 零 scenes.json diff）；
//! `--suffix` 选项出变体路径（设计稿迭看，不落定）。

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::repo::snapshot::FrozenSnapshot;

/// 白名单动作解析结果：runner + 完整参数（逐参数；禁 shell 拼接）。
/// 执行面规格：runner + 完整参数 + 环境变量（key, value）。
type ExecSpec = (String, Vec<String>, Vec<(&'static str, String)>);

/// UI 向导固定步骤（30 号 §2.2 六步；顺序即执行序，不可跳步）。
const UI_STEPS: [&str; 6] = [
    "design",    // 1 设计稿先行（当前状态出图）
    "variant",   // 2 视觉变体器（ui_variant tint 同路径替换）
    "layout",    // 3 版面（可选：改 scenes.json 原语；纯视觉变体跳过）
    "build",     // 4 重建（cargo build，嵌 scenes.json）
    "shot",      // 5 出图验收（多视口）
    "zero_rust", // 6 零 Rust + 签收
];

/// 每步对应的白名单动作 id（`console_commands.json`）；其余为平台自身面。
/// `设计/layout/build/shot/zero_rust` 为平台自身动作（scene shot / scenes.json 写 / cargo build / git diff）。
const STEP_ACTION: [Option<&str>; 6] = [
    None,               // design：平台自身 scene shot（当前状态）
    Some("ui_variant"), // variant：ui_variant tint（同路径替换）
    None,               // layout：平台自身 scenes.json 写（可选，跳过=只读）
    None,               // build：平台自身 cargo build
    None,               // shot：平台自身 scene shot（多视口）
    None,               // zero_rust：平台自身 `git diff crates/` 断言
];

/// 每步写点集（`params` 无关的平台自身写点）；`[]` = 只读步。
/// design/shot 写 `evidence/`（出图，非冻结路径 ⇒ 探针天然 0 差，仍记写点语义）。
fn step_write_paths(step: &str) -> &'static [&'static str] {
    match step {
        "design" | "shot" => &["evidence/"],
        "variant" => &["assets/ui/", "assets/ui_manifest.json"],
        // F1 修复（B 审计 r1 P1）：layout 步为「可选跳过断言」（纯视觉变体，0 scenes.json diff），
        // 只读不写盘；版面编辑（scenes.json 写）不在本向导内（30 号 §2.2 step 3 已对齐）。
        // 其余步骤（build/zero_rust）无写点（build 产物在 target/，zero_rust 只读断言）。
        _ => &[],
    }
}

/// UI 向导状态（持久化 JSON）。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UiWizardState {
    /// 变体 id（`^[a-z0-9_]{1,16}$`）。
    pub id: String,
    /// `目标场景（index/main_map/...，30` 号 §2.2 `--scene=<id>`）。
    pub scene: String,
    /// 视口（"" = 壳默认；`WxH` 如 1920x1080，S29.5 多视口出图）。
    pub viewport: String,
    /// 变体资产（逗号分隔资产名，step 2 用）。
    pub assets: String,
    /// `变体着色参数（tint_cool，30` 号 §2.2 step 2）。
    pub color: f64,
    /// Brightness 增强（v2b=1.10/0.94）。
    pub brightness: f64,
    /// R 通道乘数（冷移，v2b=0.88/0.92）。
    pub rmult: f64,
    /// G 通道乘数（v2b=1.00）。
    pub gmult: f64,
    /// B 通道乘数（冷移，v2b=1.20/1.12）。
    pub bmult: f64,
    /// 用户视觉签收留痕（A 纪律硬规则 9；step 1 设计稿 / step 6 签收）。
    pub signoff: String,
    /// 当前待执行步骤索引（0 基；完成 N 步后 = N）。
    pub current_step: usize,
    /// 失败即停标志（任一步未过判据 → true）。
    pub halted: bool,
    /// 每步记录（长度 = `UI_STEPS` 6）。
    pub steps: Vec<UiStepRecord>,
}

/// 单步记录。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct UiStepRecord {
    /// 步骤名（`UI_STEPS` 枚举）。
    pub name: String,
    /// 已执行。
    pub done: bool,
    /// 通过判据（含写点探针）。
    pub passed: bool,
    /// 执行命令表示（白名单 runner + 参数）。
    pub cmd: String,
    /// 子进程退出码。
    pub exit_code: Option<i32>,
    /// 机器判据描述。
    pub criterion: String,
    /// 执行时间戳（epoch 秒）。
    pub ts: Option<String>,
}

impl UiWizardState {
    fn new(id: &str, scene: &str, viewport: &str) -> Self {
        Self {
            id: id.to_owned(),
            scene: scene.to_owned(),
            viewport: viewport.to_owned(),
            assets: String::new(),
            color: 1.0,
            brightness: 1.0,
            rmult: 1.0,
            gmult: 1.0,
            bmult: 1.0,
            signoff: String::new(),
            current_step: 0,
            halted: false,
            steps: UI_STEPS
                .iter()
                .map(|s| UiStepRecord {
                    name: (*s).to_owned(),
                    done: false,
                    passed: false,
                    cmd: String::new(),
                    exit_code: None,
                    criterion: String::new(),
                    ts: None,
                })
                .collect(),
        }
    }
}

/// 向导状态文件路径（gitignored；平台自身唯一写点）。
fn state_path(repo_root: &Path, id: &str) -> PathBuf {
    repo_root.join("evidence/console").join(format!("content-ui-{id}.json"))
}

fn read_state(repo_root: &Path, id: &str) -> Result<UiWizardState, (u16, String)> {
    let p = state_path(repo_root, id);
    if !p.is_file() {
        return Ok(UiWizardState::new(id, "index", ""));
    }
    let text =
        std::fs::read_to_string(&p).map_err(|e| (500u16, format!("读 UI 向导状态失败：{e}")))?;
    serde_json::from_str(&text).map_err(|e| (500u16, format!("解析 UI 向导状态失败：{e}")))
}

fn write_state(repo_root: &Path, state: &UiWizardState) -> Result<(), (u16, String)> {
    let p = state_path(repo_root, &state.id);
    let parent = p.parent().ok_or_else(|| (500u16, "UI 向导状态路径无父目录".to_owned()))?;
    std::fs::create_dir_all(parent).map_err(|e| (500u16, format!("建 UI 向导目录失败：{e}")))?;
    let text = serde_json::to_string_pretty(state)
        .map_err(|e| (500u16, format!("序列化 UI 向导状态失败：{e}")))?;
    std::fs::write(&p, text).map_err(|e| (500u16, format!("写 UI 向导状态失败：{e}")))
}

/// 步骤机器判据解析（30 号 §4「每步机器判据」）：读工具输出，判定该步是否通过。
/// 返回 (passed, criterion 描述)。
#[must_use]
pub fn parse_criterion(step: &str, exit_code: i32, output: &str) -> (bool, String) {
    let out = output.to_lowercase();
    match step {
        "design" | "shot" => {
            // 出图判据：exit 0（xvfb 下 bin 成功跑完）；图像非黑判据由 run_step 侧检查 evidence/ 产物。
            let ok = exit_code == 0;
            (ok, format!("exit={exit_code}（出图成功 + 图像非黑判据复核）"))
        }
        "variant" => {
            // 变体器判据：exit 0（tint 成功）+ ui_manifest 已更新（register 面，disk↔registry 一致）。
            let ok = exit_code == 0 && (out.contains("已更新") || out.contains("updated"));
            (
                ok,
                format!(
                    "exit={exit_code} + ui_manifest 登记一致（disk↔registry，validate_data 0 违规）"
                ),
            )
        }
        "layout" => {
            // 版面（可选）：跳过（纯视觉变体，0 scenes.json diff）或 scenes.json 规范形写成功。
            let ok = exit_code == 0;
            (
                ok,
                format!(
                    "exit={exit_code}（scenes.json indent=1 规范形 + check_scenes_canonical 0 差）"
                ),
            )
        }
        "build" => {
            let ok = exit_code == 0;
            (ok, format!("exit={exit_code}（build 成功，scenes.json 生效）"))
        }
        "zero_rust" => {
            let ok = exit_code == 0 && out.trim().is_empty();
            (ok, format!("exit={exit_code} + git diff crates/ 空（游戏产品 6 crate）"))
        }
        _ => (exit_code == 0, format!("exit={exit_code}")),
    }
}

/// 执行一步（`POST /api/content/ui/step`）。
///
/// body：`{"id","step","enable_write"?,"params"?}`。`step` = 步骤名（枚举 `UI_STEPS`）。
/// `params`：`{"scene","viewport","assets","color","brightness","rmult","gmult","bmult","signoff","layout"}`。
/// 不可跳步：当前步骤 != 请求步骤 ⇒ 409（前置未过）。
///
/// # Errors
/// id/step 非法（400）/ 不可跳步（409）/ 动作未登记（400）/ 写动作无 `enable_write`（400）/
/// spawn/解析失败（500）。
pub fn run_step(
    repo_root: &Path,
    id: &str,
    step: &str,
    enable_write: bool,
    params: &Value,
) -> Result<Value, (u16, String)> {
    // 校验 + 状态读 + 不可跳步/失败重跑门（抽离，控 run_step 长度）。
    let (step_idx, mut state) = validate_request(repo_root, id, step)?;
    // 从 params 合并场景/视口/资产/着色参数/签收（step 1 首次设定，后续步骤沿用）。
    // F3：merge_params 校验 scene/viewport/assets/suffix（路径穿越/注入防），非法 → 400。
    merge_params(&mut state, params)?;
    let action = STEP_ACTION[step_idx];
    // 写点集：layout 步只读（0 scenes.json diff，30 号 §2.2 step 3 可选跳过；版面编辑不在本向导）。
    let write_paths: Vec<String> = step_write_paths(step).iter().map(|s| (*s).to_owned()).collect();
    let is_write_step = !write_paths.is_empty();
    if is_write_step && !enable_write {
        return Err((
            400u16,
            format!(
                "第 {} 步（{}）为写动作，需 `enable_write=true`（30 号 §3.1 默认关）",
                step_idx + 1,
                step
            ),
        ));
    }

    // 北极星写点探针：写动作前后各取快照，写点外差异 = 漂移（30 号 §3.2）。
    let before_snap = FrozenSnapshot::take(repo_root);

    // 解析执行面（白名单动作 / 平台自身 design/layout/build/shot/zero_rust）→ (runner, args, env)。
    let (runner, mut args, env_vars) = resolve_execution(repo_root, &state, step, action, params)?;

    // 参数填充（<id> 占位符 → 实际 id）。
    for a in &mut args {
        if a.contains("<id>") {
            *a = a.replace("<id>", id);
        }
    }

    // 执行（同步；向导步骤判据须等待完成，不后台 spawn）。
    let mut cmd = Command::new(&runner);
    cmd.args(&args).current_dir(repo_root);
    for (k, v) in &env_vars {
        cmd.env(k, v);
    }
    crate::runner::apply_minimal_env(&mut cmd);
    let output = cmd.output().map_err(|e| (500u16, format!("spawn {runner} 失败：{e}")))?;
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}");

    let (passed, criterion) = evaluate_criterion(step, repo_root, &state, exit_code, &combined);

    // 北极星写点探针判定（写步骤）。
    let (probe_ok, probe_note) = if is_write_step {
        let after_snap = FrozenSnapshot::take(repo_root);
        let ok = before_snap
            .write_probe(&after_snap, &write_paths.iter().map(String::as_str).collect::<Vec<_>>());
        let note = if ok {
            format!("write_probe 0 差（写点 {} 豁免）", write_paths.join(", "))
        } else {
            "write_probe 漂移：写点外冻结路径有差异（检查越界写点）".to_owned()
        };
        (ok, note)
    } else {
        (true, String::new())
    };

    // 记录步（先算判定，再借 state 写入——避免 mut/immut 借用冲突）。
    let step_passed = passed && probe_ok;
    let criterion_resp = criterion.clone();
    let rec = &mut state.steps[step_idx];
    rec.done = true;
    rec.passed = step_passed;
    rec.cmd = format!("{runner} {}", args.join(" "));
    rec.exit_code = Some(exit_code);
    rec.criterion = criterion;
    rec.ts = Some(now_iso());

    if step_passed {
        state.current_step = step_idx + 1;
    } else {
        state.halted = true; // 失败不推进 current_step（须重跑该步）。
    }
    let current_step = state.current_step;
    let halted = state.halted;
    write_state(repo_root, &state)?;

    Ok(json!({
        "ok": step_passed,
        "id": id,
        "step": step,
        "step_index": step_idx + 1,
        "exit_code": exit_code,
        "criterion": criterion_resp,
        "probe": probe_note,
        "passed": step_passed,
        "current_step": current_step,
        "halted": halted,
        "output_excerpt": combined.chars().take(1500).collect::<String>(),
    }))
}

/// 步骤判据求值（抽离 `run_step`，控其长度）：解析判据 + 出图图像非黑 + 版面跳过断言 + 签收留痕。
fn evaluate_criterion(
    step: &str,
    repo_root: &Path,
    state: &UiWizardState,
    exit_code: i32,
    combined: &str,
) -> (bool, String) {
    let (mut passed, mut criterion) = parse_criterion(step, exit_code, combined);

    // 出图步图像非黑判据（S27.5 口径；纯色判据拦不住 blank frame，须像素多样性判据）。
    if step == "design" || step == "shot" {
        let shot_path = shot_output_path(repo_root, state);
        let (img_ok, note) = shot_image_ok(&shot_path);
        passed = passed && img_ok;
        criterion = format!("{criterion}；图像判据：{note}");
    }

    // 版面步（F1：只读断言）：纯视觉变体跳过（0 scenes.json diff），版面编辑不在本向导。
    if step == "layout" {
        let (no_diff, note) = scenes_json_no_diff(repo_root);
        passed = passed && no_diff;
        criterion = format!("{criterion}；版面跳过：{note}");
    }

    // F2 修复（B 审计 r1 P1）：zero_rust 步判据须含视觉签收留痕（A 纪律硬规则 9）。
    // 规格 = `git diff crates/` 空 + `state.signoff` 非空含签收关键词（用户视觉验收/过目/确认/签收/拍板）。
    // 无 signoff = 判败（不推进向导；须重跑该步补签收）。
    // B-N1 修正（r2）：signoff 实为「用户 <日期> <任务> 视觉验收通过（…）」格式，日期/语境隔断「用户视觉验收」
    // 连续子串 ⇒ 追加核心动词短语「视觉验收」（覆盖「…视觉验收通过」）；与 doccheck2 第 35 项同口径。
    if step == "zero_rust" {
        const SIGNOFF_KW: [&str; 6] =
            ["用户视觉验收", "用户视觉过目", "用户视觉确认", "视觉签收", "用户拍板", "视觉验收"];
        let has_signoff = SIGNOFF_KW.iter().any(|k| state.signoff.contains(k));
        if has_signoff {
            criterion = format!(
                "{criterion}；签收留痕判据：state.signoff 含「{}」",
                state.signoff.chars().take(24).collect::<String>()
            );
        } else {
            passed = false;
            criterion = format!(
                "{criterion}；签收留痕判据：state.signoff 空或不含签收关键词（须「用户视觉验收通过」等，A 纪律 EXP-114）"
            );
        }
    }

    (passed, criterion)
}

/// 请求校验：id 合法性 + step 在 6 步内 + 读状态 + 不可跳步/失败重跑门。
/// 返回 (`step_idx`, state)；state.halted 已按重跑当前步复位。
fn validate_request(
    repo_root: &Path,
    id: &str,
    step: &str,
) -> Result<(usize, UiWizardState), (u16, String)> {
    if id.is_empty()
        || id.len() > 16
        || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err((400u16, format!("ui_id `{id}` 非法（^[a-z0-9_]{{1,16}}$）")));
    }
    let step_idx = UI_STEPS.iter().position(|s| *s == step).ok_or_else(|| {
        (
            400u16,
            format!(
                "step `{step}` 不在 UI 向导 6 步（design/variant/layout/build/shot/zero_rust）"
            ),
        )
    })?;

    let mut state = read_state(repo_root, id)?;
    // 失败即停后：仅允许重跑当前（失败）步；重跑同一步 = 复位 halted 继续。
    if state.halted && step_idx != state.current_step {
        return Err((
            409u16,
            format!(
                "向导已停止（{id} 第 {} 步失败），须重跑该步（第 {} 步 = {}）",
                state.current_step + 1,
                state.current_step + 1,
                UI_STEPS[state.current_step]
            ),
        ));
    }
    if state.halted && step_idx == state.current_step {
        state.halted = false;
    }
    // 不可跳步：请求步骤必须 == 当前待执行步骤。
    if step_idx != state.current_step {
        return Err((
            409u16,
            format!(
                "不可跳步：当前待执行第 {} 步（{}），请求第 {} 步（{}）",
                state.current_step + 1,
                UI_STEPS[state.current_step],
                step_idx + 1,
                step
            ),
        ));
    }
    Ok((step_idx, state))
}

/// 从 `params` 合并场景/视口/资产/着色参数/签收到 state（幂等；缺省不覆盖）。
/// F3 修复（B 审计 r1 P1）：`scene`/`viewport`/`assets`/`suffix` 用户可控字符串**校验**，
/// 防路径穿越/命令参数注入（id 已有 `^[a-z0-9_]{1,16}$`；这些字段直达出图文件名与子进程参数）。
fn merge_params(state: &mut UiWizardState, params: &Value) -> Result<(), (u16, String)> {
    if let Some(v) = params.get("scene").and_then(Value::as_str) {
        if !v.is_empty() {
            if !is_ident(v) {
                return Err((
                    400u16,
                    format!("`scene` `{v}` 非法（^[a-z0-9_]{{1,16}}$，场景 id）"),
                ));
            }
            state.scene = String::from(v);
        }
    }
    if let Some(v) = params.get("viewport").and_then(Value::as_str) {
        // 空 = 壳默认视口；否则 `WxH`（S29.5 多视口出图）。
        if !v.is_empty() && !is_viewport(v) {
            return Err((
                400u16,
                format!("`viewport` `{v}` 非法（空 或 ^[0-9]{{2,4}}x[0-9]{{2,4}}$，如 1920x1080）"),
            ));
        }
        state.viewport = String::from(v);
    }
    if let Some(v) = params.get("assets").and_then(Value::as_str) {
        if !v.is_empty() {
            if !v
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == ',')
            {
                return Err((
                    400u16,
                    format!("`assets` `{v}` 非法（^[a-z0-9_,]+$，逗号分隔资产名）"),
                ));
            }
            state.assets = String::from(v);
        }
    }
    for (key, field) in [("color", 0), ("brightness", 1), ("rmult", 2), ("gmult", 3), ("bmult", 4)]
    {
        if let Some(v) = params.get(key).and_then(Value::as_f64) {
            if !(0.0..=3.0).contains(&v) {
                return Err((400u16, format!("`{key}` {v} 越界（[0,3]，ui_variant 参数域）")));
            }
            match field {
                0 => state.color = v,
                1 => state.brightness = v,
                2 => state.rmult = v,
                3 => state.gmult = v,
                4 => state.bmult = v,
                _ => {}
            }
        }
    }
    // F3：`suffix`（变体路径后缀，resolve_execution 直读 params）校验：`^[A-Za-z0-9_]*$`（可空）。
    // 非空时防 `..` 逃逸 assets/ui/（ui_variant.py `f"{name}{suffix}.png"`）。
    if let Some(v) = params.get("suffix").and_then(Value::as_str) {
        if !v.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err((400u16, format!("`suffix` `{v}` 非法（^[A-Za-z0-9_]*$，变体路径后缀）")));
        }
    }
    if let Some(v) = params.get("signoff").and_then(Value::as_str) {
        if !v.is_empty() {
            state.signoff = String::from(v);
        }
    }
    Ok(())
}

/// F3：`scene`/`suffix` 类短标识（`^[a-z0-9_]{1,16}$`，与 id 同口径）。
fn is_ident(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 16
        && value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// F3：视口格式（`WxH`，W/H 2~4 位数字）。
fn is_viewport(value: &str) -> bool {
    value.split_once('x').is_some_and(|(w, h)| {
        !w.is_empty()
            && !h.is_empty()
            && w.len() <= 4
            && h.len() <= 4
            && w.chars().all(|c| c.is_ascii_digit())
            && h.chars().all(|c| c.is_ascii_digit())
    })
}

/// `解析执行面：白名单动作（子进程）或平台自身动作（design/layout/build/shot/zero_rust`）。
/// 返回 (runner, args, `env_vars)；参数注入步骤位置参数（30` 号 §3.1 param）。
fn resolve_execution(
    repo_root: &Path,
    state: &UiWizardState,
    step: &str,
    action: Option<&str>,
    params: &Value,
) -> Result<ExecSpec, (u16, String)> {
    match action {
        Some(aid) => match crate::content::resolve_action(repo_root, aid)? {
            Some((r, mut a)) => {
                // ui_variant tint：追加 tint 专属参数（assets + 着色 + --register）。
                // 白名单 args = [script, tint]；tint 参数从向导 params 注入（30 号 §2.2 step 2）。
                if aid == "ui_variant" {
                    if state.assets.is_empty() {
                        return Err((
                            400u16,
                            "variant 步缺 `params.assets`（逗号分隔资产名）".to_owned(),
                        ));
                    }
                    a.push(format!("--assets={}", state.assets));
                    a.push(format!("--color={}", fmt_num(state.color)));
                    a.push(format!("--brightness={}", fmt_num(state.brightness)));
                    a.push(format!("--rmul={}", fmt_num(state.rmult)));
                    a.push(format!("--gmult={}", fmt_num(state.gmult)));
                    a.push(format!("--bmult={}", fmt_num(state.bmult)));
                    // 同路径替换（默认）+ 注册（disk↔registry 一致）；--suffix 出变体路径（迭看不落定）。
                    let suffix = params.get("suffix").and_then(Value::as_str).unwrap_or("");
                    if !suffix.is_empty() {
                        a.push(format!("--suffix={suffix}"));
                    }
                    let register = params.get("register").and_then(Value::as_bool).unwrap_or(true);
                    if register {
                        a.push("--register".to_owned());
                    }
                }
                Ok((r, a, Vec::new()))
            }
            None => Err((
                500u16,
                format!("动作 `{aid}` runner=self 无特殊执行面（UI 向导 step={step}）"),
            )),
        },
        None => match step {
            "design" | "shot" => Ok(scene_shot_command(repo_root, state)),
            // F1 修复（B 审计 r1 P1）：layout 步 = 纯视觉变体跳过（只读断言，0 scenes.json diff）；
            // 版面编辑（scenes.json 写）不在本向导（30 号 §2.2 step 3 已对齐，平台表单写端点为后续）。
            "layout" => Ok((
                "git".to_owned(),
                vec![
                    "diff".to_owned(),
                    "--quiet".to_owned(),
                    "--".to_owned(),
                    "assets/data/scenes.json".to_owned(),
                ],
                Vec::new(),
            )),
            "build" => Ok((
                resolve_cargo().to_string_lossy().to_string(),
                vec!["build".to_owned(), "-p".to_owned(), "ti-engine".to_owned()],
                Vec::new(),
            )),
            "zero_rust" => Ok((
                "git".to_owned(),
                vec![
                    "diff".to_owned(),
                    "--quiet".to_owned(),
                    "--".to_owned(),
                    "crates/ti-core".to_owned(),
                    "crates/ti-app".to_owned(),
                    "crates/ti-engine".to_owned(),
                    "crates/ti-render".to_owned(),
                    "crates/ti-platform".to_owned(),
                    "crates/ti-formats".to_owned(),
                ],
                Vec::new(),
            )),
            _ => Err((500u16, format!("UI 向导步 {step} 无执行面"))),
        },
    }
}

/// 出图输出路径：`evidence/bevy_native_<scene><viewport_suffix>.png`（bin 口径 S18.7）。
fn shot_output_path(repo_root: &Path, state: &UiWizardState) -> PathBuf {
    let suffix =
        if state.viewport.is_empty() { String::new() } else { format!("_{}", state.viewport) };
    repo_root.join("evidence").join(format!("bevy_native_{}{suffix}.png", state.scene))
}

/// scene shot 命令：运行 ti-engine 发货 bin（`--shot` 离屏，`--scene=<id>` 单参数 + 可选 `--viewport=WxH`）。
/// 无头环境需虚拟显示（xvfb-run -a，Linux 迁移口径）；bin 路径 = 构建期 target-dir。
fn scene_shot_command(repo_root: &Path, state: &UiWizardState) -> ExecSpec {
    let bin = resolve_bin_path(repo_root);
    let xvfb = Path::new("/usr/bin/xvfb-run");
    let mut shot_args: Vec<String> = Vec::new();
    if !state.viewport.is_empty() {
        shot_args.push(format!("--viewport={}", state.viewport));
    }
    shot_args.push("--shot".to_owned());
    shot_args.push(format!("--scene={}", state.scene));
    if xvfb.is_file() {
        let mut a = vec!["-a".to_owned(), bin.to_string_lossy().to_string()];
        a.extend(shot_args);
        (xvfb.to_string_lossy().to_string(), a, Vec::new())
    } else {
        (bin.to_string_lossy().to_string(), shot_args, Vec::new())
    }
}

/// `GET /api/content/ui?id=<id>`：读向导状态（含每步 done/passed/criterion + 当前步骤）。
///
/// # Errors
/// `id` 为空或格式非法（`^[a-z0-9_]{1,16}$`，与 `validate_request` 对齐）= 400（读状态前校验）。
pub fn state(repo_root: &Path, id: &str) -> Result<Value, (u16, String)> {
    if id.is_empty()
        || id.len() > 16
        || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err((400u16, format!("ui_id `{id}` 非法（^[a-z0-9_]{{1,16}}$）")));
    }
    let st = read_state(repo_root, id)?;
    Ok(json!({
        "id": st.id,
        "scene": st.scene,
        "viewport": st.viewport,
        "assets": st.assets,
        "signoff": st.signoff,
        "current_step": st.current_step,
        "total_steps": UI_STEPS.len(),
        "halted": st.halted,
        "steps": st.steps,
        "steps_def": UI_STEPS,
    }))
}

fn now_iso() -> String {
    // 简化为 epoch 秒（时间戳留痕；无 chrono 依赖）。
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        .to_string()
}

/// 浮点格式化（tint 参数；去尾零，保确定性）。
fn fmt_num(value: f64) -> String {
    let s = format!("{value:.4}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// 解析 cargo 绝对路径（build 步用）：`CARGO_HOME/bin/cargo` > `$HOME/.cargo/bin/cargo` > `PATH` 上 cargo。
/// 服务进程 `apply_minimal_env` 清 PATH 后可能无 cargo（Linux 无头 server 无 `~/.cargo/bin` 在 PATH），
/// 故显式解析绝对路径（与 `resolve_bin_path` 同口径，禁裸 `cargo` 字符串依赖 PATH）。
fn resolve_cargo() -> PathBuf {
    if let Ok(home) = std::env::var("CARGO_HOME") {
        let p = PathBuf::from(home).join("bin").join("cargo");
        if p.is_file() {
            return p;
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home).join(".cargo").join("bin").join("cargo");
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from("cargo")
}

/// 解析 ti-engine 发货 bin 路径（scene shot 用）：`CARGO_TARGET_DIR` > `.cargo/config.toml` `target-dir` > `repo/target`。
fn resolve_bin_path(repo_root: &Path) -> PathBuf {
    let mut target: Option<PathBuf> = std::env::var("CARGO_TARGET_DIR").ok().map(PathBuf::from);
    if target.is_none() {
        let cfg = repo_root.join(".cargo").join("config.toml");
        if let Ok(text) = std::fs::read_to_string(&cfg) {
            for line in text.lines() {
                let t = line.trim();
                if let Some(v) = t.strip_prefix("target-dir") {
                    let v = v
                        .trim_start()
                        .trim_start_matches('=')
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'');
                    if !v.is_empty() {
                        target = Some(PathBuf::from(v));
                    }
                    break;
                }
            }
        }
    }
    let target = match target {
        Some(t) if t.is_absolute() => t,
        Some(t) => repo_root.join(t),
        None => repo_root.join("target"),
    };
    target.join("debug").join("treasure-bevy-native")
}

/// 出图步图像判据：PNG 存在 + 非纯黑帧（像素多样性）。
/// 只读 `evidence/bevy_native_<scene>.png`（S27.5 出图口径；纯色/黑帧 = 判败）。
fn shot_image_ok(path: &Path) -> (bool, String) {
    if !path.is_file() {
        return (false, format!("未找到出图 {}", path.display()));
    }
    let Ok(bytes) = std::fs::read(path) else {
        return (false, format!("读图失败 {}", path.display()));
    };
    if bytes.len() < 1024 {
        return (false, format!("图过小（{} B），疑纯黑/纯色帧", bytes.len()));
    }
    let mut seen = [false; 256];
    let mut distinct = 0usize;
    for &b in &bytes {
        if !seen[b as usize] {
            seen[b as usize] = true;
            distinct += 1;
            if distinct > 8 {
                break;
            }
        }
    }
    if distinct < 4 {
        return (false, format!("图字节多样性低（{distinct} 种），疑纯色帧"));
    }
    (true, format!("出图 {}（{} B，字节多样性 {distinct}）", path.display(), bytes.len()))
}

/// 版面跳过判据：scenes.json 当前无 diff（纯视觉变体，0 版面改动）。
fn scenes_json_no_diff(repo_root: &Path) -> (bool, String) {
    let st = std::process::Command::new("git")
        .arg("diff")
        .arg("--quiet")
        .arg("--")
        .arg("assets/data/scenes.json")
        .current_dir(repo_root)
        .output();
    match st {
        Ok(o) if o.status.success() => {
            (true, "0 scenes.json diff（纯视觉变体跳过版面）".to_owned())
        }
        Ok(o) => (
            false,
            format!(
                "scenes.json 有 diff（exit={}），版面步须实际应用",
                o.status.code().unwrap_or(-1)
            ),
        ),
        Err(e) => (false, format!("git diff 失败：{e}")),
    }
}
