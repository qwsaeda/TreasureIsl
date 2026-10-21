//! S18.8b：`image` 原语的 **CPU 侧**贴图资源缓存（进程级）。
//!
//! 资源路径约定 = **相对仓库根**（如 `assets/ui/board.png`，与 Bevy 资产服务器同口径）：
//! - CPU 后端：`<仓库根>/assets/<路径>` → 读文件 + PNG 解码（`ti-platform::hd_png`）；
//! - Bevy 后端：资产服务器同口径解析（`<CWD>/assets/<路径>`，壳从仓库根启动）。
//!
//! 缺资源 / 解码失败 = `None`（调用方跳过绘制；**不 panic、不留坑**）。

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::{Mutex, OnceLock};

use ti_platform::hd_png::HdPng;

type Cache = Mutex<HashMap<String, Option<Arc<HdPng>>>>;

/// 进程级缓存（首次解码后复用）。
fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 资源根（**仓库根**；与 Bevy `AssetPlugin.file_path` 同口径 —— 资产路径 = 相对仓库根）。
fn assets_root() -> Option<std::path::PathBuf> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().and_then(|p| p.parent()).map(std::path::PathBuf::from)
}

/// 取贴图（进程级缓存；缺资源 = `None`）。
#[must_use]
pub fn get(asset: &str) -> Option<Arc<HdPng>> {
    let mut guard = cache().lock().ok()?;
    if let Some(hit) = guard.get(asset) {
        return hit.clone();
    }
    let loaded = load_from_disk(asset);
    guard.insert(asset.to_owned(), loaded.clone());
    loaded
}

/// 读盘 + 解码（失败 = `None`）。
fn load_from_disk(asset: &str) -> Option<Arc<HdPng>> {
    let root = assets_root()?;
    let bytes = std::fs::read(root.join(asset)).ok()?;
    let img = ti_platform::hd_png::decode_png_rgba(&bytes).ok()?;
    Some(Arc::new(img))
}
