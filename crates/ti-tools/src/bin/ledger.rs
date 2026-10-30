//! `gate.ps1` 第 8 项：校验 `plan/ledger.yaml` 并核对证据文件。
//!
//! 用法：`cargo run -q -p ti-tools -- ledger [path/to/ledger.yaml]`
//! （缺省路径为 `plan/ledger.yaml`，证据相对仓库根目录解析。）

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_devmeta::validate;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let ledger = args.first().map_or_else(|| PathBuf::from("plan/ledger.yaml"), PathBuf::from);
    let base = Path::new(".");

    let text = match std::fs::read_to_string(&ledger) {
        Ok(text) => text,
        Err(err) => {
            eprintln!(
                "ledger: cannot read {}: {err} (cwd={:?})",
                ledger.display(),
                std::env::current_dir()
            );
            return ExitCode::FAILURE;
        }
    };

    let report = validate(&text);
    println!(
        "ledger: {} tasks, {} verified, {} error(s)",
        report.tasks.len(),
        report.verified_count(),
        report.errors.len()
    );
    for err in &report.errors {
        println!("  ERROR {err}");
    }

    let mut failed = !report.ok();
    let missing = ti_devmeta::missing_evidence(&report.tasks, base);
    if missing.is_empty() {
        println!("  evidence: ok");
    } else {
        failed = true;
        println!("  ERROR verified tasks missing evidence file: {}", missing.join(", "));
    }

    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
