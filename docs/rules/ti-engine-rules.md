---
paths:
  - "crates/ti-engine/src/**"
---
# ti-engine 编码红线（渲染/引擎层，CCGS `rules/*.md` `paths:` 模式借鉴，S50.3）

> 本文件 = `docs/rules/` 路径作用域红线速查面（给 LLM 子代理开轮读，**非机器拦截**；
> 机器拦截仍走 `check_data_driven.py` / `check_runtime_no_legacy_coords` / bevy-cpu-parity 等门禁）。
> 单一事实源 = `docs/process/08_render_arch.md`（运行时架构契约）+ `09_render_framework.md`
> （ti-render crate 边界）+ `17_layout_engine.md`（B1 bevy_ui）+ `19_bevy_ui.md` / `20_bevy_picking.md`
> 坑位清单，本文件只做路径域速查 + 反例，不重述结论。

## 红线（4~6 条，违反 = 返工）

1. **渲染层不认场景语义**：`ti-engine`（及 `ti-render`）不得出现场景名/业务判断（`if scene == "vs"` /
   `match "main_map"`），只消费通用原语闭集（`draw_image` / `draw_text` / `draw_poly` ...）+ 数据
   （`scenes.json` / `theme.json`）。反例：`render.rs` 写 `if scene.is_vs() { 画 HP 条 }`（应 =
   `scenes.json` 原语 + 数据驱动，渲染层无感）。
2. **游戏世界保持 1080 正交 + 数据驱动**：游戏内容（棋盘/实体/HUD）仍 1080 正交坐标系 + 数据驱动，
   UI chrome 转 bevy_ui 代码（B1 约定）；bevy_ui 节点与 1080 世界**两层不混坐标系**
   （`08_render_arch.md` 契约 0b + `19_bevy_ui.md` 坑位 9.8-13）。反例：把 bevy_ui 节点坐标直接写
   1080 世界 px（应按视口 `%` / flex 自动布局，resize 真重排）。
3. **通用原语闭集**：渲染层只调已登记的通用原语（`08_render_arch.md` 闭集），不自造新绘制原语
   （加原语须同步 `check_data_driven` 断言 + `gate_data.json`）。反例：`ui.rs` 新写
   `draw_rounded_rect_gradient(...)` 未登记。
4. **零游戏业务逻辑**：`ti-engine` 不得含战斗/经济/存档规则（归 `ti-core` 的 `rules/*` / `domain/*`）；
   引擎层只负责「把数据画出来 + 命中转 `HitData`」，规则判定全在 `ti-core`。反例：`ti-engine/src/vs_proto.rs`
   写伤害结算公式（应 = `rules::battle` 算完传结果进来画）。
5. **多相机口径**：UI 相机 `IsDefaultUiCamera` order=1 / clear None + `RenderLayers::layer(1)` 隔离；
   世界相机 `WorldCam` 标记（`camera_system` / `resizetest` 须显式 `WorldCam`，防多实体 `single()`
   失败致正交 scale 静默不更新，`19_bevy_ui.md` 坑位 9.8-10 + S29.5 缺陷）。反例：`camera_system` 用
   `single()` 取相机（多相机下静默失败，窗口缩放跟随失效）。

## 正确 / 反例（1~2 条，给子代理开轮速查）

- 正确：`main_map.rs` 读 `scenes.json` `main_map.ring.*` 几何 + `board.json` 31 格数据 → 调
  `draw_poly` / `draw_image` 画环形棋盘，无 `if scene == "main_map"` 分支。
- 反例：`chrome.rs` 硬编码 `let gold = [212, 175, 55];`（颜色字面量，应走 `theme.json` 色板令牌，
  doccheck2 `engine-no-color-literal` 断言守门（现行第 15 项，`assets/gate_data.json` `doccheck2_checks` 为准）。
