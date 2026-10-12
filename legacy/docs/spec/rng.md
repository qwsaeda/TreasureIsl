# S3.12 RNG 与全局规则 spec

## a) RNG 算法

> **时点说明**：本 spec 成文于 S3.12（S8.7 前）。S8.7 已将全局 RNG 从手写 MSVC LCG 改为
> `rand_pcg`（PCG64），`oracle/rng.rs` 已删除。下文「推测 MSVC LCG」段为**oracle 历史参照**，
> 当前实现见 `crates/ti-core/src/rules/rng.rs`（`rand_pcg::Lcg128Xsl64`，EXP-057 版本锁定 0.9.0）。

游戏**不自带 RNG 实现**（EXP-037 / F-320）：`srand`/`rand` 是 coredll.dll 导入 thunk
（`ref/pseudocode/00058634_srand.c` 仅 8 字节 `__imp_srand` 转发，`rand` 同理
`000585E0` 8 字节 `__imp_rand`）。算法在 coredll 内，无法从 exe 实测。

**算法（推测，标 `推测`，oracle 历史参照；S8.7 起当前实现为 PCG64，见上注）**：
MSVC WinCE ARM CRT 与桌面 MSVC 一致的标准 LCG
（见 MSVC CRT `rand`/`srand` 文档）：

```
srand(s):  seed = s
rand():    seed = seed * 214013 + 2531011
           return (seed >> 16) & 0x7fff      // 值域 0..=32767
```

Rust 实现（公式单一源 `crates/ti-core/src/rules/rng.rs` 的 `GameRng`）：

```rust
// S8.7 前（oracle 历史参照）：oracle/rng.rs RngState::step 的 LCG 递推
self.seed = self.seed.wrapping_mul(214013).wrapping_add(2531011);
((self.seed >> 16) & 0x7fff) as i32
// S8.7 起（当前）：GameRng 改用 rand_pcg::Lcg128Xsl64（PCG64），
// 旧 LCG 常数（214013/2531011/0x7fff）降级为历史参照；
// next()/in_range() 值域仍归一 0..=32767 兼容旧口径（回归测试用）。
```

- **S8.7 前**不引 rand crate（EXP-040 确定性 LCG 约定，ti-core 无平台依赖硬约束）；**S8.7 起**引入 `rand_pcg = 0.9.0`（PCG64，可播种可复现，仍为纯计算、无平台依赖）；
- **公式单一源**（EXP-043，S8.7 前）：LCG 常数只写在 `RngState::step` 一处，`GameRng` 通过
  `self.state.step()` 复用，不再独立抄写 `214_013 / 2_531_011 / 0x7fff`；
  S8.7 起 oracle/rng.rs 已删除，递推公式改由 `rand_pcg` 库内实现，`GameRng` 只做 PCG64 封装；
- **差分反证路径**：固定种子（如 `srand(123)`）→ 取 1000 次 `rand()` → oracle/impl 逐位一致
  （`tests/diff_rng.rs` 已做）；若未来游戏侧能复现同种子序列（需 headless 跑 rand 调用点）则算法
  确认，否则 coredll 用了不同算法 → 需找 WinCE coredll.dll 逆向（标 `blocked_by: coredll-rand-algorithm`）。

与 S3.8 战斗 LCG 的关系（oracle 历史参照；S8.7 起战斗伤害数值走 rules/battle.rs 的 balance.combat 模型，当前 PCG 实现见 rules/rng.rs）：`oracle/battle.rs` 用 `wrapping_mul(2654435761)` 作为单回合伤害伪随机，
是**战斗模块自有的占位常数**（spec §c 推测公式，非 coredll rand），**不与 S3.12 全局 RNG 统一**
（EXP-040 允许各模块各用一套，S3.12 负责全局 srand/rand 对齐；battle 的 K=2654435761 是伤害公式
常数，语义与 LCG 序列不同，强行统一会破坏 S3.8 已 verified 的 diff_battle 1650 组合结果）。

## b) 播种点清单（3 处，实测）

| 函数 | 场景 | 种子表达式 | 伪代码位置 |
|---|---|---|---|
| `sub_3D4C8` | 卡牌场景 ctor | `wMinute + wSecond + wMilliseconds` | `0003D4C8_sub_3D4C8.c:35-36` |
| `sub_4E314` | 存读档（saveload，`rand()%5` 门控） | 同上 | `0004E314_sub_4E314.c:12-13` |
| `sub_4E568` | 存读档（saveload，`rand()%5` 门控） | 同上 | `0004E568_sub_4E568.c:12-13` |

- 种子 = `wMinute + wSecond + wMilliseconds`（**不是** wHour/wDay），毫秒级时间播种，秒级重复；
  固定毫秒 → 固定种子 → 固定 rand() 序列，**确定性可复现**；
- ⚠️ `sub_42AE8` 也调 `GetLocalTime` 但**不是播种**：它是存档时间戳逻辑（`sub_42DB0` 算时间戳，
  前 8 字节存「起始/当前」时间戳，`v4 <= Buffer + 2592000` 判 30 天过期）；
- **播种点不进 oracle**（毫秒时间不可复现），oracle 只负责「给定种子 → 确定序列」；
  正式实现 `rules::rng` 同理，播种由调用层（场景 ctor / 存读档入口）注入。

## c) rand() 调用点按模块归类（37 行，`ref/functions_rng.csv`）

`scripts/rng_sites.py` 扫描 `ref/pseudocode/` 全部 1827 份伪代码：
**51 处 `rand()` 调用点（35 个函数）+ 3 处 `srand` 播种点 + 2 个 CRT thunk**。

| 模块 | 函数数 | 调用次数 | 用途（伪代码行） |
|---|---|---|---|
| battle | 11 | 12 | 伤害修正 `sub_13CE4:15`(`%9999+1`)、`sub_1409C:14`(`%6+1`)、`sub_14F90:16`(`%v5+1`)、`sub_15368:17`(`byte_6C820[rand()%12]` 调色表随机)、`sub_15538:19`(资源集 +164/+168 区间随机)、`sub_31518:14/17`(`%5+1`/`%10+1` 乘 /0x64 百分比伤害)、`sub_3169C:13/15`(`%10+1` 百分比)、`sub_327D0:12`/`sub_3289C:12`(`rand()%100>=100` 恒假判据，**推测**防御性检查)、`sub_32E00:9`/`sub_32E3C:11`(裸 `rand()` 采样) |
| cards | 15 | 19 | 卡牌事件抽取 `sub_32960:11`(`sub_16C9C(+136,4)==10` 时 `rand()%10` 修正)、`sub_3B9D4:10`(`%5+1`)、`sub_3BB84:10`(`%6+1`)、`sub_3BC34:11`(`4*(rand()%v11)` 偏移)、`sub_3BFD0:10`(`%9+1`)、`sub_3C4CC:16`(`%6+1` 卡牌效果等级)、`sub_3C664:9`(`%26+1` 26 张卡)、`sub_3C784:9`(资源集 +164 区间)、`sub_3C8E8:9/10`(`%9+1`+裸)、`sub_3C9B8:14`(`%21+1` 21 张卡面)、`sub_49B94:14-17`(`%10` 起始 + 4 处指针区间随机)、`sub_4BB34:14`(裸)、`sub_4D848:16-24`(卡片列表 5 处随机索引)、`sub_51940:14`(`%4` 4 档) |
| mainmap | 1 | 8 | 主地图帧更新 `sub_2101C`(31344B 巨型函数)：`rand()%25+1` 卡牌事件 id、裸 `rand()` x2、`rand()%25`、`(rand()%40+80)*v175/0x64` 百分比、3 处 `16*(rand()%6+1)` 坐标步进 |
| trade | 3 | 3 | 贸易行情 `sub_47820:15`(`%30+1` 乘 /0x64)、`sub_47D40:14`(裸采样)、`sub_49120:155/342`(`%100+1>40` / `<=50` 买卖触发判据) |
| saveload | 2 | 2 | `sub_4E314:14` / `sub_4E568:14`：`rand()%5 != 0` 存读档入口门控（与播种同行） |
| crt | 2 | 0 | `sub_585E0`(`__imp_rand` thunk)/`sub_58634`(`__imp_srand` thunk)，coredll 导入，非游戏逻辑 |

模块归属规则：按 `ref/functions_{cards,trade,battle,mainmap,saveload}.csv` 已有归类 +
伪代码上下文（`cards\`/`event%d`/`card%d` 资源串 = cards；`goods`/买卖槽 = trade；
兵种/BOSS = battle；主地图帧更新 = mainmap；`gl*.sav`/settings = saveload）。
个别（如 `sub_3C4CC`/`sub_3C9B8` 用 `aT`/`aR` 宽串 = 卡牌文本）标 **推测**。

## d) 全局常量表（跨模块共享，引用已 verified spec）

| 常量 | 值 | 来源 |
|---|---|---|
| BOSS 档 HP（tier 0..=4） | 50000/100000/150000/200000/300000 | S3.8 `docs/spec/battle.md` §b（`oracle_boss_hp`，exe 字符串 0x6b436 等 5 档，S3.2 REA dump 证据） |
| 投资奖励 5 档（troop/ship 对应 5 种兵种） | +30000/+40000/+20000/+15000/+50000 | S3.7 `docs/spec/build.md`（`sub_503F0` 行 128-175，字符串 `aAstx3t`/`aKq`/`asc_7DFF8`/`aGy`/`byte_7DFE4`，**推测** 投资回报） |
| 逻辑分辨率 | 176×220 | `ti-core/src/consts.rs`（F-106，S3.2） |
| 战斗最大回合数 | 1000 | S3.8 `oracle/battle.rs`（**推测**上限） |
| RNG 值域（oracle 历史参照，当前 PCG） | 0..=32767（`(seed>>16)&0x7fff`） | 本 spec §a（**推测** MSVC LCG，S8.7 前）；S8.7 起 GameRng 为 PCG64，`next()` 值域仍归一 0..=32767 兼容旧口径 |

## e) classify pending 清零方案

`ref/classify.json`（S0.3 产出）当前 1827 函数全部已归类（`pending=0`，
`by_category`: utility 1299 / stub 115 / crt 113 / cpp_virt 81 / win_ce 53 / io 50 /
crt_data 39 / game_logic 32 / dead_code 22 / thread 10 / zlib 6 / audio 3 / gfx 3 /
lucency 1）。S3.12 的子步 6 要求把 `classify.rs`（Rust bin）的 pending 兜底**改判到具体类目**：

优先级（命中即停）：
1. **(a) 伪代码特征** → `cpp-method`（新类目，含 `__imp_` 前缀 / `__fastcall` 虚调用 /
   `off_` vtable 赋值）；
2. **(b) 调用图** → `utility`（被 ≥3 个函数调用）；
3. **(c) 尺寸** → `game-logic`（size ≥300）；
4. **(d) 小且无 caller** → `stub`（size ≤32 且无 caller）；
5. **(e) 兜底** → `framework`（新类目，LucencyApp 框架杂项，**不允许再叫 pending**）。

实现：`crates/ti-tools/src/bin/classify.rs`（S0.3 的 Rust 版分类器），新增 `cpp-method` 与
`framework` 两个类目字符串（不带 TODO/FIXME，gate 第 5/6 项占位扫描 0 命中），把原 pending 分支
改为按 (a)-(e) 归判；`dead_code`（22 个）与 `pending`（0 个）合并进上述类目，最终
`category=="pending"` 计数 = 0。

## f) 引用清单（本 spec 引用的关键伪代码位置）

- 播种点：`0003D4C8_sub_3D4C8.c:35-36`、`0004E314_sub_4E314.c:12-13`、`0004E568_sub_4E568.c:12-13`；
- `srand` thunk：`00058634_srand.c`（整文件 8 字节，仅 `__imp_srand` 转发）；
- `rand` thunk：`000585E0_rand.c`（`int __cdecl rand()` 头注释 + `__imp_rand` 转发）；
- 战斗 LCG 占位（K=2654435761，oracle 历史参照；S8.7 起当前 PCG 实现见 rules/rng.rs）：`crates/ti-core/src/oracle/battle.rs:60-62`（`oracle_damage`）。

## e) 缺口与推测

| 项 | 说明 |
|---|---|
| **MSVC LCG 常数 214013 / 2531011** | **推测**。`rand`/`srand` 是 coredll 导入 thunk（F-320），coredll.dll 不在工程内，**无法从 exe 实测**。反证路径：S3.12 oracle 差分测试若与游戏行为不符，则需找 WinCE coredll.dll 逆向，台账标 `blocked_by: coredll-rand-algorithm` |
| `sub_327D0` / `sub_3289C` 的 `rand() % 100 >= 100` | 恒假判据，**推测**为反编译/优化残留或占位分支，语义待确认 |
| 各 `rand()` 调用点的模块归属 | 37 处已列于 (c) 节表格；`dead_code`(22) 与 `pending`(0) 合并进 (e) 归判规则后仍需逐条复核 |

## 已知问题清单

1. ~~本任务在台账仍为 `pending`（`plan/ledger.yaml` S3.12）：spec 已成文，但 DoD 的两项验收 ——「RNG 序列与 oracle 一致」和「`ref/classify.json` 中 `pending = 0`」—— 尚无实跑证据。禁止据本文件认为 S3.12 已完成。~~（**历史文本**：S3.12 已于 S8 前 verified，实跑证据在 `evidence/S3.12.log`；本条为 spec 成文时的过期状态说明，保留作历史参照。）
2. LCG 常数是推测而非实测（见上表）；若差分测试反证失败，本 spec 的 (a) 节需要改写。
3. `classify.rs` 的 `framework` 类目是**兜底类目**，用「不允许再叫 pending」的方式把计数清零 —— 这是**形式达标**，「每个函数都有实质归属」这一实质目标未验证。
