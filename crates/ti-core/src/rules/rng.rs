//! S3.12 全局 RNG 正式实现（MSVC LCG 推测算法，见 `docs/spec/rng.md` §a）。
//!
//! LCG 递推公式由本模块 [`RngState::step`] 单一源实现，[`GameRng`] 只做
//! 「结构体 + 方法」封装（`new`/`set_seed`/`next`/`fill`/`in_range`）。
//!
//! 不引 rand crate、不用 SystemRandom（EXP-040 确定性 LCG 约定，ti-core 无平台依赖）。
//! 播种点（毫秒时间）不进本模块：由调用层（场景 ctor / 存读档入口，spec §b 3 处）注入种子，
//! 本模块只负责「给定种子 → 确定序列」。

/// RNG 状态（单 u32 seed，MSVC LCG 递推，公式单一源）。
///
/// 依据 `docs/spec/rng.md` §a：`WinCE` 的 ARM CRT 中 `rand`/`srand` 与 MSVC 桌面实现一致
/// （**推测**，EXP-037/F-316）：`seed = seed * 214013 + 2531011; return (seed >> 16) & 0x7fff`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RngState {
    /// 当前 LCG 种子（`srand` 写入 / `rand` 递推更新）。
    pub seed: u32,
}

impl RngState {
    /// MSVC LCG 递推一步 + 取高 15 位（spec §a，**推测**算法）。
    #[must_use]
    #[allow(clippy::cast_possible_wrap)] // MSVC LCG 语义：取高 15 位（0..=32767）转 i32
    pub fn step(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(214_013).wrapping_add(2_531_011);
        ((self.seed >> 16) & 0x7fff) as i32
    }
}

/// oracle：播种（`srand(s)` 等价，写 seed，不产生随机值）。
pub fn srand(state: &mut RngState, s: u32) {
    state.seed = s;
}

/// oracle：取一个随机值（`rand()` 等价，值域 0..=32767）。
#[must_use]
pub fn rand(state: &mut RngState) -> i32 {
    state.step()
}

/// oracle：从当前 seed 取 n 个值（确定性序列，同 seed 同结果）。
#[must_use]
pub fn seq(state: &mut RngState, n: u32) -> Vec<i32> {
    (0..n).map(|_| state.step()).collect()
}

/// 全局游戏 RNG（MSVC LCG，单 u32 seed）。
///
/// 内部持有 [`RngState`]，递推公式只在 `RngState::step` 写一份；
/// `next` = `step`，`set_seed` = `srand`，`fill` = `seq`，行为与 LCG 序列逐位一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameRng {
    /// LCG 状态（单 u32 seed，公式源见 [`RngState::step`]）。
    state: RngState,
}

impl GameRng {
    /// 新建 RNG 并播种（`new(seed)` 等价 oracle `srand(seed)`）。
    #[must_use]
    pub fn new(seed: u32) -> Self {
        Self { state: RngState { seed } }
    }

    /// 重新播种（`set_seed(s)` 等价 oracle `srand(s)`，不产生随机值）。
    pub fn set_seed(&mut self, s: u32) {
        self.state.seed = s;
    }

    /// 取一个随机值（`next()` 等价 oracle `rand()`，MSVC LCG，值域 0..=32767）。
    ///
    /// 复用 [`RngState::step`]（公式单一源），不再独立抄写 LCG 常数。
    /// 命名为 `next` 而非实现 `Iterator`：RNG 是「取一个值」语义，不是迭代器
    /// （取完不结束、无 `size_hint`），避免与 `std::iter::Iterator::next` 混淆。
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> i32 {
        self.state.step()
    }

    /// 连续填充 `buf`（`fill` 等价 oracle `seq(buf.len())`，逐位一致）。
    pub fn fill(&mut self, buf: &mut [i32]) {
        for slot in &mut *buf {
            *slot = self.next();
        }
    }

    /// 确定性取 `0..n` 区间值（`n == 0` → 0，避免除零）。
    ///
    /// `= next().unsigned_abs() % n`（spec §a / EXP-040，确定性）；
    /// `n` 为 `u32`，`next()` 值域 0..=32767（非负），`unsigned_abs` 为防御性冗余。
    #[must_use]
    pub fn in_range(&mut self, n: u32) -> u32 {
        if n == 0 {
            return 0;
        }
        self.next().unsigned_abs() % n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_seed_resets() {
        // 两个独立 RNG 取 10 个值后状态分叉，set_seed 重置回同 seed 后序列一致
        let mut a = GameRng::new(1);
        let mut b = GameRng::new(1);
        for _ in 0..10 {
            let _ = a.next();
            let _ = b.next();
        }
        b.set_seed(1); // b 重置回 seed=1，a 仍在递推第 10 步之后
        let mut va = [0i32; 50];
        let mut vb = [0i32; 50];
        a.fill(&mut va);
        b.fill(&mut vb);
        // 注意：a 没有重置，序列不同是正常的；本测试只验证 set_seed 让 b 回到 seed=1 的确定序列
        // 用独立参照验证 b 的序列 == 全新 new(1) 的序列
        let mut c = GameRng::new(1);
        let mut vc = [0i32; 50];
        c.fill(&mut vc);
        assert_eq!(vb, vc, "set_seed(1) 后 b 的序列应等于全新 new(1) 的序列");
        // 确认 a（未重置）与 b（已重置）确实分叉
        assert_ne!(va, vb, "a 未重置应与 b 不同序列（证明 set_seed 有实效）");
    }

    #[test]
    fn in_range_bounds() {
        let mut g = GameRng::new(7);
        for _ in 0..200 {
            assert!(g.in_range(22) < 22);
        }
        assert_eq!(g.in_range(0), 0);
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
        assert_eq!(buf, single);
    }
}
