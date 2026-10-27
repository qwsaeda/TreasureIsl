//! 地图选择场景（S9 骨架）：输入判定 / 拾取命中 / 绘制三面全部落在本文件。
use ti_core::domain::flow::Sfx;
use ti_core::domain::selection::MapSlot;
use ti_core::rules::progression::boss_tier_of;

use crate::game_app::{GameInput, HitData};
use crate::scenes::chrome;
use crate::scenes::{Scene, SceneHost, SceneKind};

/// 地图槽数（域出口 `MapSlot::count()` = `MapSlot::all()` 长度，B11 单一事实源；
/// `spec/menu.md §c`）。
const MAP_COUNT: u32 = MapSlot::count();

/// 地图选择场景处理器（`SelectMap`）。
pub struct SelectMapScene;

/// 读「角色 + 地图是否齐全」两个布尔（`rules::selection` 判据）。
fn selection_done(host: &dyn SceneHost) -> bool {
    let s = host.selection();
    s.has_character() && s.has_map()
}

impl Scene for SelectMapScene {
    fn kind(&self) -> SceneKind {
        SceneKind::SelectMap
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Prev => {
                let cur = host.map_slot().map_or(0, MapSlot::zero_index);
                let n = if cur == 0 { MAP_COUNT - 1 } else { cur - 1 };
                // n ∈ [0, MAP_COUNT) 恒合法（n 由 cur ∈ [0,3) 减 1/取模得来，MAP_COUNT=3）
                let Some(slot) = MapSlot::from_index(n) else {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                };
                host.set_map_slot(Some(slot));
                host.play_sfx(Sfx::Press);
                false
            }
            GameInput::Next => {
                // 首次进 select_map 时 map_slot=None：第一次 Next 置第 0 张（M01），
                // 后续 Next 按 zero_index 推进（M02/M03/循环）。对齐 e2e 测试
                // `e2e_full_game_session` 阶段 3：连续 3 次 Next = M01/M02/M03。
                if host.map_slot().is_none() {
                    // 首次 Next 置 M01 并停留（不推进，避免跳过第 0 张）
                    let Some(first) = MapSlot::from_index(0) else {
                        host.play_sfx(Sfx::Invalid);
                        return false;
                    };
                    host.set_map_slot(Some(first));
                    host.play_sfx(Sfx::Press);
                    return false;
                }
                let cur = host.map_slot().map_or(0, MapSlot::zero_index);
                let n = (cur + 1) % MAP_COUNT;
                let Some(slot) = MapSlot::from_index(n) else {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                };
                host.set_map_slot(Some(slot));
                host.play_sfx(Sfx::Press);
                false
            }
            GameInput::Confirm => {
                // S14：首次进 select_map 时 map_slot=None（select_role 只设 character），
                // Confirm 兜底选第 0 张地图（对齐 select_role Confirm 的 picked 兜底语义）
                if host.map_slot().is_none() {
                    let Some(s) = MapSlot::from_index(0) else {
                        host.play_sfx(Sfx::Invalid);
                        return false;
                    };
                    host.set_map_slot(Some(s));
                }
                let Some(slot) = host.map_slot() else {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                };
                if !host.map_unlocked(slot) {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                }
                host.selection_mut().map_slot = Some(slot);
                let done = selection_done(host);
                host.selection_mut().completed = done;
                if done {
                    let ok = host.enter_main_map();
                    if ok {
                        true
                    } else {
                        // FIND-204：数据校验失败（balance 缺条目/坐标越界）→ 停留原场景
                        host.play_sfx(Sfx::Invalid);
                        false
                    }
                } else {
                    host.play_sfx(Sfx::Press);
                    false
                }
            }
            GameInput::Back => {
                host.enter(SceneKind::SelectRole);
                true
            }
            _ => false,
        }
    }

    /// S29.2：点某行（`pick_key=2` 地图行，`hit.index` = 行下标）= 选中该地图并确认；下滑 = 返回。
    ///
    /// 未解锁地图点按由 `on_input` 的 `Confirm` 分支判 `Sfx::Invalid`。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        if hit.swipe == 2 {
            return host.advance(GameInput::Back);
        }
        if hit.swipe != 0 || hit.key != 2 {
            return false;
        }
        let i = usize::try_from(hit.index.max(0)).unwrap_or(0);
        host.set_map_slot(MapSlot::from_index(u32::try_from(i).unwrap_or(0)));
        chrome::flash_press();
        host.advance(GameInput::Confirm)
    }

    /// S18.7d 声明式原语化：版面（夜空 / 左地图拼贴面板 / 右 3 行选项 / 提示行）=
    /// `scenes.json` `select_map.primitives`（行名 i18n `glmapm0N.Map` + map 图标 + 选中高亮
    /// 全数据）；运行时 = 按压闪影 + 锁块（锁定态 = `unlocked_mask`，几何走
    /// `locked.text_x` / 卡片右缘公式，UI-10）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let layout = chrome::scene_layout("select_map");
        let current = host.map_slot().map_or(0, |m| m.zero_index() as usize);
        chrome::render_primitives(host, "select_map", i64::try_from(current).unwrap_or(-1));
        let held = host.press_held();
        let flashing = chrome::press_flash_active();
        if held || flashing {
            let r = layout.list.item_rect(current);
            host.fill_rect(r.x, r.y, r.w, r.h, chrome::COLOR_BEVEL_SHADOW);
        }
        // UI-10：锁定行 = 深色「锁」块（压卡片右内侧，钳进卡片不溢出）；
        // QA033：锁块 x 只走 `scenes.json` `locked.text_x`（1080 数据驱动，契约 4）；
        // 飞轮 R1 ③-13：几何参数组（inset/max_w/min_w/h）同走 `locked.*` 数据键；
        // 键缺失/非正 = 不画锁块（旧 176 时代休眠回退值 130 已删，契约 1）。
        for (i, m) in MapSlot::all().iter().enumerate() {
            if host.map_unlocked(*m) {
                continue;
            }
            let r = layout.list.item_rect(i);
            let sc = &layout.extra_scalar;
            let lock_x = sc.get("locked.text_x").copied().unwrap_or(0);
            let inset = sc.get("locked.inset").copied().unwrap_or(0);
            let max_w = sc.get("locked.max_w").copied().unwrap_or(0);
            let min_w = sc.get("locked.min_w").copied().unwrap_or(0);
            let lock_h = sc.get("locked.h").copied().unwrap_or(0);
            // S26.3：锁条 = 卡片**底部**（旧 = 卡片垂直中心，与地图名同带 ⇒ 盖住「海图三」
            // 中横，观感「海图二×2」（2026-10-26 用户报告误判源）；底部与名字/星标互不重叠。
            let ty = r.y + r.h - lock_h - 12;
            if lock_x > 0 && max_w > 0 && lock_h > 0 {
                // 契约 1（`08_render_arch.md`）：钳制基准用呈现坐标系常量，
                // 不得出现 legacy 176 字面量。
                // 锁标右界按**卡片右缘**（`item_rect` 的 x+w）而非屏幕宽推导，
                // 按钮变窄时锁标随卡片收敛，不会溢出卡片（F12）
                // S26.3：锁条色 `COLOR_WOOD`（0x3A17 解 RGB565 实为蓝）出航海暖调，
                // 改 `COLOR_BRASS_DARK`（深棕红 0x51E4）—— 锁标守暖色板。
                let card_right = r.x + r.w;
                let lock_w = (card_right - inset - lock_x).min(max_w);
                host.fill_rect(lock_x, ty, lock_w.max(min_w), lock_h, chrome::COLOR_BRASS_DARK);
            }
        }
        // S19.5：选图卡显示 **BOSS 难度星**（`rules::progression::boss_tier_of` → tier+1 颗星，
        // 0 起 ⇒ 1~3 星）—— 让玩家在**选图时**就知道这张图的 BOSS 强度（难度阶梯可见 ⇒
        // 据此决定攒金/建造目标）。锁定图也显示（难度是地图固有属性）。
        // 飞轮 R1 ③-14：几何参数组（size/gap/inset/top/max/bevel_px）走 `stars.*` 数据键。
        let sc = &layout.extra_scalar;
        let star_size = sc.get("stars.size").copied().unwrap_or(0);
        let star_gap = sc.get("stars.gap").copied().unwrap_or(0);
        let star_inset = sc.get("stars.inset").copied().unwrap_or(0);
        let star_top = sc.get("stars.top").copied().unwrap_or(0);
        let star_max = sc.get("stars.max").copied().unwrap_or(0);
        let star_bevel = sc.get("stars.bevel_px").copied().unwrap_or(0);
        for (i, m) in MapSlot::all().iter().enumerate() {
            let Some(tier) = boss_tier_of(*m) else {
                continue;
            };
            let stars = i32::try_from(tier.saturating_add(1)).unwrap_or(1).min(star_max);
            if stars <= 0 || star_size <= 0 {
                continue;
            }
            let r = layout.list.item_rect(i);
            // 卡片右上角内侧（x 从右往左排，y 居中于卡片上半）
            let right = r.x + r.w - star_inset;
            let top = r.y + star_top;
            for k in 0..stars {
                let x = right - (stars - 1 - k) * (star_size + star_gap);
                host.fill_rect(x, top, star_size, star_size, chrome::COLOR_GOLD_WARM);
                if star_bevel > 0 {
                    host.fill_rect(x, top, star_size, star_bevel, chrome::COLOR_BRASS_LIGHT);
                }
            }
        }
    }
}
