---
paths:
  - "crates/ti-app/src/scenes/**"
  - "crates/ti-app/src/menu_shell.rs"
  - "crates/ti-app/src/widgets.rs"
---
# ti-app scenes 编码红线（P6 表现层包，CCGS `rules/*.md` `paths:` 模式借鉴，S50.3）

> 本文件 = `docs/rules/` 路径作用域红线速查面（给 LLM 子代理开轮读，**非机器拦截**；
> 机器拦截仍走 `check_data_driven.py` / `check_scene_narrative.py` / 场景叙事 N1~N8 门禁）。
> 单一事实源 = `docs/process/17_layout_engine.md`（B1 全 bevy_ui 响应式）+
> `docs/process/09_render_framework.md` §5.1（image 原语）+ `docs/business/08_layout_design.md`
> （1080 版面单一事实源），本文件只做路径域速查 + 反例，不重述结论。

## 红线（4~6 条，违反 = 返工）

1. **chrome 归 bevy_ui（B1）**：场景 chrome（面板/列表/卡片/徽章）迁 bevy_ui 自动布局（S29.3 五场景
   全迁：index / select_role / select_map / saveload / message）；bevy_ui 全屏根/容器节点须挂
   `Pickable::IGNORE`（挡下层 1080 命中，`20_bevy_picking.md` 坑位 20-9）。反例：`select_role.rs`
   chrome 仍手写 1080 几何（应迁 `build_list_chrome` bevy_ui flex column）。
2. **vs / main_map 游戏世界留 1080**：bevy_ui UI 相机叠在 1080 世界相机之上，同屏位会遮住 1080 动态
   内容（vs HP 条 / main_map 棋盘+实体+HUD 值），两场景 chrome 与 world 层叠 = 游戏世界场景，整体留
   1080（`17_layout_engine.md` B1「游戏世界保持 1080」+ `19_bevy_ui.md` 坑位 9.8-13 判定）。反例：
   把 main_map 的棋盘/HUD 迁 bevy_ui（应只迁 chrome，游戏内容留 1080）。
3. **场景窄接缝 `SceneHost`**：`ti-app` 只经 `SceneHost` 与 `rules` / `domain` 公开面交互（P6 不反向
   依赖玩法内部，`05_module_parallel.md` §3 包依赖铁律 4）；场景层不直接调 `ti_core::rules::*` 内部
   函数（只走公开 `SceneHost` 契约）。反例：`main_map.rs` 直接 `use ti_core::rules::territory::occupy(...)`
   （应经 `SceneHost` 公开面）。
4. **场景数据驱动（scenes.json 原语）**：场景版面坐标/配色走 `scenes.json` 原语 + `theme.json` 色板，
   不硬编码（`check_data_driven` + `check_scene_narrative` N1~N8 守门；`ui_layer` 只表达呈现层归属，
   原语 rect 仍是 1080 版面规格，`17_layout_engine.md` §3.1）。反例：`message.rs` 写
   `let panel_pos = (40, 200);`（应读 `scenes.json` `message.primitives[].rect`）。
5. **命中契约数据化（pick_key/pick_index）**：可点元素经 `scenes.json` `pick_key` / `pick_index`
   原语 + `PickTarget` / `HitData` 命中契约（`20_bevy_picking.md`），不自写命中几何（1080 命中由
   `pick::hit_1080_pick` 统一，bevy_ui 命中由 `bevy_picking`）。反例：`index.rs` 手写
   `point_in_rect(cursor, badge_rect)` 命中判定（应挂 `PickTarget` key 走统一派发）。

## 正确 / 反例（1~2 条，给子代理开轮速查）

- 正确：`select_map.rs` 用 `build_select_map_chrome`（bevy_ui `PanelSide::Right`）画 3 卡片，选中态
  高亮 + `PickTarget` key=2，`scenes.json` 18 卡片原语标 `ui_layer`。
- 反例：`saveload.rs` 渲染 5 槽列表时直接读 `ti_core::domain::save::SaveSlot` 字段（应经
  `SceneHost` 公开面拿渲染数据，不碰 `domain` 内部结构）。
