//! S44.4 打包/分发面（30 号 §2.3）：`dist/maps/` 可分发件清单只读适配。
//!
//! `dist/` 是 gitignored 产物目录（地图打包 `mappack_pack` 生成），缺失不报错（空清单）。
//! 清单项 = `{name, size, mtime}`：`size` = 目录递归字节和，`mtime` = epoch 秒（无 chrono 依赖，
//! 与 `map_wizard`/`ui_wizard` 时间戳留痕同口径；缺失 = null）。

use std::path::Path;

use serde_json::Value;

/// `dist/maps/*.map-pack` 可分发件清单（gitignored；缺失 = 空清单，不报错）。
/// 每项 = `{name, size, mtime}`（size = 目录递归字节和；mtime = epoch 秒，缺失 = null）。
#[must_use]
pub fn dist_pack_names(repo_root: &Path) -> Vec<Value> {
    let dist = repo_root.join("dist/maps");
    let mut items: Vec<Value> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dist) {
        for entry in rd.flatten() {
            let n = entry.file_name().to_string_lossy().to_string();
            if !n.ends_with(".map-pack") {
                continue;
            }
            let path = entry.path();
            let size = dir_size(&path);
            let mtime = entry.metadata().ok().and_then(|m| m.modified().ok()).and_then(|t| {
                // 简化为 epoch 秒（无 chrono 依赖，与 map_wizard/ui_wizard 时间戳留痕同口径；缺失 = null）。
                t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())
            });
            items.push(serde_json::json!({
                "name": n,
                "size": size,
                "mtime": mtime,
            }));
        }
    }
    // 按 name 稳定排序（幂等，前端防抖动）。
    items.sort_by(|a, b| {
        let na = a.get("name").and_then(Value::as_str).unwrap_or("");
        let nb = b.get("name").and_then(Value::as_str).unwrap_or("");
        na.cmp(nb)
    });
    items
}

/// 目录递归字节和（可分发件 `.map-pack/` 目录；单文件 = 其大小；缺失 = 0）。
fn dir_size(path: &Path) -> u64 {
    let mut total: u64 = 0;
    if path.is_dir() {
        if let Ok(rd) = std::fs::read_dir(path) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    total = total.saturating_add(dir_size(&p));
                } else if let Ok(m) = e.metadata() {
                    total = total.saturating_add(m.len());
                }
            }
        }
    } else if let Ok(m) = path.metadata() {
        total = m.len();
    }
    total
}
