//! S74.7 · RNG 自由函数补测（变异存活子补杀）。
//!
//! 背景（S74.4 变异基线）：9 个存活变异子全在 `rng.rs` 的自由函数/方法上——
//! `srand` 删语句、`rand` 固定返回 -1/0/1、`seq` 固定返回 vec![-1]/[0]/[1]/[]、
//! `RngState::advance` 的 `> 0` 改 `>= 0`。现有 RNG 测试只覆盖 `GameRng` API，
//! 这些自由函数从未被测试调用 ⇒ 改坏它们测试全绿（「测试抓不到 bug」的活样本）。
//!
//! 本文件用**相对性质**（不依赖 PCG64 具体 golden 值）逐一封死：
//! - `srand`：重播种后序列回到同 seed 的新序列（杀「删语句」）；
//! - `rand`：值域 0..=32767 + 连续取值不全相同（杀「固定 -1/0/1」）；
//! - `seq`：长度 == n + 与逐次 `rand` 逐位一致 + 连续两次 `seq` 分叉（杀「固定 vec![]/[-1]/[0]/[1]」）；
//! - `RngState::advance`：`advance(n)` 与逐次 `next_u64` n 次后续流一致（杀边界变异，若等价则如实标注）。

#![allow(clippy::unwrap_used, clippy::expect_used)]

use ti_core::rules::rng::{RngState, rand, seq, srand};

/// srand 重播种必须让序列回到同 seed 的新状态。
/// 变异「replace srand with ()」后：state 不重置，序列继续推进 ⇒ 两次不等 ⇒ 杀死。
#[test]
fn srand_resets_stream_to_seed() {
    let mut fresh = RngState::new(42);
    let expected = seq(&mut fresh, 16);

    let mut state = RngState::new(42);
    let _first = seq(&mut state, 16); // 先消费 16 个，推进状态
    srand(&mut state, 42); // 重播种回 seed=42
    let after = seq(&mut state, 16);

    assert_eq!(after, expected, "srand 应把状态重置回 seed=42 的新序列");
}

/// rand 值域必须 0..=32767（旧 MSVC LCG 15 位兼容口径）。
/// 变异「rand → -1」⇒ 越界 ⇒ 杀死。
#[test]
fn rand_value_range_15bit() {
    for seed in [0u32, 1, 7, 42, 123, 999] {
        let mut state = RngState::new(seed);
        for _ in 0..256 {
            let v = rand(&mut state);
            assert!((0..=32_767).contains(&v), "rand() 值域越界：seed={seed} v={v}");
        }
    }
}

/// rand 连续取值不应恒定（真随机流）。
/// 变异「rand → 固定 0 或 1」⇒ 全部相同 ⇒ 杀死（固定 -1 已由值域测试杀）。
#[test]
fn rand_stream_is_not_constant() {
    for seed in [0u32, 1, 7, 42, 123] {
        let mut state = RngState::new(seed);
        let values: Vec<i32> = (0..32).map(|_| rand(&mut state)).collect();
        assert!(
            values.iter().any(|&v| v != values[0]),
            "rand() 连续 32 次不应恒定 (seed={seed})：{values:?}"
        );
    }
}

/// seq(n) 长度必须恰为 n。
/// 变异「seq → vec![] / vec![-1] / vec![0] / vec![1]」⇒ n=5 时长度不符 ⇒ 全部杀死。
#[test]
fn seq_length_equals_n() {
    let mut state = RngState::new(7);
    for n in [0u32, 1, 2, 5, 16, 33] {
        assert_eq!(seq(&mut state, n).len(), n as usize, "seq(n={n}) 长度应为 {n}");
    }
}

/// `seq(n)` 必须与逐次 `rand()` n 次逐位一致（同源验证）。
/// 配合长度测试：即使变异凑巧长度对（如 n=1），固定返回值也与真实 `rand` 不同 ⇒ 杀死。
#[test]
fn seq_matches_repeated_rand() {
    for seed in [0u32, 3, 17, 42] {
        let mut a = RngState::new(seed);
        let via_seq = seq(&mut a, 24);

        let mut b = RngState::new(seed);
        let via_rand: Vec<i32> = (0..24).map(|_| rand(&mut b)).collect();

        assert_eq!(via_seq, via_rand, "seq(24) 应与逐次 rand() 24 次逐位一致 (seed={seed})");
    }
}

/// seq 连续两次调用必须推进状态（第二次序列 ≠ 第一次）。
/// 变异「seq → 固定序列」⇒ 两次相同 ⇒ 杀死。
#[test]
fn seq_advances_state_between_calls() {
    let mut state = RngState::new(99);
    let first = seq(&mut state, 12);
    let second = seq(&mut state, 12);
    assert_ne!(first, second, "连续两次 seq(12) 应分叉（状态已推进）");
}

/// `RngState::advance(0)` 必须 no-op。
/// 变异「n > 0 → n >= 0」：PCG64 的 `advance(0)` 本身 no-op，此变异可能等价（无法杀死）；
/// 本测试仍守住「`advance(0)` 不改变流」的语义（若底层库变更即被捕获）。
#[test]
fn advance_zero_is_noop() {
    let mut a = RngState::new(5);
    let mut b = RngState::new(5);
    a.advance(0);
    let va: Vec<u64> = (0..8).map(|_| a.next_u64()).collect();
    let vb: Vec<u64> = (0..8).map(|_| b.next_u64()).collect();
    assert_eq!(va, vb, "advance(0) 应不改变随机流");
}

/// `RngState::advance(n)` 必须与逐次 `next_u64` n 次后续流一致（存档续流语义）。
#[test]
fn advance_n_matches_consuming_n() {
    for steps in [1u128, 3, 17, 100] {
        let mut consumed = RngState::new(2024);
        for _ in 0..steps {
            let _ = consumed.next_u64();
        }

        let mut advanced = RngState::new(2024);
        advanced.advance(steps);

        let next_consumed = consumed.next_u64();
        let next_advanced = advanced.next_u64();
        assert_eq!(
            next_advanced, next_consumed,
            "advance({steps}) 后下一值应等于逐次消费 {steps} 次后的下一值"
        );
    }
}
