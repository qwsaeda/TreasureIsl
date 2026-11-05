//! S39：数据化 UI 层字号档（`tier_font_px`）纯函数契约。
//! `build_data_chrome` 的 `text` 原语按 `tier`（title/hint/空=body）定字号档；
//! 未知档回退 body（防御）。

use ti_engine::ui::{palette, tier_font_px};

#[test]
fn tier_font_px_maps_tiers() {
    assert_eq!(tier_font_px("title"), palette::DATA_TEXT_FONT_TITLE);
    assert_eq!(tier_font_px("hint"), palette::DATA_TEXT_FONT_HINT);
    assert_eq!(tier_font_px("body"), palette::DATA_TEXT_FONT_BODY);
    // 空档 + 未知档 = body（防御回退，不 panic）
    assert_eq!(tier_font_px(""), palette::DATA_TEXT_FONT_BODY);
    assert_eq!(tier_font_px("unknown"), palette::DATA_TEXT_FONT_BODY);
}

#[test]
fn tier_font_px_title_larger_than_body() {
    // title > body > hint（视觉层级单调，防字号档倒置）
    let (title, body, hint) =
        (palette::DATA_TEXT_FONT_TITLE, palette::DATA_TEXT_FONT_BODY, palette::DATA_TEXT_FONT_HINT);
    assert!(title > body);
    assert!(body > hint);
}
