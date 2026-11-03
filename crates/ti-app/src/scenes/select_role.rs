//! 角色选择场景（S9 骨架）：输入判定 / 拾取命中 / 绘制三面全部落在本文件。
use ti_core::data;
use ti_core::domain::flow::Sfx;
use ti_core::domain::selection::Character;
use ti_core::rules::selection;

use crate::game_app::{GameInput, HitData};
use crate::scenes::chrome;
use crate::scenes::{Scene, SceneHost, SceneKind};

/// 角色数（域出口 `Character::count()` = `Character::all()` 长度，B11 单一事实源；
/// `spec/menu.md §b`）。
const ROLE_COUNT: u32 = Character::count();

/// 角色选择场景处理器（`SelectRole`）。
pub struct SelectRoleScene;

/// 读「角色 + 地图是否齐全」两个布尔（`rules::selection` 判据）。
fn selection_done(host: &dyn SceneHost) -> bool {
    let s = host.selection();
    s.has_character() && s.has_map()
}

/// v3：勾选/取消勾选某角色为**人类席**（`territory.md` §1）。
///
/// - 用 `selection::seat_mark_toggle`（可取消到 0 席——**下限 2 席在「开始对局」时校验**，
///   勾选阶段允许清空，D5 修复）；
/// - 人类席数 = `balance.selection.min_players`（缺省 2）——`players ≥ 2` 才有"多人"意义。
fn toggle_seat_mark(host: &mut dyn SceneHost, picked: Character) {
    let mut marks = host.player_seats();
    let count = Character::count();
    let sfx = selection::seat_mark_toggle(&mut marks, picked, count);
    if sfx == Sfx::Press {
        host.set_player_seats(marks);
    }
    host.play_sfx(sfx);
}

/// v3：当前勾选是否满足开局条件（人类席 `≥ min_players`）。
fn seat_selection_ready(host: &dyn SceneHost) -> bool {
    selection::seat_selection_ready(&host.player_seats(), min_players())
}

/// v3：人类席下限（`balance.selection.min_players`，缺省 2）。
fn min_players() -> u32 {
    data::balance().map_or(2, |b| b.selection.min_players)
}

impl Scene for SelectRoleScene {
    fn kind(&self) -> SceneKind {
        SceneKind::SelectRole
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool {
        match input {
            GameInput::Prev => {
                let cur = host.character().map_or(0, Character::index);
                let n = if cur == 0 { ROLE_COUNT - 1 } else { cur - 1 };
                let picked = Character::from_index(n);
                host.set_character(picked);
                host.selection_mut().character = picked;
                host.play_sfx(Sfx::Press);
                false
            }
            GameInput::Next => {
                let cur = host.character().map_or(ROLE_COUNT - 1, Character::index);
                let n = (cur + 1) % ROLE_COUNT;
                let picked = Character::from_index(n);
                host.set_character(picked);
                host.selection_mut().character = picked;
                host.play_sfx(Sfx::Press);
                false
            }
            // v3：勾选/取消勾选该角色为**人类席**（§1：人类席 2..5）。
            // 键盘路径与触屏行点击**等价**（D1 修复：v2 键盘流根本选不出多人）。
            GameInput::MarkSeat => {
                let Some(picked) = host.character() else {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                };
                toggle_seat_mark(host, picked);
                false
            }
            // v3：**开始对局** = 确认当前勾选 ⇒ 进选图（与勾选动作分离，D5 修复）。
            // 开局下限校验（`seat_selection_ready`）：人类席 < `min_players` 时拒绝开局，
            // 留在选角界面让用户补选。
            GameInput::StartMatch | GameInput::Confirm => {
                // 首次进 SelectRole 时 `character`=None（Index 只 `reset_selection` 不设角色），
                // Confirm 兜底选第 0 个角色（对齐 select_map Confirm 的 map_slot 兜底语义）
                if host.character().is_none() {
                    let Some(c) = Character::from_index(0) else {
                        host.play_sfx(Sfx::Invalid);
                        return false;
                    };
                    host.set_character(Some(c));
                }
                let Some(picked) = host.character() else {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                };
                host.selection_mut().character = Some(picked);
                // v3：人类席 < min_players ⇒ 拒绝开局（留选角界面补选；Sfx::Invalid 提示）。
                // `seat_marks` 空 = 旧单选流（1 人类 + 4 AI），豁免下限（v2 兼容路径）。
                let marks = host.player_seats();
                if !marks.is_empty() && !seat_selection_ready(host) {
                    host.play_sfx(Sfx::Invalid);
                    return false;
                }
                let done = selection_done(host);
                host.selection_mut().completed = done;
                if done {
                    let ok = host.enter_main_map();
                    if ok {
                        true
                    } else {
                        // FIND-204：数据校验失败 → 停留原场景
                        host.play_sfx(Sfx::Invalid);
                        false
                    }
                } else {
                    host.enter(SceneKind::SelectMap);
                    true
                }
            }
            GameInput::Back => {
                host.enter(SceneKind::Index);
                true
            }
            _ => false,
        }
    }

    /// S29.2：点某行（`pick_key=2` 角色行，`hit.index` = 行下标）= **勾选/取消勾选**该角色
    /// 为人类席（v3：勾选与「开始对局」分离，D5 修复）；下滑 = 返回。
    ///
    /// v2 行为 = 点行即选中并确认进选图 ⇒ 只能选 1 席 ⇒ 多人局不可达。
    fn on_hit(&mut self, host: &mut dyn SceneHost, hit: &HitData) -> bool {
        if hit.swipe == 2 {
            return host.advance(GameInput::Back);
        }
        if hit.swipe != 0 || hit.key != 2 {
            return false;
        }
        let i = usize::try_from(hit.index.max(0)).unwrap_or(0);
        let Some(picked) = Character::from_index(u32::try_from(i).unwrap_or(0)) else {
            return false;
        };
        host.set_character(Some(picked));
        host.selection_mut().character = Some(picked);
        chrome::flash_press();
        toggle_seat_mark(host, picked);
        false
    }

    /// S18.7d 声明式原语化：版面（夜空 / 左角色立绘锚 / 右 5 行选项 / 提示行）=
    /// `scenes.json` `select_role.primitives`（行名/简介静态文本 + 选中高亮 `when="selected"`
    /// 全数据）；运行时 = 按压闪影。
    ///
    /// S18.20：英雄面板首字母**退役**——英雄区改为**立绘贴图**（`image` 原语，按 item 切换，
    /// 见 `assets/ui/role_*.png`），叠在立绘上的字母（"p"）已无意义。
    fn render(&mut self, host: &mut dyn SceneHost) {
        let layout = chrome::scene_layout("select_role");
        let current = host.character().map_or(0, |c| c.index() as usize);
        chrome::render_primitives(host, "select_role", i64::try_from(current).unwrap_or(-1));
        // S29.3（B1）：右侧列表 chrome 已迁 Bevy `bevy_ui`（行位置由 taffy 算，不手写像素）⇒
        // CPU 路径跳过行的木底/选中边/按压闪影（Bevy 侧画）；未迁时照旧。
        if host.chrome_in_bevy_ui() {
            return;
        }
        // 按压闪影 / 长摁按压：当前行压暗（金底之上叠 bevel_shadow，`menu_item` 旧态）
        let held = host.press_held();
        let flashing = chrome::press_flash_active();
        if held || flashing {
            let r = layout.list.item_rect(current);
            host.fill_rect(r.x, r.y, r.w, r.h, chrome::COLOR_BEVEL_SHADOW);
        }
        // S20.2 v4 面板语言：右列角色项 = 深棕木底（scenes.json 原语 `panel_wood`）+ 黄铜细边；
        // 选中项 = 3px 黄铜边（此前整块金色填充）。
        // 飞轮 R1 ⑥-07：循环走 `ROLE_COUNT` 命名常量（与 on_input/触摸带同单源，
        // 补 doccheck2 EXP-073 band 数门禁对 render 循环字面量的盲区）。
        for i in 0..ROLE_COUNT as usize {
            let r = layout.list.item_rect(i);
            chrome::wood_panel(host, r.x, r.y, r.w, r.h);
        }
        let sel = layout.list.item_rect(current);
        chrome::selected_border(host, sel.x, sel.y, sel.w, sel.h, chrome::PANEL_SELECTED_BORDER_PX);
    }
}
