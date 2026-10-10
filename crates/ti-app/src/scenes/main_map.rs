//! 主地图场景（S8.4：场景 struct 化，经 `SceneHost` 窄能力边界，不接触 `GameApp` 字段）。

use crate::game_app::GameInput;
use crate::scenes::{Scene, SceneHost, SceneKind};

/// 主地图场景处理器（`MainMap`）。
///
/// 只持有场景标识（`SceneKind`），不持有 `GameApp`：所有操作经
/// [`SceneHost`] 能力接口（`advance` 交回宿主业务判定 / `enter` 切场景）。
pub struct MainMapScene;

impl Scene for MainMapScene {
    fn kind(&self) -> SceneKind {
        SceneKind::MainMap
    }

    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> Option<SceneKind> {
        // 业务判定内聚在宿主（`GameApp::handle_X`），场景只做路由：
        // 宿主处理后若切了场景，返回新 `SceneKind`，否则停留。
        let changed = host.advance(input);
        if changed { Some(SceneKind::from_legacy(host.current_scene())) } else { None }
    }

    fn render(&mut self, host: &mut dyn SceneHost) {
        host.render_current();
    }
}
