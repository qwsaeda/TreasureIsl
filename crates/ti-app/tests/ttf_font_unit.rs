//! `ttf_font::load_ttf` 单测（S10.4，EXP-052：内联测试移至 tests/ 目录）。

use ti_app::ttf_font::{TTF_SUBSET_PATH, load_ttf, load_ttf_from_path};

#[test]
fn ttf_subset_path_is_assets_fonts() {
    assert_eq!(TTF_SUBSET_PATH, "assets/fonts/NotoSansCJK-subset.ttf");
}

#[test]
fn load_ttf_none_returns_none() {
    assert!(load_ttf(None).is_none());
}

/// `Font::default()` 需 miniquad 主线程，仅在 wasm 平台验证；
/// native 平台 `load_ttf` 逻辑（`subset?` + 字节透传）由 `load_ttf_none_returns_none` 覆盖。
#[cfg(target_arch = "wasm32")]
#[test]
fn load_ttf_some_returns_default_font() {
    // 当前占位实现（Q006，TTF 子集待补）：有字节时返回 Some(Font::default())
    let result = load_ttf(Some(b"fake-ttf-bytes"));
    assert!(result.is_some(), "有字节时应返回 Some(Font::default())");
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn load_ttf_from_path_missing_returns_none() {
    let missing = std::path::Path::new("nonexistent/font.ttf");
    assert!(load_ttf_from_path(missing).is_none());
}
