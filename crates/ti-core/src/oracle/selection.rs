//! oracle：S3.4 差分基准，测试专用。
//!
//! 直译 `docs/spec/selection.md` 的角色/地图选择状态机：
//! - 角色枚举 [`Character`]：5 角色（panda/tichy/kidd/drake/katharina，`sub_3FE84` 控件注入器 11 槽位）；
//! - 地图槽位枚举 [`MapSlot`]：3 地图（glmapm01/02/03，`sub_402D8` `CreateFileW` 探测）；
//! - 选择状态 [`SelectionState`]：已选角色 + 已选地图 + 完成标志；
//! - [`oracle_select_role`]：选择角色（未选地图前只记录角色）；
//! - [`oracle_select_map`]：选择地图（未解锁 → `Sfx::Invalid`；已解锁 + 角色已选 → `completed=true`）；
//! - [`oracle_selection_flow`]：5×3=15 组合的完整流程脚本。
//!
//! 事件码（`docs/spec/selection.md` §d，`sub_41354`/`sub_3F4E4`/`sub_4085C` 实测）：
//! - 13 = 确定/Enter（选择角色/地图，真实）；
//! - 27 = 返回（子场景回 index，真实）；
//! - 37 = 无效操作（**推测** invalid.wav）；
//! - 38 = 上一个角色/地图（真实，`sub_4085C` 行 52）；
//! - 40 = 下一个角色/地图（真实，`sub_4085C` 行 37）。

use crate::oracle::menu::Sfx;

/// 角色枚举（`evidence/S3.2.log` 字符串清单 行 15/18/20：`start_*`/`face_*`/`Player_*` × 5）。
///
/// 资源映射（`sub_3FE84` 行 61-84）：
/// - `Panda` → `face_panda.pcx` + `Player_panda.pcx` + `start_panda.pcx`；
/// - `Tichy` → `face_tichy.pcx` + `Player_tichy.pcx` + `start_tichy.pcx`；
/// - `Kidd` → `face_kidd.pcx` + `Player_kidd.pcx` + `start_kidd.pcx`；
/// - `Drake` → `face_drake.pcx` + `Player_drake.pcx` + `start_drake.pcx`；
/// - `Katharina` → `face_katharina.pcx` + `Player_katharina.pcx` + `start_katharina.pcx`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Character {
    /// 熊猫海盗（panda）。
    Panda,
    /// 蒂奇（tichy）。
    Tichy,
    /// 基德（kidd）。
    Kidd,
    /// 德雷克（drake）。
    Drake,
    /// 凯瑟琳娜（katharina）。
    Katharina,
}

impl Character {
    /// 全部角色（按枚举声明顺序，diff 测试遍历用）。
    #[must_use]
    pub const fn all() -> [Character; 5] {
        [Self::Panda, Self::Tichy, Self::Kidd, Self::Drake, Self::Katharina]
    }

    /// 角色索引（0..=4，枚举声明顺序；存档 `character` 字段 1..=5 映射）。
    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::Panda => 0,
            Self::Tichy => 1,
            Self::Kidd => 2,
            Self::Drake => 3,
            Self::Katharina => 4,
        }
    }

    /// 索引 → 角色（0..=4，越界 → `None`，供壳层选角循环 / 存档回读）。
    #[must_use]
    pub const fn from_index(i: u32) -> Option<Self> {
        match i {
            0 => Some(Self::Panda),
            1 => Some(Self::Tichy),
            2 => Some(Self::Kidd),
            3 => Some(Self::Drake),
            4 => Some(Self::Katharina),
            _ => None,
        }
    }

    /// 角色可读名称（资源映射用）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Panda => "panda",
            Self::Tichy => "tichy",
            Self::Kidd => "kidd",
            Self::Drake => "drake",
            Self::Katharina => "katharina",
        }
    }
}

/// 地图槽位枚举（`sub_402D8` 行 72/89/103：`glmapm01/02/03.Map` `CreateFileW` 探测）。
///
/// 槽位参数（`sub_406DC` 写入 `a1+13/16/19`，`000402D8_sub_402D8.c` 行 79-87/95-100/109-114）：
/// - `M01` → 编号 1、X 73、Y 100（行 79-87）；
/// - `M02` → 编号 2、X 67、Y 64（行 95-100）；
/// - `M03` → 编号 3、X 117、Y 81（行 109-114）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapSlot {
    /// 地图 1（`glmapm01.Map`，`data\glmapm01.Map` 直读探测，行 76-88）。
    M01,
    /// 地图 2（`glmapm02.Map`，`data\glmapm02.Map` 直读探测，行 92-101）。
    M02,
    /// 地图 3（`glmapm03.Map`，`data\glmapm03.Map` 直读探测，行 106-115）。
    M03,
}

impl MapSlot {
    /// 全部地图槽位（按枚举声明顺序，diff 测试遍历用）。
    #[must_use]
    pub const fn all() -> [MapSlot; 3] {
        [Self::M01, Self::M02, Self::M03]
    }

    /// 地图编号（`dword_7FB6C` 值，`sub_2077C` 行 142-155 switch 用）。
    #[must_use]
    pub const fn index(self) -> u32 {
        match self {
            Self::M01 => 1,
            Self::M02 => 2,
            Self::M03 => 3,
        }
    }

    /// 0 起索引（0..=2，枚举声明顺序；壳层选图循环用）。
    #[must_use]
    pub const fn zero_index(self) -> u32 {
        match self {
            Self::M01 => 0,
            Self::M02 => 1,
            Self::M03 => 2,
        }
    }

    /// 0 起索引 → 地图（0..=2，越界 → `None`，供壳层选图循环 / 存档回读）。
    #[must_use]
    pub const fn from_index(i: u32) -> Option<Self> {
        match i {
            0 => Some(Self::M01),
            1 => Some(Self::M02),
            2 => Some(Self::M03),
            _ => None,
        }
    }

    /// 地图可读名称（`sub_2077C` 行 145/149/153 资源字符串）。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::M01 => "glmapm01.Map",
            Self::M02 => "glmapm02.Map",
            Self::M03 => "glmapm03.Map",
        }
    }
}

/// 选择状态：角色 + 地图 + 完成标志。
///
/// - `character`：已选角色（`None` = 尚未选择）；
/// - `map_slot`：已选地图（`None` = 尚未选择）；
/// - `completed`：选择完成（角色 + 地图均已选定）→ 触发 `main.Map`（`sub_2077C`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionState {
    /// 已选角色（`None` = 未选）。
    pub character: Option<Character>,
    /// 已选地图（`None` = 未选）。
    pub map_slot: Option<MapSlot>,
    /// 选择完成标志（角色 + 地图均选定 → `true`，触发 `main.Map`）。
    pub completed: bool,
}

impl SelectionState {
    /// 新建空状态（未选角色、未选地图、未完成）。
    #[must_use]
    pub const fn new() -> Self {
        Self { character: None, map_slot: None, completed: false }
    }

    /// 是否已选角色。
    #[must_use]
    pub const fn has_character(self) -> bool {
        self.character.is_some()
    }

    /// 是否已选地图。
    #[must_use]
    pub const fn has_map(self) -> bool {
        self.map_slot.is_some()
    }
}

impl Default for SelectionState {
    fn default() -> Self {
        Self::new()
    }
}

/// oracle：选择角色。
///
/// 规则（直译 `docs/spec/selection.md` §a/§d）：
/// - 记录角色（覆盖之前选择）；
/// - 若地图已选（`map_slot` 非 `None`）且角色已选 → `completed=true`（触发 `main.Map`）；
/// - 音效：`Press`（有效选择，`music\press.wav`，**推测**框架层触发）。
///
/// 返回 `(新状态, 可选音效)`。
#[must_use]
pub fn oracle_select_role(state: SelectionState, ch: Character) -> (SelectionState, Option<Sfx>) {
    let mut next = state;
    next.character = Some(ch);
    // 地图已选 + 角色刚选 → 完成（角色后选的情形）
    if next.has_map() {
        next.completed = true;
    }
    (next, Some(Sfx::Press))
}

/// oracle：选择地图。
///
/// 规则（直译 `docs/spec/selection.md` §b/§d）：
/// - 未解锁地图（`m` 不在 `unlocked` 列表）→ `completed=false` + `Sfx::Invalid`（`music\invalid.wav`，**推测**）；
/// - 已解锁地图：
///   - 记录地图（覆盖之前选择）；
///   - 若角色已选（`character` 非 `None`）且地图已选 → `completed=true`（触发 `main.Map`）；
///   - 音效：`Press`（有效选择）。
///
/// 返回 `(新状态, 可选音效)`。
#[must_use]
pub fn oracle_select_map(
    state: SelectionState,
    m: MapSlot,
    unlocked: &[MapSlot],
) -> (SelectionState, Option<Sfx>) {
    // 未解锁 → 无效（状态不变，`completed` 保持 `false`）
    if !unlocked.contains(&m) {
        return (state, Some(Sfx::Invalid));
    }

    let mut next = state;
    next.map_slot = Some(m);
    // 角色已选 + 地图刚选 → 完成
    if next.has_character() {
        next.completed = true;
    }
    (next, Some(Sfx::Press))
}

/// oracle：完整选择流程脚本（5 角色 × 3 地图 = 15 组合）。
///
/// 遍历 `Character::all()` × `MapSlot::all()`，每个组合从空状态出发：
/// 1. `oracle_select_role(state, ch)` → 选角色；
/// 2. `oracle_select_map(state, m, unlocked)` → 选地图（`unlocked` = 全部 3 地图）；
/// 3. 断言 `completed=true`（角色 + 地图均选定 → 触发 `main.Map`）。
///
/// 返回 15 个 `(角色, 地图)` 组合。
///
/// # Panics
///
/// 任一组合的 `completed` 不为 `true` 时 panic（不应发生，因为已解锁地图
/// 选择后角色 + 地图均选定即完成）。
#[must_use]
pub fn oracle_selection_flow() -> Vec<(Character, MapSlot)> {
    let unlocked = MapSlot::all();
    let mut flows: Vec<(Character, MapSlot)> = Vec::new();

    for ch in Character::all() {
        for m in MapSlot::all() {
            let state0 = SelectionState::new();
            let (state1, _) = oracle_select_role(state0, ch);
            let (state2, _) = oracle_select_map(state1, m, &unlocked);
            assert!(
                state2.completed,
                "组合 ({ch:?}, {m:?}) 应完成，实际 completed={}",
                state2.completed
            );
            flows.push((ch, m));
        }
    }

    flows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_state_new() {
        let s = SelectionState::new();
        assert!(!s.has_character());
        assert!(!s.has_map());
        assert!(!s.completed);
    }

    #[test]
    fn select_role_sets_character() {
        let s = SelectionState::new();
        let (next, sfx) = oracle_select_role(s, Character::Panda);
        assert_eq!(next.character, Some(Character::Panda));
        assert_eq!(next.map_slot, None);
        assert!(!next.completed);
        assert_eq!(sfx, Some(Sfx::Press));
    }

    #[test]
    fn select_map_unlocked_sets_map() {
        let s = SelectionState::new();
        let (s, _) = oracle_select_role(s, Character::Kidd);
        let unlocked = [MapSlot::M01, MapSlot::M02];
        let (next, sfx) = oracle_select_map(s, MapSlot::M01, &unlocked);
        assert_eq!(next.map_slot, Some(MapSlot::M01));
        assert_eq!(next.character, Some(Character::Kidd));
        assert!(next.completed);
        assert_eq!(sfx, Some(Sfx::Press));
    }

    #[test]
    fn select_map_locked_invalid() {
        let s = SelectionState::new();
        let (s, _) = oracle_select_role(s, Character::Drake);
        let unlocked = [MapSlot::M01];
        let (next, sfx) = oracle_select_map(s, MapSlot::M03, &unlocked);
        assert_eq!(next.map_slot, None, "未解锁地图不应记录");
        assert_eq!(next.character, Some(Character::Drake), "角色应保留");
        assert!(!next.completed);
        assert_eq!(sfx, Some(Sfx::Invalid));
    }

    #[test]
    fn selection_flow_15_combinations() {
        let flows = oracle_selection_flow();
        assert_eq!(flows.len(), 15, "5 角色 × 3 地图 = 15 组合");
        for (ch, m) in &flows {
            let s0 = SelectionState::new();
            let (s1, _) = oracle_select_role(s0, *ch);
            let (s2, _) = oracle_select_map(s1, *m, &MapSlot::all());
            assert!(s2.completed, "({ch:?}, {m:?}) 应完成");
            assert_eq!(s2.character, Some(*ch));
            assert_eq!(s2.map_slot, Some(*m));
        }
    }
}
