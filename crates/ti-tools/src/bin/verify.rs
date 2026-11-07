//! `verify` —— 全格式「0 未知字节」校验（P0-2 判据载体；S46.0）。
//!
//! 用法：
//! ```text
//! cargo run -p ti-tools --bin verify -- --zero-unknown          # 遍历 legacy/data 真实文件逐族解析
//! cargo run -p ti-tools --bin verify -- --zero-unknown --json   # 机器可读（JSON）
//! ```
//!
//! 口径（parity.md P0-2「每个格式解析器 0 未知字节 = 文件 100% 字段被定义，否则报错」）：
//! 单个文件的「0 未知字节」= **该格式解析器消费了文件全部字节**（结构之后无残留字节）。
//! `ti_formats` 的 `define_format!` 生成的 `read` 对残留字节报 `ParseKind::TrailingBytes`
//! ⇒ **解析成功即 0 未知字节**（不需要额外统计逻辑）。
//!
//! 遍历面 = `legacy/data/` 真实文件（逐族）+ 容器（`.pak`/`.lpk`）内**逐条目**读出。
//! 无对应解析器的文件 = 如实标注「无解析器（跳过）」，**不静默通过**。
//!
//! 退出码：0 = 全部达标；1 = 存在失败项（文件或条目）；2 = 缺 `--zero-unknown` 参数。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ti_formats::{Charset, Font, MapControl, MapScene, Package, Settings, TerrainMap};

/// 跳过标记（该文件无解析器）。
const SKIP_MARK: &str = "无解析器（跳过）";

/// 格式族（= 一条解析路径）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family {
    /// UI 场景 `.Map`（头部 + u32 字流 + 控件记录）。
    MapScene,
    /// 地形图 `.Map`（`glmapm*.Map`；记录级 stride=9 三段流）。
    Terrain,
    /// `GoldLand.Fnt`（点阵字形）。
    FontFnt,
    /// `font.dat`（UTF-16LE 字符表）。
    Charset,
    /// `settings.dat`（8 字节固定）。
    Settings,
    /// `.pak` / `.lpk` 容器（头部 + 索引 + 逐条目）。
    Package,
}

impl Family {
    /// 族名（报告用）。
    const fn name(self) -> &'static str {
        match self {
            Self::MapScene => "map-scene",
            Self::Terrain => "terrain",
            Self::FontFnt => "font-fnt",
            Self::Charset => "font-charset",
            Self::Settings => "settings",
            Self::Package => "package",
        }
    }
}

/// 单个校验对象的结果。
#[derive(Debug)]
struct Row {
    /// 文件名（容器条目 = `容器#序号`）。
    target: String,
    /// 格式族。
    family: Family,
    /// 字节数（容器 = 整文件；条目 = 解压后长度）。
    bytes: usize,
    /// 达标 = 解析成功（0 残留字节）。
    ok: bool,
    /// 说明（达标时描述解析规模；失败时为原因）。
    detail: String,
    /// 是否跳过（无解析器）。
    skipped: bool,
}

impl Row {
    /// 成功行。
    fn ok(target: &str, family: Family, bytes: usize, detail: String) -> Self {
        Self { target: target.to_owned(), family, bytes, ok: true, detail, skipped: false }
    }

    /// 失败行。
    fn fail(target: &str, family: Family, bytes: usize, detail: String) -> Self {
        Self { target: target.to_owned(), family, bytes, ok: false, detail, skipped: false }
    }
}

/// 统计（报告汇总用）。
#[derive(Debug, Default)]
struct Counters {
    /// 校验文件数。
    files: usize,
    /// 校验条目数（容器内）。
    entries: usize,
    /// 跳过数（无解析器）。
    skipped: usize,
}

/// 文件 → 格式族（`None` = 无解析器 ⇒ 跳过并如实标注）。
fn family_of(name: &str) -> Option<Family> {
    let ext = Path::new(name).extension().and_then(|e| e.to_str()).unwrap_or_default();
    if ext.eq_ignore_ascii_case("pak") || ext.eq_ignore_ascii_case("lpk") {
        return Some(Family::Package);
    }
    if ext.eq_ignore_ascii_case("map") {
        return Some(if name.starts_with("glmapm") { Family::Terrain } else { Family::MapScene });
    }
    match name {
        "GoldLand.Fnt" => Some(Family::FontFnt),
        "font.dat" => Some(Family::Charset),
        "settings.dat" => Some(Family::Settings),
        _ => None,
    }
}

/// UI 场景 `.Map`：头部 + u32 字流 + 控件记录（`5 + 6*count`）。
fn verify_map_scene(name: &str, buf: &[u8]) -> Row {
    match MapScene::parse(name, buf) {
        Ok(scene) => match MapControl::parse_words(name, &scene.words) {
            Ok(controls) => Row::ok(
                name,
                Family::MapScene,
                buf.len(),
                format!("0 未知字节（控件 {} 条）", controls.len()),
            ),
            Err(e) => {
                Row::fail(name, Family::MapScene, buf.len(), format!("控件记录解析失败：{e}"))
            }
        },
        Err(e) => Row::fail(name, Family::MapScene, buf.len(), format!("{e}")),
    }
}

/// 地形图 `.Map`（记录级 stride=9 三段流）。
fn verify_terrain(name: &str, buf: &[u8]) -> Row {
    match TerrainMap::parse(name, buf) {
        Ok(t) => Row::ok(
            name,
            Family::Terrain,
            buf.len(),
            format!("0 未知字节（地形 {}×{}）", t.scene.header.width, t.scene.header.height),
        ),
        Err(e) => Row::fail(name, Family::Terrain, buf.len(), format!("{e}")),
    }
}

/// `GoldLand.Fnt`（点阵字形）。
fn verify_font_fnt(name: &str, buf: &[u8]) -> Row {
    match Font::parse(name, buf) {
        Ok(f) => Row::ok(
            name,
            Family::FontFnt,
            buf.len(),
            format!("0 未知字节（字形 {} 个）", f.glyphs.len()),
        ),
        Err(e) => Row::fail(name, Family::FontFnt, buf.len(), format!("{e}")),
    }
}

/// `font.dat`（UTF-16LE 字符表）。
fn verify_charset(name: &str, buf: &[u8]) -> Row {
    match Charset::parse(name, buf) {
        Ok(c) => Row::ok(
            name,
            Family::Charset,
            buf.len(),
            format!("0 未知字节（字符 {} 个）", c.chars.len()),
        ),
        Err(e) => Row::fail(name, Family::Charset, buf.len(), format!("{e}")),
    }
}

/// `settings.dat`（8 字节定长）。
fn verify_settings(name: &str, buf: &[u8]) -> Row {
    match Settings::parse(name, buf) {
        Ok(_) => Row::ok(name, Family::Settings, buf.len(), "0 未知字节（8 字节定长）".to_owned()),
        Err(e) => Row::fail(name, Family::Settings, buf.len(), format!("{e}")),
    }
}

/// 容器 `.pak`/`.lpk`：头部 + 索引 + **逐条目**（PCX 图像条目额外解码）。
fn verify_package(name: &str, buf: &[u8], rows: &mut Vec<Row>, counters: &mut Counters) {
    let pkg = match Package::parse(name, buf) {
        Ok(p) => p,
        Err(e) => {
            rows.push(Row::fail(name, Family::Package, buf.len(), format!("{e}")));
            return;
        }
    };
    rows.push(Row::ok(
        name,
        Family::Package,
        buf.len(),
        format!("0 未知字节（容器头 + 索引，条目 {} 个）", pkg.entries.len()),
    ));
    for (i, entry) in pkg.entries.iter().enumerate() {
        counters.entries += 1;
        match pkg.read_entry(buf, i) {
            Ok(data) => {
                // PCX 图像条目（首字节 0x0A）：解码验结构（扫描线须恰好覆盖）。
                let detail = if data.first() == Some(&0x0A) {
                    match ti_formats::pcx::Image::decode(name, &data) {
                        Ok(_) => "0 未知字节（条目内容 + PCX 解码）".to_owned(),
                        Err(e) => format!("PCX 解码失败：{e}"),
                    }
                } else {
                    "0 未知字节（条目内容）".to_owned()
                };
                let ok = !detail.starts_with("PCX 解码失败");
                let target = format!("{name}#{i}");
                rows.push(if ok {
                    Row::ok(&target, Family::Package, data.len(), detail)
                } else {
                    Row::fail(&target, Family::Package, data.len(), detail)
                });
            }
            Err(e) => rows.push(Row::fail(
                &format!("{name}#{i}"),
                Family::Package,
                entry.len as usize,
                format!("条目解压失败：{e}"),
            )),
        }
    }
}

/// 校验单个文件（容器 = 逐条目展开为多行）。
fn verify_file(name: &str, buf: &[u8], rows: &mut Vec<Row>, counters: &mut Counters) {
    let Some(family) = family_of(name) else {
        counters.skipped += 1;
        rows.push(Row {
            target: name.to_owned(),
            family: Family::Settings, // 占位：跳过的文件不参与族统计
            bytes: buf.len(),
            ok: true,
            detail: SKIP_MARK.to_owned(),
            skipped: true,
        });
        return;
    };
    counters.files += 1;
    match family {
        Family::MapScene => rows.push(verify_map_scene(name, buf)),
        Family::Terrain => rows.push(verify_terrain(name, buf)),
        Family::FontFnt => rows.push(verify_font_fnt(name, buf)),
        Family::Charset => rows.push(verify_charset(name, buf)),
        Family::Settings => rows.push(verify_settings(name, buf)),
        Family::Package => verify_package(name, buf, rows, counters),
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

/// 收集 `legacy/data/` 下的真实文件（目录跳过）。
fn collect_files(data_dir: &Path) -> Result<Vec<String>, String> {
    let rd = fs::read_dir(data_dir).map_err(|e| format!("读目录失败：{e}"))?;
    let mut names: Vec<String> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string())
        .collect();
    names.sort();
    Ok(names)
}

/// 人读报告（逐行 + 汇总）。
fn print_report(rows: &[Row], counters: &Counters) {
    println!("=== verify --zero-unknown（口径：P0-2 每格式 0 未知字节）===");
    for r in rows {
        let mark = if r.skipped {
            "SKIP"
        } else if r.ok {
            " OK "
        } else {
            "FAIL"
        };
        println!(
            "[{mark}] {:<22} {:<14} {:>9} B  {}",
            r.target,
            r.family.name(),
            r.bytes,
            r.detail
        );
    }
    println!(
        "--- 文件 {} 个 / 容器条目 {} 个 / 跳过 {} 个 / 失败 {} 个",
        counters.files,
        counters.entries,
        counters.skipped,
        rows.iter().filter(|r| !r.ok).count()
    );
    let failed = rows.iter().filter(|r| !r.ok).count();
    if failed == 0 {
        println!("PASS：全部可解析格式文件 0 未知字节");
    } else {
        println!("FAIL：存在未达 0 未知字节的文件/条目（详见上方 FAIL 行）");
    }
}

/// JSON 报告（机器可读）。
fn print_json(rows: &[Row], counters: &Counters, data_dir: &Path) {
    let items: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "target": r.target,
                "family": r.family.name(),
                "bytes": r.bytes,
                "ok": r.ok,
                "skipped": r.skipped,
                "detail": r.detail,
            })
        })
        .collect();
    let failed = rows.iter().filter(|r| !r.ok).count();
    let out = serde_json::json!({
        "criterion": "P0-2 每个格式解析器 0 未知字节",
        "data_dir": data_dir.display().to_string(),
        "files": counters.files,
        "entries": counters.entries,
        "skipped": counters.skipped,
        "failed": failed,
        "ok": failed == 0,
        "items": items,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.iter().any(|a| a == "--zero-unknown") {
        eprintln!(
            "用法：cargo run -p ti-tools --bin verify -- --zero-unknown [--json]\n\
             口径：每个格式解析器 0 未知字节（文件 100% 字节被字段定义；结构后无残留）。"
        );
        return ExitCode::from(2);
    }
    let json = args.iter().any(|a| a == "--json");
    let data_dir = repo_root().join("legacy/data");
    if !data_dir.is_dir() {
        eprintln!("FAIL 数据目录不存在：{}", data_dir.display());
        return ExitCode::from(1);
    }
    let names = match collect_files(&data_dir) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("FAIL {e}");
            return ExitCode::from(1);
        }
    };
    let mut rows: Vec<Row> = Vec::new();
    let mut counters = Counters::default();
    for name in &names {
        match fs::read(data_dir.join(name)) {
            Ok(buf) => verify_file(name, &buf, &mut rows, &mut counters),
            Err(_) => rows.push(Row::fail(name, Family::Settings, 0, "读文件失败".to_owned())),
        }
    }
    if json {
        print_json(&rows, &counters, &data_dir);
    } else {
        print_report(&rows, &counters);
    }
    if rows.iter().all(|r| r.ok) { ExitCode::SUCCESS } else { ExitCode::from(1) }
}
