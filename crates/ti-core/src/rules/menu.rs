//! S3.3 主菜单事件状态机正式实现。
//!
//! 与 `oracle::menu::oracle_menu_transition` 行为逐位一致（同一场景 + 同一事件产生
//! 同一 `(MenuScene, Option<Sfx>)`），但结构不同：用 `MenuStateMachine` 结构体 +
//! `handle` 方法承载状态与日志，而非自由函数。
//!
//! 依据 `docs/spec/menu.md`：
//! - 场景枚举 [`MenuScene`]（8 个，从 oracle 重导出）；
//! - 事件枚举 [`MenuEvent`]（`Click(u32)` / `Invalid` / `Quit`）；
//! - `handle` 方法返回可选音效并更新内部场景状态。
//!
//! 事件码（spec §a，`sub_41354` 实测）：
//! - 13 = 确定/Enter；27 = 返回；37 = 无效；38 = 上一页；40 = 下一页；
//! - 0..=5 = 菜单项索引（case 0..=5，**推测**占位码，与真实码 13 等价分派）。

use crate::oracle::menu::{MenuEvent, MenuScene, Sfx};

/// 主菜单状态机：当前场景 + 事件处理日志。
#[derive(Debug, Clone)]
pub struct MenuStateMachine {
    scene: MenuScene,
    log: Vec<(MenuEvent, MenuScene)>,
}

impl MenuStateMachine {
    /// 新建状态机：初始场景 = `Index`（主菜单），空日志。
    #[must_use]
    pub fn new() -> Self {
        Self { scene: MenuScene::Index, log: Vec::new() }
    }

    /// 手动设置场景（用于 diff 测试：构造任意场景而不走 handle 链）。
    pub fn set_scene(&mut self, s: MenuScene) {
        self.scene = s;
    }

    /// 当前场景（只读）。
    #[must_use]
    pub fn scene(&self) -> MenuScene {
        self.scene
    }

    /// 事件处理日志（(事件, 处理后的场景) 对）。
    #[must_use]
    pub fn log(&self) -> &[(MenuEvent, MenuScene)] {
        &self.log
    }

    /// 处理事件，更新场景，返回可选音效。
    ///
    /// 行为与 `oracle::menu::oracle_menu_transition` 逐位一致。
    #[must_use]
    pub fn handle(&mut self, ev: MenuEvent) -> Option<Sfx> {
        let (next, sfx) = transition(self.scene, ev);
        self.scene = next;
        self.log.push((ev, next));
        sfx
    }

    /// 可达场景集（从 Intro/Index 出发，与 `oracle::menu::oracle_reachable_menu` 逐位一致）。
    ///
    /// 此方法语义上为关联函数（不读 `self`），但保留为实例方法以保持 API 一致性。
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn reachable(&self) -> Vec<MenuScene> {
        reachable_from_root()
    }
}

impl Default for MenuStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// 内部自由函数：与 `oracle_menu_transition` 逻辑相同，但作为结构体方法分派的入口。
#[must_use]
fn transition(cur: MenuScene, ev: MenuEvent) -> (MenuScene, Option<Sfx>) {
    match cur {
        MenuScene::Intro => match ev {
            MenuEvent::Quit => (MenuScene::Exit, None),
            _ => (MenuScene::Index, None),
        },
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
        MenuScene::Exit => (MenuScene::Exit, None),
    }
}

/// 内部自由函数：与 `oracle_reachable_menu` 逻辑相同（从 Intro 和 Index 出发）。
#[must_use]
fn reachable_from_root() -> Vec<MenuScene> {
    let mut visited: [bool; 8] = [false; 8];
    let mut worklist: Vec<MenuScene> = vec![MenuScene::Intro, MenuScene::Index];
    let mut reachable: Vec<MenuScene> = Vec::new();

    while let Some(s) = worklist.pop() {
        let idx = s as usize;
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        reachable.push(s);
        for ev in MenuEvent::all() {
            let (next, _) = transition(s, ev);
            if !visited[next as usize] {
                worklist.push(next);
            }
        }
    }

    reachable.retain(|s| visited[*s as usize]);
    reachable.sort_by_key(|s| *s as usize);
    reachable
}
