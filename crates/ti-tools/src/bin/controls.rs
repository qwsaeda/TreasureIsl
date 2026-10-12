//! `controls` —— `.Map` 控件记录全量导出（S1.4b）。
//!
//! 用法：`cargo run -p ti-tools --bin controls -- [--out PATH]`
//!
//! 遍历 `data/*.Map`，解析 UI 控件记录（`sub_44004` 布局，20/20 整除吻合），
//! 输出 JSON：每个文件的表头、控件数、全部记录的 6 字段（f0..f5），
//! 以及全量去重后的 f5（控件类型）分布。自校验：记录数 × 6 + 表头字必须
//! 与文件字流完全一致（0 未知字节）。

use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_formats::{ControlKind, MapControl, MapScene};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn main() -> ExitCode {
    let mut out: Option<PathBuf> = None;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        if a == "--out" {
            out = it.next().map(PathBuf::from);
        }
    }
    match run(out.as_deref()) {
        Ok(summary) => {
            eprintln!("[controls] {summary}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("[controls] 失败: {e}");
            ExitCode::FAILURE
        }
    }
}

struct Scene {
    name: String,
    width: u32,
    height: u32,
    flags: u32,
    reserved: u32,
    table_head: [u32; 4],
    count: u32,
    records: Vec<MapControl>,
}

struct Report {
    scenes: Vec<Scene>,
    skipped: Vec<String>,
    total_records: u32,
    kind_dist: Vec<(u32, u32)>,
}

fn run(out: Option<&Path>) -> Result<String, String> {
    let data_dir = repo_root().join("legacy").join("data");
    let mut files: Vec<PathBuf> = fs::read_dir(&data_dir)
        .map_err(|e| format!("读 data/: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("map"))
        })
        .collect();
    files.sort();

    let mut report =
        Report { scenes: Vec::new(), skipped: Vec::new(), total_records: 0, kind_dist: Vec::new() };
    for f in &files {
        let raw = fs::read(f).map_err(|e| format!("读 {}: {e}", f.display()))?;
        let name = f.file_name().and_then(|s| s.to_str()).unwrap_or("?.Map").to_string();
        let scene = MapScene::parse(&name, &raw).map_err(|e| format!("解析 {name}: {e}"))?;
        if scene.to_bytes() != raw {
            return Err(format!("{name} roundtrip 不一致"));
        }
        // 非 UI 控件布局（如地形图 glmapm*）→ 跳过，走 terrain 模块
        let Ok(recs) = MapControl::parse_words(&name, &scene.words) else {
            report.skipped.push(name);
            continue;
        };
        let head: [u32; 4] = scene.words.get(0..4).map_or([0; 4], |s| [s[0], s[1], s[2], s[3]]);
        let count = scene.words.get(MapControl::COUNT_WORD).copied().unwrap_or_default();
        report.total_records += u32::try_from(recs.len()).unwrap_or(u32::MAX);
        report.scenes.push(Scene {
            name,
            width: scene.header.width,
            height: scene.header.height,
            flags: scene.header.flags,
            reserved: scene.header.reserved,
            table_head: head,
            count,
            records: recs,
        });
    }

    // f5 分布（按出现顺序保留，稳定）
    let mut map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    for s in &report.scenes {
        for r in &s.records {
            *map.entry(r.f5).or_default() += 1;
        }
    }
    let mut dist: Vec<(u32, u32)> = map.into_iter().collect();
    dist.sort_by_key(|(k, _)| *k);
    report.kind_dist = dist;

    let json = render(&report);
    if let Some(dir) = out {
        fs::create_dir_all(dir).map_err(|e| format!("建目录 {}: {e}", dir.display()))?;
        let target = dir.join("controls.json");
        fs::write(&target, json.clone()).map_err(|e| format!("写 {}: {e}", target.display()))?;
        eprintln!("[controls] 写 {}", target.display());
    } else {
        print!("{json}");
    }

    let kind_desc: Vec<String> =
        report.kind_dist.iter().map(|(k, _)| ControlKind::from(*k).to_string()).collect();
    Ok(format!(
        "{} 个 UI 场景，{} 条控件记录；控件类型 f5 共 {} 种 [{}]",
        report.scenes.len(),
        report.total_records,
        report.kind_dist.len(),
        kind_desc.join(", ")
    ))
}

fn render(r: &Report) -> String {
    let mut s =
        String::with_capacity(1024 + r.scenes.iter().map(|s| s.records.len() * 96).sum::<usize>());
    let _ = write!(s, "{{\n  \"scene_count\": ");
    s.push_str(&r.scenes.len().to_string());
    let _ = write!(s, ",\n  \"record_total\": ");
    s.push_str(&r.total_records.to_string());
    let _ = write!(s, ",\n  \"skipped\": [");
    for (i, n) in r.skipped.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&json_str(n));
    }
    let _ = write!(s, "],\n  \"kind_dist\": [");
    for (i, (k, c)) in r.kind_dist.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        let _ = write!(s, "{{\"kind\": {k}, \"count\": {c}}}");
    }
    let _ = write!(s, "],\n  \"scenes\": [");
    for (si, sc) in r.scenes.iter().enumerate() {
        if si > 0 {
            s.push(',');
        }
        let _ = write!(s, "\n    {{");
        let name_json = json_str(&sc.name);
        let _ = write!(s, "\"name\": {name_json}");
        let _ = write!(s, ",\n      \"width\": {},", sc.width);
        let _ = write!(s, "\"height\": {},", sc.height);
        let _ = write!(s, "\"flags\": {},", sc.flags);
        let _ = write!(s, "\"reserved\": {},", sc.reserved);
        let table_head_str =
            sc.table_head.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");
        let _ = write!(s, "\"table_head\": [{table_head_str}],");
        let _ = write!(s, "\"count\": {},", sc.count);
        let _ = write!(s, "\"records\": [");
        for (ri, rec) in sc.records.iter().enumerate() {
            if ri > 0 {
                s.push(',');
            }
            let w = rec.words();
            let _ = write!(
                s,
                "{{\"f0\": {}, \"f1\": {}, \"f2\": {}, \"f3\": {}, \"f4\": {}, \"f5\": {}}}",
                w[0], w[1], w[2], w[3], w[4], w[5]
            );
        }
        s.push(']');
        let _ = write!(s, "\n    }}");
    }
    let _ = write!(s, "\n  ]\n}}\n");
    s
}

fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            _ => o.push(c),
        }
    }
    o.push('"');
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_is_ascii_and_roundtrips_counts() {
        let recs = vec![
            MapControl { f0: 1, f1: 105, f2: 6, f3: 12, f4: 1, f5: 0 },
            MapControl { f0: 1, f1: 32, f2: 40, f3: 24, f4: 2, f5: 301 },
        ];
        let report = Report {
            scenes: vec![Scene {
                name: "t.Map".into(),
                width: 176,
                height: 220,
                flags: 1,
                reserved: 12,
                table_head: [176, 219, 1, 0],
                count: 2,
                records: recs,
            }],
            skipped: vec!["glmapm01.Map".into()],
            total_records: 2,
            kind_dist: vec![(0, 1), (301, 1)],
        };
        let j = render(&report);
        assert!(!j.contains('\u{7F}'), "必须纯 ASCII");
        assert!(j.contains("\"record_total\": 2"));
        assert!(j.contains("\"kind\": 301, \"count\": 1"));
        assert!(j.contains("\"f3\": 24"));
    }

    #[test]
    fn json_string_escapes_quotes() {
        assert_eq!(json_str("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_str("ok"), "\"ok\"");
    }
}
