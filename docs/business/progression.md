# 地图 / 据点 / 卡牌事件（Progression）

## 设计意图

地图是**探索的舞台**：等距网格 + 摄像机，玩家在格间移动，踩到据点/事件格触发
交互。地图推进玩家与 BOSS 相遇。地图与地形数据来自真实资源（`glmapm0*.Map`，
格式见 `docs/formats/`），但**玩法规则（移动/触发/卡牌）是本目录定义的设计**。

## 环岛赛道（S19，主玩法 = 大富翁式摇骰前进）

> 玩家是海盗船长，驾船**绕岛航行**：一次操作 = **摇骰定步数 → 沿环岛赛道前进 N 格
> → 落格结算**。赛道 = 岛屿外岸线环（Moore 轮廓追踪），保留原版地图形状与坐标。

### 赛道数据（单一事实源 `assets/data/board.json`）

| 项 | 设计 |
|---|---|
| 赛道 | 每图一条闭合环（m01=45 格 / m02=46 格 / m03=33 格），顺序 = 环绕方向（顺时针） |
| 生成 | `scripts/extract_board.py`（`terrain.json` 陆格掩码 → Moore 邻域轮廓 → 去重 → 起点旋转到 `player_start` 最近环格）；`--check` 校验一致性 |
| 空间类型 | `start` / `plain` / `port` / `city` / `event` / `card` / `battle` / `treasure` / `boss`（每图恰 1 起点 + 1 BOSS；港口/城镇 = balance 据点坐标就近吸附；原版特殊地块 event/card/war/MAZE/pool/Castle 按坐标就近吸附；**S19.4**：普通格按 **12 格周期**铺 `event/card/battle` 各 2 + `treasure` 1 + `plain` 5，并**按比例分配**（相位 = k×12/普通格数）⇒ 跨图落格密度一致） |
| 分区 | 9 个连续段（`group` 1..9，对齐原版地形类别码 1..9，`legacy/docs/facts/map.md` F-308） |
| 读档兼容 | 存档仍存行主序格索引；读档位置不在赛道上 = 就近吸附最近赛道格 |

### 摇骰与前进（`rules::board`）

```
Space（空格）/ 点按「摇骰前进」
  → 摇骰：步数 = 1..=dice_sides（`balance.board`，默认 6；走 `GameRng` 确定性）
  → 沿赛道前进 N 格（单向，越过末格回到首格 = 绕岛一圈，`laps` +1）
  → 落格结算（数值全走 `balance.board`）：
      绕圈过起点 → 每圈 +`lap_bonus_gold`（默认 150）
      港口 port   → 提示「可贸易/建造」（经济经 HUD 命令盘）
      城镇 city   → +`city_gold`（默认 50）
      事件/卡牌格 → 抽一张卡（`balance.cards` 卡池，`apply_card_effect` 结算；`event` 与 `card` 两类落格行为一致）
      遭遇 battle → 扣 `battle_damage` HP + 失 `battle_gold_loss` 金（默认 30/20）
      宝藏 treasure → +`treasure_gold`（默认 200）
      BOSS boss   → 见下「BOSS 触发规则」（进战斗场景；`combat.md`；击败 BOSS = 跨图推进，下节）
```

> 结算只认**终点格**（大富翁语义：路过不结算，终点才结算）⇒ 每摇骰 1 次 = 1 次落格判定。
> `balance.cards.event_chance` 只作用于 legacy 移动路径（`rules::main_map`），**不参与赛道落格结算**。

### BOSS 触发规则（S19.3）

| 项 | 规则 | 理由 |
|---|---|---|
| 触发时机 | **经过或落在 BOSS 格即遇 BOSS**（不再要求「恰好停在 BOSS 格」） | 33~46 格赛道上「恰好落格」≈ 1/40 量级运气，节奏不可预期（实测 E[摇骰] 32~41 次） |
| 现身门槛 | 绕岛满 `boss_min_laps` 圈（默认 1）才现身；未满圈 = 「BOSS 尚未现身（还需绕岛 N 圈）」提示，不进战斗 | 跨图节奏对齐 + 保证攒金建造时间（攒金→建造→打 BOSS 闭环） |
| 拦停 | 现身时位置停到 BOSS 格（被 BOSS 拦停，视觉与位置一致） | 玩家一眼看出「在这打」 |
| 跨图推进 | 击败 BOSS → 解锁下一图（`MapSlot` / `unlocked_maps`），m03 BOSS = 通关 | 已有实现（`battle_step` → `on_boss_defeated`） |

**跨图节奏实测**（蒙特卡洛 2 万次，`dice_sides=6`、`boss_min_laps=1`）：

| 地图 | 赛道 | BOSS 序号 | 旧规则 E[摇骰次数] | 新规则 E[摇骰次数] | 新规则 E[见 BOSS 时金]（S19.4 落格节奏后终值） |
|---|---|---|---|---|---|
| m01 | 45 格 | 27 | 40.6（靠运气落格） | **21** | 1339 |
| m02 | 46 格 | 15 | 37.3 | **18** | 1328 |
| m03 | 33 格 | 30 | 32.1 | **16** | 1371 |

→ 新规则下三图节奏收敛到 16~21 次摇骰（≈ 1 圈半），且到 BOSS 时已攒到 ~1300 金（够 `Investment 500` + `Bank 200` + 存款，`build` 加成 atk/hp 直通战斗）。

- 落格结果以**提示条**（toast）显示在摇骰按钮下方（文案 `strings.json` `scene.main_map.toast.*`）。
- 确定性：同 seed + 同摇骰序列 ⇒ 同落格串（可回放，`e2e_board_spin_loop` 判据）。

### 落格节奏（S19.4）

**落格类型按比例分配**（`scripts/extract_board.py`）：第 k 个普通格取 12 格周期的相位
`k×12/普通格数` ⇒ 各图「落格密度」与赛道格数等比（固定周期直接落在不同长度赛道上会得到
不同的宝藏/遭遇数量 ⇒ 跨图节奏失衡）。

**12 格周期**（`plain` 5 / `event` 2 / `card` 2 / `battle` 2 / `treasure` 1）—— 业务理由：

| 项 | 14 格周期 | 12 格周期 | 业务判断 |
|---|---|---|---|
| 死格（`plain`）占比 | 44%~45% | **39%~40%** | 终点格才结算（大富翁语义）⇒ 死格 = 白走一格；每次摇骰「有内容」的比例从 ~55% 提到 ~60% |
| 宝藏密度（+200 大奖） | 1/14（3%~4.4%） | **1/12（6.1%~6.7%）** | 宝藏是唯一大额奖励通道（占每图净收入 ~55%）⇒ 落格期望 0.6 → **0.9 次/图**，攒金节奏不再断档（否则整图不中大奖 ⇒ 见 BOSS 时仅 1150 金） |
| 跨图宝藏密度 | m03 仅 3% | **m03 6.1%** | 与 m01/m02 对齐（比例化分配后跨图节奏一致） |
| 遭遇密度 | 2/14 | 2/12 | 实测每图 1.9~2.6 次、暂存扣血 58~77（玩家 HP 2000 ⇒ 3~4%）⇒ 仍是可控风险 |

**跨图实测**（蒙特卡洛 2 万次/图，`dice_sides=6`、`boss_min_laps=1`、终点格结算）：

| 地图 | 赛道 | 落格分布（event/card/battle/treasure/plain） | 密度 | E[摇骰] | E[抽卡] | E[遭遇] | E[宝藏] | E[见 BOSS 时金] |
|---|---|---|---|---|---|---|---|---|
| m01 | 45 格 | 7 / 7 / 6 / 3 / 18 | 15.6% / 15.6% / 13.3% / 6.7% / 40% | 21.1 | 6.5 | 2.6 | 0.9 | 1339 |
| m02 | 46 格 | 6 / 6 / 7 / 3 / 20 | 13.0% / 13.0% / 15.2% / 6.5% / 43.5% | 17.9 | 5.2 | 2.0 | 0.9 | 1328 |
| m03 | 33 格 | 5 / 5 / 4 / 2 / 13 | 15.2% / 15.2% / 12.1% / 6.1% / 39.4% | 15.9 | 4.9 | 1.9 | 0.9 | 1371 |

- 「见 BOSS 时金」三图收敛到 **1328~1371（极差 3.2%）** ⇒ 攒金节奏跨图一致。
- **不变量（测试固化）**：各图 event/card/battle 占比 8%~20%、treasure ≥ 3%、plain 30%~60%
  （`rules::board` `board_landing_rhythm_is_consistent_across_maps`）；每图 ≥1 宝藏格
  （`board_data_matches_extraction`，防宝藏格再变死数值）。
- `event` 与 `card` 两类落格**行为一致**（都必抽一张卡）；两者在赛道上的分布只影响节奏计数，
  `balance.cards.event_chance` 不参与赛道结算。

### 经济节奏（S19.2 数值打磨）

奖励梯度（`balance.board`，单圈期望为正，支撑「攒金 → 贸易/建造 → 打 BOSS」节奏）：

| 项 | 数值 | 定位 |
|---|---|---|
| `treasure_gold` | 200 | 大奖（`treasure` 格 S19.2 起实际可达，密度 1/12，见 §落格节奏） |
| `city_gold` | 50 | 小奖励（据点，~1 格/图） |
| `lap_bonus_gold` | 150 | 每圈固定补给 |
| `battle_damage` / `battle_gold_loss` | 30 / 20 | 遭遇格真实代价（扣 HP 暂存 + 失金，`card_state.hp` 入战斗前并入） |

- **不变量（测试固化）**：`treasure_gold > city_gold > 0`、`battle_damage > 0`、`battle_gold_loss > 0`、`lap_bonus_gold > 0`、`max_steps ≥ dice_sides ≥ 1`（`ti-core` `board_cfg_economy_is_sane`）；每图赛道必含 ≥1 `treasure` 格（`rules::board` `board_data_matches_extraction`，防宝藏格再变死数值）；`boss_min_laps ≥ 1`（S19.3 跨图节奏门槛）。
- 节奏目标：单圈净金 > 0（宝藏 +200 主导），遭遇为可控风险（实测每图 1.9~2.6 次遭遇 = 失金 38~52 + 暂存扣血 58~77，玩家 HP 2000 ⇒ 3~4%），保证 BOSS 战前可攒金建造（`build` 加成 atk/hp）。
- **与 BOSS 门槛的关系**：`boss_min_laps=1` ⇒ 玩家必先绕岛 1 圈才见 BOSS ⇒ 圈补给（+150）+ 沿途宝藏（~2~3 个 ×200）+ 城镇（50）构成「BOSS 战前基金」（实测 ~1300 金），正好支撑 `Investment 500` / `Bank 200` / `Fort 800` 的建造选择。

### 旧移动规则（已彻底移除）

原「方向键逐格自由移动」（`GameInput::Walk` / `MapHost::map_session_move` /
`GameCommand::Walk` / headless `walk` 动作）已于 S19.1 彻底删除——主地图移动**只保留摇骰
（`GameInput::Spin`）**，方向键不再产生任何移动；`rules::main_map::MapSession`（地形
`move_to` / `dispatch`）作为规则数据仍保留供 Landmark 据点结算与旧存档坐标校验，
但**不再有输入通道驱动它**。主地图界面按赛道渲染（轨道高亮 + 空间标记 + 摇骰按钮）。

## 地图结构

| 项 | 设计 |
|---|---|
| 网格 | 每地图 `grid_w × grid_h`（来自资源，实测 20×20 / 20×20 / 15×15） |
| 投影 | 等距：`iso_x = 32*(row+col)`，`iso_y = 16*(row + grid_w - 1 - col)`（实测值，出处 `legacy/docs/facts/map.md` F-307；此处内联为业务公式，不依赖原版产物） |
| 摄像机 | 跟随玩家，按投影中心计算，整格对齐（不做平滑插值，保证确定性；实现落点 `ti_platform::gfx::FramePresenter`/`Viewport`，见 impl/03_map_system.md §5） |
| 地形类型 | 由资源 terrain 记录给出（水/陆/障碍），移动合法性查表 |

## 移动规则

> S19.1 起主地图移动**只有摇骰**（§环岛赛道）：原「方向键逐格自由移动」
> （`GameInput::Walk`）已彻底删除，方向键不再移动；`rules::main_map` 的
> 地形 `move_to` 仅作为据点结算的数据底座保留（§据点与事件），无输入通道。

```
摇骰（Space / deck_spin 触摸）→ 骰点 1..=6 → 沿环岛赛道前进 N 格 → 落格结算（见 §环岛赛道）
```

- **一格一步**，每步触发一次结算，保持回合制节奏；
- 不可通行判定由赛道数据决定（`assets/data/board.json`，不进代码）。

## 据点与事件（落格结算）

```
移动到新格
  → 若该格是据点 → 打开据点菜单（贸易/建造/信息）
  → 否则        → 掷骰判定是否触发卡牌事件（概率见 balance.cards.event_chance）
  → 遇到 BOSS 格 → 进入战斗场景（combat.md）
```

## 卡牌事件系统

卡牌是**轻量 roguelite 式事件**：抽一张牌，立刻生效（多为一次性增益/损失）。

> 以下为**示例节选**（3 张），完整数值（5 张卡，deck 1..5）以 `assets/data/balance.json` 为准。

```jsonc
"cards": {
  "deck": [                       // 卡池（示例节选；id/effect schema 与 balance.json 一致）
    { "id": 1, "name_key": "cards.1", "weight": 3, "effect": { "kind": "gold",   "value": 20 } },
    { "id": 2, "name_key": "cards.2", "weight": 2, "effect": { "kind": "heal",   "value": 15 } },
    { "id": 3, "name_key": "cards.3", "weight": 3, "effect": { "kind": "damage", "value": 10 } }
  ],
  "event_chance": 0.35,           // 移动触发卡牌的概率
  "hand_size": 1                  // 一次抽几张（balance.json 当前值 = 1）
                                     // FIND-029 设计裁决：单卡 roguelite 事件（抽一张立即生效），非占位；
                                     // 调 >1 需同步 main_map.rs draw_one 的 n 参数（每步抽 n 张逐一结算）
}
```

> 上表为示例节选（3 张），完整 5 张卡（id 1..5）以 `assets/data/balance.json` `cards.deck` 为准。
> `effect` schema 为 `{ kind, value }`（非 `{ "gold": +200 }` 旧设计占位），见 `docs/impl/06_cards_events.md`。

### 抽牌（可复现）

用 `GameRng` 按权重抽卡（可播种 PCG，`rand_pcg`，S8.7 落地，走确定性序列）：
同 seed + 同移动序列 ⇒ 同一串卡牌结果。**禁用 `thread_rng`**。

### 效果结算

- `gold`/`hp` 增减后立即做区间钳制（`0 ≤ gold ≤ gold_max`、`0 ≤ hp ≤ hp_max`）；
- 效果码采用「加法为主、覆盖为辅」，便于扩展而不改结算器。

## 可测试规则

| 规则 | 单测 |
|---|---|
| 越界/不可通行不移动、不消耗回合 | `map_blocked_step_is_noop`（待 S8.3.c 补单测落地） |
| 移动一格必触发一次落格结算 | `map_step_triggers_settle_once`（待 S8.3.c 补单测落地） |
| BOSS 格进入战斗 | `map_boss_tile_enters_combat`（待 S8.3.c 补单测落地） |
| 同 seed 同路径 ⇒ 同卡牌串 | `cards_same_seed_same_sequence`（待 S8.3.c 补单测落地） |
| 效果后 HP/金钳制在合法域 | `cards_effects_clamped`（待 S8.3.c 补单测落地） |
| 权重抽卡分布合理（单测只断言「可复现」，分布靠统计脚本） | `cards_weighted_pick_reproducible`（待 S8.3.c 补单测落地） |

## 跨图推进（地图解锁 `unlocked_maps`）

游戏共 3 图 m01/m02/m03，难度按 `boss_tier` 升序（m01=0 → m02=1 → m03=2，BOSS 更强）。
推进规则 = **击败当前图 BOSS 才解锁下一图**，非开局三图齐选。

| 阶段 | 规则 |
|---|---|
| 初始解锁 | 仅 `m01`（`unlocked_maps = [m01]`） |
| 推进 | 击败当前图 BOSS（`combat` 结果 = Player 胜）→ 解锁序列下一图，追加进 `unlocked_maps` |
| 序列 | m01 →(击败)→ 解锁 m02 →(击败)→ 解锁 m03；m03 无下一图 |
| 通关 | 击败 m03 的 BOSS = 全图通杀 = 游戏通关（本局结束） |
| 失败 | HP 归零 / 超 `combat.max_rounds` = 失败，**不解锁**；回主菜单后可**重试**当前图 |
| 选择约束 | SelectMap 显示**全部 3 图**，锁定项置灰 + 深色「锁」块（`08_layout_design.md` §锁块）；Prev/Next 可自由导航（含锁定位），仅 **Confirm** 经 `map_unlocked` 校验拒绝（`Sfx::Invalid`，选择态不提交） |

### 与存档一致（`persistence.md`）

- 规则层存档（`saveload`/`saveload_json` 的 `SaveData.unlocked_maps` 字段）**当前无生产写入端**：4 处构造恒 `Vec::new()`，仅测试 fixture 手工置值——解锁链的**唯一事实源 = app 侧 `GameApp.unlocked_maps` + `GameApp.cleared`**（`ProgressionState` 驱动）。该字段为预留扩展位（接规则层存档需 `capture_state` 增 `unlocked` 入参，c1 契约变更，见台账 S15.2 残留）。
- **app 侧存档槽（`domain::persistence::SaveSlot`）持 `unlocked_mask: u8` 逐图推进字段**（S15.2/F23 定稿，v2 向后兼容：旧档无该字节 → `#[serde(default)]` = 0）：bit 0/1/2 = m01/m02/m03 已解锁（口径 = 规则层 `ProgressionState::unlocked_mask()`），**bit 3 = CLEARED 通关位**（m03 BOSS 已击败，`ProgressionState::CLEARED_BIT`）。写盘 = `scenes/saveload.rs` 经 `SessionHost::unlocked_mask()` 取 GameApp 当前推进位（解锁集 + 通关位）；读档 `restore_save` 经 `ProgressionState::for_save_restore(mask, map_slot)` 还原：`mask != 0` 逐位忠实还原（保住「击败 m01 解锁 m02、尚未切图」的进度 + 通关位），`mask == 0`（旧档）退「最小前置链」`min_progression_to_select`，档与选图矛盾（选图未解锁）也退链——旧档恒可回读，无需迁移。
- **通关门禁（`is_cleared` 逐图）**：m03 BOSS 击败位 = 通关位，逐图判定 = `GameApp::is_map_cleared(m)`（m01/m02 = 下一图已解锁口径，m03 = `GameApp.cleared`）；总门禁 `GameApp::is_cleared()` = m03 击败位。置位时机 = `battle_step` 在 m03 胜时（解锁集此时已全开，通关另记位）。

### 可测试规则

| 规则 | 单测 |
|---|---|
| 初始仅 m01 解锁 | `progression_initial_unlocks_only_m01` |
| 击败 m01 → 解锁 m02（m03 仍锁） | `progression_defeat_m01_unlocks_m02_only` |
| 击败 m02 → 解锁 m03 | `progression_defeat_m02_unlocks_m03` |
| 击败 m03 = 通关、无下一图 | `progression_defeat_m03_clears_game` |
| 失败（未击败）不解锁 | `progression_loss_does_not_unlock` |
| 解锁链单调（只增不降，m01 恒解锁） | `progression_unlocks_monotonic` |
| 读档链恢复：选中 m(i) 的最小前置 = 击败 m(0)..m(i-1) | `min_progression_to_select_chain` |
| 解锁集重建（战斗结算 / 旧档异常集忠实保留 / 全解锁幂等） | `from_unlocked_feeds_boss_defeat_step`、`from_unlocked_legacy_anomaly_is_faithful`、`from_unlocked_all_then_m03_clears` |
| 掩码往返（`unlocked_mask()` ↔ `from_mask()`，含 CLEARED 位） | `unlocked_mask_round_trip` |
| 读档掩码忠实还原 / 旧档 `mask=0` 退链 / 矛盾掩码修当前图 | `for_save_restore_legacy_mask_zero_falls_back_to_chain`、`for_save_restore_nonzero_mask_is_faithful`、`for_save_restore_contradictory_mask_repairs_current_map` |
| 通关门禁（m03 击败位 = `is_cleared` 逐图门禁） | `is_map_defeated_is_per_map` + app 层 `app_save_load_restores_cleared_gate` |

> 实现落点：`crates/ti-core/src/rules/progression.rs`（`ProgressionState`：`initial_unlocked` /
> `unlocked` / `on_boss_defeated` / `is_cleared` / `can_select` / `from_unlocked` / `min_progression_to_select`
> / `is_map_defeated` / `unlocked_mask` / `from_mask` / `for_save_restore` + `CLEARED_BIT`（bit 3 通关位））。
> **app 层已接线**（`crates/ti-app/src/game_app.rs`）：开档 `unlocked_maps = initial_unlocked()`（仅 m01，非全开）；
> `battle_step` 胜利钩 = 击败当前图 BOSS → 规则层结算解锁下一图（幂等，失败不加），m03 胜另置 `GameApp.cleared` 通关位；
> 逐图门禁只读接口 = `GameApp::is_cleared()`（m03 击败位）/ `GameApp::is_map_cleared(m)`；
> `restore_save` 经 `unlocked_mask` 忠实还原（`mask=0` 旧档退最小前置链，矛盾掩码退链修当前图）；
> 写盘 = `scenes/saveload.rs` 经 `SessionHost::unlocked_mask()` 取推进位（解锁集 + CLEARED 位）。
> 验收：`crates/ti-app/tests/app_progression_wiring.rs`（6 条：初始仅 m01 / 胜解锁+幂等 / m01→m02 全链路 / 读档链恢复+gold/位置恢复 /
> **读档掩码忠实还原（非零掩码）** / **通关门禁 CLEARED 位读档还原**）。
> 演示：`crates/ti-core/examples/cross_map_demo.rs`（打满 3 图）。
> **残留（S15.2 登记）**：① 通关判定 `is_cleared` 在 app 游玩层暂无**画面表现**（击败 m03 后 Back 回主菜单，无通关画面/提示；门禁位与存档口径已闭环，画面属 S16 范围）；
> ② 规则层存档 `SaveData.unlocked_maps` 无生产写入端（见上「与存档一致」）。

## 与实现的接口

- 玩法：`rules::main_map`（`MapSession`）、`rules::cards`（`CardEngine`）（S8.3.c 重写中）；
- 地形/格数据：真实资源解析（`ti-formats`，保留）；
- 摄像机：表现层（`ti_platform::gfx` 的 `FramePresenter`/`Viewport` 承担缩放/呈现；`ti-app` 主地图渲染按上述投影公式绘制），玩法层只管「玩家在哪一格」。