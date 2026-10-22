//! 输入路由：键位快照（`poll`）/ 触摸手势（`poll_touch`）/ Command 分发。
//!
//! 拆分自 `game_app.rs`（S20.0b）：输入只做**路由**，判定在 `scenes/*.rs`。

use ti_core::domain::economy::Building;

use super::{FX_BATTLE_HIT_POS, FX_COIN_GOLD_POS, GameApp, GameInput, KeyState, Scene};
use crate::scenes::{SceneKind, route_input, route_touch};
use crate::touch::TouchSample;

impl GameApp {
    /// 把按键快照映射为 `GameInput` 事件并逐条 `handle`（平台无关，`docs/gamebook.md` §3）。
    ///
    /// 返回是否发生场景切换（壳层据此强制整帧渲染）。
    /// 同一帧内多个键可同时命中（如 confirm + move），按场景语义合并。
    pub fn poll(&mut self, k: &KeyState) -> bool {
        // 按住态快照（菜单项按下凹陷，EXP-079 §7.3）：每帧从键位布尔取。
        self.press_held = k.confirm;
        let mut changed = false;
        match self.scene {
            Scene::Index => {
                if k.up {
                    changed = self.handle(GameInput::MoveUp) || changed;
                }
                if k.down {
                    changed = self.handle(GameInput::MoveDown) || changed;
                }
                if k.confirm {
                    changed = self.handle(GameInput::Confirm) || changed;
                }
            }
            Scene::SelectRole | Scene::SelectMap => {
                if k.up || k.left {
                    changed = self.handle(GameInput::Prev) || changed;
                }
                if k.down || k.right {
                    changed = self.handle(GameInput::Next) || changed;
                }
                if k.confirm {
                    changed = self.handle(GameInput::Confirm) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::Back) || changed;
                }
            }
            Scene::MainMap => {
                // 环岛赛道玩法（S19）：主地图不再 4 方向自由走位，
                // 一次操作 = 摇骰（Space）→ 沿赛道前进 N 格 → 落格结算。
                if k.spin {
                    changed = self.handle(GameInput::Spin) || changed;
                }
                if k.buy {
                    changed = self.handle(GameInput::TradeBuy(1)) || changed;
                }
                if k.sell {
                    changed = self.handle(GameInput::TradeSell(1)) || changed;
                }
                if k.build {
                    changed = self.handle(GameInput::Build(Building::Castle)) || changed;
                }
                if k.landmark {
                    changed = self.handle(GameInput::Landmark(1)) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::MapBack) || changed;
                }
            }
            Scene::Vs => {
                if k.confirm || k.landmark {
                    changed = self.handle(GameInput::Attack) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::Back) || changed;
                }
            }
            Scene::Message => {
                if k.down {
                    changed = self.handle(GameInput::MsgNext) || changed;
                }
                if k.up {
                    changed = self.handle(GameInput::MsgPrev) || changed;
                }
                if k.confirm || k.back {
                    changed = self.handle(GameInput::MsgClose) || changed;
                }
            }
            Scene::Saveload => {
                if k.confirm {
                    changed = self.handle(GameInput::WriteSave(0)) || changed;
                }
                if k.back {
                    changed = self.handle(GameInput::SaveloadClose) || changed;
                }
            }
            Scene::Intro | Scene::Exit => {
                if k.confirm {
                    self.scene = Scene::Index;
                    changed = true;
                }
            }
        }
        changed
    }
    /// 把一次完成的触摸手势映射为 `GameInput` 并 `handle`（`docs/modernization.md` §5）。
    ///
    /// **直接触摸界面元素，不画虚拟按键**：点菜单行 = 选中并执行；点地图格 =
    /// 朝该格走一步；点角色/地图/存档槽 = 选中该项；滑动 = 翻页/移动；下滑 = 返回。
    ///
    /// 返回是否发生场景切换（壳层据此强制整帧渲染）。`sample = None`（手势未完成
    /// 或触屏被禁用）时无副作用。
    pub fn poll_touch(&mut self, sample: Option<TouchSample>) -> bool {
        let Some(s) = sample else { return false };
        let kind = SceneKind::from_legacy(self.scene);
        route_touch(self, kind, &s)
    }
    /// 处理输入事件：路由到 `scenes/` 里对应场景的 [`crate::scenes::Scene::on_input`]。
    ///
    /// 调用方随后调 `render` 刷新帧缓冲。返回是否发生了场景切换。
    pub fn handle(&mut self, input: GameInput) -> bool {
        let kind = SceneKind::from_legacy(self.scene);
        let gold_before = self.gold;
        let consumed = route_input(self, kind, input);
        // S18.5 FX 触发（纯视觉层：不改业务判定；攻击回合 / 金币变动才 spawn）。
        if consumed {
            match input {
                GameInput::Attack => {
                    self.fx.spawn_hit_spark(FX_BATTLE_HIT_POS.0, FX_BATTLE_HIT_POS.1);
                }
                GameInput::TradeBuy(_) | GameInput::TradeSell(_) if self.gold != gold_before => {
                    self.fx.spawn_coin_glint(FX_COIN_GOLD_POS.0, FX_COIN_GOLD_POS.1);
                }
                _ => {}
            }
        }
        consumed
    }
    /// S9.5 Command 派发入口：把 [`crate::commands::GameCommand`] 路由到当前场景。
    ///
    /// 与 [`Self::handle`] 等价（`GameCommand::to_input` 后走 `route_input`），
    /// 但语义层统一为 Command 模式，供命令派发枚举调用。
    /// 返回是否消费了该输入（发生了场景切换）。
    pub fn dispatch(&mut self, cmd: crate::commands::GameCommand) -> bool {
        let kind = SceneKind::from_legacy(self.scene);
        let input = GameInput::from(cmd);
        route_input(self, kind, input)
    }
}
