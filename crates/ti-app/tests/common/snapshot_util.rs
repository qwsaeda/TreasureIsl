//! S15.5：渲染快照 / 视觉回归基线门禁共用机制（1080×1920 代码绘制帧 → DEFLATE PNG）。
//!
//! - 帧来源：`GameApp::render_to_framebuf()`（RGB565，1080×1920，确定性，契约 3）。
//! - 基线：`<dir>/<name>.png`（RGBA 8-bit，`png` crate DEFLATE 压缩，紧凑；legacy 176×220 旧基线自动重生成）。
//! - 判据：基线缺失 **或** 基线尺寸 ≠ 1080×1920 → 重生成落盘（首跑 / S15.5 重锚）；
//!   基线尺寸匹配 → RGB 逐像素 diff（±1 量化容差，EXP-074），diff=0 才 PASS。
//! - 自包含 3 个快照门禁共用（`render_snapshot` / `render_snapshot_forward` / `visual_regression`），
//!   经 `#[path = "common/snapshot_util.rs"]` 引入（放 `tests/common/` 避免被 cargo 自动识别为独立测试目标）。
//!
//! 编解码**对称无损**（EXP-093）：存 `ColorType::Rgba`（alpha=255），解码 `STRIP_16|EXPAND` 归一，
//! 按 `OutputInfo` 真实通道数取每像素前 3 通道；上线前以小 fixture 往返探针先证 0 mismatch。

#![allow(
    clippy::panic,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::must_use_candidate,
    clippy::missing_panics_doc,
    clippy::useless_conversion,
    clippy::match_same_arms,
    clippy::doc_markdown,
    clippy::many_single_char_names,
    clippy::items_after_statements
)]

use std::fs;
use std::path::Path;

pub const FB_W: usize = 1080;
pub const FB_H: usize = 1920;
pub const FB_PIXELS: usize = FB_W * FB_H;

/// RGB565 → RGB8（5/6/5 位扩 8 位，最近邻 `<<n | >>m`，与 `render_png.rs` 同口径，确定性）。
pub fn rgb565_to_rgb8(fb: &[u16]) -> Vec<u8> {
    assert_eq!(fb.len(), FB_PIXELS, "帧缓冲尺寸应 = 1080×1920 = {FB_PIXELS}");
    let mut out = vec![0u8; FB_PIXELS * 3];
    for (i, &p) in fb.iter().enumerate() {
        let r5 = (p >> 11) & 0x1F;
        let g6 = (p >> 5) & 0x3F;
        let b5 = p & 0x1F;
        let r = ((r5 << 3) | (r5 >> 2)) as u8;
        let g = ((g6 << 2) | (g6 >> 4)) as u8;
        let b = ((b5 << 3) | (b5 >> 2)) as u8;
        let o = i * 3;
        out[o] = r;
        out[o + 1] = g;
        out[o + 2] = b;
    }
    out
}

/// RGB8 → PNG（`png` crate DEFLATE 压缩，紧凑；legacy 自研 stored 块在 1080 下会 ~6MB，弃用）。
///
/// 存储为 RGBA（alpha=255）与 `decode_png`（EXPAND→RGBA）对称，避免 RGB/RGBA 通道数错位。
pub fn encode_png_rgb8(rgb: &[u8], w: u32, h: u32) -> Vec<u8> {
    let px = (rgb.len() / 3) as u32;
    let mut rgba = vec![0u8; px as usize * 4];
    for (i, trip) in rgb.chunks_exact(3).enumerate() {
        rgba[i * 4] = trip[0];
        rgba[i * 4 + 1] = trip[1];
        rgba[i * 4 + 2] = trip[2];
        rgba[i * 4 + 3] = 0xFF;
    }
    let mut buf: Vec<u8> = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut buf, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap_or_else(|e| panic!("png 编码头失败: {e}"));
        writer.write_image_data(&rgba).unwrap_or_else(|e| panic!("png 编码像素失败: {e}"));
    } // writer 析构，`&mut buf` 借用释放
    buf
}

/// PNG → (宽, 高, RGB 三元组字节)。解码失败 / 非 PNG → None。
///
/// 与 `encode_png_rgb8`（RGBA 8-bit）对称：`STRIP_16 | EXPAND` 归一到 8-bit，
/// 按 `OutputInfo` 真实尺寸/通道数归一到每像素前 3 通道（RGB）。
pub fn decode_png(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = fs::read(path).ok()?;
    decode_png_bytes(&bytes)
}

fn decode_png_bytes(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    use std::io::Cursor;
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::STRIP_16 | png::Transformations::EXPAND);
    let mut reader = decoder.read_info().ok()?;
    // `STRIP_16|EXPAND` 归一到 8-bit；RGBA 为每像素最大 4 字节，按此上限分配。
    let (w0, h0) = (reader.info().width, reader.info().height);
    let cap = usize::try_from(w0).ok()?.saturating_mul(usize::try_from(h0).ok()?).saturating_mul(4);
    if cap == 0 {
        return None;
    }
    let mut raw = vec![0u8; cap];
    let out = reader.next_frame(&mut raw).ok()?;
    let px = usize::try_from(out.width).ok()?.saturating_mul(usize::try_from(out.height).ok()?);
    if px == 0 {
        return None;
    }
    let channels = color_channels(out.color_type);
    let mut rgb = vec![0u8; px * 3];
    for i in 0..px {
        let off = i * channels;
        rgb[i * 3] = raw[off];
        rgb[i * 3 + 1] = raw[off + 1];
        rgb[i * 3 + 2] = raw[off + 2];
    }
    Some((out.width, out.height, rgb))
}

/// `ColorType` → 每像素样本数（无 `.channels()` 方法，手动映射）。
fn color_channels(ct: png::ColorType) -> usize {
    match ct {
        png::ColorType::Grayscale => 1,
        png::ColorType::Rgb => 3,
        png::ColorType::Indexed => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgba => 4,
    }
}

/// 基线 diff=0 门禁（`08_render_arch.md` §6 契约 5）：
/// - 基线缺失或尺寸 ≠ 1080×1920 → 重生成落盘（S15.5 重锚 / 首跑），返回 `Regenerated`。
/// - 尺寸匹配 → 逐像素 diff（RGB ±1 容差），diff 像素 > 0 返回 `Drift{diff}`（调用方据此断言 FAIL）。
pub enum BaselineResult {
    /// 基线重生成（缺失或 legacy 尺寸），无 diff 判据。
    Regenerated,
    /// 基线匹配尺寸且逐像素一致（±1 容差内 diff=0）。
    Match,
    /// 基线匹配尺寸但存在漂移（diff 像素数 + 最大通道偏差）。
    Drift { diff: usize, max_delta: i32 },
}

/// S15.6 QA125：TTF 在位前置断言——快照入口渲染帧前 `shell.ttf_font` 必须 `Some`
/// （基线须为 44px 真字形态）。TTF 缺失（cwd 回溯不到 `assets/fonts/` 或字体文件缺失）
/// → panic 带诊断，**禁止**静默 Regenerated 豆腐基线入库（QA016 类空窗：基线缺失时
/// Regenerated 接受 + 无 diff 判据，缺字体环境会把 12px 豆腐帧当基线写盘）。
pub fn require_ttf_present(app: &ti_app::game_app::GameApp, context: &str) {
    assert!(
        app.shell().ttf_font.is_some(),
        "{context} 快照入口 TTF 缺失（assets/fonts/lxgw-wenkai-reg.ttf）：menu_shell 自动加载 \
         依赖 cwd 向上 8 级回溯找 assets/fonts/，检查测试 cwd / 字体文件在位；禁止豆腐基线入库（S15.6 QA125）"
    );
}

/// 单场景基线门禁入口：`rgb565` = `render_to_framebuf()` 的 1080×1920 帧。
pub fn baseline_gate(name: &str, baseline_dir: &Path, rgb565: &[u16]) -> BaselineResult {
    let rgb = rgb565_to_rgb8(rgb565);
    let path = baseline_dir.join(format!("{name}.png"));
    if let Some((bw, bh, base)) = decode_png(&path) {
        if bw == FB_W as u32 && bh == FB_H as u32 {
            // 同尺寸基线：逐像素 diff（RGB ±1 量化容差，EXP-074 口径）。
            let mut diff = 0usize;
            let mut max_delta = 0i32;
            for i in 0..FB_PIXELS {
                for c in 0..3 {
                    let cur = i32::from(rgb[i * 3 + c]);
                    let base_v = i32::from(base[i * 3 + c]);
                    let d = (cur - base_v).abs();
                    max_delta = max_delta.max(d);
                    if d > 1 {
                        diff += 1;
                        break;
                    }
                }
            }
            return if diff == 0 {
                BaselineResult::Match
            } else {
                BaselineResult::Drift { diff, max_delta }
            };
        }
        // 尺寸 ≠ 1080×1920（legacy 176×220 旧基线）→ 重生成覆盖。
    }
    // 缺失或 legacy 尺寸 → 重生成落盘。
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("基线目录创建失败");
    }
    let png_bytes = encode_png_rgb8(&rgb, FB_W as u32, FB_H as u32);
    fs::write(&path, &png_bytes).expect("基线 PNG 落盘失败");
    eprintln!("基线重生成：{name}.png（{} bytes，1080×1920 代码绘制）", png_bytes.len());
    BaselineResult::Regenerated
}
