//! S1.8b 验收：地形区域块字段语义 + 连线逻辑对齐。
//!
//! 验证项（对照 `data/glmapm0*.Map`）：
//! - cell 记录数 = `grid_w` × `grid_h` - 1；
//! - `f6` = 32 × (`row` + `col`)、`f7` = 16 × (`row` + `grid_w` - 1 - `col`)、
//!   `f8` 低 16 位 = `row` × `grid_w` + `col`；
//! - 缺格 = `grid_w` - 1（cell (0, `grid_w`-1) 无记录）；
//! - trailer `f5` = 连线记录数 + 1；
//! - 连线 `C` 升序、`M` ∈ {2, 4}；
//! - sentinel = `[0, 0, 0, 0, 0, 2]`；
//! - roundtrip 逐字节一致。

use std::fs;
use std::path::{Path, PathBuf};

use ti_formats::TerrainMap;
use ti_formats::terrain::{ISO_X_STEP, ISO_Y_STEP};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load(name: &str) -> Vec<u8> {
    let path = repo_root().join("legacy").join("data").join(name);
    fs::read(&path).expect("读取 data/glmapm0*.Map 失败")
}

struct MapInfo {
    name: &'static str,
    gw: u32,
    gh: u32,
    expected_cell_records: u64,
    expected_connections: usize,
}

const MAPS: &[MapInfo] = &[
    MapInfo {
        name: "glmapm01.Map",
        gw: 20,
        gh: 20,
        expected_cell_records: 399,
        expected_connections: 75,
    },
    MapInfo {
        name: "glmapm02.Map",
        gw: 20,
        gh: 20,
        expected_cell_records: 399,
        expected_connections: 125,
    },
    MapInfo {
        name: "glmapm03.Map",
        gw: 15,
        gh: 15,
        expected_cell_records: 224,
        expected_connections: 23,
    },
];

fn parse_map(info: &MapInfo) -> TerrainMap {
    let buf = load(info.name);
    TerrainMap::parse(info.name, &buf).expect("TerrainMap::parse 失败")
}

#[test]
fn header_fields_match_grid() {
    for info in MAPS {
        let t = parse_map(info);
        assert_eq!(t.grid_w, info.gw, "{} grid_w", info.name);
        assert_eq!(t.grid_h, info.gh, "{} grid_h", info.name);
        assert_eq!(t.region_count, info.gw - 1, "{} region_count = gw-1", info.name);
        assert_eq!(t.data_off, 32 * (info.gw - 1), "{} data_off = 32*count", info.name);
    }
}

#[test]
fn cell_record_count_matches_grid() {
    for info in MAPS {
        let t = parse_map(info);
        assert_eq!(
            t.cell_record_count(),
            info.expected_cell_records,
            "{} cell_record_count",
            info.name
        );
        assert_eq!(
            t.type1_record_count(),
            u64::from(info.gw) * u64::from(info.gh),
            "{} type1_record_count = gw*gh",
            info.name
        );
    }
}

#[test]
fn cell_fields_decode_iso_coords_and_cell_index() {
    for info in MAPS {
        let t = parse_map(info);
        let gw = info.gw;
        let cells = t.cell_records();
        assert_eq!(
            u64::try_from(cells.len()).unwrap_or(u64::MAX),
            info.expected_cell_records,
            "{} cell_records len",
            info.name
        );
        for rec in &cells {
            let row = rec.cell / gw;
            let col = rec.cell % gw;
            assert_eq!(
                rec.iso_x,
                ISO_X_STEP * (row + col),
                "{} cell {} iso_x=32*(r+c)",
                info.name,
                rec.cell
            );
            assert_eq!(
                rec.iso_y,
                ISO_Y_STEP * (row + gw - 1 - col),
                "{} cell {} iso_y=16*(r+gw-1-c)",
                info.name,
                rec.cell
            );
        }
    }
}

#[test]
fn missing_cell_is_gw_minus_1() {
    for info in MAPS {
        let t = parse_map(info);
        let cells = t.cell_records();
        let mut present = [false; 400];
        for rec in &cells {
            let idx = rec.cell as usize;
            if idx < present.len() {
                present[idx] = true;
            }
        }
        let missing = info.gw - 1;
        assert!(
            !present[missing as usize],
            "{} cell {} (0,{}) should be missing",
            info.name,
            missing,
            info.gw - 1
        );
        let total = info.gw * info.gh;
        for c in 0..total {
            if c != missing {
                assert!(present[c as usize], "{} cell {} should be present", info.name, c);
            }
        }
    }
}

#[test]
fn trailer_f5_equals_connection_count_plus_one() {
    for info in MAPS {
        let t = parse_map(info);
        let trailer = t.trailer().expect("trailer 记录缺失");
        let conn_count = t.connection_record_count();
        assert_eq!(
            trailer.tile,
            u32::try_from(conn_count).unwrap_or(u32::MAX) + 1,
            "{} trailer.f5 = conn_count + 1",
            info.name
        );
    }
}

#[test]
fn connection_records_count_matches() {
    for info in MAPS {
        let t = parse_map(info);
        assert_eq!(
            t.connection_record_count(),
            info.expected_connections,
            "{} connection_record_count",
            info.name
        );
        let recs = t.connection_records();
        assert_eq!(recs.len(), info.expected_connections, "{} connection_records len", info.name);
    }
}

#[test]
fn connection_c_is_ascending() {
    for info in MAPS {
        let t = parse_map(info);
        let recs = t.connection_records();
        for w in recs.windows(2) {
            assert!(
                w[0].y <= w[1].y,
                "{} connection C not ascending at y={} -> y={}",
                info.name,
                w[0].y,
                w[1].y
            );
        }
    }
}

#[test]
fn connection_marker_is_2_or_4() {
    for info in MAPS {
        let t = parse_map(info);
        let recs = t.connection_records();
        assert!(!recs.is_empty(), "{} has connections", info.name);
        for rec in &recs {
            assert!(
                rec.kind == 2 || rec.kind == 4,
                "{} connection kind {} not in {{2,4}}",
                info.name,
                rec.kind
            );
        }
    }
}

#[test]
fn connection_tile_in_mapinfo_range() {
    for info in MAPS {
        let t = parse_map(info);
        let recs = t.connection_records();
        for rec in &recs {
            assert!(rec.tile < 49, "{} conn tile {} < 49 (mapinfo filenum)", info.name, rec.tile);
        }
    }
}

#[test]
fn sentinel_is_present() {
    for info in MAPS {
        let t = parse_map(info);
        assert!(t.has_sentinel(), "{} sentinel [0,0,0,0,0,2]", info.name);
    }
}

#[test]
fn roundtrip_byte_identical() {
    for info in MAPS {
        let buf = load(info.name);
        let t = TerrainMap::parse(info.name, &buf).expect("TerrainMap::parse 失败");
        assert_eq!(t.to_bytes(), buf, "{} roundtrip", info.name);
    }
}

#[test]
fn link_fields_are_valid_cells_and_kind_codes() {
    // f0 = (u16 邻接目标 cell, u16 连线类别 1..=9)；f1 = (u16 区域锚点 cell, u16 区域码)
    // 实测：非零 f0.lo / f1.lo 全部落在合法 cell 区间内，f0.hi 恒为 1..=9。
    const MAX_LINK_KIND: u32 = 9;
    for info in MAPS {
        let t = parse_map(info);
        let total_cells = info.gw * info.gh;
        let cells = t.cell_records();
        let mut f0_links = 0u32;
        let mut f1_anchors = 0u32;
        let mut kinds = std::collections::BTreeSet::new();
        for rec in &cells {
            let f0_lo = rec.link_target & 0xFFFF;
            let f0_hi = rec.link_target >> 16;
            let f1_lo = rec.region_anchor & 0xFFFF;
            if f0_lo != 0 {
                assert!(
                    f0_lo < total_cells,
                    "{} f0.lo {} 不是合法 cell (total={})",
                    info.name,
                    f0_lo,
                    total_cells
                );
                // f0.lo != 0 时 f0.hi 才是连线类别码（1..=9）
                assert!(
                    (1..=MAX_LINK_KIND).contains(&f0_hi),
                    "{} f0.hi {} 不在 1..={}",
                    info.name,
                    f0_hi,
                    MAX_LINK_KIND
                );
                kinds.insert(f0_hi);
                f0_links += 1;
            }
            if f1_lo != 0 {
                assert!(
                    f1_lo < total_cells,
                    "{} f1.lo {} 不是合法 cell (total={})",
                    info.name,
                    f1_lo,
                    total_cells
                );
                f1_anchors += 1;
            }
        }
        assert!(f0_links > 0, "{} 应存在连线记录", info.name);
        assert!(f1_anchors > 0, "{} 应存在区域锚点记录", info.name);
        assert_eq!(
            kinds.len(),
            usize::try_from(MAX_LINK_KIND).unwrap_or(usize::MAX),
            "{} f0.hi 应覆盖全部 9 种类别",
            info.name
        );
    }
}

#[test]
fn link_target_is_nearby_cell() {
    // 实测：f0 目标格恒在同一列或相邻列（|dc| <= 1），且行号不增（dr <= 0）。
    for info in MAPS {
        let t = parse_map(info);
        let gw = info.gw;
        let cells = t.cell_records();
        for rec in &cells {
            let f0_lo = rec.link_target & 0xFFFF;
            if f0_lo == 0 {
                continue;
            }
            let dc = i64::from(f0_lo % gw) - i64::from(rec.cell % gw);
            let dr = i64::from(f0_lo / gw) - i64::from(rec.cell / gw);
            assert!(dc.abs() <= 1, "{} cell {} -> {} dc={} 超界", info.name, rec.cell, f0_lo, dc);
            assert!(dr <= 0, "{} cell {} -> {} dr={} 应不向下", info.name, rec.cell, f0_lo, dr);
        }
    }
}
