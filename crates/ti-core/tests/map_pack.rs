//! S31.2 地图包运行期读取（`docs/process/21_map_package_spec.md` §7 选 B）集成测试。
//!
//! 判据（台账 S31.2 dod）：包 manifest 合规校验 + 三段表直读 + 注入接缝
//! （`table_source` 首访前注入生效 / 迟注入忽略）+ §6 回退语义（缺包 = 内嵌默认）。

#![allow(clippy::panic)]
use std::path::Path;

use ti_core::data::map_pack;
use ti_core::data::table_source;

/// 仓库根（`crates/ti-core` 上两级）。
fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// 已装包根（`assets/maps/`；S31.1 产物）。
fn packs_root() -> std::path::PathBuf {
    repo_root().join("..").join("..").join("assets").join("maps")
}

#[test]
fn pack_manifest_rejects_bad_schema() {
    let err = map_pack::check_pack_manifest(
        r#"{ "schema": "map-pack/9", "id": "m01", "data_version": "board-v5" }"#,
        "m01",
    );
    assert!(matches!(err, Err(ti_core::data::LoadError::Schema(_))), "schema 不符须拒");
}

#[test]
fn pack_manifest_rejects_bad_data_version() {
    let err = map_pack::check_pack_manifest(
        r#"{ "schema": "map-pack/1", "id": "m01", "data_version": "board-v4" }"#,
        "m01",
    );
    assert!(matches!(err, Err(ti_core::data::LoadError::Schema(_))), "data_version 不符须拒");
}

#[test]
fn pack_manifest_rejects_id_mismatch() {
    let err = map_pack::check_pack_manifest(
        r#"{ "schema": "map-pack/1", "id": "m02", "data_version": "board-v5" }"#,
        "m01",
    );
    assert!(matches!(err, Err(ti_core::data::LoadError::Schema(_))), "id 与目录名不符须拒");
}

#[test]
fn installed_pack_reads_three_sections() {
    let root = packs_root();
    let Some(tables) = map_pack::read_pack_tables(&root, "m01") else {
        eprintln!("S31.1 已装包 m01 不可读（assets/maps/m01/ 未生成？先跑 scripts/map_pack.py）");
        return;
    };
    assert_eq!(tables.id, "m01");
    assert!(tables.board_json.is_some(), "m01 map.json 在位（S31.1 产物）");
    assert!(tables.scenes_json.is_some(), "m01 layout.json 在位（S31.1 产物）");
    assert!(tables.strings_json.is_some(), "m01 strings.json 在位（S31.1 产物）");
    // 三段顶层须为对象（precheck 口径；board/scenes 段 = 整表回注形状）
    if let Some(board) = &tables.board_json {
        assert!(
            map_pack::precheck_table(board, map_pack::table_key::BOARD),
            "board 段整表形状（{{\"maps\": {{…}}}}）顶层须为对象"
        );
    }
}

#[test]
fn missing_pack_falls_back_to_embedded() {
    let root = packs_root();
    assert!(
        map_pack::read_pack_tables(&root, "zz_missing").is_none(),
        "缺包 = None（§6 回退语义，不 panic）"
    );
    assert!(
        map_pack::read_pack_tables(Path::new("does/not/exist"), "m01").is_none(),
        "缺包根 = None（wasm 部署侧无包目录是常态）"
    );
}

#[test]
fn available_ids_and_default_pointer() {
    let root = packs_root();
    let ids = map_pack::available_pack_ids(&root);
    assert!(ids.iter().any(|id| id == "m01"), "S31.1 已装包 m01 须在册（实得 {ids:?}）");
    assert!(!ids.iter().any(|id| id == "_template"), "_template 骨架无 manifest，不入清单");
    let default = map_pack::default_pack_id(&root);
    assert!(default.is_some(), ".default 指针或首个已装包须可定位（S31.1 产物 .default=m01）");
}

/// 注入接缝：`install` 首访前生效（`table_text` 取注入文本）、迟注入忽略。
///
/// 独立进程（每个集成测试文件单独编译）⇒ 本文件内表访问仅发生在 `install` 之后，
/// 无「其他测试先首访」竞态（`table_source::PACK` 只被本文件消费）。
#[test]
fn install_before_first_access_wins() {
    use ti_core::data::table_source::table_text;

    let injected_board = r#"{ "maps": { "m99": { "grid_w": 20, "grid_h": 20, "spaces": [] } } }"#;
    let accepted = table_source::install(table_source::PackTables {
        source_id: Some("m99".to_owned()),
        board: Some(injected_board.to_owned()),
        scenes: None,
        strings: None,
    });
    assert!(accepted, "首次表访问前 install 须生效");
    assert_eq!(table_source::source_id(), Some("m99"));
    // 注入段 = 包文本；未注入段 = 内嵌默认文本（sentinel 不出现）。
    assert_eq!(
        table_text(map_pack::table_key::BOARD, "embedded-sentinel"),
        injected_board,
        "board 段须取注入文本"
    );
    assert_ne!(
        table_text(map_pack::table_key::SCENES, "embedded-sentinel"),
        injected_board,
        "未注入段须回退内嵌文本（sentinel 不出现）"
    );
    // 迟注入 = 忽略（`OnceLock` 已定型），保持 m99。
    let ignored = table_source::install(table_source::PackTables {
        source_id: Some("m88".to_owned()),
        ..Default::default()
    });
    assert!(!ignored, "迟注入（首访后）须忽略");
    assert_eq!(table_source::source_id(), Some("m99"), "迟注入不改已定型来源");
}
