//! 主地图场景（S9 骨架）：移动 / 贸易 / 建造 / 事件分派 + 等距网格绘制。
//!
//! 事件分派的跳转结果（`Vs` / `Message`）由路由层 [`crate::scenes::enter_target`]
//! 处理——那是跨场景的公共跳转点，不属于本场景。
use ti_core::data::balance;
use ti_core::domain::economy::Building;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{Dir, GridPos, MapTerrain};
use ti_core::rules::main_map::MapSession;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_LANDMARK, COLOR_PLAYER, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind, enter_target};
use crate::terrain_atlas::TerrainAtlas;
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 单次点按的最大步数（避免一次触摸跨越半张图）。
const MAX_TOUCH_WALK: u32 = 6;

/// 主地图场景处理器（`MainMap`）。
pub struct MainMapScene;

/// 从当前位置朝 `to` 逐格移动：优先走差值大的轴，遇阻挡/越界即停（不穿墙）。
///
/// 每步经 [`SceneHost::advance`] 的 `Walk` 分支落地（与键盘路径同一实现），
/// 合法性预检走 `rules::main_map::MapSession::can_walk`。
fn walk_towards(host: &mut dyn SceneHost, terrain: &MapTerrain, to: GridPos) -> bool {
    let mut changed = false;
    for _ in 0..MAX_TOUCH_WALK {
        let Some(pos) = host.map_pos() else { break };
        let dr = i64::from(to.row) - i64::from(pos.row);
        let dc = i64::from(to.col) - i64::from(pos.col);
        if dr == 0 && dc == 0 {
            break;
        }
        // 行差绝对值 ≥ 列差 → 走纵向，否则走横向
        let dir = if dr.unsigned_abs() >= dc.unsigned_abs() {
            if dr < 0 { Dir::North } else { Dir::South }
        } else if dc < 0 {
            Dir::West
        } else {
            Dir::East
        };
        // 预检：不可通行则停下（不穿墙、不越界）
        if !MapSession::can_walk(terrain, pos, dir) {
            break;
        }
        changed |= host.advance(GameInput::Walk(dir));
    }
    changed
}

/// B 根治：买入走 `TradeLedger` 校验，可玩路径与业务线同一逻辑（双轨语义对齐）。
fn handle_trade_buy(host: &mut dyn SceneHost, gid: u32) {
    let b = match balance() {
        Ok(b) => b,
        Err(_e) => {
            #[cfg(feature = "stats")]
            eprintln!("[stats] TradeBuy: balance.json 加载失败（硬报错，不 fallback）");
            return;
        }
    };
    let mut ledger = ti_core::rules::trade::TradeLedger::new(
        b.trade.clone(),
        host.gold(),
        &host.owned_snapshot(),
        host.gold_cap(),
    );
    match ledger.buy(gid, 1) {
        Ok(res) => {
            host.set_gold(res.gold_left);
            match host.owned_index_of(res.gid) {
                Some(index) => host.owned_bump(index),
                None => host.owned_push(res.gid, res.qty),
            }
            host.play_sfx(Sfx::Press);
        }
        Err(_) => {
            host.play_sfx(Sfx::Invalid);
        }
    }
}

/// B 根治：卖出走 `TradeLedger` 结算 + 溢出进银行（修复 P1-6 T1 漏接点）。
fn handle_trade_sell(host: &mut dyn SceneHost, gid: u32) {
    let b = match balance() {
        Ok(b) => b,
        Err(_e) => {
            #[cfg(feature = "stats")]
            eprintln!("[stats] TradeSell: balance.json 加载失败（硬报错，不 fallback）");
            return;
        }
    };
    let mut ledger = ti_core::rules::trade::TradeLedger::new(
        b.trade.clone(),
        host.gold(),
        &host.owned_snapshot(),
        host.gold_cap(),
    );
    match ledger.sell(gid, 0) {
        Ok(res) => {
            host.set_gold(res.gold_left);
            host.owned_take_all(res.gid);
            if res.overflow > 0 {
                host.settle_overflow(res.overflow);
            }
            host.play_sfx(Sfx::Press);
        }
        Err(_) => {
            host.play_sfx(Sfx::Invalid);
        }
    }
}

/// B 根治：建造走 `BuildManager` 结算（等级 + 抬上限），可玩路径与业务线对齐。
fn handle_build(host: &mut dyn SceneHost, kind: Building) {
    let b = match balance() {
        Ok(b) => b,
        Err(_e) => {
            #[cfg(feature = "stats")]
            eprintln!("[stats] Build: balance.json 加载失败（硬报错，不 fallback）");
            return;
        }
    };
    let mut mgr = ti_core::rules::build::BuildManager::new(
        b.build.clone(),
        ti_core::rules::build::Bank { gold: host.bank_gold(), deposit: 0 },
        &[],
    );
    let bank_before = mgr.bank().gold;
    match mgr.build(ti_core::rules::build::kind_id(kind)) {
        Ok(()) => {
            let spent = bank_before.saturating_sub(mgr.bank().gold);
            host.spend_bank_gold(spent);
            host.build_increment(kind);
            host.play_sfx(Sfx::Press);
        }
        Err(_) => {
            host.play_sfx(Sfx::Invalid);
        }
    }
}

impl Scene for MainMapScene {
    fn kind(&self) -> SceneKind {
        SceneKind::MainMap
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Walk(dir) => {
                // `rules::main_map::MapSession::move_to` 行为（几何 + 据点 + 卡牌事件，可复现）
                let result = host.map_session_move(dir);
                if result.moved {
                    host.set_map_pos(Some(result.pos));
                }
                // P0-6：BOSS 格落格触发战斗（`progression.md` 落格结算）
                if result.boss {
                    host.start_battle();
                    return true;
                }
                // P1-7（M5）：卡牌事件真接进可玩路径——`card_event`（`StepResult`）
                // 经 `EconomyHost::apply_card_effect` 结算（扣金/加金/扣 HP/加 HP，`rules::cards`）。
                if let Some(event_id) = result.card_event {
                    if host.apply_card_effect(event_id) {
                        host.play_sfx(Sfx::Press);
                    }
                }
                // 落在据点 → 触发事件分派（与 `Landmark` 同路径）
                if let Some(target) = result.landmark {
                    host.set_last_dispatch(Some(target));
                    return enter_target(host, target);
                }
                false
            }
            GameInput::Landmark(code) => {
                // `rules::main_map::MapSession::dispatch` 行为（事件码 → 场景目标）
                let target = host.map_session_dispatch(code);
                host.set_last_dispatch(target);
                match target {
                    Some(t) => enter_target(host, t),
                    None => false,
                }
            }
            GameInput::TradeBuy(gid) => {
                handle_trade_buy(host, gid);
                false
            }
            GameInput::TradeSell(gid) => {
                handle_trade_sell(host, gid);
                false
            }
            GameInput::Build(kind) => {
                handle_build(host, kind);
                false
            }
            GameInput::MapBack => {
                host.enter(SceneKind::Index);
                true
            }
            _ => false,
        }
    }

    /// - 点格子 = 朝该格逐格走（每次点按走到能走的最远格，不穿墙）；
    /// - 点自己所在格 = 触发据点事件（`Landmark(code)`）；
    /// - 滑动 = 朝该方向走一步。
    ///
    /// 命中坐标与渲染同坐标系：真实地形（等距投影）走 `iso_grid_cell_at`，
    /// 占位网格（无地形资源）走 `grid_cell_at`（6×6 占位命中不变）。
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        let Some(cur) = host.map_pos() else {
            return false;
        };
        match sample.gesture {
            Gesture::Tap => {
                let Some(terrain) = host.map_terrain() else {
                    return false;
                };
                let cam = host.terrain_camera();
                let cell = if host.has_terrain_atlas() {
                    layout::iso_grid_cell_at(
                        sample.x,
                        sample.y,
                        terrain.grid_w,
                        terrain.grid_h,
                        cam,
                    )
                } else {
                    layout::grid_cell_at(sample.x, sample.y, terrain.grid_w, terrain.grid_h)
                };
                let Some((row, col)) = cell else {
                    return false;
                };
                let target = GridPos::new(row, col);
                // P0-5：点自身格 = 走 dispatch(1)→Msgl（据点菜单），与 business/progression.md L33 对齐
                if target == cur {
                    return host.advance(GameInput::Landmark(1));
                }
                walk_towards(host, &terrain, target)
            }
            Gesture::SwipeUp => host.advance(GameInput::Walk(Dir::North)),
            Gesture::SwipeDown => host.advance(GameInput::Walk(Dir::South)),
            Gesture::SwipeLeft => host.advance(GameInput::Walk(Dir::West)),
            Gesture::SwipeRight => host.advance(GameInput::Walk(Dir::East)),
        }
    }

    /// 等距地形瓦片（真资源）+ 玩家棋子 + 据点 + 金/持有量 HUD。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);

        // 真实地形瓦片（等距投影 + 摄像机），无地形资源时保持占位背景
        let terrain_drawn = host.draw_terrain();

        // P1-3：坐标/文案硬编码改读 `ti_core::data::scenes()`（scenes.json 字段已建）
        let layout = ti_core::data::scenes()
            .ok()
            .and_then(|s| s.get("main_map"))
            .cloned()
            .unwrap_or_default();

        // HUD（S14：棕色标题条 + 米白背景 + 等距瓦片 + 代码棋子/据点）
        host.fill_rect(0, 0, 176, layout.title.h, COLOR_TITLE_BAR);
        host.fill_rect(layout.hud.x, layout.hud.y, layout.hud.w, layout.hud.h, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.main_map.title"), layout.title.x, layout.title.y);
        let gold = host.gold();
        let gold_str = i18n::sgetf("scene.main_map.hud.gold_fmt", &[&gold]);
        host.draw_text(&gold_str, layout.hud.x, layout.hud.y);
        let owned = host.owned_total();
        let own_str = i18n::sgetf("scene.main_map.hud.owned_fmt", &[&owned]);
        host.draw_text(&own_str, layout.hud.x, layout.hud.y + 12);

        // 玩家棋子 + 据点（等距投影坐标，相机与地形一致；`terrain_camera` 为真值可能为负）
        let gw = host.terrain_grid_w();
        let (cam_x, cam_y) = host.terrain_camera();
        let cx = i32::try_from(cam_x).unwrap_or(0);
        let cy = i32::try_from(cam_y).unwrap_or(0);
        let g0 = layout.extra_scalar.get("grid.x0").copied().unwrap_or(20);
        let g1 = layout.extra_scalar.get("grid.y0").copied().unwrap_or(30);
        let gs = layout.extra_scalar.get("grid.step").copied().unwrap_or(6);
        if let Some(pos) = host.map_pos() {
            if terrain_drawn {
                let iso_x = TerrainAtlas::iso_x(pos.row, pos.col);
                let iso_y = TerrainAtlas::iso_y(pos.row, pos.col, gw);
                host.fill_rect(iso_x + cx - 2, iso_y + cy - 2, 5, 5, COLOR_PLAYER);
            } else {
                let px = i32::try_from(pos.col).unwrap_or(0) * gs + g0;
                let py = i32::try_from(pos.row).unwrap_or(0) * gs + g1;
                host.fill_rect(px - 1, py - 1, 4, 4, COLOR_PLAYER);
            }
        }
        for (r, c) in host.map_landmarks() {
            if terrain_drawn {
                let iso_x = TerrainAtlas::iso_x(r, c);
                let iso_y = TerrainAtlas::iso_y(r, c, gw);
                host.fill_rect(iso_x + cx - 1, iso_y + cy - 1, 3, 3, COLOR_LANDMARK);
            } else {
                let px = i32::try_from(c).unwrap_or(0) * gs + g0;
                let py = i32::try_from(r).unwrap_or(0) * gs + g1;
                host.fill_rect(px - 1, py - 1, 3, 3, COLOR_LANDMARK);
            }
        }
        // P0-2（M1）：BOSS 格橙红标记（4×4 方块，`COLOR_DANGER`，与玩家棋子同坐标系）
        if let Some(bt) = host.map_boss_tile() {
            if terrain_drawn {
                let iso_x = TerrainAtlas::iso_x(bt.row, bt.col);
                let iso_y = TerrainAtlas::iso_y(bt.row, bt.col, gw);
                host.fill_rect(
                    iso_x + cx - 2,
                    iso_y + cy - 2,
                    4,
                    4,
                    crate::menu_shell::COLOR_DANGER,
                );
            } else {
                let px = i32::try_from(bt.col).unwrap_or(0) * gs + g0;
                let py = i32::try_from(bt.row).unwrap_or(0) * gs + g1;
                host.fill_rect(px - 1, py - 1, 4, 4, crate::menu_shell::COLOR_DANGER);
            }
        }
        host.draw_text(i18n::sget("scene.main_map.hint"), layout.hint.x, layout.hint.y);
    }
}
