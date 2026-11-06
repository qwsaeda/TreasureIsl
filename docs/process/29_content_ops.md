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

1. **`map_pack.py add <id>`**：从 `_template` 引导骨架（manifest/layout/strings）。
2. **填内容**：
   - `map.json`：复制现有图保证 31 格不变量（I1/I2/I3/I5/I9）+ 定制 isle 排布（简单区互换，
     仅换 isle/resource/building/group，cell/kind/terrain 原位不动）/ events（换种类）/
     merchants（换驻点倍率）。
   - `layout.json`：ring 几何（`center_y`/`radius`/`card_w`/`card_h` 8 整除）+ `island_home`。
   - `strings.json`：`glmap<id>.Map` = 图名。
   - `manifest.json`：`id` = 目录名 + 资产（`source=pool` 共享池复用 / `source=pack` 专属入包）。
3. **`manifest["map"]` ← `map.json` 同步**（⚠ 坑位，见 §5）。
4. **`verify <id>`**：schema/兼容/资产齐+哈希/I1~I9 **0 违规**。
5. **`pack <id>`**：出 `dist/maps/<id>.map-pack/`（+ `.zip`，含池资产副本 `pool/` + `POOL_INDEX.json`）。
6. **`install <dist 路径>`**：装回 `assets/maps/<id>/` + 接线（board.json 段 + index.json 包清单）。
7. **`check`**：已装态 ↔ 内嵌默认数据漂移 **0**。
8. **`sync`**：全部包 map → board.json + 默认包 layout → scenes/manifest/strings。
9. **出图**：`TI_MAP=<id> --shot --scene=main_map`（换图生效；与现有图渲染 diff 或数据级差异
   证明「真新图」）。
10. **零 Rust 判据**：`git diff crates/` = 空。

> ⚠ 视觉来源提示：v5 环形棋盘视觉 = `index + layout ring 几何 + spaces[].isle 卡面`
> （`chrome::ring_card_center` + `draw_ring_card` 的 `isle_card_path`），**`spaces[].cell` 坐标
> 不驱动 v5 视觉**（仅 v3 iso 回退路径）。故「新图」差异须靠 isle 排布 + ring 几何 +
> events/merchants，非 cell（S41.0 实测）。

## 3. 换 UI 流程（S39.0/S42.0）

1. **设计稿先行**（A 纪律，硬规则 9）：改 `scenes.json` 或换 `assets/ui/` → `--shot --viewport`
   出图 → **用户视觉确认留痕** → 才固化。
2. **改数据**：`scenes.json`（版面/配色原语）+ 或换资产（着色/新图 `assets/ui/`）。
3. **重建二进制**（⚠ 坑位，见 §5）：`scenes.json` 是 `include_str!` **编译期内嵌**——改它必须
   `cargo build` 才生效；`assets/ui/*.png` 运行时读盘，无需 rebuild。
4. **出图**：`--shot --scene=<id> --viewport=<方向>`（多视口）。
5. **零 Rust 判据**：`git diff crates/` = 空（重建不改 Rust 源码，仅内嵌数据 blob）。
6. **视觉签收留痕**：`evidence/<id>.log` 含「用户视觉验收/过目/确认/签收」之一
   （doccheck2 第 35 项 visual-signoff 检查）。

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
