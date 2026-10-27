//! S29.3 P0：1080 层手动命中几何（`pick::hit_1080_pick`）无头单测。
//!
//! 根因（用户报「菜单按键无响应」）：`bevy_picking` 的 mesh 后端只拾取
//! `Mesh2d`/`Mesh3d`/`SimplifiedMesh`，1080 拾取标记是 `Sprite` ⇒ 从未被拾取；
//! 且 bevy_ui 全屏根节点默认 `Pickable` 会挡掉下层 1080 命中。修法 =
//! ①bevy_ui 装饰节点挂 `Pickable::IGNORE`；②1080 层走**纯几何**命中
//! （窗口逻辑 px 经 letterbox 换算 1080 → 取包含点且 `z` 最大的 `pick_key` 原语矩形）。
//! 本文件 = 该几何的无头验证（不依赖渲染 / 指针 / GPU）。
//!
//! 单测只落 `crates/*/tests/`（EXP-052：src 内联 `#[cfg(test)]` 会被当占位白跑）。

use ti_engine::pick::{hit_1080_pick, PickRect};

/// 540×960 窗口（9:16）↔ 1080×1920 画布：scale=0.5、offset=0。
/// index 徽章 item0（1080 中心 540,570）→ 窗口逻辑 (270,285)。
#[test]
fn hit_index_medallion() {
    let rects = [PickRect {
        key: 1, index: 0, z: -8, x: 468.0, y: 498.0, w: 144.0, h: 144.0,
    }];
    assert_eq!(hit_1080_pick(540.0, 960.0, 270.0, 285.0, 1080.0, 1920.0, &rects), Some((1, 0)));
}

/// 点落在矩形外 = 无命中。
#[test]
fn miss_outside() {
    let rects = [PickRect {
        key: 1, index: 0, z: -8, x: 468.0, y: 498.0, w: 144.0, h: 144.0,
    }];
    // 窗口 (10,10) = 1080 (20,20)，远离新游戏徽章。
    assert_eq!(hit_1080_pick(540.0, 960.0, 10.0, 10.0, 1080.0, 1920.0, &rects), None);
}

/// 同点多矩形 = 取 `z` 最大（最上层）者。
#[test]
fn topmost_z_wins() {
    let rects = [
        PickRect { key: 1, index: 0, z: -8, x: 400.0, y: 400.0, w: 200.0, h: 200.0 },
        PickRect { key: 2, index: 7, z: -6, x: 450.0, y: 450.0, w: 200.0, h: 200.0 },
    ];
    // 窗口 (300,300) = 1080 (600,600)，两矩形重叠区；z=-6 更上。
    assert_eq!(hit_1080_pick(540.0, 960.0, 300.0, 300.0, 1080.0, 1920.0, &rects), Some((2, 7)));
}

/// 非 9:16 窗口（1080×1080 正方形 ↔ 1080×1920 画布 = 左右 pillarbox）：
/// scale=min(1, 0.5625)=0.5625，off_x=236.25，off_y=0。命中仍正确（offset 生效）。
#[test]
fn letterbox_offset() {
    let rects = [PickRect { key: 3, index: 4, z: 0, x: 500.0, y: 500.0, w: 200.0, h: 200.0 }];
    // 窗口 (540,300) → 1080 ((540-236.25)/0.5625, 300/0.5625) = (540, 533.3)，落在 rect 内。
    assert_eq!(hit_1080_pick(1080.0, 1080.0, 540.0, 300.0, 1080.0, 1920.0, &rects), Some((3, 4)));
    // 窗口 (100,540) = 左侧 pillarbox（1080 x 为负）→ 无命中。
    assert_eq!(hit_1080_pick(1080.0, 1080.0, 100.0, 540.0, 1080.0, 1920.0, &rects), None);
}
