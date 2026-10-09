//! wasm 壳的 UI 文案：主菜单表 + 状态行文本（壳与验收测试共用同一份数据）。
//!
//! 抽到 lib 层的原因（S2.5 补实）：`tests/glyph_raster.rs` 要对**壳真正会画的
//! 字符串**断言字形覆盖（哪些字符命中真字形、哪些必须走豆腐块回退），
//! 若测试与 bin 各写一份字面量，两边会静默漂移。

/// 主菜单条目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItem {
    /// 中文标签（逐字过字库，缺字时画豆腐块）。
    pub label: &'static str,
    /// 菜单项事件码（`docs/spec/menu.md §a`，`sub_41354` 的 `v3[14]` case 0..=5；
    /// 0=新游戏 / 1=继续 / 2=选项 / 3=说明1 / 4=说明2 / 5=退出）。
    /// 真实确定键事件码 = 13（`spec/menu.md §a`）；本字段是「选中菜单项索引」，
    /// S5.1 起 `GameApp::handle_index` 用 `menu_selected` 直接匹配 case，不再靠此占位。
    pub event_id: u32,
}

/// 主菜单条目表（中文文案取自原版 UI 资源，`docs/gamebook.md` §3.1）。
///
/// 菜单项顺序对齐 `sub_41354` case 0..=5：新游戏→角色选择；继续→存读档；
/// 选项→Help；说明1/说明2→Help；退出→退出游戏。
pub const MENU: &[MenuItem] = &[
    MenuItem { label: "新游戏", event_id: 0 },
    MenuItem { label: "继续游戏", event_id: 1 },
    MenuItem { label: "读取存档", event_id: 2 },
    MenuItem { label: "选项设置", event_id: 3 },
    MenuItem { label: "帮助说明", event_id: 4 },
    MenuItem { label: "离开游戏", event_id: 5 },
];

/// 底部状态行的三段文本（从左到右：字形数 → 音效条数 → 未接入 MOD 数）。
#[must_use]
pub fn status_texts(font_glyphs: usize, wav_count: usize, mod_count: usize) -> [String; 3] {
    [format!("字形{font_glyphs}"), format!("音效{wav_count}条"), format!("MOD{mod_count}")]
}
