# 21 · 地图包（Map Package）规范 v1（S31 定案）

> **本文件 = 地图包格式与流程的唯一事实源**：一张地图 = 一个**自带专属资产、可分发、可校验、
> 可安装、可运行期切换**的包（共享资产按 §4 声明依赖）。
>
> **与 18 的分工**：[`18_asset_content_pipeline.md`](18_asset_content_pipeline.md) = 资产/内容数据管线
> **总纲**（行业三档 + 总纲「不手写像素：声明意图/数据，引擎算坐标」）；本文 = **③参数化档在地图上的
> 落地规范**（包格式 / 清单字段 / 版本兼容 / 完整性 / 运行期加载契约 / 失败回退）。
>
> **与 `assets_hd_pack` 的分工**：媒体包 = **贴图**分发（分辨率档 + `manifest_hd.json`）；
> 地图包 = **玩法 + 版面 + 文案 + 资产**分发（本文）。
>
> 触发：用户 2026-10-28「我们还没有标准的地图打包工具，标准，不能随时更新更换地图？每次都要改代码
> 和手工编辑 `scenes.json` / `*.json`？」→「现在是打包规范，标准在哪，打包的产物在哪」→
> 「打包规范 / 工具 / 产物」。选定 **B 运行期换图**：包在**运行期**被加载并切换，不靠重新构建。
>
> 相关：[`14_tooling_catalog.md`](14_tooling_catalog.md)（工具台账 + 流水线）·
> [`13_ui_asset_pipeline.md`](13_ui_asset_pipeline.md)（UI 资产生成/抠图/登记）·
> [`08_render_arch.md`](08_render_arch.md)（契约 4 版面数据驱动）·
> [`17_layout_engine.md`](17_layout_engine.md)（B1 布局引擎）·
> 业务规格 [`../business/territory.md`](../business/territory.md)（领地/副本规则）·
> [`../ui-redesign-v3/10_main_map_v5_spec.md`](../ui-redesign-v3/10_main_map_v5_spec.md)（主地图 v5 版面）

---

## 0. 设计意图

1. **一张图 = 一个包**（不是散在 `board.json` / `scenes.json` / `strings.json` / `ui_manifest.json` 四处的碎片）。
2. **换图 = 装包 + 切包**（不改 Rust、不手改多份 JSON、不重新构建）。
3. **单一事实源**：包内 `manifest.json` 是该图的权威清单；仓库内的共享数据文件（`board.json` 等）
   是**构建期默认包**（内嵌 fallback），不是运行期事实源。

> 「地图随便换」= 数据驱动可行性的**延伸**：本文把「加图 = 加 JSON」升级为「加图 = 出一个包 + 装上」。

## 1. 包 = 什么

包 = **一个目录**（不用 zip：运行期热换需要 `AssetServer` 直读；zip 需解压，wasm 不便）。

```
assets/maps/<id>/                  ← 包根（仓库内 = 已安装态；运行期可直读）
  manifest.json                    ← 包清单（权威：版本 / 兼容 / 哈希 / 不变量 / 资产索引）
  map.json                         ← 地图内容（网格 + 31 格赛道 + 岛型 + 分区）
  layout.json                      ← 该图版面参数（ring / world_canvas / island_home / deck_*）
  strings.json                     ← 该图自己的文案（岛名 / 副本名 / 功能格名）
  assets/                          ← 该图**专属**资产（island_home_<id>.png / terrain_<id>.png …）
```

| 文件 | 职责 | 谁消费 |
|---|---|---|
| `manifest.json` | 清单：身份 + 版本 + 兼容 + 完整性 + 不变量 + 资产索引 | 工具（verify/install）+ 运行期加载器 |
| `map.json` | 玩法内容：网格尺寸 + 31 格赛道（格序 = 前进方向） | `rules::board` / `rules::territory` |
| `layout.json` | 版面参数：环几何 + 焦点矩形 + 中央岛 + CTA/键位 | `chrome::scene_layout`（表现层） |
| `strings.json` | 该图自己的文案（键名沿用 `strings.json` 的键空间） | i18n 层 |
| `assets/` | 该图**专属**资产（**共享**资产不入包，见 §4） | 资产层 |

## 2. `manifest.json` 字段规范

```json
{
 "schema": "map-pack/1",
 "id": "m01",
 "version": "1.0.0",
 "data_version": "board-v5",
 "map": { "grid_w": 20, "grid_h": 20, "spaces": [ { "cell": [10,0], "group": 1, "kind": "start", "terrain": "L" } ] },
 "layout": {
  "ring": { "center_x": 540, "center_y": 784, "radius": 480, "card_w": 96, "card_h": 120 },
  "world_canvas": { "x": 11, "y": 250, "w": 1048, "h": 1056 },
  "island_home": { "x": 160, "y": 400, "w": 760, "h": 760 },
  "deck": { "spin": { "x": 408, "y": 1336, "w": 264, "h": 264 } }
 },
 "strings": { "isle.harbour": "港口城" },
 "assets": [ { "file": "assets/island_home_m01.png", "source": "pack", "sha256": "<64 hex>", "bytes": 123456 },
             { "file": "assets/ui/isle_harbour.png", "source": "pool", "sha256": "<64 hex>", "bytes": 123456 } ],
 "invariants": { "spaces": 31, "start": 1, "territory": 20, "feature": 9, "boss": 1, "isle_kinds": 10 }
}
```

| 字段 | 必填 | 语义 | 机器判定 |
|---|---|---|---|
| `schema` | ✓ | 包格式版本（当前 `map-pack/1`） | 不匹配 ⇒ 拒绝安装 |
| `id` | ✓ | 地图 id（= 目录名 = `MapSlot` 口径） | 与目录名一致 |
| `version` | ✓ | 包自身的语义化版本（内容变化即升） | — |
| `data_version` | ✓ | **玩法数据结构兼容声明**（当前 `board-v5` = 31 格赛道结构） | 不受支持 ⇒ 拒绝加载 |
| `map` | ✓ | 地图内容（= `board.json` `maps.<id>` 的内容面） | 见 §5 不变量 |
| `layout` | ✓ | 版面参数（环几何等） | 8pt 校验（§5） |
| `strings` | ✓ | 该图文案（键 → 值） | 键在 `strings.json` 键空间内 |
| `assets` | ✓ | 包用到的资产索引：`file` + `source`（`pack`=包内自带 / `pool`=共享池）+ `sha256` + `bytes` | 两种来源都在位 + 哈希一致 + 字节数一致 |
| `invariants` | ✓ | 该包自述的构成（工具据此做交叉校验） | 与 `map.spaces` 实测一致 |

> 哈希口径 = **SHA-256**（小写 hex）；与 `assets_hd_pack/manifest_hd.json` 的 `fnv1a64` 并存
> （媒体包用 FNV 是历史选择；地图包用 SHA-256，因包需跨机器/跨版本校验完整性）。

## 3. 包内数据文件规范

### 3.1 `map.json`（玩法内容）
= `board.json` `maps.<id>` 的内容面：`grid_w` / `grid_h` / `spaces[]`；每格
`{ "cell": [row, col], "kind": "…", "group": N, "terrain": "S|R|L|T", "isle": "<isle_id>"? }`。
- `kind` 取值集合 = `rules::board::SpaceKind::key()`（`start/plain/port/city/event/card/battle/
  treasure/territory/boss`）；**未知 kind = 硬失败**（不 fallback）。
- `isle` 仅 `kind=territory` 有值；`isle_id` 必须已注册于 `balance.territory.prototypes`。
- `cell` = 等距格坐标（保留原版地图形状；表现层 v5 环用格序，格坐标供规则层沿用）。

### 3.2 `layout.json`（版面参数）
环几何 + 焦点 + 中央岛 + CTA/键位；键名与 `scenes.json` `main_map` 对齐（同一键空间）。
- 环几何：`ring.{center_x,center_y,radius,card_w,card_h}`（`extra_scalar` 点号口径）。
- 表现层读包内 `layout`；**无包/未激活时回退 `scenes.json main_map`**（内嵌默认包）。

### 3.3 `strings.json`（该图文案）
键空间与全局 `strings.json` 相同（`isle.*` / `dungeon.*` / `scene.main_map.node.*` …），
**值由包自带** ⇒ 换包即换文案，不污染全局。

## 4. 资产规范

- **命名约定**：岛卡面 = `isle_<isle_id>.png`；中央岛 = `island_home_<map_id>.png`；
  地形图 = `terrain_<map_id>.png`（可选）。
- **两种来源（`assets[].source`）**：
  - `pack` = **该图专属**（随包分发，物理落在包内 `assets/`）：中央岛、该图专属地形图等。
  - `pool` = **共享池**（全图共用，物理在 `assets/ui/`，**不入包**）：10 类岛卡面等跨图复用资产。
- **为什么分两种**：岛卡面（10 张 ≈ 5.9MB）三图共用，入包即 3 份副本（+21MB 仓库膨胀）；
  专属资产（中央岛 ≈ 1.2MB/图）才入包。**两种来源的完整性口径一致**（都记 sha256 + bytes，verify 都校验）。
- **登记口径**：安装时把 `source=pack` 的资产登记进 `ui_manifest.json`（构建期默认包路径）；
  `source=pool` 的资产**已在全局池登记**，不重复登记。运行期按 `source` 解析位置。

## 5. 不变量（可机器判定，`verify` 逐条查）

| # | 不变量 | 判据 |
|---|---|---|
| I1 | 赛道 = **31 格** | `spaces.len() == 31` |
| I2 | 构成 = 起点 1 + 领地 20 + 功能 9 + BOSS 1 | 按 `kind` 计数：`territory`=20、`start`=1、`boss`=1、功能格（`event`/`card`/`treasure`/`port`/`city`…）=9 |
| I3 | 10 类岛各 **2 格且同属一个领地** | 按 `isle` 计数 = 每类 2；两格 `group` 相同 |
| I4 | 岛型已注册 | 每个 `isle` ∈ `balance.territory.prototypes[].id` |
| I5 | 格序闭合 | 首格（`spaces[0]`）`kind=start`；BOSS 在**环半周** `spaces[31//2]`（第 15 格 = 摇半圈正对 BOSS；末格无不变量） |
| I6 | 环几何 8pt | `ring.center_y`/`radius`/`card_w`/`card_h` 8 整除（`center_x` 屏心 540 豁免） |
| I7 | 资产齐全 + 哈希一致 | manifest `assets[]` 每项（`source=pack` 与 `source=pool` 都算）：文件在位、`bytes` 一致、`sha256` 一致 |
| I8 | 焦点 N8 往返 | `layout.world_canvas` ≡ manifest `focus.bbox_pct` × (1080,1920) ±1px |

> I1~I5 = 赛道结构不变量（`docs/business/territory.md` §2 的数据面）；I6~I8 = 版面/完整性不变量
> （与 `check_scene_narrative` N8、`doccheck2` 8pt 同一口径）。

## 6. 版本与兼容 + 失败语义

| 情形 | 行为 |
|---|---|
| `schema` 不受支持 | **拒绝安装**（工具非 0）/ 运行期**跳过该包** |
| `data_version` 不受支持 | 同上 |
| `version` 较低（降级安装） | 允许，`install` 记警告；`version` 较高 ⇒ 需 `--force` |
| 资产缺失 / 哈希不符 / 字节数不符 | **拒绝安装**（工具非 0）/ 运行期**跳过该包** |
| 不变量 I1~I8 任一不满足 | **拒绝安装**（工具非 0）/ 运行期**跳过该包** |
| 运行期该包加载失败 / 未就绪 | **回退内嵌默认包**（`scenes.json` + `board.json` 内嵌版）⇒ **可玩性优先，不崩** |

> 「跳过该包」= 不激活，继续用当前激活包；「回退内嵌默认包」= 无激活包时的兜底。

## 7. 运行期加载契约（选定 B：运行期换图）

| 环节 | 契约 |
|---|---|
| 资产根 | Bevy `AssetPlugin.file_path` = 仓库根（native）/ 部署根（wasm）⇒ 包目录**按相对路径直读** |
| 加载时机 | 启动后异步加载已装包的 `manifest.json` → `map.json`/`layout.json`/`strings.json` → 校验（I1~I8）→ 注入数据层 |
| 首帧 | **不阻塞首帧**：未就绪先用内嵌默认包（现状行为），就绪后热切 |
| 注入点 | `ti_core::data` 访问器（`board()`/`scenes()`/`strings()`/`terrain()`）**可注入源**；**无注入 = 现状内嵌**（零回归） |
| 切包 | `data::activate_pack(id)` → 数据层换源 + 表现层重取 `layout`；规则层零改造（生产路径吃 `&BoardMap` 参数） |
| 打包边界 | 数据 = 运行期可换（需注入通道）；**资产** = 运行期按路径加载（`web/` 部署链须拷贝 `assets/maps/<id>/`） |

## 8. 工具与流水线

工具 = [`scripts/map_pack.py`](../scripts/map_pack.py)（台账登记见 `14_tooling_catalog.md` §3.2）。

| 子命令 | 作用 | 幂等/退出码 |
|---|---|---|
| `pack <id>` | 已装包/源 → **可分发件** `dist/maps/<id>.map-pack/`（+ `.zip`），写清单哈希 | 幂等（同输入同产物）；失败非 0 |
| `verify <path>` | 校验包：schema/兼容/资产齐+哈希/I1~I8 | 只读；违规非 0 |
| `install <path>` | 装包 → `assets/maps/<id>/` + 接线（默认包数据 / 资产登记 / 文案合并） | 幂等（upsert）；违规非 0 |
| `uninstall <id>` | 卸包（保留默认包数据引用） | 幂等；不存在即 0 |
| `list` | 列出已装包（id / version / data_version / 校验结果） | 只读 |
| `check` | 已装态 ↔ 内嵌默认数据漂移对账 | 只读；漂移非 0 |

```powershell
# 加图 / 换图 = 出一包 + 装上
python scripts/map_pack.py pack m01
python scripts/map_pack.py verify dist/maps/m01.map-pack
python scripts/map_pack.py verify m01        # 裸 id = 已装包目录（S32.3 平台制作指引面）
python scripts/map_pack.py install dist/maps/m01.map-pack
python scripts/map_pack.py list
```

## 9. 产物落点

| 产物 | 落点 | 说明 |
|---|---|---|
| **已安装包** | `assets/maps/<id>/` | 仓库内态 = 运行期可直读（native 根=仓库根；wasm 需拷进 `web/assets/`） |
| **可分发件** | `dist/maps/<id>.map-pack/`（+ `.zip`） | 交付物 = 「这张图拿去装」的单体 |
| **构建期默认包** | `assets/data/{board,scenes,strings}.json` + `assets/ui_manifest.json` | 内嵌 fallback（`include_str!`），保证无包也能玩 |

## 10. 与既有体系的关系（谁是事实源）

| 主题 | 运行期事实源 | 构建期默认 | 关系 |
|---|---|---|---|
| 赛道格序 | 包内 `map.json` | `board.json` | 默认包 = 内嵌副本；运行期以包为准 |
| 主地图版面 | 包内 `layout.json` | `scenes.json main_map` | 同上（`chrome::scene_layout` 走注入源） |
| 该图文案 | 包内 `strings.json` | `strings.json` | 同上（键空间相同，值由包覆盖） |
| 该图资产 | 包内 `assets/`（专属）+ 资产池（共享，manifest 声明） | `ui_manifest.json` + `assets/ui/` | 同上（`ui_manifest` 只登记默认包） |
| 贴图分辨率档 | `assets_hd_pack/`（媒体包） | 同 | 两包体系独立：媒体包管分辨率，地图包管玩法+版面 |

## 11. 门禁耦合点（改包格式必须同改）

| 门禁 | 锚点 | 要求 |
|---|---|---|
| `doccheck2` 第 26/27 项 | `scripts/map_pack.py` | 台账表格行 + 幂等/只读/非 0 退出声明 |
| `check_scene_narrative` N8 | `scenes.json focus_anchor` ↔ manifest `focus.bbox_pct` | 默认包与包内 layout 双向一致（±1px） |
| `doccheck2` 8pt | `scenes.json extra_scalar` | 环几何 8 整除 |
| `validate_data` `check_scenes_canonical` | `scenes.json` | indent=1 逐字节规范形（工具写出须同口径） |
| `check_code_standards` CS-1/3 | 资产路径字面量 / 参数单字母 | 资产路径只在 `chrome.rs`；工具参数禁单字母 |

## 12. 未决项

| # | 未决 | 影响 | 当前处置 |
|---|---|---|---|
| U1 | 包内 `strings.json` 与全局 `strings.json` 同键不同值的**合并优先级** | 换包后全局键是否残留 | 包值覆盖全局同名键；**全局多余键保留**（不影响运行期） |
| U2 | 运行期已装包的**发现方式**（扫目录 vs 读索引文件） | wasm 端能否列目录 | 以 `assets/maps/<id>/manifest.json` **逐个探测**（不依赖目录列举） |
| U3 | wasm 端 `AssetServer` 读包文本的加载器实现 | 首帧时序 | 自定义 `AssetLoader`（扩展名 `json`/`.map-pack`） |
| U4 | 包的**签名/来源可信**（是否需要） | 第三方包接入 | 暂不做（内部工具链）；哈希只保完整性，不保来源 |
