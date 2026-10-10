# 15 — 完整玩法规格（全系统实现手册）

> 本文档汇总所有玩法系统的**完整实现级规格**，读者仅凭本文档 + `assets/data/*.json`
> 即可从头实现全部游戏逻辑。各系统分散在 02~13 号文档中，本文件是**统一入口与交叉参考**，
> 同时补齐现有文档中尚未写透的边界情况、状态组合、数值衔接与完整数据流。
>
> 数值以 `assets/data/balance.json` 为单一事实源，本文件只写「公式结构与调用点」，
> 不重复写数值；如需调数值，改 JSON 不改本文件。

---

## 1. 全局数据流总览

```
启动
 ├─ 加载 ui.pak + glpics.pak + settings.dat + gl.rec
 ├─ Intro 片头（自动跳 Index）
 └─ Index 主菜单
      ├─ 新游戏
      │    ├─ SelectRole（5 角色循环，完成=角色非空）
      │    ├─ SelectMap（已解锁地图循环，完成=地图非空）
      │    └─ MainMap（探索→据点→贸易→建造→卡牌→战斗）
      │         ├─ 移动：方向键 → oracle_move → 落格结算
      │         │    ├─ 据点命中 → Msgl（据点名+资源结算）
      │         │    ├─ 卡牌事件（event_chance）→ CardEngine.apply
      │         │    └─ BOSS 格 → 进入 Vs
      │         ├─ 据点交互 → 贸易面板（买/卖）/ 建造面板
      │         └─ 战斗结束 → Victory（奖励金）/ Defeat（HP=0 回主菜单）
      ├─ 继续游戏 → Saveload 子窗口（读档→恢复 MainMap）
      ├─ 选项 → 设置菜单（音量/缩放/画质/语言）
      ├─ 说明 → Help（3 页）
      └─ 退出 → Exit（终态）
```

---

## 2. 场景状态机（完整转换表）

### 2.1 场景枚举（9 个）

| 场景 | 资源文件 | 叠加层 | 说明 |
|---|---|---|---|
| Intro | `intro.Map` | 否 | 片头动画，自动跳 Index |
| Index | `index.Map` | 否 | 主菜单（6 项） |
| SelectRole | `selectrole.Map` | 否 | 角色选择（5 循环） |
| SelectMap | `selectmap.Map` | 否 | 地图选择（已解锁循环） |
| MainMap | `main.Map` + `glmapm0x.Map` | 否 | 主地图探索 |
| Vs | `vs.Map` | 否 | 战斗 |
| Message | `msgl/msgm/msgs/help/information.Map` | 是（叠加） | 消息/帮助/信息 |
| Saveload | `saveload.Map` | 是（叠加） | 存读档（子窗口） |
| Exit | — | — | 终态 |

### 2.2 场景状态码

| 状态码 | 语义 | Rust 枚举 |
|---|---|---|
| 0 | 主菜单态 | `Menu` |
| 2 | 场景卸载中 | `Unloading` |
| 3 | 回主菜单 | `BackToMenu` |
| 4 | 过场/坐标切换 | `Transition` |
| 25 | 战斗态 | `Battle` |
| 26 | 过场动画 phase 26 | `Anim26` |
| 27 | 过场动画 phase 27 | `Anim27` |

### 2.3 状态转换表（`oracle_next_state(当前, 触发码) → (新状态, 目标场景)`）

| 当前 | 触发码 | 含义 | 新状态 | 目标场景 |
|---|---|---|---|---|
| 0 | 1 | 进战斗 | 25 | Vs |
| 0 | 其他 | — | 0 | — |
| 2 | 5 | 动画完成 | 0 | Index |
| 2 | 其他 | — | 2 | — |
| 3 | 任意 | 回主菜单 | 0 | Index |
| 4 | 4 | 触发过场 | 27 | Intro |
| 4 | 其他 | — | 4 | — |
| 25 | 2 | 战斗结束 | 0 | Index |
| 25 | 其他 | — | 25 | — |
| 26 | 5 | 动画完成 | 27 | Intro |
| 26 | 其他 | — | 26 | — |
| 27 | 5 | 动画完成 | 0 | Index |
| 27 | 其他 | — | 27 | — |

**触发码含义**：0=无触发，1=进战斗，2=战斗结束，3=回菜单，4=触发过场，5=过场完成。

### 2.4 场景跳转规则（用户操作驱动）

| 当前场景 | 触发条件 | 目标场景 |
|---|---|---|
| Index | 菜单项 0（新游戏） | SelectRole |
| Index | 菜单项 1（继续游戏） | Saveload（叠加） |
| Index | 菜单项 2（选项） | Message（Option kind） |
| Index | 菜单项 3/4（说明） | Message（Help kind） |
| Index | 菜单项 5（退出） | Exit |
| SelectRole | 角色选定 | SelectMap |
| SelectRole | 返回（27） | Index |
| SelectMap | 地图选定 | MainMap |
| SelectMap | 返回（27） | SelectRole |
| MainMap | 事件码 6（BOSS 格） | Vs |
| MainMap | 事件码 1/4 | Message（Msgl） |
| MainMap | 事件码 2/5/7 | Message（Msgm） |
| MainMap | 事件码 8 | Message（Msgs/Msgm） |
| MainMap | 返回（27） | Index |
| Vs | 战斗结束 | Index |
| Message | 关闭 | 来源场景 |
| Saveload | 读档成功 | MainMap |
| Saveload | 关闭 | Index |

---

## 3. 全局事件码体系

### 3.1 触发事件码（状态机内部）

| 事件码 | 常量名 | 语义 |
|---|---|---|
| 0 | `EVENT_NONE` | 无触发 |
| 1 | `EVENT_BATTLE` | 进战斗 |
| 2 | `EVENT_BATTLE_END` | 战斗结束 |
| 3 | `EVENT_MENU` | 回菜单 |
| 4 | `EVENT_CUTSCENE` | 触发过场 |
| 5 | `EVENT_CUTSCENE_DONE` | 过场完成 |

### 3.2 用户输入事件码

| 事件码 | 常量名 | 语义 | 音效 |
|---|---|---|---|
| 13 | `EVENT_CONFIRM` | 确定/Enter | Press |
| 27 | `EVENT_CANCEL` | 返回/取消 | 无 |
| 37 | `EVENT_INVALID` | 无效操作 | Invalid |
| 38 | `EVENT_PREV_PAGE` | 上一页/上一个 | 无 |
| 39 | `EVENT_PAGE_CONFIRM` | 翻页确认（贸易专用） | 无 |
| 40 | `EVENT_NEXT_PAGE` | 下一页/下一个 | 无 |

### 3.3 主地图事件分派码（与触发码独立）

| 分派码 | 目标场景 | 说明 |
|---|---|---|
| 1 | Msgl | 据点事件 |
| 2 | Msgm | 面板事件 |
| 3 | 无场景 | 卡牌随机数生成 |
| 4 | Msgl | 货物结果 |
| 5 | Msgm | 卡片翻转 |
|  | 6 | Vs | 战斗触发 |
| 7 | Msgm | 消息提示 |
| 8 | Msgs/Msgm | 退出/返回 |

> 触发码 1 = 进战斗；分派码 1 = Msgl 据点。两套体系，同名数值语义不同。

### 3.4 事件 id 值域与重映射

```
event_id ∈ 1..25（rand() % 25 + 1）
重映射：{4→8, 13→17, 14→18, 15→19}
```

全部 25 个 id 均有处理路径（`cards` 统一存储），覆盖率 100%。

---

## 4. 核心循环（单次玩家操作）

每次玩家操作 = 一次移动 / 一次据点交互 / 一次战斗回合 / 翻一张牌。
每次操作后按固定顺序结算：

```
1. 玩家动作生效（移动一格 / 交易结算 / 攻击一回合 / 翻一张牌）
2. 据点/地图响应（驻留据点触发事件、移动触发随机遭遇）
3. 状态检查（HP≤0 → 失败；到达 BOSS → 战斗；gold 溢出 → 银行）
```

**设计约束（可写单测）**：

| 约束 | 断言 |
|---|---|
| 单次操作不产生多步连锁 | 一帧一次玩家操作 ⇒ 至多一次结算流水线 |
| 金币非负、≤ 上限 | `0 ≤ gold ≤ balance.economy.gold_max` 恒成立 |
| HP ∈ [0, hp_max] | 扣血后不越界；归零即 `Defeat` |
| 场景无死锁 | 任意场景可经 ≤ `balance.ui.max_back_steps` 次操作回到主菜单 |

---

## 5. 角色与地图选择

### 5.1 角色枚举（5 个，来自 `balance.json` `selection.characters`）

| 索引 | id | 头像 | 立绘 | 精灵 |
|---|---|---|---|---|
| 0 | panda | `face_panda.pcx` | `start_panda.pcx` | `Player_panda.pcx` |
| 1 | tichy | `face_tichy.pcx` | `start_tichy.pcx` | `Player_tichy.pcx` |
| 2 | kidd | `face_kidd.pcx` | `start_kidd.pcx` | `Player_kidd.pcx` |
| 3 | drake | `face_drake.pcx` | `start_drake.pcx` | `Player_drake.pcx` |
| 4 | katharina | `face_katharina.pcx` | `start_katharina.pcx` | `Player_katharina.pcx` |

### 5.2 选择流程

```
SelectRole：
  进入 → 显示 5 个角色头像 + Target_knife.pcx 选中标记
  事件码 38 → 上一个角色（索引 0 → 4 循环）
  事件码 40 → 下一个角色（4 → 0 循环）
  事件码 13 → 确认选择，记录角色到全局变量
  事件码 27 → 返回 Index

SelectMap：
  进入 → 显示已解锁地图按钮（文件存在判定）
  事件码 38 → 上一个地图（已解锁列表循环）
  事件码 40 → 下一个地图
  事件码 13 → 确认，设置全局地图编号（1/2/3）
  事件码 27 → 返回 SelectRole
```

### 5.3 地图枚举（3 张，来自 `balance.json` `selection.maps`）

| 槽位 | id | 文件 | 网格 | BOSS 档 |
|---|---|---|---|---|
| 0 | m01 | `glmapm01.Map` | 20×20 | tier 0 |
| 1 | m02 | `glmapm02.Map` | 20×20 | tier 1 |
| 2 | m03 | `glmapm03.Map` | 15×15 | tier 2 |

**解锁判定**：对应 `data/glmapm0x.Map` 文件存在（`CreateFileW` 探测）。

### 5.4 完成条件

```
completed = (character != None) && (map_slot != None)
completed → 触发 main.Map 装载 → 进入 MainMap
```

---

## 6. 主地图系统

### 6.1 地图数据

| 地图 | 网格 | cell 记录数 | 连线记录数 | 缺格 |
|---|---|---|---|---|
| M01 | 20×20 | 399 | 75 | cell(0,19) |
| M02 | 20×20 | 399 | 125（含 4 条 M=4） | cell(0,19) |
| M03 | 15×15 | 224 | 23 | cell(0,14) |

每个地图 cell 记录数 = `grid_w × grid_h - 1`（恒缺 1 格）。
缺格恒为 `cell(row=0, col=grid_w-1)`，不可通行。

### 6.2 等距投影

```
iso_x = 32 × (row + col)
iso_y = 16 × (row + grid_w - 1 - col)
```

- row 增大 = 屏幕下移 = 南
- col 增大 = 屏幕右移 = 东

### 6.3 方向枚举

| 方向码 | 枚举 | Δ(row,col) | 探测优先级 | 方向位 |
|---|---|---|---|---|
| 0 | North（北） | (-1, 0) | 4（兜底） | 8 |
| 1 | West（西） | (0, -1) | 3 | 4 |
| 2 | South（南） | (+1, 0) | 1（优先） | 2 |
| 3 | East（东） | (0, +1) | 2 | 1 |

### 6.4 可通行判定

```
可通行 = (cell.flags & 3) == 1
越界格子一律不可通行
```

### 6.5 移动算法

```rust
fn move_one_step(grid_w, grid_h, blocked, pos, dir) -> Option<GridPos> {
    let (dr, dc) = dir.delta();
    let new_row = pos.row as i32 + dr;
    let new_col = pos.col as i32 + dc;
    if new_row < 0 || new_col < 0 { return None; }
    if new_row >= grid_h as i32 || new_col >= grid_w as i32 { return None; }
    let idx = (new_row as u32) * grid_w + (new_col as u32);
    if blocked[idx as usize] { return None; }
    Some(GridPos { row: new_row as u32, col: new_col as u32 })
}

fn move_n_steps(grid_w, grid_h, blocked, pos, dir, steps) -> (GridPos, u32) {
    // 逐格走，撞墙即停，返回 (终点, 实际步数)
}
```

### 6.6 落格结算（移动后必触发）

```
移动到新格
  → 若该格是据点 → 打开据点菜单（Msgl：据点名 + 资源结算）
  → 否则 → 掷骰判定是否触发卡牌事件（概率 = balance.cards.event_chance）
  → 遇到 BOSS 格 → 进入战斗场景（Vs）
```

**据点判定**：

```rust
fn at_landmark(landmarks: &[Landmark], pos: GridPos) -> Option<&Landmark> {
    landmarks.iter().find(|lm| lm.row == pos.row && lm.col == pos.col)
}
```

**据点资源结算**（命中时）：

1. 遍历精灵匹配场景 id；
2. 命中 → 累加资源值（上限 999）；
3. 未命中 → 扣减（贸易/建造消耗）；
4. 资源值为 0 且计数 > 1 → 补 1（保底机制）。

**据点触发概率**：

```
命中 = (rand() % 100 + 1) > 40   // 60% 命中率
```

### 6.7 摄像机

```
跟随玩家，按投影中心计算，整格对齐（不做平滑插值，保证确定性）
目标：cam_x = unit_iso_x - viewport_w / 2
      cam_y = unit_iso_y - viewport_h / 2
边界钳制：clamp(0, map_w - viewport_w)
```

### 6.8 渲染管线（主地图）

```
1. 清屏（背景色）
2. 绘制地形瓦片（painter's 算法，按 iso_y 非降序）
3. 绘制连线（道路/路线）
4. 绘制据点标记
5. 绘制玩家精灵
6. 绘制 HUD（金量/货物状态行，y=150/162）
```

---

## 7. 贸易系统

### 7.1 商品目录（来自 `balance.json` `trade.goods`）

| id | buy 价格 | qty_max |
|---|---|---|
| 1 | 20 | 99 |
| 2 | 76 | 99 |
| 3 | 95 | 99 |
| 4 | 114 | 99 |
| 5 | 133 | 99 |

### 7.2 买卖公式

```
买入：
  cost = goods.buy × qty
  gold -= cost
  goods_owned[goods.id] += qty
  约束：0 ≤ qty ≤ goods.qty_max，0 ≤ gold ≤ gold_max

卖出：
  value = round(goods.buy × balance.trade.markup) × qty
  gold += value
  goods_owned[goods.id] -= qty
  约束：goods_owned[goods.id] ≥ qty
```

**设计选择**：价格固定，不做动态行情。贸易是稳定金币来源，非投机玩法。

### 7.3 贸易场景交互

| 事件码 | 行为 |
|---|---|
| 22 | 进入贸易场景（买入方向） |
| 23 | 进入贸易场景（卖出方向） |
| 13 | 买入/卖出确认 |
| 37 | 取消当前操作 |
| 38 | 商品索引 -1（循环） |
| 39 | 翻页确认（买→卖 或 卖→买） |
| 40 | 商品索引 +1（循环） |

**前置条件**：据点结算完成态（状态码 21）。

### 7.4 事务性

失败时（金量不足/无库存）状态不变，返回 `TradeError`，UI 显示 Invalid 音效。

---

## 8. 建造系统

### 8.1 建筑类型（来自 `balance.json` `build.buildings`）

| id | base_cost | gold_max | 说明 |
|---|---|---|---|
| Castle | 1000 | 20000 | 城堡，最高上限 |
| Investment | 500 | 12000 | 投资设施 |
| Bank | 200 | 5000 | 银行 |
| Fort | 800 | 16000 | 据点/堡垒 |

### 8.2 建造成本公式

```
cost = base_cost(kind) × (level + 1)
level ∈ [0, balance.build.max_level]（= 10）
```

等级 0 展开值 = base_cost × 1；等级 10 展开值 = base_cost × 11。

### 8.3 建造操作

```rust
fn build(bank, kind, level) -> Result<(), BuildError> {
    if level > balance.build.max_level { return Err(InvalidLevel); }
    let cost = base_cost(kind) * (level + 1);
    if bank.gold < cost { return Err(Insufficient); }
    bank.gold -= cost;
    // 抬升金量上限到该建筑 gold_max（取已建建筑的最大值）
    Ok(())
}
```

### 8.4 金量上限

```
gold_max = max(已建建筑的 gold_max, 各档 max)
溢出部分进银行累计（bank.deposit）
```

### 8.5 银行系统

**双槽位模型**：

| 槽位 | 语义 |
|---|---|
| `bank.gold` | 流动金（可用于建造/贸易/投资） |
| `bank.deposit` | 银行存款金（安全保管） |

**存款**：

```rust
fn deposit(bank, amount) -> Result<(), BuildError> {
    if amount > bank.gold { return Err(Insufficient); }
    bank.gold -= amount;
    bank.deposit += amount;
    Ok(())
}
```

**取款**：

```rust
fn withdraw(bank, amount) {
    bank.gold = (bank.gold + amount).min(MAX_GOLD); // 999999
    bank.deposit -= amount;
}
```

取款永不失败（金量只增不减，上限截断）。

**银行步进器**：按压 1/2/3/4/5 次 = 10/100/1000/10000/100000，6..10 保持当前。

### 8.6 投资奖励阶梯

兵种 A..E 奖励：+30000 / +40000 / +20000 / +15000 / +50000（战斗胜利后发放）。

---

## 9. 战斗系统

### 9.1 BOSS 档（来自 `balance.json` `combat.boss_tiers`）

| tier | hp | atk | 地图 |
|---|---|---|---|
| 0 | 50000 | 10 | m01 |
| 1 | 100000 | 12 | m02 |
| 2 | 150000 | 14 | m03 |
| 3 | 200000 | 16 | （高阶，需建造解锁） |
| 4 | 300000 | 20 | （最高） |

`tier > 4` → HP = 0 → 视为 BOSS 已死 → 玩家立即获胜，0 回合。

### 9.2 伤害公式（设计选择，非逆向）

```
damage = round(atk × atk_coef) + rand_offset()
rand_offset ∈ [0, balance.combat.damage.rng_span]（= 3）
下限 = balance.combat.damage.flat（= 1），保证每回合至少掉 1 血
```

- 确定性：随机来自 `GameRng`（可播种 LCG），同 seed 同操作序列 ⇒ 同结果。
- 玩家先手（`balance.combat.player_first = true`）。

### 9.3 回合流程

```
玩家确认攻击
  → 玩家按 damage 公式结算对 BOSS 的伤害
  → 若 boss_hp == 0 → Victory，战斗结束
  → 否则 BOSS 按 damage 公式结算对玩家的伤害
  → 若 player_hp == 0 → Defeat，战斗结束
  → 否则回合数 +1，回到等待输入
```

### 9.4 胜负判定

| 条件 | 结果 |
|---|---|
| `boss_hp == 0` | `Victory`（同回合判玩家胜） |
| `player_hp == 0` 且 `boss_hp > 0` | `Defeat` |
| 超过 `balance.combat.max_rounds`（= 1000） | 仍按 HP 高者判（同回合为 `Defeat`） |

### 9.5 战斗胜利奖励

```
播 victory.wav
兵种奖励阶梯（+15000~+50000，按兵种档位）
回主菜单（advance(2) → 状态 0）
```

### 9.6 战斗场景交互

| GameInput | 行为 |
|---|---|
| Confirm / Attack | 攻击一回合 |
| Cancel | 退出战斗 → 回 Index |
| 方向 | 无（战斗无方向操作） |

### 9.7 战斗渲染

- 每回合后渲染：玩家 HP、BOSS HP、当前回合数
- 战斗结束渲染胜方面板
- 场景资源：`ui.pak + vs.Map`
- 战斗态标志：状态码 25

---

## 10. 卡牌事件系统

### 10.1 卡牌目录（来自 `balance.json` `cards.deck`）

| id | weight | 效果 | 数值 |
|---|---|---|---|
| 1 | 3 | damage | -10 HP |
| 2 | 2 | heal | +15 HP |
| 3 | 3 | gold | +20 金 |
| 4 | 1 | move | 随机移动 |
| 5 | 1 | unknown | 无效果 |

### 10.2 触发机制

```
移动到新格（非据点）
  → 掷骰：rand() < event_chance（0.35）→ 触发卡牌事件
  → 生成 event_id = rand() % 25 + 1
  → 重映射：{4→8, 13→17, 14→18, 15→19}
  → CardEngine.apply(event_id, &mut state)
```

### 10.3 卡牌效果结算

```
gold 增减后立即钳制：0 ≤ gold ≤ gold_max
hp 增减后立即钳制：0 ≤ hp ≤ hp_max
效果码采用「加法为主、覆盖为辅」，便于扩展
```

### 10.4 卡牌库存门控

```
卡牌库存 >= 10 时：
  装载 ui.pak + card.Map + bigcard.Map
  状态码 = 4（卡牌态）
```

### 10.5 抽牌确定性

同 seed + 同移动序列 ⇒ 同一串卡牌结果。**禁用 `thread_rng`**，全走 `GameRng`。

### 10.6 卡牌场景交互

| GameInput | 行为 |
|---|---|
| Up / Left | 上一张卡 |
| Down / Right | 下一张卡 |
| Confirm | 选中/确认卡牌 |
| Cancel | 关闭 → 回 MainMap |

---

## 11. 消息界面系统

### 11.1 五类界面

| 种类 | 资源文件 | 总页数 | 语义 |
|---|---|---|---|
| Msgs | `msgs.map` | 1 | 通用消息/退出对话框 |
| Msgl | `msgl.map` | 1 | 据点名/地标 + 资源结算 |
| Msgm | `msgm.Map` | 1 | 多页面板/卡片/贸易结果 |
| Help | `help.map` | 3 | 帮助文本 |
| Info | `information.map` | 1 | 信息/角色状态 |

### 11.2 翻页状态机

```
状态：cur: Option<MsgPage>
MsgPage { kind, page, total_pages }

事件：
  Open(kind)  → Some({kind, page: 0, total: total_pages(kind)})
  NextPage    → page+1 < total ? page+1 : page（末页保持）
  PrevPage    → page == 0 ? total-1 : page-1（首页回末页，循环）
  Close       → None
```

**边界规则**：

| 规则 | 说明 |
|---|---|
| NextPage 到末页 | 保持末页，不回绕 |
| PrevPage 到首页 | 回末页（循环） |
| Close 幂等 | None → None |
| Open 重置 | page = 0 |
| total_pages == 1 | NextPage/PrevPage 均无变化 |

### 11.3 消息事件对应原版事件码

| 事件 | 对应原版事件码 |
|---|---|
| Open | 13（进入）/ 主地图事件分派 |
| NextPage | 38 |
| PrevPage | 37 |
| Close | 27 |

---

## 12. 存读档系统

### 12.1 存档槽（5 个，`gl0.sav` ~ `gl4.sav`）

**v2 布局（26 字节）**：

| 偏移 | 字节数 | 字段 | 类型 | 值域 |
|---|---|---|---|---|
| 0 | 2 | 前缀 | u8[2] | `00 00` |
| 2 | 4 | 魔数 | u32 LE | `0x01315A47` |
| 6 | 4 | version | u32 LE | `2` |
| 10 | 4 | character | u32 LE | `1..=5` |
| 14 | 4 | gold | u32 LE | `0..=999999` |
| 18 | 4 | map_slot | u32 LE | `0..=4` |
| 22 | 4 | position | u32 LE | `0..` |

**v1 布局（18 字节）**：无 `version` 和 `gold` 字段。

### 12.2 v1 → v2 迁移

```
version ∈ {0, 1} → 2
gold 原样保留（v1 读档时已默认 0）
迁移幂等（再迁一次不变）
```

### 12.3 存读档流程

**读档**：

```
Index → [继续游戏] → Saveload 子窗口
  → 选槽位（38/40 循环，13 确认）
    → read_slot_auto(bytes)
      ├─ Ok(slot) → migrate_v1_to_v2 → 恢复游戏状态 → MainMap
      └─ Err(e) → 显示错误提示
```

**写档**：

```
主菜单/主地图 → [存档] → Saveload 子窗口
  → 选空槽位 → write_slot_v2(slot) → 写入 gl{N}.sav
```

### 12.4 存档门控

```
rand() % 5 != 0 → 进入存读档流程（80% 概率）
种子 = wMinute + wSecond + wMilliseconds
```

### 12.5 错误类型

| 错误 | 触发条件 | 用户提示 |
|---|---|---|
| `TooShort` | 字节数不足 | 「存档损坏」 |
| `BadMagic` | 魔数不符 | 「存档损坏」 |
| `BadVersion` | version > 2 | 「版本不兼容」 |
| `Io` | 文件系统错误 | 「读写失败」 |

---

## 13. RNG 系统

### 13.1 LCG 算法（MSVC 标准）

```
seed = (seed × 214013 + 2531011) mod 2^32
rand() = (seed >> 16) & 0x7FFF   // 值域 0..=32767
```

| 常数 | 值 |
|---|---|
| 乘数 A | 214,013（`0x343FD`） |
| 加数 C | 2,531,011（`0x269EC3`） |
| 输出掩码 | `0x7FFF` |

### 13.2 播种公式

```
seed = wMinute + wSecond + wMilliseconds
```

三处播种点：卡牌场景 ctor、存读档入口 ×2。

### 13.3 确定性要求

- 全走 `GameRng`，禁 `thread_rng`
- 战斗伤害 K 常数（`0x9E3779B1`）独立于全局 LCG
- 同 seed + 同操作序列 ⇒ 同结果（可复现、可回放、可测试）
- 测试用固定 seed = 42

### 13.4 战斗伤害公式（独立于全局 RNG）

```
damage(seed, atk) = (seed × 2654435761) % (atk + 1) + 1
atk = 0 → 恒 1
seed = 0 → 恒 1
```

---

## 14. 全局游戏状态（GameState）

```rust
pub struct GameState {
    pub character: Option<Character>,   // 角色 id 1..=5
    pub map_slot: Option<MapSlot>,      // 地图 id 1..=3
    pub gold: u32,                      // 当前金量（≤ 999999）
    pub owned: Vec<(u32, u32)>,         // (goods_id, qty)
    pub bank: Bank,                     // { gold, deposit }
    pub buildings: Vec<(Building, u32)>, // (kind, level)
    pub battle: Option<BattleView>,     // 战斗视图（HP/回合/结果）
    pub msg_kind: Option<MsgKind>,      // 消息种类
    pub msg_page: u32,
    pub msg_total: u32,
    pub save_slots: Vec<SaveSlot>,
    pub rng: GameRng,
    pub rng_seed: u32,
    pub hp: u32,                        // 玩家 HP
    pub unlocked_maps: Vec<u32>,        // 已解锁地图
}
```

**状态不变量**（每次操作后必须成立）：

| 不变量 | 断言 |
|---|---|
| 金量范围 | `0 ≤ gold ≤ gold_max` |
| HP 范围 | `0 ≤ hp ≤ hp_max` |
| 货物数量 | `0 ≤ owned[i].1 ≤ goods[i].qty_max` |
| 建筑等级 | `0 ≤ level ≤ build.max_level` |
| 场景可达 | 任意场景 ≤ `ui.max_back_steps` 步回 Index |

---

## 15. 输入系统（全场景行为表）

### 15.1 逻辑输入事件

```rust
pub enum GameInput {
    Up, Down, Left, Right,
    Confirm, Cancel, Attack,
    CycleScale, CycleFilter, CycleBorder,
}
```

### 15.2 各场景行为

**Index（主菜单）**：

| GameInput | 行为 |
|---|---|
| Up | 菜单索引 -1（循环，6 项） |
| Down | 菜单索引 +1（循环） |
| Confirm | 执行选中项 |
| Cancel | 无（已在主菜单） |

**SelectRole**：

| GameInput | 行为 |
|---|---|
| Up / Left | 上一个角色（循环） |
| Down / Right | 下一个角色（循环） |
| Confirm | 选定 → SelectMap |
| Cancel | 返回 Index |

**SelectMap**：

| GameInput | 行为 |
|---|---|
| Up / Left | 上一张地图（已解锁列表循环） |
| Down / Right | 下一张地图 |
| Confirm | 选定 → MainMap |
| Cancel | 返回 SelectRole |

**MainMap**：

| GameInput | 行为 |
|---|---|
| Up | 向北走一步 |
| Down | 向南走一步 |
| Left | 向西走一步 |
| Right | 向东走一步 |
| Confirm | 触发据点事件 |
| Cancel | 返回 Index |

**Trade**：

| GameInput | 行为 |
|---|---|
| Up / Left | 商品索引 -1 |
| Down / Right | 商品索引 +1 |
| Confirm | 确认买卖 |
| Cancel | 取消 |

**Vs（战斗）**：

| GameInput | 行为 |
|---|---|
| Confirm / Attack | 攻击一回合 |
| Cancel | 退出 → Index |

**Saveload**：

| GameInput | 行为 |
|---|---|
| Up / Left | 上一槽位 |
| Down / Right | 下一槽位 |
| Confirm | 有档→读；空槽→存 |
| Cancel | 关闭 → Index |

**Message**：

| GameInput | 行为 |
|---|---|
| Up / Left | 上一页（循环） |
| Down / Right | 下一页（末页保持） |
| Confirm / Cancel | 关闭 |

### 15.3 物理键映射

| 物理键 | GameInput |
|---|---|
| ↑ / W | Up |
| ↓ / S | Down |
| ← / A | Left |
| → / D | Right |
| Enter / Space | Confirm |
| Esc / Backspace | Cancel |
| F1 | CycleScale |
| F2 | CycleFilter |
| F3 | CycleBorder |

### 15.4 触摸手势（S5.5 实际方案）

| 场景 | 点按 | 滑动 |
|---|---|---|
| Index | 点菜单行 = 选中并执行 | — |
| SelectRole/SelectMap | 点某行 = 选中 + 确认 | 下滑 = 返回 |
| MainMap | 点格子 = 朝该格走 | 滑动 = 走一步 |
| Vs | 点任意处 = 攻击 | 下滑 = 退出 |
| Message | 左半屏 = 关闭；右半屏 = 下一页 | 上滑 = 下一页 |
| Saveload | 点有档槽 = 读档；点空槽 = 存档 | 下滑 = 关闭 |

**判定阈值**：滑动距离 > 20 逻辑像素才判定为方向手势；长按 500ms = Cancel。

---

## 16. 音效触发点

| 音效 | 文件 | 触发条件 |
|---|---|---|
| Press | `music/press.wav` | 事件码 13（有效确认） |
| Invalid | `music/invalid.wav` | 事件码 37（无效操作） |
| Victory | `music/victory.wav` | 战斗胜利 |
| BGM | `music/glmusic.ogg` | 主菜单/主地图循环播放 |

---

## 17. 数值平衡指引（调参参考，非规则）

| 参数 | 当前值 | 调参方向 |
|---|---|---|
| 战斗目标回合数 | 10~40 | 玩家 atk ≈ boss_hp / 40 |
| BOSS hp 递增 | ×1.5~2/档 | 让高等级靠建造/卡牌硬堆 |
| `rng_span` | 3 | 越大波动越大 |
| `event_chance` | 0.35 | 越高卡牌越多 |
| 据点命中率 | 60% | 越高探索奖励越多 |
| 商品 markup | 1.0 | > 1 有利润空间；= 1 无利润（纯金币回收） |

---

## 18. 实现检查清单

- [ ] 场景状态机 9 场景全覆盖，无死锁
- [ ] 事件码分派完整（触发码 + 分派码 + 用户输入码）
- [ ] 角色/地图选择循环正确
- [ ] 移动算法边界处理（越界/不可通行）
- [ ] 落格结算（据点/卡牌/BOSS 三路径）
- [ ] 贸易买卖事务性（失败状态不变）
- [ ] 建造成本公式 + 金上限抬升
- [ ] 银行存取 + 步进器
- [ ] 战斗伤害公式 + 回合流程 + 胜负判定
- [ ] 卡牌事件生成 + 效果结算 + 钳制
- [ ] 消息翻页状态机（5 类 × 边界规则）
- [ ] 存读档 v1/v2 迁移 + 校验
- [ ] RNG 确定性（同 seed 同结果）
- [ ] 输入全场景映射正确
- [ ] 音效触发点完整
