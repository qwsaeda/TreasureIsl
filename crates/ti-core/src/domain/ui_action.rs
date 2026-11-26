//! UI action 单一事实源（`docs/process/33_ui_component_schema.md` §4）。
//!
//! schema 的 `action` 字符串、埋点用的 opaque `PickTarget.key`、场景 `on_hit` 的语义分派
//! 三处全部从 [`ACTION_TABLE`] 派生，消除「同一语义裸数字在两处独立硬编码」（S85.4）：
//! `ShellComponentHost::action_key` 由 schema 字符串查表得 key，场景由 key 反解回 action。
//!
//! key 口径 = 旧 `route_hit` 的 opaque 值（首页木牌 1 / 选角·选图 2 / 存读档槽 3 /
//! 主地图摇骰 4），保持交互零变化；VS 世界的 `attack`/`do_item`/`defend` 沿用 2/3/4
//! （B1：游戏世界留 1080，不经本表反解）；非命中动作（翻页/返回）占 8/9/10，
//! 与历史 `pick_key` 族不冲突，也不参与 `PickTarget` 挂载数据。
//! S87.1 新增 `vs_skip`/`vs_confirm`/`vs_reset` = 11/12/13：修正「`attack`/`do_item`/`defend`
//! 的 2/3/4 与 vs 实际 `on_hit` 语义（快进/确认/重打）不符」的真漂移；现有三 action
//! 的 key 与场景 `on_hit` 行为不变（S87.4 才接新 action）。

/// UI 交互动作（33 号 schema `Action` 闭集，15 项）。
///
/// 变体名与 schema 字符串一一对应，由 [`ACTION_TABLE`] 统一维护。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiAction {
    /// 首页木牌选择（`menu_select`）。
    MenuSelect,
    /// 角色选择（`character_select`）。
    CharacterSelect,
    /// 地图选择（`map_select`）。
    MapSelect,
    /// 读档（`slot_load`）。
    SlotLoad,
    /// 存档（`slot_write`）。
    SlotWrite,
    /// 帮助下一页（`msg_next`，键盘/滑动驱动）。
    MsgNext,
    /// 帮助上一页（`msg_prev`，键盘/滑动驱动）。
    MsgPrev,
    /// 主地图摇骰（`spin`）。
    Spin,
    /// 返回（`back`，键盘/滑动驱动）。
    Back,
    /// VS 攻击（`attack`）。
    Attack,
    /// VS 道具（`do_item`）。
    DoItem,
    /// VS 防御（`defend`）。
    Defend,
    /// VS 快进/跳过（`vs_skip`，S87.1；key 11，语义 = 旧 `attack` 键 2 的实际 `on_hit` 行为）。
    VsSkip,
    /// VS 确认（`vs_confirm`，S87.1；key 12，语义 = 旧 `do_item` 键 3 的实际 `on_hit` 行为）。
    VsConfirm,
    /// VS 重打/重置（`vs_reset`，S87.1；key 13，语义 = 旧 `defend` 键 4 的实际 `on_hit` 行为）。
    VsReset,
}

/// action 闭集表：`schema 字符串 ↔ variant ↔ opaque key` 的唯一出处。
///
/// 声明顺序即 [`UiAction::from_key`] 在同 key 多 variant 时的代表元：key 2 →
/// `CharacterSelect`（与 `MapSelect` 同族）、key 3 → `SlotLoad`（与 `SlotWrite` 同族）、
/// key 4 → `Spin`（与 `Defend` 不同场景，VS 世界不经此反解）。
/// 场景按 **variant 族** 分派（`CharacterSelect | MapSelect` 等），故同 key 共用不丢语义。
const ACTION_TABLE: [(&str, UiAction, u32); 15] = [
    ("menu_select", UiAction::MenuSelect, 1),
    ("character_select", UiAction::CharacterSelect, 2),
    ("map_select", UiAction::MapSelect, 2),
    ("slot_load", UiAction::SlotLoad, 3),
    ("slot_write", UiAction::SlotWrite, 3),
    ("msg_next", UiAction::MsgNext, 8),
    ("msg_prev", UiAction::MsgPrev, 9),
    ("spin", UiAction::Spin, 4),
    ("back", UiAction::Back, 10),
    ("attack", UiAction::Attack, 2),
    ("do_item", UiAction::DoItem, 3),
    ("defend", UiAction::Defend, 4),
    ("vs_skip", UiAction::VsSkip, 11),
    ("vs_confirm", UiAction::VsConfirm, 12),
    ("vs_reset", UiAction::VsReset, 13),
];

impl UiAction {
    /// schema `action` 字符串 → action（闭集外 = `None`，即该控件不可点）。
    #[must_use]
    pub fn from_schema_str(raw: &str) -> Option<Self> {
        ACTION_TABLE.iter().find(|(name, ..)| *name == raw).map(|(_, action, _)| *action)
    }

    /// action → schema `action` 字符串（写回/诊断用）。
    #[must_use]
    pub fn as_str(self) -> &'static str {
        ACTION_TABLE.iter().find(|(_, action, _)| *action == self).map_or("", |(name, ..)| *name)
    }

    /// action → opaque `PickTarget.key`（与旧 `route_hit` 口径一致）。
    #[must_use]
    pub fn key(self) -> u32 {
        ACTION_TABLE.iter().find(|(_, action, _)| *action == self).map_or(0, |(_, _, key)| *key)
    }

    /// opaque key → action 代表元（供场景 `on_hit` 按 variant 分派）。
    ///
    /// `key=0` = 无元素命中，恒 `None`；同 key 多 action 取表中首个（见 [`ACTION_TABLE`]）。
    #[must_use]
    pub fn from_key(key: u32) -> Option<Self> {
        if key == 0 {
            return None;
        }
        ACTION_TABLE
            .iter()
            .find(|(_, _, candidate)| *candidate == key)
            .map(|(_, action, _)| *action)
    }
}
