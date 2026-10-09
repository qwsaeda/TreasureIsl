//! 全局常量（`docs/style.md` §5.3 的「全局」一列）。
//!
//! 来源：`docs/facts/index.md` F-106（UI 场景头实测）。
//! 分域常量放在各自 crate 的 `consts.rs`，数值表一律资源化，不进代码。

/// 逻辑分辨率宽（像素）。原版 UI 场景头字段 `0xB0`。
pub const LOGICAL_WIDTH: u32 = 176;
/// 逻辑分辨率高（像素）。原版 UI 场景头字段 `0xDC`。
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
