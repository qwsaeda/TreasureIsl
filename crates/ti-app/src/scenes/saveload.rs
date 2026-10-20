//! 存读档场景（S9 骨架）：读档/写档判定 + 槽位列表绘制。
use ti_core::domain::flow::Sfx;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::MapSlot;

use crate::game_app::GameInput;
use crate::i18n;
use crate::scenes::chrome;
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
                    unlocked_mask: host.unlocked_mask(),
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
                // QA031：统一走 `chrome::scene_layout`（scenes.json 单一读点，契约 4）
                let scene = chrome::scene_layout("saveload");
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
                chrome::flash_press();
                // S15.6 QB136：三态卡（has/empty/cursor）——点选先落光标（卡片描边态），再推进
                host.set_saveload_cursor(slot);
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

    /// S18.7d 声明式原语化：版面（夜空 / 左立绘锚 / 中槽列表 / 右元数据栏 / 提示行）=
    /// `scenes.json` `saveload.primitives`（光标高亮 = `when="selected"` 行，QB136 三态收编：
    /// 有档/空档区分改由运行时文案 + 金字号列承担）；运行时 = 按压闪影 +
    /// 每档文案（`slot.label_fmt`/`slot_summary_fmt`/`hud.gold_fmt`，`list.text_x`/`gold_x` 数据槽）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        // P1-3：坐标/文案硬编码改读 `ti_core::data::scenes()`（scenes.json 字段已建）
        let layout = chrome::scene_layout("saveload");
        let slots: Vec<SaveSlot> = host.save_slots().to_vec();
        let cursor = host.saveload_cursor();
        chrome::render_primitives(host, "saveload", i64::try_from(cursor).unwrap_or(-1));
        // 按压闪影 / 长摁按压：光标行压暗（金底之上叠 bevel_shadow）
        let held = host.press_held();
        let flashing = chrome::press_flash_active();
        if (held || flashing) && cursor < slots.len() {
            let r = layout.list.item_rect(cursor);
            host.fill_rect(r.x, r.y, r.w, r.h, chrome::COLOR_BEVEL_SHADOW);
        }
        for (i, s) in slots.iter().enumerate() {
            let r = layout.list.item_rect(i);
            let ty = r.y + (r.h - host.shell().glyph_px()) / 2; // QA009：字形高按活跃字体（TTF 44px / 位图 12px）
            let has_str =
                if s.version > 0 { i18n::sget("slot.has") } else { i18n::sget("slot.empty") };
            let idx = i + 1;
            let slot_str = i18n::sgetf("slot.label_fmt", &[&idx, &has_str]);
            chrome::draw_body_text(host, &slot_str, layout.list.text_x, ty);
            // 第二行摘要：有档 = 角色/金/回合（`slot_summary_fmt`）；空档 = `empty_line`。
            // S18.7d：摘要降提示档（TTF 28px / 位图 8px，`draw_hint_text`）——v3 行高 224 下
            // 双行 44px 会挤叠（EXP-105 视觉对账），28px 次级墨 + 上移保持层级。
            let summary = if s.version > 0 {
                i18n::sgetf(
                    "scene.saveload.slot_summary_fmt",
                    &[&s.character, &s.gold, &s.rng_steps],
                )
            } else {
                i18n::sget("scene.saveload.empty_line").to_string()
            };
            host.draw_hint_text(&summary, layout.list.text_x, ty + 52, chrome::COLOR_SECONDARY);
            if s.version > 0 {
                let g = i18n::sgetf("hud.gold_fmt", &[&s.gold]);
                chrome::draw_body_text(host, &g, layout.list.gold_x, ty);
            }
        }
    }
}
