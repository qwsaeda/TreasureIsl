# 存读档 / 设置 / RNG（Persistence）

## 设计意图

- **存档**：可读（JSON）、可迁移（v1→v2）、可复现（存 RNG 状态）；
- **设置**：音量/画质/存档位等玩家偏好；
- **RNG**：可播种、可复现，同 seed 同操作序列必得同结果——这是全流程回归测试的基础。

## 存档结构（v2）

存 `assets/data/` 无关，存**运行时状态**。用 JSON（人可读、可 diff、可迁移）：

```jsonc
// save v2 schema（v2 字段以 design/persistence.md 为准，此处为最小示例）
// 注：示例用业务名；`rules::saveload_json` 序列化格式为 char{N}/m{NN}（数字索引+1），
// 见 `crates/ti-core/src/rules/saveload_json.rs`（from_legacy_slot 保守映射）。
{
  "version": 2,
  "character": "drake",          // 角色 id（business/rng 无关）
  "map": "m01",                  // 地图 id
  "scene": "MainMap",            // 退出时所在场景
  "position": { "row": 5, "col": 7 },  // 玩家所在格
  "gold": 1200,
  "hp": 85,
  "buildings": ["palisade", "harbor"],
  "goods": [ { "id": "spice", "qty": 5 } ],
  "rng_seed": 12345,             // 复现关键：存 RNG 当前状态/种子
  "bank": 300,
  "unlocked_maps": ["m01", "m02"]
}
```

### 迁移

- v1 → v2：v1 缺 `version`/`bank`/`rng_seed`，迁移时补默认值
  （`version=2`、`bank=0`、`rng_seed` 由存档时间或默认种子填）；
- 迁移函数纯函数、幂等、可单测（`tests/save_migration.rs`，已存在）；
  注：business 可测规则单测（`save_*` 等）待 S8.3.d 落地。

## RNG 设计（可复现）

| 场景 | RNG |
|---|---|
| 战斗伤害浮动 | `GameRng`（可播种 PCG，`rand_pcg`，S8.7 落地） |
| 卡牌抽牌 | 同上，共享一个可播种 RNG 流 |
| 全流程回归测试 | 固定 seed（测试用 `42`），同脚本 ⇒ 同结果 |

约定：

- 玩法层**只经 `rules::rng::GameRng` 取随机**，禁 `rand::thread_rng`（不可复现）；
- RNG 状态进存档，保证「读档后继续」的随机序列可复现；
- 目标库复用：`rand` + `rand_pcg`（见 [`process/01_architecture.md`](../process/01_architecture.md) §4，S8.7 落地），当前实现为库化 `rand_pcg`（旧自造 LCG 为 oracle 历史参照）。

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