//! S3.3 / S8.3.e 主菜单事件状态机正式实现（数据化：转移表落本文件，不再调 oracle）。
//!
//! S8 定位变更：旧 `oracle::menu::oracle_menu_transition`/`oracle_reachable_menu` 降级参照物；
//! 本模块的转移表直接落本文件（`docs/business/loop_core.md` §菜单），行为与 oracle 一致
//! （同场景 + 同事件 → 同 `(MenuScene, Option<Sfx>)`），但**不再调用** oracle 自由函数。
//!
//! [`MenuScene`]/[`MenuEvent`]/[`Sfx`] 仍从 oracle 重导出（枚举定义非推测公式，S8.3.e 允许）；
//! 事件码语义（13/27/37/38/40 + 菜单项 0..=5）在 `docs/business/loop_core.md`，
//! 菜单标签走 `strings.json`（`menu.*` key，非中文字面量）。

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
    /// 转移表（`docs/business/loop_core.md` §菜单，与 oracle 行为一致但本文件为唯一事实源）。
    #[must_use]
    pub fn handle(&mut self, ev: MenuEvent) -> Option<Sfx> {
        let (next, sfx) = transition(self.scene, ev);
        self.scene = next;
        self.log.push((ev, next));
        sfx
    }

    /// 可达场景集（从 Intro/Index 出发，转移表在本文件，与 oracle 行为一致）。
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
