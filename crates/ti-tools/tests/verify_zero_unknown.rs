#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! S46.0 `verify --zero-unknown` 判据载体单测（gate tests/ 目录内允许 unwrap/expect）。
//!
//! 口径：全格式 0 未知字节（P0-2）= 每个格式解析器消费了文件全部字节（结构后无残留）。
//! 本测试实测真实 `legacy/data/` 数据面（与门禁第 7 项同一命令同一数据源）。

use std::path::Path;
use std::process::Command;

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn zero_unknown_verify_passes_on_real_data() {
    // 实跑 `verify --zero-unknown`（与 gate 第 7 项同命令）：0 失败 ⇒ exit 0。
    let out = Command::new(env!("CARGO_BIN_EXE_verify"))
        .args(["--zero-unknown"])
        .current_dir(repo_root())
        .output()
        .expect("verify bin 应可执行");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "verify --zero-unknown 应 0 失败（exit={:?}）；stderr={stderr}",
        out.status.code()
    );
    assert!(
        stdout.contains("PASS：全部可解析格式文件 0 未知字节"),
        "报告应含 PASS 结论；stdout 尾段={}",
        stdout.lines().last().unwrap_or_default()
    );
}

#[test]
fn zero_unknown_verify_json_reports_zero_failed() {
    // `--json` 机器可读：ok=true + failed=0 + 文件/条目计数 > 0。
    let out = Command::new(env!("CARGO_BIN_EXE_verify"))
        .args(["--zero-unknown", "--json"])
        .current_dir(repo_root())
        .output()
        .expect("verify bin 应可执行");
    assert!(out.status.success(), "verify --json 应 exit 0");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("JSON 输出应可解析");
    assert_eq!(v["ok"], serde_json::json!(true), "ok 应为 true");
    assert_eq!(v["failed"], serde_json::json!(0), "failed 应为 0");
    assert_eq!(v["skipped"], serde_json::json!(0), "skipped 应为 0（全部文件有解析器）");
    assert!(
        v["files"].as_u64().unwrap_or(0) >= 30,
        "文件数应 ≥ 30（真实 legacy/data 面），实际={}",
        v["files"]
    );
    assert!(v["entries"].as_u64().unwrap_or(0) >= 300, "容器条目数应 ≥ 300，实际={}", v["entries"]);
}

#[test]
fn zero_unknown_requires_flag() {
    // 无参 = 打印用法并退出码 2（防"以为跑了其实没跑"）。
    let out = Command::new(env!("CARGO_BIN_EXE_verify"))
        .current_dir(repo_root())
        .output()
        .expect("verify bin 应可执行");
    assert_eq!(out.status.code(), Some(2), "无参应退出码 2");
    assert!(String::from_utf8_lossy(&out.stderr).contains("用法"), "无参应打印用法到 stderr");
}

#[test]
fn legacy_data_dir_exists_with_formats() {
    // 前置：真实数据面在位（判据载体读的是它；缺失则门禁第 7 项会误判）。
    let data = repo_root().join("legacy/data");
    assert!(data.is_dir(), "legacy/data 应存在");
    let names: Vec<String> = std::fs::read_dir(&data)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    for expect in [
        "GoldLand.Fnt",
        "font.dat",
        "settings.dat",
        "ui.pak",
        "glpics.pak",
        "music.lpk",
        "main.Map",
        "glmapm01.Map",
    ] {
        assert!(names.iter().any(|n| n == expect), "legacy/data 应含 {expect}；实际={names:?}");
    }
    assert!(Path::new(&repo_root().join("legacy/data")).is_dir(), "数据目录可定位");
}
