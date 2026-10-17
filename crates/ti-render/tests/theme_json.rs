//! S15.7 数据分离：`Theme` JSON 解析（`assets/data/theme.json` = 色板 + 字号唯一事实源）。
//!
//! 判据：
//! 1. 仓内 `theme.json` 解析结果 == `Theme::default()`（数据文件与 Rust 兜底值同值，
//!    证明迁移无损；此后改色只改 JSON）。
//! 2. 缺字段回落 `default`，**不静默改值**（只有显式出现的键生效）。
//! 3. 色值接受 `"0x10A3"` / `"10A3"` / 裸数字三种写法。
//! 4. 语法错误 / 结构不符 → 整体回落 `default`（不 panic）。

use ti_render::theme::{Hex16, Theme, ThemeJson};

/// 仓内主题数据文件（经 `include_str!` 编译期内联，测试不依赖 cwd）。
const THEME_JSON: &str = include_str!("../../../assets/data/theme.json");

#[test]
fn repo_theme_json_matches_default() {
    let from_json = Theme::from_json_str(THEME_JSON);
    assert_eq!(
        from_json,
        Theme::default(),
        "assets/data/theme.json 与 Theme::default() 不同值：数据文件应与 Rust 兜底同值（S15.7 迁移无损）"
    );
}

#[test]
fn repo_theme_json_declares_every_key() {
    let root: ThemeJson = serde_json::from_str(THEME_JSON).expect("theme.json 语法错");
    let p = root.palette.expect("theme.json 缺 palette 组");
    let f = root.fonts.expect("theme.json 缺 fonts 组");
    // 21 色
    for (name, present) in [
        ("parch_dark", p.parch_dark.is_some()),
        ("parch_light", p.parch_light.is_some()),
        ("wood", p.wood.is_some()),
        ("gold", p.gold.is_some()),
        ("gold_glow", p.gold_glow.is_some()),
        ("gold_bright", p.gold_bright.is_some()),
        ("menu_bg", p.menu_bg.is_some()),
        ("menu_border", p.menu_border.is_some()),
        ("hilite", p.hilite.is_some()),
        ("bevel_hi", p.bevel_hi.is_some()),
        ("bevel_shadow", p.bevel_shadow.is_some()),
        ("text_on_parch", p.text_on_parch.is_some()),
        ("text_on_wood", p.text_on_wood.is_some()),
        ("danger", p.danger.is_some()),
        ("player", p.player.is_some()),
        ("landmark", p.landmark.is_some()),
        ("bg", p.bg.is_some()),
        ("title_bar", p.title_bar.is_some()),
        ("selected", p.selected.is_some()),
        ("secondary", p.secondary.is_some()),
        ("black", p.black.is_some()),
    ] {
        assert!(present, "theme.json palette 缺 `{name}`（键缺失 = 静默回落 default = 隐藏漂移）");
    }
    // 6 字号
    for (name, present) in [
        ("font_size", f.font_size.is_some()),
        ("title_font_size", f.title_font_size.is_some()),
        ("hint_font_size", f.hint_font_size.is_some()),
        ("bitmap_advance", f.bitmap_advance.is_some()),
        ("bitmap_advance_title", f.bitmap_advance_title.is_some()),
        ("bitmap_advance_hint", f.bitmap_advance_hint.is_some()),
    ] {
        assert!(present, "theme.json fonts 缺 `{name}`");
    }
}

#[test]
fn partial_json_only_overrides_present_keys() {
    let t = Theme::from_json_str(r#"{"palette":{"danger":"0x1234"},"fonts":{"font_size":50}}"#);
    let d = Theme::default();
    assert_eq!(t.danger, 0x1234, "显式键须生效");
    assert_eq!(t.font_size, 50, "显式字号须生效");
    assert_eq!(t.player, d.player, "未出现的色键须回落 default");
    assert_eq!(t.title_font_size, d.title_font_size, "未出现的字号键须回落 default");
}

#[test]
fn hex_accepts_prefixed_unprefixed_and_numeric() {
    assert_eq!(Hex16::Str("0x10A3".into()).to_u16(), Some(0x10A3));
    assert_eq!(Hex16::Str("10A3".into()).to_u16(), Some(0x10A3));
    assert_eq!(Hex16::Str("FFFF".into()).to_u16(), Some(0xFFFF));
    assert_eq!(Hex16::Num(0x10A3).to_u16(), Some(0x10A3));
    // 越界 / 非法 → None（回落 default，不 panic）
    assert_eq!(Hex16::Str("ZZZZ".into()).to_u16(), None);
    assert_eq!(Hex16::Num(-1).to_u16(), None);
    assert_eq!(Hex16::Num(70000).to_u16(), None);
}

#[test]
fn malformed_json_falls_back_to_default() {
    assert_eq!(Theme::from_json_str("{ not json"), Theme::default());
    assert_eq!(Theme::from_json_str("[]"), Theme::default());
    assert_eq!(Theme::from_json_str(""), Theme::default());
}
