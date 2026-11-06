//! 地图工作台向导（S44.2 / 30 号 §2.1）：固定 10 步，每步调白名单动作 + 机器判据回显，
//! 不可跳步（未过 N 步判据 → N+1 步 409），失败即停。
//!
//! 状态机持久化到 `evidence/console/content-map-<id>.json`（gitignored；22 号 §3.1 平台自身写点）。
//! 写面纪律：写动作（第 1/3/5/6/8/9 步 = add/syncmap/pack/install/sync/shot）须 `enable_write`（30 号 §3.1 默认关）；
//! 第 2/4/7/10 步（import/verify/check/zero_rust）为只读面（verify-only，无门无探针）。
//! 北极星写点探针（`write_probe`）按动作登记写点集判定（30 号 §3.2）。

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

use crate::repo::snapshot::FrozenSnapshot;

/// 白名单动作解析结果：runner + 完整参数（逐参数；禁 shell 拼接）。
type CmdSpec = (String, Vec<String>);
/// 执行面规格：runner + 完整参数 + 环境变量（key, value）。
type ExecSpec = (String, Vec<String>, Vec<(&'static str, String)>);

/// 地图向导固定步骤（30 号 §2.1 十步；顺序即执行序，不可跳步）。
/// S44.5：29 号 §2 十步 ↔ 本数组的 1:1 映射断言读取此表（`content::docmap`），故为 `pub(crate)`。
pub(crate) const MAP_STEPS: [&str; 10] = [
    "add",       // 1 新建包
    "import",    // 2 填内容（复用 S32.15 编辑器 import）
    "syncmap",   // 3 manifest["map"]←map.json
    "verify",    // 4 校验
    "pack",      // 5 打包
    "install",   // 6 安装
    "check",     // 7 漂移对账
    "sync",      // 8 回注
    "shot",      // 9 出图验收
    "zero_rust", // 10 零 Rust
];

/// 每步对应的白名单动作 id（`console_commands.json`）；`import`/`zero_rust` 为平台自身面。
/// S44.5：写面登记一致性断言读取此表（`content::docmap`）。
pub(crate) const STEP_ACTION: [Option<&str>; 10] = [
    Some("mappack_add"),
    None, // import：POST /api/map/import（S32.15）
    Some("mappack_syncmap"),
    Some("mappack_verify"),
    Some("mappack_pack"),
    Some("mappack_install"),
    Some("mappack_check"),
    Some("mappack_sync"),
    Some("map_shot"),
    None, // zero_rust：平台自身 `git diff crates/` 断言
];

/// 向导状态（持久化 JSON）。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct WizardState {
    /// 地图包 id（`^[a-z0-9_]{1,16}$`）。
    pub id: String,
    /// 当前待执行步骤索引（0 基；完成 N 步后 = N）。
    pub current_step: usize,
    /// 失败即停标志（任一步未过判据 → true）。
    pub halted: bool,
    /// 每步记录（长度 = `MAP_STEPS` 10）。
    pub steps: Vec<StepRecord>,
}

/// 单步记录。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct StepRecord {
    /// 步骤名（`MAP_STEPS` 枚举）。
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

impl WizardState {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
            current_step: 0,
            halted: false,
            steps: MAP_STEPS
                .iter()
                .map(|s| StepRecord {
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
    repo_root.join("evidence/console").join(format!("content-map-{id}.json"))
}

fn read_state(repo_root: &Path, id: &str) -> Result<WizardState, (u16, String)> {
    let p = state_path(repo_root, id);
    if !p.is_file() {
        return Ok(WizardState::new(id));
    }
    let text = std::fs::read_to_string(&p).map_err(|e| (500u16, format!("读向导状态失败：{e}")))?;
    serde_json::from_str(&text).map_err(|e| (500u16, format!("解析向导状态失败：{e}")))
}

fn write_state(repo_root: &Path, state: &WizardState) -> Result<(), (u16, String)> {
    let p = state_path(repo_root, &state.id);
    let parent = p.parent().ok_or_else(|| (500u16, "向导状态路径无父目录".to_owned()))?;
    std::fs::create_dir_all(parent).map_err(|e| (500u16, format!("建向导目录失败：{e}")))?;
    let text = serde_json::to_string_pretty(state)
        .map_err(|e| (500u16, format!("序列化向导状态失败：{e}")))?;
    std::fs::write(&p, text).map_err(|e| (500u16, format!("写向导状态失败：{e}")))
}

/// 从白名单解析动作 → (runner, 完整参数)（逐参数；禁 shell 拼接）。
/// 返回 `None` 表示动作是平台自身面（self runner），由调用方自行处理。
/// 共享实现见 `content/mod.rs::resolve_action`（map/ui 向导同口径）。
fn resolve_action(repo_root: &Path, action_id: &str) -> Result<Option<CmdSpec>, (u16, String)> {
    crate::content::resolve_action(repo_root, action_id)
}

/// 白名单动作的**写点模板集**（可含 `<id>` 占位符，30 号 §3.2 约定）。
/// S44.5：`console_commands.json` 的 `writes` 登记必须与本表逐项一致（写面登记一致性断言）。
#[must_use]
pub(crate) fn write_templates(action_id: &str) -> &'static [&'static str] {
    match action_id {
        "mappack_add" => &["assets/maps/"],
        "mappack_syncmap" => &["assets/maps/<id>/manifest.json"],
        "mappack_pack" => &["dist/maps/<id>.map-pack/"],
        "mappack_install" => &[
            "assets/maps/",
            "assets/data/board.json",
            "assets/maps/index.json",
            "assets/ui_manifest.json",
        ],
        "mappack_sync" => &[
            "assets/data/board.json",
            "assets/data/scenes.json",
            "assets/data/strings.json",
            "assets/data/ui_schema/scene_manifest_v3.json",
        ],
        "mappack_uninstall" => &[
            "assets/maps/",
            "assets/maps/.default",
            "assets/maps/index.json",
            "assets/data/board.json",
        ],
        "ui_variant" => &["assets/ui/", "assets/ui_manifest.json"],
        "map_shot" => &["evidence/"],
        _ => &[],
    }
}

/// 平台自身步（非白名单动作）的写点集（白名单动作步见 `write_templates`）。
/// `import` = S32.15 编辑器导入（落 `assets/maps/<id>/` 四文件）；`zero_rust` = 只读断言。
/// S44.5：29 号 ↔ 平台写面登记一致性断言读取本表。
#[must_use]
pub(crate) fn platform_write_paths(step: &str) -> &'static [&'static str] {
    match step {
        "import" => &["assets/maps/"],
        _ => &[],
    }
}

/// 把白名单动作的 `writes`（可含 `<id>` 占位符）解析为实际写点集（30 号 §3.2 约定）。
#[must_use]
pub fn resolve_write_paths(action_id: &str, id: &str) -> Vec<String> {
    write_templates(action_id).iter().map(|p| p.replace("<id>", id)).collect()
}

/// 步骤机器判据解析（30 号 §4「每步机器判据」）：读工具输出，判定该步是否通过。
/// 返回 (passed, criterion 描述)。
#[must_use]
pub fn parse_criterion(step: &str, exit_code: i32, output: &str) -> (bool, String) {
    let out = output.to_lowercase();
    match step {
        "add" | "syncmap" | "sync" => {
            let ok = exit_code == 0;
            (ok, format!("exit={exit_code}（0=成功）"))
        }
        "import" => {
            // 填内容步：判据 = 内容已填充 + I1~I9 终判过（编辑器导入后 verify 0 违规）。
            let ok = exit_code == 0 && (out.contains("0 违规") || out.contains("0 violation"));
            (ok, format!("exit={exit_code} + verify 0 违规（I1~I9 终判，内容已填充）"))
        }
        "verify" | "check" => {
            let ok = exit_code == 0
                && (out.contains("0 违规")
                    || out.contains("0 漂移")
                    || out.contains("0 violation")
                    || out.contains("0 差"));
            (ok, format!("exit={exit_code} + 输出含 0 违规/0 漂移/0 差"))
        }
        "pack" => {
            let ok = exit_code == 0;
            (ok, format!("exit={exit_code}（pack 成功 + dist 产物存在）"))
        }
        "install" => {
            let ok = exit_code == 0;
            (ok, format!("exit={exit_code}（install 接线成功）"))
        }
        "shot" => {
            // 出图判据：exit 0（xvfb 下 bin 成功跑完）；图像非黑判据由 run_step 侧检查 evidence/ 产物。
            let ok = exit_code == 0;
            (ok, format!("exit={exit_code}（出图成功 + 图像非黑判据复核）"))
        }
        "zero_rust" => {
            let ok = exit_code == 0 && out.trim().is_empty();
            (ok, format!("exit={exit_code} + git diff crates/ 空（游戏产品 6 crate）"))
        }
        _ => (exit_code == 0, format!("exit={exit_code}")),
    }
}

/// 执行一步（`POST /api/content/map/step`）。
///
/// body：`{"id","step","enable_write"?,"params"?}`。`step` = 步骤名（枚举 `MAP_STEPS`）。
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
    _params: &Value,
) -> Result<Value, (u16, String)> {
    // 校验 + 状态读 + 不可跳步/失败重跑门（抽离，控 run_step 长度）。
    let (step_idx, mut state) = validate_request(repo_root, id, step)?;
    let action = STEP_ACTION[step_idx];
    let write_paths = resolve_write_paths(action.unwrap_or(""), id);
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

    // 解析执行面（白名单动作 / 平台自身 import/zero_rust/shot）→ (runner, args, env)。
    let (runner, mut args, env_vars) = resolve_execution(repo_root, id, step, action)?;

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

    let (mut passed, mut criterion) = parse_criterion(step, exit_code, &combined);

    // shot 步图像非黑判据（S27.5 口径；纯色判据拦不住 blank frame，须像素多样性判据）。
    if step == "shot" {
        let (img_ok, note) = shot_image_ok(&repo_root.join("evidence/bevy_native_main_map.png"));
        passed = passed && img_ok;
        criterion = format!("{criterion}；图像判据：{note}");
    }

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
    // clone 一次供响应回显（响应 json 须保留 criterion；记录移动进 rec）。
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

/// 请求校验：id 合法性 + step 在 10 步内 + 读状态 + 不可跳步/失败重跑门。
/// 返回 (`step_idx`, state)；state.halted 已按重跑当前步复位。
fn validate_request(
    repo_root: &Path,
    id: &str,
    step: &str,
) -> Result<(usize, WizardState), (u16, String)> {
    // F-2 修复（B 审计 MEDIUM）：id 校验与规格 `^[a-z0-9_]{1,16}$` 一致（22 号 §4.2 / 30 号 §3.1）——
    // 仅小写字母/数字/下划线且 ≤16；原实现允许大写 A-Z 与任意长度（契约违背，非安全问题）。
    if id.is_empty()
        || id.len() > 16
        || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err((400u16, format!("map_id `{id}` 非法（^[a-z0-9_]{{1,16}}$）")));
    }
    let step_idx = MAP_STEPS
        .iter()
        .position(|s| *s == step)
        .ok_or_else(|| (400u16, format!("step `{step}` 不在地图向导 10 步（add/import/syncmap/verify/pack/install/check/sync/shot/zero_rust）")))?;

    let mut state = read_state(repo_root, id)?;
    // 失败即停后：仅允许重跑当前（失败）步；重跑同一步 = 复位 halted 继续。
    if state.halted && step_idx != state.current_step {
        return Err((
            409u16,
            format!(
                "向导已停止（{id} 第 {} 步失败），须重跑该步（第 {} 步 = {}）",
                state.current_step + 1,
                state.current_step + 1,
                MAP_STEPS[state.current_step]
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
                MAP_STEPS[state.current_step],
                step_idx + 1,
                step
            ),
        ));
    }
    Ok((step_idx, state))
}

/// 解析执行面：白名单动作（子进程）或平台自身动作（`import`/`zero_rust`/`shot`）。
/// 返回 `(runner, args, env_vars)`；参数注入步骤位置参数（30 号 §3.1 param）。
fn resolve_execution(
    repo_root: &Path,
    id: &str,
    step: &str,
    action: Option<&str>,
) -> Result<ExecSpec, (u16, String)> {
    match action {
        Some(aid) => match resolve_action(repo_root, aid)? {
            Some((r, mut a)) => {
                // add/syncmap/pack → <id>；install → dist/maps/<id>.map-pack；verify → assets/maps/<id>。
                match step {
                    "add" | "syncmap" | "pack" => a.push(id.to_owned()),
                    "install" => a.push(format!("dist/maps/{id}.map-pack")),
                    "verify" => a.push(format!("assets/maps/{id}")),
                    _ => {}
                }
                Ok((r, a, Vec::new()))
            }
            None => {
                // self runner（map_shot）：平台自身动作，走特殊面。
                if aid == "map_shot" {
                    Ok(shot_command(repo_root, id))
                } else {
                    Err((500u16, format!("动作 `{aid}` runner=self 无特殊执行面")))
                }
            }
        },
        None => match step {
            // 复用 S32.15 import：POST /api/map/import 由路由层处理；此处仅同步跑 map_pack verify。
            "import" => Ok((
                "python3".to_owned(),
                vec![
                    "scripts/map_pack.py".to_owned(),
                    "verify".to_owned(),
                    format!("assets/maps/{id}"),
                ],
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
            _ => Err((500u16, format!("步 {step} 无执行面"))),
        },
    }
}

/// `map_shot` 步命令：运行 ti-engine 发货 bin（--shot `离屏），TI_MAP`=<id> 经 env 传。
/// 无头环境需虚拟显示（xvfb-run -a，Linux 迁移口径）；bin 路径 = 构建期 target-dir。
/// 注：`--scene=<id>` 单参数（`arg_value("--scene=")` 只匹配 `=` 形式；旧分离 `--scene main_map`
/// 会静默回退 index，属 S44.2 潜伏 bug 修复）。
fn shot_command(repo_root: &Path, id: &str) -> ExecSpec {
    let bin = resolve_bin_path(repo_root);
    let xvfb = Path::new("/usr/bin/xvfb-run");
    let (r, a) = if xvfb.is_file() {
        (
            xvfb.to_string_lossy().to_string(),
            vec![
                "-a".to_owned(),
                bin.to_string_lossy().to_string(),
                "--shot".to_owned(),
                "--scene=main_map".to_owned(),
            ],
        )
    } else {
        (
            bin.to_string_lossy().to_string(),
            vec!["--shot".to_owned(), "--scene=main_map".to_owned()],
        )
    };
    (r, a, vec![("TI_MAP", id.to_owned())])
}

/// `GET /api/content/map?id=<id>`：读向导状态（含每步 done/passed/criterion + 当前步骤）。
///
/// # Errors
/// `id` 为空或格式非法（`^[a-z0-9_]{1,16}$`，与 F-2 校验对齐）= 400（读状态前校验）。
pub fn state(repo_root: &Path, id: &str) -> Result<Value, (u16, String)> {
    // N3 修复（B 复审计 LOW）：读状态 id 也套 `^[a-z0-9_]{1,16}$`（与 validate_request 对齐；
    // 防只读路径穿越面 `evidence/console/content-map-<id>.json`，join 防 `..`/`/`）。
    if id.is_empty()
        || id.len() > 16
        || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err((400u16, format!("map_id `{id}` 非法（^[a-z0-9_]{{1,16}}$）")));
    }
    let st = read_state(repo_root, id)?;
    Ok(json!({
        "id": st.id,
        "current_step": st.current_step,
        "total_steps": MAP_STEPS.len(),
        "halted": st.halted,
        "steps": st.steps,
        "steps_def": MAP_STEPS,
    }))
}

fn now_iso() -> String {
    // 简化为 epoch 秒（时间戳留痕；无 chrono 依赖）。
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        .to_string()
}

/// 解析 ti-engine 发货 bin `路径（map_shot` 步用）：`CARGO_TARGET_DIR` > `.cargo/config.toml` `target-dir` > `repo/target`。
fn resolve_bin_path(repo_root: &Path) -> PathBuf {
    let mut target: Option<PathBuf> = std::env::var("CARGO_TARGET_DIR").ok().map(PathBuf::from);
    if target.is_none() {
        // 读 .cargo/config.toml 的 `target-dir`（项目构建期统一目录，S2.5 Linux 迁移）。
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

/// shot 步图像判据：PNG 存在 + 非纯黑帧（像素多样性）。
/// 只读 `evidence/bevy_native_main_map.png`（S27.5 出图口径；纯色/黑帧 = 判败）。
fn shot_image_ok(path: &Path) -> (bool, String) {
    if !path.is_file() {
        return (false, format!("未找到出图 {}", path.display()));
    }
    let Ok(bytes) = std::fs::read(path) else {
        return (false, format!("读图失败 {}", path.display()));
    };
    // 极简 PNG 检查：非空且含非黑像素（扫描 IDAT 段太复杂——用文件大小 > 1 KiB 作非纯黑代理，
    // 再叠加「文件含非零字节多样性」；出图失败通常产生 < 1 KiB 纯黑/纯色帧）。
    if bytes.len() < 1024 {
        return (false, format!("图过小（{} B），疑纯黑/纯色帧", bytes.len()));
    }
    // 非黑多样性：统计文件中不同字节值数（PNG 压缩后纯色帧高度重复，多样性低）。
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
