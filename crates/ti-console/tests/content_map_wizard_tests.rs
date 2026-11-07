#![allow(clippy::unwrap_used, clippy::expect_used)]

//! S44.2 地图向导单测（src 内联测试迁移；gate tests/ 目录内允许 unwrap/expect）。
//!
//! 只测**非 spawn 面**：`parse_criterion`（机器判据解析）/ `resolve_write_paths`（写点占位符解析）/
//! `run_step` 的不可跳步 `409` + 写动作无 `enable_write` `400`（真实 spawn 是慢路径，归 e2e 验收）。
//! 仓库根 = `CARGO_MANIFEST_DIR/../..`（同 `close_tests` 口径，脚本须读真实仓库）。

use ti_console::content::map_wizard::{parse_criterion, resolve_write_paths, run_step};

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn criterion_verify_zero_violations() {
    // verify/check 步：exit 0 + 输出含「0 违规/0 漂移/0 差」→ 过。
    let (ok, desc) = parse_criterion("verify", 0, "map_pack verify: m01 0 违规（I1~I9）");
    assert!(ok, "verify 应判过：{desc}");
    let (ok2, _) = parse_criterion("verify", 1, "map_pack verify: 违规");
    assert!(!ok2, "verify exit 1 应判败");
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
fn criterion_syncmap_zero() {
    let (ok, _) = parse_criterion("syncmap", 0, "map_pack syncmap: m01 manifest[map] ← map.json");
    assert!(ok);
}

#[test]
fn write_paths_resolves_id_placeholder() {
    // 30 号 §3.2 占位符解析约定：<id> → 实际包 id。
    // 写点数据单一事实源 = `console_commands.json` 的 `writes`（S46.1 整理：平台侧不再另存一份）。
    let p = resolve_write_paths(&repo_root(), "mappack_syncmap", "m01");
    assert_eq!(p, vec!["assets/maps/m01/manifest.json"]);
    let p2 = resolve_write_paths(&repo_root(), "mappack_pack", "m22");
    assert_eq!(p2, vec!["dist/maps/m22.map-pack/"]);
    let p3 = resolve_write_paths(&repo_root(), "mappack_add", "m99");
    assert_eq!(p3, vec!["assets/maps/"]);
    // 只读动作 = 空写点集（`mappack_verify` / `mappack_check` 的 writes 为 []）。
    let p4 = resolve_write_paths(&repo_root(), "mappack_verify", "m01");
    assert!(p4.is_empty(), "只读动作应无写点：{p4:?}");
    // 未登记动作 = 空写点集（不 panic、不静默当成有写点）。
    let p5 = resolve_write_paths(&repo_root(), "no_such_action", "m01");
    assert!(p5.is_empty(), "未登记动作应无写点：{p5:?}");
}

#[test]
fn run_step_rejects_unknown_step() {
    // 未知 step = 400。
    let r = run_step(&repo_root(), "m01", "nonexistent", true, &serde_json::json!({}));
    assert_eq!(r.unwrap_err().0, 400);
}

#[test]
fn run_step_rejects_invalid_id() {
    // 非法 id（非 ^[a-z0-9_]+$）= 400。
    let r = run_step(&repo_root(), "BAD-ID!", "add", true, &serde_json::json!({}));
    assert_eq!(r.unwrap_err().0, 400);
}

#[test]
fn run_step_no_skip_returns_409() {
    // 不可跳步：未过第 1 步判据时请求第 2 步（import）= 409（前置未过）。
    // 用未初始化的新 id（状态文件不存在 = current_step 0 = 待执行 add）。
    let r = run_step(&repo_root(), "wiz_notexist_99", "import", true, &serde_json::json!({}));
    let err = r.unwrap_err();
    assert_eq!(err.0, 409, "不可跳步应 409：{err:?}");
}

#[test]
fn run_step_write_action_requires_enable_write() {
    // 写动作（add）无 enable_write = 400（30 号 §3.1 默认关）。
    // 用新 id（不实际写盘——enable_write=false 早退，未触发 spawn）。
    let r = run_step(&repo_root(), "wiz_nw_99", "add", false, &serde_json::json!({}));
    let err = r.unwrap_err();
    assert_eq!(err.0, 400, "写动作无 enable_write 应 400：{err:?}");
}
