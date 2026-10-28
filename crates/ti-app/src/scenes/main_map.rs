//! 主地图场景（S9 骨架）：移动 / 贸易 / 建造 / 事件分派 + 等距网格绘制。
//!
//! 事件分派的跳转结果（`Vs` / `Message`）由路由层 [`crate::scenes::enter_target`]
//! 处理——那是跨场景的公共跳转点，不属于本场景。
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
        build::Bank { gold: host.bank_gold(), deposit: 0 },
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

/// S21.12 打开建造面板（选中复位第 0 项；面板渲染/交互由后续帧与点按接管）。
fn open_build_panel(host: &mut dyn SceneHost) {
    host.set_build_panel_sel(0);
    host.set_build_panel_open(true);
    host.play_sfx(Sfx::Press);
}

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

    // 4) 玩家船（当前格；缺资产 = 代码 `flag` 图标回退）
    //    S21.12：船贴图按 `boat_level` 切 4 张（1 级 = 通用船顶替）。
    let idx = host.board_index().min(spaces.len() - 1);
    let (cx, cy) = centers[idx];
    let ship_edge = chrome::ship_edge_px(layout, gw);
    let (dx, dy, dw, dh) =
        chrome::asset_dest_rect(cx, cy, ship_edge, chrome::SHIP_PLAYER_VISIBLE_PX);
    let ship_asset = chrome::boat_asset(host.boat_level());
    if chrome::asset_present(ship_asset) {
        host.blit_image(ship_asset, dx, dy, dw, dh);
    } else if chrome::asset_present(chrome::SHIP_PLAYER_ASSET) {
        host.blit_image(chrome::SHIP_PLAYER_ASSET, dx, dy, dw, dh);
    } else {
        chrome::entity_icon(host, "flag", dx, dy, dw, chrome::COLOR_GOLD_WARM);
    }
    let lbl = i18n::sget("scene.main_map.player_label");
    entity_texts.push((cx + ship_edge / 2, cy - 8, lbl));

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
    if host.board_toast().is_some() && idx < spaces.len() {
        let (nx, ny) = centers[idx];
        let def = chrome::node_asset(spaces[idx].kind);
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
        match hit.key {
            4 => host.advance(GameInput::Spin),
            5 => host.advance(GameInput::TradeBuy(1)),
            6 => host.advance(GameInput::TradeSell(1)),
            7 => {
                open_build_panel(host);
                true
            }
            _ => false,
        }
    }

    /// 等距地形瓦片（真资源）+ 玩家棋子 + 据点 + 金/持有量 HUD（v2 羊皮纸/木纹/金色系）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        // P1-3：坐标/文案硬编码改读 `ti_core::data::scenes()`（scenes.json 字段已建）；
        // QA031：统一走 `chrome::scene_layout`（scenes.json 单一读点，契约 4）
        let layout = chrome::scene_layout("main_map");
        // S17.8 v3：v3 模式判定（world_canvas 键在位 = v3；缺席回退 v2 静态 grid.*）。
        let v3 = chrome::has_world_canvas(&layout);
        // S15.4：legacy 地形瓦片像素层移除。主地图 = 羊皮纸底 + 航海网格装饰 + 代码绘制等距棋盘（§4.3）
        // S18.7f：夜空底 + 画布框 + HUD/命令盘 chrome = JSON 原语（**必须先于瓦片**：
        // 夜空渐变是背景层，瓦片/实体叠其上）；v2 回退 = 羊皮纸底 + 航海网格。
        if v3 {
            chrome::render_primitives(host, "main_map", -1);
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
            render_hud_v3(host, &layout);
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
            render_v5_ring(host, &layout, &mut entity_texts);
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
    for (index, space) in spaces.iter().enumerate() {
        draw_ring_card(host, space, index, count, layout, &theme, brass);
    }

    // 3) 玩家船（落在 board_index 卡；缺资产 = 代码 flag 回退；船边长随卡宽缩放，非硬编码）
    let idx = host.board_index().min(count - 1);
    let (cx, cy) = chrome::ring_card_center(idx, count, layout);
    let ship_edge: i32 = chrome::ring_geom(layout).w * 83 / 100;
    let ship_asset = chrome::boat_asset(host.boat_level());
    if chrome::asset_present(ship_asset) {
        host.blit_image(ship_asset, cx - ship_edge / 2, cy - ship_edge / 2, ship_edge, ship_edge);
    } else {
        chrome::entity_icon(
            host,
            "flag",
            cx - ship_edge / 2,
            cy - ship_edge / 2,
            ship_edge,
            gold_warm,
        );
    }
    let lbl = i18n::sget("scene.main_map.player_label");
    entity_texts.push((cx + ship_edge / 2, cy - 8, lbl));

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
    render_board_toast(host);
}

/// S27.4 v5 单卡绘制：领地格 = `isle_<id>.png` 整幅卡面；功能格 / 缺图 = 羊皮卡底 + `node_` 牌。
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
    if space.kind == SpaceKind::Territory {
        if let Some(isle) = &space.isle {
            let path = chrome::isle_card_path(isle);
            if chrome::asset_present(&path) {
                host.blit_image(&path, x, y, w, h);
                return;
            }
        }
    }
    // 功能格 / 领地缺图回退：羊皮卡底 + node_ 黄铜牌（可见框 ≈ 87% 卡宽，随卡宽缩放）
    draw_feature_card_base(host, x, y, w, h, theme, brass);
    let def = chrome::node_asset(space.kind);
    let (cx, cy) = chrome::ring_card_center(index, count, layout);
    let medallion_edge: i32 = chrome::ring_geom(layout).w * 87 / 100;
    let (mx, my, mw, mh) = chrome::asset_dest_rect(cx, cy, medallion_edge, def.visible_px);
    host.blit_image(def.path, mx, my, mw, mh);
}

/// S27.4 v5 功能卡羊皮卡底（`panel_warm` 暖面板 + `brass` 2px 边）；node_ 牌再叠其上。
fn draw_feature_card_base(
    host: &mut dyn SceneHost,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    theme: &ti_render::theme::Theme,
    brass: u16,
) {
    let base = theme.by_name("panel_warm").unwrap_or(chrome::COLOR_PANEL_WARM);
    host.fill_rect(x, y, width, height, base);
    host.fill_rect(x, y, width, 2, brass);
    host.fill_rect(x, y + height - 2, width, 2, brass);
    host.fill_rect(x, y, 2, height, brass);
    host.fill_rect(x + width - 2, y, 2, height, brass);
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
