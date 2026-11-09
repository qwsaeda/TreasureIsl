//! 地图包「编辑已有包」覆盖合并语义（`repo::import::merge_map_manifest` 纯函数面；0 写盘）。
//!
//! 判据：平台地图编辑器的导入 payload 只带它能表达的内容（31 格属性 + 环几何 + 画布 +
//! 焦点 + `events`/`merchants`/`vs_theme`），**不带**原包的资产登记（`assets`）、包级
//! `layout` 附加键、`strings` 附加条目 ⇒ 直接覆盖写入 = 包内容降级（`assets` 丢 ⇒ I7 资产
//! 引用消失；`layout.island_home`/`deck_*` 丢 ⇒ 布局回默认；`strings` 丢 ⇒ 图名条目消失）。
//!
//! 合并规则（编辑器管辖键 = 编辑器 UI 暴露的；其余 = 包级配置保留原值）。

use serde_json::{Value, json};
use ti_console::repo::import::merge_map_manifest;

/// 原包 manifest（编辑前）：含资产登记 / 包级 layout 附加键 / strings 附加条目。
fn base_manifest() -> Value {
    json!({
        "schema": "map-pack/1",
        "id": "m01",
        "version": "1.0.0",
        "data_version": "board-v5",
        "central_asset": "island_home_m01",
        "isle_assets": ["harbour"],
        "map": {
            "grid_w": 20, "grid_h": 20,
            "spaces": [{ "cell": [10, 0], "group": 1, "kind": "start", "terrain": "L" }]
        },
        "layout": {
            "ring": { "center_x": 540, "center_y": 784, "radius": 480, "card_w": 96, "card_h": 120 },
            "world_canvas": { "x": 11, "y": 250, "w": 1048, "h": 1056 },
            "island_home": { "x": 160, "y": 400, "w": 760, "h": 760 },
            "deck_trade": { "x": 48, "y": 1608, "w": 312, "h": 136 },
            "focus_bbox_pct": { "x": 1, "y": 13, "w": 97, "h": 55 }
        },
        "strings": { "glmapm01.Map": "m01", "glmapm01.Title": "原标题" },
        "assets": [{ "file": "assets/ui/t01_bg.png", "source": "pool", "sha256": "abc" }],
        "invariants": { "spaces": 31 },
        "vs_theme": { "bg": "vs_bg_m01" }
    })
}

/// 编辑器导入 payload（只带它能表达的内容；`assets` 空数组 = 未提供）。
fn incoming_manifest() -> Value {
    json!({
        "schema": "map-pack/1",
        "id": "m01",
        "version": "1.0.0",
        "data_version": "board-v5",
        "central_asset": "island_home_m01",
        "isle_assets": ["harbour", "wreck"],
        "map": {
            "grid_w": 20, "grid_h": 20,
            "spaces": [{ "cell": [10, 0], "group": 1, "kind": "start", "terrain": "L", "pos": [100, 300] }]
        },
        "layout": {
            "ring": { "center_x": 540, "center_y": 784, "radius": 480, "card_w": 96, "card_h": 120 },
            "world_canvas": { "x": 11, "y": 250, "w": 1048, "h": 1056 },
            "island_home": { "x": 160, "y": 400, "w": 760, "h": 760 },
            "deck_trade": { "x": 48, "y": 1608, "w": 312, "h": 136 },
            "focus_anchor": { "type": "world_canvas", "bbox": { "x": 11, "y": 250, "w": 1048, "h": 1056 } },
            "focus_bbox_pct": { "x": 1, "y": 13, "w": 97, "h": 55 }
        },
        "strings": { "glmapm01.Map": "m01" },
        "assets": [],
        "invariants": { "spaces": 31 },
        "vs_theme": { "bg": "vs_bg_m01" }
    })
}

#[test]
fn 覆盖导入保留原包资产登记() {
    let m = merge_map_manifest(&base_manifest(), &incoming_manifest());
    let assets = m["assets"].as_array().expect("assets 段保留");
    assert_eq!(assets.len(), 1, "assets 空数组 = 编辑器未提供 ⇒ 保留原登记");
    assert_eq!(assets[0]["file"], "assets/ui/t01_bg.png");
}

#[test]
fn 覆盖导入保留包级_layout附加键() {
    let m = merge_map_manifest(&base_manifest(), &incoming_manifest());
    assert!(m["layout"]["island_home"].is_object(), "island_home（包级）保留");
    assert!(m["layout"]["deck_trade"].is_object(), "deck_trade（包级）保留");
    assert_eq!(m["layout"]["ring"]["radius"], 480, "编辑器管辖键 ring 用编辑器值");
    assert!(m["layout"]["focus_anchor"].is_object(), "编辑器管辖键 focus_anchor 写入");
}

#[test]
fn 覆盖导入保留_strings附加条目() {
    let m = merge_map_manifest(&base_manifest(), &incoming_manifest());
    assert_eq!(m["strings"]["glmapm01.Map"], "m01");
    assert_eq!(m["strings"]["glmapm01.Title"], "原标题", "原 strings 附加条目保留");
}

#[test]
fn 覆盖导入写入编辑器改动格与岛表() {
    let m = merge_map_manifest(&base_manifest(), &incoming_manifest());
    assert_eq!(m["map"]["spaces"][0]["pos"][0], 100, "编辑器拖拽位置 pos 生效");
    assert_eq!(m["map"]["spaces"][0]["kind"], "start");
    assert_eq!(m["vs_theme"]["bg"], "vs_bg_m01");
    assert_eq!(m["isle_assets"].as_array().map(Vec::len), Some(2), "isle_assets 用编辑器值");
}

#[test]
fn 新建包无原manifest按原样采用() {
    let m = merge_map_manifest(&Value::Null, &incoming_manifest());
    assert_eq!(m, incoming_manifest(), "新建包语义不受合并影响");
}
