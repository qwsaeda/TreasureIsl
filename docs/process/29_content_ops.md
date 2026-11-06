# 29 · 内容运营规范（零 Rust 内容运营：程序构成模型 + 换 UI/改地图流程）

> **本文件 = 内容运营（只动数据、零 Rust）的唯一事实源**。触发：用户 2026-11-05
> 「是不是不改业务，我们只用换 UI 和改地图了？」→ S40.0（m21）/S41.0（m22）改地图验证 +
> S39.0/S42.0 换 UI 验证 →「把流程制定规范出来」→ 本文件。
>
> 关联（单一事实源引用，不复述）：`21_map_package_spec.md`（地图包格式/I1~I9）、
> `18_asset_content_pipeline.md`（资产管线）、`13_ui_asset_pipeline.md`（UI 资产管线）、
> `00_design.md` §2（业务数据驱动分层）。

## 0. 程序构成模型（代码 + UI数据 + 地图数据）

程序 = **代码 + UI + 地图**，其中 UI 和地图是**数据**（不是代码）。这解释「只换 UI/改地图 = 零 Rust」。

| 层 | 载体 | 改它 | 变更面声明（硬规则 7） |
|---|---|---|---|
| **代码** | `crates/`（Rust：`ti-core` 规则 + `ti-engine` 引擎 + `ti-app` 平台 + `ti-render` 渲染框架 + `ti-tools` 工具） | 改 Rust → 编译 | 业务规则 / 引擎 / 平台 |
| **UI 数据** | `assets/data/scenes.json`（版面原语）+ `assets/data/theme.json`（配色令牌）+ `assets/ui/*.png`（图片资产） | 改数据 → **零 Rust** | UI 视觉 |
| **地图数据** | `assets/maps/<id>/`（`map.json` 玩法 + `layout.json` 版面 + `strings.json` 文案 + `manifest.json` 清单） | 改数据 → **零 Rust** | 地图内容 |

- **UI/地图 = 数据驱动**：代码只读取并渲染（UI = `build_data_chrome` 等数据通道；地图 =
  `&BoardMap` 参数）。换它们不碰 Rust。
- **业务规则在代码里**（`ti-core` 规则：战斗/经济/领地/存档）——**改业务才碰 Rust**。
  `00_design.md` §2 是**业务数据**分层（数值/文本/机制），本模型是**内容/资产**视角，两者互补。
- **内容运营前提**：业务已定案稳定（不改业务 = `ti-core` 规则零改动）。

## 1. 内容运营核心判据（每次内容运营任务必查）

1. **零 Rust**：`git diff crates/` = 空（机器证明，核心不变量）。
2. **不改业务**：`ti-core` 规则/存档/战斗/经济零改动。
3. **只动数据**：`assets/maps/<id>/` + `assets/data/scenes.json`/`theme.json` + `assets/ui/`。
4. **验收**：`map_pack verify/check` + `--shot` 出图 + gate 全绿 + `evidence/<id>.log`
   （含任务自身判据；硬规则 2）。

## 2. 改地图流程（S40.0/S41.0 验证；固定顺序不可跳）

> **平台向导列 = 机器断言的文档侧**（S44.5）：下表「平台向导步骤 / 判据 / 写面」三列由
> `GET /api/content/mapping`（只读）与 `crates/ti-console/tests/content_docmap_tests.rs` 对账
> 平台向导（`content::map_wizard`）——**步骤 id 恰好一次 + 判据关键词 + 写面登记**三者一致，
> 任一侧漂移即 FAIL。§2 ↔ 地图向导 10 步为**严格序号 1:1**。

| # | 流程（29 号） | 平台向导步骤 | 白名单动作 | 判据 | 写面 |
|---|---|---|---|---|---|
| 1 | **`map_pack.py add <id>`**：从 `_template` 引导骨架（manifest/layout/strings） | `add` | `mappack_add` | exit=0（0=成功） | `assets/maps/` |
| 2 | **填内容**：`map.json` 复制现有图保证 31 格不变量（I1/I2/I3/I5/I9）+ 定制 isle 排布（简单区互换，仅换 isle/resource/building/group，cell/kind/terrain 原位不动）/ events（换种类）/ merchants（换驻点倍率）；`layout.json` ring 几何（`center_y`/`radius`/`card_w`/`card_h` 8 整除）+ `island_home`；`strings.json` `glmap<id>.Map` = 图名；`manifest.json` `id` = 目录名 + 资产（`source=pool` 共享池复用 / `source=pack` 专属入包） | `import` | 平台自身面（`POST /api/map/import`） | exit=0 + verify 0 违规（I1~I9 终判，内容已填充） | `assets/maps/` |
| 3 | **`manifest["map"]` ← `map.json` 同步**（⚠ 坑位，见 §5） | `syncmap` | `mappack_syncmap` | exit=0（0=成功） | `assets/maps/<id>/manifest.json` |
| 4 | **`verify <id>`**：schema/兼容/资产齐+哈希/I1~I9 **0 违规** | `verify` | `mappack_verify` | exit=0 + 输出含 0 违规/0 漂移/0 差 | 只读 |
| 5 | **`pack <id>`**：出 `dist/maps/<id>.map-pack/`（+ `.zip`，含池资产副本 `pool/` + `POOL_INDEX.json`） | `pack` | `mappack_pack` | exit=0（pack 成功 + dist 产物存在） | `dist/maps/<id>.map-pack/` |
| 6 | **`install <dist 路径>`**：装回 `assets/maps/<id>/` + 接线（board.json 段 + index.json 包清单） | `install` | `mappack_install` | exit=0（install 接线成功） | `assets/maps/` + `assets/data/board.json` + `assets/maps/index.json` + `assets/ui_manifest.json` |
| 7 | **`check`**：已装态 ↔ 内嵌默认数据漂移 **0** | `check` | `mappack_check` | exit=0 + 输出含 0 违规/0 漂移/0 差 | 只读 |
| 8 | **`sync`**：全部包 map → board.json + 默认包 layout → scenes/manifest/strings | `sync` | `mappack_sync` | exit=0（0=成功） | `assets/data/board.json` + `assets/data/scenes.json` + `assets/data/strings.json` + `assets/data/ui_schema/scene_manifest_v3.json` |
| 9 | **出图**：`TI_MAP=<id> --shot --scene=main_map`（换图生效；与现有图渲染 diff 或数据级差异证明「真新图」） | `shot` | `map_shot` | exit=0（出图成功 + 图像非黑判据复核） | `evidence/` |
| 10 | **零 Rust 判据**：`git diff crates/` = 空 | `zero_rust` | 平台自身面（`git diff --quiet -- crates/…`） | exit=0 + git diff crates/ 空（游戏产品 6 crate） | 只读 |

> ⚠ 视觉来源提示：v5 环形棋盘视觉 = `index + layout ring 几何 + spaces[].isle 卡面`
> （`chrome::ring_card_center` + `draw_ring_card` 的 `isle_card_path`），**`spaces[].cell` 坐标
> 不驱动 v5 视觉**（仅 v3 iso 回退路径）。故「新图」差异须靠 isle 排布 + ring 几何 +
> events/merchants，非 cell（S41.0 实测）。

## 3. 换 UI 流程（S39.0/S42.0）

> **平台向导列 = 机器断言的文档侧**（S44.5）：下表三列由 `GET /api/content/mapping` 与
> `crates/ti-console/tests/content_docmap_tests.rs` 对账 UI 向导（`content::ui_wizard`）。
> §3 ↔ UI 向导 6 步为**锚点映射**（非严格序号）：§3.2「改数据」在平台拆为 `variant`（换资产）
> + `layout`（改版面原语）；§3.5「零 Rust」+ §3.6「视觉签收留痕」在平台合为 `zero_rust` 一步
> （判据含签收关键词）。断言口径 = 每步 id 至少出现一次 + 每行步骤集合与平台锚点**逐行相等**
> + 判据关键词 + 写面登记三者一致。

| # | 流程（29 号） | 平台向导步骤 | 白名单动作 | 判据 | 写面 |
|---|---|---|---|---|---|
| 1 | **设计稿先行**（A 纪律，硬规则 9）：改 `scenes.json` 或换 `assets/ui/` → `--shot --viewport` 出图 → **用户视觉确认留痕** → 才固化 | `design` | 平台自身面（scene shot 当前状态出图） | exit=0（出图成功 + 图像非黑判据复核） | `evidence/` |
| 2 | **改数据**：`scenes.json`（版面/配色原语）+ 或换资产（着色/新图 `assets/ui/`） | `variant` + `layout` | `ui_variant`（换资产）+ 平台自身面（版面原语只读断言） | `variant`：exit=0 + ui_manifest 登记一致（disk↔registry，validate_data 0 违规）；`layout`：exit=0（scenes.json indent=1 规范形 + check_scenes_canonical 0 差） | `variant`：`assets/ui/` + `assets/ui_manifest.json`；`layout`：只读（可选跳过断言） |
| 3 | **重建二进制**（⚠ 坑位，见 §5）：`scenes.json` 是 `include_str!` **编译期内嵌**——改它必须 `cargo build` 才生效；`assets/ui/*.png` 运行时读盘，无需 rebuild | `build` | 平台自身面（`cargo build`） | exit=0（build 成功，scenes.json 生效） | 只读（产物落 `target/`，非冻结路径） |
| 4 | **出图**：`--shot --scene=<id> --viewport=<方向>`（多视口） | `shot` | 平台自身面（scene shot 多视口） | exit=0（出图成功 + 图像非黑判据复核） | `evidence/` |
| 5 | **零 Rust 判据**：`git diff crates/` = 空（重建不改 Rust 源码，仅内嵌数据 blob） | `zero_rust` | 平台自身面（`git diff --quiet -- crates/…`） | exit=0 + git diff crates/ 空（游戏产品 6 crate） | 只读 |
| 6 | **视觉签收留痕**：`evidence/<id>.log` 含「用户视觉验收/过目/确认/签收」之一（doccheck2 第 35 项 visual-signoff 检查）——与第 5 步**同一步**，是 `zero_rust` 判据的后半段 | `zero_rust` | 平台自身面（同第 5 步） | exit=0 + git diff crates/ 空 + `state.signoff` 含签收关键词（A 纪律 EXP-114） | 只读 |

## 4. 打包/分发链（内容运营产物）

- **包**：`pack` → `dist/maps/<id>.map-pack/`（+ `.zip`）可分发件。
- **装上**：`install` → `assets/maps/<id>/`（构建期默认数据回注）。
- **部署**：`build_wasm.ps1` 4e 段 → `web/assets/maps/<id>/`（浏览器侧）。
- **运行期换图**：`TI_MAP=<id>` 显式 / `.default` 指针选默认包 / 缺包回退内嵌默认。
- **换图不改 Rust**：运行期 `data::activate_pack(id)` 数据层换源 + 表现层重取 layout；
  规则层吃 `&BoardMap` 参数（`21_map_package_spec.md` §7）。

## 5. 坑位教训（EXP 固化；S40.0~S42.0 踩坑）

| 坑 | 症状 | 治本 |
|---|---|---|
| **`manifest["map"]` 不同步** | `map_pack` verify/sync/install 读 `manifest["map"]`（内嵌权威字段）；`check` 对账 board.json vs `manifest["map"]`；`map.json` 是冗余副本**无自动同步**。只改 `map.json` 不同步 `manifest["map"]` ⇒ 出图仍旧视觉（S41.0） | 填内容后**必** `manifest["map"]` ← `map.json` 同步再 verify/pack/install/check/sync |
| **`scenes.json` 内嵌须 rebuild** | `SCENES_JSON` 是 `include_str!` 编译期内嵌，非运行时读盘。改 `scenes.json` 不出效果（S42.0） | 改 `scenes.json` 后**必** `cargo build` 再 `--shot`；资产 `assets/ui/*.png` 运行时读盘无需 rebuild |
| **dod 措辞触发误报** | doccheck2 visual-signoff 只看 dod 含 `scenes.json`/`theme.json` 字面。地图包任务（非默认包）dod 若提「sync 回注 scenes.json」会误报（S41.0） | 地图包任务 dod 不写 `scenes.json` 字面（非默认包全局版面未变） |

## 6. 内容运营 vs 从零设计（分界）

- **内容运营**（本规范）：复制现有图 + 数据定制，零 Rust；换 UI 视觉仍受 A 纪律（设计稿先行）。
- **从零设计**（`21_map_package_spec.md` §7a）：规格定案→元素创作→登记→迭看→落码→打包，
  含 AI 生图 + 完整设计 + 用户迭看留痕。

## 7. 关联

- 地图包格式/I1~I9：`21_map_package_spec.md`。
- 工具：`scripts/map_pack.py`（台账 `14_tooling_catalog.md` §3.2）。
- 资产管线：`18_asset_content_pipeline.md`（③参数化「不手写像素」）。
- UI 资产管线：`13_ui_asset_pipeline.md`。
- 业务数据分层：`00_design.md` §2。
- 验证记录：S40.0（m21）/S41.0（m22）/S42.0（index 变体）`evidence/*.log`。
