# 存读档 / 设置 / RNG（Persistence）

## 设计意图

- **存档**：可读（JSON）、可迁移（v1→v5）、可复现（存 RNG 状态）；
- **设置**：音量/画质/存档位等玩家偏好；
- **RNG**：可播种、可复现，同 seed 同操作序列必得同结果——这是全流程回归测试的基础。

## 存档结构（v5，当前 = `SaveSlotV5`）

存 `assets/data/` 无关，存**运行时状态**。用 JSON（人可读、可 diff、可迁移）：

```jsonc
// save v5 schema（v5 = 席位全量 + 领地图，S33.4；席位/领地字段见 territory.md §8）
// 注：正式 JSON 的角色/地图字段为**稳定字符串 label**（panda/tichy/… + m01/m02/m03），
// 旧 26 字节数字槽位经 `from_legacy_slot` 保守映射（legacy 数字均 1 起：character 1..=5 → 角色 label，
// map_slot 1..=3 → m0N；0/越界钳位到首项，产物必可被 `character_from_label`/`map_slot_from_label` 回读，
// 第 12 轮审计 Q010 修复：旧 "char{N}"/0 起 "m0N" 映射不可解析已废弃），
// 见 `crates/ti-core/src/rules/saveload_json.rs`。
// buildings/goods 的 id 以 balance.json 为准（Castle/Investment/Bank/Fort 字符串 id、goods.1..5 数字 id）。
{
  "version": 5,
  "character": "drake",          // 主玩家席位角色；席位全量在 `seats`
  "map": "m01",                  // 地图 id
  "scene": "MainMap",            // 退出时所在场景
  "position": { "row": 5, "col": 7 },  // v4 起席位位置走 seats[].position（保留 v3 兼容）
  "gold": 1200,                  // v4 起走 seats[]（保留 v3 兼容）
  "hp": 85,
  "buildings": ["Castle", "Bank"],     // v4 起走 seats[].buildings（保留 v3 兼容）
  "goods": [ { "id": 1, "qty": 5 } ],
  "rng_seed": 0,                 // 复现关键：固定默认种子（存时间戳会破坏可复现性）
  "rng_steps": 0,                // RNG 已消费 next_u64 次数
  "bank": 300,
  "unlocked_maps": ["m01", "m02"],
  "boat_level": 1,               // 1..=4（船等级，缺省 1）
  "seats": [ /* 席位全量 = 主玩家 + 4 AI（人手/武器/建筑/位置），见 territory.md §8 */ ],
  "territories": [ /* 领地图 = 谁占哪格/驻军/加固等级，见 territory.md §8 */ ],
  "current_seat": 0              // 热座交接：当前轮到哪个席位
}
```

### 迁移

- v1 → v2：v1 缺 `version`/`bank`/`rng_seed`，迁移时补默认值
  （`version=2`、`bank=0`、`rng_seed=0`（固定默认种子，保证可复现，不存时间戳））；
- 迁移函数纯函数、幂等、可单测（`tests/save_migration.rs`，已存在）；
  注：business 可测规则单测（`save_*` 等）待 S8.3.d 落地。
- app 侧存档槽（`rules::saveload::SaveSlotV5`，JSON v5 schema）：v3 起 `unlocked_maps: Vec<String>`；
  S15.2 时代的 `unlocked_mask: u8`（逐图推进位，bit 0/1/2 = m01/m02/m03 已解锁，bit 3 = CLEARED 通关位）
  为 v2 字段，v3 起改为列表；字段口径与读档还原规则 = [`progression.md`](progression.md) §与存档一致（单一事实源）。
- **v2 → v3（S21.2 船系统，实现 S21.11）**：新增 `boat_level: u8`（1..=4，船等级 1-based，
  `balance.boat.levels` 有效域）；v2 旧档缺字段 = 迁移补 **1（小船）**，不回退不报错；
  迁移纯函数、幂等（再迁不变），单测锚点 `save_migrate_v2_to_v3_defaults`。
- **v3 → v4（S28.2 存档 v4，领地系统）**：新增 `seats: Vec<SeatSave>`（席位全量 = 主玩家 + 4 AI）+
  `territories: Vec<TerritorySave>`（领地图）+ `current_seat: u8`；v3 旧档补单席位 + 空领地图。
- **v4 → v5（S33.4 存档 v5，领地 v2 兵力模型）**：席位补默认人手/武器
  （`balance.manpower.initial_crew/arms`）+ 领地驻军空；迁移纯函数、幂等
  （单测锚点 `save_migrate_v4_to_v5_defaults`），见 [`territory.md`](territory.md) §8。

## RNG 设计（可复现）

| 场景 | RNG |
|---|---|
| 战斗伤害浮动 | `GameRng`（可播种 PCG，`rand_pcg`，S8.7 落地） |
| 卡牌抽牌 | 同上，共享一个可播种 RNG 流 |
| 全流程回归测试 | 固定 seed（测试用 `42`），同脚本 ⇒ 同结果 |

约定：

- 玩法层**只经 `rules::rng::GameRng` 取随机**，禁 `rand::thread_rng`（不可复现）；
- RNG 状态进存档，保证「读档后继续」的随机序列可复现；
- 目标库复用：`rand` + `rand_pcg`（见 [`process/01_architecture.md`](../process/01_architecture.md) §4，S8.7 落地），当前实现为库化 `rand_pcg`（自造 LCG 已删除，见 S8.7）。

## 设置

音量（BGM/SE）、画质（滤波/HD）、存档位等。存 `ti-platform::settings`（`Settings` 结构）+
`ti-formats::save`（`settings.dat` 读写），三端一致。设置**不影响玩法数值**，只影响表现与音频。

## 可测试规则

| 规则 | 单测 |
|---|---|
| 存→读往返逐字段一致 | `save_roundtrip_preserves_fields`（已落地于 `crates/ti-core/tests/rules_saveload_json.rs`） |
| v1 → v2 迁移补默认值 | `save_migrate_v1_to_v2_defaults`（同上） |
| 迁移幂等（再迁一次不变） | `save_migration_idempotent`（同上） |
| 同 seed + 同操作 ⇒ 同随机流 | `rng_same_seed_same_stream`（同上） |
| 存档含 rng_seed，读档后可续 | `save_rng_seed_restores_stream`（同上） |
| 设置不改变玩法状态 | `settings_do_not_affect_gameplay`（同上） |