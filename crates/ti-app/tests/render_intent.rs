//! 渲染意图门禁（`docs/business/08_layout_design.md` §4 版面意图 + `07_render_pipeline.md` §2 z-order 契约）。
//!
//! 现有门禁的缺口：`render_snapshot`（第 19 项）只测**纯 `.Map` 精灵**渲染（不叠场景层），
//! `visual_regression`（第 18 项）做场景层+精灵叠加但**与自生成基线 diff**（自证循环，基线错则全错），
//! `e2e_render_alignment` 只验证 `.Map` 内部 z-order 契约。三者都不回答「场景意图被满足」：
//! 例如立绘盖住菜单文字区、瓦片区被 `.Map` 精灵污染——**像素没变**（与基线一致）但**意图不满足**。
//!
//! 本测试补「像素应该是这样」的意图断言（不依赖基线，纯像素 + 几何判定）：
//!
//! 1. **类 1 层间遮挡**：场景层文字区（标题条/菜单列表/状态行/瓦片区/血条区）必须
//!    **可见**——精灵层在该区非背景像素密度有上界（立绘不能把文字区完全淹没）。
//!    `MAX_DENSITY` 数据定标：全 7 场景实测最大合法基线 68%（selectmap 地图列表区
//!    11039/16128）+ 22pp 余量 = 90%（详见 `MAX_DENSITY` 注释；全淹没 → 密度趋 100%）。
//! 2. **类 2 版面意图**：scenes.json 坐标 = §4 表值（标题条高度/列表起点/HUD 区间/血条 x 范围），
//!    纯数据断言，文档改了 scenes.json 没跟 → 当场 FAIL。
//! 3. **类 3 元素存在性**：渲染场景层后，§4 关键矩形（标题条/HUD/血条/菜单高亮）
//!    区域内**必须有非背景像素**（防「坐标全 0 时静默不画」）。

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::expect_used,
    clippy::items_after_statements,
    clippy::panic,
    clippy::many_single_char_names,
    clippy::type_complexity,
    clippy::cast_precision_loss,
    clippy::doc_markdown,
    clippy::explicit_iter_loop,
    clippy::cast_sign_loss
)]

// 呈现层画布 = 1080×1920（`render_pipeline::FB_W/FB_H`，契约 1）。
// S15.4：legacy .Map 176×220 像素写帧层移除，代码绘制判据（本文件保留部分）直接按 1080 比对。
const FB_W: usize = 1080;
const FB_W_I32: i32 = 1080;
const FB_H: usize = 1920;

// S15.4：类 1「.Map 精灵层密度/淹没」判据（`text_regions_not_flooded_by_sprites`）依赖已删除的
// CPU 逐像素 blit 路径（`blit_scene_frame`），随 legacy 像素写帧层一并移除。
// 重锚到代码绘制区 = S15.5（`08_render_arch.md` §4 契约 5）。

/// 类 2 版面意图：scenes.json 坐标 = §4 表值。
fn scenes() -> std::collections::BTreeMap<String, ti_core::data::SceneLayout> {
    ti_core::data::scenes().expect("scenes.json 解析失败").clone()
}

/// index §4.1（S22 罗盘环；单一事实源 `09_index_compass_spec.md`）：满屏 chart_full + 6 medallion 沿环。
#[test]
fn index_layout_matches_doc() {
    let s = scenes();
    let Some(idx) = s.get("index") else {
        panic!("scenes.json 缺 index");
    };
    let img_asset = |p: &ti_core::data::Primitive, a: &str| {
        p.kind == "image" && p.params.get("asset").and_then(|v| v.as_str()) == Some(a)
    };
    assert!(
        idx.primitives.iter().any(|p| img_asset(p, "assets/ui/chart_full.png")),
        "index S22 满屏 chart_full 海图底缺失"
    );
    assert_eq!(
        idx.primitives
            .iter()
            .filter(|p| img_asset(p, "assets/ui/medallion_ring.png") && p.item >= 0 && p.item < 6)
            .count(),
        6,
        "index S22 应 6 枚 medallion（item 0..5）"
    );
    let status = idx.extra.get("status").expect("index 缺 status 矩形");
    assert_eq!(status.x, 540, "index status x=540（屏心居中）");
    assert_eq!(status.y, 1840, "index 尾行 y=1840");
}

/// select_role/select_map §4.2（1080 现代版面，S17.2 8pt）：标题条 h=96，列表基线 y=240，提示行 y=1840（QB115/123）。
#[test]
fn select_scenes_layout_matches_doc() {
    let s = scenes();
    let role = s.get("select_role").expect("scenes.json 缺 select_role");
    assert_eq!(role.title.h, 96, "select_role 标题条 h=96（1080 现代版面，S17.2 8pt）");
    assert_eq!(role.list.y, 480, "select_role 列表基线 y=480（S18.7d v3 右列 5 卡）");
    assert_eq!(role.list.row_h, 192, "select_role row_h=192（S18.7d v3 中央选项列）");
    assert_eq!(role.list.item_h, 192, "select_role item_h=192（S18.7d v3）");
    assert_eq!(role.hint.y, 1840, "select_role 提示行 y=1840（QB115/123）");
    let role_top = role.list.y - role.list.highlight_offset;
    assert!(role_top >= 108, "select_role 首块顶 {role_top} 压标题条");
    let map = s.get("select_map").expect("scenes.json 缺 select_map");
    assert_eq!(map.title.h, 96, "select_map 标题条 h=96（1080 现代版面，S17.2 8pt）");
    assert_eq!(map.list.y, 480, "select_map 列表基线 y=480（S18.7d v3 右列 3 卡）");
    assert_eq!(map.list.row_h, 320, "select_map row_h=320（S18.7d v3；S15.6 QB103 3 地图卡）");
    assert_eq!(map.list.item_h, 320, "select_map item_h=320（S18.7d v3）");
    assert_eq!(map.hint.y, 1840, "select_map 提示行 y=1840（QB115/123）");
}

/// main_map §4.3（S21.6 v4「羊皮海图桌面」）：标题 h=96，v4 版面键 = `08_main_map_v4_spec.md` §2 坐标表。
#[test]
fn main_map_layout_matches_doc() {
    let s = scenes();
    let Some(main) = s.get("main_map") else {
        panic!("scenes.json 缺 main_map");
    };
    assert_eq!(main.title.h, 96, "main_map 标题条 h=96（1080 现代版面，S17.2 8pt）");
    assert_eq!(main.hud.y, 152, "main_map HUD y=152（v2 回退键保留，§4.3）");
    assert_eq!(main.hud.h, 128, "main_map HUD h=128（v2 回退键，§4.3）");
    assert!(main.hud.y + main.hud.h <= 520, "main_map HUD 与等距区 y0=520 重叠（QB105 重锚）");
    assert_eq!(main.hint.y, 1632, "main_map 提示签条 y=1632（S21.6 v4）");
    // S21.6 v4 版面键（§4.3 S21.6 坐标表：主焦点 / 双铜牌 / 舵轮 CTA / 进度尺）
    let ex = &main.extra;
    let rect = |k: &str| ex.get(k).map(|r| (r.x, r.y, r.w, r.h));
    assert_eq!(
        rect("world_canvas"),
        Some((43, 250, 994, 826)),
        "main_map world_canvas v4（§4.3 S21.6）"
    );
    assert_eq!(
        rect("voyage_bar"),
        Some((48, 1088, 984, 48)),
        "main_map voyage_bar v4（§4.3 S21.6）"
    );
    let v4_keys = ["hud_plaque_l", "hud_plaque_r", "deck_spin"];
    assert!(v4_keys.iter().all(|k| ex.get(*k).is_some()), "main_map 缺 v4 键（双铜牌 / 舵轮 CTA）");
    // S15.6 QA133：实体 size 键在位（键缺失时 main_map.rs 静默回退旧公式 —— 键本身须被守护）
    let extra = &main.extra_scalar;
    for k in ["player.size", "landmark.size", "boss.size"] {
        assert!(
            extra.get(k).is_some_and(|v| *v > 0),
            "main_map 缺实体 size 键或 ≤0：`{k}`（QA133）"
        );
    }
    // S15.6 QB105/QA106：实体 size ≤ 瓦片行高（2*tile_h）——棋子/据点/BOSS 不越格
    let tile_h = extra.get("grid.tile_h").copied().unwrap_or(0);
    assert!(tile_h > 0, "main_map grid.tile_h 应为正（QB105 重锚数据在位）");
    for k in ["player.size", "landmark.size", "boss.size"] {
        let v = extra.get(k).copied().expect("实体 size 键在位（QA133）");
        assert!(v <= 2 * tile_h, "main_map `{k}` = {v} > 2*tile_h（QA106：实体边长 ≤ 瓦片行高）");
    }
}

/// vs §4.4（1080 现代版面，S17.2 8pt）：标题条 h=96，血条 64..512 / 576..1024（S15.6 QB120 解重叠）
/// y=168 h=48，回合 540/416，结果/空 540/960（QA141 屏心居中 + S15.6 批次6 挪出血条带）。
#[test]
fn vs_layout_matches_doc() {
    let s = scenes();
    let Some(vs) = s.get("vs") else {
        panic!("scenes.json 缺 vs");
    };
    assert_eq!(vs.title.h, 96, "vs 标题条 h=96（1080 现代版面，S17.2 8pt）");
    let ph = vs.extra.get("player_hp").expect("vs 缺 player_hp 矩形");
    let bh = vs.extra.get("boss_hp").expect("vs 缺 boss_hp 矩形");
    assert_eq!((ph.x, ph.y, ph.w, ph.h), (64, 168, 448, 48), "vs 玩家血条（§4.4，QB120 解重叠）");
    assert_eq!((bh.x, bh.y, bh.w, bh.h), (576, 168, 448, 48), "vs BOSS 血条（§4.4）");
    assert!(ph.x + ph.w <= 1080, "vs 玩家血条右缘越屏 1080");
    assert!(bh.x + bh.w <= 1080, "vs BOSS 血条右缘越屏 1080");
    // S15.6 QA140 互斥不变式（§4.4）：双条 gap ≥ 40（64..512 / 576..1024，gap=64）
    assert!(bh.x - (ph.x + ph.w) >= 40, "vs 双血条 gap < 40（QA140 互斥不变式）");
    // QA140：empty（「击键」提示）不得落在血条带 y=168..216（与双条撞行）
    let em = vs.extra.get("empty").expect("vs 缺 empty 矩形");
    assert!(em.y < 168 || em.y >= 168 + 48, "vs empty 撞血条带 168..216（QA140）");
    let rd = vs.extra.get("rounds").expect("vs 缺 rounds 矩形");
    assert_eq!(rd.x, 540, "vs 回合数 x=540（QA141 屏心居中语义坐标）");
    assert_eq!(rd.y, 416, "vs 回合数 y=416（S17.2 8pt：血条带之下首屏区）");
    let oc = vs.extra.get("outcome").expect("vs 缺 outcome 矩形");
    assert_eq!(oc.x, 540, "vs 战斗结果 x=540（QA141 屏心居中语义坐标）");
    assert_eq!(oc.y, 960, "vs 战斗结果 y=960（S15.6 批次6 屏心区）");
}

/// message §4.5（S20.2 v4 面板语言）：标题条 h=96，正文区 y=768 h=176（面板按内容收缩，区间约束），页码 1008（QA124/144），提示 y=1840。
#[test]
fn message_layout_matches_doc() {
    let s = scenes();
    let Some(msg) = s.get("message") else {
        panic!("scenes.json 缺 message");
    };
    assert_eq!(msg.title.h, 96, "message 标题条 h=96（1080 现代版面，S17.2 8pt）");
    let body = msg.extra.get("body").expect("message 缺 body 矩形");
    assert_eq!((body.y, body.h), (768, 176), "message 正文区（S20.2 v4 面板语言：按内容收缩）");
    // body.y + body.h ≤ 1800（正文区上界不越页码行，QA124/144 批次6 重分布）
    assert!(body.y + body.h <= 1800, "message 正文区越页码行 y=1800（FIND-026）");
    assert_eq!(msg.page.y, 1008, "message 页码 y=1008（S20.2 v4 随卡片收缩上移，QA144）");
    assert_eq!(msg.hint.y, 1840, "message 提示行 y=1840（QB115/123）");
}

/// saveload §4.6（S20.2 v4 面板语言）：列表基线 y=240 row_h=256 item_h=224（行高加大），金字列 x=792（左移居中），提示 y=1840。
#[test]
fn saveload_layout_matches_doc() {
    let s = scenes();
    let Some(sv) = s.get("saveload") else {
        panic!("scenes.json 缺 saveload");
    };
    assert_eq!(sv.title.h, 96, "saveload 标题条 h=96（1080 现代版面，S17.2 8pt）");
    assert_eq!(sv.list.y, 240, "saveload 列表基线 y=240（1080）");
    assert_eq!(sv.list.row_h, 256, "saveload 列表 row_h=256（S20.2 v4 面板语言：行高加大）");
    assert_eq!(sv.list.item_h, 224, "saveload 列表 item_h=224（S20.2 v4 面板语言）");
    assert_eq!(sv.list.gold_x, 792, "saveload 金字号列 x=792（S20.2 v4：列表左移后随栏居中）");
    assert_eq!(sv.hint.y, 1840, "saveload 提示行 y=1840（QB115/123）");
}

/// 假 `SceneHost`：录 `clear/fill_rect/draw_text` 调用 + 自带 `Framebuf`，
/// 其余 7 子 trait 方法给编译期最小 no-op（`Scene::render` 静态分发 `dyn SceneHost`，
/// 运行时只触发 clear/fill/draw_text/battle_snapshot 这几条）。
struct VsHost {
    pub fb: ti_render::framebuffer::Framebuf,
    pub shell: ti_app::menu_shell::MenuShell,
    pub fills: Vec<(i32, i32, i32, i32, u16)>,
    pub texts: Vec<(String, i32, i32)>,
    pub battle: Option<ti_app::game_app::BattleSnapshot>,
    pub sel_state: ti_core::domain::selection::SelectionState,
}

impl VsHost {
    fn new(battle: Option<ti_app::game_app::BattleSnapshot>) -> Self {
        Self {
            fb: ti_render::framebuffer::Framebuf::new(),
            shell: ti_app::menu_shell::MenuShell::with_upload_mode(
                1080.0,
                1920.0,
                ti_platform::gfx::UploadMode::FullRebuild,
            ),
            fills: Vec::new(),
            texts: Vec::new(),
            battle,
            sel_state: ti_core::domain::selection::SelectionState::default(),
        }
    }
    fn shell_fill(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        if w <= 0 || h <= 0 {
            return;
        }
        let slice = self.fb.as_mut_slice();
        for yy in y.max(0)..(y + h).min(FB_H as i32) {
            for xx in x.max(0)..(x + w).min(FB_W as i32) {
                slice[yy as usize * FB_W + xx as usize] = color;
            }
        }
    }
}

impl ti_app::scenes::NavHost for VsHost {
    fn current_scene(&self) -> ti_app::game_app::Scene {
        ti_app::game_app::Scene::Index
    }
    fn enter(&mut self, _next: ti_app::scenes::SceneKind) {}
    fn advance(&mut self, _input: ti_app::game_app::GameInput) -> bool {
        false
    }
}

impl ti_app::scenes::RenderHost for VsHost {
    fn shell(&self) -> &ti_app::menu_shell::MenuShell {
        &self.shell
    }
    /// S18.7a: 绘制目标 = 测试宿主的 `MenuShell`（`MenuShell` 实现 `Canvas`）；
    /// 本宿主已覆盖全部绘制方法（记录 + 写自带帧缓冲），
    /// 此处仅为满足 trait 契约（场景代码未改动）。
    fn draw_target(&mut self) -> &mut dyn ti_render::canvas::Canvas {
        &mut self.shell
    }
    fn shell_mut(&mut self) -> &mut ti_app::menu_shell::MenuShell {
        &mut self.shell
    }
    fn clear_frame(&mut self, color: u16) {
        self.fb.clear(color);
    }
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u16) {
        self.fills.push((x, y, w, h, color));
        self.shell_fill(x, y, w, h, color);
    }
    fn draw_text(&mut self, text: &str, x: i32, y: i32) {
        self.texts.push((text.to_string(), x, y));
    }
    fn draw_text_colored(&mut self, text: &str, x: i32, y: i32, _color: u16) {
        self.texts.push((text.to_string(), x, y));
    }
    fn draw_title_text(&mut self, text: &str, x: i32, y: i32, _color: u16) {
        self.texts.push((text.to_string(), x, y));
    }
    fn draw_hint_text(&mut self, text: &str, x: i32, y: i32, _color: u16) {
        self.texts.push((text.to_string(), x, y));
    }
    fn pixel_at(&self, x: i32, y: i32) -> u16 {
        let sl = self.fb.as_slice();
        let w = FB_W;
        let xi = usize::try_from(x).unwrap_or(0);
        let yi = usize::try_from(y).unwrap_or(0);
        sl.get(yi * w + xi).copied().unwrap_or(0)
    }
}

impl ti_app::scenes::SpriteHost for VsHost {
    fn terrain_grid_w(&self) -> u32 {
        0
    }
    fn has_terrain_atlas(&self) -> bool {
        false
    }
}

impl ti_app::scenes::MenuHost for VsHost {
    fn menu_selected(&self) -> usize {
        0
    }
    fn set_menu_selected(&mut self, _value: usize) {}
    fn reset_selection(&mut self) {}
    fn open_help(&mut self) {}
    fn request_quit(&mut self) {}
}

impl ti_app::scenes::SelectionHost for VsHost {
    fn character(&self) -> Option<ti_core::domain::selection::Character> {
        None
    }
    fn set_character(&mut self, _value: Option<ti_core::domain::selection::Character>) {}
    fn map_slot(&self) -> Option<ti_core::domain::selection::MapSlot> {
        None
    }
    fn set_map_slot(&mut self, _value: Option<ti_core::domain::selection::MapSlot>) {}
    fn selection(&self) -> ti_core::domain::selection::SelectionState {
        self.sel_state
    }
    fn selection_mut(&mut self) -> &mut ti_core::domain::selection::SelectionState {
        &mut self.sel_state
    }
    fn map_unlocked(&self, _slot: ti_core::domain::selection::MapSlot) -> bool {
        false
    }
    fn enter_main_map(&mut self) -> bool {
        false
    }
}

impl ti_app::scenes::MapHost for VsHost {
    fn map_pos(&self) -> Option<ti_core::domain::map::GridPos> {
        None
    }
    fn set_map_pos(&mut self, _value: Option<ti_core::domain::map::GridPos>) {}
    fn map_terrain(&self) -> Option<ti_core::domain::map::MapTerrain> {
        None
    }
    fn map_landmarks(&self) -> Vec<(u32, u32)> {
        Vec::new()
    }
    fn map_boss_tile(&self) -> Option<ti_core::domain::map::GridPos> {
        None
    }
    fn map_session_dispatch(&self, _code: u32) -> Option<ti_core::domain::map::SceneTarget> {
        None
    }
    fn set_last_dispatch(&mut self, _target: Option<ti_core::domain::map::SceneTarget>) {}
    fn start_battle(&mut self) {}
    fn open_message(&mut self, _kind: ti_core::domain::messages::MsgKind) {}
}

impl ti_app::scenes::EconomyHost for VsHost {
    fn price_of(&self, _gid: u32) -> Option<u32> {
        None
    }
    fn sell_price_of(&self, _gid: u32) -> Option<u32> {
        None
    }
    fn gold(&self) -> u32 {
        0
    }
    fn set_gold(&mut self, _value: u32) {}
    fn owned_index_of(&self, _gid: u32) -> Option<usize> {
        None
    }
    fn owned_push(&mut self, _gid: u32, _qty: u32) {}
    fn owned_bump(&mut self, _index: usize) {}
    fn owned_count(&self, _gid: u32) -> u32 {
        0
    }
    fn owned_snapshot(&self) -> Vec<(u32, u32)> {
        Vec::new()
    }
    fn owned_total(&self) -> usize {
        0
    }
    fn owned_take_all(&mut self, _gid: u32) -> u32 {
        0
    }
    fn bank_gold(&self) -> u32 {
        0
    }
    fn spend_bank_gold(&mut self, _amount: u32) {}
    fn build_increment(&mut self, _kind: ti_core::domain::economy::Building) {}
}

impl ti_app::scenes::SessionHost for VsHost {
    fn play_sfx(&mut self, _sfx: ti_core::domain::flow::Sfx) {}
    fn battle_step(&mut self) {}
    fn battle_snapshot(&self) -> Option<ti_app::game_app::BattleSnapshot> {
        self.battle
    }
    fn msg_kind(&self) -> Option<ti_core::domain::messages::MsgKind> {
        None
    }
    fn msg_page(&self) -> u32 {
        0
    }
    fn msg_total(&self) -> u32 {
        0
    }
    fn msg_next_page(&mut self) {}
    fn msg_prev_page(&mut self) {}
    fn close_message(&mut self) {}
    fn save_slots(&self) -> &[ti_core::domain::persistence::SaveSlot] {
        &[]
    }
    fn write_save_slot(&mut self, _slot: usize, _data: ti_core::domain::persistence::SaveSlot) {}
    fn restore_save(&mut self, _data: ti_core::domain::persistence::SaveSlot) {}
    fn in_map(&self) -> bool {
        false
    }
    fn in_battle(&self) -> bool {
        false
    }
    fn rng_steps(&self) -> u128 {
        0
    }
    fn saveload_cursor(&self) -> usize {
        0
    }
    fn set_saveload_cursor(&mut self, _slot: usize) {}
    fn unlocked_mask(&self) -> u8 {
        0
    }
    fn displayed_player_hp(&self) -> u32 {
        0
    }
    fn displayed_boss_hp(&self) -> u32 {
        0
    }
}

impl ti_app::scenes::SceneHost for VsHost {}

/// 类 3 元素存在性：场景层渲染后关键矩形必有非背景像素。
///
/// 用 `MenuShell` 直接清屏 + fill_rect 模拟场景层（不依赖 macroquad / GameApp 状态机），
/// 断言「标题条/HUD/血条 区有像素」—— 坐标全 0 时 fill_rect 画 (0,0,0,0) 不写像素，
/// 本测试会抓到「关键矩形静默不画」。
/// 底色基准 = 运行时画布底色 `menu_shell::COLOR_PARCH_DARK`（S15.6 裁决①深底口径；
/// scenes.json `bg` 键已删零消费，QA029 旧「清屏用 layout.bg 浅米白」判据已失真）。
///
/// vs 血条进一步走 `VsScene::render`（真实场景入口）：假 `SceneHost` 记录全部
/// `fill_rect` 调用，断言**玩家血条底衬矩形 = §4.4 的 (16,32,144,8)**——
/// vs.rs 若把宽度置 0 / 坐标改错，`render` 产出的血条 fill 调用会偏离 §4.4 表值。
#[test]
fn key_rects_have_pixels() {
    use ti_app::menu_shell::{COLOR_DANGER, COLOR_PARCH_DARK, COLOR_TITLE_BAR, MenuShell};

    let scs = scenes();
    let mut shell =
        MenuShell::with_upload_mode(1080.0, 1920.0, ti_platform::gfx::UploadMode::FullRebuild);
    let Some(layout) = scs.get("index") else {
        panic!("scenes.json 缺 index");
    };
    let bg_index = COLOR_PARCH_DARK; // QA046：判据色引 §2 单一源常量（禁内联 hex）
    shell.fb.clear(bg_index);
    // S22 罗盘环：关键区 = item 0 medallion 徽章 rect（坐标 0 时不写像素 → 静默不画）
    let med = layout
        .primitives
        .iter()
        .find(|p| {
            p.item == 0
                && p.params.get("asset").and_then(|v| v.as_str())
                    == Some("assets/ui/medallion_ring.png")
        })
        .expect("index S22 缺 medallion item0");
    let mr = med.rect;
    shell.fill_rect(mr.x, mr.y, mr.w, mr.h, COLOR_TITLE_BAR);
    let px = shell.fb.as_slice().to_vec();
    let ring_px = (mr.y..(mr.y + mr.h))
        .flat_map(|y| (mr.x..(mr.x + mr.w)).map(move |x| y * FB_W_I32 + x))
        .filter(|&i| px.get(usize::try_from(i).unwrap_or(0)) != Some(&bg_index))
        .count();
    assert!(ring_px > 0, "index S22 medallion 徽章 fill_rect 后无像素 —— 静默不画");

    // vs 场景层：走真实 `VsScene::render`（假 SceneHost 录 fill 调用），血条底衬
    // 必须是 §4.4 表值（scenes.json vs.player_hp 1080 矩形）；vs.rs 置 w=0 / 改坐标时这里抓得到
    let Some(layout_vs) = scs.get("vs") else {
        panic!("scenes.json 缺 vs");
    };
    let ph = layout_vs.extra.get("player_hp").expect("player_hp 缺");
    let mut vs_host = VsHost::new(Some(ti_app::game_app::BattleSnapshot {
        player_hp: 10,
        boss_hp: 20,
        rounds: 1,
        outcome: None,
        player_hp_max: 10,
        boss_hp_max: 20,
    }));
    let mut vs_scene = ti_app::scenes::VsScene;
    ti_app::scenes::Scene::render(&mut vs_scene, &mut vs_host);
    // 玩家血条**槽底**（`chrome::bar_slot` 铺 `COLOR_HILITE` 1080 槽位，§4.4 表值）——
    // vs.rs 把槽底宽度置 0 / 坐标偏离 §4.4 时，这条 fill 调用偏离表值，当场抓住。
    // S18.10：槽底色随色板演进（S18.7f 起槽底 = `vs.primitives` 原语，色 = brass_dark），
    // 故按**几何**判定（意图 = 槽底矩形符合 §4.4 表值，不锁死具体色值）。
    let backing = vs_host.fills.iter().find(|f| f.0 == ph.x && f.1 == ph.y && f.2 == ph.w).copied();
    assert!(
        backing.is_some(),
        "vs 玩家血条槽底 fill_rect 未按 §4.4 画 ({},{},{},{})（COLOR_HILITE）—— 宽度 0 / 整段不画 静默偏离",
        ph.x,
        ph.y,
        ph.w,
        ph.h
    );

    // 像素级复验：血条区（scenes.json 矩形）必有非背景像素
    let bg_vs = COLOR_PARCH_DARK; // QA029：底色基准 = 运行时画布底色（非 scenes.json bg 键）
    shell.fb.clear(bg_vs);
    shell.fill_rect(ph.x, ph.y, ph.w, ph.h, COLOR_DANGER); // QA046：§2 单一源（S14 内联 0xFDAF 已废）
    let px = shell.fb.as_slice().to_vec();
    let hp_px = (ph.y..(ph.y + ph.h))
        .flat_map(|y| (ph.x..(ph.x + ph.w)).map(move |x| y * FB_W_I32 + x))
        .filter(|&i| {
            let ui = usize::try_from(i).unwrap_or(0);
            px.get(ui) != Some(&bg_vs)
        })
        .count();
    assert!(hp_px > 0, "vs 玩家血条 fill_rect 后无像素 —— 坐标全 0 静默不画");
}

// S15.5 重锚（`08_render_arch.md` §6 契约 5）：legacy 逐像素写帧层移除后，门禁判据从
// `.Map` 精灵像素改为**代码绘制区**（chrome 原语覆盖的 1080 矩形 + 代码动态实体几何）。
// 三个判据：场景层（chrome）关键区 ≠ 底（title band）、文字区被 chrome 面板/底条覆盖（content 区）、
// 代码绘制动态实体（玩家/据点/BOSS）中心 ∈ `scenes.json` `portrait_zones`。

/// 渲染某场景的 1080×1920 代码绘制帧（纯代码渲染态；`forward_render` S15.6 QA017 已退役）。
fn render_scene_1080(scene: &str) -> Vec<u16> {
    let mut app = ti_app::game_app::GameApp::new(1080.0, 1920.0);
    // S15.6 QA125：像素类判据必须 TTF 在位（44px 真字形态），缺失即 panic 带诊断
    // （menu_shell 自动加载依赖 cwd 向上 8 级回溯找 assets/fonts/；禁止豆腐帧进判据）
    assert!(
        app.shell().ttf_font.is_some(),
        "render_scene_1080({scene}) TTF 缺失（assets/fonts/lxgw-wenkai-reg.ttf）：像素判据须 44px \
         真字形态，禁止豆腐帧（S15.6 QA125）；检查 cwd 能否回溯到仓库根 + 字体文件在位"
    );
    let _ = ti_app::forward_nav::navigate(&mut app, scene);
    app.render_to_framebuf().to_vec()
}

/// chrome 元素色集合（均 ≠ 羊皮纸底 ~0x10A3）：命中任一即视为「chrome 在该像素画了东西」。
fn chrome_colors() -> &'static [u16] {
    use ti_app::menu_shell as m;
    use ti_app::scenes::chrome as c;
    &[
        m::COLOR_WOOD,
        m::COLOR_GOLD_BRIGHT,
        m::COLOR_MENU_BG,
        m::COLOR_HILITE,
        c::COLOR_DANGER,
        c::COLOR_PLAYER,
        c::COLOR_LANDMARK,
        c::COLOR_WOOD_WARM, // S18.7 暖木框
        c::COLOR_GOLD_WARM, // S18.7 暖金标题/选中底
        // S18.10 暖色板：面板/甲板/海水/沙地令牌（中景换色后的 chrome 色集）
        c::COLOR_PANEL_WARM,
        c::COLOR_BRASS,
        c::COLOR_BRASS_LIGHT,
        c::COLOR_BRASS_DARK,
        c::COLOR_SEA_DEEP,
        c::COLOR_SEA_SHALLOW,
        c::COLOR_REEF,
        c::COLOR_LAND_SAND,
        c::COLOR_LAND_SAND_LIGHT,
        c::COLOR_LAND_SAND_DARK,
        0xFFFF, // chrome::COLOR_TEXT_ON_PARCH
    ]
}

/// 区内命中 chrome 元素色的像素数（1080×1920 帧；越界自动裁剪）。
fn region_chrome_count(px: &[u16], x: i32, y: i32, w: i32, h: i32) -> usize {
    let colors = chrome_colors();
    let y0 = y.max(0);
    let x0 = x.max(0);
    let y1 = (y + h).min(FB_H as i32);
    let x1 = (x + w).min(FB_W as i32);
    let mut n = 0usize;
    for py in y0..y1.max(y0) {
        for pxx in x0..x1.max(x0) {
            if colors.contains(&px[(py as usize) * FB_W + (pxx as usize)]) {
                n += 1;
            }
        }
    }
    n
}

/// S15.5 重锚（§6）：场景层（chrome）关键区「标题条带」≠ 底（chrome 必在标题条画
/// 木纹/金条/标题文字）——替换旧「场景层帧 XOR 精灵层帧 > 0」判据（无 legacy 精灵层可比）。
#[test]
fn scene_layer_contributes_text_region_pixels() {
    for scene in ["index", "select_role", "select_map", "main_map", "vs", "message", "saveload"] {
        let px = render_scene_1080(scene);
        assert_eq!(px.len(), FB_W * FB_H, "{scene} 帧缓冲应 = 1080×1920");
        if scene == "index" {
            // S18.8b：index 标题区 = **木牌贴图**（`image` 原语：assets/ui/board.png，
            // 120,250,840,360）→ 判据改为「标题区有暖色（木纹 r>b）像素」——
            // 既证明贴图被画上，又证明它压住了背景夜空（夜空是冷色 b>r）。
            let n = region_warm_count(&px, 120, 250, 840, 360);
            assert!(
                n > 0,
                "index 标题区（120,250,840,360）无木牌暖色像素 —— board 贴图未画 / 被盖死"
            );
            eprintln!("  index 木牌暖色像素 = {n}");
            continue;
        }
        // S18.20：index / select_role 改**贴图版式**（AI 夜空背景全屏铺底）→
        // 判据 = 标题区有「暖色像素」（木牌/立绘贴图把冷色夜空压住）。
        if matches!(scene, "index" | "select_role") {
            let region = if scene == "index" { (120, 250, 840, 360) } else { (76, 130, 356, 534) };
            let n = region_warm_count(&px, region.0, region.1, region.2, region.3);
            assert!(
                n > 0,
                "{scene} 标题/立绘区（{:?}, {:?}）无暖色像素 —— 贴图未画 / 被盖死",
                region.0,
                region.1
            );
            eprintln!("  {scene} 标题/立绘区暖色像素 = {n}");
            continue;
        }
        // S18.20：其余 5 屏 = **贴图背景版式**（AI 夜空 / 俯视夜海全屏铺底，
        // 旧木框与占位横线已退役）→ 判据 = 非黑覆盖率 ≥ 990‰（贴图铺满画面）。
        let covered = non_black_permille(&px);
        assert!(
            covered >= 990,
            "{scene} 非黑覆盖率 {covered}‰ < 990‰（背景贴图未铺满 / 被整块盖死）"
        );
        eprintln!("  {scene} 背景贴图覆盖率 = {covered}‰");
    }
}

/// S15.6 QA131（S20.2 v4 面板语言重锚）：选中态判据——index = 选中标签变金（S18.8b）；
/// select_role = **3px 黄铜边**（`chrome::selected_border`，旧 S18.7d「gold_warm 整块填充底」退役，
/// 卡底统一 `panel_wood`）：行 0 卡顶缘外 3px 边带**有** `brass` 像素，行 1 卡对应边带**无**（边框不外溢）。
fn region_color_count(px: &[u16], x: i32, y: i32, w: i32, h: i32, color: u16) -> usize {
    let y0 = y.max(0);
    let x0 = x.max(0);
    let y1 = (y + h).min(FB_H as i32);
    let x1 = (x + w).min(FB_W as i32);
    let mut n = 0usize;
    for py in y0..y1.max(y0) {
        for pxx in x0..x1.max(x0) {
            if px[(py as usize) * FB_W + (pxx as usize)] == color {
                n += 1;
            }
        }
    }
    n
}

/// 非黑像素覆盖率（千分比；整数运算，无浮点转换）——贴图背景版式的判据。
fn non_black_permille(px: &[u16]) -> u64 {
    let n = px.iter().filter(|c| **c != 0).count() as u64;
    let total = px.len().max(1) as u64;
    n * 1000 / total
}

/// S18.8b：区域内「暖色」像素数（RGB565 解码后 `r > b`）——木纹/金调判据
/// （夜空背景是冷色 `b > r` → 贴图压上后区域转暖）。
fn region_warm_count(px: &[u16], x: i32, y: i32, w: i32, h: i32) -> usize {
    let y0 = y.max(0);
    let x0 = x.max(0);
    let y1 = (y + h).min(FB_H as i32);
    let x1 = (x + w).min(FB_W as i32);
    let mut n = 0usize;
    for py in y0..y1.max(y0) {
        for pxx in x0..x1.max(x0) {
            let c = px[(py as usize) * FB_W + (pxx as usize)];
            let r = i32::from((c >> 11) & 0x1F);
            let b = i32::from(c & 0x1F);
            if r > b {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn selected_state_marker_visible_and_no_leak() {
    let scs = scenes();
    for scene in ["index", "select_role"] {
        let px = render_scene_1080(scene);
        let layout = scs.get(scene).expect("scenes.json 缺场景");
        if scene == "index" {
            // S22：选中徽章（item 0）标签区有 gold_warm，未选中（item 1）无（选中态不外溢）
            let gold = ti_render::theme::Theme::default()
                .by_name("gold_warm")
                .unwrap_or(ti_app::menu_shell::COLOR_SELECTED);
            let lbl = |i: i64| {
                layout
                    .primitives
                    .iter()
                    .find(|p| p.item == i && p.kind == "text")
                    .expect("index S22 缺 medallion 标签原语")
            };
            let (r0, r1) = (lbl(0).rect, lbl(1).rect);
            assert!(
                region_color_count(&px, r0.x, r0.y, r0.w, r0.h, gold) > 0,
                "index S22 选中徽章 0 标签区无 gold_warm（选中标签应变金）"
            );
            assert_eq!(
                region_color_count(&px, r1.x, r1.y, r1.w, r1.h, gold),
                0,
                "index S22 未选中徽章 1 标签区有 gold_warm（选中态泄漏）"
            );
            continue;
        }
        // S20.2 v4 面板语言：选中态 = **黄铜/亮金边**（`chrome::selected_border`，S26.3 外圈
        // `gold_warm`，此前 = 整块金色填充，刺眼且抢戏；卡底统一 `panel_wood`，只画边不填底）；
        // 判据 = 行 0 卡顶缘**外** 边带有 `gold_warm` 像素（该带只有选中边框会画）；
        // 行 1 卡左缘**外** 竖条无（避开行 0 边框触及高度：其外框止于顶+高+边= top1-8 内）。
        let sel = ti_render::theme::Theme::default()
            .by_name("gold_warm")
            .unwrap_or(ti_app::scenes::chrome::COLOR_GOLD_WARM);
        let top0 = layout.list.y - layout.list.highlight_offset;
        let top1 = top0 + layout.list.row_h;
        assert!(
            region_color_count(&px, layout.list.x, top0 - 3, layout.list.w, 3, sel) > 0,
            "{scene} 选中卡（行 0，顶 {top0}）顶外边带无 {sel:#06x} 像素（S20.2 QA131：选中态 = 3px 黄铜边）"
        );
        assert_eq!(
            region_color_count(&px, layout.list.x - 3, top1 + 10, 3, layout.list.row_h - 20, sel),
            0,
            "{scene} 未选中卡（行 1，顶 {top1}）左外竖条不应有 {sel:#06x}（S20.2：选中边框泄漏）"
        );
    }
}

/// S15.6 QA132①（QB150 重锚）：index 尾行（i18n `scene.index.footer`，开发诊断不再进 UI）
/// 走 `chrome::hint_row` = **提示档**（TTF 28px / 位图 8px advance，QB115 三档）+ 屏心居中。
/// ① 条宽 == `hint_text_width_px(footer)+8`（hint_row 同式，跟活跃字体）；
/// ② 条高 ≥ `hint_glyph_px`（TTF 态 28px；条高固定 16px 盖不住提示档字形 → 回归当场 FAIL）；
/// ③ 两态：同一文本 TTF 档条宽 > 位图档条宽（字号档单调，QA132② 两态判据）。
#[test]
fn hint_bar_width_height_follow_active_font() {
    let px = render_scene_1080("index");
    let scs = scenes();
    let idx = scs.get("index").expect("scenes.json 缺 index");
    let status = idx.extra.get("status").expect("index 缺 status 矩形");
    // index.rs 尾行 = `chrome::hint_row(host, footer, status.x + 2, status.y)`；
    // footer 与代码同源（`i18n::sget("scene.index.footer")`，缺失 key 回 key 本身）。
    // QB123：`hint_row` 忽略 x 参数、**屏心居中**（条左缘 cx = (1080 - 条宽) / 2）
    let bar_top = status.y - 2;
    let footer = ti_app::i18n::sget("scene.index.footer");
    // 渲染管线 `MenuShell::new` = 内置 TTF 三档自动在位（QA128 进程级缓存，EXP-050 回溯定位）
    let shell_ttf = ti_app::menu_shell::MenuShell::new(1080.0, 1920.0);
    assert!(shell_ttf.ttf_loaded(), "测试态 TTF 三档须在位（内置字体缺失 → QA132 判据全灭）");
    let expect_w = shell_ttf.hint_text_width_px(footer) + 8;
    assert!(expect_w > 8, "尾行文本宽非零（TTF 态 28px 提示档字宽 > 0，实际 {expect_w}）");
    let bx = (1080 - expect_w) / 2;
    // ① 宽：条顶行连续 0x31A9（bevel 亮线）像素数 = 条宽
    let row = &px[bar_top as usize * FB_W..(bar_top + 1) as usize * FB_W];
    let mut w = 0i32;
    for i in 0..(FB_W - bx as usize) {
        if row[bx as usize + i] == ti_app::menu_shell::COLOR_BEVEL_HI {
            w += 1;
        } else {
            break;
        }
    }
    assert_eq!(
        w, expect_w,
        "index 尾行 hint 条宽 {w} ≠ hint_text_width_px+8 = {expect_w}（QA132①：条宽跟提示档字体文本宽，chrome::hint_row 同式）"
    );
    // ② 高：条顶行往下连续非底色像素 ≥ hint_glyph_px（提示档字形高）
    let dark = ti_app::menu_shell::COLOR_PARCH_DARK;
    let mut h = 0i32;
    let mut yy = bar_top;
    while yy < 1920 {
        let v = px[yy as usize * FB_W + bx as usize];
        if v == ti_app::menu_shell::COLOR_BEVEL_HI
            || v == ti_app::menu_shell::COLOR_MENU_BG
            || v == ti_app::scenes::chrome::COLOR_TEXT_ON_PARCH
            || v == ti_app::menu_shell::COLOR_WOOD
        {
            h += 1;
        } else if v == dark {
            break; // 底行暗角以下 = 条已结束
        } else {
            h += 1;
        }
        yy += 1;
    }
    assert!(
        h >= shell_ttf.hint_glyph_px(),
        "index 尾行 hint 条高 {h} < hint_glyph_px {}（QA104：条高须盖提示档字形高，TTF 态 = 28px）",
        shell_ttf.hint_glyph_px()
    );
    // ③ 两态（QA132②）：位图回退壳（清空 TTF 三档）同文本条宽 < TTF 壳（提示档 advance 8 < 28px 字宽）
    let mut shell_bitmap = ti_app::menu_shell::MenuShell::new(1080.0, 1920.0);
    shell_bitmap.set_ttf_fonts(None, None, None);
    let w_bitmap = shell_bitmap.hint_text_width_px(footer) + 8;
    let w_ttf = shell_ttf.hint_text_width_px(footer) + 8;
    assert!(
        w_bitmap < w_ttf,
        "QA132② 两态单调：位图档条宽 {w_bitmap} 应 < TTF 提示档 {w_ttf}（字号档单调）"
    );
}

/// S15.5 重锚（§6）：「文字区被 chrome 面板/底条覆盖，非底色占比 > 0」——替换旧「文字区 legacy
/// 精灵密度 ≤ 90%」判据。对 5 个有明确 chrome 面板/血条的场景，断言其内容区（列表/ HUD/ 血条）
/// 有 chrome 元素像素（列表项卡片/HUD 面板/血条槽底，均非羊皮纸底）。
#[test]
fn text_regions_not_flooded_by_chrome() {
    // 场景 → 内容区（scenes.json 1080 矩形；S15.6 QB101/QB103/QB120 重分布后坐标）
    let regions: [(&str, i32, i32, i32, i32); 5] = [
        ("index", 90, 360, 900, 170), // 列表首项区（QB101：顶 360，item_h 170）
        ("select_role", 90, 240, 900, 230), // 角色列表区
        ("select_map", 90, 200, 900, 280), // 地图列表区（QB103：顶 200，item_h 280）
        ("main_map", 80, 150, 920, 120), // HUD 面板
        ("vs", 60, 170, 440, 48),     // 玩家血条区（QB120：60..500）
    ];
    for (scene, x, y, w, h) in regions {
        let px = render_scene_1080(scene);
        assert_eq!(px.len(), FB_W * FB_H, "{scene} 帧缓冲应 = 1080×1920");
        let n = region_chrome_count(&px, x, y, w, h);
        assert!(
            n > 0,
            "{scene} 内容区 ({x},{y},{w}×{h}) 无 chrome 像素 —— 文字区未被面板/底条覆盖（全底 / 静默不画）"
        );
        eprintln!("  {scene} 内容区 chrome 像素 = {n}");
    }
}

/// S15.5 重锚（§6），S17.8 v3 再锚：`large_portraits` 原「.Map 大精灵 clip 矩形 ∈
/// portrait_zones」改为「**代码绘制动态实体**（玩家棋子/据点/BOSS 格）中心 ∈ `scenes.json`
/// `portrait_zones` 声明区」。main_map 的等距坐标经 `chrome::iso_cell_fit`（v3 `board_fit`
/// G 依赖拟合，缺席回退 `grid.*`）落 1080 呈现层，纯几何判定。
#[test]
fn dynamic_entities_land_in_declared_portrait_zones() {
    let scs = ti_core::data::scenes().expect("scenes.json");
    let layout = scs.get("main_map").cloned().expect("scenes.json 缺 main_map");
    let zones: Vec<(i32, i32, i32, i32)> =
        layout.portrait_zones.iter().map(|r| (r.x, r.y, r.w, r.h)).collect();
    assert!(!zones.is_empty(), "scenes.json main_map 未声明 portrait_zones（动态实体无落区可判）");

    let mut app = ti_app::game_app::GameApp::new(1080.0, 1920.0);
    let _ = ti_app::forward_nav::navigate(&mut app, "main_map");
    let g = app.map_grid_width();

    // 等距 tile 半宽/半高（S17.8 v3：`chrome::iso_tile_fit` 读 `board_fit` G 依赖几何，缺席回退 `grid.tile_w/h`）。
    let (tw, th) = ti_app::scenes::chrome::iso_tile_fit(&layout, g);
    assert!(tw > 0 && th > 0, "main_map 棋盘 tile_w/h 应为正（等距投影可用）");

    fn in_zones(cx: i32, cy: i32, zones: &[(i32, i32, i32, i32)]) -> bool {
        zones.iter().any(|&(zx, zy, zw, zh)| zx <= cx && cx <= zx + zw && zy <= cy && cy <= zy + zh)
    }

    let mut checked = 0usize;
    // 玩家棋子：iso_cell_fit 中心 = (sx + tw, sy + th)（菱形中心）。
    if let Some(pos) = app.map_pos() {
        let (sx, sy) = ti_app::scenes::chrome::iso_cell_fit(
            i32::try_from(pos.row).unwrap_or(0),
            i32::try_from(pos.col).unwrap_or(0),
            &layout,
            g,
        );
        let (cx, cy) = (sx + tw, sy + th);
        assert!(in_zones(cx, cy, &zones), "玩家棋子中心 ({cx},{cy}) 越出 main_map portrait_zones");
        checked += 1;
    }
    for (r, c) in app.map_landmarks.clone() {
        let (sx, sy) = ti_app::scenes::chrome::iso_cell_fit(
            i32::try_from(r).unwrap_or(0),
            i32::try_from(c).unwrap_or(0),
            &layout,
            g,
        );
        let (cx, cy) = (sx + tw, sy + th);
        assert!(in_zones(cx, cy, &zones), "据点 ({r},{c}) 中心 ({cx},{cy}) 越出 portrait_zones");
        checked += 1;
    }
    if let Some(bt) = app.map_boss_tile() {
        let (sx, sy) = ti_app::scenes::chrome::iso_cell_fit(
            i32::try_from(bt.row).unwrap_or(0),
            i32::try_from(bt.col).unwrap_or(0),
            &layout,
            g,
        );
        let (cx, cy) = (sx + tw, sy + th);
        assert!(in_zones(cx, cy, &zones), "BOSS 格中心 ({cx},{cy}) 越出 portrait_zones");
        checked += 1;
    }
    assert!(checked >= 1, "main_map 无任何动态实体可判（player/landmark/boss 全缺）");
    eprintln!("  main_map 动态实体 {checked} 个（玩家/据点/BOSS）中心均 ∈ portrait_zones");
}
