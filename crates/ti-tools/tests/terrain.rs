//! S1.8 验收：地形图 `glmapm0*.Map` 0 未知字节 + 往返逐字节 + 网格尺寸吻合。
//!
//! 区域块字段语义待 S1.8b。

use std::path::{Path, PathBuf};

use ti_formats::terrain::TerrainMap;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// (文件名, 网格宽, 网格高, 文件字节数)
const CASES: [(&str, u32, u32, usize); 3] = [
    ("glmapm01.Map", 20, 20, 17_148),
    ("glmapm02.Map", 20, 20, 18_948),
    ("glmapm03.Map", 15, 15, 8_976),
];

#[test]
fn all_terrain_maps_roundtrip_byte_identical() {
    for (name, gw, gh, size) in CASES {
        let raw = std::fs::read(repo_root().join("data").join(name)).expect("地形图存在");
        assert_eq!(raw.len(), size, "F-301：{name} 文件大小");
        let t = TerrainMap::parse(name, &raw).expect("解析成功");
        assert_eq!((t.grid_w, t.grid_h), (gw, gh), "{name} 网格尺寸不符");
        assert_eq!(t.cell_count(), u64::from(gw) * u64::from(gh));
        assert_eq!(t.region_count, if gw == 20 { 19 } else { 14 }, "{name} 区域数");
        assert_eq!(t.to_bytes(), raw, "{name} 往返必须逐字节一致");
        let j = t.to_json(name);
        assert!(j.contains(&format!("\"grid_w\": {gw}")), "{j}");
        eprintln!(
            "[s1.8] {name}: {gw}×{gh} = {} 格，{} 区域，往返无损",
            t.cell_count(),
            t.region_count
        );
    }
}

#[test]
fn terrain_maps_have_zero_unknown_bytes() {
    for (name, ..) in CASES {
        let raw = std::fs::read(repo_root().join("data").join(name)).expect("地形图存在");
        let t = TerrainMap::parse(name, &raw).expect("解析成功");
        // 4 字节头 + 全部 u32 字流 == 文件长度，无残留
        let consumed = 16 + t.scene.words.len() * 4;
        assert_eq!(consumed, raw.len(), "{name} 存在未消费字节（0 未知字节校验）");
        assert_eq!(raw.len() % 4, 0, "{name} 长度应为 4 的倍数");
    }
}

#[test]
fn ui_maps_are_not_treated_as_terrain() {
    // UI 场景 176×220 同样能被通用解析，但不能被误判为 20×20 地形
    let raw = std::fs::read(repo_root().join("data").join("main.Map")).expect("main.Map 存在");
    let t = TerrainMap::parse("main.Map", &raw).expect("解析成功");
    assert_eq!((t.grid_w, t.grid_h), (176, 220));
}
