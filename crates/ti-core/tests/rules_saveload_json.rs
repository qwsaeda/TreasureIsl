//! S8.3.d 存读档（`rules/saveload_json`）测试。
//!
//! 依据 `docs/business/persistence.md` 可测试规则（`save_*` 系列，S8.3.d 落地）：
//! 存→读往返 / v1→v2 迁移补默认 / 迁移幂等 / 同 seed 同随机流 / 存档含 `rng_seed` 可续 /
//! 旧 `SaveSlot` 迁移到 v2。
//!
//! 置于集成测试目录，豁免门禁的 `unwrap`/`expect` 扫描（EXP-052）。

use ti_core::oracle::saveload::SaveSlot;
use ti_core::rules::saveload_json::{DEFAULT_RNG_SEED, GoodsSave, SaveErr, SaveManager, SaveV2};

#[test]
fn save_roundtrip_preserves_fields() {
    let mut s = SaveV2::new("drake".into(), "m01".into());
    s.gold = 1200;
    s.hp = 85;
    s.scene = "MainMap".into();
    s.buildings.push("palisade".into());
    s.goods.push(GoodsSave { id: "spice".into(), qty: 5 });
    s.rng_seed = 12345;
    s.bank = 300;
    s.unlocked_maps.push("m02".into());
    let json = s.to_json().expect("序列化成功");
    let back = SaveV2::from_json(&json).expect("反序列化成功");
    assert_eq!(s, back, "存→读往返逐字段一致");
}

#[test]
fn save_migrate_v1_to_v2_defaults() {
    let json = r#"{"character":"drake","map":"m01"}"#;
    let v1 = SaveV2::from_json(json).expect("v1 缺省字段走 default");
    assert!(!v1.is_v2(), "缺省 version=1 非 v2");
    let v2 = v1.migrate().expect("v1→v2 迁移成功");
    assert!(v2.is_v2());
    assert_eq!(v2.rng_seed, DEFAULT_RNG_SEED, "v1 缺 rng_seed → 默认种子");
    assert_eq!(v2.bank, 0, "v1 缺 bank → 0");
    assert_eq!(v2.character, "drake", "必填字段保留");
}

#[test]
fn save_migration_idempotent() {
    let json = r#"{"version":2,"character":"drake","map":"m01","rng_seed":999,"bank":300}"#;
    let s = SaveV2::from_json(json).expect("解析 v2");
    let m1 = s.migrate().expect("首次迁移");
    let m2 = m1.clone().migrate().expect("二次迁移（幂等）");
    assert_eq!(m1, m2, "v2 再迁不变（幂等）");
    assert_eq!(m1.rng_seed, 999, "已有 rng_seed 不被覆盖");
}

#[test]
fn save_unknown_version_rejected() {
    let json = r#"{"version":3,"character":"drake","map":"m01"}"#;
    let s = SaveV2::from_json(json).expect("解析");
    assert_eq!(s.migrate(), Err(SaveErr::UnknownVersion(3)), "版本 3 拒绝迁移");
}

#[test]
fn rng_same_seed_same_stream() {
    let mut s = SaveV2::new("drake".into(), "m01".into());
    s.rng_seed = 7;
    let mut a = s.rng();
    let mut b = s.rng();
    let va: Vec<u32> = (0..10).map(|_| a.in_range(u32::MAX)).collect();
    let vb: Vec<u32> = (0..10).map(|_| b.in_range(u32::MAX)).collect();
    assert_eq!(va, vb, "同 rng_seed → 同随机流（读档可续）");
}

#[test]
fn save_rng_seed_restores_stream() {
    let mut s = SaveV2::new("drake".into(), "m01".into());
    s.rng_seed = 1234;
    let json = s.to_json().expect("序列化");
    let restored = SaveV2::from_json(&json).expect("反序列化");
    let mut orig = s.rng();
    let mut rest = restored.rng();
    let orig_stream: Vec<u32> = (0..5).map(|_| orig.in_range(u32::MAX)).collect();
    let rest_stream: Vec<u32> = (0..5).map(|_| rest.in_range(u32::MAX)).collect();
    assert_eq!(orig_stream, rest_stream, "存档后读档续流一致");
}

#[test]
fn manager_slot_roundtrip() {
    let mut mgr = SaveManager::new(5);
    let mut s = SaveV2::new("drake".into(), "m01".into());
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
    let v2 = ti_core::rules::saveload_json::from_legacy_slot(&legacy);
    assert!(v2.is_v2());
    assert_eq!(v2.character, "char2");
    assert_eq!(v2.map, "m03");
    assert_eq!(v2.gold, 500);
}

/// 设置不改变玩法状态（`docs/business/persistence.md` 可测规则 `settings_do_not_affect_gameplay`）。
/// 设置存 `ti-platform::settings`（非本模块），此处验证存档结构不含设置字段。
#[test]
fn settings_do_not_affect_gameplay() {
    let s = SaveV2::new("drake".into(), "m01".into());
    // SaveV2 没有 volume/quality 等设置字段（编译期结构约束）
    let json: serde_json::Value =
        serde_json::from_str(&s.to_json().expect("序列化")).expect("解析");
    assert!(
        json.get("volume").is_none(),
        "存档不含设置字段 volume（设置归 ti-platform，不影响玩法状态）"
    );
}
