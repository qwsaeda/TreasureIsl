//! 地图包运行期读取层（S31.2，`docs/process/21_map_package_spec.md` §7 选 B 契约）：
//! 把「一张图 = 一个包」的运行期数据源从编译期内嵌（`include_str!`）解耦——
//! 壳层（ti-engine）启动时选定一个已装包 id，本模块从包路径直读三段表
//! （`map.json` → `board.json` / `layout.json` → `scenes.json` / `strings.json`），
//! 交 [`super::table_source`] 注入；缺包 / 未注入 = 回退内嵌默认包数据
//! （§6 失败回退语义，0 行为变化）。
//!
//! 设计约束（`docs/process/02_conventions.md`）：
//! - 本模块**零游戏语义**（只读包文件 + 校验 manifest schema/版本，不含规则判定）；
//! - 缺段 = 该段回退内嵌（`None`），不 panic、不整进程失败。
//!
//! 包文件契约（spec §1/§7，`scripts/map_pack.py` 产物）：
//! - `manifest.json`：`schema`/`id`/`data_version` 合规性（不符即拒整包）；
//! - `map.json`：`{grid_w, grid_h, spaces[]}`（= `board.json` 的 `maps.<id>` 段形状）；
//! - `layout.json`：`main_map` 场景布局参数（= `scenes.json` `main_map` 段形状）；
//! - `strings.json`：`string → string` 键值表（= `strings.json` 顶层形状）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{LoadError, SCHEMA_VERSION_KEY};

/// 地图包文件 IO 抽象（S31.2b：跨平台读包面）。
///
/// 背景：`map_pack` 原实现直读 `std::fs`（`read_dir`/`read_to_string`/`is_file`），
/// wasm 浏览器里无文件系统 ⇒ 读包静默失败 ⇒ 运行期换图在浏览器端失效。
/// 本 trait 把 IO 收敛为两个方法，`std::fs` 实现（native）与 `fetch` 实现（wasm）
/// 各注册一份；合并逻辑（[`merge_board_upsert`] 等）与校验逻辑（[`check_pack_manifest`]）
/// 保持纯 std 不变（0 平台分叉）。
pub trait PackReader {
    /// 包根下已装包 id 清单（`manifest.json` 在位的目录名；字典序；缺目录 = 空清单）。
    fn list_packs(&self) -> Vec<String>;
    /// 读包 `<id>/<file>` 的 UTF-8 文本；缺文件 = `None`（调用方按段回退内嵌）。
    fn read_file(&self, map_id: &str, file: &str) -> Option<String>;
}

/// `std::fs` 后端（native / android）：包根 = 本地文件系统目录。
pub struct FsPackReader {
    root: PathBuf,
}

impl FsPackReader {
    /// 以包根目录构造（`assets/maps` 或 `TI_MAP_ROOT` 覆盖值）。
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
}

impl PackReader for FsPackReader {
    fn list_packs(&self) -> Vec<String> {
        let Ok(entries) = std::fs::read_dir(&self.root) else {
            return Vec::new();
        };
        entries
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|id| self.root.join(id).join("manifest.json").is_file())
            .collect()
    }

    fn read_file(&self, map_id: &str, file: &str) -> Option<String> {
        let path = self.root.join(map_id).join(file);
        if path.is_file() { std::fs::read_to_string(&path).ok() } else { None }
    }
}

/// 平台默认 reader（native / android = [`FsPackReader`]；包根 = [`pack_root`]）。
///
/// wasm 侧不经此函数（fetch 实现在 ti-engine 壳层，`apply_map_pack` 按 `cfg` 分发，
/// 见 `docs/process/21_map_package_spec.md` §7）。
pub fn default_reader() -> &'static dyn PackReader {
    use std::sync::OnceLock;
    static READER: OnceLock<FsPackReader> = OnceLock::new();
    READER.get_or_init(|| FsPackReader::new(pack_root()))
}

/// 地图包 schema 标识（spec §6 版本兼容；`manifest.json` `schema` 字段须等值，不符即拒）。
pub const MAP_PACK_SCHEMA: &str = "map-pack/1";
/// 地图包数据版本（spec §6；`manifest.json` `data_version` 须 = `board-v5`，不符即拒）。
pub const MAP_PACK_DATA_VERSION: &str = "board-v5";

/// 一个地图包运行期直读的三段表（spec §7）。
///
/// 值为 **原始 JSON 文本**（非解析结果）：注入 [`super::table_source`] 后由既有
/// 访问器统一解析（0 双解析、0 双份结构体），本模块保持 0 游戏语义。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MapPackTables {
    /// 包 id（如 `m01`；= 目录名，须与 `manifest.id` 一致，见 [`check_pack_manifest`]）。
    pub id: String,
    /// `board.json` 注入文本（包内 `map.json` 回注 `maps.<id>`；`None` = 该段走内嵌）。
    pub board_json: Option<String>,
    /// `scenes.json` 注入文本（包内 `layout.json` 回注 `main_map`；`None` = 走内嵌）。
    pub scenes_json: Option<String>,
    /// `strings.json` 注入文本（包内 `strings.json` 键 upsert；`None` = 走内嵌）。
    pub strings_json: Option<String>,
}

/// 地图包 `manifest.json` 读形状（只取运行期必需键；多余键忽略，前向兼容）。
#[derive(Debug, Deserialize)]
pub struct RawPackManifest {
    /// schema 标识（须 = [`MAP_PACK_SCHEMA`]，见 [`check_pack_manifest`]）。
    schema: String,
    /// 包 id（须与目录名一致）。
    id: String,
    /// 数据版本（须 = [`MAP_PACK_DATA_VERSION`]）。
    data_version: String,
}

/// 校验 `manifest.json` 文本（spec §6 版本兼容 + I1~I8 的 schema 面）。
///
/// # Errors
/// JSON 语法错 / schema / `data_version` / id 不符时返回 [`LoadError`]（**拒绝该包**，
/// 调用方回退内嵌默认包，§6 回退语义）。
pub fn check_pack_manifest(raw: &str, map_id: &str) -> Result<RawPackManifest, LoadError> {
    let manifest: RawPackManifest = serde_json::from_str(raw)
        .map_err(|e| LoadError::Syntax(format!("manifest.json 解析失败: {e}")))?;
    if manifest.schema != MAP_PACK_SCHEMA {
        return Err(LoadError::Schema(format!(
            "manifest.schema = `{}` ≠ `{MAP_PACK_SCHEMA}`（拒绝加载）",
            manifest.schema
        )));
    }
    if manifest.data_version != MAP_PACK_DATA_VERSION {
        return Err(LoadError::Schema(format!(
            "manifest.data_version = `{}` ≠ `{MAP_PACK_DATA_VERSION}`（拒绝加载）",
            manifest.data_version
        )));
    }
    if manifest.id != map_id {
        return Err(LoadError::Schema(format!(
            "manifest.id = `{}` ≠ 目录名 `{map_id}`（拒绝加载）",
            manifest.id
        )));
    }
    Ok(manifest)
}
/// 包根目录的已装包 id 清单（`_template` 等缺 manifest 的目录跳过；字典序）。
///
/// 目录不存在 = 空清单（回退内嵌默认包，不报错——wasm 部署侧无包目录是常态）。
/// 平台差异：native / android = [`default_reader`]（`std::fs`）；wasm 侧不经此函数
/// （fetch 实现在 ti-engine 壳层，`docs/process/21_map_package_spec.md` §7）。
#[must_use]
pub fn available_pack_ids(root: &Path) -> Vec<String> {
    available_pack_ids_with(&FsPackReader::new(root))
}

/// 已装包 id 清单（reader 版；签名同 [`available_pack_ids`]，IO 走 [`PackReader`]）。
#[must_use]
pub fn available_pack_ids_with(reader: &dyn PackReader) -> Vec<String> {
    reader
        .list_packs()
        .into_iter()
        .filter(|id| reader.read_file(id, "manifest.json").is_some())
        .collect()
}

/// 默认包 id（`<root>/.default` 指针文件；缺 = 首个已装包；无已装包 = `None`）。
///
/// 平台差异：native / android = [`default_reader`] 直读；wasm = 浏览器无本地文件系统，
/// 读包走 ti-engine 壳层 fetch reader（`docs/process/21_map_package_spec.md` §7），
/// 本函数在 wasm 侧返回 `None`（缺包 = 内嵌默认包，§6 回退，不打日志）。
///
/// `root` 参数保留 = native 侧显式指定包根（`FsPackReader::new(root)`），
/// 与 `TI_MAP_ROOT` 环境变量语义一致（壳层 `pack_root()` 解析后传入）。
#[must_use]
pub fn default_pack_id(root: &Path) -> Option<String> {
    default_pack_id_with(&FsPackReader::new(root))
}

/// 默认包 id（reader 版；签名同 [`default_pack_id`]，IO 走 [`PackReader`]）。
///
/// 根级文件（`.default` 指针）约定 `map_id` 传空串。
#[must_use]
pub fn default_pack_id_with(reader: &dyn PackReader) -> Option<String> {
    let raw = reader.read_file("", ".default")?;
    let want = raw.trim().to_owned();
    if !want.is_empty() && reader.read_file(&want, "manifest.json").is_some() {
        return Some(want);
    }
    available_pack_ids_with(reader).into_iter().next()
}

/// 内存后端（纯 std，零平台依赖）：已预取好的包文件表 → [`PackReader`]。
///
/// wasm 侧用法（`docs/process/21_map_package_spec.md` §7）：壳层先经浏览器 fetch
/// 预取包文件进 `BTreeMap`，再用本后端驱动 [`read_pack_tables_with`] /
/// [`available_pack_ids_with`] / [`default_pack_id_with`]（0 新依赖、可单测）。
#[derive(Debug, Default, Clone)]
pub struct MemoryPackReader {
    /// 键 = `<map_id>/<file>`；`map_id` 空串 = 包根级文件（`.default`）。
    files: BTreeMap<String, String>,
    /// 已装包 id 清单（`list_packs` 直接返回；缺 manifest 的 id 由调用方 `read_file` 过滤）。
    pack_ids: Vec<String>,
}

impl MemoryPackReader {
    /// 以预取文件表 + 已装包清单构造（`pack_ids` 须字典序，同 [`PackReader::list_packs`] 口径）。
    #[must_use]
    pub fn new(files: BTreeMap<String, String>, pack_ids: Vec<String>) -> Self {
        Self { files, pack_ids }
    }

    /// 文件表（`<map_id>/<file>` → 文本；测试断言 / 预取结果导出用）。
    #[must_use]
    pub fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }

    /// 本 reader 承载的选定包 id（`files` 键首段去重；缺 = `None`，§6 回退）。
    #[must_use]
    pub fn pack_id(&self) -> Option<String> {
        self.files.keys().find_map(|k| k.split_once('/').map(|(id, _)| id.to_owned()))
    }
}

impl PackReader for MemoryPackReader {
    fn list_packs(&self) -> Vec<String> {
        self.pack_ids.clone()
    }

    fn read_file(&self, map_id: &str, file: &str) -> Option<String> {
        let key = if map_id.is_empty() { file.to_owned() } else { format!("{map_id}/{file}") };
        self.files.get(&key).cloned()
    }
}

/// 从包路径直读三段表（spec §7 运行期加载契约）。
///
/// 包根 = `root/<id>/`，须含 `manifest.json`（合规）；`map.json`/`layout.json`/
/// `strings.json` 各自可选（缺段 = 该段 `None`，`table_source` 按段回退内嵌）。
/// IO 走 [`FsPackReader`]（native = `std::fs`）；wasm 侧由 ti-engine 壳层 fetch reader
/// 驱动 [`read_pack_tables_with`]（`docs/process/21_map_package_spec.md` §7）。
///
/// 注入文本为**内嵌默认表 upsert 包段**（整表形状；保留其余内嵌图/场景/文案，
/// 只替换本包自己的段）：
/// - `board_json` = 内嵌 `board.json` 的 `maps.<id>` 被包 `map.json` upsert；
/// - `scenes_json` = 内嵌 `scenes.json` 的 `main_map` 场景键被包 `layout.json` upsert；
/// - `strings_json` = 内嵌 `strings.json` 被包 `strings.json` 键 upsert。
///
/// # 返回
/// `Some(tables)` = 包可读（manifest 合规）；`None` = 缺包 / manifest 不合规
/// （调用方回退内嵌默认包，§6；不 panic）。
#[must_use]
pub fn read_pack_tables(root: &Path, map_id: &str) -> Option<MapPackTables> {
    read_pack_tables_with(&FsPackReader::new(root), map_id)
}

/// 从包路径直读三段表（reader 版；签名同 [`read_pack_tables`]，IO 走 [`PackReader`]）。
#[must_use]
pub fn read_pack_tables_with(reader: &dyn PackReader, map_id: &str) -> Option<MapPackTables> {
    let manifest_raw = reader.read_file(map_id, "manifest.json")?;
    if check_pack_manifest(&manifest_raw, map_id).is_err() {
        eprintln!("[map-pack] {map_id}: manifest 校验失败（schema/版本/id）⇒ 回退内嵌默认包");
        return None;
    }
    // 注入文本 = **内嵌默认表 upsert 包段**（整表形状；保留其余内嵌图/场景/文案，
    // 只替换本包自己的段——否则注入 m02 会丢 m01/m03 内嵌图，默认选 m01 时硬错误）。
    let board_json = merge_board_upsert(reader.read_file(map_id, "map.json").as_deref(), map_id);
    let scenes_json = merge_scenes_upsert(reader.read_file(map_id, "layout.json").as_deref());
    let strings_json = merge_strings_upsert(reader.read_file(map_id, "strings.json").as_deref());
    Some(MapPackTables { id: map_id.to_owned(), board_json, scenes_json, strings_json })
}

/// 包 `map.json` upsert 进内嵌 `board.json` 的 `maps.<id>`（缺段 = `None` 走内嵌）。
fn merge_board_upsert(map_raw: Option<&str>, map_id: &str) -> Option<String> {
    let map_value: serde_json::Value = serde_json::from_str(map_raw?).ok()?;
    let mut board: serde_json::Value = serde_json::from_str(super::BOARD_JSON)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::default()));
    if let Some(maps) = board.get_mut("maps").and_then(serde_json::Value::as_object_mut) {
        maps.insert(map_id.to_owned(), map_value);
    }
    serde_json::to_string(&board).ok()
}

/// 包 `layout.json` upsert 进内嵌 `scenes.json` 的 `main_map` 场景键（缺段 = `None`）。
fn merge_scenes_upsert(layout_raw: Option<&str>) -> Option<String> {
    let layout_value: serde_json::Value = serde_json::from_str(layout_raw?).ok()?;
    let mut scenes: serde_json::Value = serde_json::from_str(super::SCENES_JSON)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::default()));
    let layout_keys: BTreeMap<String, serde_json::Value> = match layout_value {
        serde_json::Value::Object(obj) => obj.into_iter().collect(),
        _ => BTreeMap::new(),
    };
    if let Some(main_map) = scenes.get_mut("main_map").and_then(serde_json::Value::as_object_mut) {
        for (k, v) in layout_keys {
            main_map.insert(k, v);
        }
    }
    serde_json::to_string(&scenes).ok()
}

/// 包 `strings.json` 键 upsert 进内嵌 `strings.json`（缺段 = `None`）。
fn merge_strings_upsert(strings_raw: Option<&str>) -> Option<String> {
    let extra: BTreeMap<String, String> = serde_json::from_str(strings_raw?).ok()?;
    let mut strings: serde_json::Value = serde_json::from_str(super::STRINGS_JSON)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::default()));
    if let Some(map) = strings.as_object_mut() {
        for (k, v) in extra {
            map.insert(k, serde_json::Value::String(v));
        }
    }
    serde_json::to_string(&strings).ok()
}

/// 包根目录（`TI_MAP_ROOT` 环境变量 > `assets/maps`，壳层相对 CWD 跑；0 硬编码绝对路径）。
///
/// 返回**候选目录**供壳层判定在位（`available_pack_ids` 空 = 走内嵌，§6 回退）。
#[must_use]
pub fn pack_root() -> PathBuf {
    if let Ok(root) = std::env::var("TI_MAP_ROOT") {
        return PathBuf::from(root);
    }
    PathBuf::from("assets/maps")
}

/// 表段键名（[`super::table_source`] 注入 / 读取用，避免裸字符串散落）。
pub mod table_key {
    /// `board.json` 段。
    pub const BOARD: &str = "board";
    /// `scenes.json` 段。
    pub const SCENES: &str = "scenes";
    /// `strings.json` 段。
    pub const STRINGS: &str = "strings";
}

/// 校验一段注入文本的顶层形状（注入前预检：顶层须为对象且含非 `schema_version` 键）。
///
/// 预检失败 = 该段不注入（`None`，`table_source` 回退内嵌），不整包失败。
#[must_use]
pub fn precheck_table(raw: &str, key: &str) -> bool {
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(raw) else {
        eprintln!("[map-pack] {key}.json 顶层非对象/解析失败 ⇒ 该段回退内嵌");
        return false;
    };
    map.keys().any(|k| k != SCHEMA_VERSION_KEY)
}
