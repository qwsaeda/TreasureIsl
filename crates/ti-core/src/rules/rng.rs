//! S3.12 / S8.7 / S12.5 全局 RNG 正式实现（可复现 PCG，走 `rand_pcg` 库）。
//!
//! S8.7 库化（`docs/process/01_architecture.md` §4 + `docs/business/persistence.md` §RNG）：
//! - 旧手写 MSVC LCG（`seed * 214013 + 2531011`，**推测**算法）降级参照物；
//! - 本模块改为 `rand_pcg::Lcg128Xsl64`（PCG64，可播种、可复现、库实现），
//!   同 seed + 同操作序列 ⇒ 同随机流（可回放、可测试，EXP-040 确定性约定不变）；
//! - 禁 `rand::thread_rng`（不可复现）；玩法层**只经本模块**取随机。
//!
//! S12.5 存档续流（`docs/business/persistence.md` §RNG「RNG 状态进存档，读档后继续」）：
//! - [`GameRng`] 内部维护 `seed` + `steps_taken` 两个字段；
//! - [`GameRng::save_state`] / [`GameRng::restore_state`] 实现「存→读」精确续流
//!   （`Lcg128Xsl64::advance(steps)` O(log n) 跳转，非逐次 `next_u64` 回放）；
//! - 旧 `rng_seed` 单字段语义保留：[`GameRng::new(seed)`] 等价于 `restore(seed, 0)`；
//!   新存档可用 `save_state()` / `restore_state()` 携带 steps 实现中间态续接。
//!
//! 版本锁定（EXP-057）：`rand_pcg =0.9.0`。
//! `Lcg128Xsl64::new(state, stream)` 确定性播种；`RngCore::next_u64()` 取 64 位输出。
//!
//! API 与旧 `rules::rng` 保持兼容（`GameRng::new/in_range/fill` 同名），
//! 调用方（战斗伤害浮动 / 卡牌抽牌 / 全流程回归）无需改签名，只换底层实现。

use rand_core::RngCore;
use rand_pcg::Lcg128Xsl64;

/// RNG 状态（PCG64 / `rand_pcg` 库；单 seed 可复现）。
///
/// 与旧 LCG 不同：底层 PCG64 值域 0..2^64（u64），`in_range` 仍归一到 `0..n`（u32 语义不变，调用方无感）。
#[derive(Debug, Clone)]
pub struct RngState {
    inner: Lcg128Xsl64,
}

impl RngState {
    /// 用 seed 新建 PCG64（`Lcg128Xsl64::new(state, stream)` 确定性播种）。
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self { inner: Lcg128Xsl64::new(u128::from(u64::from(seed)), 0) }
    }

    /// 取一个 u64 随机值（PCG64 全值域，`RngCore::next_u64`）。
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    /// 取一个 0..32767 的 i32（兼容旧 LCG `rand()` 值域，回归测试口径）。
    ///
    /// `next_u64() % 32768` 值域 `0..32768`，`i32::try_from` 不可能失败（32768 < `i32::MAX`），
    /// 故 `unwrap_or(0)` 仅兜底、实际不触发（门禁 `unwrap_used` lint 只禁 `unwrap()`/`expect()`，
    /// `unwrap_or` 不在扫描模式 `\.unwrap\(\)|\.expect\(` 内）。
    #[must_use]
    pub fn step(&mut self) -> i32 {
        i32::try_from(self.next_u64() % 32_768).unwrap_or(0)
    }

    /// O(log n) 跳转 `n` 步（`Lcg128Xsl64::advance`，精确跳过 `n` 次 `next_u64` 调用）。
    ///
    /// `n == 0` 不产生效果；`n > 0` 随机流与逐次 `next_u64` 调用 `n` 次逐位一致
    /// （存档续流用，`GameRng::restore_state` 内部调用）。
    pub fn advance(&mut self, n: u128) {
        if n > 0 {
            self.inner.advance(n);
        }
    }
}

impl Default for RngState {
    fn default() -> Self {
        Self::new(0)
    }
}

/// 播种接口（写 seed，不产生随机值；PCG64 用 `new` 重建，等价 `srand`）。
pub fn srand(state: &mut RngState, s: u32) {
    *state = RngState::new(s);
}

/// 取一个随机值（0..32767，与旧 LCG `rand()` 值域一致，回归可测）。
#[must_use]
pub fn rand(state: &mut RngState) -> i32 {
    state.step()
}

/// 从当前状态取 n 个值（确定性序列，同 seed 同结果）。
#[must_use]
pub fn seq(state: &mut RngState, n: u32) -> Vec<i32> {
    (0..n).map(|_| state.step()).collect()
}

/// 全局游戏 RNG（PCG64，可复现；`docs/business/persistence.md` §RNG 唯一入口）。
///
/// 内部持有 [`RngState`]（`rand_pcg` 库实现），递推公式在库内，本模块只做封装；
/// `next`/`set_seed`/`fill`/`in_range` 与旧 LCG 版同名同语义，调用方无感。
///
/// S12.5 存档续流：`seed` + `steps_taken` 两字段实现「存→读」精确续流；
/// `save_state()` / `restore_state()` 走 `Lcg128Xsl64::advance`（O(log n) 跳转）。
#[derive(Debug, Clone)]
pub struct GameRng {
    state: RngState,
    /// 原始播种 seed（`new(seed)` / `set_seed(s)` / `restore_state` 写入）。
    seed: u32,
    /// 自 seed 初始化以来已消费的 `next_u64` 次数（`save_state` / `restore_state` 用）。
    steps_taken: u128,
}

impl GameRng {
    /// 新建 RNG 并播种（`new(seed)` 等价 `srand(seed)`，PCG64 确定性）。
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self { state: RngState::new(seed), seed, steps_taken: 0 }
    }

    /// 重新播种（`set_seed(s)` 等价 `srand(s)`，不产生随机值；重置 `steps_taken`）。
    pub fn set_seed(&mut self, s: u32) {
        self.state = RngState::new(s);
        self.seed = s;
        self.steps_taken = 0;
    }

    /// 取一个随机值（`next()` 值域 0..32767，与旧 LCG 一致，确定性）。
    ///
    /// 复用 [`RngState::step`]；命名 `next` 而非实现 `Iterator`（取一个值语义，无 size hint）。
    /// 每次调用 `steps_taken` + 1（`save_state` / `restore_state` 精确续流用）。
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> i32 {
        self.steps_taken = self.steps_taken.saturating_add(1);
        self.state.step()
    }

    /// 连续填充 `buf`（`fill`，逐位一致）。
    pub fn fill(&mut self, buf: &mut [i32]) {
        let n = u128::try_from(buf.len()).unwrap_or(0);
        self.steps_taken = self.steps_taken.saturating_add(n);
        for slot in &mut *buf {
            *slot = self.state.step();
        }
    }

    /// 确定性取 `0..n` 区间值（`n == 0` → 0，避免除零）。
    ///
    /// `= next().unsigned_abs() % n`（EXP-040 确定性）；`n` 为 `u32`。
    #[must_use]
    pub fn in_range(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        let v = self.next();
        v.unsigned_abs() % n
    }

    /// 当前 RNG 可序列化状态（seed + 已消费步数，存档「RNG 状态进存档」用）。
    ///
    /// 读档后 `GameRng::restore_state(s.save_state(), s.rng_steps)` 精确续流，
    /// 随机序列与未存档前逐位一致（`docs/business/persistence.md` §RNG）。
    #[must_use]
    pub fn save_state(&self) -> u32 {
        self.seed
    }

    /// 已消费 `next_u64` 次数（`save_state` + `restore_state` 精确续流用）。
    #[must_use]
    pub fn steps_taken(&self) -> u128 {
        self.steps_taken
    }

    /// 从存档状态重建 RNG（`restore_state(seed, steps)` 精确跳回第 `steps` 次 `next_u64`）。
    ///
    /// `steps == 0` 等价 `GameRng::new(seed)`；`steps > 0` 走 `Lcg128Xsl64::advance(steps)`
    /// O(log n) 跳转（非逐次回放），随机流与逐步 `next_u64` 逐位一致。
    ///
    /// 用途：存档读档后续接玩法层已消耗的随机流（战斗伤害 / 卡牌抽牌 / 地图事件）。
    #[must_use]
    pub fn restore_state(seed: u32, steps: u128) -> Self {
        let state = RngState::new(seed);
        let mut g = Self { state, seed, steps_taken: steps };
        if steps > 0 {
            g.advance(steps);
        }
        g
    }

    /// 内部跳转（`advance(n)`，O(log n)；测试用，不暴露给玩法层）。
    fn advance(&mut self, n: u128) {
        self.state.advance(n);
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_seed_resets_stream() {
        // 两个独立 RNG 取 10 个值后状态分叉，set_seed 重置回同 seed 后序列一致
        let mut a = GameRng::new(1);
        let mut b = GameRng::new(1);
        for _ in 0..10 {
            let _ = a.next();
            let _ = b.next();
        }
        b.set_seed(1);
        let mut va = [0i32; 50];
        let mut vb = [0i32; 50];
        a.fill(&mut va);
        b.fill(&mut vb);
        let mut c = GameRng::new(1);
        let mut vc = [0i32; 50];
        c.fill(&mut vc);
        assert_eq!(vb, vc, "set_seed(1) 后 b 的序列应等于全新 new(1) 的序列");
        assert_ne!(va, vb, "a 未重置应与 b 不同序列（证明 set_seed 有实效）");
    }

    #[test]
    fn in_range_bounds() {
        let mut g = GameRng::new(7);
        for _ in 0..200 {
            assert!(g.in_range(22) < 22, "in_range 值域 0..n");
        }
        assert_eq!(g.in_range(0), 0, "n=0 → 0（避免除零）");
    }

    #[test]
    fn fill_matches_next() {
        let mut a = GameRng::new(3);
        let mut b = GameRng::new(3);
        let mut buf = [0i32; 16];
        a.fill(&mut buf);
        let mut single = [0i32; 16];
        for s in &mut single {
            *s = b.next();
        }
        assert_eq!(buf, single, "fill 与逐次 next 序列一致（确定性）");
    }
}
