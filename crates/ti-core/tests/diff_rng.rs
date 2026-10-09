//! S3.12 全局 RNG 性质测试。
//!
//! 遍历 seed 0..=20 × 取 1000 个值：`GameRng::fill` 序列确定性（同 seed 同序列，
//! MSVC LCG 推测算法，见 `docs/spec/rng.md` §a）；另加 5 个语义测试：
//! (a) 固定 seed=123 序列确定（两次独立调用 `GameRng::new(123).fill` 逐位相同）；
//! (b) `set_seed` 改 seed 后序列变（不同 seed 产生不同序列）；
//! (c) `in_range(22)` 值域 0..22（200 次采样无越界，`n=0` → 0）；
//! (d) `next()` 值域 0..=32767（15 位）；
//! (e) `fill` 与逐次 `next` 逐位一致。
//!
//! 置于集成测试目录，豁免门禁的 unwrap/expect 扫描（EXP-021）。

use ti_core::rules::rng::GameRng;

/// 断言同 seed 取 n 个值时 `fill` 与独立 `next` 序列逐位一致。
fn assert_fill_matches_next(seed: u32, n: u32) {
    let mut g1 = GameRng::new(seed);
    let mut buf = vec![0i32; n as usize];
    g1.fill(&mut buf);

    let mut g2 = GameRng::new(seed);
    let mut single = vec![0i32; n as usize];
    for slot in &mut single {
        *slot = g2.next();
    }

    assert_eq!(buf, single, "fill vs next 序列不一致 (seed={seed}, n={n})");
}

/// 遍历 seed 0..=20 × n=1000：`fill` 序列确定性（同 seed 两次独立调用逐位相同）。
#[test]
fn diff_rng_all_seeds_deterministic() {
    for seed in 0..=20u32 {
        let mut a = GameRng::new(seed);
        let mut b = GameRng::new(seed);
        let mut va = vec![0i32; 1000];
        let mut vb = vec![0i32; 1000];
        a.fill(&mut va);
        b.fill(&mut vb);
        assert_eq!(va, vb, "同 seed={seed} 两次独立调用 fill 序列不同（确定性违例）");
    }
}

/// 遍历 seed 0..=20 × n=1..=7：`fill` 与逐次 `next` 逐位一致（同源验证）。
#[test]
fn diff_rng_fill_matches_next_varied_lengths() {
    for seed in 0..=20u32 {
        for n in 1..=7u32 {
            assert_fill_matches_next(seed, n);
        }
    }
}

/// 语义 (a)：固定 seed=123 序列确定（两次独立调用 `new(123).fill` 逐位相同）。
#[test]
fn semantic_a_fixed_seed_deterministic() {
    let mut a = GameRng::new(123);
    let mut b = GameRng::new(123);
    let mut va = vec![0i32; 512];
    let mut vb = vec![0i32; 512];
    a.fill(&mut va);
    b.fill(&mut vb);
    assert_eq!(va, vb, "同 seed=123 两次独立调用 fill 序列不同（确定性违例）");
}

/// 语义 (b)：`set_seed` 改 seed 后序列变（不同 seed 产生不同序列）。
#[test]
fn semantic_b_set_seed_changes_sequence() {
    let mut g1 = GameRng::new(123);
    let mut g2 = GameRng::new(123);
    g2.set_seed(321); // 改 seed
    let mut v1 = [0i32; 256];
    let mut v2 = [0i32; 256];
    g1.fill(&mut v1);
    g2.fill(&mut v2);
    assert_ne!(v1, v2, "set_seed(321) 后序列未变（改 seed 应改变后续随机序列）");
}

/// 语义 (c)：`in_range(22)` 值域 0..22（200 次采样无越界，`n=0` → 0）。
#[test]
fn semantic_c_in_range_bounds() {
    let mut g = GameRng::new(99);
    for _ in 0..200 {
        let v = g.in_range(22);
        assert!(v < 22, "in_range(22) 越界：v={v} (应 0..=21)");
    }
    assert_eq!(g.in_range(0), 0, "in_range(0) 应为 0（避免除零）");
}

/// 值域检查：MSVC LCG `(seed>>16)&0x7fff` → 0..=32767（15 位）。
#[test]
fn value_range_is_15bit() {
    let mut g = GameRng::new(0);
    for _ in 0..1000 {
        let v = g.next();
        assert!((0..=32_767).contains(&v), "next() 值域越界：{v}");
    }
}

/// 语义 (e)：`fill` 与逐次 `next` 逐位一致（同一 RNG 的两种取法应产生相同序列）。
#[test]
fn semantic_fill_matches_next() {
    for seed in 0..=30u32 {
        let mut g1 = GameRng::new(seed);
        let mut buf = [0i32; 16];
        g1.fill(&mut buf);

        let mut g2 = GameRng::new(seed);
        let mut single = [0i32; 16];
        for s in &mut single {
            *s = g2.next();
        }
        assert_eq!(buf, single, "fill vs next 不一致 (seed={seed})");
    }
}
