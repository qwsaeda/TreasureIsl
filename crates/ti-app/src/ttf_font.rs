//! S10.4：TTF 矢量字形加载（macroquad `Font` + `load_ttf`）。
//!
//! 设计（`docs/style.md` §5.6.5）：
//! - [`load_ttf`]：从 TTF 字节加载 macroquad `Font`（`draw_text_ex` 用）。
//! - 原版 `GoldLand.Fnt` 点阵字保留为 feature `legacy-font`（怀旧模式），
//!   默认构建走 TTF 矢量字形（`gpu-render` feature，默认开）。
//!
//! CJK 字形依赖 TTF 子集文件；缺失时 `load_ttf` 返回 `None`（调用方回退 legacy 点阵字），
//! 不静默假文本。
//!
//! 不引入新 crate 依赖（macroquad 已有）。

use macroquad::text::Font;

/// 默认 TTF 子集文件路径（相对项目根；`assets/fonts/NotoSansCJK-subset.ttf`）。
pub const TTF_SUBSET_PATH: &str = "assets/fonts/NotoSansCJK-subset.ttf";

/// S10.4（feature `gpu-render`）：加载 TTF 子集为 macroquad `Font`。
///
/// `subset_bytes` 为 TTF 文件字节；`None` 或字节非法时返回 `None`
/// （调用方回退 legacy 点阵字，不 panic）。
#[must_use]
pub fn load_ttf(subset: Option<&[u8]>) -> Option<Font> {
    let bytes = subset?;
    if bytes.is_empty() {
        return None;
    }
    macroquad::text::load_ttf_font_from_bytes(bytes).ok()
}

/// 从文件路径加载 TTF 子集（native 宿主；wasm 走 `macroquad::file::load_file`）。
///
/// 路径不存在或读取失败返回 `None`（调用方回退 legacy 点阵字，不 panic）。
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn load_ttf_from_path(path: &std::path::Path) -> Option<Font> {
    let bytes = std::fs::read(path).ok()?;
    load_ttf(Some(&bytes))
}

/// wasm 路径：从预打包 TTF 字节加载（`init` 时注入）。
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn load_ttf_from_bytes(bytes: &[u8]) -> Option<Font> {
    load_ttf(Some(bytes))
}

/// 同步加载 TTF（同步路径；native 端从文件读后调用）。
#[must_use]
pub fn load_ttf_sync(bytes: &[u8]) -> Option<Font> {
    load_ttf(Some(bytes))
}
