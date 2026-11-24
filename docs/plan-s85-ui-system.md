# S85 计划 · 完整 UI 系统重构（声明式组件 schema）

> 用户 2026-11-24 拍板「完整 UI 系统重构」，起因是复核发现 `scenes.json` 名牌文字（#19~#28）
> 的 rect 坐标逐条写死（`60,915,720,80` 重复 5 次），且这批 prim 在发货路径（bevy_ui）根本不消费——
> 数据层在描述死坐标、业务绑定硬编码在 Rust 的 `format!` 里、元素没有类型语义。
>
> 用户理想：元素分**有功能业务**与**无功能业务**两类——无功能随数据渲染（不写坐标）；
> 有功能绑定业务、带类型约束（按钮/输入框/列表），业务代码与 UI 两侧按同一 schema 对齐。
>
> **硬规则 9（A 纪律）**：本文件 = 设计稿，必须用户过目确认后才落码。未确认就开工 = 返工。

## 0. 现状诊断（重写前基线）

> 来源：主线程查证 + 3 路 explore 子代理（2026-11-24），带路径+行号。

- **数据层死坐标**：`scenes.json` 7 场景逐条 prim 带绝对 rect；`select_role` 31 条中 10 条名牌文字
  `60,915,720,80`/`60,1000,720,50` 重复 5 次，且 bevy_ui 发货路径不消费（CPU 后端遗留）。
- **业务绑定硬编码**：`build_select_role_chrome`（`treasure_bevy.rs:1346`）从 `Character::all()` 生成行，
  但 i18n key `role.name.{label}` 与资产 `assets/ui/role_{label}.png` 都是 Rust `format!` 拼的；
  `ChromeRow{image,line1,line2,selected}` 是通用结构，无「按钮/输入框」类型语义。
- **既有意图化基础设施**：bevy_ui flex + `%` 自动布局（S29 B1）、`ui.rs::build_list_chrome_pos`
  的 `PanelSide`、叙事 DSL `02_spec_lang.md`、`ui_schema/scene_manifest_v3.json` N8 bbox_pct。

## 1. 目标架构（待用户确认）

> 详见 §2~§4，落码前按本节与用户对齐。

### 1.1 元素二分

| 类 | 定义 | 渲染方式 | 坐标 |
|---|---|---|---|
| **无功能（展示层）** | 背景/装饰/氛围（image/rect/grad/glow） | 场景 schema 声明「意图」（满屏/贴边/居中），引擎算坐标 | 不写死像素 |
| **有功能（控件层）** | 按钮/输入框/列表/卡片/徽章 | 声明 `type` + `binding` + `constraints`，绑定业务对象自动展开 | 布局约束（flex/%/gap） |

### 1.2 声明式组件 schema（示意）

```json
{
  "scene": "select_role",
  "elements": [
    { "id": "bg", "type": "image", "role": "parchment_bg", "layout": "fill" },
    { "id": "roster", "type": "list", "binding": "characters", "layout": {"side":"left","width":"34%"},
      "item": { "type": "button", "icon": "role_portrait", "title": "role.name.{label}",
                "subtitle": "role.desc.{label}", "action": "select_character" } },
    { "id": "nav", "type": "text", "binding": "scene.hint", "layout": "bottom_bar" }
  ]
}
```

- `type`：受限枚举（image/text/button/list/input/panel/badge），非法 type = 数据门禁 FAIL。
- `binding`：业务数据路径（对象列表或 i18n key），运行时解析；不再 Rust 拼字符串。
- `role`：资产语义名，从 `assets/registry.json` / `ui_manifest.json` 按 role 解析路径（替 `format!`）。
- `action`：控件 → 业务动作的声明，事件系统按 action 路由到 `GameInput`/业务方法。
- `layout`：意图（fill/side/center/bottom_bar/flex），引擎算坐标；无绝对像素。

### 1.3 通用渲染器与事件系统

- `ti-engine::ui` 增 schema→bevy_ui 的通用 builder：按 `type` 分派到控件构造器，按 `binding` 取数，
  按 `layout` 交给 taffy 自动布局；删 5 个 `build_*_chrome` 的硬拼逻辑。
- 事件：控件声明 `action` → 统一 `PickTarget`/`Message` 路由（替散落的 `attach_row_pick` 硬编码 key）。
- CPU 后端（仅 parity 测试用）与 bevy 共用同一 schema→布局逻辑或明确退役边界。
- **实施捷径**：扩展既有 `DataChromeItem`/`build_data_chrome`（§2），不重写渲染器；5 场景中 index 已数据化，改造量最小。

## 2. 影响面（explore 回填 · 第 1 路 bevy_ui 渲染/事件层）

> 来源：bevy_ui 渲染/事件 explore 子代理实测（2026-11-24），带路径+行号。

**关键发现：index 场景已是数据化种子，通用渲染器可复用而非从零造**：
- `index_data_items`（`treasure_bevy.rs:1050-1112`）读 `scenes.json` `index.primitives`（`ui_layer=bevy_ui`），
  转 `ui.rs::DataChromeItem`（`:599-623`，与 `Primitive` 近乎同构），交 `build_data_chrome`（`:642-770`）
  按 z 升序/when 过滤/rect `%` 换算/kind 分派（image/icon/text/rect）/Pickable 覆盖渲染。
- `build_data_chrome` 已是「通用 schema-lite 渲染器」：**S85 的 type 约束/binding/action 只需扩展
  `DataChromeItem` + `build_data_chrome`，无需重写渲染器**。

**5 个 `build_*_chrome` 硬拼清单（S85 要消灭的业务取数硬编码）**：
- `build_index_chrome`（`:1026-1044`）已数据化（复用上面种子）⇒ 改造量最小。
- `build_message_chrome`（`:1123-1343`）面板参数 `message_panel_params()` + i18n `format!("msg.{}")` 硬拼（`:1118-1143`）。
- `build_select_role_chrome`（`:1346-1372`）`format!("role.name.{label}")` + `format!("assets/ui/role_{label}.png")` 硬拼（`:1352-1364`）。
- `build_saveload_chrome`（`:1375-1406`）`sgetf("slot.label_fmt",...)` 等硬拼。
- `build_select_map_chrome`（`:1409-1436`）`format!("glmap{}.Map", m.id())` 硬拼（`:1415`）。

**事件硬拼四段（管道本身通用，opaque key 可保留）**：
① `attach_row_pick(commands,&chrome,key)` 裸数字（`treasure_bevy.rs:1444-1450`，key=2/3）；
② 7 个场景 `on_hit` 里 `if hit.key == N`（index=1/select_role=2/select_map=2/saveload=3/vs=2,3,4/main_map=4）；
③ i18n key 的 `format!`；④ 资产路径的 `format!`。
`PickTarget`/`PickEvent`/`HitData`/`route_hit` 管道通用 ⇒ S85 只需把「key 常量 + 业务动作」改为 schema 声明的 `action`。

**资产 role 解析缺失**：`assets/ui` 是「前缀语义+后缀 id」扁平命名（339 文件），
无 `role→path` 注册表解析；`assets/registry.json`/`ui_manifest.json` 已登记 UI 图但未被 role 消费。
S85 设计稿 §1.2 的 `role: "role_portrait"` 从 registry 解析正是对症下药。

**vs/main_map 边界（一期不纳入 bevy_ui 化）**：两场景留 1080 是 B1 签核（UI 相机 order=1 会遮住 1080 动态内容）。
其元素已在 scenes.json 原语里由 `render_primitives` 数据驱动，不需要「迁层」，但动态 HUD/棋盘/HP 填充
是 Rust 过程式（`render_hud_v3`/`render_v6_elem`/`render_siege`）⇒ 二期可纳入 schema 的 `binding`/`type` 约束。
一期只在 schema 声明「world 层由 1080 渲染器消费」的边界标注，并守 UI 相机不覆盖。

**通用渲染器起点**：`ui.rs:599-770`（DataChromeItem/build_data_chrome）+ `ChromeRow:218-231`/`ListChrome:237-243`。

## 2b. 影响面（explore 回填 · 第 2 路 数据/业务可绑定性）

> 来源：数据/业务 explore 子代理实测（2026-11-24）。

**现成可复用三件套**：① 叙事 DSL enum/schema 校验（`scene_manifest_v3.schema.json`）+ `check_scene_narrative` N1~N7；
② `DataChromeItem`/`build_data_chrome`（通用渲染）；③ `ui_icons.json::icon()`（语义名→资产路径+tint 的运行时单源）。
缺的是 `type`/`binding`/`action` 三列 schema + 按 role 解析**全量** UI 资产的索引（现 `ui_icons` 仅 14 枚图标）。

**业务绑定源 API 已齐但需注册表**：`Character::all()`/`MapSlot::all()`/`MsgKind::all()`/`SaveSlot`/经济/领地/面板开关
全是 `SceneHost` trait 方法（稳定出口），但没有「binding 名 → 取值」注册表；需要一个 binding resolver
（声明 `"characters" → Character::all()+SelectionHost::character`），且容忍 default fake 宿主的空/0 值。

**i18n 两通路**：`Primitive.text_key`（Rust 消费）与 5 处 `params.text_key`（Rust **不读**的死数据）需统一；
参数化 key `role.name.{label}` 需加「key 模板 + binding 变量 → sget/sgetf」解析器（现有 sgetf 只填值占位不填 key 占位）。

**⚠️ S84 遗留口径漂移（新 schema 落地前必须先修，否则踩旧写点）**：
- 主题注入链已死：`scenes.json` 0 处 `landscape`，但 `map_pack.rs::merge_theme_scenes`（`:346-373`，写 `index.landscape.primitives`）、
  `ui_theme.py:95-98`、`themes.rs` 注释仍按旧键写 ⇒ apply 成功但游戏无视觉变化。
- N8 对账失真：`check_scene_narrative.py:74-76,216-219` / `layout_spec_check.py:112-127` 仍用 1080×1920，
  prim rect 已是 1920×1080。
- `ui_assembly.py:80-88` `layers_of()` 仍取 `scene_data["landscape"]`，S84 后会 KeyError。
- `SceneLayout::prims_for()` 的 `landscape_prims` 回退（data/mod.rs:800-810）恒空，兼容分支待清。

**设计决策（供用户确认）**：schema 独立文件 `assets/data/ui_schema/components_v1.json` + build-time 编译/校验器，
产出 `DataChromeItem` 兼容视图（替 `treasure_bevy.rs:1050` 的散落转换）；注入点统一到 schema 段，
主题/地图包工具改指向新 schema 明确写点，strings.json upsert 保留为 i18n binding 源。

## 3. WBS（阶段门，逐阶段验收 · 已在台账 S85.0~S85.7 全 pending）

| 任务 | 阶段 | 内容 | 判据 |
|---|---|---|---|
| S85.0 | 立项 | 设计稿 + 影响面 + 用户过目 + 登记 | doccheck 0 error + 用户确认留痕 |
| S85.1 | schema 规范 | type/binding/layout/action/role 契约 + 负例门禁 | enum 校验 FAIL 负例 + 规范单一源 |
| S85.2 | 数据落地 | 5 纯 UI 场景 schema + 组件索引 + 清 S84 漂移（主题/N8/装配工具） | validate_data 0 违规 + 0 死坐标 |
| S85.3 | 通用渲染器 | 扩 `DataChromeItem`/`build_data_chrome` 支持 type/binding/layout | 5 场景 schema 渲染非黑 + resize 重排 |
| S85.4 | 事件绑定 | action→GameInput 路由 + binding resolver + PickTarget 通用化 | 点击/键盘全链路 e2e |
| S85.5 | 删硬拼 | 5 个 `build_*_chrome` 硬拼 + 死原语删除 | `grep format!(assets/) 0` + 死 prim 0 |
| S85.6 | 门禁迁移 | doccheck2/validate_data/check_scene_narrative/ui_wizard/themes 迁移 | gate 36 项 0 failure |
| S85.7 | 收口 | 用户视觉/交互确认 + 计数 + 三方对账 + push | gate 全绿 + push |

### 执行顺序

S85.0 → S85.1 → S85.2（含清 S84 漂移）→ S85.3 → S85.4 → S85.5 → S85.6 → S85.7。每阶段门实跑判据 → evidence → gate 全绿 → 台账 verified。

## 2c. 影响面（explore 回填 · 第 3 路 依赖/门禁/迁移边界）

> 来源：依赖门禁 explore 子代理实测（2026-11-24）。

**测试迁移**（读 scenes.json prim 的 7 个测试文件）：`render_intent.rs`（资产锚/坐标锚/选中态防外溢——选中态语义必须保留）、
`landscape_drawlist.rs`（derived=menu 18 条→schema list 控件项）、`main_map_pick_e2e.rs`（key=4 语义保留，载体改 action）、
`e2e_playability.rs`（死 rect 断言→resizetest/ui_verify 探针）、`bevy_native_render.rs`（1080 层 0 原语后可能退役）。
vs/main_map 的坐标锚**保留**（游戏世界不迁）。

**gate/doccheck2 迁移**：`validate_data`/`check_data_driven`/`check_scene_narrative`（N8）+ doccheck2 约 10 项检查
（`touch-render-pair`/`scene-layout-fidelity`/`primitives-data`→`components-data`/`no-handwritten-grid`/`theme-consumption`/
`visual-signoff` TOUCH 扩新文件名 等）。**检查项 id 增减须同步 `gate_data.json` + `studio_tests.py` + `doc_numbers.py` 三处**。
`bevy-cpu-parity` 口径不改（仅游戏世界，UI 层 Node=0）。

**脚本/平台写点**：`validate_data.py`/`check_data_driven.py`/`ui_assets.py`/`ui_theme.py`/`check_scene_narrative.py` 改指向新 schema；
`map_pack.py` 不动（游戏世界/地图包面）；ti-console `ui_wizard.rs`/`themes.rs` + 引擎 3 个注入点
（`map_pack.rs::merge_theme_scenes`/`apply_theme_preview`/`patch_theme_assets`）统一写点。
`ui_verify.py` 不读 scenes.json ⇒ 升为主判据（非黑+resize 重排+5 场景）。

**阴影期双轨（防 gate 空窗）**：S85.2 旧 prim 暂不删、标 deprecated；壳双读但只渲染新 schema；S85.5 才删旧。
**迁移底线**：vs/main_map + extra/ring/focus_anchor/world_canvas/地图包 layout.json **不迁**；
保留语义不可丢（选中态不外溢/缺键=不可点/资产必登记/i18n 不裸 key/视觉签收机器面/pick_key 5/6/7 防回潮）。
须在 `17_layout_engine.md` 增签核：5 纯 UI 场景 chrome 从「Bevy 代码硬拼」回退为「schema 数据+通用 Bevy builder」，B1 其余约定不变。

## 4. 判据

- 用户视觉/交互确认（A 纪律硬规则 9）
- gate 36 项 0 failure + doccheck/doccheck2 0 error + 台账三方对账
- `scenes.json` 0 逐条死坐标 prim；`build_*_chrome` 0 `format!("assets/...")`/`role.name.` 硬拼
- 负例：schema 非法 type/binding/action → 数据门禁 FAIL

## 5. 风险

- 架构级重写，触及 5 纯 UI 场景 + vs/main_map 游戏世界边界（后者留 1080）。
- 平台 ti-console 内容向导、地图包主题注入写 scenes.json 的旧键需同步。
- 历史「不手写像素」总纲（18 号）与 B1（17 号）须兼容或显式取代。
