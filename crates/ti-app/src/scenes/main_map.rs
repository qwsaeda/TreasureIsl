//! 主地图场景（S9 骨架）：移动 / 贸易 / 建造 / 事件分派 + 等距网格绘制。
//!
//! 事件分派的跳转结果（`Vs` / `Message`）由路由层 [`crate::scenes::enter_target`]
//! 处理——那是跨场景的公共跳转点，不属于本场景。
use ti_core::data::balance;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::{Dir, GridPos, MapTerrain};
use ti_core::rules::build::cost_of;
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
                // S10.3 改动 5：买前查 `balance.trade.goods[i].qty_max`，超限时拒买
                let b = match balance() {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("balance.json 加载失败：{e}（硬报错，不 fallback）");
                        return false;
                    }
                };
                let Some(goods) = b.trade.goods.iter().find(|g| g.id == gid) else {
                    return false;
                };
                let held = host.owned_count(gid);
                if held >= goods.qty_max {
                    return false; // 持有量已达上限，拒买
                }
                if let Some(price) = host.price_of(gid) {
                    let gold = host.gold();
                    if gold >= price {
                        host.set_gold(gold - price);
                        match host.owned_index_of(gid) {
                            Some(index) => host.owned_bump(index),
                            None => host.owned_push(gid, 1),
                        }
                        host.play_sfx(Sfx::Press);
                    }
                }
                false
            }
            GameInput::TradeSell(gid) => {
                // S10.3 改动 3：卖价走 `balance.trade.markup`（`host.price_of` 已应用 markup 系数）
                let qty = host.owned_count(gid);
                if qty > 0 {
                    let gain = host.sell_price_of(gid).unwrap_or(0).saturating_mul(qty);
                    let gold = host.gold();
                    host.set_gold(gold.saturating_add(gain));
                    host.owned_take_all(gid);
                    host.play_sfx(Sfx::Press);
                }
                false
            }
            GameInput::Build(kind) => {
                // S10.3 改动 4：造价走 `balance.build.buildings`（`rules::build::cost_of` 纯函数）
                let b = match balance() {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("balance.json 加载失败：{e}（硬报错，不 fallback）");
                        return false;
                    }
                };
                let cost = cost_of(&b.build, kind);
                if cost > 0 && host.bank_gold() >= cost {
                    host.spend_bank_gold(cost);
                    host.build_increment(kind);
                    host.play_sfx(Sfx::Press);
                }
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
                // 点自己所在格 = 触发据点事件（等价键盘 Space）
                if target == cur {
                    return host.advance(GameInput::Landmark(6));
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

        // HUD（标题条 + 金/持有量）
        host.fill_rect(0, 0, 176, 20, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.main_map.title"), 64, 2);
        let gold = host.gold();
        let gold_str = i18n::sgetf("scene.main_map.hud.gold_fmt", &[&gold]);
        host.draw_text(&gold_str, 4, 150);
        let owned = host.owned_total();
        let own_str = i18n::sgetf("scene.main_map.hud.owned_fmt", &[&owned]);
        host.draw_text(&own_str, 4, 162);

        // 玩家棋子 + 据点（等距投影坐标，相机与地形一致；`terrain_camera` 为真值可能为负）
        let gw = host.terrain_grid_w();
        let (cam_x, cam_y) = host.terrain_camera();
        let cx = i32::try_from(cam_x).unwrap_or(0);
        let cy = i32::try_from(cam_y).unwrap_or(0);
        if let Some(pos) = host.map_pos() {
            if terrain_drawn {
                let iso_x = TerrainAtlas::iso_x(pos.row, pos.col);
                let iso_y = TerrainAtlas::iso_y(pos.row, pos.col, gw);
                host.fill_rect(iso_x + cx - 2, iso_y + cy - 2, 5, 5, COLOR_PLAYER);
            } else {
                let px = i32::try_from(pos.col).unwrap_or(0) * 6 + 20;
                let py = i32::try_from(pos.row).unwrap_or(0) * 6 + 30;
                host.fill_rect(px - 1, py - 1, 4, 4, COLOR_PLAYER);
            }
        }
        for (r, c) in host.map_landmarks() {
            if terrain_drawn {
                let iso_x = TerrainAtlas::iso_x(r, c);
                let iso_y = TerrainAtlas::iso_y(r, c, gw);
                host.fill_rect(iso_x + cx - 1, iso_y + cy - 1, 3, 3, COLOR_LANDMARK);
            } else {
                let px = i32::try_from(c).unwrap_or(0) * 6 + 20;
                let py = i32::try_from(r).unwrap_or(0) * 6 + 30;
                host.fill_rect(px - 1, py - 1, 3, 3, COLOR_LANDMARK);
            }
        }
        host.draw_text(i18n::sget("scene.main_map.hint"), 4, 210);
    }
}
