//! S15.1 存读档域业务场景穷举测试（`exhaust_saveload`）。
//!
//! 枚举空间全部由 `assets/data/exhaust_scenarios.json`（`ti_core::data::exhaust()`）驱动，
//! 本文件只做"加载 + 逐条断言"薄壳（加新组合 = 改 JSON，不改测试代码，
//! `02_conventions.md` §1 数值入 JSON 纪律）。
//!
//! 穷举空间：`balance.save.slot_count`（= 5）全槽 × JSON `saveload.seeds` ×
//! 业务字段（`gold`/`hp`/`bank` 各档，JSON 驱动）× 2 套实现（`rules::saveload_json::SaveV2` +
//! `rules::saveload::SaveSlotV2`）双轨交叉，每组合断言：
//! - 逐字段 roundtrip 相等（含 `rng_steps` 精确值）
//! - 二次迁移幂等（`migrate()` 恒等变换，全字段不变，`SaveSlotV2` 语义）
//! - `rng_for_slot` 续流后 `in_range` 序列与独立逐步回放 `GameRng::new(seed).next()`
//!   的期望序列逐位一致（RNG 精确续流，`advance` 跳跃 vs 逐步 `next` 两条路径交叉验证）
//! - `from_legacy_slot`（二进制 26 字节）→ `SaveSlotV2`/`SaveV2` 保守映射对 JSON 用例全量不 panic
//!   （旧档缺 `rng_steps`/`rng_seed` → 默认 0 / `DEFAULT_RNG_SEED`，`persistence.md` §迁移）

use ti_core::data::{balance, exhaust};
use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::rng::GameRng;
use ti_core::rules::saveload::{DEFAULT_RNG_SEED, SaveSlotV2, SaveState as V2SaveState};
use ti_core::rules::saveload_json::{SaveManager as V2JsonManager, SaveV2, from_legacy_slot};

fn real_slot_count() -> u32 {
    balance().expect("balance.json 有效").save.slot_count
}

/// `usize` → `u32` 安全转换（测试枚举空间规模远小于 `u32::MAX`，越界钳到 `u32::MAX`，不 panic）。
#[allow(clippy::cast_possible_truncation)]
fn usize_as_u32(v: usize) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// `SaveV2`（`saveload_json` 实现）全槽 × seed × 业务字段组合穷举（档位由 JSON 驱动）。
#[test]
fn exhaust_saveload_json_all_slots_seeds_fields() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let slot_count = real_slot_count() as usize;
    assert_eq!(slot_count, 5, "balance.save.slot_count = 5");

    let mut manager = V2JsonManager::new(slot_count);
    let mut combos: u32 = 0;

    for slot in 0..slot_count {
        for &seed in &es.saveload.seeds {
            for &gold in &es.saveload.gold_levels {
                for &hp in &es.saveload.hp_levels {
                    for &bank in &es.saveload.bank_levels {
                        let mut save = SaveV2::new("drake".into(), "m01".into());
                        save.rng_seed = seed;
                        save.rng_steps = 0;
                        save.gold = gold;
                        save.hp = hp;
                        save.bank = bank;

                        let json = save.to_json().expect("JSON 序列化应成功");
                        let restored = SaveV2::from_json(&json).expect("JSON 反序列化应成功");
                        assert_eq!(restored.version, 2, "roundtrip 后 version 恒 2");
                        assert_eq!(restored.rng_seed, seed, "rng_seed 逐字段 roundtrip");
                        // FIND-A8 修复：`rng_steps` 独立逐字段断言（不能只靠整体 `==`，
                        // 否则未来 JSON 序列化时该字段 key 名被误改，整体断言可能仍过但实际未 roundtrip）
                        assert_eq!(
                            restored.rng_steps, 0,
                            "rng_steps 逐字段 roundtrip（本用例恒 0）"
                        );
                        assert_eq!(restored.gold, gold, "gold 逐字段 roundtrip");
                        assert_eq!(restored.hp, hp, "hp 逐字段 roundtrip");
                        assert_eq!(restored.bank, bank, "bank 逐字段 roundtrip");

                        manager.set_slot(slot, restored.clone());
                        let read_back = manager.slot(slot).expect("set_slot 后必可读回");
                        assert_eq!(*read_back, restored, "SaveManager slot 存取 roundtrip");
                        combos += 1;
                    }
                }
            }
        }
    }
    let expected = usize_as_u32(
        slot_count
            * es.saveload.seeds.len()
            * es.saveload.gold_levels.len()
            * es.saveload.hp_levels.len()
            * es.saveload.bank_levels.len(),
    );
    assert_eq!(combos, expected, "组合数 = JSON 驱动的档位乘积");
}

/// RNG 精确续流穷举：全槽 × seed × `rng_steps` 档（JSON 驱动），
/// 逐步回放路径（`GameRng::new(seed)` 逐次 `next()` 消费 `steps` 次后取 5 个 `in_range`）
/// 与跳跃路径（`manager.rng_for_slot` → `restore_state` O(log n) 跳转后取 5 个 `in_range`）
/// 必须逐位一致（`06_flywheel` EXP-067 类"正逆成对往返"交叉验证，非自我循环）。
#[test]
fn exhaust_rng_resume_exact_continuation() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let slot_count = real_slot_count() as usize;
    let mut combos: u32 = 0;

    let mut manager = V2JsonManager::new(slot_count);
    for slot in 0..slot_count {
        for &seed in &es.saveload.seeds {
            for &steps in &es.saveload.rng_step_levels {
                let mut save = SaveV2::new("nemo".into(), "m02".into());
                save.rng_seed = seed;
                save.rng_steps = steps;
                manager.set_slot(slot, save);

                // 逐步回放路径（独立参照，非被测函数）：先逐次 next 消费 steps 次
                let mut expected_rng = GameRng::new(seed);
                for _ in 0..steps {
                    let _ = expected_rng.next();
                }
                let expected: Vec<u32> = (0..5).map(|_| expected_rng.in_range(1000)).collect();

                // 跳跃路径（被测 = saveload 模块续流接线）：经 `SaveManager::rng_for_slot`
                // 读槽内 `rng_seed`+`rng_steps` 调 `restore_state`（EXP-067 类往返交叉验证）
                let mut advance_rng = manager.rng_for_slot(slot);
                let actual: Vec<u32> = (0..5).map(|_| advance_rng.in_range(1000)).collect();
                assert_eq!(
                    actual, expected,
                    "RNG 续流违例：slot={slot} seed={seed} steps={steps}（跳跃 vs 逐步回放不一致）"
                );
                combos += 1;
            }
        }
    }
    let expected =
        usize_as_u32(slot_count * es.saveload.seeds.len() * es.saveload.rng_step_levels.len());
    assert_eq!(combos, expected, "组合数 = JSON 驱动的档位乘积");
}

/// `SaveSlotV2`（`saveload` 实现，语义更宽：`capture_state`/`restore_state` 跨包状态桥）
/// 全槽 × seed 穷举 + 迁移幂等（强断言：全字段恒等，非仅"两次结果互比"）。
#[test]
fn exhaust_saveload_v2_all_slots_migrate_idempotent() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let slot_count = real_slot_count() as usize;
    let mut combos: u32 = 0;

    for _slot in 0..slot_count {
        for &seed in &es.saveload.seeds {
            let mut v2 = SaveSlotV2::new("blackbeard".into(), "m03".into());
            v2.rng_seed = seed;
            v2.rng_steps = 3;
            v2.gold = 500;
            v2.hp = 120;
            v2.bank = 80;

            // 迁移幂等：v2 已是 version=2，`migrate()` 应为恒等变换
            let migrated_once = v2.clone().migrate();
            let migrated_twice = migrated_once.clone().migrate();
            assert_eq!(migrated_once, migrated_twice, "迁移幂等违例（二次迁移结果不一致）");
            // 强断言（FIND-2 修复）：迁移后全字段 == 原始已设值，而非仅"两次结果互比"
            // （若 migrate 未来被错误扩展为对 v2 也重置 bank/hp 等字段，弱断言仍过，此强断言会挂）
            assert_eq!(
                migrated_once, v2,
                "迁移 v2 恒等：全字段不变（bank/hp/gold/rng_steps 均未改）"
            );
            assert!(migrated_once.is_v2(), "迁移后须 is_v2");
            assert_eq!(migrated_once.rng_seed, seed, "迁移不改 rng_seed（已设值）");
            assert_eq!(migrated_once.bank, 80, "迁移不改 bank（已设值 80）");
            assert_eq!(migrated_once.hp, 120, "迁移不改 hp（已设值 120）");

            // 跨包状态桥 roundtrip（`capture_state` + `restore_state`，c1 契约纯函数）
            let state = V2SaveState {
                character: Some(Character::Panda),
                map_slot: Some(MapSlot::M01),
                ..Default::default()
            };
            let mut captured = SaveSlotV2::new("x".into(), "y".into());
            captured.capture_state(&state);
            let restored_state = captured.restore_state();
            assert_eq!(
                restored_state.character,
                Some(Character::Panda),
                "capture/restore 角色 roundtrip"
            );
            assert_eq!(
                restored_state.map_slot,
                Some(MapSlot::M01),
                "capture/restore 地图 roundtrip"
            );
            combos += 1;
        }
    }
    assert_eq!(
        combos,
        usize_as_u32(slot_count * es.saveload.seeds.len()),
        "组合数 = JSON 驱动的档位乘积"
    );
}

/// `from_legacy_slot`（二进制 26 字节 `domain::persistence::SaveSlot`）→ `SaveV2`/`SaveSlotV2`
/// 保守映射，全 5 槽 × JSON `saveload.legacy_slot_cases` 全量用例不 panic
/// （旧档缺 `rng_steps` → 默认 0，`rng_seed` 缺省 → `DEFAULT_RNG_SEED`（42），`persistence.md` §迁移）。
#[test]
fn exhaust_from_legacy_slot_all_slots_no_panic() {
    let es = exhaust().expect("exhaust_scenarios.json 有效");
    let slot_count = real_slot_count() as usize;
    let mut combos: u32 = 0;

    for _slot in 0..slot_count {
        for case in &es.saveload.legacy_slot_cases {
            let legacy = SaveSlot {
                version: case.version,
                character: case.character,
                gold: case.gold,
                map_slot: case.map_slot,
                position: case.position,
                rng_steps: case.rng_steps,
                unlocked_mask: 0,
                boat_level: 1,

                v4: None,
            };

            let v2_json = from_legacy_slot(&legacy);
            assert_eq!(
                v2_json.rng_seed, DEFAULT_RNG_SEED,
                "from_legacy_slot 缺 seed 须落默认种子（domain::persistence::SaveSlot 无 rng_seed 字段）"
            );
            assert_eq!(v2_json.rng_steps, case.rng_steps, "from_legacy_slot 保 rng_steps");
            assert_eq!(v2_json.version, 2, "from_legacy_slot 迁移后 version 恒 2");
            // Q010：迁移产物标签必须可回读（旧映射 char{N}/m0N-0 起 off-by-one 已修）
            assert!(
                ti_core::rules::saveload_json::character_from_label(&v2_json.character).is_some(),
                "from_legacy_slot 角色标签须可解析（case character={}）",
                case.character
            );
            assert!(
                ti_core::rules::saveload_json::map_slot_from_label(&v2_json.map).is_some(),
                "from_legacy_slot 地图标签须可解析（case map_slot={}）",
                case.map_slot
            );

            let v2 = SaveSlotV2::from_legacy_slot(&legacy);
            assert_eq!(
                v2.rng_seed, DEFAULT_RNG_SEED,
                "SaveSlotV2::from_legacy_slot 缺 seed 须落默认种子"
            );
            assert_eq!(v2.rng_steps, case.rng_steps, "SaveSlotV2 保 rng_steps");
            combos += 1;
        }
    }
    assert_eq!(
        combos,
        usize_as_u32(slot_count * es.saveload.legacy_slot_cases.len()),
        "组合数 = 5 槽 × JSON legacy 用例数"
    );
}
