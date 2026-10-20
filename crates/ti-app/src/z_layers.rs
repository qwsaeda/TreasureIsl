//! S18.5：`z_index` 层系统 —— 双渲染路径（CPU 调用顺序 / Bevy `Transform.translation.z`）的
//! 层序值域单一事实源。
//!
//! 契约（`docs/process/08_render_arch.md` z-order = 调用顺序；
//! `docs/process/09_render_framework.md`：`ti-render` 无 z-order 枚举，层序在
//! `chrome::scene_shell` 调用序；S18.2 `BevyCanvas` 层序 = `Transform.translation.z`）：
//! - **CPU 路径**（`Framebuf`）：z 无显式值，层序 = 原语调用顺序（不变）；
//! - **Bevy 路径**：层 = `Transform.translation.z`（0.20 无独立 `ZIndex` 组件），
//!   取值 = 本表常量；`BevyCanvas::set_z` 值域 = i16 可表示（`f32::from(i16)`）；
//! - 两路径同值域（i16 可容纳；层间 100 整间距，防等值 z-fighting，
//!   层内可用 +0..99 微偏移排实体）。
//!
//! 用法：BevyCanvas `set_z` / chrome 层序断言一律取本表值；禁止再写裸字面量
//! （`canvas_contract.rs` S18.2 既有 0/2/5 字面量保持可复现，其相对序与本表同序）。

/// 层间步长（100 整间距：层内微偏移 +0..99 不跨层）。
pub const Z_STEP: i32 = 100;

/// 地形/背景层（最低层）。
pub const Z_TERRAIN: i32 = 0;

/// 实体层（船/角色/建筑）。
pub const Z_ENTITY: i32 = Z_TERRAIN + Z_STEP;

/// FX 层（S18.5 粒子：金币光/战斗命中火花）。
pub const Z_FX: i32 = Z_ENTITY + Z_STEP;

/// 文本层（三档文本）。
pub const Z_TEXT: i32 = Z_FX + Z_STEP;

/// HUD/chrome 层。
pub const Z_HUD: i32 = Z_TEXT + Z_STEP;

/// 覆盖层（最高：过渡暗化/场景提示）。
pub const Z_OVERLAY: i32 = Z_HUD + Z_STEP;

/// 全部层值（terrain → overlay 升序）。
#[must_use]
pub fn layer_values() -> [i32; 6] {
    [Z_TERRAIN, Z_ENTITY, Z_FX, Z_TEXT, Z_HUD, Z_OVERLAY]
}

/// 层值是否落在 Bevy `translation.z` 值域（i16 可表示，`BevyCanvas` flush 契约）。
#[must_use]
pub fn is_bevy_z_range(z: i32) -> bool {
    i16::try_from(z).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layers_strictly_monotone() {
        let v = layer_values();
        for w in v.windows(2) {
            assert!(w[0] < w[1], "层序须严格升序：{v:?}");
        }
    }

    #[test]
    fn layer_spacing_is_z_step() {
        let v = layer_values();
        for w in v.windows(2) {
            assert_eq!(w[1] - w[0], Z_STEP, "层间距 = Z_STEP（防等值 z-fighting）");
        }
    }

    #[test]
    fn all_layers_in_bevy_z_range() {
        for z in layer_values() {
            assert!(is_bevy_z_range(z), "层值 {z} 超出 i16 值域（Bevy translation.z 契约）");
        }
        assert!(Z_OVERLAY < i32::from(i16::MAX), "overlay 之上须留 i16 余量");
    }
}
