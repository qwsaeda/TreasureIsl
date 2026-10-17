//! `TtfFont` 软件光栅化单测（UI-21，EXP-084；S15.6 QB115 三档 96/44/28 API）。
//!
//! 测试策略：
//! - 无效字体数据返回 None（不依赖外部文件）
//! - 系统字体存在时验证光栅化正确性（不存在则跳过，不失败）
//! - 字形尺寸可变性、advance 推进、垂直对齐
//! - QA130：固定 TTF 字节 + 固定字符集 → ink 矩阵全等钉住（FNV1a 摘要锁，
//!   换 TTF 文件 / 换光栅器版本 → 摘要变 → 当场 FAIL；`ab_glyph` 纯 CPU 光栅，
//!   同字节跨端确定性 = QA127 跨端帧等价判据的单端锁，S16.1 浏览器试玩时跨端核）

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss)]

use ti_app::glyph_raster::{Raster, TtfFont};

/// 系统字体路径（Windows）；不存在时测试跳过。
const SYSTEM_FONT: &str = "C:/Windows/Fonts/simhei.ttf";

/// QA130 钉住字符集（正文 44px 档）：CJK 常用 + 缺字哨兵。
const PIN_CHARSET: &[char] = &['金', '银', '岛', '战', '斗', '攻', '击'];
/// QA130 钉住摘要（FNV1a-64 over 各字符 ink 矩阵序列：cols/rows 头 + 逐位 bit）。
/// 换 TTF 字节 / 换光栅器 → 须重钉（首跑断言消息打印实际值）。
const PIN_FNV64: u64 = 0x12BD_01F9_F1F9_9A1A;

fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| std::path::PathBuf::from("."), std::path::Path::to_path_buf)
}

/// 仓库内置 TTF（快照/实机同源，`menu_shell` 内置加载同一文件；QA127 单端钉住源）。
fn builtin_ttf_bytes() -> Option<Vec<u8>> {
    let bytes = std::fs::read(repo_root().join("assets/fonts/lxgw-wenkai-reg.ttf")).ok()?;
    Some(bytes)
}

#[test]
fn ttf_from_invalid_bytes_returns_none() {
    assert!(TtfFont::from_bytes(b"not a font", 44.0).is_none());
    assert!(TtfFont::from_bytes(&[], 44.0).is_none());
}

#[test]
fn ttf_rasterize_basic_properties() {
    let Ok(bytes) = std::fs::read(SYSTEM_FONT) else {
        eprintln!("跳过：系统字体 {SYSTEM_FONT} 不存在");
        return;
    };
    let Some(ttf) = TtfFont::from_bytes(&bytes, 12.0) else {
        eprintln!("跳过：simhei.ttf 加载失败");
        return;
    };

    // 中文字符「金」应光栅化出非空白位图（本实例绑定 12px 档）
    let raster = ttf.rasterize('金');
    assert!(raster.ink_count() > 0, "「金」应有墨点");
    assert!(raster.cols() > 0 && raster.rows() > 0, "尺寸应 > 0");

    // 空格应返回空白
    let space = ttf.rasterize(' ');
    assert_eq!(space.ink_count(), 0, "空格应无墨点");

    // 缺失字符（如私有区）应返回豆腐块（有墨点但非正常字形）
    let missing = ttf.rasterize('\u{E000}');
    assert!(missing.ink_count() > 0, "缺失字符应有豆腐块");
}

#[test]
fn ttf_advance_and_line_height() {
    let Ok(bytes) = std::fs::read(SYSTEM_FONT) else {
        eprintln!("跳过：系统字体 {SYSTEM_FONT} 不存在");
        return;
    };
    let Some(ttf) = TtfFont::from_bytes(&bytes, 12.0) else {
        eprintln!("跳过：simhei.ttf 加载失败");
        return;
    };

    // 中文字符 advance 应 > 0
    let adv = ttf.advance('金');
    assert!(adv > 0.0, "advance 应 > 0，实际 {adv}");

    // 行高应 > 0
    let lh = ttf.line_height();
    assert!(lh > 0.0, "line_height 应 > 0，实际 {lh}");

    // 不同字号档（QB115：实例绑定单一字号）应产生不同尺寸
    let Some(small) = TtfFont::from_bytes(&bytes, 8.0) else {
        eprintln!("跳过：simhei.ttf 8px 档加载失败");
        return;
    };
    let Some(large) = TtfFont::from_bytes(&bytes, 24.0) else {
        eprintln!("跳过：simhei.ttf 24px 档加载失败");
        return;
    };
    let s = small.rasterize('A');
    let l = large.rasterize('A');
    assert!(l.rows() > s.rows(), "大字号档应产生更高位图：{}/{}", s.rows(), l.rows());
}

#[test]
fn ttf_deterministic_rendering() {
    let Ok(bytes) = std::fs::read(SYSTEM_FONT) else {
        eprintln!("跳过：系统字体 {SYSTEM_FONT} 不存在");
        return;
    };
    let Some(ttf) = TtfFont::from_bytes(&bytes, 12.0) else {
        eprintln!("跳过：simhei.ttf 加载失败");
        return;
    };

    // 同字体同字符同字号两次光栅化应完全一致（确定性；QA128 命中缓存 = 克隆）
    let a = ttf.rasterize('银');
    let b = ttf.rasterize('银');
    assert_eq!(a, b, "同参数光栅化应确定性");
}

/// QB115：三档原生光栅（96 标题 / 44 正文 / 28 提示）同字节实例化，档间尺寸单调。
#[test]
fn ttf_three_tier_sizes() {
    let Some(bytes) = builtin_ttf_bytes() else {
        eprintln!("跳过：仓库内置 TTF 缺失（assets/fonts/lxgw-wenkai-reg.ttf）");
        return;
    };
    let Some(title) = TtfFont::from_bytes(&bytes, 96.0) else {
        eprintln!("跳过：TTF 96px 档解析失败");
        return;
    };
    let Some(body) = TtfFont::from_bytes(&bytes, 44.0) else {
        eprintln!("跳过：TTF 44px 档解析失败");
        return;
    };
    let Some(hint) = TtfFont::from_bytes(&bytes, 28.0) else {
        eprintln!("跳过：TTF 28px 档解析失败");
        return;
    };
    assert_eq!(title.size_px() as i32, 96);
    assert_eq!(body.size_px() as i32, 44);
    assert_eq!(hint.size_px() as i32, 28);
    let t = title.rasterize('金');
    let b = body.rasterize('金');
    let h = hint.rasterize('金');
    assert!(
        t.rows() > b.rows() && b.rows() > h.rows(),
        "三档字号行高应单调 96>44>28：{}/{}/{}",
        t.rows(),
        b.rows(),
        h.rows()
    );
}

/// FNV1a-64 摘要（ink 矩阵序列化：每字符 cols/rows 头 + 逐行逐列 bit）。
fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcb79_118e_edda_1cc8;
    for &b in data {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// 固定 TTF 字节 + 固定字符集的 ink 矩阵序列化摘要（QA130 钉住判据源）。
#[must_use]
fn pinned_ink_digest(ttf: &TtfFont) -> u64 {
    let mut buf: Vec<u8> = Vec::new();
    for &ch in PIN_CHARSET {
        let raster: Raster = ttf.rasterize(ch);
        let cols = u32::try_from(raster.cols()).unwrap_or(u32::MAX);
        let rows = u32::try_from(raster.rows()).unwrap_or(u32::MAX);
        buf.extend_from_slice(&cols.to_le_bytes());
        buf.extend_from_slice(&rows.to_le_bytes());
        for &bit in raster.as_slice() {
            buf.push(u8::from(bit));
        }
    }
    fnv1a64(&buf)
}

/// QA130：ink 矩阵全等钉住——固定 TTF 字节（仓库内置 lxgw-wenkai-reg）+ 固定字符集，
/// 44px 档光栅摘要 == 钉住常数（换 TTF 文件 / 光栅器版本漂移 → 当场 FAIL 而非静默漂移）。
/// QA127 跨端帧等价判据（单端锁）：`ab_glyph` 纯 CPU 确定性光栅，同字节同字符集
/// 跨端 ink 全等 → native/wasm 同场景帧字节等价（S16.1 浏览器试玩时跨端核）。
#[test]
fn ttf_ink_matrix_pinned() {
    let Some(bytes) = builtin_ttf_bytes() else {
        eprintln!("跳过：仓库内置 TTF 缺失（QA130 钉住判据需 assets/fonts 在位）");
        return;
    };
    let Some(ttf) = TtfFont::from_bytes(&bytes, 44.0) else {
        eprintln!("跳过：TTF 44px 档解析失败");
        return;
    };
    let digest = pinned_ink_digest(&ttf);
    assert_eq!(
        digest, PIN_FNV64,
        "QA130 ink 矩阵钉住摘要漂移（实际 {digest:#x}）：TTF 字节或光栅器已变，须重钉"
    );
}
