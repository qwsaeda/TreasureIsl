//! UI 清单登记契约（`scripts/ui_assets.py register-plot` = 清单唯一写入路径）。
//!
//! 背景（2026-11-10）：平台 `POST /api/gen-plot` 的登记旧实现用 `serde_json::to_string_pretty`
//! 重写整个 `assets/ui_manifest.json` → 2 空格 + 键序重排 = **每次地块生图全文件 churn**。
//! 修法 = 登记改走 `scripts/ui_assets.py register-plot`（`json.dumps(indent=2)` 保序）→ 零 churn。
//!
//! 判据：
//! ① 登记后清单含 `roles.plot` + `assets[tile_<map_id>_<格号>]`（disk↔registry 同步所需条目）；
//! ② **二次登记清单逐字节不变**（幂等 + 零 churn：重复登记不重排缩进/键序）；
//! ③ 登记只写清单、不生图（临时 repo 内不产生任何 PNG 文件）。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// 仓库根（crates/ti-console → crates → 根）。
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("仓库根")
        .to_path_buf()
}

/// Python 解释器（Linux/macOS = python3；Windows = python；`TI_PYTHON` 可覆盖）。
fn python_cmd() -> &'static str {
    static CMD: OnceLock<String> = OnceLock::new();
    CMD.get_or_init(|| {
        if let Ok(v) = std::env::var("TI_PYTHON") {
            return v;
        }
        for cand in ["python3", "python"] {
            if Command::new(cand).arg("--version").output().is_ok() {
                return cand.to_owned();
            }
        }
        "python3".to_owned()
    })
}

/// 临时 repo：`scripts/ui_assets.py`（脚本按自身位置算 ROOT）+ `assets/ui_manifest.json` 基线。
fn temp_repo(tag: &str) -> PathBuf {
    let base =
        std::env::temp_dir().join(format!("ti_manifest_register_{tag}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(base.join("scripts")).expect("建 scripts 目录");
    fs::create_dir_all(base.join("assets")).expect("建 assets 目录");
    fs::copy(
        repo_root().join("scripts").join("ui_assets.py"),
        base.join("scripts").join("ui_assets.py"),
    )
    .expect("复制 ui_assets.py");
    // 基线清单 = 工具标准格式（indent=2），登记后应**只增量**该条目
    fs::write(
        base.join("assets").join("ui_manifest.json"),
        "{\n  \"version\": 1,\n  \"_comment\": \"base\",\n  \"roles\": {},\n  \"assets\": []\n}\n",
    )
    .expect("写基线清单");
    base
}

fn run_register(root: &Path, map_id: &str, idx: &str) {
    let out = Command::new(python_cmd())
        .arg(root.join("scripts").join("ui_assets.py"))
        .arg("register-plot")
        .arg("--map-id")
        .arg(map_id)
        .arg("--idx")
        .arg(idx)
        .arg("--what")
        .arg("沉船滩地块提示词")
        .current_dir(root)
        .output()
        .expect("跑 register-plot");
    assert!(
        out.status.success(),
        "register-plot 非 0 退出：{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn manifest_text(root: &Path) -> String {
    fs::read_to_string(root.join("assets").join("ui_manifest.json")).expect("读清单")
}

#[test]
fn register_plot_adds_role_and_asset_entry() {
    let root = temp_repo("add");
    run_register(&root, "m04", "7");
    let text = manifest_text(&root);
    let v: serde_json::Value = serde_json::from_str(&text).expect("清单是合法 JSON");

    // roles.plot 补齐（1:1 透明地块插画模板）
    let role = &v["roles"]["plot"];
    assert_eq!(role["ratio"], "1:1", "roles.plot.ratio 应为 1:1");
    assert!(
        role["prompt"].as_str().unwrap_or_default().contains("{what}"),
        "roles.plot.prompt 含 {{what}} 占位"
    );

    // assets 条目登记（按格唯一：tile_<map_id>_<格号>）
    let assets = v["assets"].as_array().expect("assets 是数组");
    let entry = assets.iter().find(|a| a["name"] == "tile_m04_7").expect("assets 含 tile_m04_7");
    assert_eq!(entry["role"], "plot");
    assert_eq!(entry["file"], "assets/ui/tile_m04_7.png");
    assert_eq!(entry["params"]["what"], "沉船滩地块提示词");

    // 只登记不生图（临时 repo 内不产生 PNG）
    let pngs: Vec<_> = fs::read_dir(root.join("assets"))
        .expect("读 assets 目录")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .collect();
    assert!(pngs.is_empty(), "register-plot 不该生图（发现 PNG：{pngs:?}）");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn register_plot_is_idempotent_and_zero_churn() {
    let root = temp_repo("idem");
    run_register(&root, "m04", "7");
    let first = manifest_text(&root);
    run_register(&root, "m04", "7");
    let second = manifest_text(&root);

    // 幂等 + 零 churn：二次登记**逐字节不变**（不重排缩进/键序、不覆盖既有条目）
    assert_eq!(first, second, "二次登记应逐字节不变（幂等 + 零 churn）");
    // 首次登记相对基线只增量该条目（不整文件重排）
    assert!(first.contains("\"tile_m04_7\""), "首次登记应含 tile_m04_7 条目");
    assert!(!first.contains("\"version\": 1,\n \"version\""), "不应出现键序重排痕迹");

    let _ = fs::remove_dir_all(&root);
}
