//! 战斗场景（S9 骨架）：回合推进 / 战斗面板绘制。
use ti_core::domain::combat::{Side, hp_fill_pct};

use crate::game_app::{GameInput, HitData};
use crate::i18n;
use crate::scenes::chrome;
use crate::scenes::chrome::{COLOR_DANGER, COLOR_PLAYER};
use crate::scenes::{Scene, SceneHost, SceneKind};

/// 战斗场景处理器（`Vs`）。
pub struct VsScene;

impl Scene for VsScene {
    fn kind(&self) -> SceneKind {
        SceneKind::Vs
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Attack => {
                host.battle_step();
                false
            }
            GameInput::Back => {
                // S28.6：副本战结束/中途退出 = 回主地图（落格续局，territory.md §4）；
                // BOSS 战 = 回主菜单（`spec/battle.md §d`，状态码 25→0）
                if host.in_dungeon_battle() {
                    host.enter(SceneKind::MainMap);
                } else {
                    host.enter(SceneKind::Index);
                }
                true
            }
            _ => false,
        }
    }

    /// S29.2：点任意处（`key=0`）/ 上·左·右滑 = 攻击一回合；下滑 = 退出战斗。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        match hit.swipe {
            2 => host.advance(GameInput::Back),
            _ => host.advance(GameInput::Attack),
        }
    }

    /// 战斗面板（双方 HP 进度条 + 回合 + 结果，v2 羊皮纸/木纹/金色系）。
    fn render(&mut self, host: &mut dyn SceneHost) {
        // S27.7 选型期：环境变量 `TI_VS_PROTO=a|b|c|d|e` ⇒ 走 5 套候选版式原型渲染
        // （`scenes::vs_proto`）；未设置 = 正式版式（旧布局不变）。
        if let Some(variant) = crate::scenes::vs_proto::selected_variant() {
            crate::scenes::vs_proto::render(host, variant);
            return;
        }
        let layout = chrome::scene_layout("vs");
        // S18.7f：vs 版面 100% 数据驱动 = `scenes.json` `vs.primitives`（夜空底 + 星点 +
        // 血条槽底 + 竞技区 + 标题 + 提示 + 木框；`chrome::scene_shell`/`bar_slot`/
        // `scene_hint` 的代码外壳已全部退役）。运行时内容 = HP 填充 + 回合/结果文本。
        chrome::render_primitives(host, "vs", -1);
        if let Some(b) = host.battle_snapshot() {
            let ph_bar = layout.extra.get("player_hp").copied().unwrap_or_default();
            let bh_bar = layout.extra.get("boss_hp").copied().unwrap_or_default();
            let rd_pos = layout.extra.get("rounds").copied().unwrap_or_default();
            let oc_pos = layout.extra.get("outcome").copied().unwrap_or_default();
            // EXP-085：用显示 HP（平滑插值）渲染血条，而非实际 HP（跳变）。
            // S15.6 QA141：填充宽走 `ti_core::domain::combat::hp_fill_pct`（显示值钳制 + 百分比折算），
            // 血条宽度永不超过槽宽（旧直接乘除在插值过冲时越槽，QA140 判据域）。
            let ph_w = ph_bar.w
                * i32::try_from(hp_fill_pct(host.displayed_player_hp(), b.player_hp_max))
                    .unwrap_or(0)
                / 100;
            let bh_w = bh_bar.w
                * i32::try_from(hp_fill_pct(host.displayed_boss_hp(), b.boss_hp_max)).unwrap_or(0)
                / 100;
            // 血条语义 = 08_layout_design §2 L52-53（S15.6 B1 裁决②）：玩家青绿 COLOR_PLAYER /
            // BOSS 血红 COLOR_DANGER（旧实现按 §4.4 表互置 + S14 色板，QA003/QA038）。
            // 槽底 = JSON 原语（`vs.primitives`），此处只画运行时 HP 填充。
            // S26.3：填充 + 黄铜描边 + 名字标签 + 数值（消除「裸色块」观感；
            // 文案键 `scene.vs.hp_*`，`{0}/{1}` 占位 = 当前/上限）。
            let theme = host.theme().clone();
            let brass = theme.by_name("brass").unwrap_or(chrome::COLOR_GOLD_WARM);
            let pl_label = i18n::sget("scene.vs.hp_player");
            let bo_label = i18n::sget("scene.vs.hp_boss");
            let pl_num = i18n::sgetf("scene.vs.hp_fmt", &[&b.player_hp, &b.player_hp_max]);
            let bo_num = i18n::sgetf("scene.vs.hp_fmt", &[&b.boss_hp, &b.boss_hp_max]);
            for (bar, label, num, right_align) in
                [(&ph_bar, pl_label, pl_num, false), (&bh_bar, bo_label, bo_num, true)]
            {
                let fill_w = if bar == &ph_bar { ph_w } else { bh_w };
                let fill_color = if bar == &ph_bar { COLOR_PLAYER } else { COLOR_DANGER };
                host.fill_rect(bar.x, bar.y, fill_w, bar.h, fill_color);
                // 黄铜描边（4 边 2px，内嵌槽沿）
                host.fill_rect(bar.x, bar.y, bar.w, 2, brass);
                host.fill_rect(bar.x, bar.y + bar.h - 2, bar.w, 2, brass);
                host.fill_rect(bar.x, bar.y, 2, bar.h, brass);
                host.fill_rect(bar.x + bar.w - 2, bar.y, 2, bar.h, brass);
                // 标签（槽上沿；右对齐 = BOSS 侧贴槽右缘）
                let ly = bar.y - 32;
                if right_align {
                    host.draw_hint_text(
                        label,
                        bar.x + bar.w - host.shell().hint_text_width_px(label),
                        ly,
                        chrome::COLOR_TEXT_ON_PARCH,
                    );
                } else {
                    host.draw_hint_text(label, bar.x, ly, chrome::COLOR_TEXT_ON_PARCH);
                }
                // 数值（槽内右缘，留 8px 内缩）
                host.draw_hint_text(
                    &num,
                    bar.x + bar.w - 8 - host.shell().hint_text_width_px(&num),
                    bar.y + (bar.h - host.shell().hint_glyph_px()) / 2,
                    chrome::COLOR_TEXT_ON_PARCH,
                );
            }
            let rd = i18n::sgetf("scene.vs.rounds_fmt", &[&b.rounds]);
            // QA141：回合/结果/空态文本 44px 屏心居中（scenes.json rounds/outcome/empty x=540 = 居中语义）
            let rd_x = chrome::centered_body_x(host, &rd);
            chrome::draw_body_text(host, &rd, rd_x, rd_pos.y);
            match b.outcome {
                Some(Side::Player) => {
                    let text = i18n::sget("scene.vs.victory");
                    let text_x = chrome::centered_body_x(host, text);
                    chrome::draw_body_text(host, text, text_x, oc_pos.y);
                }
                Some(Side::Enemy) => {
                    let text = i18n::sget("scene.vs.defeat");
                    let text_x = chrome::centered_body_x(host, text);
                    chrome::draw_body_text(host, text, text_x, oc_pos.y);
                }
                None => {
                    let aw = layout.extra.get("empty").copied().unwrap_or_default();
                    let text = i18n::sget("scene.vs.awaiting");
                    let text_x = chrome::centered_body_x(host, text);
                    chrome::draw_body_text(host, text, text_x, aw.y);
                }
            }
        } else {
            let em = layout.extra.get("empty").copied().unwrap_or_default();
            let text = i18n::sget("scene.vs.empty");
            let text_x = chrome::centered_body_x(host, text);
            chrome::draw_body_text(host, text, text_x, em.y);
        }
    }
}
