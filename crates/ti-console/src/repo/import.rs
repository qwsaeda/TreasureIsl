//! 地图包导入写面（S32.14 ②：平台编辑器出描述符 → `POST /api/map/import` 落盘 + 回注）。
//!
//! 写点纪律（22 号北极星「0 游戏产物变更」的唯一受控例外）：
//! - **唯一写点**：只写 `assets/maps/<id>/{manifest,map,layout,strings}.json` 四文件（`id` = `^[a-z0-9_]{1,16}$`，
//!   防穿越/防目录注入）；`write_whitelist` 已扩 `assets/maps/`（`console_sources.json`），
//!   但**本模块是仓库内唯一往该目录写盘的代码路径**（平台其余 0 写盘）。
//! - 写完后**同进程先判后回注**（S32.15 审计 r1 F-2 修）：`python3 scripts/map_pack.py verify <id>`
//!   （I1~I8 终判，含 I7 资产哈希）PASS 才 `sync`（回注 `board.json` `maps` 段）；
//!   `verify` 非 0 = 不 sync（`board.json` 不留坏 `maps` 段，无需回滚；包目录保留供修复重导），
//!   导入结果如实上报 `errors`，**不静默绕过**（与 21 号 §6「版本/不变量不符即拒」同口径）。
//! - `strings.json` 由 `id` 派生文案键（`glmap<id>.Map`），4 文件由请求体 JSON 生成（前端已做
//!   I1~I8 结构判定，本模块是落盘前的最后一道 schema/键完整性校验）。
//!
//! 0 游戏依赖：只用 `std` + `serde_json` + 子进程（python3），不引 ti-core/ti-engine/bevy。

use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::repo::config::RepoSources;

/// 地图包 `id` 合法字符集（`^[a-z0-9_]{1,16}$`；小写 + 数字 + 下划线，长度 1~16）。
fn is_legal_pack_id(id: &str) -> bool {
    let bytes = id.as_bytes();
    (1..=16).contains(&bytes.len())
        && bytes
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == &b'_')
}

/// 4 文件键完整性校验（`manifest` 必须带 `schema`/`id`/`data_version`/`map`/`layout`/`strings`；
/// `map` 必须带 `spaces` 数组；`layout` 必须带 `ring`）。缺 = 带原因 `Err`（不落盘）。
fn check_payload_shape(payload: &Value) -> Result<&Value, String> {
    let manifest = payload
        .get("manifest")
        .ok_or("请求体缺 `manifest`")?;
    for key in ["schema", "id", "data_version", "map", "layout", "strings"] {
        if manifest.get(key).is_none() {
            return Err(format!("manifest 缺必需键 `{key}`"));
        }
    }
    if manifest["map"].get("spaces").and_then(Value::as_array).is_none() {
        return Err("manifest.map 缺 `spaces` 数组".to_owned());
    }
    if manifest["layout"].get("ring").is_none() {
        return Err("manifest.layout 缺 `ring`".to_owned());
    }
    Ok(manifest)
}

/// 导入一张地图包（写 4 文件 + 回注 + 终判）。
///
/// 返回 JSON 结果对象（`ok` 布尔 + `written` 4 文件 + `synced`/`verified` 状态 + `errors` 清单）。
/// 失败 = `ok=false` + 原因（调用方 4xx 如实上报，不静默；本函数恒 `Ok`，`Err` 无实际分支）。
pub fn import_map_pack(repo_root: &Path, _sources: &RepoSources, payload: &Value) -> Value {
    // `_sources` = `console_sources.json` 数据源面（`write_whitelist` 语义 = 数据面登记「平台可写目录」，
    // 实码写点防线 = id 正则 + `assert_write_point` canonicalize 前缀断言；本函数不消费白名单做门禁。
    // 参数保留与 `repo::` 模块签名一致，S32.4 鉴权面启用时在此接 token + 白名单消费校验）。
    // 1) 形状校验（缺键不落盘）。
    let manifest = match check_payload_shape(payload) {
        Ok(m) => m,
        Err(reason) => {
            return json!({ "ok": false, "error": reason });
        }
    };

    // 2) id 白名单校验（防穿越）。
    let pack_id = manifest
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    if !is_legal_pack_id(&pack_id) {
        return json!({
            "ok": false,
            "error": format!("非法包 id `{pack_id}`（须 ^[a-z0-9_]{{1,16}}$）")
        });
    }

    // 3) 写 4 文件到 assets/maps/<id>/（唯一写点；防穿越断言 + 目录存在则覆盖，缺则创建）。
    let pack_dir = repo_root.join("assets").join("maps").join(&pack_id);
    let _canon = match assert_write_point(repo_root, &pack_dir) {
        Ok(c) => c,
        Err(reason) => {
            return json!({ "ok": false, "error": reason });
        }
    };
    let written = match write_four_files(repo_root, &pack_dir, manifest, payload) {
        Ok(files) => files,
        Err(reason) => {
            return json!({ "ok": false, "error": reason });
        }
    };

    // 4) 终判 I1~I8（verify 裸 id；含 I7 资产哈希）——**先判后回注**（S32.15 审计 r1 F-2 修）：
    //    verify PASS 才 sync 回注 board.json maps 段，FAIL = 不 sync（board.json 不留坏段，
    //    无需回滚；包目录保留供修复后重导）。无 map 键草稿包 sync 自动跳过。
    let (verified, verify_detail) = run_map_pack(repo_root, "verify", Some(pack_id.as_str()));
    let mut errors = Vec::new();
    if !verified {
        errors.push(verify_detail.clone());
    }
    let (synced, sync_detail) = if verified {
        run_map_pack(repo_root, "sync", None)
    } else {
        (false, "verify FAIL，跳过 sync（board.json 不留坏 maps 段；修复包后重导）".to_owned())
    };
    if !synced {
        errors.push(sync_detail);
    }

    let mut out = json!({
        "ok": verified && synced,
        "pack_id": pack_id,
        "written": written,
        "synced": synced,
        "verified": verified,
        "next": format!("map_pack.py pack {pack_id}  出可分发包 → dist/maps/{pack_id}.map-pack"),
    });
    if !errors.is_empty() {
        out["errors"] = json!(errors);
    }
    out
}

/// 写 4 文件（manifest/map/layout/strings；`map`/`layout`/`strings` 顶层段缺则取 manifest 内嵌段）。
fn write_four_files(
    repo_root: &Path,
    pack_dir: &Path,
    manifest: &Value,
    payload: &Value,
) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(pack_dir).map_err(|e| format!("建包目录 {} 失败：{e}", pack_dir.display()))?;

    let map_body = payload.get("map").unwrap_or(&manifest["map"]);
    let layout_body = payload.get("layout").unwrap_or(&manifest["layout"]);
    let strings_body = payload.get("strings").unwrap_or(&manifest["strings"]);

    let files: [(&str, &Value); 4] = [
        ("manifest.json", manifest),
        ("map.json", map_body),
        ("layout.json", layout_body),
        ("strings.json", strings_body),
    ];
    let mut written = Vec::new();
    for (name, body) in &files {
        let target = pack_dir.join(name);
        let text = serde_json::to_string_pretty(body)
            .map_err(|e| format!("序列化 {name} 失败：{e}"))?;
        std::fs::write(&target, text + "\n")
            .map_err(|e| format!("写 {} 失败：{e}", target.display()))?;
        // 写点相对路径（assets/maps/<id>/<name>；供响应回报，strip 失败回退全路径）。
        let rel = target
            .strip_prefix(repo_root)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or(target.to_string_lossy().into_owned());
        written.push(rel);
    }
    Ok(written)
}

/// 跑 `python3 scripts/map_pack.py <sub> [args]`（同进程；返回 `(是否成功, 输出)`）。
/// `verify` 带包 id（裸 id = 已装包目录）；`sync` 无参（全量重算 board.json maps 段）。
fn run_map_pack(repo_root: &Path, sub: &str, verify_pack_id: Option<&str>) -> (bool, String) {
    let mut cmd = std::process::Command::new("python3");
    cmd.arg("scripts/map_pack.py").arg(sub);
    if let Some(pack_id) = verify_pack_id {
        cmd.arg(pack_id);
    }
    let out = cmd.current_dir(repo_root).output();
    match out {
        Ok(child) => {
            let stdout = String::from_utf8_lossy(&child.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&child.stderr).into_owned();
            let ok = child.status.success();
            let detail = if stderr.trim().is_empty() {
                stdout.trim().to_owned()
            } else {
                stderr.trim().to_owned()
            };
            (ok, format!("map_pack.py {sub}: {} {detail}", if ok { "OK" } else { "FAIL" }))
        }
        Err(e) => (false, format!("跑 map_pack.py {sub} 失败：{e}")),
    }
}

/// 防穿越断言：写点必须落在 `repo_root/assets/maps/` 下（canonicalize 后前缀比对，22 号 §10.2 口径）。
/// 包目录可能尚不存在（首次导入），故先建目录再 canonicalize。
///
/// # Errors
///
/// 建目录 / canonicalize 失败 = 带原因 `Err`。
fn assert_write_point(repo_root: &Path, pack_dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(pack_dir).map_err(|e| format!("建包目录 {} 失败：{e}", pack_dir.display()))?;
    let canon_root = std::fs::canonicalize(repo_root)
        .map_err(|e| format!("canonicalize repo_root 失败：{e}"))?;
    let canon_dir = std::fs::canonicalize(pack_dir)
        .map_err(|e| format!("canonicalize 包目录 失败：{e}"))?;
    if !canon_dir.starts_with(canon_root.join("assets").join("maps")) {
        return Err(format!(
            "写点越界（非 assets/maps/ 下）：{}",
            canon_dir.display()
        ));
    }
    Ok(canon_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_pack_id_rules() {
        assert!(is_legal_pack_id("m04"));
        assert!(is_legal_pack_id("isle_map_2"));
        assert!(!is_legal_pack_id(""));
        assert!(!is_legal_pack_id("UPPER"));
        assert!(!is_legal_pack_id("bad../x"));
        assert!(!is_legal_pack_id(&"a".repeat(17)));
    }

    #[test]
    fn payload_shape_rejects_missing_keys() {
        let bad = json!({ "manifest": { "schema": "map-pack/1" } });
        assert!(check_payload_shape(&bad).is_err());
    }

    #[test]
    fn payload_shape_accepts_full() {
        let ok = json!({
            "manifest": {
                "schema": "map-pack/1",
                "id": "m04",
                "data_version": "board-v5",
                "map": { "spaces": [{}] },
                "layout": { "ring": {} },
                "strings": { "glmapm04.Map": "m04" }
            }
        });
        assert!(check_payload_shape(&ok).is_ok());
    }

    #[test]
    fn write_point_asserts_stay_inside_assets_maps() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        // canonicalize 成功 = 真实仓库路径（可判越界）；失败面（测试环境异常）= 跳过越界断言（正常写点面仍判）。
        let canon_root: Option<std::path::PathBuf> = manifest_dir.canonicalize().ok();
        if let Some(root) = &canon_root {
            // 正常写点（assets/maps/<id>）= 断言通过。
            let ok_dir = root.join("assets").join("maps").join("m04");
            assert!(assert_write_point(root, &ok_dir).is_ok());
            // 越界写点（含 `..` 爬到 assets/ 外）= canonicalize 后前缀断言拒。
            let bad_dir = root.join("assets").join("maps").join("../..").join("evil");
            assert!(assert_write_point(root, &bad_dir).is_err());
            // repo_root 不存在 = 断言 Err（canonicalize 失败面；包目录先建再判）。
            let probe_dir = root.join("assets").join("maps").join("m04_missing_root_probe");
            let _ = std::fs::create_dir_all(&probe_dir);
            assert!(assert_write_point(&root.join("no_such_subdir"), &probe_dir).is_err());
            let _ = std::fs::remove_dir(&probe_dir);
        } else {
            // 降级面：manifest 相对路径跑正常写点（canonicalize 失败时前缀断言退化为 join 相对口径）。
            let ok_dir = manifest_dir.join("assets").join("maps").join("m04");
            assert!(assert_write_point(manifest_dir, &ok_dir).is_ok());
        }
    }
}
