//! S3.3 / S8.3.e / S12.4 主菜单事件状态机正式实现（数据化：转移表落本文件）。
//!
//! 本模块的转移表直接落本文件（`docs/business/loop_core.md` §菜单 + `07_ui_flow.md` §1/§3），
//! 类型全部来自 `domain::flow`（S11 类型唯一家）。
//!
//! 事件码语义（13/27/37/38/40 + 菜单项 0..=5）在 `docs/business/loop_core.md`，
//! 菜单标签走 `strings.json`（`menu.*` key，非中文字面量）。

use crate::domain::flow::{MenuDir, MenuEvent, MenuScene, MenuTarget, Sfx};

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
    /// 转移表（`docs/business/loop_core.md` §菜单，本文件为唯一事实源）。
    #[must_use]
    pub fn handle(&mut self, ev: MenuEvent) -> Option<Sfx> {
        let (next, sfx) = transition(self.scene, ev);
        self.scene = next;
        self.log.push((ev, next));
        sfx
    }

    /// 可达场景集（从 `Intro`/`Index` 两入口出发，转移表在本文件）。
    ///
    /// 两入口各走一次转移表取并集：`07_ui_flow.md` §1 转换表中 `SelectMap`（经
    /// `SelectRole` 的 `Click(0|13)` 出边）经 `SelectRole` 出边可达；`Option` 场景
    /// 由地图/选项流驱动（不经菜单转移表），可达闭包按转移表实际边集计算。
    ///
    /// 此方法语义上为关联函数（不读 `self`），但保留为实例方法以保持 API 一致性。
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn reachable(&self) -> Vec<MenuScene> {
        reachable_from_roots(&[MenuScene::Intro, MenuScene::Index])
    }
}

impl Default for MenuStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// 内部自由函数：菜单事件 → 下一场景 + 音效，作为结构体方法分派的入口。
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
            MenuEvent::Click(2..=4) => (MenuScene::Help, Some(Sfx::Press)),
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

/// 内部自由函数：从一组入口场景出发做可达闭包（从 `Intro` 与 `Index` 两入口出发取并集）。
#[must_use]
fn reachable_from_roots(roots: &[MenuScene]) -> Vec<MenuScene> {
    let mut visited: [bool; 8] = [false; 8];
    let mut worklist: Vec<MenuScene> = roots.to_vec();
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

    reachable.sort_by_key(|s| *s as usize);
    reachable
}

/// 菜单光标移动（`07_ui_flow.md` §3.1：`Index` ↑/↓ 在 n 项间循环，不越界）。
///
/// `07_ui_flow.md` §6 规则 1（`menu_cursor_wraps`）：首项上移 → 末项，末项下移 → 首项。
/// 返回 `(新选中项, 是否变化)`（表现层据此决定重绘）。
#[must_use]
pub fn move_menu_cursor(selected: usize, dir: MenuDir, count: usize) -> (usize, bool) {
    if count == 0 {
        return (0, false);
    }
    let next = match dir {
        MenuDir::Up => selected.wrapping_add(count - 1) % count,
        MenuDir::Down => (selected + 1) % count,
    };
    (next, next != selected)
}

/// 菜单项确认目标（`07_ui_flow.md` §6 规则 2 `menu_item_targets`：0/1/2..4/5 →
/// `SelectRole`/`Saveload`/`Help`/`Exit`，路由语义唯一事实源在 `domain::flow::MenuTarget`）。
#[must_use]
pub fn menu_item_target(selected: usize) -> Option<MenuTarget> {
    MenuTarget::from_index(selected)
}
