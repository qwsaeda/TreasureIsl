#![allow(clippy::unwrap_used, clippy::expect_used)]
//! 飞轮 R21 / S61 · AI 对抗对局快照导出（`adversarial_shot.py` 出图的数据源）。
//!
//! 经 `cargo test --test adversarial_snapshot -- seeds=..,seats=..,rounds=..` 跑 N seed headless
//! 对局，dump 终局快照（胜者/回合/各席位置/金/兵力/存活 + BOSS 终结标记）到
//! `evidence/adv_snapshot.json`；`scripts/adversarial_shot.py` 读它出「各环节 PNG」。
//! 本测试本身也断言快照自洽（可当普通回归跑：0 参数 = 默认 seed 集，快照写盘 + 结构校验）。

mod adversarial_common;
use adversarial_common::{GameOutcome, run_adversarial_game};
use std::path::Path;

/// 把 [`GameOutcome`] 序列化成快照记录（`adversarial_shot.py` 消费的 JSON 形状）。
fn outcome_record(seed: u32, seats: u8, out: &GameOutcome, boss_final: bool) -> serde_json::Value {
    serde_json::json!({
        "seed": seed,
        "seats": seats,
        "winner": out.winner,
        "rounds": out.rounds,
        "positions": out.positions,
        "alive": out.alive,
        "gold": out.gold,
        "crew": out.crew,
        "boss_final": boss_final,
    })
}

/// 跑一批对局 + 写快照（`seeds` 逗号分隔；`rounds` 兜底上限）。
fn dump(seeds: &[u32], seats: u8, rounds: u32) -> Vec<serde_json::Value> {
    let mut recs = Vec::new();
    for &seed in seeds {
        let out = run_adversarial_game(seed, seats, rounds);
        // BOSS 终结标记：胜者非 None 且非全破产终局 = 经 BOSS/唯一存活收敛（驱动器内 BOSS engaged
        // 即返回，故 winner=Some 且 rounds < rounds = 收敛于 BOSS/终局判据）。
        let boss_final = out.winner.is_some() && out.rounds < rounds;
        recs.push(outcome_record(seed, seats, &out, boss_final));
    }
    // `CARGO_MANIFEST_DIR` = `crates/ti-core`；repo 根 = 上上级（`crates/../` = 仓库根）。
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root（crates/ti-core → crates → 仓库根）");
    let snap = repo_root.join("evidence").join("adv_snapshot.json");
    std::fs::create_dir_all(snap.parent().expect("evidence dir")).expect("evidence dir 可写");
    std::fs::write(&snap, serde_json::to_string_pretty(&recs).expect("JSON 序列化"))
        .expect("快照写盘");
    eprintln!("[snapshot] {} 条对局快照 → {}", recs.len(), snap.display());
    recs
}

/// 默认回归：0 参数跑 3 seed × 4 AI 席 × 200 回合，断言快照自洽（可重复跑，0 副作用断言）。
#[test]
fn adversarial_snapshot_self_consistent() {
    let recs = dump(&[2026u32, 7, 11], 4, 200);
    assert_eq!(recs.len(), 3, "3 seed = 3 快照");
    for r in &recs {
        assert_eq!(r["seats"], 4, "席数字段对齐");
        let winner = r["winner"].as_i64();
        let alive: Vec<bool> = r["alive"]
            .as_array()
            .expect("alive 数组")
            .iter()
            .map(|v| v.as_bool().unwrap_or(false))
            .collect();
        let n = alive.len();
        assert_eq!(n, 4, "席位长度 = 席数");
        // 自洽：winner=Some ⇒ 恰 1 席存活；winner=None ⇒ 全破产或未收敛（alive 计数 ≥1 = 未收敛）。
        if let Some(w) = winner {
            assert_eq!(alive.iter().filter(|a| **a).count(), 1, "收敛对局恰 1 存活席");
            assert!(usize::try_from(w).is_ok_and(|w| w < n), "胜者席位 < 席数");
        }
        let golds: Vec<u32> = r["gold"]
            .as_array()
            .expect("gold 数组")
            .iter()
            .map(|v| u32::try_from(v.as_u64().unwrap_or(0)).unwrap_or(u32::MAX))
            .collect();
        assert_eq!(golds.len(), n, "金长度 = 席数");
    }
}

/// 显式导出（`adversarial_shot.py` 经环境变量 `TI_ADV_SEEDS`/`TI_ADV_SEATS`/`TI_ADV_ROUNDS` 触发；
/// 普通回归不带 env = no-op，默认面由 `adversarial_snapshot_self_consistent` 覆盖）。
///
/// 用 env 传参（`cargo test -- <args>` 被 libtest 吃掉不可靠，EXP：env 最稳）。
#[test]
fn adversarial_snapshot_explicit_export() {
    let seeds = std::env::var("TI_ADV_SEEDS").ok();
    if let Some(s) = seeds {
        let seats =
            std::env::var("TI_ADV_SEATS").ok().and_then(|v| v.parse::<u8>().ok()).unwrap_or(4);
        let rounds =
            std::env::var("TI_ADV_ROUNDS").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(400);
        let list: Vec<u32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        assert!(!list.is_empty(), "TI_ADV_SEEDS 非空");
        dump(&list, seats, rounds);
    }
}
