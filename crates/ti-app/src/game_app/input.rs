//! 输入路由：键位快照（`poll`）/ 拾取命中（`poll_hit`）/ Command 分发。
//!
//! 拆分自 `game_app.rs`（S20.0b）：输入只做**路由**，判定在 `scenes/*.rs`。
//! S29.2：触摸手势路由（`poll_touch`）退役为 Bevy 拾取命中路由（`poll_hit`，B1 命中层）。

use super::{GameApp, GameInput, HitData, KeyState, Scene};
use crate::scenes::chrome;
use crate::scenes::{SceneKind, route_hit, route_input};

/// S18.5 FX 触发坐标（飞轮 R1 ③-01/③-02 数据化）：读 `scenes.json`
/// `vs.fx_hit` / `main_map.fx_coin`（`chrome::keys::FX_HIT`/`FX_COIN`）；
/// 键缺失 = `(0,0)` crash-safe 兜底（FX 纯视觉层，不影响业务判定，同 ①-07 族口径）。
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn fx_pos(scene: &str, key: &str) -> (f32, f32) {
    let layout = chrome::scene_layout(scene);
    let r = layout.extra.get(key).copied().unwrap_or_default();
    (r.x as f32, r.y as f32)
}

impl GameApp {
    /// 把按键快照映射为 `GameInput` 事件并逐条 `handle`（平台无关，`docs/gamebook.md` §3）。
    ///
    /// 返回是否发生场景切换（壳层据此强制整帧渲染）。
    /// 同一帧内多个键可同时命中（如 confirm + move），按场景语义合并。
    #[allow(clippy::too_many_lines)] // 7 场景输入路由 + R22-010 面板态 ↑↓/Enter/Back 分支，集中一处
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
                // S63：T/B 打开可视化面板（面板内 ↑↓/Enter/Y/Back 由 `on_input` 消费）；
                // 旧 T=直接买 gid1 / B=直接建 Castle 的"无面板盲操作"废弃（场景不全根因）。
                if k.buy {
                    changed = self.handle(GameInput::OpenTradePanel) || changed;
                }
                if k.sell {
                    changed = self.handle(GameInput::TradeSell(1)) || changed;
                }
                if k.build {
                    changed = self.handle(GameInput::OpenBuildPanel) || changed;
                }
                if k.merchant {
                    changed = self.handle(GameInput::OpenMerchantPanel) || changed;
                }
                if k.event {
                    changed = self.handle(GameInput::OpenEventPanel) || changed;
                }
                if k.ship {
                    changed = self.handle(GameInput::OpenShipPanel) || changed;
                }
                if k.garrison {
                    changed = self.handle(GameInput::OpenGarrisonPanel) || changed;
                }
                if k.upgrade {
                    changed = self.handle(GameInput::OpenUpgradePanel) || changed;
                }
                if k.dungeon {
                    changed = self.handle(GameInput::OpenDungeonPanel) || changed;
                }
                // R22-010：任一 S63/S64 面板打开时，↑↓/Enter/Back 必须路由到面板事件——
                // 否则真实键盘无法选/买/关面板（5 个 e2e 直接 handle() 绕过 poll() 故长期漏网）。
                let panel_open = self.trade_panel_open
                    || self.build_panel_open
                    || self.merchant_panel_open
                    || self.event_panel_open
                    || self.ship_panel_open
                    || self.garrison_panel_open
                    || self.upgrade_panel_open
                    || self.dungeon_panel_open;
                if panel_open {
                    if k.up {
                        changed = self.handle(GameInput::MoveUp) || changed;
                    }
                    if k.down {
                        changed = self.handle(GameInput::MoveDown) || changed;
                    }
                    // Enter（k.confirm）独立作面板确认；Space（k.spin）在面板态不摇骰。
                    if k.confirm {
                        changed = self.handle(GameInput::Confirm) || changed;
                    }
                    if k.back {
                        changed = self.handle(GameInput::Back) || changed;
                    }
                } else {
                    // 环岛赛道玩法（S19）：无面板时一次操作 = 摇骰（Space）→ 前进 N 格 → 落格结算。
                    if k.spin {
                        changed = self.handle(GameInput::Spin) || changed;
                    }
                    if k.landmark {
                        changed = self.handle(GameInput::Landmark(1)) || changed;
                    }
                    if k.back {
                        changed = self.handle(GameInput::MapBack) || changed;
                    }
                    // 无面板时 Space 摇骰（见上方 k.spin）；Shift+Y 卖出等保持原路径。
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
    /// S29.2：把一次完成的拾取命中映射为 `GameInput` 并 `handle`（B1：命中/几何全在 Bevy 拾取层，
    /// 逻辑核心只消费归一 [`HitData`]）。**直接命中界面元素，不画虚拟按键**：点徽章/行/槽 =
    /// 选中并执行；滑动 = 翻页/返回。`hit = None`（本帧无交互）时无副作用。
    ///
    /// 返回是否发生场景切换（壳层据此强制整帧渲染）。
    pub fn poll_hit(&mut self, hit: Option<HitData>) -> bool {
        let Some(h) = hit else {
            return false;
        };
        let kind = SceneKind::from_legacy(self.scene);
        route_hit(self, kind, &h)
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
                    let (x, y) = fx_pos("vs", chrome::keys::FX_HIT);
                    self.fx.spawn_hit_spark(x, y);
                }
                GameInput::TradeBuy(_) | GameInput::TradeSell(_) if self.gold != gold_before => {
                    let (x, y) = fx_pos("main_map", chrome::keys::FX_COIN);
                    self.fx.spawn_coin_glint(x, y);
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
