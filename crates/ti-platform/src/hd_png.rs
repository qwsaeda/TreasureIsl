//! S7.5（feature `hd-8bit`）：HD 资产 PNG 解码 → RGBA8 图像。
//!
//! 把 `assets_hd_pack/` 的真彩 PNG（1x/2x/4x 档）解码为 RGBA8 行主序字节，
//! 供 `SpriteAtlas`/`TerrainAtlas` 的 RGBA8 blit 路径直写 `FramebufRgba`
//! （`docs/impl/14_asset_enhancement.md` §3.5 路径 1 并行）。
//!
//! 解码走 `png` crate（§5.9 定案清单，纯 Rust）；失败返回 `Err`，
//! 调用方跳过该资产（不阻塞渲染，退回 RGB565 原路径或占位）。

use std::io::Cursor;

/// 已解码的 HD PNG 图像（RGBA8，行主序，`len = width × height × 4`）。
#[derive(Debug, Clone)]
pub struct HdPng {
    /// 宽（像素）。
    pub width: u32,
    /// 高（像素）。
    pub height: u32,
    /// RGBA8 字节（`[r, g, b, a]` × w×h，行主序）。
    pub rgba: Vec<u8>,
}

/// 解码 PNG 字节为 RGBA8 图像。
///
/// 支持任意色彩类型（`png` crate 归一化为 RGBA8）；非 PNG / 截断 / 解码失败 → `Err`。
///
/// # Errors
///
/// PNG 头无效、像素解码失败或 IO 错误。
pub fn decode_png_rgba(bytes: &[u8]) -> Result<HdPng, String> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    // 归一化输出为 RGBA8（调色板/灰度/指数 16 位统一扩展）
    decoder.set_transformations(png::Transformations::STRIP_16 | png::Transformations::EXPAND);
    let mut reader = decoder.read_info().map_err(|e| format!("png 头解析失败: {e}"))?;
    let info = reader.info();
    let width = info.width;
    let height = info.height;
    let pixels = usize::try_from(width)
        .map_or(0, |w| w.saturating_mul(usize::try_from(height).unwrap_or(0)));
    if pixels == 0 {
        return Err("png 宽高为 0".to_owned());
    }
    let mut rgba = vec![0u8; pixels * 4];
    reader.next_frame(&mut rgba).map_err(|e| format!("png 像素解码失败: {e}"))?;
    Ok(HdPng { width, height, rgba })
}
