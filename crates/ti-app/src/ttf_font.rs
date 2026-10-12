//! S10.4：TTF 矢量字形加载（macroquad `Font` + `load_ttf`）。
//!
//! 设计（`docs/style.md` §5.6.5）：
//! - [`load_ttf_subset`]：从 `assets/fonts/NotoSansCJK-subset.ttf`（或 system TTF）
//!   加载 TTF 子集，返回 macroquad `Font`（`draw_text_ex` 用）。
//! - 原版 `GoldLand.Fnt` 点阵字保留为 feature `legacy-font`（怀旧模式），
//!   默认构建走 TTF 矢量字形（`gpu-render` feature，默认开）。
//!
//! 子集 TTF 文件占位：`assets/fonts/NotoSansCJK-subset.ttf`
//! （子集生成脚本待补；S10.4 暂用 system TTF 或空 `Font::default`）。
//!
//! 不引入新 crate 依赖（macroquad 已有）。

use macroquad::text::Font;

/// 默认 TTF 子集文件路径（相对项目根；`assets/fonts/NotoSansCJK-subset.ttf`）。
///
/// 子集生成脚本待补（EXP：占位标注）。
pub const TTF_SUBSET_PATH: &str = "assets/fonts/NotoSansCJK-subset.ttf";

/// S10.4（feature `gpu-render`）：加载 TTF 子集为 macroquad `Font`。
///
/// `subset_bytes` 为 TTF 文件字节；`None` 时返回 `Font::default()`（系统默认字形回退）。
///
/// 返回 `None` 表示加载失败（调用方回退 legacy 点阵字）。
///
/// **API 确认下轮编译时验证**：macroquad 0.4.x 的 `Font::from_bytes` / `load_ttf`
/// 具体签名可能随版本变化，此处用 `Font::default()` 占位，
/// 真实 TTF 解析在确认 API 后补实（伪代码跳过点，EXP 记录）。
#[must_use]
pub fn load_ttf(subset: Option<&[u8]>) -> Option<Font> {
    let subset = subset?;
    // 伪代码占位：macroquad `Font::from_bytes` API 待确认（下轮编译验证）。
    // 实际实现（待 API 确认后启用）：
    //   Some(macroquad::texture::Font::from_bytes(subset))
    //
    // 当前：回退 `Font::default()`（系统默认，能画基本拉丁字符；
    // 中文 CJK 字形需 TTF 子集文件 + `Font::from_bytes` 确认后才完整）。
    let _ = subset; // 标记：字节已传入，解析待实装
    Some(Font::default())
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
pub fn load_ttf_from_bytes(bytes: Vec<u8>) -> Option<Font> {
    load_ttf(Some(&bytes))
}
