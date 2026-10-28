//! 存读档场景（S9 骨架）：读档/写档判定 + 槽位列表绘制。
use ti_core::domain::flow::Sfx;
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::MapSlot;

use crate::game_app::{GameInput, HitData};
use crate::i18n;
use crate::scenes::chrome;
use crate::scenes::{Scene, SceneHost, SceneKind};

/// 存读档场景处理器（`Saveload`）。
pub struct SaveloadScene;

/// N4：写档版本号固定 = v3（3，S21.11 加 `boat_level` 船等级；v2 旧档读回 = 小舢板缺省 1），
/// 与 `domain::persistence::REC_MAGIC` 配套的版本标识；不再读 `balance().save.v2_size`
/// （那是槽字节大小，与版本号语义无关）。
const WRITE_VERSION: u32 = 3;

impl Scene for SaveloadScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Saveload
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::LoadSave(slot) => {
                let Some(data) = host.save_slots().get(slot).cloned() else {
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
                    // S21.11：船等级逐档写入（全局账号级；旧档缺此字段 ⇒ 读回 = 1 小舢板）。
                    boat_level: host.boat_level(),
                    v4: host.save_v4_ext().map(Box::new),
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

    /// S29.2：点有档的槽（`pick_key=3`，`hit.index` = 槽位）= 读档；点空槽 = 存档；下滑 = 关闭。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        if hit.swipe == 2 {
            return host.advance(GameInput::SaveloadClose);
        }
        if hit.swipe != 0 || hit.key != 3 {
            return false;
        }
        let slot = usize::try_from(hit.index.max(0)).unwrap_or(0);
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
        // S29.3（B1）：槽位列表 chrome 已迁 Bevy `bevy_ui`（行位置由 taffy 算，不手写像素）⇒
        // CPU 路径跳过槽位木底/选中边/按压闪影/槽文案（Bevy 侧画）；未迁时照旧。
        if host.chrome_in_bevy_ui() {
            return;
        }
        // 按压闪影 / 长摁按压：光标行压暗（金底之上叠 bevel_shadow）
        let held = host.press_held();
        let flashing = chrome::press_flash_active();
        if (held || flashing) && cursor < slots.len() {
            let r = layout.list.item_rect(cursor);
            host.fill_rect(r.x, r.y, r.w, r.h, chrome::COLOR_BEVEL_SHADOW);
        }
        // S20.2 v4 面板语言：槽位 = 深棕木底（scenes.json 原语 `panel_wood`）+ 黄铜细边
        // （统一走 `chrome::wood_panel`，避免每个面板各画一套边）。
        for i in 0..slots.len() {
            let r = layout.list.item_rect(i);
            chrome::wood_panel(host, r.x, r.y, r.w, r.h);
        }
        // 选中项 = 3px 黄铜边（此前整块金色填充，刺眼且抢戏）。
        if cursor < slots.len() {
            let r = layout.list.item_rect(cursor);
            chrome::selected_border(host, r.x, r.y, r.w, r.h, chrome::PANEL_SELECTED_BORDER_PX);
        }
        for (i, s) in slots.iter().enumerate() {
            let r = layout.list.item_rect(i);
            let ty = r.y + (r.h - host.shell().glyph_px()) / 2; // QA009：字形高按活跃字体（TTF 44px / 位图 12px）
            let has_str =
                if s.version > 0 { i18n::sget("slot.has") } else { i18n::sget("slot.empty") };
            let idx = i + 1;
            let slot_str = i18n::sgetf("slot.label_fmt", &[&idx, &has_str]);
            chrome::draw_body_text(host, &slot_str, layout.list.text_x, ty);
            // 第二行摘要（`slot_summary_fmt`）；空档 = `empty_line`。
            // S18.7d：摘要降提示档（TTF 28px / 位图 8px，`draw_hint_text`）——v3 行高 224 下
            // 双行 44px 会挤叠（EXP-105 视觉对账），28px 次级墨 + 上移保持层级。
            // 飞轮 R1 ③-15：摘要 y 偏移走 `list.summary_dy` 数据键。
            let summary = if s.version > 0 {
                i18n::sgetf(
                    "scene.saveload.slot_summary_fmt",
                    &[&s.character, &s.gold, &s.rng_steps],
                )
            } else {
                i18n::sget("scene.saveload.empty_line").to_string()
            };
            host.draw_hint_text(
                &summary,
                layout.list.text_x,
                ty + layout.list.summary_dy,
                chrome::COLOR_SECONDARY,
            );
            if s.version > 0 {
                let g = i18n::sgetf("hud.gold_fmt", &[&s.gold]);
                chrome::draw_body_text(host, &g, layout.list.gold_x, ty);
            }
        }
    }
}
