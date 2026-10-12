//! wasm 壳的 UI 文案：主菜单表 + 状态行文本（壳与验收测试共用同一份数据）。
//!
//! 抽到 lib 层的原因（S2.5 补实）：`tests/glyph_raster.rs` 要对**壳真正会画的
//! 字符串**断言字形覆盖（哪些字符命中真字形、哪些必须走豆腐块回退），
//! 若测试与 bin 各写一份字面量，两边会静默漂移。

use crate::i18n::sget;

/// 主菜单条目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItem {
    /// i18n key（运行时经 `sget` 解析成中文标签；启动缺失 key 时显示 key 本身）。
    pub key: &'static str,
    /// 菜单项事件码（`docs/spec/menu.md §a`，`saveload` 的 `v3[14]` case 0..=5；
    /// 0=新游戏 / 1=继续 / 2=选项 / 3=说明1 / 4=说明2 / 5=退出）。
    /// 真实确定键事件码 = 13（`spec/menu.md §a`）；本字段是「选中菜单项索引」，
    /// S5.1 起 `GameApp::handle_index` 用 `menu_selected` 直接匹配 case，不再靠此占位。
    pub event_id: u32,
}

/// 主菜单条目表（顺序对齐 `saveload` case 0..=5：新游戏→角色选择；
/// 继续→存读档；选项→Help；说明1/说明2→Help；退出→退出游戏）。
///
/// 条目保持纯 const（仅 i18n key + 事件码）；中文标签在运行时经
/// [`menu_labels`] 解析（`sget` 非 const，`const` 上下文不能调用，见 `EXP-052`）。
pub const MENU: &[MenuItem] = &[
    MenuItem { key: "menu.new_game", event_id: 0 },
    MenuItem { key: "menu.continue", event_id: 1 },
    MenuItem { key: "menu.load", event_id: 2 },
    MenuItem { key: "menu.options", event_id: 3 },
    MenuItem { key: "menu.help", event_id: 4 },
    MenuItem { key: "menu.quit", event_id: 5 },
];

/// 主菜单中文标签表（运行时经 i18n 解析；缺失 key 显示 key 本身）。
///
/// 与 [`MENU`] 下标一一对应。
#[must_use]
pub fn menu_labels() -> Vec<&'static str> {
    MENU.iter().map(|i| sget(i.key)).collect()
}

/// 底部状态行的三段文本（从左到右：字形数 → 音效条数 → 未接入 MOD 数）。
#[must_use]
pub fn status_texts(font_glyphs: usize, wav_count: usize, mod_count: usize) -> [String; 3] {
    [
        crate::i18n::sgetf("diag.fonts", &[&font_glyphs]),
        crate::i18n::sgetf("diag.wav", &[&wav_count]),
        crate::i18n::sgetf("diag.mod_fmt", &[&mod_count]),
    ]
}
