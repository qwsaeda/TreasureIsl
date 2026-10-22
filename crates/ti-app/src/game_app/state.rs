//! 状态访问器：`GameApp` 的只读/轻写状态入口。
//!
//! 拆分自 `game_app.rs`（S20.0b）：场景模块只经这些入口取状态
//!（`docs/process/01_architecture.md` §2 分层边界）。

use ti_core::data;
use ti_core::domain::economy::{Bank, Building, PriceTable};
use ti_core::domain::flow::Sfx;
use ti_core::domain::map::GridPos;
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::build;
use ti_core::rules::rng::GameRng;

use crate::menu_shell::MenuShell;

use super::{GameApp, Scene};
use crate::scenes::SpriteHost;
use crate::sprite_atlas::SpriteAtlas;

impl GameApp {
    /// 当前场景（只读）。
    #[must_use]
    pub fn scene(&self) -> Scene {
        self.scene
    }
    /// 是否退出。
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.quit
    }
    /// 当前选中角色（只读）。
    #[must_use]
    pub fn character(&self) -> Option<Character> {
        self.character
    }
    /// 当前选中地图槽位（只读）。
    #[must_use]
    pub fn map_slot(&self) -> Option<MapSlot> {
        self.map_slot
    }
    /// 当前地图坐标（只读）。
    #[must_use]
    pub fn map_pos(&self) -> Option<GridPos> {
        self.map_pos
    }
    /// 主地图地形网格宽（v3 `chrome::board_fit` 的 G 依赖源；未载图缺省 20）。
    ///
    /// 暴露给测试/上层：`board_fit`/`iso_cell_fit` 正逆几何须与实际渲染同 G
    /// （`main_map.rs` render 用 `host.terrain_grid_w()`，S17.8）。
    #[must_use]
    pub fn map_grid_width(&self) -> u32 {
        self.terrain_grid_w()
    }
    /// 是否已通关（`is_cleared` 逐图门禁，`business/progression.md` §通关）：
    /// m03 的 BOSS 已击败 = 全图通杀（`battle_step` 在 m03 胜时置位，
    /// `restore_save` 按存档掩码 CLEARED 位（bit 3）还原）。
    #[must_use]
    pub fn is_cleared(&self) -> bool {
        self.cleared
    }
    /// 某图 BOSS 是否已击败（逐图门禁判据，`rules::progression`）：
    /// m01/m02 的击败位 = 下一图已解锁（`can_select` 口径），m03 的击败位 = 通关位。
    #[must_use]
    pub fn is_map_cleared(&self, m: MapSlot) -> bool {
        match m {
            MapSlot::M01 => self.unlocked_maps.contains(&MapSlot::M02),
            MapSlot::M02 => self.unlocked_maps.contains(&MapSlot::M03),
            MapSlot::M03 => self.cleared,
        }
    }
    /// BOSS 格坐标（`MapSession::boss_tile`，主地图渲染橙红标记 + 战斗触发用，只读）。
    #[must_use]
    pub fn map_boss_tile(&self) -> Option<GridPos> {
        self.map_session.boss_tile()
    }
    /// 当前金币数（只读）。
    #[must_use]
    pub fn gold(&self) -> u32 {
        self.gold
    }
    /// 已拥有物品列表（只读）。
    #[must_use]
    pub fn owned(&self) -> &[(u32, u32)] {
        &self.owned
    }
    /// 经济管理器（只读）。
    #[must_use]
    pub fn bank(&self) -> &Bank {
        &self.bank
    }
    /// 价格表（只读，`PriceTable` 来自 `domain::economy` 的域类型）。
    #[must_use]
    pub fn price_table(&self) -> &PriceTable {
        &self.price_table
    }
    /// GPU 渲染器只读访问器（feature `gpu-render`，P4-4 测试用）。
    #[cfg(feature = "gpu-render")]
    #[must_use]
    pub fn gpu_renderer(&self) -> &ti_platform::gpu_renderer::GpuRenderer {
        &self.gpu_renderer
    }
    /// GPU 渲染器可变访问器（feature `gpu-render`，blit 路径用）。
    #[cfg(feature = "gpu-render")]
    pub fn gpu_renderer_mut(&mut self) -> &mut ti_platform::gpu_renderer::GpuRenderer {
        &mut self.gpu_renderer
    }
    /// GPU 累计 draw call 数（feature `gpu-render`，P4-4 判据 2/3）。
    #[cfg(feature = "gpu-render")]
    #[must_use]
    pub fn gpu_draw_calls(&self) -> u64 {
        self.gpu_renderer.draw_call_count()
    }
    /// 已建造列表（只读）。
    #[must_use]
    pub fn buildings(&self) -> &[(Building, u32)] {
        &self.buildings
    }
    /// 全部已建建筑提供的战斗 atk 总加成（`Σ atk_bonus × level`，`docs/business/combat.md` §建造加成）。
    ///
    /// 成长机制入口：`start_battle` 用 `player.atk + build_atk_bonus()`。无建筑 / balance 加载失败 → 0。
    #[must_use]
    pub fn build_atk_bonus(&self) -> u32 {
        let Some(cfg) = data::balance().ok().map(|b| &b.build) else {
            return 0;
        };
        self.buildings.iter().map(|(k, lvl)| cfg.atk_bonus_at(build::kind_id(*k), *lvl)).sum()
    }
    /// 全部已建建筑提供的战斗 hp 总加成（`Σ hp_bonus × level`，同上）。
    ///
    /// 成长机制入口：`start_battle` 用 `player.hp + build_hp_bonus()`。无建筑 / balance 加载失败 → 0。
    #[must_use]
    pub fn build_hp_bonus(&self) -> u32 {
        let Some(cfg) = data::balance().ok().map(|b| &b.build) else {
            return 0;
        };
        self.buildings.iter().map(|(k, lvl)| cfg.hp_bonus_at(build::kind_id(*k), *lvl)).sum()
    }
    /// 主菜单选中项索引（只读）。
    #[must_use]
    pub fn menu_selected(&self) -> usize {
        self.menu_selected
    }
    /// 消息页码（只读）。
    #[must_use]
    pub fn msg_page(&self) -> u32 {
        self.msg_page
    }
    /// 取可变 RNG 句柄（测试用）。
    #[must_use]
    pub fn rng_mut(&mut self) -> &mut GameRng {
        &mut self.rng
    }
    /// 设置 RNG 种子（测试用）。
    /// S18.7a：切换绘制目标模式。
    ///
    /// - `true` = **记录模式**：绘制命令进 `绘制列表`（Bevy 路径；实体由 `ti-engine` 转换）；
    /// - `false` = CPU 帧缓冲模式（诊断 bin 路径；macroquad 发货壳 S18.8a 已退役）。
    ///
    /// 切换时同步度量口径（主题 + 三档 TTF）→ 两路径版面完全一致。
    pub fn set_record_draw(&mut self, on: bool) {
        if on && !self.record_draw {
            self.draw_list.theme = self.shell.theme.clone();
            self.draw_list.ttf = ti_render::drawlist::TtfSet {
                body: self.shell.ttf_font.clone(),
                title: self.shell.ttf_title.clone(),
                hint: self.shell.ttf_hint.clone(),
            };
            self.draw_list.clear();
        }
        self.record_draw = on;
    }
    /// S18.7a：取本帧绘制列表（记录模式下有效；CPU 模式下为空）。
    #[must_use]
    pub fn draw_list(&self) -> &ti_render::drawlist::DrawList {
        &self.draw_list
    }
    /// 设置随机种子（测试可复现）。
    pub fn set_rng_seed(&mut self, seed: u32) {
        self.rng_seed = seed;
        self.rng.set_seed(seed);
    }
    /// 取渲染壳（只读）。
    #[must_use]
    pub fn shell(&self) -> &MenuShell {
        &self.shell
    }
    /// 取可变渲染壳（壳层对 `MenuShell` 的可变操作，如注入字体/音频、切缩放模式、`present`）。
    ///
    /// `shell()` 给 `&MenuShell` 无法驱动 `MenuShell` 的 `&mut` 方法，bins 统一走本访问器。
    pub fn shell_mut(&mut self) -> &mut MenuShell {
        &mut self.shell
    }
    /// 音效访问器（`last_sfx` 字段的 `&mut` 视图，壳层交互后读/清）。
    pub fn last_sfx(&mut self) -> &mut Option<Sfx> {
        &mut self.last_sfx
    }
    /// 清空本帧音效（`last_sfx = None`，壳层交互后调用，避免同一音效连播）。
    pub fn clear_last_sfx(&mut self) {
        self.last_sfx = None;
    }
    /// 设置本帧经过毫秒（S6.4，驱动多帧精灵动画推进）。
    pub fn set_frame_delta_ms(&mut self, ms: u32) {
        self.frame_delta_ms = ms;
    }
    /// 取可变精灵图集（测试用）。
    #[must_use]
    pub fn atlas_mut(&mut self) -> Option<&mut SpriteAtlas> {
        self.atlas.as_mut()
    }
}
