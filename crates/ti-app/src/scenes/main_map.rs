//! 主地图场景（S9 骨架）：移动 / 贸易 / 建造 / 事件分派 + 等距网格绘制。
//!
//! 事件分派的跳转结果（`Vs` / `Message`）由路由层 [`crate::scenes::enter_target`]
//! 处理——那是跨场景的公共跳转点，不属于本场景。
//!
//! 布局数据 `main_map_v6_layout.json`（S33/2026-10-31 不规则岛 v6）经
//! `ti-core data 层常量`（`data::V6_LAYOUT_JSON`，CS-1 资产路径单源口径）
//! 编译期读入；场景层 0 资产路径字面量。
use ti_core::data;
use ti_core::data::{SceneLayout, balance};
use ti_core::domain::economy::Building;
use ti_core::domain::flow::Sfx;
use ti_core::rules::board::{SpaceKind, TrackSpace};
use ti_core::rules::build;
use ti_core::rules::trade;

use crate::game_app::{GameInput, HitData};
use crate::i18n;
use crate::menu_shell::{COLOR_LANDMARK, COLOR_PLAYER};
use crate::scenes::chrome;
use crate::scenes::main_map_island;
use crate::scenes::main_map_node;
use crate::scenes::{Scene, SceneHost, SceneKind, enter_target};

/// A5 生物动画时钟（S34.6 前处理）：`frame_delta_ms` 累计（帧率无关，S17.4 唯一时间源）。
/// creatures.png 是单张合成生态场景（无法拆元素级鲨/触手/浪花），程序 bob 微动
/// 满足 spec §1.4「海洋内容会动」+ U21 不叠格（位移 ±4px 保持原边界 1350>1349.2）。
static CREATURES_ACC: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// `TI_MAP_ANIM_T0`：生物动画初始相位（ms；离屏出多相位帧证明逐帧 bob，默认 0）。
static CREATURES_T0: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// 主地图场景处理器（`MainMap`）。
pub struct MainMapScene;

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
    let mut ledger = trade::TradeLedger::new(
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
    let mut ledger = trade::TradeLedger::new(
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
/// HUD 经济按钮（面板 + 图标 + 28px 标签；S17.3 图标取 `ui_icons.json` 注册表，
/// 宽度不足时自动退回纯文字居中）。
fn render_hud_button(host: &mut dyn SceneHost, layout: &SceneLayout, key: &str, label_key: &str) {
    let Some(r) = layout.extra.get(key).copied() else {
        return;
    };
    if r.w <= 0 || r.h <= 0 {
        return;
    }
    chrome::panel(host, r.x, r.y, r.w, r.h);
    let label = i18n::sget(label_key);
    let icon = chrome::hud_icon_def(key);
    let (label_x, icon_x) = chrome::layout_icon_label(host, &r, label, icon.as_ref());
    if let Some((id, size, ix)) = icon_x {
        chrome::ui_icon(host, id, ix, r.y + (r.h - size) / 2, false);
    }
    let ly = r.y + (r.h - host.shell().hint_glyph_px()) / 2;
    host.draw_hint_text(label, label_x, ly, chrome::COLOR_TEXT_ON_PARCH);
}

/// S21.6 v4 HUD：顶角双铜牌（`hud_plaque_l` 金币 / `hud_plaque_r` 船·持有量）+
/// 底部三键铜牌（JSON 原语 `deck_plaque` 贴图，点击命中仍走 `deck_*` 数据键）。
///
/// 各铜牌键缺失 = 回退 v3 边角矩形（`hud_gold`/`hud_player`）；再缺失 = 该部分不画。
///
/// 运行时只画数值文本（金币数 / 持有量），面板 = `main_map.primitives` 原语。
fn render_hud_v3(host: &mut dyn SceneHost, layout: &SceneLayout) {
    let (gold_str, own_str) = hud_labels(host);
    let mut plaque_text = |rc: &data::Rect, s: &str| {
        // 文本落铜牌右半（左侧 = 96/64px 图标区）：x = 牌左 + 176，基线 = 牌心 - 22
        chrome::draw_body_text(host, s, rc.x + 176, rc.y + rc.h / 2 - 22);
    };
    if let Some(rc) = layout
        .extra
        .get(chrome::keys::HUD_PLAQUE_L)
        .or_else(|| layout.extra.get(chrome::keys::HUD_GOLD))
    {
        if rc.w > 0 && rc.h > 0 {
            plaque_text(rc, &gold_str);
        }
    }
    if let Some(rc) = layout
        .extra
        .get(chrome::keys::HUD_PLAQUE_R)
        .or_else(|| layout.extra.get(chrome::keys::HUD_PLAYER))
    {
        if rc.w > 0 && rc.h > 0 {
            plaque_text(rc, &own_str);
        }
    }
}

/// S17.8 v2 HUD：羊皮纸 HUD 面板（金币/持有量两行 44px）+ 三经济按钮（QA149）。
fn render_hud_v2(host: &mut dyn SceneHost, layout: &SceneLayout) {
    chrome::panel(host, layout.hud.x, layout.hud.y, layout.hud.w, layout.hud.h);
    let (gold_str, own_str) = hud_labels(host);
    chrome::draw_body_text(host, &gold_str, layout.hud.x + 4, layout.hud.y + 2);
    chrome::draw_body_text(host, &own_str, layout.hud.x + 4, layout.hud.y + 56);
    for (key, label_key) in [
        (chrome::keys::HUD_TRADE, "scene.main_map.hud.trade"),
        (chrome::keys::HUD_SELL, "scene.main_map.hud.sell"),
        (chrome::keys::HUD_BUILD, "scene.main_map.hud.build"),
    ] {
        render_hud_button(host, layout, key, label_key);
    }
}

fn handle_build(host: &mut dyn SceneHost, kind: Building) {
    let b = match balance() {
        Ok(b) => b,
        Err(_e) => {
            #[cfg(feature = "stats")]
            eprintln!("[stats] Build: balance.json 加载失败（硬报错，不 fallback）");
            return;
        }
    };
    let mut mgr = build::BuildManager::new(
        b.build.clone(),
        build::Bank { gold: host.bank_gold(), deposit: host.bank_deposit() },
        &[],
    );
    let bank_before = mgr.bank().gold;
    match mgr.build(build::kind_id(kind)) {
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

/// S21.12 建造面板条目 → `Building`（第 5 项 = 造船，`None`）。
fn build_entry_kind(entry: usize) -> Option<Building> {
    [Building::Castle, Building::Investment, Building::Bank, Building::Fort].get(entry).copied()
}

/// 建筑条目文案键（`strings.json` 既有键，与 HUD/面板同一文案源）。
fn build_entry_name_key(kind: Building) -> &'static str {
    match kind {
        Building::Castle => "build.castle",
        Building::Investment => "build.investment",
        Building::Bank => "build.bank",
        Building::Fort => "build.fort",
    }
}

/// HUD 双铜牌经济文案（金币 / 持有量；归并 `hud.gold_fmt`+`hud.owned_fmt` 两处重复取值，
/// 文案键单一读点 = 本 helper，`audit_code` 重复模式去重）。
fn hud_labels(host: &dyn SceneHost) -> (String, String) {
    let gold_str = i18n::sgetf("scene.main_map.hud.gold_fmt", &[&host.gold()]);
    let own_str = i18n::sgetf("scene.main_map.hud.owned_fmt", &[&host.owned_total()]);
    (gold_str, own_str)
}

/// 面板条目右侧「造价 / 下一级」文案（建筑 = `build.cost_fmt`；造船 = `build.next_boat` / 满级）。
fn build_right_label(host: &dyn SceneHost, cfg: &data::BuildCfg, entry: usize) -> String {
    if let Some(kind) = build_entry_kind(entry) {
        return i18n::sgetf(
            "scene.main_map.build.cost_fmt",
            &[&build::cost_of(cfg, kind).to_string()],
        );
    }
    let level = host.boat_level();
    match host.boat_next_cost() {
        Some(cost) => {
            let next_key = format!("boat.{}", level + 1);
            i18n::sgetf(
                "scene.main_map.build.next_boat",
                &[&i18n::sget(&next_key).to_string(), &cost.to_string()],
            )
        }
        None => i18n::sget("boat.max").to_string(),
    }
}

/// 造船成功提示条文案（`build.boat_up_fmt`）。
fn boat_upgrade_toast(name: &str) -> String {
    let name_arg = name.to_string();
    i18n::sgetf("scene.main_map.build.boat_up_fmt", &[&name_arg])
}

/// 面板条目图标（4 建筑 = `chrome::BUILD_ENTRY_ICONS` 图标表；造船 = 当前级船图，缺图回退链在渲染侧）。
fn build_entry_icon(host: &dyn SceneHost, entry: usize) -> &'static str {
    chrome::BUILD_ENTRY_ICONS
        .get(entry)
        .copied()
        .unwrap_or_else(|| chrome::boat_asset(host.boat_level()))
}

// S34 用户 2026-11-01「底部三键取掉」：`open_build_panel`（原 pick_key=7 触发）随三键原语删除；
// 建造面板交互 = 键盘 ↑↓/Enter/Back（`on_input`，`build_panel_open` 状态机保留）。
/// S21.12 面板当前选中项（越界 = 末条目；选中域 0..=`BUILD_LAST_ENTRY`）。
fn panel_sel_clamped(host: &dyn SceneHost) -> usize {
    usize::from(host.build_panel_sel().min(chrome::BUILD_LAST_ENTRY))
}

/// S21.12 确认面板条目：0..3 = 建筑（`GameInput::Build` 既有事件 → `handle_build` 扣金/记录口径）；
/// 4 = 造船（`GameInput::BoatUpgrade` 事件 → `handle_boat_upgrade`；成功关面板，失败保持打开 + 提示条说明原因）。
fn confirm_build_panel(host: &mut dyn SceneHost, entry: usize) {
    if let Some(kind) = build_entry_kind(entry) {
        host.set_build_panel_open(false);
        host.set_track_node_sel(None);
        let _ = host.advance(GameInput::Build(kind));
    } else {
        host.set_track_node_sel(None);
        let before = host.boat_level();
        let _ = host.advance(GameInput::BoatUpgrade);
        if host.boat_level() != before {
            host.set_build_panel_open(false);
        }
    }
}

/// v4 黄铜航线线宽（px；实线；实现几何，非版面坐标）。
const TRACK_LINE_W: i32 = 4;

/// 黄铜航线实线（相邻赛道格中心连线；沿线等距铺线宽方块 ⇒ 连续实线）。
fn draw_route_line(host: &mut dyn SceneHost, centers: &[(i32, i32)], width: i32, color: u16) {
    let half = width / 2;
    for w in centers.windows(2) {
        let (ax, ay) = (w[0].0, w[0].1);
        let (bx, by) = (w[1].0, w[1].1);
        let (dx, dy) = (bx - ax, by - ay);
        let steps = dx.abs().max(dy.abs()).max(1);
        for i in 0..=steps {
            let x = ax + dx * i / steps;
            let y = ay + dy * i / steps;
            host.fill_rect(x - half, y - half, width, width, color);
        }
    }
}

/// v3 席位徽标（`entity_texts` 只收 `&'static str` ⇒ 编译期常量表，不 format）。
///
/// 热座（2~5 人类 + 可配 AI）必须能一眼分辨「哪枚棋子是哪一席」，否则同格散开后
/// 仍是四枚无标识灰旗。`P1` 起编号 = 席位序号 + 1。
const SEAT_BADGES: [&str; 5] = ["P1", "P2", "P3", "P4", "P5"];

/// v3 同格多席棋子散开（大富翁惯例：开局全员同起点出发，叠成一枚 = 看不出谁在哪）。
///
/// 输入 `ordinal` = 同格组内序位、`total` = 同格席数、`radius` = 散开半径（px）。
/// 输出相对格心的 `(dx, dy)`：单席 = `(0, 0)` 居中不偏移；n 席 = 沿小圆周均分
/// （起始角 -90° 顶部、顺时针），相邻中心距 = `2·radius·sin(π/n)`。
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn seat_piece_offset(ordinal: usize, total: usize, radius: i32) -> (i32, i32) {
    if total <= 1 || radius <= 0 {
        return (0, 0);
    }
    let step = std::f64::consts::TAU / total as f64;
    let ang = -std::f64::consts::FRAC_PI_2 + ordinal.min(total - 1) as f64 * step;
    let (sin, cos) = ang.sin_cos();
    let dx = (f64::from(radius) * cos).round() as i32;
    let dy = (f64::from(radius) * sin).round() as i32;
    (dx, dy)
}

/// v3：把 `seat_positions()` 归到「同格分组」，返回逐席 `(seat, idx, ordinal, total)`。
///
/// `ordinal`/`total` 交 [`seat_piece_offset`] 扇出；`idx` 已按 `count` 夹紧。
fn seat_slots(positions: &[(u8, usize)], count: usize) -> Vec<(u8, usize, usize, usize)> {
    let n = count.max(1);
    let mut totals = vec![0usize; n];
    for (_, idx) in positions {
        if let Some(t) = totals.get_mut((*idx).min(n - 1)) {
            *t += 1;
        }
    }
    let mut seen = vec![0usize; n];
    positions
        .iter()
        .map(|(seat, idx)| {
            let idx = (*idx).min(n - 1);
            let ord = seen.get(idx).copied().unwrap_or(0);
            if let Some(s) = seen.get_mut(idx) {
                *s += 1;
            }
            (*seat, idx, ord, totals.get(idx).copied().unwrap_or(1))
        })
        .collect()
}

/// S21.5 v4 环岛赛道绘制：**黄铜航线实线 + 45 枚多形状节点牌 + 玩家船**。
///
/// - 航线：相邻赛道格中心连线（theme 令牌 `brass`），先画线后压牌 ⇒ 线在牌下、牌间露出；
/// - 节点牌：`SpaceKind` → `chrome::node_asset`（**唯一事实源** = 规格 §6：9 类多形状黄铜牌）；
/// - 玩家船：`ship_player` 贴图落在当前格（缺资产 = 代码 `flag` 图标回退）。
///
/// 坐标全部经 `chrome::iso_cell_fit`（与命中判定同源，EXP-067 正逆成对）。
fn render_track(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    gw: u32,
    entity_texts: &mut Vec<(i32, i32, &str)>,
) {
    let spaces = host.board_spaces();
    if spaces.len() < 2 {
        return;
    }
    let (tw, th) = chrome::iso_tile_fit(layout, gw);
    let theme = host.theme().clone();
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
    // 节点中心（等距菱形中心 = 格子绘制盒中心；与 `iso_cell_fit` 同源）
    let centers: Vec<(i32, i32)> = spaces
        .iter()
        .map(|sp| {
            let r = i32::try_from(sp.cell.row).unwrap_or(0);
            let c = i32::try_from(sp.cell.col).unwrap_or(0);
            let (sx, sy) = chrome::iso_cell_fit(r, c, layout, gw);
            (sx + tw, sy + th)
        })
        .collect();

    // 1) 黄铜航线实线
    draw_route_line(host, &centers, TRACK_LINE_W, brass);

    // 2) 45 枚节点牌（可见外接框归一化 ⇒ 9 类牌视觉大小一致、形状各异）
    let edge = chrome::node_edge_px(layout, gw);
    for (i, sp) in spaces.iter().enumerate() {
        let (cx, cy) = centers[i];
        let def = chrome::node_asset(sp.kind);
        let (dx, dy, dw, dh) = chrome::asset_dest_rect(cx, cy, edge, def.visible_px);
        host.blit_image(def.path, dx, dy, dw, dh);
    }

    // 3) BOSS 标签（全部文本最上层，QA142）
    for (i, sp) in spaces.iter().enumerate() {
        if sp.kind == SpaceKind::Boss {
            let (cx, cy) = centers[i];
            let lbl = i18n::sget("scene.main_map.boss_label");
            entity_texts.push((cx + 12, cy - 8, lbl));
        }
    }

    // 4) 多席棋子（v3：遍历 `seat_positions()`；当前席 = 金色船 + 名牌，
    // 其余席 = 暗灰小旗；缺资产 = 代码 `flag` 图标回退）。
    let turn_seat = host.turn_seat();
    let cur_idx = host.board_index().min(spaces.len() - 1);
    let cur_asset = chrome::boat_asset(host.boat_level());
    let ship_edge = chrome::ship_edge_px(layout, gw);
    // v3：同格多席（开局全员同起点）沿小圆周扇出 + 缩到 70%，叠成一枚 = 看不出谁在哪。
    for (seat, idx, ord, total) in seat_slots(&host.seat_positions(), spaces.len()) {
        let (cx, cy) = centers[idx];
        let (ox, oy) = seat_piece_offset(ord, total, ship_edge * 2 / 5);
        let (cx, cy) = (cx + ox, cy + oy);
        let edge = if total > 1 { ship_edge * 7 / 10 } else { ship_edge };
        let (dx, dy, dw, dh) =
            chrome::asset_dest_rect(cx, cy, edge, chrome::SHIP_PLAYER_VISIBLE_PX);
        let badge = SEAT_BADGES[usize::from(seat).min(SEAT_BADGES.len() - 1)];
        if seat == turn_seat {
            if chrome::asset_present(cur_asset) {
                host.blit_image(cur_asset, dx, dy, dw, dh);
            } else if chrome::asset_present(chrome::SHIP_PLAYER_ASSET) {
                host.blit_image(chrome::SHIP_PLAYER_ASSET, dx, dy, dw, dh);
            } else {
                chrome::entity_icon(host, "flag", dx, dy, dw, chrome::COLOR_GOLD_WARM);
            }
            let lbl = i18n::sget("scene.main_map.player_label");
            entity_texts.push((cx + edge / 2, cy - 8, lbl));
            entity_texts.push((cx - edge / 2, cy + edge / 2, badge));
        } else {
            let small = dw.min(dh) * 35 / 100;
            chrome::entity_icon(host, "flag", cx - small / 2, cy - small / 2, small, 0x7BEF);
            entity_texts.push((cx - small / 2, cy + small / 2, badge));
        }
    }

    // 4.5) S21.7：节点预览选中（`gold_warm` 高亮环 + 信息签）+ 落点脉冲（toast 窗口内
    // 当前格画环，「1 拍」= `board_toast` 120 帧寿命）。
    let gold_warm = theme.by_name("gold_warm").unwrap_or(chrome::COLOR_GOLD_WARM);
    if let Some(i) = host.track_node_sel() {
        if i < spaces.len() {
            let (nx, ny) = centers[i];
            let def = chrome::node_asset(spaces[i].kind);
            chrome::draw_node_ring(host, nx, ny, edge, def.visible_px, gold_warm);
            main_map_node::draw_node_card(
                host,
                nx,
                ny - edge / 2,
                ny + edge / 2,
                spaces[i].kind,
                &theme,
            );
        }
    }
    if host.board_toast().is_some() && cur_idx < spaces.len() {
        let (nx, ny) = centers[cur_idx];
        let def = chrome::node_asset(spaces[cur_idx].kind);
        chrome::draw_node_ring(host, nx, ny, edge, def.visible_px, gold_warm);
    }

    // 5) 骰子数（摇骰按钮上方居中，`board_last_roll` = 0 → 不画）
    let roll = host.board_last_roll();
    if roll > 0 {
        let roll_str = chrome::roll_label(roll);
        let rx = 540 - host.shell().hint_text_width_px(&roll_str) / 2;
        let ry = layout.extra.get(chrome::keys::DECK_SPIN).map_or(1296, |r| r.y - 24);
        host.draw_hint_text(&roll_str, rx, ry, chrome::COLOR_GOLD_WARM);
    }
    // 6) 落格提示签条（S21.6 v4 = `hint` 键中心；`board_toast` 过期 = `None`）
    // 6.5) S21.12 建造面板覆盖层（状态在 `GameApp`；关闭 = 不画）
    if host.build_panel_open() {
        render_build_panel(host);
    }
    render_board_toast(host);
    // S36.3 终局面板（`territory.md` §4.4）：`match_outcome` 在位 = 对局结束，
    // 主地图中心画胜利/失败/平局面板 + 「按 Confirm 返回主菜单」提示。
    render_match_outcome(host);
    // S36.3 热座交接提示条（§1 + §1.2）：`handoff_pending` 在位 = 轮到下一人类席，
    // 画底部提示条（「热座交接：按 Confirm 确认」），确认前隐藏信息（§1.2）。
    render_handoff(host);
}

/// S21.12 造船（`GameInput::BoatUpgrade` 事件处理）：走 `rules::boat` 单向升级链
/// （扣金 + 抬 cargo/atk/hp 立即生效）；失败 = 提示条说明原因（金不足 / 已满级）。
fn handle_boat_upgrade(host: &mut dyn SceneHost) {
    if host.boat_next_cost().is_none() {
        host.set_board_toast(i18n::sget("boat.max").to_string());
        host.play_sfx(Sfx::Invalid);
        return;
    }
    if host.boat_upgrade() {
        let key = format!("boat.{}", host.boat_level());
        let name = i18n::sget(&key);
        host.set_board_toast(boat_upgrade_toast(name));
        host.play_sfx(Sfx::Press);
    } else {
        host.set_board_toast(i18n::sget("scene.main_map.build.need_gold").to_string());
        host.play_sfx(Sfx::Invalid);
    }
}

/// S21.12 建造面板渲染（主地图覆盖层；标题 + 5 条目 + 底部提示，toast 同底板风格）。
fn render_build_panel(host: &mut dyn SceneHost) {
    let (bx, by, bw, bh) = chrome::BUILD_PANEL;
    let theme = host.theme().clone();
    let panel = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
    host.fill_rect(bx, by, bw, bh, panel);
    host.fill_rect(bx, by, bw, 2, brass);
    host.fill_rect(bx, by + bh - 2, bw, 2, brass);
    host.fill_rect(bx, by, 2, bh, brass);
    host.fill_rect(bx + bw - 2, by, 2, bh, brass);
    host.draw_title_text(
        i18n::sget("scene.main_map.build.panel_title"),
        bx + 24,
        by + 16,
        chrome::COLOR_TEXT_ON_PARCH,
    );
    // §4.4 数据加载失败 = 硬错误（不进 fallback）：跳过本帧面板（`handle_build` 同一口径）
    let b = match balance() {
        Ok(b) => b,
        Err(_e) => {
            #[cfg(feature = "stats")]
            eprintln!("[stats] Build panel: balance.json 加载失败，跳过本帧面板");
            return;
        }
    };
    let sel = panel_sel_clamped(host);
    for i in 0..chrome::BUILD_PANEL_ENTRIES {
        let (ex, ey, ew, eh) = chrome::build_entry_rect(i);
        if i == sel {
            host.fill_rect(ex, ey, ew, eh, brass);
        }
        // 图标（白蒙版 + gold tint；造船 = 当前级船图；缺失 = 跳过，名称仍在）
        let icon = build_entry_icon(host, i);
        if chrome::asset_present(icon) {
            host.blit_image_tint(icon, ex + 16, ey + eh / 2 - 28, 56, 56, chrome::COLOR_GOLD_WARM);
        }
        // 名称（左）
        let name = build_entry_kind(i).map_or_else(
            || i18n::sget("scene.main_map.build.boat").to_string(),
            |k| i18n::sget(build_entry_name_key(k)).to_string(),
        );
        host.draw_text(&name, ex + 88, ey + 16);
        // 造价 / 下一级（右；文案键归并 `build_right_label`）
        let right = build_right_label(host, &b.build, i);
        host.draw_hint_text(&right, ex + ew - 320, ey + 56, chrome::COLOR_TEXT_ON_PARCH);
    }
    host.draw_hint_text(
        i18n::sget("scene.main_map.build.hint"),
        bx + 24,
        by + bh - 48,
        chrome::COLOR_TEXT_ON_PARCH,
    );
}

impl Scene for MainMapScene {
    fn kind(&self) -> SceneKind {
        SceneKind::MainMap
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        // S21.12 建造面板键盘控制（↑↓ 选 / Enter 确认 / Back 关闭；面板打开时吞掉其它输入）
        if host.build_panel_open() {
            match input {
                GameInput::MoveUp => {
                    let n = host.build_panel_sel();
                    host.set_build_panel_sel(if n == 0 { chrome::BUILD_LAST_ENTRY } else { n - 1 });
                    host.play_sfx(Sfx::Press);
                }
                GameInput::MoveDown => {
                    let n = host.build_panel_sel();
                    host.set_build_panel_sel(if n >= chrome::BUILD_LAST_ENTRY { 0 } else { n + 1 });
                    host.play_sfx(Sfx::Press);
                }
                GameInput::Confirm => {
                    let sel = panel_sel_clamped(host);
                    confirm_build_panel(host, sel);
                }
                GameInput::Back => host.set_build_panel_open(false),
                // 面板确认动作：落入下方主分发（`handle_boat_upgrade` 落 toast/音效）
                GameInput::BoatUpgrade => {}
                _ => return false,
            }
        }
        match input {
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
            GameInput::BoatUpgrade => {
                handle_boat_upgrade(host);
                false
            }
            GameInput::Spin => {
                // 环岛赛道（`rules::board`，`progression.md` §环岛赛道）：摇骰定步数 → 前进 N 格 → 落格结算。
                // 落格效果（金/HP/卡/提示条）已由 `board_spin` 结算；S19.3：经过/落在 BOSS 格且
                // 绕满 `boss_min_laps` 圈 → BOSS 现身 → 切战斗场景。
                let Some(turn) = host.board_spin() else {
                    return false;
                };
                // S21.7：摇骰 = 移动唯一入口 ⇒ 预览选中清空（落点脉冲由 toast 窗口接管）。
                host.set_track_node_sel(None);
                // S21.12：离开主地图状态（摇骰/进战斗）前关闭建造面板
                host.set_build_panel_open(false);
                if turn.boss.engaged() {
                    host.start_battle();
                    return true;
                }
                false
            }
            GameInput::MapBack => {
                host.set_build_panel_open(false);
                host.enter(SceneKind::Index);
                true
            }
            // S28.6：落无主领地格后的待决副本入口（`pending_dungeon` 在位时 Confirm = 进守军战；
            // 缺位 no-op，主地图 Confirm 无其它语义）。S33.5：落他人领地格后的待决攻占
            // （`pending_siege` 在位时 Confirm = 进攻占战报）——优先级 = 攻占 > 副本（不同格）。
            // v3：**热座交接确认优先**（§1 + §1.2）：轮到新人类席后须显式确认才能操作，
            // 否则下一位玩家会看到上一位的操作残留（隐藏信息）。
            // S36.3 终局面板（§4.4）：`match_outcome` 在位 = 对局结束，Confirm = 清终局 + 回主菜单。
            GameInput::Confirm => {
                // v3：热座交接确认优先（§1 + §1.2）：轮到新人类席后须显式确认才能操作。
                if host.confirm_handoff() {
                    return false;
                }
                // S36.3 终局面板：对局已结束 ⇒ 清 `match_outcome` + 回主菜单（零回归：旧档无终局）。
                if host.match_outcome().is_some() {
                    host.clear_match_outcome();
                    host.enter(SceneKind::Index);
                    return true;
                }
                if host.confirm_siege() {
                    true
                } else {
                    host.confirm_dungeon();
                    false
                }
            }
            // S33.5：落他人领地格后的待决攻占，`Back` = 放弃攻占交过路费（`pending_siege` 在位）。
            GameInput::Back => host.pay_siege_toll(),
            _ => false,
        }
    }

    /// S29.2：主地图拾取命中（`pick_key` 数据驱动）。完整 HUD 三经济臂 / 摇骰 CTA / 赛道节点 /
    /// 建造面板的 `pick_key` 数据 + 命中随 **S27.4（v5 环形棋盘）/ S29.3（全场景 `bevy_ui` 迁移）**
    /// 补齐；本任务（S29.2）先接**基础臂**：`key 4..=7` = 摇骰/买/卖/建造（与键盘 Space/T/Y/B 同
    /// 语义），`key=0`（无标注）/ 滑动 = 无动作（S19：移动唯一 = 摇骰）。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        if hit.swipe != 0 {
            return false;
        }
        // S34 用户 2026-11-01「底部三键取掉」：原 5/6/7（TradeBuy/TradeSell/建造）随三键原语
        // 一并删除（scenes.json main_map primitives 只余 pick_key=4 摇骰 CTA）；建造面板
        // 交互 = 键盘 ↑↓/Enter/Back（`on_input`），命中层不再有 5/6/7。
        match hit.key {
            4 => host.advance(GameInput::Spin),
            _ => false,
        }
    }

    /// 等距地形瓦片（真资源）+ 玩家棋子 + 据点 + 金/持有量 HUD（v2 羊皮纸/木纹/金色系）。
    #[allow(clippy::too_many_lines)]
    fn render(&mut self, host: &mut dyn SceneHost) {
        // P1-3：坐标/文案硬编码改读 `ti_core::data::scenes()`（scenes.json 字段已建）；
        // QA031：统一走 `chrome::scene_layout`（scenes.json 单一读点，契约 4）
        let layout = chrome::scene_layout("main_map");
        // S17.8 v3：v3 模式判定（world_canvas 键在位 = v3；缺席回退 v2 静态 grid.*）。
        let v3 = chrome::has_world_canvas(&layout);
        // S34：v6 终码默认路径（无 `TI_MAP_V6_PROTO` / `TI_MAP_V5_RING` 环境变量）。
        // S34 用户 2026-11-01 复验：v6 海图（`render_v6_elem`）满屏盖住早画的 HUD 数值
        // （`render_hud_v3` 是运行时 text 非 z>=3 原语，`render_primitives_filtered` 只重画
        // z>=3 原语 ⇒ 顶角铜牌无数值）。v6 默认路径把 HUD 数值**延迟到世界层之后**重画。
        let v6_default = v3
            && std::env::var("TI_MAP_V6_PROTO").is_err()
            && std::env::var("TI_MAP_V5_RING").is_err();
        // S15.4：legacy 地形瓦片像素层移除。主地图 = 羊皮纸底 + 航海网格装饰 + 代码绘制等距棋盘（§4.3）
        // S18.7f：夜空底 + 画布框 + HUD/命令盘 chrome = JSON 原语（**必须先于瓦片**：
        // 夜空渐变是背景层，瓦片/实体叠其上）；v2 回退 = 羊皮纸底 + 航海网格。
        // S31+/v6：v6 世界层（`render_v6_elem`）已画全屏海图，若先画 primitives(z>=2 背景) 会盖住 HUD（z>=3）
        // ⇒ v6 默认路径先画世界层，后画 HUD 原语（min_z=3）。
        if v3 {
            if !v6_default {
                chrome::render_primitives(host, "main_map", -1);
            }
            // v6_default：原语延迟到 render_v6_elem 后（下方分支），避免海图盖住 HUD 原语。
        } else {
            chrome::parchment_bg(host);
            chrome::nautical_grid(host);
        }
        let gw = host.terrain_grid_w();
        // S27.4 v5：`world_canvas` 在位 = v3 ⇒ **环形棋盘**（`render_v5_ring`：中央据点岛 +
        // 31 张岛型/功能卡环 + 玩家船；几何 = `chrome::ring_geom` 数据驱动，`10_main_map_v5_spec.md`）。
        // v2 回退（`world_canvas` 缺席，dormant）= 等距岛屿 + 等距赛道（`terrain_grid_w`² G 依赖
        // 拟合进 `world_canvas` 2:1 菱形，缺席回退静态 `grid.*`）。
        if !v3 {
            let bg = chrome::board_fit(&layout, gw);
            let gw_i = i32::try_from(gw).unwrap_or(0);
            let (tw, th) = (bg.tw, bg.th);
            main_map_island::draw_board(host, &layout, gw, gw_i, tw, th);
        }

        // S18.7f：v3 版面外壳（夜空底 + 世界画布框 + HUD 边角面板 + 底部命令盘 + 标题 +
        // 提示 + 木框）100% 数据驱动 = `scenes.json` `main_map.primitives`（画布框
        // = 旧 `shell::world_canvas_frame` 同几何像素等价）。瓦片/实体/HUD 数值仍运行时。
        // v2 回退（v3 键缺席）：羊皮纸底 + 木纹标题条（双写纪律，v3.5 收口删）。
        if !v3 {
            chrome::wood_frame(host);
            chrome::title_bar(host, layout.title.h);
            let title = i18n::sget("scene.main_map.title");
            let cx = chrome::centered_title_x(host, title);
            let title_color = host.theme().text_on_wood;
            chrome::draw_title_text(host, title, cx, layout.title.y, title_color);
        }

        // S17.8：v3 边角化 HUD + 底部命令盘 / v2 羊皮纸 HUD 条 + 三按钮（双写，键缺席即 v2）。
        if v3 {
            // v6 默认路径：HUD 数值延迟到世界层之后（`render_v6_elem` 海图会盖住早画文本；
            // 见 `v6_default` 注释 + 下方 v6 分支补画）。其余 v3 路径照旧先画。
            if !v6_default {
                render_hud_v3(host, &layout);
            }
        } else {
            render_hud_v2(host, &layout);
        }

        // 玩家棋子 + 据点 + BOSS 格（§4.3 代码绘制等距坐标，S17.8 v3 `chrome::iso_cell_fit`
        // G 依赖几何 / 缺席回退 v2 `iso_cell`；实体边长 `chrome::entity_size_fit`：
        // v3 = `canvas_entities.<key>` 百分比 × 棋盘 tw，缺席回退 v2 `<key>.size`（QA106））。
        let entity_size = |key: &str| -> i32 { chrome::entity_size_fit(&layout, key, gw) };
        // S15.6 QA142：主地图 z-order（渲染线契约）= parchment_bg → nautical_grid → 瓦片 →
        // chrome 填充（wood_frame/title_bar/HUD 面板/经济按钮面板）→ 实体填充 → **全部文本最上层**。
        // 旧实现把 HUD 文字/按钮标签/实体标签**交错**画在各自 fill 之后，实体方块会盖住
        // 早画的 HUD 文字（QA142 缺陷）。这里先把实体方块画完，再统一补上所有文本。

        // 第一遍：实体填充（玩家棋子 + 据点 + BOSS），记录它们的文本落点。
        let mut entity_texts: Vec<(i32, i32, &str)> = Vec::new();
        // S21.5：有环岛赛道时，玩家标记 = **船**（当前格，`render_track` 内画）；
        // **无赛道**（v2 回退）= 绿方块 + `flag` 代码图标。
        let spaces = host.board_spaces();
        if let Some(pos) = host.map_pos() {
            let pr = i32::try_from(pos.row).unwrap_or(0);
            let pc = i32::try_from(pos.col).unwrap_or(0);
            let (sx, sy) = chrome::iso_cell_fit(pr, pc, &layout, gw);
            let (tw, th) = chrome::iso_tile_fit(&layout, gw);
            let s = entity_size("player");
            // 实体在瓦片绘制盒（2tw × 2th）内居中
            let (px, py) = (sx + (tw * 2 - s) / 2, sy + (th * 2 - s) / 2);
            if spaces.is_empty() {
                host.fill_rect(px, py, s, s, COLOR_PLAYER);
                // S18.12：实体图标化（旗 = 玩家船队；48px 原生图标居中于实体盒）
                chrome::entity_icon(host, "flag", px, py, s, chrome::COLOR_GOLD_WARM);
                // QA142：实体 1px bevel 描边（凸出感，不越实体盒）
                host.fill_rect(px, py, s, 1, chrome::COLOR_BEVEL_HI);
                host.fill_rect(px, py + s - 1, s, 1, chrome::COLOR_BEVEL_SHADOW);
                let lbl = i18n::sget("scene.main_map.player_label");
                entity_texts.push((px + s, py, lbl));
            }
        }
        for (r, c) in host.map_landmarks() {
            // S19：有环岛赛道时，据点由赛道空间标记接管（下方 `render_track`），
            // 不再单独画 balance 据点（避免与赛道空间双标记）。
            if !spaces.is_empty() {
                continue;
            }
            let rr = i32::try_from(r).unwrap_or(0);
            let cc = i32::try_from(c).unwrap_or(0);
            let (sx, sy) = chrome::iso_cell_fit(rr, cc, &layout, gw);
            let (tw, th) = chrome::iso_tile_fit(&layout, gw);
            let s = entity_size("landmark");
            let (lx, ly) = (sx + (tw * 2 - s) / 2, sy + (th * 2 - s) / 2);
            host.fill_rect(lx, ly, s, s, COLOR_LANDMARK);
            chrome::entity_icon(
                host,
                "beacon",
                lx,
                ly,
                s,
                ti_render::theme::Theme::DEFAULT.moon_pale,
            );
            host.fill_rect(lx, ly, s, 1, chrome::COLOR_BEVEL_HI);
            host.fill_rect(lx, ly + s - 1, s, 1, chrome::COLOR_BEVEL_SHADOW);
            let lbl = i18n::sget("scene.main_map.landmark_label");
            entity_texts.push((lx + s, ly, lbl));
        }
        // P0-2（M1）：BOSS 格血红标记（有赛道时由 `render_track` 的 BOSS 空间接管）
        if let Some(bt) = host.map_boss_tile() {
            if spaces.is_empty() {
                let br = i32::try_from(bt.row).unwrap_or(0);
                let bc = i32::try_from(bt.col).unwrap_or(0);
                let (sx, sy) = chrome::iso_cell_fit(br, bc, &layout, gw);
                let (tw, th) = chrome::iso_tile_fit(&layout, gw);
                let s = entity_size("boss");
                let (bx, by) = (sx + (tw * 2 - s) / 2, sy + (th * 2 - s) / 2);
                host.fill_rect(bx, by, s, s, crate::menu_shell::COLOR_DANGER);
                chrome::entity_icon(host, "sword", bx, by, s, crate::menu_shell::COLOR_DANGER);
                host.fill_rect(bx, by, s, 1, chrome::COLOR_BEVEL_HI);
                host.fill_rect(bx, by + s - 1, s, 1, chrome::COLOR_BEVEL_SHADOW);
                let lbl = i18n::sget("scene.main_map.boss_label");
                entity_texts.push((bx + s, by, lbl));
            }
        }
        if v3 {
            // S27.4 v5 环形棋盘：中央据点岛 + 31 张卡环 + 玩家船 + 选中环 + 骰/落格签条。
            // 2026-10-30 用户拍板 v6「羊皮石柱环岛」（设计稿先行）：`TI_MAP_V6_PROTO` 环境变量
            // ⇒ 原型渲染（羊皮泛黄底 + 相连石柱格 + 弹出骰子），未确认终稿前不改 v5 主路径。
            // 2026-10-31 用户拍板「不规则岛，格子紧围岛不规则排列」+ 迭看确认 + 引擎渲染确认
            // ⇒ **v6 落终码**：默认走 `render_v6_elem`（不规则岛 + 格子紧围岛，布局 JSON 数据驱动）；
            //    v5 环形棋盘退 `TI_MAP_V5_RING` 环境变量（原型保留）。
            if std::env::var("TI_MAP_V6_PROTO").is_ok() {
                chrome::render_primitives(host, "main_map", -1);
                render_v6_proto(host, &layout, &mut entity_texts);
            } else if std::env::var("TI_MAP_V5_RING").is_ok() {
                chrome::render_primitives(host, "main_map", -1);
                render_v5_ring(host, &layout, &mut entity_texts);
            } else {
                // v6 终码：不规则岛 + 格子紧围岛不规则排列（布局 JSON = compose 导出，数据驱动）
                // 世界层先画，再画 HUD 原语（z>=3），避免海图满屏盖住 HUD
                render_v6_elem(host, &layout, &mut entity_texts);
                chrome::render_primitives_filtered(host, "main_map", -1, 3);
                // v6 补层（最上层）：①HUD 数值（`render_hud_v3` 在 642 行被 `v6_default`
                // 跳过，此处海图+原语之后重画，顶角铜牌不再空）②摇骰 CTA 骰点 + 骰数
                // （舵轮 = z=7 原语，骰点须叠其轴心之上，`render_spin_die` 的 `deck_spin`
                // 是 v5 残留位 ⇒ 用 v6 布局 helm 定位）。
                render_hud_v3(host, &layout);
                render_v6_spin_die(host);
            }
        } else {
            // S19：等距环岛赛道（路径连线 + 空间标记 + 骰子数 + 落格提示条）+ S19.7 航程条（v2 dormant）
            render_track(host, &layout, gw, &mut entity_texts);
            chrome::voyage_bar(host, "main_map");
        }

        // 第二遍：实体标签（28px 提示档，QA142 全部文本最上层）
        for (x, y, lbl) in &entity_texts {
            host.draw_hint_text(lbl, *x, *y, chrome::COLOR_TEXT_ON_PARCH);
        }

        // QA032：hint 行统一走 `chrome::scene_hint`（i18n 单查收敛，EXP-082）
        // S15.6：key = `nav_hint`（`hint` 已并入，`strings.json` 删孤儿）
    }
}

fn render_v5_ring(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    entity_texts: &mut Vec<(i32, i32, &str)>,
) {
    let spaces = host.board_spaces();
    let count = spaces.len();
    if count < 2 {
        return;
    }
    let theme = host.theme().clone();
    let gold_warm = theme.by_name("gold_warm").unwrap_or(chrome::COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_BRASS);

    // 1) 中央据点岛（运行时 blit `island_home_<mid>.png` @ `island_home` 矩形；缺图 = 不画，不占位）
    if let Some(mid) = host.map_slot().map(crate::game_app::map_slot_id) {
        let path = chrome::central_island_path(mid);
        if chrome::asset_present(&path) {
            let (ix, iy, iw, ih) = layout
                .extra
                .get(chrome::keys::ISLAND_HOME)
                .map_or((160, 400, 760, 760), |r| (r.x, r.y, r.w, r.h));
            host.blit_image(&path, ix, iy, iw, ih);
        }
    }

    // 2) 31 张卡环（领地格 = isle 整幅卡面；功能格 = 羊皮卡底 + node_ 牌）
    // 绘制序 = **按卡心 y 升序**（上方卡先画、下方卡后画）⇒ 相邻卡重叠时呈「牌叠」层次，
    // 而非按 index 顺序的杂乱穿插（概念稿变体 B 的环形牌语言）。
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by_key(|i| chrome::ring_card_center(*i, count, layout).1);
    for index in order {
        draw_ring_card(host, &spaces[index], index, count, layout, &theme, brass);
    }

    // 3) 多席棋子（v3：遍历 `seat_positions()` 画所有在局席；当前席 = 金色船，
    // 其余席 = 暗灰旗；缺资产 = 代码 flag 回退；船边长随卡宽缩放，非硬编码）。
    let ship_edge: i32 = chrome::ring_geom(layout).w * 55 / 100;
    let turn_seat = host.turn_seat();
    let cur_asset = chrome::boat_asset(host.boat_level());
    // v3：同格多席扇出（开局全员同起点）；单席保持卡面中心原尺寸。
    for (seat, idx, ord, total) in seat_slots(&host.seat_positions(), count) {
        let (cx, cy) = chrome::ring_card_center(idx, count, layout);
        let (ox, oy) = seat_piece_offset(ord, total, ship_edge * 2 / 5);
        let (cx, cy) = (cx + ox, cy + oy);
        let edge = if total > 1 { ship_edge * 7 / 10 } else { ship_edge };
        let badge = SEAT_BADGES[usize::from(seat).min(SEAT_BADGES.len() - 1)];
        if seat == turn_seat {
            // 当前行动席：金色船（同旧路径）
            if chrome::asset_present(cur_asset) {
                host.blit_image(cur_asset, cx - edge / 2, cy - edge / 2, edge, edge);
            } else {
                chrome::entity_icon(host, "flag", cx - edge / 2, cy - edge / 2, edge, gold_warm);
            }
            let lbl = i18n::sget("scene.main_map.player_label");
            entity_texts.push((cx + edge / 2, cy - 8, lbl));
            entity_texts.push((cx - edge / 2, cy + edge / 2, badge));
            draw_card_name_plate(host, &spaces[idx], cx, cy, layout, &theme, brass, gold_warm);
        } else {
            // 非当前席：暗灰小旗（区分，不遮岛面）
            let small_edge = edge * 35 / 100;
            chrome::entity_icon(
                host,
                "flag",
                cx - small_edge / 2,
                cy - small_edge / 2,
                small_edge,
                0x7BEF, // 暗灰 RGB565
            );
            entity_texts.push((cx - small_edge / 2, cy + small_edge / 2, badge));
        }
    }

    // 4) 选中高亮（track_node_sel = 预览选中卡；金环 3px 外扩）
    if let Some(sel) = host.track_node_sel() {
        if sel < count {
            let (sx, sy, sw, sh) = chrome::ring_card_rect(sel, count, layout);
            host.fill_rect(sx - 4, sy - 4, sw + 8, 3, gold_warm);
            host.fill_rect(sx - 4, sy + sh + 1, sw + 8, 3, gold_warm);
            host.fill_rect(sx - 4, sy - 4, 3, sh + 8, gold_warm);
            host.fill_rect(sx + sw + 1, sy - 4, 3, sh + 8, gold_warm);
        }
    }

    // 5) 骰子数（摇骰按钮上方；board_last_roll = 0 不画）
    let roll = host.board_last_roll();
    if roll > 0 {
        let roll_str = chrome::roll_label(roll);
        let rx = 540 - host.shell().hint_text_width_px(&roll_str) / 2;
        let ry = layout.extra.get(chrome::keys::DECK_SPIN).map_or(1312, |r| r.y - 24);
        host.draw_hint_text(&roll_str, rx, ry, gold_warm);
    }
    // 6) 建造面板 + 落格提示签条（v5 不画航程进度尺）
    if host.build_panel_open() {
        render_build_panel(host);
    }
    // 7) 摇骰 CTA 轴心骰点（`dice.png`；S27.5 用户报「没有筛子」= CTA 只有空舵轮；
    // 舵轮本体 = `scenes.json` 原语（更早画），此处补轴心骰点，与卡环区无重叠）。
    render_spin_die(host, layout);
    render_board_toast(host);
    // S36.3 飞轮 F-7.3：v5 环路径补终局面板 + 热座交接提示（render_track 已有，v5 此前缺失）。
    render_match_outcome(host);
    render_handoff(host);
}

// —— v6 原型几何常量（`TI_MAP_V6_PROTO` 专用；不相叠平面拼凑格：格宽 < 每格弧长留海水缝）——
const V6_CENTER: (i32, i32) = (540, 740);
const V6_RADIUS: i32 = 460;
// V6_CARD_W/V6_CARD_H 退役（S26.2 岛屿图片化后格子不再用卡片尺寸）。
/// 动漫海蓝（用户 2026-10-31：海洋要蓝色非深绿黑；本地常量不动 theme）。
const V6_SEA_BLUE: u16 = 0x29B5; // ≈(46,110,170)
/// 动漫浅波（海面波纹高光）。
const V6_SEA_FOAM: u16 = 0x7ADB; // ≈(120,180,220)

// —— v6 不规则布局（`TI_MAP_V6_ELEM=1`，2026-10-31 用户「不规则岛，格子紧围岛不规则排列」）——
// 单一事实源 = `scripts/compose_main_map_v6.py::tile_positions_around_island`（凸包弧长等分 +
// 径向外推贴沿），导出 `assets/data/main_map_v6_layout.json`，本模块 `include_str!` 编译期读入，
// 引擎只按数据 blit（数据驱动，不硬编码坐标；§3a「精确布局 = 脚本算」）。
#[derive(serde::Deserialize)]
struct V6Layout {
    tile_r: f32,
    #[serde(default)]
    island: V6Rect,
    #[serde(default)]
    ship: V6Rect,
    #[serde(default)]
    helm: V6Rect,
    #[serde(default)]
    compass: V6Rect,
    #[serde(default)]
    tiles: Vec<V6Tile>,
}

#[derive(serde::Deserialize, Default)]
struct V6Rect {
    #[serde(default)]
    cx: f32,
    #[serde(default)]
    cy: f32,
    #[serde(default)]
    w: f32,
}

#[derive(serde::Deserialize)]
struct V6Tile {
    #[serde(default)]
    cx: f32,
    #[serde(default)]
    cy: f32,
    #[serde(default)]
    owned: bool,
}

/// 编译期读入 v6 布局（缺文件 = 空布局，回退环布局）。
fn v6_layout() -> V6Layout {
    serde_json::from_str(data::V6_LAYOUT_JSON).unwrap_or(V6Layout {
        tile_r: 36.0,
        island: V6Rect::default(),
        ship: V6Rect::default(),
        helm: V6Rect::default(),
        compass: V6Rect::default(),
        tiles: Vec::new(),
    })
}

/// v6 第 `index` 个石柱格心（`θ = -90° + index·360°/count`，index=0 顶部起点顺时针）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
fn v6_pillar_center(index: usize, count: usize) -> (i32, i32) {
    let count_f = f32::from(u16::try_from(count.max(1)).unwrap_or(1));
    let index_f = f32::from(u16::try_from(index).unwrap_or(0));
    let radius_f = V6_RADIUS as f32;
    let rad = (-90.0 + index_f * 360.0 / count_f) * std::f32::consts::PI / 180.0;
    let dx = (f64::from(radius_f) * f64::from(rad.cos())).round();
    let dy = (f64::from(radius_f) * f64::from(rad.sin())).round();
    (V6_CENTER.0 + dx as i32, V6_CENTER.1 + dy as i32)
}

/// S27.5 v6 原型（`TI_MAP_V6_PROTO` 环境变量 ⇒ 走本函数；未确认终稿前不改 v5 主路径）。
///
/// 用户 2026-10-30 拍板 `main_map` 4 条缺陷的可视化（A 纪律设计稿先行，未确认不落终码）：
/// ①骰子到谁弹出（不常驻底部，带摇骰动作+语音）②建造三键占领格才弹出
/// ③默认地图 = 羊皮泛黄岛屿风 ④格子相连不叠加、立体间隔海水、类浅水不规则石柱（人踩石柱）。
///
/// 本函数 = 原型渲染（羊皮泛黄底 + 石柱格 + 弹出骰子 + 海水间隔），
/// 几何/色板数据驱动（`theme.by_name`），不改 `scenes.json` 版面坐标。
#[allow(clippy::too_many_lines)]
fn render_v6_proto(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    entity_texts: &mut Vec<(i32, i32, &str)>,
) {
    let spaces = host.board_spaces();
    let count = spaces.len();
    if count < 2 {
        return;
    }
    let theme = host.theme().clone();
    let gold_warm = theme.by_name("gold_warm").unwrap_or(chrome::COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_BRASS);
    let sand_light = theme.by_name("land_sand_light").unwrap_or(chrome::COLOR_GOLD_WARM);
    let sand_dark = theme.by_name("land_sand_dark").unwrap_or(chrome::COLOR_BRASS_DARK);
    let sea = V6_SEA_BLUE; // 角切/凹槽透出海蓝（与海洋圆一致；theme sea_shallow 偏深绿黑非蓝色）
    let ink = theme.by_name("parch_dark").unwrap_or(chrome::COLOR_TEXT_ON_PARCH);
    let shadow = chrome::COLOR_BRASS_DARK;

    // 0) 羊皮泛黄底：**满屏**（盖掉壳灰青 chart 底；用户 2026-10-31 背景不羊皮 → 泛黄羊皮全屏）
    let (wx, wy, ww, wh) = (0, 0, 1080, 1920);
    host.fill_rect(wx, wy, ww, wh, sand_light);
    // 旧纸纤维噪点（确定性伪随机浅斑）
    let mut seed: u32 = 0x9E37_79B9;
    for _ in 0..180 {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let px = wx + i32::try_from(seed % u32::try_from(ww.max(1)).unwrap_or(1)).unwrap_or(0);
        let py =
            wy + i32::try_from((seed >> 16) % u32::try_from(wh.max(1)).unwrap_or(1)).unwrap_or(0);
        let dot = 2 + i32::try_from((seed >> 24) % 5).unwrap_or(0);
        host.fill_rect(px, py, dot, dot, ink);
    }
    // 四边暗角做旧（**无网格**，用户 2026-10-30 二轮：泛黄羊皮不是网格；边缘深棕渐晕）
    let stain = shadow;
    for w in [48i32, 30, 14] {
        let c = if w == 14 { ink } else { stain };
        host.fill_rect(wx, wy, ww, w, c);
        host.fill_rect(wx, wy + wh - w, ww, w, c);
        host.fill_rect(wx, wy, w, wh, c);
        host.fill_rect(wx + ww - w, wy, w, wh, c);
    }
    // 罗盘装饰（左上弱化圈）：外圈 + 8 方位短线
    let (comp_x, comp_y, comp_r) = (180, 300, 110);
    host.fill_rect(comp_x - comp_r, comp_y - 2, comp_r * 2, 4, brass);
    host.fill_rect(comp_x - 2, comp_y - comp_r, 4, comp_r * 2, brass);
    host.fill_rect(
        comp_x - comp_r * 71 / 100,
        comp_y - comp_r * 71 / 100,
        comp_r * 142 / 100,
        3,
        brass,
    );
    host.fill_rect(
        comp_x - comp_r * 71 / 100,
        comp_y + comp_r * 71 / 100 - 3,
        comp_r * 142 / 100,
        3,
        brass,
    );

    // 1) 海洋区（环 + 中央岛被**动漫蓝色海洋**包围；羊皮纸露四角/外缘）——fill_rect 逐行模拟圆
    let sea_r = V6_RADIUS + 110;
    let (scx, scy) = V6_CENTER;
    let mut dy = -sea_r;
    while dy <= sea_r {
        // half ≤ sea_r（i32 有界）；截断安全（`sea_r` 远小于 `i32::MAX`，`as` 无精度损失）。
        #[allow(clippy::cast_possible_truncation)]
        let half = f64::from(sea_r * sea_r - dy * dy).max(0.0).sqrt() as i32;
        host.fill_rect(scx - half, scy + dy, half * 2, 1, sea);
        dy += 1;
    }
    // 海洋波纹（动漫海面：浅色弧纹，确定性伪随机）
    let foam = V6_SEA_FOAM;
    let mut seed2: u32 = 0x00C0_FFEE;
    for _ in 0..30 {
        seed2 = seed2.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let rx =
            scx + i32::try_from(seed2 % u32::try_from(sea_r * 2).unwrap_or(1)).unwrap_or(0) - sea_r;
        let ry = scy
            + i32::try_from((seed2 >> 16) % u32::try_from(sea_r * 2).unwrap_or(1)).unwrap_or(0)
            - sea_r;
        let rw = 34 + i32::try_from((seed2 >> 24) % 46).unwrap_or(0);
        host.fill_rect(rx, ry, rw, 3, foam);
    }

    // 2) 中央据点岛（**紧贴环格**：放大到接近环内缘，海水窄缝；保留 island_home 资产）
    if let Some(mid) = host.map_slot().map(crate::game_app::map_slot_id) {
        let path = chrome::central_island_path(mid);
        if chrome::asset_present(&path) {
            let ir = V6_RADIUS - 66; // 岛半径 ≈ 环半径 - 格 - 海水缝 ⇒ 紧贴
            let (ix, iy) = (V6_CENTER.0 - ir, V6_CENTER.1 - ir);
            host.blit_image(&path, ix, iy, ir * 2, ir * 2);
        }
    }

    // 3) 31 个平面棋盘格（随机拼凑形状，浮于海洋上；格间海水间隔不相叠）。
    //    绘制序 = 按格心 y 升序（上方先画、下方后画）。
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by_key(|i| v6_pillar_center(*i, count).1);
    for index in order {
        draw_v6_tile(
            host,
            &spaces[index],
            index,
            count,
            &theme,
            brass,
            gold_warm,
            sand_light,
            sand_dark,
            sea,
            shadow,
        );
    }

    // 4) 弹出骰子（中央大区域掷骰动画，先画 → 玩家棋子顶层不被遮）
    draw_v6_spin_popup(host, &theme, gold_warm, brass);

    // 5) 多席棋子（v3：遍历 `seat_positions()` 画所有在局席；当前席 = 金色船 + 岛名牌，
    // 其余席 = 暗灰小旗；圆铜牌节点上，非浮柱）。
    let turn_seat = host.turn_seat();
    let cur_asset = chrome::boat_asset(host.boat_level());
    // v3：同格多席扇出（开局全员同起点）；单席保持柱顶原尺寸。
    for (seat, idx, ord, total) in seat_slots(&host.seat_positions(), count) {
        let (cx, cy) = v6_pillar_center(idx, count);
        let (ox, oy) = seat_piece_offset(ord, total, 20);
        let (cx, ship_y) = (cx + ox, cy - 22 + oy);
        let edge = if total > 1 { 44 * 7 / 10 } else { 44 };
        let badge = SEAT_BADGES[usize::from(seat).min(SEAT_BADGES.len() - 1)];
        if seat == turn_seat {
            if chrome::asset_present(cur_asset) {
                host.blit_image(cur_asset, cx - edge / 2, ship_y, edge, edge);
            } else {
                chrome::entity_icon(host, "flag", cx - edge / 2, ship_y, edge, gold_warm);
            }
            let lbl = i18n::sget("scene.main_map.player_label");
            entity_texts.push((cx + edge / 2, ship_y - 4, lbl));
            entity_texts.push((cx - edge / 2, ship_y + edge, badge));
            draw_card_name_plate(host, &spaces[idx], cx, cy, layout, &theme, brass, gold_warm);
        } else {
            let small_edge = edge * 35 / 100;
            chrome::entity_icon(host, "flag", cx - small_edge / 2, ship_y, small_edge, 0x7BEF);
            entity_texts.push((cx - small_edge / 2, ship_y + small_edge, badge));
        }
    }

    // 5) 底部三键弹出式（v6 用户裁决）：不常驻——覆盖常驻三键区域为羊皮底，只留
    //    「落占领格才弹出建造/贸易/出售」说明铜牌；`build_panel_open` 时弹出建造面板
    //   （用毕确定退出副本 = 交互语义，原型静态画面板示意）。
    // S34 用户 2026-11-01「底部三键取掉」：deck_trade/deck_sell/deck_build 三键原语已从
    // scenes.json 删除（本函数不再有羊皮底覆盖对象）；`layout.extra` 无 deck_* 键（顶层键），
    // 旧覆盖逻辑是死代码，一并删除。建造面板（`build_panel_open`）交互 = 键盘（on_input），
    // 面板 + 提示签条照旧。
    if host.build_panel_open() {
        render_build_panel(host);
    }
    render_board_toast(host);
    // S36.3 飞轮 F-7.3：v6 原型路径补终局面板 + 热座交接提示。
    render_match_outcome(host);
    render_handoff(host);
}

/// 缆绳段：`fill_rect` 逐像素 Bresenham 近似（`blit_image` 无 line 原语；两格间连线）。
/// `width` 奇偶 + 中心对齐；RGB565 色（compose 用 RGB(74,52,28)=深棕 / RGB(102,72,40)=高光）。
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
fn rope_segment(
    host: &mut dyn SceneHost,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    width: i32,
    color: u16,
) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        host.fill_rect(x - width / 2, y - width / 2, width, width, color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// v6 元素合成版（`TI_MAP_V6_ELEM=1`，2026-10-31 用户「方向对了」→ 引擎真实渲染）。
///
/// 2026-10-31 用户「不规则岛，格子紧围岛不规则排列」：直接 blit AI 元素到 1080×1920，
/// 布局 = `assets/data/main_map_v6_layout.json`（`compose_main_map_v6.py` 导出，数据驱动）。
/// 元素路径 = `chrome::V6_*` 单源常量。
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
fn render_v6_elem(
    host: &mut dyn SceneHost,
    _layout: &SceneLayout,
    entity_texts: &mut Vec<(i32, i32, &str)>,
) {
    let spaces = host.board_spaces();
    let count = spaces.len();
    if count < 2 {
        return;
    }
    let v6 = v6_layout();
    // 1) 海洋全景底（羊皮框由 ocean_bg 自身带；直接满屏）
    host.blit_image(chrome::V6_OCEAN_BG, 0, 0, 1080, 1920);
    // 2) 羊皮做旧边框（56px 木框，四边）
    let ink = 0x1E14; // 深木色 ≈(60,40,20)
    for w in [8i32, 3] {
        host.fill_rect(0, 0, 1080, w, ink);
        host.fill_rect(0, 1920 - w, 1080, w, ink);
        host.fill_rect(0, 0, w, 1920, ink);
        host.fill_rect(1080 - w, 0, w, 1920, ink);
    }

    // 3) 中央岛（小元素拆分组合：不规则岛体 + 灯塔 + 岸边宝箱，替代整图 V6_ISLAND 圆盘观感）
    let iw = v6.island.w.max(1.0) as i32;
    let icx = v6.island.cx as i32;
    let icy = v6.island.cy as i32;
    // 岛体（不规则海岸线 + 烘焙周围海水，与 ocean_bg 自然拼合）
    if chrome::asset_present(chrome::V6_ISLE_LAND) {
        host.blit_image(chrome::V6_ISLE_LAND, icx - iw / 2, icy - iw / 2, iw, iw);
    }
    // 灯塔（岛顶偏右，约 45% 岛宽）
    if chrome::asset_present(chrome::V6_ISLE_LIGHTHOUSE) {
        let lw = iw * 45 / 100;
        host.blit_image(chrome::V6_ISLE_LIGHTHOUSE, icx + iw / 6, icy - iw / 2 + iw / 12, lw, lw);
    }
    // 岸边宝箱金光（岛下缘，约 30% 岛宽）
    if chrome::asset_present(chrome::V6_ISLE_TREASURE) {
        let tw = iw * 30 / 100;
        host.blit_image(chrome::V6_ISLE_TREASURE, icx - iw / 8, icy + iw / 3, tw, tw);
    }
    // 村落（岛岸 settlement：木屋 + 码头；补全岛内容 山/棕榈/沙滩/灯塔/村落/宝箱金光）
    if chrome::asset_present(chrome::V6_ISLE_VILLAGE) {
        let vw = iw * 34 / 100;
        host.blit_image(
            chrome::V6_ISLE_VILLAGE,
            icx + iw * 30 / 100,
            icy - iw / 2 + iw * 55 / 100,
            vw,
            vw,
        );
    }
    // 整图版 V6_ISLAND 退役（小元素组合后不再使用；保留常量供回退）
    let _ = chrome::V6_ISLAND;

    // 4) 31 格紧围岛不规则排列（布局 JSON 数据驱动；不硬编码坐标）
    let tile_r = v6.tile_r.max(1.0) as i32;
    let size = tile_r * 2;
    // 4a) 缆绳路径（沿格子顺序，先画压在格下；不规则连线 = 前进路线）
    for i in 0..v6.tiles.len().max(1) {
        let a = &v6.tiles[i];
        let b = &v6.tiles[(i + 1) % v6.tiles.len().max(1)];
        rope_segment(host, a.cx as i32, a.cy as i32, b.cx as i32, b.cy as i32, 13, 0x4A23); // 深棕 RGB(74,52,28)
        rope_segment(host, a.cx as i32, a.cy as i32, b.cx as i32, b.cy as i32, 8, 0x6245); // 高光 RGB(102,72,40)
    }
    for t in &v6.tiles {
        let path = if t.owned { chrome::V6_TILE_OWNED } else { chrome::V6_TILE_DENY };
        host.blit_image(path, t.cx as i32 - tile_r, t.cy as i32 - tile_r, size, size);
    }

    // 5) 多席棋子（v3：遍历 `seat_positions()`；当前席 = `V6_SHIP` 贴格，
    // 其余席 = 暗灰小旗；S34.6 玩家船站落格格 = 格径适配，不盖相邻格）。
    let turn_seat = host.turn_seat();
    let cur_idx = host.board_index().min(count - 1);
    let sw_full = (v6.tile_r * 2.0).max(1.0) as i32;
    // v3：同格多席扇出（开局全员同起点出发 ⇒ 不扇出 = 5 枚棋子叠成一枚看不见）。
    for (seat, idx, ord, total) in seat_slots(&host.seat_positions(), count) {
        let (pcx, pcy) = v6.tiles.get(idx).map_or((v6.ship.cx, v6.ship.cy), |t| (t.cx, t.cy));
        let (ox, oy) = seat_piece_offset(ord, total, sw_full * 2 / 5);
        let (ix, iy) = (pcx as i32 + ox, pcy as i32 + oy);
        let sw = if total > 1 { sw_full * 7 / 10 } else { sw_full };
        let badge = SEAT_BADGES[usize::from(seat).min(SEAT_BADGES.len() - 1)];
        if seat == turn_seat {
            host.blit_image(chrome::V6_SHIP, ix - sw / 2, iy - sw / 2, sw, sw);
        } else {
            let sw_small = sw * 5 / 10;
            chrome::entity_icon(
                host,
                "flag",
                ix - sw_small / 2,
                iy - sw_small / 2,
                sw_small,
                0x7BEF,
            );
        }
        entity_texts.push((ix - sw / 2, iy + sw / 2, badge));
    }
    // 当前席名牌（与 S34.6 一致，画在落格格旁）
    {
        let (pcx, pcy) = v6.tiles.get(cur_idx).map_or((v6.ship.cx, v6.ship.cy), |t| (t.cx, t.cy));
        let lbl = i18n::sget("scene.main_map.player_label");
        entity_texts.push((pcx as i32 + sw_full / 2, pcy as i32 - 8, lbl));
    }

    // 6) 舵轮 CTA + 罗盘（底部）
    let hw = v6.helm.w.max(1.0) as i32;
    host.blit_image(
        chrome::V6_HELM,
        v6.helm.cx as i32 - hw / 2,
        v6.helm.cy as i32 - hw / 2,
        hw,
        hw,
    );
    let cw = v6.compass.w.max(1.0) as i32;
    host.blit_image(
        chrome::V6_COMPASS,
        v6.compass.cx as i32 - cw / 2,
        v6.compass.cy as i32 - cw / 2,
        cw,
        cw,
    );

    // 7) 海洋生物（creatures.png = 单张合成海洋生态场景；A4 单处放中央岛下方海域
    // （x340-720/y1350-1600，不叠格环/CTA）+ A5 bob 微动（鲨鱼游弋/浪花翻涌，位移±4px）。
    let ms = CREATURES_ACC.fetch_add(host.frame_delta_ms(), std::sync::atomic::Ordering::Relaxed);
    let t0 = *CREATURES_T0.get_or_init(|| {
        std::env::var("TI_MAP_ANIM_T0").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0)
    });
    let t = f64::from(ms.saturating_add(t0)) / 1000.0;
    let bob_x = (t.sin() * 4.0) as i32;
    let bob_y = ((t * 0.7).sin() * 3.0) as i32;
    host.blit_image(chrome::V6_CREATURES, 340 + bob_x, 1350 + bob_y, 380, 250);

    // 8) 玩家位置名牌（当前格；`cur_idx` = 当前席 index）
    let (px, py) = v6.tiles.get(cur_idx).map_or((540, 890), |t| (t.cx as i32, t.cy as i32));
    let lbl = i18n::sget("scene.main_map.player_label");
    entity_texts.push((px + 40, py - 6, lbl));
    // S36.3 R2-F2：v6 默认路径补终局面板 + 交接提示（R1 漏 v6_elem 主渲染路径）。
    render_match_outcome(host);
    render_handoff(host);
}

/// v6 名牌文字**真截断**（逐字测宽，超 `max_w` 截断 + "…"；`draw_hint_text` 画全宽无截断，
/// clamp 只居中无效 ⇒ 用户 2026-10-31 自审：白字横跨两格是「屎」主因）。
fn clip_text(host: &dyn SceneHost, text: &str, max_w: i32) -> String {
    if host.shell().hint_text_width_px(text) <= max_w {
        return text.to_owned();
    }
    let mut acc = String::new();
    for ch in text.chars() {
        let candidate = format!("{acc}{ch}");
        if host.shell().hint_text_width_px(&candidate) > max_w {
            let mut ell = format!("{acc}…");
            while host.shell().hint_text_width_px(&ell) > max_w && !acc.is_empty() {
                acc.pop();
                ell = format!("{acc}…");
            }
            return ell;
        }
        acc.push(ch);
    }
    acc
}

/// v6 岛型 id → 扁平图标名（`ui_icons` 注册表；识别用，非照片）。
fn v6_isle_icon(isle: &str) -> &'static str {
    if isle.contains("light") {
        "beacon"
    } else if isle.contains("fort") || isle.contains("boss") {
        "castle"
    } else if isle.contains("harbour") || isle.contains("port") {
        "anchor"
    } else if isle.contains("lair") {
        "flag"
    } else if isle.contains("ship") || isle.contains("yard") {
        "gear"
    } else if isle.contains("skull") {
        "monogram"
    } else if isle.contains("trade") || isle.contains("post") {
        "coin"
    } else if isle.contains("volcano") {
        "gear"
    } else if isle.contains("wreck") {
        "chest"
    } else {
        "map"
    }
}

/// v6 功能格顶部色带（类型语义色；领地格用 `isle_token_color`）。
fn v6_func_band_color(theme: &ti_render::theme::Theme, kind: SpaceKind, gold_warm: u16) -> u16 {
    match kind {
        SpaceKind::Start => theme.by_name("gold_warm").unwrap_or(gold_warm),
        SpaceKind::Port => theme.by_name("sea_shallow").unwrap_or(gold_warm),
        SpaceKind::City => theme.by_name("brass").unwrap_or(gold_warm),
        SpaceKind::Event => theme.by_name("brass_dark").unwrap_or(gold_warm),
        SpaceKind::Card => theme.by_name("isle_sand").unwrap_or(gold_warm),
        SpaceKind::Battle => theme.by_name("isle_volcano").unwrap_or(gold_warm),
        SpaceKind::Treasure => theme.by_name("isle_treasure").unwrap_or(gold_warm),
        SpaceKind::Boss => theme.by_name("isle_boss").unwrap_or(gold_warm),
        _ => theme.by_name("land_sand_dark").unwrap_or(gold_warm),
    }
}

/// v6 实心圆（逐行填充模拟；无多边形 API 的代码圆牌）。
fn fill_circle(host: &mut dyn SceneHost, cx: i32, cy: i32, radius: i32, color: u16) {
    let mut dy = -radius;
    while dy <= radius {
        #[allow(clippy::cast_possible_truncation)]
        let hw = (f64::from(radius * radius - dy * dy).max(0.0)).sqrt() as i32;
        host.fill_rect(cx - hw, cy + dy, hw * 2, 1, color);
        dy += 1;
    }
}

/// v6 实心凸多边形（scanline 逐行填充；顶点有序）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn fill_polygon(host: &mut dyn SceneHost, pts: &[(i32, i32)], color: u16) {
    if pts.len() < 3 {
        return;
    }
    let ymin = pts.iter().map(|p| p.1).min().unwrap_or(0);
    let ymax = pts.iter().map(|p| p.1).max().unwrap_or(0);
    for y in ymin..=ymax {
        let mut xs: Vec<i32> = Vec::new();
        for i in 0..pts.len() {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % pts.len()];
            if (y1 <= y && y < y2) || (y2 <= y && y < y1) {
                let denom = f64::from(y2 - y1);
                if denom.abs() < 1.0 {
                    continue;
                }
                let t = f64::from(y - y1) / denom;
                let x = f64::from(x1) + t * f64::from(x2 - x1);
                xs.push(x.round() as i32);
            }
        }
        xs.sort_unstable();
        if xs.len() % 2 != 0 {
            xs.pop();
        }
        let mut j = 0;
        while j + 1 < xs.len() {
            let x0 = xs[j].max(0);
            let x1 = xs[j + 1].min(1079);
            if x1 > x0 {
                host.fill_rect(x0, y, x1 - x0, 1, color);
            }
            j += 2;
        }
    }
}

/// v6 第 `index` 个格的不规则多边形（**真不规则**：6 顶点、每顶点半径 seed 随机 0.72~0.99 ⇒
/// 每格形状差异大，非标准拼图块重复；用户 2026-10-31「你Tm知道什么叫不规则不」= 要真随机）。
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn v6_tile_polygon(index: usize, cx: i32, cy: i32, base_r: i32) -> Vec<(i32, i32)> {
    let mut seed: u32 =
        u32::try_from(index).unwrap_or(0).wrapping_mul(0x9E37_79B9).wrapping_add(0x1234_5678);
    let n = 6usize;
    let mut pts = Vec::with_capacity(n);
    for k in 0..n {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let ang = -90.0
            + f64::from(u32::try_from(k).unwrap_or(0)) * 360.0
                / f64::from(u32::try_from(n).unwrap_or(1));
        let rr = f64::from(base_r) * (0.72 + f64::from(seed % 28) / 100.0);
        let rad = ang * std::f64::consts::PI / 180.0;
        pts.push((cx + (rr * rad.cos()).round() as i32, cy + (rr * rad.sin()).round() as i32));
    }
    pts
}

/// v6 平面棋盘格绘制（**大富翁式平面格 + 随机拼凑感**：平坦顶面 + 短侧壁矮立体 +
/// 底部投影；每格形状随机差异——角切/凹槽/凸舌拼图块；格间海水间隔 = 海洋底透出；
/// 下缘归属色带；功能格 = `node_` 铜牌）。
#[allow(clippy::too_many_arguments)]
#[allow(clippy::used_underscore_binding)]
fn draw_v6_tile(
    host: &mut dyn SceneHost,
    space: &TrackSpace,
    index: usize,
    count: usize,
    theme: &ti_render::theme::Theme,
    brass: u16,
    gold_warm: u16,
    sand_light: u16,
    _sand_dark: u16,
    _sea: u16,
    shadow: u16,
) {
    let (cx, cy) = v6_pillar_center(index, count);
    let d = 68;
    let r = d / 2;
    let tile_face = theme.by_name("land_sand").unwrap_or(sand_light);

    // 2) 语义色带环（领地 = 归属色 / 功能 = 类型色）——色彩是主要识别
    let band = if space.kind == SpaceKind::Territory {
        space
            .isle
            .as_deref()
            .map_or(gold_warm, |i| theme.by_name(&isle_token_color(i)).unwrap_or(gold_warm))
    } else {
        v6_func_band_color(theme, space.kind, gold_warm)
    };

    // 1) **真不规则多边形格**（seed 随机 6 顶点，每格形状差异大；用户「不规则」= 真随机）
    //    投影 + 主体（羊皮卡面）+ 色带圆环 + 中央扁平图标。
    let poly = v6_tile_polygon(index, cx, cy + 3, r);
    for p in &poly {
        let _ = p;
    }
    // 投影（下移 3px 深色）
    let shadow_poly: Vec<(i32, i32)> = poly.iter().map(|(px, py)| (*px, py + 3)).collect();
    fill_polygon(host, &shadow_poly, shadow);
    fill_polygon(host, &poly, tile_face);

    // 色带圆环（语义色）
    fill_circle(host, cx, cy, r * 66 / 100, band);
    fill_circle(host, cx, cy, r * 50 / 100, tile_face);

    // 3) 中央扁平图标（识别）：领地 = 岛型图标（tint 色带色）；功能 = node_* 铜牌
    if space.kind == SpaceKind::Territory {
        if let Some(isle) = &space.isle {
            let ic = d * 40 / 100;
            let icon = v6_isle_icon(isle);
            chrome::entity_icon(host, icon, cx - ic / 2, cy - ic / 2, ic, band);
        }
    } else {
        let def = chrome::node_asset(space.kind);
        let medallion_edge = d * 46 / 100;
        let (mx, my, mw, mh) = chrome::asset_dest_rect(cx, cy, medallion_edge, def.visible_px);
        if chrome::asset_present(def.path) {
            host.blit_image(def.path, mx, my, mw, mh);
        } else {
            chrome::entity_icon(
                host,
                "map",
                cx - medallion_edge / 2,
                cy - medallion_edge / 2,
                medallion_edge,
                gold_warm,
            );
        }
    }

    // 4) 名牌（节点下小字，真截断不溢出）
    let name = if space.kind == SpaceKind::Territory {
        space.isle.as_deref().map_or_else(Default::default, chrome::isle_name)
    } else {
        chrome::space_name(space.kind)
    };
    let name = clip_text(host, &name, 88);
    let nw = host.shell().hint_text_width_px(&name);
    host.draw_hint_text(&name, cx - nw / 2, cy + r + 8, chrome::COLOR_TEXT_ON_PARCH);

    // 5) 驻军签（已占领领地格，节点下）
    if space.kind == SpaceKind::Territory {
        let idx_u = u32::try_from(index).unwrap_or(u32::MAX);
        if let Some(own) = host.territory_ownership(idx_u) {
            if own.owner.is_some() {
                let g_txt = i18n::sgetf("scene.main_map.node.garrison_fmt", &[&own.garrison]);
                let panel_w = (host.shell().hint_text_width_px(&g_txt) + 16).min(88);
                host.fill_rect(cx - panel_w / 2, cy + r + 26, panel_w, 16, shadow);
                host.draw_hint_text(
                    &g_txt,
                    cx - panel_w / 2 + 8,
                    cy + r + 28,
                    chrome::COLOR_TEXT_ON_PARCH,
                );
            }
        }
    }
    let _ = (_sea, brass);
}

/// v6 弹出骰子（不常驻底部）：屏幕中上弹出大骰 + 金环 + 摇骰动作提示（原型静止帧）。
fn draw_v6_spin_popup(
    host: &mut dyn SceneHost,
    theme: &ti_render::theme::Theme,
    gold_warm: u16,
    brass: u16,
) {
    // 大富翁式：骰子掷在棋盘**中央大区域**（环内偏上，岛上缘海面；用户 2026-10-31 点破
    // 「骰子是在中心大区域」——非顶部小弹窗）。
    let (px, py) = (540, 470);
    let edge = 170;
    // 3 帧滚动残影（向左上偏移的弱化骰影 = 摇骰动作）
    let ghost = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_BRASS_DARK);
    if chrome::asset_present(chrome::DICE_ASSET) {
        host.blit_image(chrome::DICE_ASSET, px - edge / 2 - 26, py - edge / 2 - 14, edge, edge);
        host.fill_rect(px - edge / 2 - 26, py - edge / 2 - 14, edge, edge, ghost);
        host.blit_image(chrome::DICE_ASSET, px - edge / 2, py - edge / 2, edge, edge);
    }
    // 金环描边（主骰）
    host.fill_rect(px - edge / 2 - 6, py - edge / 2 - 6, edge + 12, 4, brass);
    host.fill_rect(px - edge / 2 - 6, py + edge / 2 + 2, edge + 12, 4, brass);
    host.fill_rect(px - edge / 2 - 6, py - edge / 2 - 6, 4, edge + 12, brass);
    host.fill_rect(px + edge / 2 + 2, py - edge / 2 - 6, 4, edge + 12, brass);
    // 提示（轮到你 + 点击摇骰 + 语音图标）
    let hint = i18n::sget("scene.main_map.hud.spin");
    let hw = host.shell().hint_text_width_px(hint);
    host.draw_hint_text(hint, px - hw / 2, py + edge / 2 + 14, gold_warm);
    let hint2 = i18n::sget("scene.main_map.nav_hint");
    let hw2 = host.shell().hint_text_width_px(hint2);
    host.draw_hint_text(hint2, px - hw2 / 2, py + edge / 2 + 40, chrome::COLOR_TEXT_ON_PARCH);
}

/// S27.5 摇骰 CTA 轴心骰点（`dice.png`，1080 游戏世界层）。
///
/// 摇骰按钮本体贴图 = `scenes.json` 原语（`deck_spin` 区域），这里只补轴心上的
/// 骰点（用户 2026-10-28 报「没有筛子」= CTA 只有空舵轮，看不见骰子，不知能否摇）。
///
/// 资产来源 = `assets/ui/dice.png`（`ui_manifest.json` `dice` role=board，S21.1
/// 入仓但此前零消费）。缺图 = 不画（不抛异常）。
fn render_spin_die(host: &mut dyn SceneHost, layout: &SceneLayout) {
    let Some(rc) = layout.extra.get(chrome::keys::DECK_SPIN) else {
        return;
    };
    if !chrome::asset_present(chrome::DICE_ASSET) {
        return;
    }
    let edge = rc.w * chrome::DICE_EDGE_RATIO / 100;
    let dx = rc.x + (rc.w - edge) / 2;
    let dy = rc.y + (rc.h - edge) / 2;
    host.blit_image(chrome::DICE_ASSET, dx, dy, edge, edge);
}

/// S34 用户 2026-11-01 复验「没有下一步（摇骰 CTA/骰点）」：v6 舵轮 CTA = `helm.png` 资产
/// （`main_map_v6_layout.json` `helm`，非 scenes.json `deck_spin` 原语位——后者是 v5 残留，
/// 位 (408,1336) 与 v6 舵轮 (291,1740) 不符，若复用 `render_spin_die` 骰点会漂到空海面）。
///
/// 本函数用 v6 布局 helm 定位：①骰点叠舵轮轴心（用户「看不见骰子，不知能否摇」）②最近骰数
/// 在舵轮上方（`board_last_roll` = 0 不画）。缺骰图 = 只画骰数不画骰点（不抛异常）。
#[allow(clippy::cast_possible_truncation)]
fn render_v6_spin_die(host: &mut dyn SceneHost) {
    let v6 = v6_layout();
    let hw = v6.helm.w.max(1.0) as i32;
    let hx = v6.helm.cx as i32;
    let hy = v6.helm.cy as i32;
    // 骰点（舵轮轴心；边长 = 舵轮边长的 DICE_EDGE_RATIO 百分比，与 v5 `render_spin_die` 同口径）
    if chrome::asset_present(chrome::DICE_ASSET) {
        let edge = hw * chrome::DICE_EDGE_RATIO / 100;
        host.blit_image(chrome::DICE_ASSET, hx - edge / 2, hy - edge / 2, edge, edge);
    }
    // 最近骰数（舵轮上方；0 = 尚未摇，不画）
    let roll = host.board_last_roll();
    if roll > 0 {
        let roll_str = chrome::roll_label(roll);
        let rx = hx - host.shell().hint_text_width_px(&roll_str) / 2;
        let ry = hy - hw / 2 - 30;
        let theme = host.theme();
        let gold_warm = theme.by_name("gold_warm").unwrap_or(chrome::COLOR_GOLD_WARM);
        host.draw_hint_text(&roll_str, rx, ry, gold_warm);
    }
}

/// S27.5 v5 单卡绘制：领地格 = `isle_<id>.png` 整幅卡面（**径向旋转**）；功能格 = 旋转卡底板
/// （`card_panel` 白蒙版 × `panel_wood`）+ 旋转 `node_` 铜牌。
///
/// S27.5 二轮（概念稿变体 B 对齐）：卡面按方位角旋转（[`chrome::ring_card_rotation_mdeg`]），
/// 卡底缘叠**归属色带**（`card_panel` 蒙版 × `isle_*` 令牌，随卡旋转）——色带用蒙版 blit 而非
/// `fill_rect`，因为矩形填充不支持旋转。
fn draw_ring_card(
    host: &mut dyn SceneHost,
    space: &TrackSpace,
    index: usize,
    count: usize,
    layout: &SceneLayout,
    theme: &ti_render::theme::Theme,
    brass: u16,
) {
    let (x, y, w, h) = chrome::ring_card_rect(index, count, layout);
    let rot = chrome::ring_card_rotation_mdeg(index, count);
    if space.kind == SpaceKind::Territory {
        if let Some(isle) = &space.isle {
            let path = chrome::isle_card_path(isle);
            if chrome::asset_present(&path) {
                host.blit_image_rot(&path, x, y, w, h, 0xFFFF, rot);
                // 归属色带（卡面下缘素羊皮留白区；高 = 卡高 1/4，左右各内缩 8%）。
                // 色带心 = **卡心沿卡的局部 +y 方向平移**（局部偏移须随卡旋转到屏幕坐标，
                // 否则旋转卡上的色带会「漂」到卡外 —— 按 (x,y,w,h) 直算矩形是错的）。
                let (band_screen_x, band_screen_y) =
                    chrome::rotate_local_offset(x + w / 2, y + h / 2, 0, h * 225 / 1000, rot);
                let bw = w * 84 / 100;
                let bh = h * 25 / 100;
                let band = theme.by_name(&isle_token_color(isle)).unwrap_or(theme.parch_light);
                if chrome::asset_present(chrome::CARD_PANEL_ASSET) {
                    host.blit_image_rot(
                        chrome::CARD_PANEL_ASSET,
                        band_screen_x - bw / 2,
                        band_screen_y - bh / 2,
                        bw,
                        bh,
                        band,
                        rot,
                    );
                }
                // S33.5 驻军显示：已占领格（`territory_ownership` 有 owner）在色带下挂
                // 「驻军 N」小签（`brass_dark` 底 + 黄铜边 + 金字，水平文字可读）。
                let idx = u32::try_from(index).unwrap_or(u32::MAX);
                if let Some(own) = host.territory_ownership(idx) {
                    if own.owner.is_some() {
                        let g_txt =
                            i18n::sgetf("scene.main_map.node.garrison_fmt", &[&own.garrison]);
                        let panel_w = (host.shell().hint_text_width_px(&g_txt) + 20).min(bw);
                        let panel_h = 28;
                        // 挂色带下缘（卡心沿局部 +y 平移卡高×0.42）
                        let (gx, gy) = chrome::rotate_local_offset(
                            x + w / 2,
                            y + h / 2,
                            0,
                            h * 420 / 1000,
                            rot,
                        );
                        let panel = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_GOLD_WARM);
                        host.fill_rect(gx - panel_w / 2, gy - panel_h / 2, panel_w, panel_h, panel);
                        host.fill_rect(gx - panel_w / 2, gy - panel_h / 2, panel_w, 2, brass);
                        host.fill_rect(gx - panel_w / 2, gy + panel_h / 2 - 2, panel_w, 2, brass);
                        host.draw_hint_text(
                            &g_txt,
                            gx - panel_w / 2 + 10,
                            gy - (host.shell().hint_glyph_px()) / 2,
                            chrome::COLOR_TEXT_ON_PARCH,
                        );
                    }
                }
                return;
            }
        }
    }
    // 功能格 / 领地缺图回退：旋转卡底板（黄铜描边层 = 外扩 4px 同蒙版 × brass）
    draw_rot_card_base(host, x, y, w, h, rot, theme, brass);
    let def = chrome::node_asset(space.kind);
    let (cx, cy) = chrome::ring_card_center(index, count, layout);
    let medallion_edge: i32 = w * 62 / 100;
    let (mx, my, mw, mh) = chrome::asset_dest_rect(cx, cy, medallion_edge, def.visible_px);
    if chrome::asset_present(def.path) {
        host.blit_image_rot(def.path, mx, my, mw, mh, 0xFFFF, rot);
    }
}

/// S27.5：旋转卡底板（`card_panel` 白蒙版三层：黄铜外框 / 暖棕卡面 / 顶部高光内描边）。
#[allow(clippy::too_many_arguments)]
fn draw_rot_card_base(
    host: &mut dyn SceneHost,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    rot_mdeg: i32,
    theme: &ti_render::theme::Theme,
    brass: u16,
) {
    if !chrome::asset_present(chrome::CARD_PANEL_ASSET) {
        return;
    }
    // 外扩 6px 的黄铜底 ⇒ 只露出 3px 边框（蒙版自身圆角让边框自然收角）
    host.blit_image_rot(chrome::CARD_PANEL_ASSET, x - 6, y - 6, w + 12, h + 12, brass, rot_mdeg);
    let wood = theme.by_name("panel_wood").unwrap_or(chrome::COLOR_PANEL_WARM);
    host.blit_image_rot(chrome::CARD_PANEL_ASSET, x, y, w, h, wood, rot_mdeg);
}

/// S27.5：当前格**岛名牌**（未旋转水平文字，挂在卡内侧；卡旋转不影响可读性）。
///
/// 用户 2026-10-29「各类岛屿都不知道」⇒ 玩家所在格必须有名字；牌心 = 卡心沿环心方向内移
/// `卡高 × 0.85 + 24`，宽按字自适应（最小 160），`brass_dark` 底 + 黄铜描边 + 金字。
#[allow(clippy::too_many_arguments)]
fn draw_card_name_plate(
    host: &mut dyn SceneHost,
    space: &TrackSpace,
    plate_center_x: i32,
    plate_center_y: i32,
    layout: &SceneLayout,
    theme: &ti_render::theme::Theme,
    brass: u16,
    text_color: u16,
) {
    let name = match space.isle.as_deref() {
        Some(isle) => chrome::isle_name(isle),
        None => chrome::space_name(space.kind),
    };
    let tw = host.shell().hint_text_width_px(&name);
    let panel_w = (tw + 48).max(160);
    let panel_h = 56;
    let g = chrome::ring_geom(layout);
    // 卡心 → 环心方向的单位向量（卡在环外，内移即朝环心）
    let dx = f64::from(g.cx - plate_center_x);
    let dy = f64::from(g.cy - plate_center_y);
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    // 牌贴卡内缘（卡半高 + 8 缝 + 牌半高）⇒ 视觉上「挂」在卡上，不与卡分离
    let offset = f64::from(g.h) * 0.5 + 8.0 + f64::from(panel_h) * 0.5;
    let px = plate_center_x + round_to_i32(dx / len * offset) - panel_w / 2;
    let py = plate_center_y + round_to_i32(dy / len * offset) - panel_h / 2;
    let panel = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_GOLD_WARM);
    host.fill_rect(px, py, panel_w, panel_h, panel);
    host.fill_rect(px, py, panel_w, 2, brass);
    host.fill_rect(px, py + panel_h - 2, panel_w, 2, brass);
    host.fill_rect(px, py, 2, panel_h, brass);
    host.fill_rect(px + panel_w - 2, py, 2, panel_h, brass);
    host.draw_hint_text(&name, px + 24, py + (panel_h - 28) / 2, text_color);
}

/// f64 → i32（四舍五入；环几何换算用，越界钳制后转换）。
#[allow(clippy::cast_possible_truncation)]
fn round_to_i32(value: f64) -> i32 {
    value.round().clamp(-21_474_836.0, 21_474_836.0) as i32
}

/// S27.5：`isle_id` → 归属色带色令牌（prototype 序 = `balance.territory.isle_tokens` 序，
/// `S27.2` 资产按同名生成；S28 归属覆盖后此令牌仍为「无主默认」卡底/色带色）。
#[allow(clippy::match_same_arms)]
fn isle_token_color(isle: &str) -> String {
    let key = match isle {
        "harbour" => "isle_coral",
        "lighthouse" => "isle_sand",
        "wreck" => "isle_gold",
        "pirate_lair" => "isle_dark",
        "shipyard" => "isle_teal",
        "trading_post" => "isle_olive",
        "skull_cliff" => "isle_bone",
        "volcano" => "isle_ember",
        "lightship" => "isle_mist",
        "boss_fort" => "isle_black",
        _ => "isle_sand",
    };
    key.to_owned()
}

/// 落格提示签条（`brass_dark` 底板 + 黄铜边框，S27.4 v5 位置）：宽按字自适应（最小 392）；
/// 中心 = `hint` 键（v5 = (540,1760)，三键之下；`layout.hint.y ≤ 0` = 回退 1760）。
fn render_board_toast(host: &mut dyn SceneHost) {
    let Some(toast) = host.board_toast() else {
        return;
    };
    let layout = chrome::scene_layout("main_map");
    let ty = if layout.hint.y > 0 { layout.hint.y } else { 1760 };
    let tw = host.shell().hint_text_width_px(&toast);
    let pad = 24;
    let panel_w = (tw + pad * 2).max(392);
    let panel_h = 64;
    let px = 540 - panel_w / 2;
    let py = ty - panel_h / 2;
    let theme = host.theme().clone();
    // 底板用 `brass_dark`（`panel_warm` 在深海底上≈隐形 —— RGB565 (24,24,16) 与底图同色）
    let panel = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
    host.fill_rect(px, py, panel_w, panel_h, panel);
    host.fill_rect(px, py, panel_w, 2, brass);
    host.fill_rect(px, py + panel_h - 2, panel_w, 2, brass);
    host.fill_rect(px, py, 2, panel_h, brass);
    host.fill_rect(px + panel_w - 2, py, 2, panel_h, brass);
    let tx = 540 - tw / 2;
    host.draw_hint_text(&toast, tx, ty, chrome::COLOR_TEXT_ON_PARCH);
}

/// S36.3 终局面板（`territory.md` §4.4 / `territory_v2::victory_seat_active`）：
/// `match_outcome` 在位 = 对局结束，主地图中央画羊皮面板 + 胜负文案 + 「按 Confirm 返回」。
///
/// - `Victory { seat }`：`"{0} 获胜！"`（`scene.main_map.match.victory_fmt`，i18n 槽位 0 = 席位名）；
/// - `Defeat`：`"全部人类席破产出局——失败"`（`scene.main_map.match.defeat`）；
/// - `Draw`：`"多人同时破产——平局"`（`scene.main_map.match.draw`）。
fn render_match_outcome(host: &mut dyn SceneHost) {
    use crate::game_app::MatchOutcome;

    let Some(outcome) = host.match_outcome() else {
        return;
    };
    let theme = host.theme().clone();
    let panel = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
    let gold = theme.by_name("gold_warm").unwrap_or(chrome::COLOR_GOLD_WARM);
    let ink = chrome::COLOR_TEXT_ON_PARCH;
    let pw = 880i32;
    let ph = 320i32;
    let px = 540 - pw / 2;
    let py = 780;
    host.fill_rect(px, py, pw, ph, panel);
    host.fill_rect(px, py, pw, 3, brass);
    host.fill_rect(px, py + ph - 3, pw, 3, brass);
    host.fill_rect(px, py, 3, ph, brass);
    host.fill_rect(px + pw - 3, py, 3, ph, brass);
    let (title, sub) = match outcome {
        MatchOutcome::Victory { seat } => {
            // 席位名：`P{n}`（1-based 序号）；缺 `balance` 时回退 `P{n}`（不 panic）。
            let name = format!("P{seat}");
            let s = i18n::sgetf("scene.main_map.match.victory_fmt", &[&name]);
            (s, i18n::sget("scene.main_map.match.continue").to_string())
        }
        MatchOutcome::Defeat => {
            let s = i18n::sget("scene.main_map.match.defeat").to_string();
            (s, i18n::sget("scene.main_map.match.continue").to_string())
        }
        MatchOutcome::Draw => {
            let s = i18n::sget("scene.main_map.match.draw").to_string();
            (s, i18n::sget("scene.main_map.match.continue").to_string())
        }
    };
    let tw = host.shell().hint_text_width_px(&title);
    host.draw_hint_text(&title, 540 - tw / 2, py + 80, gold);
    let sw = host.shell().hint_text_width_px(&sub);
    host.draw_hint_text(&sub, 540 - sw / 2, py + ph - 60, ink);
}

/// S36.3 热座交接提示条（§1 + §1.2 隐藏信息）：`handoff_pending` 在位 =
/// 底部画「热座交接：按 Confirm 确认开始回合」提示条（确认前不画其它玩家信息）。
fn render_handoff(host: &mut dyn SceneHost) {
    let Some(seat) = host.handoff_pending() else {
        return;
    };
    let theme = host.theme().clone();
    let panel = theme.by_name("brass_dark").unwrap_or(chrome::COLOR_GOLD_WARM);
    let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
    let ink = chrome::COLOR_TEXT_ON_PARCH;
    let msg = i18n::sget("scene.main_map.toast.handoff");
    let tw = host.shell().hint_text_width_px(msg);
    let pad = 24;
    let pw = (tw + pad * 2).max(392);
    let ph = 56i32;
    let px = 540 - pw / 2;
    let py = 1860;
    host.fill_rect(px, py, pw, ph, panel);
    host.fill_rect(px, py, pw, 2, brass);
    host.fill_rect(px, py + ph - 2, pw, 2, brass);
    host.fill_rect(px, py, 2, ph, brass);
    host.fill_rect(px + pw - 2, py, 2, ph, brass);
    let seat_label = format!("P{seat}");
    let full = format!("{msg}（{seat_label}）");
    let ftw = host.shell().hint_text_width_px(&full);
    host.draw_hint_text(&full, 540 - ftw / 2, py + ph / 2 + 4, ink);
}
