//! 地图选择场景（S9 骨架）：输入判定 / 触摸手势 / 绘制三面全部落在本文件。
use ti_core::domain::flow::Sfx;
use ti_core::domain::selection::MapSlot;

use crate::game_app::GameInput;
use crate::scenes::chrome;
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 地图槽数（`MapSlot::all()` 长度，`spec/menu.md §c`）。
const MAP_COUNT: u32 = 3;

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

    /// 点某行 = 选中该地图并确认；下滑 = 返回。
    ///
    /// 未解锁地图点按由 `on_input` 的 `Confirm` 分支判 `Sfx::Invalid`。
    #[allow(clippy::cast_precision_loss)]
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                // N3：命中带与渲染 `item_rect` 同公式（scenes.json select_map 段），正逆成对
                let scene = chrome::scene_layout("select_map");
                let bands = layout::scene_list_bands(
                    scene.list.x as f32,
                    scene.list.x as f32 + scene.list.w as f32,
                    scene.list.y as f32,
                    scene.list.row_h as f32,
                    scene.list.item_h as f32,
                    scene.list.highlight_offset as f32,
                    MAP_COUNT as usize,
                );
                let Some(i) = layout::scene_list_index_at(&bands, sample.x, sample.y) else {
                    return false;
                };
                host.set_map_slot(MapSlot::from_index(u32::try_from(i).unwrap_or(0)));
                chrome::flash_press();
                host.advance(GameInput::Confirm)
            }
            Gesture::SwipeDown => host.advance(GameInput::Back),
            _ => false,
        }
    }

    /// 地图列表 + 选中高亮 + 锁定标记 + 导航提示（v2 羊皮纸/木纹/金色系）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let layout = chrome::scene_layout("select_map");
        chrome::scene_shell(host, "select_map", "scene.select_map.title");
        let current = host.map_slot().map_or(0, |m| m.zero_index() as usize);
        let held = host.press_held();
        let flashing = chrome::press_flash_active();
        for (i, m) in MapSlot::all().iter().enumerate() {
            let r = layout.list.item_rect(i);
            let locked = !host.map_unlocked(*m);
            let chosen = i == current && !locked;
            chrome::menu_item(host, r.x, r.y, r.w, r.h, chosen, chosen && (held || flashing));
            let ty = r.y + (r.h - host.shell().glyph_px()) / 2; // QA009：字形高按活跃字体（TTF 44px / 位图 12px）
            chrome::draw_body_text(host, m.label(), layout.list.text_x, ty);
            if locked {
                // UI-10：未解锁项 = 深色「锁」块（压按钮右内侧，钳进画布内不溢出）。
                // QA033：锁块 x 只走 `scenes.json` `locked.text_x`（1080 数据驱动，契约 4）；
                // 键缺失/非正 = 不画锁块（旧 176 时代休眠回退值 130 已删，契约 1）。
                let lock_x = layout.extra_scalar.get("locked.text_x").copied().unwrap_or(0);
                if lock_x > 0 {
                    // 契约 1（`08_render_arch.md`）：钳制基准用呈现坐标系常量，
                    // 不得出现 legacy 176 字面量。
                    // 锁标右界按**卡片右缘**（`item_rect` 的 x+w）而非屏幕宽推导，
                    // 按钮变窄时锁标随卡片收敛，不会溢出卡片（F12）
                    let card_right = r.x + r.w;
                    let lock_w = (card_right - 6 - lock_x).min(40);
                    host.fill_rect(lock_x, ty, lock_w.max(4), 12, crate::menu_shell::COLOR_WOOD);
                }
            }
        }
        chrome::scene_hint(host, "select_map", "scene.select_map.nav_hint");
    }
}
