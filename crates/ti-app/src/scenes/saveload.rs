//! 存读档场景（S9 骨架）：读档/写档判定 + 槽位列表绘制。
use ti_core::domain::flow::Sfx;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::MapSlot;

use crate::game_app::GameInput;
use crate::i18n;
use crate::menu_shell::{COLOR_BG, COLOR_TITLE_BAR};
use crate::scenes::{Scene, SceneHost, SceneKind};
use crate::touch::{TouchSample, layout};
use ti_platform::settings::Gesture;

/// 存读档场景处理器（`Saveload`）。
pub struct SaveloadScene;

/// N4：写档版本号固定 = v2（2），与 `domain::persistence::REC_MAGIC` 配套的
/// v2 格式标识；不再读 `balance().save.v2_size`（那是槽字节大小，与版本号语义无关）。
const WRITE_VERSION: u32 = 2;

impl Scene for SaveloadScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Saveload
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::LoadSave(slot) => {
                let Some(data) = host.save_slots().get(slot).copied() else {
                    return false;
                };
                host.restore_save(data);
                host.play_sfx(Sfx::Press);
                true
            }
            GameInput::WriteSave(slot) => {
                let character = host.character().map_or(1, |c| c.index() + 1);
                let map_slot = host.map_slot().map_or(0, MapSlot::zero_index);
                let gold = host.gold();
                // N5：网格边长走 `terrain_grid_w`（P0-4 后地图尺寸走 terrain_map），
                // 无坐标时回退 0，无地形资源时 `terrain_grid_w` 本身回退 `POSITION_GRID_W`（20）。
                let grid_w = host.terrain_grid_w();
                let position = host.map_pos().map_or(0, |p| p.index(grid_w));
                // P1-8（S1）：写档捕获当前 RNG 步数（精确续流，读档后随机流可复现）
                let rng_steps = host.rng_steps();
                let data = SaveSlot {
                    version: WRITE_VERSION,
                    character,
                    gold,
                    map_slot,
                    position,
                    rng_steps,
                };
                if host.save_slots().len() > slot {
                    host.write_save_slot(slot, data);
                    host.play_sfx(Sfx::Press);
                }
                false
            }
            GameInput::SaveloadClose | GameInput::Back => {
                host.enter(SceneKind::Index);
                true
            }
            _ => false,
        }
    }

    /// 点有档的槽 = 读档；点空槽 = 存档；下滑 = 关闭。
    #[allow(clippy::cast_precision_loss)]
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        match sample.gesture {
            Gesture::Tap => {
                // N3：命中带与渲染 `item_rect` 同公式（scenes.json saveload 段），正逆成对
                let scene = ti_core::data::scenes()
                    .ok()
                    .and_then(|s| s.get("saveload"))
                    .cloned()
                    .unwrap_or_default();
                let bands = layout::slot_bands(
                    scene.list.x as f32,
                    scene.list.x as f32 + scene.list.w as f32,
                    scene.list.y as f32,
                    scene.list.row_h as f32,
                    scene.list.item_h as f32,
                    scene.list.highlight_offset as f32,
                    host.save_slots().len(),
                );
                let Some(slot) = layout::slot_band_index_at(&bands, sample.x, sample.y) else {
                    return false;
                };
                // 有档（version > 0）→ 读档；空槽 → 存档
                let has = host.save_slots().get(slot).is_some_and(|s| s.version > 0);
                if has {
                    host.advance(GameInput::LoadSave(slot))
                } else {
                    host.advance(GameInput::WriteSave(slot))
                }
            }
            Gesture::SwipeDown => host.advance(GameInput::SaveloadClose),
            _ => false,
        }
    }

    /// 槽位列表 + 每档金/空满标记 + 导航提示。
    fn render(&mut self, host: &mut dyn SceneHost) {
        host.clear_frame(COLOR_BG);
        // P1-3：坐标/文案硬编码改读 `ti_core::data::scenes()`（scenes.json 字段已建）
        let layout = ti_core::data::scenes()
            .ok()
            .and_then(|s| s.get("saveload"))
            .cloned()
            .unwrap_or_default();
        host.fill_rect(0, 0, 176, layout.title.h, COLOR_TITLE_BAR);
        host.draw_text(i18n::sget("scene.saveload.title"), layout.title.x, layout.title.y);
        let slots: Vec<SaveSlot> = host.save_slots().to_vec();
        for (i, s) in slots.iter().enumerate() {
            let r = layout.list.item_rect(i);
            let has = s.version > 0;
            let has_str = if has { i18n::sget("slot.has") } else { i18n::sget("slot.empty") };
            let idx = i + 1;
            let slot_str = i18n::sgetf("slot.label_fmt", &[&idx, &has_str]);
            host.draw_text(&slot_str, r.x, r.y);
            if has {
                let g = i18n::sgetf("hud.gold_fmt", &[&s.gold]);
                let gold_x = layout.list.gold_x;
                host.draw_text(&g, gold_x, r.y);
            }
        }
        host.draw_text(i18n::sget("scene.saveload.nav_hint"), layout.hint.x, layout.hint.y);
    }
}
