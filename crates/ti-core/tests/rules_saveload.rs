//! S8.3.d 存读档（`rules/saveload`）测试。
//!
//! 依据 `docs/business/persistence.md` 可测规则：存→读往返 / v1→v2 迁移补默认 /
//! 迁移幂等 / 同 seed 同随机流 / 存档含 `rng_seed` 可续 / 旧 `SaveSlot` 迁移到 v2。
//! 置于集成测试目录，豁免门禁的 `unwrap`/`expect` 扫描（EXP-052）。

use ti_core::oracle::saveload::SaveSlot;
use ti_core::rules::saveload::{DEFAULT_RNG_SEED, SaveManager, SaveSlotV2};

#[test]
fn save_v2_roundtrip_json() {
    let mut s = SaveSlotV2::new("drake".into(), "m01".into());
    s.gold = 1200;
    s.hp = 85;
    s.scene = "MainMap".into();
    s.buildings.push("palisade".into());
    s.goods.push(ti_core::rules::saveload::GoodsSave { id: "spice".into(), qty: 5 });
    s.rng_seed = 12345;
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
    let legacy = SaveSlot { version: 1, character: 2, gold: 500, map_slot: 3, position: 777 };
    let v2 = SaveSlotV2::from_legacy_slot(&legacy);
    assert!(v2.is_v2());
    assert_eq!(v2.character, "char2");
    assert_eq!(v2.map, "m03");
    assert_eq!(v2.gold, 500);
}
