//! S14 正向渲染导航辅助：`GameApp` 状态机导航到 7 个有 `.Map` 的场景，
//! 供 `render-png` bin（出图）与 `tests/render_snapshot_forward.rs`（第三套像素快照门禁）共用。
//!
//! 数据源口径：S15.6 QA017——`forward_render` 开关已退役（S15.4 删 legacy 像素写帧层后
//! 渲染只有纯代码绘制一态，`route_render` 按 §4 版面；两态帧恒等 QA014 实证）。
//! 本模块只负责导航。
//!
//! 场景导航路径（与 `visual_regression.rs` legacy 态同口径，但 SelectRole/SelectMap 的
//! Confirm 兜底选第 0 项后，`select_role.rs`/`select_map.rs` 已修首次进场景 character/map_slot
//! 为 None 卡死的问题）：
//! - index = 默认
//! - select_role = Confirm
//! - select_map = Confirm + Confirm + Next（Next 设第 0 张地图）
//! - main_map = Confirm + Confirm + Next + Confirm
//! - vs = main_map + 直走 Boss 格（地图 0 的 player_start=(10,10)，boss_tile=(5,5)，
//!   10 步北 + 10 步西；每步经 `MapSession::move_to`，落 Boss 格 → `start_battle` → Vs）
//! - message = Confirm + Back + MoveDown×3 + Confirm（选项 → Help）
//! - saveload = MoveDown + Confirm（菜单项 1 → 存读档）

use ti_core::domain::map::GridPos;

use crate::game_app::{GameApp, GameInput, Scene};

/// 7 个有 `.Map` 的场景名（快照门禁出图清单，与 `render_snapshot.rs` 同 7 个）。
pub const FORWARD_SCENES: [&str; 7] =
    ["index", "select_role", "select_map", "main_map", "vs", "message", "saveload"];

/// 导航到目标场景（纯代码渲染态；`forward_render` 开关 S15.6 已退役，无状态前置）。
///
/// 返回落到的 `Scene`（调用方可核对 `actual == target`，防导航路径漂移）。
pub fn navigate(app: &mut GameApp, target: &str) -> Scene {
    match target {
        "index" => {} // 默认即 Index
        "select_role" => {
            app.handle(GameInput::Confirm);
        }
        "select_map" => {
            app.handle(GameInput::Confirm); // Index → SelectRole
            app.handle(GameInput::Confirm); // SelectRole → SelectMap（selection 不全）
            // select_map 的 Next 首次置 M01 并停留，再推进 M02/M03
            app.handle(GameInput::Next);
        }
        "main_map" => {
            app.handle(GameInput::Confirm); // Index → SelectRole
            app.handle(GameInput::Confirm); // SelectRole → SelectMap
            app.handle(GameInput::Next); // SelectMap 选第 0 张地图（设 map_slot）
            app.handle(GameInput::Confirm); // SelectMap → MainMap
        }
        "vs" => {
            app.handle(GameInput::Confirm); // Index → SelectRole
            app.handle(GameInput::Confirm); // SelectRole → SelectMap
            app.handle(GameInput::Next); // SelectMap 选第 0 张地图（设 map_slot）
            app.handle(GameInput::Confirm); // SelectMap → MainMap
            // 主地图 → 直走 Boss 格（player_start 到 boss_tile 的逐格 move_to，落格触发战斗）
            if !matches!(app.scene(), Scene::Vs) {
                walk_to_boss(app);
            }
        }
        "message" => {
            app.handle(GameInput::Confirm); // Index → SelectRole
            app.handle(GameInput::Back); // 回 Index
            app.handle(GameInput::MoveDown);
            app.handle(GameInput::MoveDown);
            app.handle(GameInput::MoveDown);
            app.handle(GameInput::Confirm); // 选项 → Help（Message）
        }
        "saveload" => {
            app.handle(GameInput::MoveDown);
            app.handle(GameInput::Confirm); // 菜单项 1 → Saveload
        }
        other => {
            eprintln!("forward_nav: unknown scene '{other}'，停留当前场景");
        }
    }
    app.scene()
}

/// 主地图 → 直走 Boss 格（`player_start` 到 `boss_tile` 逐格 `move_to`，落格触发 `start_battle`）。
///
/// 地图 0 数据：`player_start=(10,10)`，`boss_tile=(5,5)` → 需 10 步北 + 10 步西。
/// 逐格走（非大步跳）保证每步经 `MapSession::move_to` 的卡牌事件/据点判定，
/// 落 Boss 格那一步 `StepResult::boss=true` → `start_battle` → 切 Vs。
fn walk_to_boss(app: &mut GameApp) {
    let Some(boss) = app.map_boss_tile() else {
        eprintln!("forward_nav: 主地图无 Boss 格（boss_tile=None），无法触发战斗");
        return;
    };
    let Some(start) = app.map_pos() else {
        eprintln!("forward_nav: 主地图无当前坐标（map_pos=None）");
        return;
    };
    let (dr, dc) = boss_delta(start, boss);
    let vdir =
        if dr < 0 { ti_core::domain::map::Dir::North } else { ti_core::domain::map::Dir::South };
    let hdir =
        if dc < 0 { ti_core::domain::map::Dir::West } else { ti_core::domain::map::Dir::East };
    // 先走差值小的轴（横竖交错，不撞墙），再补差值大的轴
    for _ in 0..dr.abs().min(dc.abs()) {
        app.handle(GameInput::Walk(vdir));
        if matches!(app.scene(), Scene::Vs) {
            return;
        }
        app.handle(GameInput::Walk(hdir));
        if matches!(app.scene(), Scene::Vs) {
            return;
        }
    }
    if dr.abs() > dc.abs() {
        for _ in 0..(dr.abs() - dc.abs()) {
            app.handle(GameInput::Walk(vdir));
            if matches!(app.scene(), Scene::Vs) {
                return;
            }
        }
    } else {
        for _ in 0..(dc.abs() - dr.abs()) {
            app.handle(GameInput::Walk(hdir));
            if matches!(app.scene(), Scene::Vs) {
                return;
            }
        }
    }
}

/// `(start → boss)` 的行/列差（带符号：负 = 北/西，正 = 南/东）。
fn boss_delta(start: GridPos, boss: GridPos) -> (i64, i64) {
    let dr = i64::from(boss.row) - i64::from(start.row);
    let dc = i64::from(boss.col) - i64::from(start.col);
    (dr, dc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_app::GameApp;

    /// 导航路径漂移防护：7 场景 navigate 后实际 Scene 与目标一致（防状态机改动破坏导航）。
    #[test]
    fn forward_nav_reaches_all_scenes() {
        for target in FORWARD_SCENES {
            // QA020：呈现层 1080×1920 夹具（旧 176×220 字面已废，契约 1）
            let mut app = GameApp::new(1080.0, 1920.0);
            let actual = navigate(&mut app, target);
            let expected = expected_scene(target);
            assert_eq!(
                actual, expected,
                "forward_nav: '{target}' 实际落到 {actual:?}，期望 {expected:?}（导航路径漂移）"
            );
        }
    }

    fn expected_scene(target: &str) -> Scene {
        match target {
            "index" => Scene::Index,
            "select_role" => Scene::SelectRole,
            "select_map" => Scene::SelectMap,
            "main_map" => Scene::MainMap,
            "vs" => Scene::Vs,
            "message" => Scene::Message,
            "saveload" => Scene::Saveload,
            _ => unreachable!("FORWARD_SCENES 只含 7 个已列场景名"),
        }
    }
}
