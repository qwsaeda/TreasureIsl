//! S29.3 P0：1080 层手动命中几何（`pick::hit_1080_pick`）无头单测。
//!
//! 根因（用户报「菜单按键无响应」）：`bevy_picking` 的 mesh 后端只拾取
//! `Mesh2d`/`Mesh3d`/`SimplifiedMesh`，1080 拾取标记是 `Sprite` ⇒ 从未被拾取；
//! 且 `bevy_ui` 全屏根节点默认 `Pickable` 会挡掉下层 1080 命中。修法 =
//! ①`bevy_ui` 装饰节点挂 `Pickable::IGNORE`；②1080 层走**纯几何**命中
//! （窗口逻辑 px 经 letterbox 换算 1920×1080 横屏画布 → 取包含点且 `z` 最大的 `pick_key` 原语矩形）。
//! 本文件 = 该几何的无头验证（不依赖渲染 / 指针 / GPU）。
//!
//! 单测只落 `crates/*/tests/`（EXP-052：src 内联 `#[cfg(test)]` 会被当占位白跑）。

use ti_engine::pick::{PickRect, hit_1080_pick};

/// 960×540 窗口（16:9）↔ 1920×1080 横屏画布：scale=0.5、offset=0。
/// 横屏画布中心 (960,540) 附近的目标 (888,468,144,144) → 窗口逻辑 (480,270)。
#[test]
fn hit_index_medallion() {
    let rects = [PickRect { key: 1, index: 0, z: -8, x: 888.0, y: 468.0, w: 144.0, h: 144.0 }];
    assert_eq!(hit_1080_pick(960.0, 540.0, 480.0, 270.0, 1920.0, 1080.0, &rects), Some((1, 0)));
}

/// 点落在矩形外 = 无命中。
#[test]
fn miss_outside() {
    let rects = [PickRect { key: 1, index: 0, z: -8, x: 888.0, y: 468.0, w: 144.0, h: 144.0 }];
    // 窗口 (10,10) = 画布 (20,20)，远离目标。
    assert_eq!(hit_1080_pick(960.0, 540.0, 10.0, 10.0, 1920.0, 1080.0, &rects), None);
}

/// 同点多矩形 = 取 `z` 最大（最上层）者。
#[test]
fn topmost_z_wins() {
    let rects = [
        PickRect { key: 1, index: 0, z: -8, x: 760.0, y: 360.0, w: 200.0, h: 200.0 },
        PickRect { key: 2, index: 7, z: -6, x: 810.0, y: 410.0, w: 200.0, h: 200.0 },
    ];
    // 窗口 (450,250) = 画布 (900,500)，两矩形重叠区；z=-6 更上。
    assert_eq!(hit_1080_pick(960.0, 540.0, 450.0, 250.0, 1920.0, 1080.0, &rects), Some((2, 7)));
}

/// 非 16:9 窗口（1080×1080 正方形 ↔ 1920×1080 画布 = 上下 letterbox）：
/// `scale = min(1080/1920, 1080/1080) = 0.5625`、`off_x = 0`、`off_y = 236.25`。
/// 命中仍正确（offset 生效）。
#[test]
fn letterbox_offset() {
    let rects = [PickRect { key: 3, index: 4, z: 0, x: 860.0, y: 460.0, w: 200.0, h: 200.0 }];
    // 窗口 (540,551.25) → 画布 ((540-0)/0.5625, (551.25-236.25)/0.5625) = (960,560)，落在 rect 内。
    assert_eq!(hit_1080_pick(1080.0, 1080.0, 540.0, 551.25, 1920.0, 1080.0, &rects), Some((3, 4)));
    // 窗口 (540,100) = 顶部 letterbox（画布 y 为负）→ 无命中。
    assert_eq!(hit_1080_pick(1080.0, 1080.0, 540.0, 100.0, 1920.0, 1080.0, &rects), None);
}
