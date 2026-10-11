# 实现文档总索引

本文档是 Rust 重建项目的**实现级入口**。读者仅凭本目录 17 份文档（00-16）即可从头实现完整游戏（含 HD 资产管线 + 现代多端实现）。

---

## 1. 游戏概述

| 项 | 值 |
|---|---|
| 游戏名 | 金银岛 TreasureIsl |
| 类型 | 回合制策略经营 + 探索冒险 |
| 原平台 | Windows Mobile（ARM PE32，176×220 逻辑分辨率） |
| 目标平台 | 浏览器（WASM）+ Android + 桌面 |
| 玩家角色 | 5 选 1（panda/tichy/kidd/drake/katharina） |
| 地图 | 3 张等距网格地图（20×20 / 20×20 / 15×15） |
| 核心循环 | 探索 → 据点/事件 → 贸易赚金 → 建造升级 → 卡牌事件 → 战斗挑战 |

---

## 2. 文档结构

| 文件 | 内容 | 行数 |
|---|---|---|
| [00_index.md](00_index.md) | 本文件：总索引与导航 | — |
| [01_architecture.md](01_architecture.md) | 系统架构、模块依赖、核心类型定义、帧循环 | ~400 |
| [02_scene_flow.md](02_scene_flow.md) | 场景状态机、事件码分派、输入映射、全局单例 | ~450 |
| [03_map_system.md](03_map_system.md) | 地图数据结构、移动算法、等距投影、摄像机 | ~400 |
| [04_economy.md](04_economy.md) | 贸易系统、建造系统、银行系统、全部数值表 | ~450 |
| [05_combat.md](05_combat.md) | 战斗系统（BOSS 档/伤害公式/回合制/胜负判定） | ~350 |
| [06_cards_events.md](06_cards_events.md) | 卡牌/事件系统（效果码/事件 ID 映射/生成算法） | ~350 |
| [07_ui_controls.md](07_ui_controls.md) | 消息界面、控件表、资源容器、渲染管线 | ~300 |
| [08_input_mapping.md](08_input_mapping.md) | 输入系统（键盘/触摸/手势/场景行为表） | ~300 |
| [09_persistence.md](09_persistence.md) | 持久化（文件布局/魔数/v1/v2 迁移/校验） | ~400 |
| [10_rng.md](10_rng.md) | RNG 系统（LCG 算法（oracle 历史参照）/PCG（当前实现）/播种点/调用点/确定性） | ~300 |
| [11_modernization.md](11_modernization.md) | 现代化方案（双层渲染/缩放/DPI/虚拟手柄/PWA/Android） | ~350 |
| [12_audio.md](12_audio.md) | 音频系统（BGM/音效/音量/静音/浏览器策略） | ~370 |
| [13_appendix.md](13_appendix.md) | 附录（全常量表/事件码/场景码/错误类型/资源清单） | ~530 |
| [14_asset_enhancement.md](14_asset_enhancement.md) | HD 资产管线（超分/矢量化/多分辨率打包方案） | ~300 |
| [15_gameplay_full.md](15_gameplay_full.md) | **完整玩法规格**（全系统实现手册：场景状态机/事件码/核心循环/角色地图/主地图/贸易/建造/战斗/卡牌/消息/存读档/RNG/输入/音效/数值平衡） | ~600 |
| [16_modern_platform.md](16_modern_platform.md) | **现代多端实现指南**（架构分层/双层渲染/响应式布局/触屏/字体/音频/持久化/PWA/Android/资源分包/性能预算/错误处理） | ~500 |

---

## 3. 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                        GameApp                               │
│  ┌──────────┐  ┌───────────┐  ┌──────────┐  ┌───────────┐  │
│  │ shell    │  │ scenes    │  │ input    │  │ settings  │  │
│  │(Framebuf)│  │(SceneStack│  │(KeyState)│  │(Settings) │  │
│  └────┬─────┘  └─────┬─────┘  └────┬─────┘  └─────┬─────┘  │
│       │              │              │              │         │
│  ┌────▼─────┐  ┌─────▼─────┐  ┌────▼─────┐  ┌────▼─────┐  │
│  │ atlas    │  │ game_state│  │ rng      │  │ save_sys │  │
│  │(SpriteAt)│  │(GameApp*) │  │(GameRng) │  │(SaveSys) │  │
│  └──────────┘  └───────────┘  └──────────┘  └──────────┘  │
│  * S8.4 起无独立 GameState struct，各字段直接内嵌 GameApp（game_app.rs:180-248）
└─────────────────────────────────────────────────────────────┘
```

---

## 4. 模块依赖

```
ti-formats (资源解析)
    ↑
ti-core (核心类型/常量/规则)
    ↑
ti-platform (平台抽象: 输入/存储/音频)
    ↑
ti-app (游戏逻辑: 场景/状态机/经济/战斗)

ti-tools (工具: 台账/doccheck/controls) —— 独立工具 crate，仅依赖 ti-formats + ti-platform，
     不依赖 ti-app / ti-core（无业务代码依赖，供开发管线使用）。
```

---

## 5. 数据流

```
资源文件 → ti-formats 解析 → SpriteAtlas/MapTerrain/PriceTable
                                     ↓
输入事件 → ti-platform → KeyState → ti-app 场景状态机
                                     ↓
                               GameApp 字段更新（S8.4 起无独立 GameState，各字段内嵌 GameApp）
                                     ↓
                               shell 渲染 → 帧缓冲
```

---

## 6. 核心类型速查

| 类型 | 定义位置 | 说明 |
|---|---|---|
| `GameApp` | 01_architecture.md §3 | 顶层应用结构 |
| `Scene` | 01_architecture.md §2.1 | 场景枚举 |
| `GameState` | 01_architecture.md §2.5（文档占位；S8.4 起各字段直接内嵌 `GameApp`，无独立 struct） | 全局游戏状态 |
| `SceneStack` | 01_architecture.md §4 | 场景栈管理器 |
| `EventCode` | 02_scene_flow.md §3 事件码表（数值码，非枚举名） | 事件码定义（13/27/37/38/39/40） |
| `MapData` | 03_map_system.md §2（文档占位类型；crates 无此 struct，实际类型见 `rules::main_map::MapTerrain` / `formats::mapfile::MapScene`） | 地图数据结构 |
| `GridPos` | 01_architecture.md §2.4 | 网格坐标 |
| `Direction` | 01_architecture.md §2.3 | 方向枚举 |
| `TradeLedger` | 04_economy.md §3 | 贸易账本 |
| `BuildManager` | 04_economy.md §3 | 建造管理器 |
| `Bank` | 04_economy.md §3 | 银行系统（`Bank` struct 定义见 §3，行为见 §2.5） |

---

## 7. 实现优先级

1. **P0 地基**：`ti-formats` 解析 + `ti-core` 常量/类型
2. **P1 场景**：场景状态机 + 主菜单 + 角色/地图选择
3. **P2 地图**：地图渲染 + 移动 + 摄像机
4. **P3 经济**：贸易 + 建造 + 银行
5. **P4 战斗**：战斗系统 + 卡牌事件
6. **P5 持久化**：存读档 + 设置

---

## 8. 验收标准

1. **场景全覆盖**：9 个场景（Intro..Exit，Exit 为终态）均可进入/返回，无死锁
2. **输入正确**：每个按键在每个场景的行为符合映射表
3. **资源注入**：ui.pak + 9 个场景 .Map 正确加载并渲染
4. **三端编译**：native/wasm/android 均可编译
5. **数值精确**：价格表、BOSS HP、伤害公式、RNG 序列逐位一致（当前实现为 PCG，`rand_pcg`，S8.7；旧 MSVC LCG 序列为 oracle 历史参照）
6. **全流程**：开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单

---

## 9. 引用关系

| 本文档引用 | 来源 |
|---|---|
| 场景枚举/状态机 | `docs/design/main_loop.md` |
| 事件码体系 | `docs/design/scene_flow.md` |
| 地图数据结构 | `docs/design/map_system.md` |
| 经济系统 | `docs/design/economy.md` |
| 全局常量 | `docs/design/constants.md` |
| 游戏设计总纲 | `docs/game_design.md` |
