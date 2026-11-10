# 14 地块格规范（Plot Tile Spec）—— 单一事实源

> **改 main_map「地块格」视觉/资产/生图前必读**。本规范收口 v5「岛型卡」/ v6「节点牌/插旗」
> 的散落旧措辞，定案「一个地块格 = 底 + 地块图 + 归属色带」三层模型。
> 用户 2026-11-10 三轮定案：①「格子卡图是过渡设计」→ 地块图按**格**独立（非按岛型共用）；
> ②「格子里有船」→ 地块图**只画该岛标志物**，禁船/旗/角色/动物/文字（这些属运行时叠层）；
> ③「重做格子/地块，只画标志物（像牌）」→ 牌面风格。
> 旧文档 `10_main_map_v5_spec.md`「岛型卡 96×120」、`12_main_map_v6_vs_v2_spec.md` §1.2「插旗」
> 的「卡/旗」措辞**由本规范取代**；「格子 = 多边形地块」几何仍按 12 号 §1（不规则岛 + 格子紧围岛）。

## 0 北极星

**地块格** = 主地图 31 格赛道的**一格**。一格由**三层**叠出（z 序自下而上）：

```
┌─────────────────────────────┐
│  ③ 归属色带（占/无主）      │  程序绘制 · 逐局动态
│  ② 地块图 tile_<map_id>_<格号>.png │  每格专属 · AI 生图（只画该岛标志物）
│  ① 格子底（多边形 + 海水缝） │  程序绘制 · 真不规则 6 顶点
└─────────────────────────────┘
        ── 之上再叠运行时层：玩家棋子（船/旗/点）/ 归属名牌 / 驻军角标 ──
```

- **地块图（②）** = 一格**一张专属图**（`tile_<map_id>_<格号>.png`，命名约定自动发现，0 描述符改动，同 `island_home_<map_id>.png` 口径）。
- **只画该岛的「标志物」（牌面风格）**：港口=码头/缆桩/灯塔剪影、灯塔=石塔、火山=山、沉船=残骸、骷髅崖=尖岩…
- **禁画**：航行中的船、旗帜、海盗角色、动物、文字 —— 这些属**运行时叠层**（玩家棋子等），**不属于地块**（用户「格子里有船」根因 = 旧提示词诱导 AI 画船）。
- **缺图 = 回退**：该格无专属图 → 程序绘制 ①+③（羊皮多边形 + 归属色带），0 视觉变化。

## 1 三层定义

| 层 | 内容 | 来源 | 动态性 |
|---|---|---|---|
| ① 格子底 | 真不规则 6 顶点多边形（seed 随机）+ 格间海水缝 + 投影 | 程序 `v6_tile_polygon`（`main_map.rs`）；布局 `main_map_v6_layout.json` | 每格形状固定（seed） |
| ② 地块图 | `tile_<map_id>_<格号>.png`（1024² RGBA，品红抠透明） | AI 生图 `scripts/agnes_gen.py` → `ui_assets.py` 登记 | 每格可独立换 |
| ③ 归属色带 | 内环色带（已占=岛色 `isle_*` / 无主=灰 `tile_deny`） | 程序 `main_map.rs`（逐局 `territory` 随机） | 逐局动态 |

**运行时叠层（非地块，属玩家/状态）**：玩家棋子（当前玩家船 `V6_SHIP` / 其余暗旗 `flag`，`seat_slots` 扇出）、归属名牌（`entity_texts`）、驻军/等级角标。**这些永远画在地块图之上，与地块图无关。**

## 2 地块图生成规范

- **唯一生图入口** = `scripts/agnes_gen.py`（04_tooling §6）；平台登记 = `scripts/ui_assets.py register-plot`（唯一清单写入路径，EXP-184 零 churn）。
- **提示词模板**（负向清单统一）：
  - 通用（无岛型时）= `ED_PLOT_GEN_PROMPT`：`…俯视孤立小岛地表…负向：禁止船/旗帜/海盗角色/动物/文字；纯透明，居中，牌面风格`。
  - 每岛型（10 类）= `ED_ISLE_GEN_PROMPT`：`…该岛标志物（纯地标，无船）…` + `ED_PLOT_NEG`（统一负向尾）。
  - 清单模板 = `ui_manifest.json roles.plot.prompt`（英文：`NO ships, NO flags, NO pirate characters, NO animals, NO text`）。
- **负向清单（硬约束，缺一即返工）**：`no sailing ship / no boat / no flag / no pirate character / no animal / no text / no figure`。**例外**：沉船残骸、灯塔剪影等**静态地标**可留（是「标志物」不是「航行物」）。
- **抠图**：品红底 → `ui_assets.py` `key_magenta`（去紫边）→ 透明 RGBA；入 `assets/ui/tile_<map_id>_<格号>.png`。
- **格号 = 环序索引**（0=起点，`board.json`/`spaces` 与 `tiles` 1:1 对应，31 格 → idx 0..=30）。

## 3 命名与 0 描述符约定

- 文件：`assets/ui/tile_<map_id>_<格号>.png`（**无零填充**，格号 = 环序 idx；与 `ui_assets.py register-plot` / 真渲染 `tile_plot_path` 一致）。
- 真渲染自动发现（`main_map.rs::tile_plot_path`）：`map_id` 来自 `host.map_slot()`，缺图回退程序绘制 → **加图/换图 = 改数据 + 换图，0 Rust 改动**（内容运营判据；if-blit 回退例外已认可）。
- 清单登记 `roles.plot` + `assets[]` 条目 `role=plot`（`ui_manifest.json`，`validate_data::check_asset_registry` 双向同步守门）。

## 4 编辑器（地图编辑器 v3 仪表盘）

- **当前地块卡**（`#ed-plot-card`）：选中格 → 该格缩略图（`tile_<map_id>_<格号>.png`）+「🎨 生成本地块图」。
- **左侧岛型面板** `✎` = **取该岛提示词**（填入当前选中格的地块图生图框，可复用修改；不生成、不改选岛）。
- **右侧入口**：「中央岛图」+「地块图」（= 当前选中格）两按钮 → 弹模态（提示词编辑框 + 预览 + 应用保存）。
- **应用保存后**：`edGenBust` 破缓存 → 地块卡缩略图 + 画布 blit 立即刷新。

## 5 机器判据（验收）

| 判据 | 载体 |
|---|---|
| 地块图无船/旗/角色（负向清单） | 提示词模板含负向尾（`ED_PLOT_NEG` / `roles.plot`）；人工过目样张 |
| 按格独立（非按岛型共用） | 文件命名 `tile_<map_id>_<格号>.png`；`ui_manifest` `role=plot` 每格一条 |
| 缺图回退 0 视觉变化 | `main_map.rs::tile_plot_path` `asset_present` 判 + `--shot main_map` 像素对拍 |
| 真渲染同显示 | `TI_MAP=<id> --shot main_map` 出 `evidence/bevy_native_main_map.png`（有图格 vs 回退格像素分布不同） |
| 清单登记零 churn | `ui_assets.py register-plot`（EXP-184）；契约测试 `ui_manifest_register_tests`（幂等 + 逐字节不变） |

## 6 与旧规范关系（取代口径）

- `10_main_map_v5_spec.md`「岛型卡 96×120」= v5 定案，**已被 v6 不规则地块取代**（本规范 §0/§1 为准）。
- `12_main_map_v6_vs_v2_spec.md` §1.2「可占领牌+插旗 / 功能格锁链」中「**插旗**」措辞**退役**：归属用**色带**表达，**旗帜属运行时叠层**，不烤进地块图。
- `business/territory.md`（领地格/功能格/31 格赛道/10 类岛）= 业务规则单一事实源（不变；本规范只定「地块格视觉/资产」）。
- `business/08_layout_design.md` §4.3（坐标）= 版面坐标（不变；本规范只定「一格三层模型 + 地块图」）。
