//! S8.3.d / S12.5 存读档（`rules/saveload_json`）测试。
//!
//! 依据 `docs/business/persistence.md` 可测试规则（`save_*` 系列，S8.3.d 落地）：
//! 存→读往返 / v1→v2 迁移补默认 / 迁移幂等 / 同 seed 同随机流 / 存档含 `rng_seed` 可续 /
//! 旧 `SaveSlot` 迁移到 v2。
//!
//! S12.5 新增（`docs/business/persistence.md` §RNG 精确续流 + 跨包状态桥）：
//! - RNG 中间态续流（`restore_state` + `rng_steps`）；
//! - 跨包状态快照（`capture_state` / `restore_state`，`domain` 公共类型）；
//! - 旧存档缺 `rng_steps` 兼容（`serde(default)` 补 0 = 种子重置语义）。
//!
//! 置于集成测试目录，豁免门禁的 `unwrap`/`expect` 扫描（EXP-052）。

use ti_core::domain::persistence::SaveSlot;
use ti_core::domain::selection::{Character, MapSlot};
use ti_core::rules::rng::GameRng;
use ti_core::rules::saveload_json::{
    DEFAULT_RNG_SEED, GoodsSave, SaveErr, SaveManager, SaveState, SaveV2,
};

#[allow(unused_imports)]
use ti_core::rules::saveload_json as _;

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

// ── S12.5 新增：RNG 精确续流 ─────────────────────────────────────────────

/// RNG 中间态精确续流（`docs/business/persistence.md` §RNG「RNG 状态进存档，读档后继续」）：
/// 取 N 次 `in_range` 后存档 `rng_steps`，读档 `restore_state` 后续接的随机流与未存档前逐位一致。
#[test]
fn rng_mid_stream_restore_continues_exact() {
    let seed: u32 = 99;
    let mut g = GameRng::new(seed);
    // 取 50 次
    for _ in 0..50 {
        let _ = g.in_range(u32::MAX);
    }
    let steps = g.steps_taken();
    // 存档状态
    let save_seed = g.save_state();
    assert_eq!(save_seed, seed, "save_state 返回原始 seed");
    assert_eq!(steps, 50, "50 次 in_range → steps_taken=50");

    // 读档续流
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

/// `steps_taken` 随 `next`/`fill` 累积（`fill(buf.len())` 批量加，非逐次）。
#[test]
fn rng_steps_taken_accumulates() {
    let mut g = GameRng::new(1);
    assert_eq!(g.steps_taken(), 0, "新建 RNG steps_taken=0");
    for _ in 0..5 {
        let _ = g.in_range(10);
    }
    assert_eq!(g.steps_taken(), 5, "5 次 in_range → steps_taken=5");
    let mut buf = [0i32; 10];
    g.fill(&mut buf);
    assert_eq!(g.steps_taken(), 15, "fill(10) 批量 +10 → steps_taken=15");
}

/// 旧存档缺 `rng_steps` 字段 → `serde(default)` 补 0 = 种子重置语义（`rng_seed` 单字段兼容）。
#[test]
fn legacy_save_without_rng_steps_defaults_to_zero() {
    // 旧格式 JSON（无 rng_steps 字段）
    let json = r#"{"version":2,"character":"drake","map":"m01","rng_seed":77}"#;
    let s = SaveV2::from_json(json).expect("旧格式解析成功");
    assert_eq!(s.rng_steps, 0, "缺 rng_steps → default 补 0 = 种子重置");
    assert_eq!(s.rng_seed, 77, "rng_seed 保留");
    // rng() 应等价于 GameRng::new(77)（steps=0 不 advance）
    let mut a = s.rng();
    let mut b = GameRng::new(77);
    for _ in 0..10 {
        assert_eq!(a.in_range(u32::MAX), b.in_range(u32::MAX), "缺 rng_steps 等价 new(seed)");
    }
}

/// `SaveV2` JSON 往返保留 `rng_steps` 字段（序列化/反序列化不丢）。
#[test]
fn save_roundtrip_preserves_rng_steps() {
    let mut s = SaveV2::new("drake".into(), "m01".into());
    s.rng_seed = 55;
    s.rng_steps = 12345;
    let json = s.to_json().expect("序列化");
    let back = SaveV2::from_json(&json).expect("反序列化");
    assert_eq!(back.rng_seed, 55, "rng_seed 保留");
    assert_eq!(back.rng_steps, 12_345, "rng_steps 保留");
}

// ── S12.5 新增：跨包状态桥（`domain` 公共类型）─────────────────────────────

/// `capture_state` / `restore_state` 往返：P1–P4 玩法层状态 → 存档 → 还原，逐字段一致。
#[test]
fn cross_package_state_roundtrip() {
    let st = SaveState {
        character: Some(Character::Drake),
        map_slot: Some(MapSlot::M02),
        position: ti_core::domain::map::GridPos::new(5, 7),
        bank: ti_core::domain::economy::Bank::new(300),
        scene: Some(ti_core::domain::flow::SceneId::MainMap),
    };
    let mut s = SaveV2::new(String::new(), String::new());
    s.rng_seed = 42;
    s.rng_steps = 100;
    s.capture_state(&st);

    // 存档字段
    assert_eq!(s.character, "drake", "capture_state 写 character label");
    assert_eq!(s.map, "m02", "capture_state 写 map id");
    assert_eq!(s.position.row, 5, "capture_state 写 position.row");
    assert_eq!(s.position.col, 7, "capture_state 写 position.col");
    assert_eq!(s.bank, 300, "capture_state 写 bank.gold");
    assert_eq!(s.scene, "MainMap", "capture_state 写 scene 名");

    // 还原
    let back = s.restore_state();
    assert_eq!(back, st, "restore_state 往返逐字段一致（domain 公共类型）");
}

/// `capture_state` 未知 `character` 字符串 → `restore_state().character == None`（不 panic）。
#[test]
fn restore_state_unknown_character_returns_none() {
    let mut s = SaveV2::new("unknown_char".into(), "m01".into());
    s.capture_state(&SaveState::default());
    let back = s.restore_state();
    assert!(back.character.is_none(), "未知角色字符串 → None（门禁红线不 panic）");
}

/// `capture_state` 未知 `map` 字符串 → `restore_state().map_slot == None`（不 panic）。
#[test]
fn restore_state_unknown_map_returns_none() {
    let mut s = SaveV2::new("drake".into(), "m99".into());
    s.capture_state(&SaveState::default());
    let back = s.restore_state();
    assert!(back.map_slot.is_none(), "未知地图字符串 → None（门禁红线不 panic）");
}

/// `from_state` 一步完成「状态 + RNG → 存档」（`capture_state` + `rng_seed`/`rng_steps`）。
#[test]
fn from_state_builds_complete_save() {
    let st = SaveState {
        character: Some(Character::Panda),
        map_slot: Some(MapSlot::M01),
        position: ti_core::domain::map::GridPos::new(3, 4),
        bank: ti_core::domain::economy::Bank::new(500),
        scene: Some(ti_core::domain::flow::SceneId::MainMap),
    };
    let s = SaveV2::from_state(&st, 77, 200);
    assert_eq!(s.rng_seed, 77, "from_state 写 rng_seed");
    assert_eq!(s.rng_steps, 200, "from_state 写 rng_steps");
    assert_eq!(s.character, "panda", "from_state 写 character label");
    assert_eq!(s.map, "m01", "from_state 写 map id");
    let back = s.restore_state();
    assert_eq!(back, st, "from_state 构造的存档 restore_state 往返一致");
}

/// `character_from_label` / `map_slot_from_label` 查表对称性（`label()` 输出可查回）。
#[test]
fn character_and_mapslot_from_label_symmetric() {
    use ti_core::rules::saveload_json::{character_from_label, map_slot_from_label};
    for c in Character::all() {
        assert_eq!(character_from_label(c.label()), Some(c), "Character 查表往返");
    }
    for m in MapSlot::all() {
        let label = format!("m{:02}", m.index());
        assert_eq!(map_slot_from_label(&label), Some(m), "MapSlot 查表往返");
    }
    assert!(character_from_label("nonexistent").is_none(), "未知字符 → None");
    assert!(map_slot_from_label("m99").is_none(), "未知地图 → None");
}
