# S86 计划 · 平台可视化编辑器（地图制作 + UI 组件，可见即可得）

> 用户 2026-11-24 拍板「需要平台的地图制作功能，在游戏场景中有限制的随意布局，可见即可得」，
> 并确认：**两者都要**（地图编辑器升级 + S85 UI 组件可视化编辑器），
> **可见即可得双重判据**（编辑器内嵌真渲染预览 + 导出即生效）。
>
> **硬规则 9（A 纪律）**：本文件 = 设计稿，必须用户过目确认后才落码。S86 全任务 pending，
> 依赖 S85.7 完成后开工。

## 0. 现状基线

- **地图编辑器 v3**（`crates/ti-console/web/index.html`）：左 Tile Palette + 中 SVG 画布（`ed-canvas`）+ 右 Inspector +
  底状态条；31 格可拖拽（`spaces[].pos`），`POST /api/map/import` 落包；**但画布 `viewBox="0 0 1080 1920"` 仍是竖屏**
  （S84 后游戏为 1920×1080 = 遗留漂移），且 SVG 画的是**元数据格子**，不是游戏真实渲染。
- **S85 UI 组件 schema**（`components_v1.json`）：5 纯 UI 场景声明式组件（type/binding/layout/action/role），
  禁绝对坐标；layout 语义意图 + anchored 锚点偏移（用户拍板「两者都要」）；有 `ui_schema_check.py` 校验器。
- **运行期注入**：`ti_core::data::table_source::install`（S31.2）、`TI_MAP`/`TI_THEME` 环境变量注入；
  wasm 浏览器端 S31.2 部署链 `assets/maps/<id>/` fetch 加载。

## 1. 目标（两者都要 × 双重判据）

| 维度 | 地图编辑器升级 | UI 组件可视化编辑器 |
|---|---|---|
| 对象 | 31 格环/领地/地图包（`assets/maps/<id>/`） | `components_v1.json` 5 场景 19+ 组件 |
| 受限随意布局 | 格位置拖拽，但受 I1~I9 不变量 + world_canvas/bbox_pct 约束，校验 FAIL 即禁摆 | 元素拖拽，受 layout 意图/anchored + 叙事 DSL（focus/HUD/死区）约束，校验 FAIL 即禁摆 |
| 内嵌真渲染预览 | 编辑器内看到 = 游戏引擎实际渲染的地图（非 SVG 近似） | 编辑器内看到 = 游戏实际 UI 画面 |
| 导出即生效 | 导出包导入后画面与编辑时一致（视觉 diff 0） | 导出 schema 后游戏画面一致（视觉 diff 0） |

## 2. 真渲染预览技术路径（explore 回填 · 定案）

> 来源：真渲染预览技术路径 explore 子代理实测（2026-11-24）。

**三方案对比与定案**：

| 方案 | 真渲染 | 延迟 | 改动面 | 定位 |
|---|---|---|---|---|
| C. SVG/JSON 操作层 | 元数据近似 | 0 | 仅前端（现成） | **操作层**（保留） |
| A. 后端 `--shot` 服务 + 后台 job | ★★★ 权威（Bevy 同一二进制+注入链） | 30~90 s/张 | ti-console API + env 窄接缝；游戏 crate 零改 | **主路径 / 验收判据** |
| B. WASM 同页 iframe | ★★★ 真引擎 | 冷启数秒 | web 白名单 + `install_injected` + postMessage 桥 | **交互态增益**（S86.4 后） |

**定案 = C + A（+ 后续 B）**：拖拽 → 前端实时受限校验（FAIL 即禁摆）→「松手/点预览」→ `POST /api/preview`
后台 job（`kind=preview`，python3 包装 native bin 不动 `JOB_PROGRAMS` 白名单）→ PNG overlay → 用户确认 →
导出/导入 → 同 env 同 scene 复跑 shot → **PNG 像素 diff 0**（S86.4 判据）。

**关键技术约束（实测）**：
- `table_source::install` / `ui_schema::install_injected` 均 `OnceLock` **一次性定型** ⇒ 连续多次不同布局必须每次重启引擎进程
  （与 `--shot` 冷启模型天然契合）⇒ 这是选 A 而非持久 iframe B 作验收判据的核心理由。
- 未落盘数据：`POST /api/preview` body 写 gitignored `evidence/console/preview/<id>/`（写点白名单已有），
  env `TI_PREVIEW_DIR` 指向临时 JSON，引擎在 `inject_map_pack()` **之前**注入 ⇒ 零跟踪文件写盘。
- env 次序：`apply_minimal_env` 后重设 `TI_MAP`/`TI_THEME`/`TI_SHOT_NAME`/`TI_PREVIEW_*`（照 themes.rs:314-318）。
- 图片回传：出图落 `evidence/`，前端 `<img src="/evidence/...?t=">` 破缓存（白名单 + ETag 已具备）。

**地图编辑器 3 处现存漂移（S86.1 必改）**：
① viewBox `0 0 1080 1920` → `0 0 1920 1080`（缩放/平移/拖拽 ky/导入 payload/focus_bbox_pct 分母全翻）；
② 默认 `ring{center_y:784,radius:480}` 与实际绘制 `V6_CENTER(540,740)/R=460` 口径不一致 ⇒ 从运行时/包 layout 读几何，
  不再手抄常量；③ I8 校验硬编码 `LAYOUT={11,250,1048,1056}` 不验 `st.canvas` ⇒ 用 st.canvas + 1920×1080。

**受限随意布局落地**：地图 = `spaces[].pos` 任意但落 1920×1080 world_canvas 内 + I1~I9/S34.5 双闸
（前端禁摆 + 后端 verify 禁导入）；UI = `layout.intent` 8 值 + anchored 百分比（禁绝对像素）+ role/action/binding 登记
+ 叙事 DSL focus/HUD/死区；拖拽像素**反算为百分比**写回 offset_pct/size_pct，FAIL 即回滚。

**⚠️ 依赖**：S86.3 的 UI schema 预览依赖 S85.3 壳侧接线——当前 `build_component_chrome`（ui.rs:975-1054）
与 `install_injected`（ui_schema.rs）**全库无调用方**（treasure_bevy.rs 仅在注释提及）。S86.2/3 必须与 S85.3/5 协同，
不能独立先做。台账 S86.0 deps=S85.7 已体现此序。

**新增 API（红线）**：`GET /api/ui/components`/`roles`、`POST /api/ui/components/validate|apply`、
`POST /api/preview` + `GET /api/jobs?kind=preview`、`POST /api/map/validate`（dry-run）。
全部须 X-TI-Token + enable_write + 白名单 id，同步 `22_dev_platform` 写点表 + `console_commands.json` +
`ti-console-rules.md`（doccheck2 对账抓幻影端点）。

## 3. WBS（已在台账 S86.0~S86.5 全 pending）

| 任务 | 内容 | 判据 |
|---|---|---|
| S86.0 | 立项：技术路径调研 + 设计稿 + 用户过目 + 登记 | 用户确认留痕 + doccheck 0 error |
| S86.1 | 地图编辑器升级：viewBox 1920×1080 + 受限拖拽 + 真渲染预览 | 横屏画布 + FAIL 禁摆 + 内嵌真渲染 |
| S86.2 | UI 组件可视化编辑器：CRUD + 拖拽 + layout 约束 | 组件可编辑 + schema FAIL 禁摆 |
| S86.3 | 真渲染预览服务（API） | 所见 == 游戏渲染 + 导出导入一致 |
| S86.4 | 导出即生效链路 + 视觉回归 | 画面 diff 0 + ui_verify/resizetest |
| S86.5 | 门禁/文档/平台同步 + 用户视觉确认 | gate 36 项 0 failure + 用户确认 |

## 4. 关键约束

- **受限**：任意布局必须过机器校验（地图 I1~I9 / UI schema enum + 禁绝对坐标 + 叙事 DSL），FAIL 即不能摆/不能导出。
- **零 Rust 改位置**：编辑器导出的是数据（包/schema），玩家/内容运营改布局不改代码（延续 18 号「不手写像素」+ 29 号内容运营）。
- **B1 边界**：vs/main_map 游戏世界场景的 1080 动态内容不在 UI 组件编辑器范围（UI 相机叠层遮挡）。
- **A 纪律**：设计稿确认 + 用户视觉确认留痕；每次改视觉意图须重新过目。

## 5. 风险

- 真渲染预览性能（bevy --shot 同步调用慢；WASM 同页嵌入需解决 canvas 尺寸/resize 与数据注入）。
- S84 遗留 viewBox 1080×1920 改 1920×1080 影响 world_canvas/bbox_pct 对账（N8）。
- 平台写面红线（docs/rules/ti-console-rules.md）：新增写点须白名单 + 审计，不得静默路径。
