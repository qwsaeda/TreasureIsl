#![allow(clippy::unwrap_used, clippy::expect_used)]

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
