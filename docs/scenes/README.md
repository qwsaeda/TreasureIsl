# 七场景详细图文拆解（S76 · 总览索引）

> 本目录 = 7 个有实体界面的游戏场景的**逐元素图文拆解**，每场景一份文档。
> 数据源单一事实源：`assets/data/scenes.json`（1080 坐标/原语/参数）+ `assets/data/ui_schema/scene_manifest_v3.json`（叙事/版面约束）+ `docs/business/08_layout_design.md`（坐标表）+ 各场景源码 + `evidence/bevy_native_<scene>_*.png`（**引擎真实渲染截图**，项目验收口径，非 AI 概念图）。
> 模板与纪律见实施期 `docs/scenes/_TEMPLATE.md`（收口后删除）。

## 1. 场景索引（7 份详细文档）

| 场景 | 文档 | 职责 | 竖屏原语 / 横屏原语 | 真实截图 |
|---|---|---|---|---|
| `index` | [`index.md`](index.md) | 首页/标题页（竖屏罗盘环 S22 → 横屏羊皮藏宝图 S39） | 42 / 22 | `bevy_native_index_*.png` |
| `select_role` | [`select_role.md`](select_role.md) | 选角（角色立绘 + 列表 + 铜牌） | 45 / 31 | `bevy_native_select_role_*.png` |
| `select_map` | [`select_map.md`](select_map.md) | 选图（地图包卡片 + 锁/开放态） | 31 / 13 | `bevy_native_select_map_*.png` |
| `main_map` | [`main_map.md`](main_map.md) | 主地图（环岛赛道 + 领地 + 中央据点岛 + 8 面板） | 7 / 6（世界层代码绘制） | `bevy_native_main_map_*.png` |
| `vs` | [`vs.md`](vs.md) | 对战（立绘对峙 + 战斗动画 + 道具/兵力 HUD） | 19 / 16 | `bevy_native_vs_*.png` |
| `message` | [`message.md`](message.md) | 消息/帮助（面板正文 + 翻页） | 5 / 1 | `bevy_native_message_*.png` |
| `saveload` | [`saveload.md`](saveload.md) | 存读档（5 槽位 + 选中态） | 13 / 8 | `bevy_native_saveload_*.png` |

## 2. 九场景态图（游戏状态机）

游戏有 9 个 `Scene` 变体（`crates/ti-app/src/game_app/state.rs`），7 个有实体界面，2 个是逻辑态：

```
              ┌─────────────────────────────────────────────┐
              │                                             │
              ▼                                             │
          [Intro]  →  [Index]  ──┬─→ [SelectRole] → [SelectMap] → [MainMap]
          逻辑态     首页/标题     │                             │  │
                                  ├─→ [Message] 帮助            │  ├─→ [Vs] 战斗
                                  └─→ [Saveload] 存读档          │  │    （打完回 MainMap）
                                                             │  └─→ 8 面板（贸易/建造/造船/事件/驻军/升级/副本/图鉴/成就）
                                                             │
                                                        摇骰 → 落格 → 结算
                                                             │
                                                          [Exit] 逻辑态
```

- **流转规则**：20 条合法转换，非法转换编译期拒（Type-State 保证，见 `docs/business/01_game_logic_overview.md` §场景态图）。
- **主循环**：`Index → SelectRole → SelectMap → MainMap → (Spin 摇骰 → 落格结算 → 领地/副本/战斗) → …`
- 每个场景文档 §1 写自身进入/退出条件与前后场景。

## 3. 布局是如何组合而成的（通用链路）

> 单一事实源：[`docs/process/17_layout_engine.md`](../process/17_layout_engine.md) §1 B1 架构 + §2 三条松绑（S29 定案）；本段只做索引式摘要，不复述。

```
assets/data/scenes.json（每场景原语清单 + 参数）
  + assets/data/ui_schema/scene_manifest_v3.json（叙事/版面约束）
  + assets/data/*.json（balance / board / maps 地图包）
  + assets/ui_manifest.json（贴图/图标登记）
       │
       ├─ ① 游戏世界层（1080 正交，数据驱动）：
       │     场景 render 按 z-order（调用顺序）画地图/环格/实体/HUD
       │     换图 = 参数化地图包（ring 几何 + 31 格 + 命名约定资产），零 Rust
       │
       └─ ② UI chrome 层（bevy_ui 自动布局）：
             build_data_chrome 把原语 → 节点树（按 z 排序 + when 选中态过滤）
             flex column + gap + % 定位 ⇒ 窗口 resize 真重排（不手写像素）
             5 个纯 UI 场景（index/select_role/select_map/saveload/message）走此层
            ─────────────────────────────────────────────
             双相机叠加：世界相机 + UI 相机（RenderLayers::layer(1) 隔离，clear None）
             命中：UI 控件走 bevy_picking/Interaction；游戏内容走 pick_key/pick_index → HitData
```

**为什么 main_map / vs 的游戏世界留 1080 而不进 bevy_ui**：S29.3e 判定——bevy_ui UI 相机叠在世界相机之上，同屏位会遮住动态实体（棋盘/立绘/HP 条）；游戏世界保持 1080 正交 + 数据驱动，chrome 才进 bevy_ui。见 `19_bevy_ui.md` 坑位 9.8-13。

### 3.1 各场景「不手写像素」落地状态（2026-11-23 实测）

总纲 = `docs/process/18_asset_content_pipeline.md` §15「不手写像素：声明意图/数据，引擎算坐标」。
**关键区分**：该治的是「**规律排列的重复结构**被逐条手写坐标」；单件元素（背景/标题/单面板）写坐标是合理的（无可抽参数）。
下表为逐场景实测（`scenes.json` 原语的 `derived` / `ui_layer` 标记统计 + Rust 构建方式）：

| 场景 | 重复结构 | 位置真正来源 | 死坐标风险 | 参数化状态 |
|---|---|---|---|---|
| `index` | 6 枚木牌/图标（横屏 18 行） | S85 后 = `components_v1.json` 的 `index.elements`（list `binding=menu` + role 意图） | 无 | ✅ 声明式 schema + role 意图（S85.5 已删旧 `index_param_rect`/死 prim） |
| `select_role` | 5 行角色 | S85 后 = `components_v1.json` 的 `roster` list（`binding=characters` + `action=character_select`） | 无 | ✅ 声明式 schema（S85.5 已删旧 `build_list_chrome`/死 prim） |
| `select_map` | 20 张地图卡 | S85 后 = `components_v1.json` 的 `maps` list（`binding=map_slots` + `action=map_select`） | 无 | ✅ 声明式 schema（S85.5） |
| `saveload` | 5 槽位 | Rust `build_list_chrome_pos(PanelSide::Left)`，5 原语标 `ui_layer=bevy_ui` | 无 | ✅ 自动布局（S29.3b） |
| `message` | 无（单面板） | Rust `build_message_chrome`（% 居中 + 文本自动折行），4 原语标 `ui_layer=bevy_ui` | 无 | ✅ 意图化（S29.3e） |
| `vs` | 无重复列表 | 代码绘制立绘/HP 条（运行时），游戏世界留 1080（S29.3e） | 无（单件） | 合理写坐标（无可抽重复参数） |
| `main_map` | 31 格环 | `ring.{center_x,center_y,radius,card_w,card_h}` + `ring_card_center` 运行期算 | 无 | ✅ 参数化（S27.4 早已做） |

**结论**：除 `index`（原手写重复结构，S80 机器断言 + S81 运行期参数化已修）外，其它场景要么早已自动布局（4 个纯 UI 场景）、要么早已参数化（main_map 31 格环）、要么单件无需参数化（vs）。
注意：`select_role`/`select_map`/`saveload` 的 `scenes.json` 原语 rect 仍是非零手写值，但那是 **CPU 世界层版面规格 / 人读镜像**（`ui_layer=bevy_ui` 原语在 Bevy 壳被跳过），真正位置由 Rust flex 算——按 `17_layout_engine.md` §S29.4 口径合规，不是「渲染依赖的死坐标」。

> 机器守门：doccheck2 第 57 项 `no-handwritten-grid` 直接对账 scenes.json——带 `derived=menu` 的横屏木牌原语 x/y=0（死坐标已删）+ `index.menu` 参数在位；`scripts/validate_data.py` V2 守参数键合法性（含 `_dx/_dy` 负偏移豁免）。旧 `gen_index_medallions.py`（服务竖屏 index.ring）已随 S84 纯横屏退役。

## 4. 每份场景文档的 10 固定章节（通用阅读法）

1. 场景职责与状态机位置（进入/退出、前后流转）
2. 真实渲染截图（横屏 `1920×1080` + 竖屏 `1080×1920`，引擎出图命令与视口）
3. ASCII 布局图（两张：竖屏/横屏，框线标主要区域）
4. 逐元素坐标表（scenes.json 原语逐条：kind/rect/z/when/params；main_map 额外列几何参数表）
5. 横屏差异（竖屏 vs 横屏原语数/坐标、landscape 专属参数）
6. 资产接线（PNG 路径 → ui_manifest 登记；export-icon-boundary 守门）
7. 命中契约（pick_key/pick_index → HitData → route_hit）
8. 业务规则（docs/business/ 对应系统，引用 ID 不复述）
9. 测试覆盖（相关 e2e + 出图/像素门禁）
10. 数据驱动改法（换 UI/改坐标/换资产的具体步骤 + 零 Rust 证据）

## 5. 出图口径（复现截图）

真实渲染截图由 `--shot` 离屏模式产出（`crates/ti-engine/src/bin/treasure_bevy.rs`）：
```
xvfb-run -a cargo run -q -p ti-engine --bin treasure-bevy-native -- \
  --scene=<scene> --shot --viewport=1920x1080
```
设备矩阵批量出图 + 机检（非黑/探针/安全区）走 `python3 scripts/ui_verify.py`（S67.0，14号台账登记）。**AI 概念图（`evidence/ui_concept/`）仅作设计参考，不作验收图**。

## 6. 登记与治理

- 本目录 7 份场景文档 + 总览（本文件）由 S76 系列任务产出，进度见 `plan/ledger.yaml`。
- 文档治理（单一事实源/长行禁令/doccheck 门禁）见 `docs/README.md` §文档治理约定。
