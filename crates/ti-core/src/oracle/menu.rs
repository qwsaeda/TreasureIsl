//! oracle：S3.3 差分基准，测试专用。
//!
//! 直译 `docs/spec/menu.md` 的主菜单事件状态机：
//! - 场景枚举 [`MenuScene`]：8 个可到达状态（Intro/Index/SelectRole/SelectMap/Saveload/Option/Help/Exit）；
//! - 事件枚举 [`MenuEvent`]：`Click(u32)`（事件码）/ `Invalid` / `Quit`；
//! - [`oracle_menu_transition`]：同一 (场景, 事件) 产生同一 (新场景, Option<`Sfx`>)，逐位基准；
//! - [`oracle_reachable_menu`]：从 Intro/Index 出发的可达场景集。
//!
//! 事件码（`docs/spec/menu.md` §a，`saveload` 实测）：
//! - 13 = 确定/Enter（菜单项选择）；
//! - 27 = 返回（子场景回 index）；
//! - 37 = 无效操作（**推测** invalid.wav 触发）；
//! - 38 = 上一页；40 = 下一页；
//! - `Click(u32)` 参数 = 上述事件码，其中 13 的「选中菜单项」由 `MenuScene` 决定（Index 场景下 case 0..=5）。
//!
//! 事件码 0/1/2 为**推测**占位码（spec §a 标注），对应菜单项 case 0..=5：
//! - `Click(0)` = 新游戏 → `SelectRole`；
//! - `Click(1)` = 继续 → `Saveload`；
//! - `Click(2)` = 选项 → `Option`；
//! - `Click(3)` = 说明 → `Help`；
//! - `Click(4)` = 说明续 → `Help`（第二页，oracle 简化为 `Help`）；
//! - `Click(5)` = 退出 → `Exit`；
//!
//! 真实事件码为 13（确定键），菜单项索引 `v3[14]` 0..=5 与 Click 参数 0..=5 一一对应。

/// 菜单场景枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuScene {
    /// intro.Map 过场动画（`intro`）。
    Intro,
    /// index.Map 主菜单（`index`）。
    Index,
    /// selectrole.Map 角色选择（`selectrole`）。
    SelectRole,
    /// selectmap.Map 地图选择（`selectmap`）。
    SelectMap,
    /// saveload.map 继续游戏（`saveload` case 1 子窗口叠加）。
    Saveload,
    /// Option.map 选项（`saveload` case 2 / `选项场景帧处理`）。
    Option,
    /// help.map 说明（`saveload` case 3/4 / `帮助`）。
    Help,
    /// 退出游戏（`saveload` case 5：`*(a1[1]+12)=1`）。
    Exit,
}

impl MenuScene {
    /// 全部场景（按枚举声明顺序，diff 测试遍历用）。
    #[must_use]
    pub const fn all() -> [MenuScene; 8] {
        [
            Self::Intro,
            Self::Index,
            Self::SelectRole,
            Self::SelectMap,
            Self::Saveload,
            Self::Option,
            Self::Help,
            Self::Exit,
        ]
    }
}

/// 音效枚举（`docs/spec/menu.md` §d，**推测**触发点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    /// 点击有效菜单项（`music\press.wav`）。
    Press,
    /// 无效操作（`music\invalid.wav`，**推测** `a3==37` 分支）。
    Invalid,
    /// 战斗胜利（`music\victory.wav`）。
    Victory,
}

/// 菜单事件。
///
/// - `Click(u32)`：事件码（真实码 13=确定、27=返回、38=上一页、40=下一页、37=无效；
///   推测占位码 0..=5 对应菜单项索引 case 0..=5）。
/// - `Invalid`：无效操作（与 `Click(37)` 等价，diff 测试用独立变体）。
/// - `Quit`：退出游戏。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    /// 点击事件，参数为事件码。
    Click(u32),
    /// 无效操作。
    Invalid,
    /// 退出。
    Quit,
}

impl MenuEvent {
    /// 全部事件变体（diff 测试遍历用）。
    ///
    /// `Click(0)..Click(5)`（推测占位码）+ `Click(13)`（真实确定码）
    /// + `Click(27)`（真实返回码）+ `Invalid` + `Quit` + `Click(37)` + `Click(38)`。
    #[must_use]
    pub const fn all() -> [MenuEvent; 12] {
        [
            Self::Click(0),
            Self::Click(1),
            Self::Click(2),
            Self::Click(3),
            Self::Click(4),
            Self::Click(5),
            Self::Click(13),
            Self::Click(27),
            Self::Click(37),
            Self::Invalid,
            Self::Quit,
            Self::Click(38),
        ]
    }
}

/// oracle：给定当前场景 + 事件，返回 (新场景, 可选音效)。
///
/// 规则（直译 `docs/spec/menu.md` §a/§c/§d）：
/// - `Intro` + 任何事件 → 自动进入 `Index`（过场结束，`00029110:场景管理器` 完成态检查），无音效；
/// - `Index` + `Click(0)`（新游戏）→ `SelectRole` + `Press`；
/// - `Index` + `Click(1)`（继续）→ `Saveload` + `Press`；
/// - `Index` + `Click(2)`（选项）→ `Option` + `Press`；
/// - `Index` + `Click(3)`/`Click(4)`（说明）→ `Help` + `Press`；
/// - `Index` + `Click(5)`（退出）→ `Exit`（无音效）；
/// - `Index` + `Click(13)`（真实确定码）→ 与 `Click(0)` 等价（新游戏，**推测**），`Press`；
/// - `Index` + `Click(37)`/`Invalid` → 保持 `Index` + `Invalid`；
/// - `Index` + `Quit` → `Exit`；
/// - 子场景（SelectRole/SelectMap/Saveload/Option/Help）+ `Click(27)`（返回）→ `Index`；
/// - 子场景 + `Invalid` → 保持 + `Invalid`；
/// - 子场景 + `Quit` → `Exit`；
/// - `Exit` + 任何事件 → 保持 `Exit`（终态）。
#[must_use]
pub fn oracle_menu_transition(cur: MenuScene, ev: MenuEvent) -> (MenuScene, Option<Sfx>) {
    match cur {
        // 过场动画结束 → 主菜单（无音效，动画自动播放）
        MenuScene::Intro => match ev {
            MenuEvent::Quit => (MenuScene::Exit, None),
            _ => (MenuScene::Index, None),
        },
        // 主菜单事件分派（spec §a 事件表）
        MenuScene::Index => match ev {
            MenuEvent::Click(0 | 13) => (MenuScene::SelectRole, Some(Sfx::Press)),
            MenuEvent::Click(1) => (MenuScene::Saveload, Some(Sfx::Press)),
            MenuEvent::Click(2) => (MenuScene::Option, Some(Sfx::Press)),
            MenuEvent::Click(3 | 4) => (MenuScene::Help, Some(Sfx::Press)),
            MenuEvent::Click(5) | MenuEvent::Quit => (MenuScene::Exit, None),
            MenuEvent::Click(37 | 38) | MenuEvent::Invalid => {
                (MenuScene::Index, Some(Sfx::Invalid))
            }
            MenuEvent::Click(_) => (MenuScene::Index, None),
        },
        // 子场景：返回 → Index；无效 → 保持；退出 → Exit
        MenuScene::SelectRole
        | MenuScene::SelectMap
        | MenuScene::Saveload
        | MenuScene::Option
        | MenuScene::Help => match ev {
            MenuEvent::Click(27) => (MenuScene::Index, None),
            MenuEvent::Invalid | MenuEvent::Click(37) => (cur, Some(Sfx::Invalid)),
            MenuEvent::Quit => (MenuScene::Exit, None),
            MenuEvent::Click(_) => (cur, None),
        },
        // 终态：任何事件保持 Exit
        MenuScene::Exit => (MenuScene::Exit, None),
    }
}

/// oracle：从 Intro/Index 出发的可达场景集。
///
/// 遍历所有 (`MenuScene` × `MenuEvent`) 组合的不动点迭代，返回可达场景列表（按枚举序）。
#[must_use]
pub fn oracle_reachable_menu() -> Vec<MenuScene> {
    let mut reachable: Vec<MenuScene> = Vec::new();
    let mut visited: [bool; 8] = [false; 8];
    let mut worklist: Vec<MenuScene> = vec![MenuScene::Intro, MenuScene::Index];

    while let Some(s) = worklist.pop() {
        let idx = s as usize;
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        reachable.push(s);
        for ev in MenuEvent::all() {
            let (next, _) = oracle_menu_transition(s, ev);
            if !visited[next as usize] {
                worklist.push(next);
            }
        }
    }

    reachable.retain(|s| visited[*s as usize]);
    reachable.sort_by_key(|s| *s as usize);
    reachable
}
