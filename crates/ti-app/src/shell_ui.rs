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
    /// ⚠️ **占位值**：1..6 未实测，真实事件 id 由 S3.3 从 `ref/event_ids.json`
    /// 解出（`docs/plan-s3.md`），在此之前不得当作原版事实使用。
    pub event_id: u32,
}

/// 主菜单条目表（中文文案取自原版 UI 资源）。
pub const MENU: &[MenuItem] = &[
    MenuItem { label: "新游戏", event_id: 1 },
    MenuItem { label: "继续游戏", event_id: 2 },
    MenuItem { label: "读取存档", event_id: 3 },
    MenuItem { label: "选项设置", event_id: 4 },
    MenuItem { label: "帮助说明", event_id: 5 },
    MenuItem { label: "离开游戏", event_id: 6 },
];

/// 底部状态行的三段文本（从左到右：字形数 → 音效条数 → 未接入 MOD 数）。
#[must_use]
pub fn status_texts(font_glyphs: usize, wav_count: usize, mod_count: usize) -> [String; 3] {
    [format!("字形{font_glyphs}"), format!("音效{wav_count}条"), format!("MOD{mod_count}")]
}
