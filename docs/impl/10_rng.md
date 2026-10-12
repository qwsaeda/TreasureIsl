# 10 — RNG 系统实现规格

> 单一事实源：本文件是 `ti-core::rules::rng::GameRng` 的实现规格。
> 逆向事实引用：F-320（RNG 来源：coredll srand/rand、播种点、LCG 推测算法）。

---

## 1. LCG 算法

> **实现现状（S8.7 起）**：当前实现为 PCG（`rand_pcg::Lcg128Xsl64`，`rules/rng.rs` 库化 rand_pcg），
> 本节 MSVC LCG（214013/2531011，**推测**算法）为 **oracle 历史参照物**（S8.7 前实现），
> 仅用于 diff 对拍锚点与逆向事实记录，非当前运行时算法。

### 1.1 算法定义

采用 MSVC 标准线性同余发生器（LCG），全部 u32 环绕运算：

```
srand(s):
    seed = s

rand():
    seed = (seed * 214_013 + 2_531_011) mod 2^32
    return (seed >> 16) & 0x7FFF
```

### 1.2 常数表

| 常数 | 十进制 | 十六进制 | 说明 |
|---|---|---|---|
| 乘数 `A` | 214,013 | `0x000343FD` | MSVC LCG 乘数 |
| 加数 `C` | 2,531,011 | `0x00269EC3` | MSVC LCG 加数 |
| 掩码 `M` | 32,767 | `0x00007FFF` | 输出掩码 |
| 值域 | 0..=32,767 | 15 位 | `rand()` 返回范围 |

### 1.3 Rust 实现（公式单一源；**oracle 历史参照，S8.7 起当前为 PCG**）

> 以下为 MSVC LCG 的手写实现（`oracle/rng.rs` 历史参照物）。
> **当前代码** `rules/rng.rs` 已改用 `rand_pcg::Lcg128Xsl64`（PCG64），不再手写 LCG。

```rust
/// RNG 内部状态，LCG 常数只在此处定义
pub struct RngState {
    seed: u32,
}

impl RngState {
    const A: u32 = 214_013;
    const C: u32 = 2_531_011;

    pub fn new(seed: u32) -> Self {
        RngState { seed }
    }

    /// 单步推进，返回 rand() 值（0..=32767）
    pub fn step(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(Self::A).wrapping_add(Self::C);
        ((self.seed >> 16) & 0x7FFF) as i32
    }
}
```

### 1.4 GameRng 封装

```rust
pub struct GameRng {
    state: RngState,
}

impl GameRng {
    pub fn new(seed: u32) -> Self {
        GameRng { state: RngState::new(seed) }
    }

    /// 等价于 srand(s)
    pub fn set_seed(&mut self, s: u32) {
        self.state = RngState::new(s);
    }

    /// 等价于 rand()，复用 RngState::step（公式单一源）
    pub fn next(&mut self) -> i32 {
        self.state.step()
    }

    /// 批量填充，等价于 seq
    pub fn fill(&mut self, buf: &mut [i32]) {
        for slot in buf.iter_mut() {
            *slot = self.next();
        }
    }

    /// 范围随机：返回 0..n（n==0 时返回 0）
    pub fn in_range(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        (self.next().unsigned_abs() as u32) % n
    }
}
```

---

## 2. 播种点

### 2.1 种子公式

```
seed = wMinute + wSecond + wMilliseconds
```

- 毫秒级时间播种，秒级重复。
- 固定毫秒 → 固定种子 → 固定 `rand()` 序列，确定性可复现。

### 2.2 三处播种点

| # | 函数 | 场景 | 播种用途 |
|---|---|---|---|
| 1 | `卡牌场景` | 卡牌场景 ctor | 初始化卡牌随机序列 |
| 2 | `saveload` | 存读档入口 | 门控 `rand()%5` |
| 3 | `saveload` | 存读档入口 | 门控 `rand()%5` |

### 2.3 播种 Rust 实现

```rust
/// 从系统时间获取种子（毫秒级）
pub fn time_seed() -> u32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    // 模拟 wMinute + wSecond + wMilliseconds
    let minute = ((secs / 60) % 60) as u32;
    let second = (secs % 60) as u32;
    minute + second + millis
}
```

---

## 3. 全部 51 处 rand() 调用点

### 3.1 模块分布总览

> 各分项（battle 12 / cards 19 / main_map 8 / trade 5 / saveload 4）与 `spec/rng.md` c)
> 权威合计 51 处（35 函数）逐项吻合；§3.5/§3.6 明细表已按伪代码行逐项拆全。

| 模块 | 函数数 | rand() 调用次数 | srand 播种数 |
|---|---|---|---|
| battle | 11 | 12 | 0 |
| cards | 15 | 19 | 1 |
| main_map | 1 | 8 | 0 |
| trade | 3 | 5 | 0 |
| saveload | 2 | 4 | 2 |
| crt | 2 | 0 | 0 |
| **合计** | **35** | **51** | **3** |

### 3.2 战斗模块（12 处，11 个函数）

| # | 函数 | 表达式 | 用途 |
|---|---|---|---|
| 1 | `battle` | `rand() % 9999 + 1` | 伤害修正（1..=9999） |
| 2 | `battle` | `rand() % 6 + 1` | 伤害乘数（1..=6） |
| 3 | `battle` | `rand() % v5 + 1` | 区间随机 |
| 4 | `battle` | `byte_6C820[rand() % 12]` | 调色表随机索引 |
| 5 | `battle` | `rand() % 5 + 1` | 乘数（1..=5） |
| 6 | `battle` | `rand() % 10 + 1` | 百分比伤害基数 |
| 7 | `卡牌效果` | `rand() % 6 + 1` | 卡牌效果等级 |
| 8 | `卡牌效果` | `rand() % 9999 + 1` | 伤害修正 |
| 9 | `事件` | `rand() % 6 + 1` | 乘数 |
| 10 | `事件` | `rand() % 5 + 1` | 乘数 |
| 11 | `伤害` | `rand() % 10 + 1` | 百分比 |
| 12 | `伤害` | `rand() % 9999 + 1` | 伤害修正 |

### 3.3 卡牌模块（19 处，15 个函数）

| # | 函数 | 表达式 | 用途 |
|---|---|---|---|
| 1 | `cards` | `rand() % 26 + 1` | 26 张卡抽取 |
| 2 | `cards` | `rand() % 21 + 1` | 21 张卡面 |
| 3 | `cards` | `rand() % 6 + 1` | 效果等级 |
| 4 | `cards` | `rand() % 5 + 1` | 索引 |
| 5 | `cards` | `rand() % 9 + 1` | 索引 |
| 6 | `cards` | `rand() % 4` | 偏移 |
| 7 | `cards` | `4 * (rand() % v11)` | 偏移计算 |
| 8 | `cards` | `rand() % 10` | 库存修正 |
| 9 | `事件` | `rand() % 25 + 1` | 事件抽取 |
| 10 | `事件` | `rand() % 25` | 库存修正 |
| 11 | `事件` | `rand() % 6 + 1` | 效果等级 |
| 12 | `事件` | `rand() % 26 + 1` | 卡牌抽取 |
| 13 | `事件` | `rand() % 21 + 1` | 卡面 |
| 14 | `卡牌` | `rand() % 25 + 1` | 事件 id |
| 15 | `卡牌` | `rand() % 6 + 1` | 效果 |
| 16 | `卡牌` | `rand() % 5 + 1` | 索引 |
| 17 | `卡牌` | `rand() % 9 + 1` | 索引 |
| 18 | `卡牌` | `rand() % 4` | 偏移 |
| 19 | `卡牌` | `rand() % 10` | 库存门控 |

### 3.4 主地图模块（8 处，1 个函数）

| # | 函数 | 表达式 | 用途 |
|---|---|---|---|
| 1 | `主地图帧更新` | `rand() % 25 + 1` | 卡牌事件 id（1..=25） |
| 2 | `主地图帧更新` | `rand() % 22 + 1` | 商品选择（22 种货物） |
| 3 | `主地图帧更新` | `rand() % 25` | 卡牌库存修正 |
| 4 | `主地图帧更新` | `(rand() % 40 + 80) * v / 100` | 百分比（80%~119%） |
| 5 | `主地图帧更新` | `16 * (rand() % 6 + 1)` | 坐标步进 X |
| 6 | `主地图帧更新` | `16 * (rand() % 6 + 1)` | 坐标步进 Y |
| 7 | `主地图帧更新` | `16 * (rand() % 6 + 1)` | 坐标步进 Z |
| 8 | `主地图帧更新` | 裸 `rand()` | 通用采样 |

### 3.5 贸易模块（5 处，3 个函数）

| # | 函数 | 表达式 | 用途 |
|---|---|---|---|
| 1 | `据点` | `rand() % 30 + 1` | 行情波动（1..=30） |
| 2 | `价格` | `rand() % 100 + 1 > 40` | 买卖触发（60% 概率） |
| 3 | `价格` | `rand() % 30 + 1` | 价格修正 |
| 4 | `trade` | `rand() % 100 + 1`（行 154/342 双行） | 据点区域匹配/概率命中 |
| 5 | `trade` | `rand() % 100 + 1` | 据点资源结算（§3.4 口径合并） |

### 3.6 存读档模块（4 处，2 个函数）

| # | 函数 | 表达式 | 用途 |
|---|---|---|---|
| 1 | `saveload` | `rand() % 5`（行 ×2） | 门控（80% 通过） |
| 2 | `saveload` | `rand() % 5` | 门控（次分支） |
| 3 | `saveload` | `rand() % 5`（行 ×2） | 门控（80% 通过） |
| 4 | `saveload` | `rand() % 5` | 门控（次分支） |

### 3.7 CRT 模块（2 处，0 次调用）

| # | 函数 | 说明 |
|---|---|---|
| 1 | `__imp_rand` | coredll 导入 thunk（非游戏逻辑） |
| 2 | `__imp_srand` | coredll 导入 thunk（非游戏逻辑） |

---

## 4. 确定性要求

### 4.1 跨平台一致性

| 要求 | 说明 |
|---|---|
| 不引 `rand` crate | 避免平台相关随机源（S8.7 前约束；现库化 `rand_pcg`，`Cargo.toml` 已引 `rand_pcg = 0.9` + `rand_core`，此处表述过时，以 L249 括号注为准） |
| 不用 `SystemRandom` | ti-core 无平台依赖硬约束 |
| 自实现 LCG | 保证 native/wasm/android 序列逐位一致（S8.7 前；现 `GameRng` 库化 `rand_pcg`，旧 LCG 降级 oracle 历史参照） |
| 环绕运算 | `wrapping_mul` / `wrapping_add`，非 panic |

### 4.2 确定性验收

```
测试：tests/diff_rng.rs
    srand(123)
    取 1000 次 rand()
    oracle 序列 vs impl 序列 → 逐位一致
```

### 4.3 已知波动性

- F-319 波动性可用（S3.2 实测可用 EXP-039 / S3.12 实测失败 EXP-036）。
- 反证路径：若未来游戏侧能复现同种子序列 → 算法确认；
  否则需找 WinCE `coredll.dll` 逆向。

---

## 5. 战斗伤害常数（与全局 RNG 独立）

### 5.1 伤害常数 K

```
K = 2_654_435_761 = 0x9E3779B1   // 占位常数（非标准 Knuth，exe 实测值）
```

### 5.2 伤害公式

```
damage(seed, atk) = (seed * 2_654_435_761) % (atk + 1) + 1
```

### 5.3 独立性说明

`oracle_damage` 使用 K 常数，**不与全局 RNG 统一**（S8.7 前为全局 LCG，现为 PCG）。理由：
- 战斗模块自有的伤害公式常数，语义与全局随机序列不同。
- 强行统一会破坏已验证的 diff_battle 组合结果。

### 5.4 Rust 实现

```rust
pub const DAMAGE_K: u64 = 2_654_435_761;  // 0x9E3779B1

/// 战斗伤害计算（独立于 GameRng）
pub fn damage(seed: u32, atk: u32) -> u32 {
    ((seed as u64).wrapping_mul(DAMAGE_K) % (atk as u64 + 1)) as u32 + 1
}
```

---

## 6. 调用约束

1. **RNG 单一源**：战斗/事件随机全走 `GameRng`（可播种 PCG，`rand_pcg`，S8.7 落地），壳不另开 rand。
2. **战斗模块例外**：伤害公式使用独立 K 常数，不经过 `GameRng`。
3. **播种可见**：每次 `srand` 必须记录日志（调试用），生产可关闭。
   注：约束 3 为壳层职责，`rules::rng`（ti-core）不实现日志（无平台依赖，
   `set_seed` 纯状态重置，见 `crates/ti-core/src/rules/rng.rs`）。
4. **种子注入**：headless 测试可注入固定种子，保证可复现。
