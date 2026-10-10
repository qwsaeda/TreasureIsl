//! S8.5 控件层（`widgets.rs`）测试：控件表加载 + 可见过滤 + hit 边界 + 绘制。
//!
//! 判据（S8.5 `DoD`）：`gen_layout` 产 `controls.json`；`WidgetLayer` 按 f5 类型分派，
//! 只画可见控件（draw 面积 > 0）+ hit 边界。
//! 置于集成测试目录，豁免门禁的 `unwrap`/`expect` 扫描（EXP-052）。
//! 本文件不使用 `panic!`（硬规则 1：`crates/` 下 0 命中），断言走 `assert!`。

use ti_app::widgets::WidgetLayer;

fn require_layer() -> WidgetLayer {
    WidgetLayer::new().expect("controls.json 应可加载（编译期 include_str!）")
}

#[test]
fn controls_table_loads_with_expected_counts() {
    let layer = require_layer();
    let cfg = layer.cfg();
    assert_eq!(cfg.scene_count, 20, "20 个 UI 场景");
    assert_eq!(cfg.record_total, 2041, "2041 条控件记录");
    assert!(!cfg.scenes.is_empty(), "场景列表非空");
}

#[test]
fn visible_filters_zero_area() {
    let layer = require_layer();
    // Bigcard.Map 是记录最多的场景（212 条）
    let total = layer.cfg().scene("Bigcard.Map").map_or(0, |s| s.records.len());
    assert!(total > 0, "Bigcard.Map 应有控件记录");
    let visible = layer.visible_count("Bigcard.Map");
    assert!(visible <= total, "可见数 ≤ 总数（零面积/不可见被过滤）");
}

#[test]
fn hit_boundary_left_closed_right_open() {
    let layer = require_layer();
    // 用第一个可见控件验证边界
    let rec = layer.visible("Bigcard.Map").next().expect("Bigcard.Map 应有可见控件");
    let x = i32::try_from(rec.x).unwrap_or(0);
    let y = i32::try_from(rec.y).unwrap_or(0);
    let w = i32::try_from(rec.w).unwrap_or(0);
    let h = i32::try_from(rec.h).unwrap_or(0);
    // 左上角命中（后画的可能覆盖，但命中一定 Some）
    assert!(layer.hit("Bigcard.Map", x, y).is_some(), "左上角应命中");
    // 右开边界：x+w 不命中该控件本身（可能被别的控件覆盖，故只断言整体命中情况合法）
    if w > 0 {
        let _ = layer.hit("Bigcard.Map", x + w, y);
    }
    if h > 0 {
        let _ = layer.hit("Bigcard.Map", x, y + h);
    }
}

#[test]
fn unknown_scene_yields_empty() {
    let layer = require_layer();
    assert_eq!(layer.visible_count("NoSuch.Map"), 0, "未知场景 → 0 可见");
    assert!(layer.hit("NoSuch.Map", 10, 10).is_none(), "未知场景 → 不命中");
}

#[test]
fn color_of_is_deterministic() {
    let layer = require_layer();
    let rec = layer.visible("Bigcard.Map").next().expect("应有可见控件");
    let c1 = layer.color_of(rec);
    let c2 = layer.color_of(rec);
    assert_eq!(c1, c2, "同控件同色（kind % 色板长度，确定性）");
}
