//! 全局常量（`docs/style.md` §5.3 的「全局」一列）。
//!
//! 来源：`legacy/docs/facts/index.md` F-106（UI 场景头实测）。
//! **逻辑层**分辨率：游戏逻辑坐标（地图几何 / .Map / grid）保持 `176×220`，
//! 与 UI 呈现层（`ti-platform::gfx` 的 `CANVAS` 1080×1920 现代画布）**解耦**。
//! 分域常量放在各自 crate 的 `consts.rs`，数值表一律资源化，不进代码。

/// 逻辑分辨率宽（像素，游戏逻辑坐标系）。原版 UI 场景头字段 `0xB0`。
pub const LOGICAL_WIDTH: u32 = 176;
/// 逻辑分辨率高（像素，游戏逻辑坐标系）。原版 UI 场景头字段 `0xDC`。
pub const LOGICAL_HEIGHT: u32 = 220;

#[cfg(test)]
mod tests {
    use super::{LOGICAL_HEIGHT, LOGICAL_WIDTH};

    #[test]
    fn matches_original_scene_header_fields() {
        assert_eq!(LOGICAL_WIDTH, 0xB0);
        assert_eq!(LOGICAL_HEIGHT, 0xDC);
    }
}
