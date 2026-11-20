# S63 · 补全业务子场景（让 rules 层能力真正可玩）

> 用户 2026-11-20 拍板「场景不全，补全子场景 WBS」。本文 = 差距分析 + WBS + 分阶段验收判据。
> 单一事实源：业务规则 = `docs/business/*.md`（economy/trade/build/territory/combat/progression）；
> 场景枚举 = `ti_app::Scene`；UI 呈现 = `scenes.json`（数据驱动）+ `ti-engine` bevy_ui；
> rules 能力 = `crates/ti-core/src/rules/*.rs`。

## 1. 现状差距（用户 3 问题的回答）

| 问题 | 现状 | 性质 | 处置 |
|---|---|---|---|
| ① 地图全锁 | **已修**（S62，commit `590828d`）：20 图开局全解锁，单调链废弃 | 业务规则 | ✅ 完成 |
| ② 角色/Vs 渲染 | 横屏图渲染正常，问题是立绘偏小/留白/比例 | **数据+资产**（`scenes.json` 坐标 + 换图），零 Rust | 用户给方向 → 改数据（本 WBS 不含，单独轮） |
| ③ 场景不全 | `Scene` 只有 8 态；贸易/建造/副本/据点/事件/卡牌 是 MainMap 上的**子面板**，S34 删底部三键后 `deck_trade`/`deck_sell`/`deck_build` 变**空壳**（只剩矩形无原语）；app 壳只接了 CardEngine/buy_crew/effect + 建造面板键盘交互，**TradeLedger/BuildManager 完整 UI 缺失** | **业务逻辑接 UI 缺口**（非纯美术） | 本 WBS |

## 2. 差距清单：rules 有能力 vs UI 不可玩

| 业务能力（rules 层有） | 规则文档 | app 壳接线现状 | UI 缺口 |
|---|---|---|---|
| 贸易 `TradeLedger::buy/sell/sell_price` | `economy.md` | ❌ 无 UI 入口（`deck_trade`/`deck_sell` 空壳） | 贸易面板：买/卖商品、数量、价格、金币 |
| 建造 `BuildManager::build/cost_of/build_atk_bonus` | `economy.md` | ⚠️ 仅 `build_panel_open` 键盘交互（`on_input`），面板渲染 `scenes.json` 无原语 | 建造面板：建筑列表/造价/加成/确认 |
| 船只 `BoatSession::upgrade/level/cargo` | `economy.md` §船系统 | ❌ 无 UI 入口 | 造船面板：船级/造价/货舱 |
| 副本 `apply_dungeon` / `TerritoryRules::defense_at` | `territory.md` | ⚠️ 落格自动结算（无玩家交互面板） | 副本面板：守军/奖励/进入确认 |
| 据点升级 `apply_upgrade/upgrade_cost_at` | `territory.md` | ⚠️ AI 自动升级，玩家落自己格交互缺 | 领地面板：升级加固/产出/驻军 |
| 事件 `events::effect` / 卡牌 `CardEngine::apply` | `progression.md`/`combat.md` | ✅ 接线（`effect()`/`apply_card`） | 事件/卡牌结果提示签条已在（board_toast），可增强 |
| 商人 `buy_crew/buy_arms` | `economy.md` | ✅ 接线（`buy_crew`/`buy_arms`） | 商人面板（可增强展示） |
| 存读档 `Saveload` | `persistence.md` | ✅ 独立 Scene + 5 槽 UI | 无缺口 |

**结论**：核心缺口 = **贸易 / 建造 / 造船 / 副本 / 领地升级 5 个子面板**（rules 全有，UI 无/空壳）。

## 3. 设计原则（约束）

1. **不新增 Scene 枚举**（用户要"场景全"= 子面板完整，不是加顶层态）；子面板 = MainMap 上的叠加层（bevy_ui 节点），关掉回 MainMap（`docs/process/19_bevy_ui.md`）。
2. **数据驱动**：面板版面 = `scenes.json` 新段（如 `main_map.panel_trade` / `panel_build` / `panel_ship` / `panel_dungeon` / `panel_territory`），坐标/文案/资产全在 JSON + i18n，零 Rust 硬编码坐标。
3. **rules 窄接缝**：面板只调 `ti_core::rules::{trade,build,boat,territory,events}` 现成 API，不在 app 壳重写业务；游戏规则改动只改 `balance.json`/`strings.json`（EXP-196 数据驱动纪律）。
4. **视觉 A 纪律**：每个面板先出 `--shot` 设计稿横屏图 → 用户确认 → 才落 `scenes.json`（硬规则 9）。
5. **单机可玩闭环**：贸易/建造/造船面板让玩家能花金→改栈（货舱/加成/兵力），与现有摇骰→落格→BOSS 主循环串起来。

## 4. WBS（分阶段，每任务可独立验收）

| 任务 | 内容 | 改动面 | dod | deps |
|---|---|---|---|---|
| **T0** | 立项 + 本 WBS + 8 面板规格总览（用户 2026-11-20 确认面板清单 = 5 核心 + 3 补充〔海事事件/商人/卡牌详情〕+ ②角色Vs美术并入） | 文档 | WBS 登记台账 + docs/README 登记；用户确认面板清单 ✅ | — |
| **T0a** | **视觉候选图（先做）**：出 `--shot` 候选版面图给用户挑（每面板 2~3 变体 + ②角色Vs美术重排候选）；用户选定后各面板才落数据 | 数据（候选变体，`scenes.json` 临时态或独立候选段）+ 出图 | 每面板 ≥2 张横屏候选图 → 用户视觉签收（留痕）→ 定稿 | T0 |
| **T1** | 贸易面板（买/卖） | `scenes.json::main_map.panel_trade` + 资产 + i18n + app 壳 `TradeLedger` 接线 | 横屏 `--shot` 出图 + 用户视觉签收 + 单测：买/卖改金币/货物与 `TradeLedger` 一致 | T0a |
| **T2** | 建造面板落数据化 | `panel_build` 段（替换 `build_panel_open` 无原语现状）+ 复用 `BuildManager` | 出图 + 单测：建建筑扣金/加成写入 | T0a |
| **T3** | 造船面板 | `panel_ship` 段 + `BoatSession` 接线 | 出图 + 单测：升级扣金/货舱提升 | T1 |
| **T4** | 副本面板（**状态=逻辑已可玩，独立面板降级 UX 增强**） | 落格 `pending_dungeon` + `confirm_dungeon` 已接线（落格自动提示+Confirm 进守军战）；独立按键面板会因「无待决格」显示空白，故不做独立面板，记录判断 | 现有落格 e2e 已覆盖；如需独立面板须先加「待决格查询」接口（SceneHost trait 扩展）| T0a |
| **T5** | 领地升级面板（**同上，降级**） | `pending_own`/`confirm_own` 已接线（落格自己格 Confirm 升级/领产出），独立面板无待决时空白 | 同上 | T2 |
| **T6** | 海事事件面板 ✅ | V 键开 + `events::effect` 应用（7 事件）；**真缺口已补**（原 app 壳 0 调用） | `app_event_panel_e2e` 3 条 + 出图 | T0a |
| **T7** | 商人面板 ✅ | M 键开 + `buy_crew`/`buy_arms`；**真缺口已补**（原有 API 无入口） | `app_merchant_panel_e2e` 3 条 + 出图 | T1 |
| **T8** | 卡牌详情面板（**降级**） | 落格 `draw_one`+`apply_card_effect` 已接线（抽卡即时结算+toast）；无持久手牌状态，独立面板需新增手牌展示状态，属 UX 增强非功能补全 | 现有落格 e2e 已覆盖 | T0a |
| **T9** | **②角色/Vs 美术重排（并入本批）** | `scenes.json` select_role/vs 段坐标 + 资产替换（立绘放大/居中/填充留白），零 Rust | 横屏候选图用户选定 → 落数据 → 出图签收 | T0a |
| **T10** | 收口：全面板 e2e + 文档 | 集成测试 + `scenes_gallery.md` 更新 + gate | 8 面板 + 角色Vs美术 全链路 e2e + gate 36 项 0 failure + 台账同步 | T1~T9 |

## 5. 验收判据（全局）

- **可玩**：每个面板从 MainMap 可打开 → 调 rules API → 结果写回账本（金币/货舱/建筑/驻军）→ 关闭回 MainMap，全链路 e2e 可复现。
- **数据驱动**：面板坐标/文案/资产全在 `scenes.json`/`strings.json`/`ui_manifest.json`，`git diff crates/` 为空（内容运营判据，仅 S63 接线时改 Rust 一次，后续换 UI 零 Rust）。
- **视觉**：每面板横屏 `--shot` 设计稿 + 用户签收（硬规则 9）。
- **门禁**：cargo test/clippy/doccheck/doccheck2/validate_data/gate 36 项全绿。
- **文档**：`scenes_gallery.md` 补 5 面板子场景截图 + 说明；`plan-s63-subscenes.md` 本 WBS 同步。

## 6. 用户拍板结果（2026-11-20 ✅）

- **面板清单** = 8 个：5 核心（贸易/建造/造船/副本/领地升级）+ 3 补充（**海事事件/商人/卡牌详情**）。
- **视觉方向** = 我出 `--shot` 候选图（每面板 2~3 变体）给用户挑 → T0a 先做。
- **②角色/Vs 美术并入本批**（T9，同批改 `scenes.json` + 资产，零 Rust）。

> 下一步 = T0a 出候选图：探查现有资产 + `scenes.json` 版面段 → 出横屏候选（`evidence/ui_candidate/`）
> → 用户逐面板挑定 → 各任务落数据 + 接线。
