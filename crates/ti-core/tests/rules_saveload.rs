//! S8.3.d / S12.5 存读档（`rules/saveload`）测试。
//!
//! 依据 `docs/business/persistence.md` 可测规则：存→读往返 / v1→v2 迁移补默认 /
//! 迁移幂等 / 同 seed 同随机流 / 存档含 `rng_seed` 可续 / 旧 `SaveSlot` 迁移到 v2。
//! S12.5 新增：RNG 精确续流（`rng_steps`）+ 跨包状态桥（`SaveState`）。
//! 置于集成测试目录，豁免门禁的 `unwrap`/`expect` 扫描（EXP-052）。

use ti_core::domain::persistence::SaveSlot;
use ti_core::rules::rng::GameRng;
use ti_core::rules::saveload::{DEFAULT_RNG_SEED, SaveManager, SaveSlotV2, SaveState};

#[test]
fn save_v2_roundtrip_json() {
    let mut s = SaveSlotV2::new("drake".into(), "m01".into());
    s.gold = 1200;
    s.hp = 85;
    s.scene = "MainMap".into();
    s.buildings.push("palisade".into());
    s.goods.push(ti_core::rules::saveload::GoodsSave { id: "spice".into(), qty: 5 });
    s.rng_seed = 12345;
    s.rng_steps = 999;
    s.bank = 300;
    s.unlocked_maps.push("m02".into());
    let json = s.to_json();
    let back = SaveSlotV2::from_json(&json).expect("反序列化成功");
    assert_eq!(s, back, "存→读往返逐字段一致");
}

#[test]
fn migrate_v1_fills_defaults() {
    let json = r#"{"character":"drake","map":"m01"}"#;
    let v1 = SaveSlotV2::from_json(json).expect("v1 缺省字段走 default");
    assert!(!v1.is_v2(), "缺省 version=1 非 v2");
    let v2 = v1.migrate();
    assert!(v2.is_v2());
    assert_eq!(v2.rng_seed, DEFAULT_RNG_SEED, "v1 缺 rng_seed → 默认种子");
    assert_eq!(v2.bank, 0, "v1 缺 bank → 0");
    assert_eq!(v2.character, "drake", "必填字段保留");
}

#[test]
fn migration_idempotent() {
    let json = r#"{"version":2,"character":"drake","map":"m01","rng_seed":999,"bank":300}"#;
    let s = SaveSlotV2::from_json(json).expect("解析 v2");
    let m1 = s.clone().migrate();
    let m2 = m1.clone().migrate();
    assert_eq!(m1, m2, "v2 再迁不变（幂等）");
    assert_eq!(m1.rng_seed, 999, "已有 rng_seed 不被覆盖");
}

#[test]
fn rng_seed_restores_stream() {
    let mut s = SaveSlotV2::new("drake".into(), "m01".into());
    s.rng_seed = 7;
    let mut a = s.rng();
    let mut b = s.rng();
    let va: Vec<u32> = (0..10).map(|_| a.in_range(u32::MAX)).collect();
    let vb: Vec<u32> = (0..10).map(|_| b.in_range(u32::MAX)).collect();
    assert_eq!(va, vb, "同 rng_seed → 同随机流（读档可续）");
}

#[test]
fn manager_slot_roundtrip() {
    let mut mgr = SaveManager::new(5);
    let mut s = SaveSlotV2::new("drake".into(), "m01".into());
    s.gold = 999;
    mgr.set_slot(2, s.clone());
    assert_eq!(mgr.slot(2).expect("槽 2 存在"), &s, "写读一致");
    assert!(mgr.slot(5).is_none(), "越界 → None");
    let j = mgr.slot_json(2).expect("序列化");
    assert!(mgr.set_slot_json(3, &j).is_some(), "JSON 写入成功");
    assert_eq!(mgr.slot(3), Some(&s));
}

#[test]
fn manager_from_balance_slot_count() {
    let mgr = SaveManager::from_balance();
    assert_eq!(mgr.slot_count(), 5, "balance.save.slot_count = 5");
}

#[test]
fn legacy_slot_to_v2() {
    let legacy =
        SaveSlot { version: 1, character: 2, gold: 500, map_slot: 3, position: 777, rng_steps: 0 };
    let v2 = SaveSlotV2::from_legacy_slot(&legacy);
    assert!(v2.is_v2());
    assert_eq!(v2.character, "tichy", "legacy character=2（1 起）→ 角色 label tichy（Q010 修复）");
    assert_eq!(v2.map, "m03", "legacy map_slot=3（1 起）→ m03");
    assert_eq!(v2.gold, 500);
}

/// legacy 数字 0/越界保守钳位：`character` 0/9 → 首角色 `panda`；`map_slot` 0/9 → m01/m03（不 panic）。
#[test]
fn legacy_slot_out_of_range_clamps_conservative() {
    let legacy =
        SaveSlot { version: 1, character: 0, gold: 0, map_slot: 0, position: 0, rng_steps: 0 };
    let v2 = SaveSlotV2::from_legacy_slot(&legacy);
    assert_eq!(v2.character, "panda", "character=0（无角色）保守落首角色");
    assert_eq!(v2.map, "m01", "map_slot=0（无图）钳位 m01");
    let legacy =
        SaveSlot { version: 1, character: 9, gold: 0, map_slot: 9, position: 0, rng_steps: 0 };
    let v2 = SaveSlotV2::from_legacy_slot(&legacy);
    assert_eq!(v2.character, "panda", "character 越界保守落首角色（不 panic）");
    assert_eq!(v2.map, "m03", "map_slot 越界钳位 m03");
}

// ── S12.5 新增：RNG 精确续流 + 跨包状态桥 ─────────────────────────────

/// RNG 中间态精确续流（`docs/business/persistence.md` §RNG「RNG 状态进存档，读档后继续」）：
/// 取 N 次 `in_range` 后存 `rng_steps`，读档 `restore_state` 后续接的随机流与未存档前逐位一致。
#[test]
fn rng_mid_stream_restore_continues_exact() {
    let seed: u32 = 99;
    let mut g = GameRng::new(seed);
    for _ in 0..50 {
        let _ = g.in_range(u32::MAX);
    }
    let steps = g.steps_taken();
    assert_eq!(steps, 50, "50 次 in_range → steps_taken=50");
    let save_seed = g.save_state();
    assert_eq!(save_seed, seed, "save_state 返回原始 seed");

    let mut restored = GameRng::restore_state(save_seed, steps);
    let mut continued = g.clone();
    for _ in 0..20 {
        assert_eq!(
            restored.in_range(u32::MAX),
            continued.in_range(u32::MAX),
            "restore_state 后续接流与原 RNG 逐位一致"
        );
    }
}

/// 旧存档缺 `rng_steps` 字段 → `serde(default)` 补 0 = 种子重置语义（向后兼容）。
#[test]
fn legacy_save_without_rng_steps_defaults_to_zero() {
    let json = r#"{"version":2,"character":"drake","map":"m01","rng_seed":77}"#;
    let s = SaveSlotV2::from_json(json).expect("旧格式解析成功");
    assert_eq!(s.rng_steps, 0, "缺 rng_steps → 0（种子重置）");
    assert_eq!(s.rng_seed, 77, "rng_seed 保留");
    let mut a = s.rng();
    let mut b = GameRng::new(77);
    for _ in 0..10 {
        assert_eq!(a.in_range(u32::MAX), b.in_range(u32::MAX), "缺 rng_steps 等价 new(seed)");
    }
}

/// 跨包状态桥：P1–P4 玩法层状态（`domain` 公共类型）→ `SaveState` → `SaveSlotV2` → 还原，逐字段一致。
#[test]
fn cross_package_state_roundtrip() {
    use ti_core::domain::economy::Bank;
    use ti_core::domain::flow::SceneId;
    use ti_core::domain::map::GridPos;
    use ti_core::domain::selection::{Character, MapSlot};

    let st = SaveState {
        character: Some(Character::Drake),
        map_slot: Some(MapSlot::M02),
        position: GridPos::new(5, 7),
        bank: Bank::new(300),
        scene: Some(SceneId::MainMap),
    };
    let mut s = SaveSlotV2::new(String::new(), String::new());
    s.rng_seed = 42;
    s.rng_steps = 100;
    s.capture_state(&st);

    assert_eq!(s.character, "drake", "capture_state 写 character label");
    assert_eq!(s.map, "m02", "capture_state 写 map id");
    assert_eq!(s.position.row, 5, "capture_state 写 position.row");
    assert_eq!(s.position.col, 7, "capture_state 写 position.col");
    assert_eq!(s.bank, 300, "capture_state 写 bank.gold");
    assert_eq!(s.scene, "MainMap", "capture_state 写 scene 名");

    let back = s.restore_state();
    assert_eq!(back, st, "restore_state 往返逐字段一致");
}

/// `from_state` 一步完成「状态 + RNG → 存档」（`capture_state` + `rng_seed`/`rng_steps`）。
#[test]
fn from_state_builds_complete_save() {
    use ti_core::domain::economy::Bank;
    use ti_core::domain::flow::SceneId;
    use ti_core::domain::map::GridPos;
    use ti_core::domain::selection::{Character, MapSlot};

    let st = SaveState {
        character: Some(Character::Panda),
        map_slot: Some(MapSlot::M01),
        position: GridPos::new(3, 4),
        bank: Bank::new(500),
        scene: Some(SceneId::MainMap),
    };
    let s = SaveSlotV2::from_state(&st, 77, 200);
    assert_eq!(s.rng_seed, 77, "from_state 写 rng_seed");
    assert_eq!(s.rng_steps, 200, "from_state 写 rng_steps");
    assert_eq!(s.character, "panda", "from_state 写 character label");
    assert_eq!(s.map, "m01", "from_state 写 map id");
    let back = s.restore_state();
    assert_eq!(back, st, "from_state 构造的存档 restore_state 往返一致");
}

/// 未知角色/地图字符串 → `restore_state` 对应字段为 `None`（不 panic，门禁红线）。
#[test]
fn restore_state_unknown_ids_return_none() {
    let mut s = SaveSlotV2::new("unknown_char".into(), "m99".into());
    s.capture_state(&SaveState::default());
    let back = s.restore_state();
    assert!(back.character.is_none(), "未知角色字符串 → None");
    assert!(back.map_slot.is_none(), "未知地图字符串 → None");
}

/// `SaveManager::rng_for_slot` 使用 `restore_state` 精确续流（`rng_steps > 0` 时跳转）。
#[test]
fn manager_rng_for_slot_uses_restore_state() {
    let mut mgr = SaveManager::new(5);
    let mut s = SaveSlotV2::new("drake".into(), "m01".into());
    s.rng_seed = 55;
    s.rng_steps = 30;
    mgr.set_slot(0, s);

    let mut a = mgr.rng_for_slot(0);
    let mut b = GameRng::restore_state(55, 30);
    for _ in 0..10 {
        assert_eq!(a.in_range(u32::MAX), b.in_range(u32::MAX), "rng_for_slot 走 restore_state");
    }
}
