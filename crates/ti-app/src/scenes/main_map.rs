//! 主地图场景（S9 骨架）：移动 / 贸易 / 建造 / 事件分派 + 等距网格绘制。
//!
//! 事件分派的跳转结果（`Vs` / `Message`）由路由层 [`crate::scenes::enter_target`]
//! 处理——那是跨场景的公共跳转点，不属于本场景。
use ti_core::data;
use ti_core::data::{SceneLayout, balance};
use ti_core::domain::economy::Building;
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::GridPos;
use ti_core::rules::board::SpaceKind;
use ti_core::rules::build;
use ti_core::rules::trade;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_LANDMARK, COLOR_PLAYER};
use crate::scenes::chrome;
use crate::scenes::{Scene, SceneHost, SceneKind, enter_target};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

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
    let gold = host.gold();
    let gold_str = i18n::sgetf("scene.main_map.hud.gold_fmt", &[&gold]);
    let owned = host.owned_total();
    let own_str = i18n::sgetf("scene.main_map.hud.owned_fmt", &[&owned]);
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
    let gold = host.gold();
    let gold_str = i18n::sgetf("scene.main_map.hud.gold_fmt", &[&gold]);
    chrome::draw_body_text(host, &gold_str, layout.hud.x + 4, layout.hud.y + 2);
    let owned = host.owned_total();
    let own_str = i18n::sgetf("scene.main_map.hud.owned_fmt", &[&owned]);
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

impl Scene for MainMapScene {
    fn kind(&self) -> SceneKind {
        SceneKind::MainMap
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
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
            GameInput::Spin => {
                // 环岛赛道（`rules::board`，`progression.md` §环岛赛道）：摇骰定步数 → 前进 N 格 → 落格结算。
                // 落格效果（金/HP/卡/提示条）已由 `board_spin` 结算；S19.3：经过/落在 BOSS 格且
                // 绕满 `boss_min_laps` 圈 → BOSS 现身 → 切战斗场景。
                let Some(turn) = host.board_spin() else {
                    return false;
                };
                if turn.boss.engaged() {
                    host.start_battle();
                    return true;
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

    /// - S15.6 QA149：点 HUD 三按钮（scenes.json `hud_trade`/`hud_sell`/`hud_build`
    ///   矩形）= 买商品 1 / 卖商品 1 / 造 Castle（与键盘 T/Y/B 同语义，0 键盘纯触摸经济闭环）；
    /// - S19：点「摇骰前进」按钮（`deck_spin`）= 摇骰前进；点自己所在格 = 触发据点事件
    ///   （`Landmark(code)`）；点其它格 / 滑动 = 无动作（移动唯一 = 摇骰）。
    ///
    /// 命中坐标与渲染同坐标系：`grid_cell_at` = `chrome::iso_cell` 的逆
    /// （数据源 = `scenes.json` `main_map.grid.*`，EXP-067 正逆成对；legacy 32/16
    /// 投影 + (88,110) 摄像机已退出运行时，S15.6 QA022/023）。
    #[allow(clippy::cast_possible_truncation)] // HUD 臂：1080 呈现坐标 f32 → i32 命中判定
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        let Some(cur) = host.map_pos() else {
            return false;
        };
        match sample.gesture {
            Gesture::Tap => {
                // S15.6 QA149：HUD 三经济臂——先命中 scenes.json 三个 extra 按钮矩形
                // （渲染同一坐标系，契约 4 数据驱动；语义 = 键盘 T/Y/B，game_app.rs poll），
                // 未命中落回下方网格点按逻辑。
                let l = chrome::scene_layout("main_map");
                let px = sample.x as i32;
                let py = sample.y as i32;
                let hud_hit = |key: &str| -> bool {
                    l.extra.get(key).is_some_and(|r| {
                        r.w > 0
                            && r.h > 0
                            && (r.x..(r.x + r.w)).contains(&px)
                            && (r.y..(r.y + r.h)).contains(&py)
                    })
                };
                // S17.8 v3：底部命令盘（deck_trade/sell/build）优先命中；未命中落回
                // v2 HUD 按钮（双写兼容——`e2e_touch_economy_loop` 仍按 v2 坐标命中）。
                // S19：摇骰前进主操作按钮（`deck_spin`）最先命中（主操作优先）。
                if hud_hit(chrome::keys::DECK_SPIN) {
                    return host.advance(GameInput::Spin);
                }
                if hud_hit(chrome::keys::DECK_TRADE) {
                    return host.advance(GameInput::TradeBuy(1));
                }
                if hud_hit(chrome::keys::DECK_SELL) {
                    return host.advance(GameInput::TradeSell(1));
                }
                if hud_hit(chrome::keys::DECK_BUILD) {
                    return host.advance(GameInput::Build(Building::Castle));
                }
                if hud_hit(chrome::keys::HUD_TRADE) {
                    return host.advance(GameInput::TradeBuy(1));
                }
                if hud_hit(chrome::keys::HUD_SELL) {
                    return host.advance(GameInput::TradeSell(1));
                }
                if hud_hit(chrome::keys::HUD_BUILD) {
                    return host.advance(GameInput::Build(Building::Castle));
                }
                let Some(terrain) = host.map_terrain() else {
                    return false;
                };
                let cell = layout::grid_cell_at(sample.x, sample.y, terrain.grid_w, terrain.grid_h);
                let Some((row, col)) = cell else {
                    return false;
                };
                let target = GridPos::new(row, col);
                // P0-5：点自身格 = 走 dispatch(1)→Msgl（据点菜单）；点其它格 = 无动作（S19 收口：移动唯一 = 摇骰）。
                if target == cur {
                    return host.advance(GameInput::Landmark(1));
                }
                false
            }
            // S19 收口：滑动无动作（移动唯一 = 摇骰）。
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
        let bg = chrome::board_fit(&layout, gw);

        // 等距棋盘（代码画，1080 呈现层；S17.8 v3：G 依赖拟合进 `world_canvas`（2:1 菱形、
        // 8pt 对齐、画布中心对齐），缺席回退 v2 静态 `grid.*`（QB105）；`terrain_grid_w`² 铺满）
        // S18.9：**按地形取色**（`assets/data/terrain.json` 数据 → theme 令牌）：
        // 海/礁/陆/宝藏 四类各有底色 + 顶棱高光；陆块按 (r+c) 奇偶交错 `alt` 变体。
        // 地形表缺失 = 回退 v2 单色（不 panic、不空图）。
        let gw_i = i32::try_from(gw).unwrap_or(0);
        let (tw, th) = (bg.tw, bg.th);
        draw_board(host, &layout, gw, gw_i, tw, th);

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
        // S19：环岛赛道（路径连线 + 空间标记 + 骰子数 + 落格提示条）
        render_track(host, &layout, gw, &mut entity_texts);

        // S19.7：航程条（赛道摊平成进度条：玩家位置 + BOSS 目的地 + 特殊格标记）
        chrome::voyage_bar(host, "main_map");

        // 第二遍：实体标签（28px 提示档，QA142 全部文本最上层）
        for (x, y, lbl) in &entity_texts {
            host.draw_hint_text(lbl, *x, *y, chrome::COLOR_TEXT_ON_PARCH);
        }

        // QA032：hint 行统一走 `chrome::scene_hint`（i18n 单查收敛，EXP-082）
        // S15.6：key = `nav_hint`（`hint` 已并入，`strings.json` 删孤儿）
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
    let idx = host.board_index().min(spaces.len() - 1);
    let (cx, cy) = centers[idx];
    let ship_edge = chrome::ship_edge_px(layout, gw);
    let (dx, dy, dw, dh) =
        chrome::asset_dest_rect(cx, cy, ship_edge, chrome::SHIP_PLAYER_VISIBLE_PX);
    if chrome::asset_present(chrome::SHIP_PLAYER_ASSET) {
        host.blit_image(chrome::SHIP_PLAYER_ASSET, dx, dy, dw, dh);
    } else {
        chrome::entity_icon(host, "flag", dx, dy, dw, chrome::COLOR_GOLD_WARM);
    }
    let lbl = i18n::sget("scene.main_map.player_label");
    entity_texts.push((cx + ship_edge / 2, cy - 8, lbl));

    // 5) 骰子数（摇骰按钮上方居中，`board_last_roll` = 0 → 不画）
    let roll = host.board_last_roll();
    if roll > 0 {
        let roll_str = i18n::sgetf("scene.main_map.toast.roll_fmt", &[&roll]);
        let rx = 540 - host.shell().hint_text_width_px(&roll_str) / 2;
        let ry = layout.extra.get(chrome::keys::DECK_SPIN).map_or(1296, |r| r.y - 24);
        host.draw_hint_text(&roll_str, rx, ry, chrome::COLOR_GOLD_WARM);
    }
    // 6) 落格提示签条（S21.6 v4 = `hint` 键中心；`board_toast` 过期 = `None`）
    render_board_toast(host);
}

/// 落格提示签条（S19.7 D3 底板 + S21.6 v4 位置）：`brass_dark` 底板 + 黄铜边框，
/// 宽按字自适应（最小 392）；中心 = `hint` 键（v4 = (540,1632)，spec §2 签条 {344,1600,392,64}）。
fn render_board_toast(host: &mut dyn SceneHost) {
    let Some(toast) = host.board_toast() else {
        return;
    };
    let layout = chrome::scene_layout("main_map");
    let ty = if layout.hint.y > 0 { layout.hint.y } else { 1632 };
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

/// S18.9：等距棋盘绘制 —— **按地形取色**（`assets/data/terrain.json` 数据 → theme 令牌）。
///
/// 地形表缺失 / 越界 = 回退 v2 单色（不 panic、不空图）。陆地按 (r+c) 奇偶用图例 `alt` 交错。
/// S21.4 岛屿重绘：只画陆/礁/宝格（海洋格不画 → 巨型蓝菱形消失）+ 墨线海岸描边 +
/// 礁石浅滩环 + 陆内确定性纹理（`terrain.json` 为唯一数据源；`legend.edge` = 边饰色：
/// reef→`sea_shallow` 浅滩环 / land·treasure→`land_sand_light` 陆内纹理）。
///
/// 可见块口径：等距格底 = `2tw × 2th` 矩形，但**下半被后画的邻块覆盖** ⇒ 每格的可见
/// 面 = 顶半 `2tw × th`；装饰（墨线/浅滩环/纹理）一律画在可见块上，坐标与
/// `chrome::iso_cell_fit` 同源（EXP-067 正逆成对）。
///
/// 可见块的 4 边邻居（共享整边）：上 = `(r-1,c)`/`(r,c-1)`、下 = `(r+1,c)`/`(r,c+1)`、
/// 左 = `(r+1,c-1)`、右 = `(r-1,c+1)`（半砖错位布局）。
const ISLAND_INK_TOKEN: &str = "wood_warm";
const ISLAND_INK_PX: i32 = 1;
const ISLAND_REEF_SHALLOWS_PX: i32 = 3;
const ISLAND_TEXTURE_LEN: i32 = 4;
const ISLAND_TEXTURE_INSET: i32 = 3;
/// 海洋格码（`terrain.json` legend `S`）；不画 = 岛屿剪影。
const ISLAND_SEA_CODE: char = 'S';
/// 礁石格码（legend `R`）。
const ISLAND_REEF_CODE: char = 'R';

/// 确定性格哈希（FNV-1a 变体；**禁 RNG** → 陆内纹理在快照/像素对拍中可复现）。
fn island_cell_hash(r: i32, c: i32) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for b in r.to_le_bytes().into_iter().chain(c.to_le_bytes()) {
        h ^= u32::from(b);
        h = h.wrapping_mul(16_777_619);
    }
    h
}

/// 画岛屿（`terrain.json` 驱动）：底色 → 墨线海岸 → 礁石浅滩环 → 陆内纹理。
/// S21.4 岛屿绘制上下文（地形查询 + 主题色 + 墨线色）。
struct IslandCtx<'a> {
    /// 地形表（缺 = 全海 ⇒ 不画）。
    terrain: Option<&'a data::TerrainDb>,
    /// 当前地图 id（缺 = 全海 ⇒ 不画）。
    map_id: Option<&'a str>,
    /// 主题（色令牌）。
    theme: &'a ti_render::theme::Theme,
    /// 墨线海岸色（`wood_warm` 暖墨棕；缺令牌 = 回退面板底色）。
    ink: u16,
}

impl IslandCtx<'_> {
    /// 地形码（缺表/缺图/越界 = 海）。
    fn code_at(&self, r: i32, c: i32) -> char {
        let (Some(t), Some(mid)) = (self.terrain, self.map_id) else {
            return ISLAND_SEA_CODE;
        };
        let (Ok(rr), Ok(cc)) = (u32::try_from(r), u32::try_from(c)) else {
            return ISLAND_SEA_CODE;
        };
        t.code_at(mid, rr, cc).unwrap_or(ISLAND_SEA_CODE)
    }

    /// 是否海洋格（不画 → 岛屿剪影）。
    fn is_sea(&self, r: i32, c: i32) -> bool {
        self.code_at(r, c) == ISLAND_SEA_CODE
    }

    /// 图例（地形类别 → 令牌）。
    fn legend(&self, code: char) -> Option<&data::TerrainLegend> {
        self.terrain.and_then(|t| t.legend_of(code))
    }

    /// 图例令牌色（缺令牌 = 面板底色回退）。
    fn token_color(&self, token: &str) -> u16 {
        self.theme.by_name(token).unwrap_or(crate::menu_shell::COLOR_MENU_BG)
    }
}

/// 画岛屿（`terrain.json` 驱动）：底色 → 墨线海岸 → 礁石浅滩环 → 陆内纹理。
fn draw_board(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    gw: u32,
    gw_i: i32,
    tw: i32,
    th: i32,
) {
    if gw_i <= 0 || tw <= 0 || th <= 0 {
        return;
    }
    let theme = host.theme().clone();
    let ctx = IslandCtx {
        terrain: data::terrain().ok(),
        map_id: host.map_slot().map(crate::game_app::map_slot_id),
        theme: &theme,
        ink: theme.by_name(ISLAND_INK_TOKEN).unwrap_or(crate::menu_shell::COLOR_MENU_BG),
    };
    draw_island_bases(host, layout, gw, gw_i, tw, th, &ctx);
    draw_island_decor(host, layout, gw, gw_i, tw, th, &ctx);
}

/// pass 1：底色（陆/礁/宝格；海洋格跳过 → 岛屿剪影）。
///
/// v4 岛屿 = **暖砂剪影**（S21.3 §0）：`legend.alt` 的 (r+c) 奇偶棋盘变体退役
/// （v3「现代棋盘感」残留），陆面统一 `token` 色 + 确定性纹理；`alt` 字段保留
/// （不删数据，S21.8 存量豁免清零时统一处置）。
fn draw_island_bases(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    gw: u32,
    gw_i: i32,
    tw: i32,
    th: i32,
    ctx: &IslandCtx<'_>,
) {
    for r in 0..gw_i {
        for c in 0..gw_i {
            let code = ctx.code_at(r, c);
            if code == ISLAND_SEA_CODE {
                continue;
            }
            let (sx, sy) = chrome::iso_cell_fit(r, c, layout, gw);
            let base = ctx
                .legend(code)
                .map_or_else(|| crate::menu_shell::COLOR_MENU_BG, |l| ctx.token_color(&l.token));
            host.fill_rect(sx, sy, tw * 2, th * 2, base);
        }
    }
}

/// pass 2：装饰（墨线海岸 / 礁石浅滩环 / 陆内纹理）——画在可见块（顶半）上，
/// 顶层绘制 ⇒ 不被邻块底色覆盖（海岸线连续）。
fn draw_island_decor(
    host: &mut dyn SceneHost,
    layout: &SceneLayout,
    gw: u32,
    gw_i: i32,
    tw: i32,
    th: i32,
    ctx: &IslandCtx<'_>,
) {
    let bw = tw * 2;
    let bh = th;
    for r in 0..gw_i {
        for c in 0..gw_i {
            let code = ctx.code_at(r, c);
            if code == ISLAND_SEA_CODE {
                continue;
            }
            let (sx, sy) = chrome::iso_cell_fit(r, c, layout, gw);
            let edge_color = ctx.legend(code).map(|l| ctx.token_color(&l.edge));
            let up_sea = ctx.is_sea(r - 1, c) || ctx.is_sea(r, c - 1);
            let down_sea = ctx.is_sea(r + 1, c) || ctx.is_sea(r, c + 1);
            let left_sea = ctx.is_sea(r + 1, c - 1);
            let right_sea = ctx.is_sea(r - 1, c + 1);

            // 墨线海岸描边：**所有非海格**的临海边（岛的外轮廓连续；陆↔礁交界不画线）。
            if up_sea {
                host.fill_rect(sx, sy, bw, ISLAND_INK_PX, ctx.ink);
            }
            if down_sea {
                host.fill_rect(sx, sy + bh - ISLAND_INK_PX, bw, ISLAND_INK_PX, ctx.ink);
            }
            if left_sea {
                host.fill_rect(sx, sy, ISLAND_INK_PX, bh, ctx.ink);
            }
            if right_sea {
                host.fill_rect(sx + bw - ISLAND_INK_PX, sy, ISLAND_INK_PX, bh, ctx.ink);
            }

            if code == ISLAND_REEF_CODE {
                // 礁石浅滩环：向可见块外扩的浅滩带（`legend.edge` = sea_shallow）。
                let shallow = edge_color.unwrap_or(ctx.theme.sea_shallow);
                if up_sea {
                    host.fill_rect(
                        sx,
                        sy - ISLAND_REEF_SHALLOWS_PX,
                        bw,
                        ISLAND_REEF_SHALLOWS_PX,
                        shallow,
                    );
                }
                if down_sea {
                    host.fill_rect(sx, sy + bh, bw, ISLAND_REEF_SHALLOWS_PX, shallow);
                }
                if left_sea {
                    host.fill_rect(
                        sx - ISLAND_REEF_SHALLOWS_PX,
                        sy,
                        ISLAND_REEF_SHALLOWS_PX,
                        bh,
                        shallow,
                    );
                }
                if right_sea {
                    host.fill_rect(sx + bw, sy, ISLAND_REEF_SHALLOWS_PX, bh, shallow);
                }
                continue;
            }

            // 陆内确定性纹理：哈希定点的短划（`legend.edge` = land_sand_light 浅砂颗粒）。
            let Some(tex) = edge_color else { continue };
            let span_x = (bw - ISLAND_TEXTURE_INSET * 2 - ISLAND_TEXTURE_LEN).max(1);
            let span_y = (bh - ISLAND_TEXTURE_INSET * 2).max(1);
            for h in [island_cell_hash(r, c), island_cell_hash(c, r)] {
                let ox = ISLAND_TEXTURE_INSET
                    + i32::try_from(h % u32::try_from(span_x).unwrap_or(1)).unwrap_or(0);
                let oy = ISLAND_TEXTURE_INSET
                    + i32::try_from((h >> 8) % u32::try_from(span_y).unwrap_or(1)).unwrap_or(0);
                host.fill_rect(sx + ox, sy + oy, ISLAND_TEXTURE_LEN, 1, tex);
            }
        }
    }
}
