#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! S44.3 UI 向导单测（gate tests/ 目录内允许 unwrap/expect）。
//!
//! 只测**非 spawn 面**：`parse_criterion`（机器判据解析）/ `run_step` 的不可跳步 `409` +
//! 写动作无 `enable_write` `400` + 非法 id `400`（真实 spawn 是慢路径，归 e2e 验收）。
//! 仓库根 = `CARGO_MANIFEST_DIR/../..`（同 `close_tests` 口径，脚本须读真实仓库）。

use ti_console::content::ui_wizard::{parse_criterion, run_step, state};

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn criterion_design_shot_exit_zero() {
    // design/shot 步：exit 0 → 过（图像非黑判据由 run_step 侧复核）。
    let (ok, _) = parse_criterion("design", 0, "treasure-bevy-native --shot --scene=index");
    assert!(ok, "design exit0 应过");
    let (ok2, _) = parse_criterion("shot", 0, "");
    assert!(ok2, "shot exit0 应过");
    let (ok3, _) = parse_criterion("design", 1, "");
    assert!(!ok3, "design exit1 应判败");
}

#[test]
fn criterion_variant_requires_manifest_update() {
    // variant 步：exit 0 + 输出含「已更新」（ui_manifest register 面）→ 过。
    let (ok, _) =
        parse_criterion("variant", 0, "ui_variant: ui_manifest 已更新 3 条（source=ui_variant）");
    assert!(ok, "variant exit0 + 已更新 应过");
    let (ok2, _) = parse_criterion("variant", 0, "ui_variant tint: assets/ui/index_plank_0.png");
    assert!(!ok2, "variant exit0 无「已更新」应判败（register 未确认）");
    let (ok3, _) = parse_criterion("variant", 1, "");
    assert!(!ok3, "variant exit1 应判败");
}

#[test]
fn criterion_zero_rust_requires_empty() {
    // zero_rust 步：exit 0 + 输出空（git diff --quiet）→ 过；有 diff → 败。
    let (ok, _) = parse_criterion("zero_rust", 0, "");
    assert!(ok, "zero_rust exit0 空输出应过");
    let (ok2, _) = parse_criterion("zero_rust", 1, "");
    assert!(!ok2, "zero_rust exit1 应判败（有 diff）");
}

#[test]
fn criterion_build_zero() {
    let (ok, _) = parse_criterion("build", 0, "Finished dev profile");
    assert!(ok, "build exit0 应过");
}

#[test]
fn run_step_rejects_unknown_step() {
    // 未知 step = 400。
    let r = run_step(&repo_root(), "ui99", "nonexistent", true, &serde_json::json!({}));
    assert_eq!(r.unwrap_err().0, 400);
}

#[test]
fn run_step_rejects_invalid_id() {
    // 非法 id（非 ^[a-z0-9_]+$）= 400。
    let r = run_step(&repo_root(), "BAD-ID!", "design", true, &serde_json::json!({}));
    assert_eq!(r.unwrap_err().0, 400);
}

#[test]
fn run_step_no_skip_returns_409() {
    // 不可跳步：未过第 1 步判据时请求第 2 步（variant）= 409（前置未过）。
    // 用未初始化的新 id（状态文件不存在 = current_step 0 = 待执行 design）。
    let r = run_step(&repo_root(), "ui_notexist_99", "variant", true, &serde_json::json!({}));
    let err = r.unwrap_err();
    assert_eq!(err.0, 409, "不可跳步应 409：{err:?}");
}

#[test]
fn run_step_write_action_requires_enable_write() {
    // 写动作（design，写 evidence/）无 enable_write = 400（30 号 §3.1 默认关）。
    // 用新 id（不实际写盘——enable_write=false 早退，未触发 spawn）。
    let r = run_step(&repo_root(), "ui_nw_99", "design", false, &serde_json::json!({}));
    let err = r.unwrap_err();
    assert_eq!(err.0, 400, "写动作无 enable_write 应 400：{err:?}");
}

#[test]
fn state_rejects_bad_id() {
    // 读状态 id 也套 ^[a-z0-9_]{1,16}$（与 validate_request 对齐）。
    let r = state(&repo_root(), "../etc/passwd");
    assert_eq!(r.unwrap_err().0, 400, "state 坏 id 应 400");
    let r2 = state(&repo_root(), "UI99"); // 大写 → 400
    assert_eq!(r2.unwrap_err().0, 400, "state 大写 id 应 400");
}

// ---- F3（B 审计 r1 P1）：merge_params 参数校验（scene/viewport/assets/suffix 路径穿越/注入防）----

#[test]
fn run_step_rejects_bad_scene_param() {
    // scene 含路径穿越（../）→ 400（F3）。
    let r = run_step(
        &repo_root(),
        "ui_f3_01",
        "design",
        true,
        &serde_json::json!({"scene": "../../etc/passwd"}),
    );
    assert_eq!(r.unwrap_err().0, 400, "scene 路径穿越应 400");
}

#[test]
fn run_step_rejects_bad_viewport_param() {
    // viewport 非法格式 → 400（F3）。
    let r = run_step(
        &repo_root(),
        "ui_f3_02",
        "design",
        true,
        &serde_json::json!({"viewport": "19x10x80"}),
    );
    assert_eq!(r.unwrap_err().0, 400, "viewport 非法应 400");
    // 合法 WxH 不报错（设计步 enable_write=false 早退于 enable_write 门，不会 spawn）。
    let r2 = run_step(
        &repo_root(),
        "ui_f3_02b",
        "design",
        false,
        &serde_json::json!({"viewport": "1920x1080"}),
    );
    let err = r2.unwrap_err();
    assert_eq!(err.0, 400, "合法 viewport 但写动作无 enable_write 应 400（未到 spawn）：{err:?}");
}

#[test]
fn run_step_rejects_bad_assets_param() {
    // assets 含注入字符 → 400（F3）。
    let r = run_step(
        &repo_root(),
        "ui_f3_03",
        "design",
        true,
        &serde_json::json!({"assets": "index_plank_0;rm -rf /"}),
    );
    assert_eq!(r.unwrap_err().0, 400, "assets 注入应 400");
}

#[test]
fn run_step_rejects_color_out_of_range() {
    // color 越界（>3）→ 400（F3 参数域 [0,3]）。
    let r = run_step(&repo_root(), "ui_f3_04", "design", true, &serde_json::json!({"color": 9.9}));
    assert_eq!(r.unwrap_err().0, 400, "color 越界应 400");
}

#[test]
fn run_step_rejects_bad_suffix_param() {
    // suffix 含 `..`/注入 → 400（F3）。
    let r = run_step(
        &repo_root(),
        "ui_f3_05",
        "design",
        true,
        &serde_json::json!({"suffix": "../evil"}),
    );
    assert_eq!(r.unwrap_err().0, 400, "suffix 路径穿越应 400");
}

// ---- F2（B 审计 r1 P1）：zero_rust 步判据须含视觉签收留痕 ----

/// 构造一个 `current_step=5`（前 5 步已过）的 state JSON，写入 `state_path`。
fn seed_state_at_step6(repo: &std::path::Path, id: &str, signoff: &str) {
    let steps: Vec<serde_json::Value> = [
        "design",
        "variant",
        "layout",
        "build",
        "shot",
        "zero_rust",
    ]
    .iter()
    .enumerate()
    .map(|(i, name)| {
        serde_json::json!({
            "name": name,
            "done": i < 5,
            "passed": i < 5,
            "cmd": "",
            "exit_code": if i < 5 { serde_json::Value::from(0) } else { serde_json::Value::Null },
            "criterion": "",
            "ts": serde_json::Value::Null,
        })
    })
    .collect();
    let state = serde_json::json!({
        "id": id,
        "scene": "index",
        "viewport": "1920x1080",
        "assets": "index_plank_0",
        "color": 0.32,
        "brightness": 1.10,
        "rmult": 0.88,
        "gmult": 1.00,
        "bmult": 1.20,
        "signoff": signoff,
        "current_step": 5,
        "halted": false,
        "steps": steps,
    });
    let p = repo.join("evidence/console").join(format!("content-ui-{id}.json"));
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(&p, serde_json::to_string_pretty(&state).unwrap()).unwrap();
}

#[test]
fn run_step_zero_rust_requires_signoff() {
    // F2：zero_rust 步无签收（signoff 空）→ 判败（ok=false），即使 git diff crates/ 为 0。
    // 真实 spawn git diff --quiet（只读，快）；写路径 = [] ⇒ 免 enable_write。
    let id = "ui_f2_nosignoff";
    seed_state_at_step6(&repo_root(), id, "");
    let r = run_step(&repo_root(), id, "zero_rust", false, &serde_json::json!({}));
    let v = r.unwrap_or_else(|e| panic!("zero_rust 应可执行（非 Err）：{e:?}"));
    assert_eq!(v["ok"], false, "无签收应判败：{v}");
    assert!(
        v["criterion"].as_str().unwrap_or("").contains("签收留痕"),
        "判据应含签收留痕说明：{v}"
    );
    // 清理（tests 允许写盘，但不留痕）。
    let _ = std::fs::remove_file(
        repo_root().join("evidence/console").join(format!("content-ui-{id}.json")),
    );
}

#[test]
fn run_step_zero_rust_passes_with_signoff() {
    // F2 正例：signoff 含「用户视觉验收」→ 判过（git diff crates/ 为 0 空工作树）。
    let id = "ui_f2_signoff";
    seed_state_at_step6(
        &repo_root(),
        id,
        "用户 2026-11-06 S44.3 UI 工作台 e2e 视觉验收通过（S39 横版 index_plank 绳挂木牌冷调变体，1920x1080）",
    );
    let r = run_step(&repo_root(), id, "zero_rust", false, &serde_json::json!({}));
    let v = r.unwrap_or_else(|e| panic!("zero_rust 应可执行：{e:?}"));
    // 若 git diff crates/ 有未提交改动（当前 r1 修复中），判据 = diff 非空 → 判败；但**签收判据**本身应过。
    assert!(
        v["criterion"].as_str().unwrap_or("").contains("签收留痕判据：state.signoff 含"),
        "有签收应过签收判据：{v}"
    );
    let _ = std::fs::remove_file(
        repo_root().join("evidence/console").join(format!("content-ui-{id}.json")),
    );
}
