# 存读档 / 设置 / RNG（Persistence）

## 设计意图

- **存档**：可读（JSON）、可迁移（v1→v2）、可复现（存 RNG 状态）；
- **设置**：音量/画质/存档位等玩家偏好；
- **RNG**：可播种、可复现，同 seed 同操作序列必得同结果——这是全流程回归测试的基础。

## 存档结构（v2）

存 `assets/data/` 无关，存**运行时状态**。用 JSON（人可读、可 diff、可迁移）：

```jsonc
// save v2 schema（v2 字段以 design/persistence.md 为准，此处为最小示例）
// 注：示例用业务名；正式 JSON v2 的角色/地图字段为**稳定字符串 label**（panda/tichy/… + m01/m02/m03），
// 旧 26 字节数字槽位经 `from_legacy_slot` 保守映射（legacy 数字均 1 起：character 1..=5 → 角色 label，
// map_slot 1..=3 → m0N；0/越界钳位到首项，产物必可被 `character_from_label`/`map_slot_from_label` 回读，
// 第 12 轮审计 Q010 修复：旧 "char{N}"/0 起 "m0N" 映射不可解析已废弃），
// 见 `crates/ti-core/src/rules/saveload_json.rs`。
// buildings/goods 的 id 以 balance.json 为准（Castle/Investment/Bank/Fort 字符串 id、
// goods.1..5 数字 id）；下方 palisade/harbor/spice 等为旧设计占位名，照抄会写错。
{
  "version": 2,
  "character": "drake",          // 角色 id（business/rng 无关）
  "map": "m01",                  // 地图 id
  "scene": "MainMap",            // 退出时所在场景
  "position": { "row": 5, "col": 7 },  // 玩家所在格
  "gold": 1200,
  "hp": 85,
  "buildings": ["Castle", "Bank"],
  "goods": [ { "id": 1, "qty": 5 } ],
  "rng_seed": 0,                 // 复现关键：固定默认种子（存时间戳会破坏可复现性）
  "bank": 300,
  "unlocked_maps": ["m01", "m02"]
}
```

### 迁移

- v1 → v2：v1 缺 `version`/`bank`/`rng_seed`，迁移时补默认值
  （`version=2`、`bank=0`、`rng_seed=0`（固定默认种子，保证可复现，不存时间戳））；
- 迁移函数纯函数、幂等、可单测（`tests/save_migration.rs`，已存在）；
  注：business 可测规则单测（`save_*` 等）待 S8.3.d 落地。
- app 侧存档槽（`domain::persistence::SaveSlot`，JSON v2 schema）：S15.2 新增 `unlocked_mask: u8`
  （逐图推进位，bit 0/1/2 = m01/m02/m03 已解锁，bit 3 = CLEARED 通关位，旧档缺字节 = 0 = 退最小前置链）；
  字段口径与读档还原规则 = [`progression.md`](progression.md) §与存档一致（单一事实源）。

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