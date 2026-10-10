//! 容器条目 key → 业务名反查（S6.5）。
//!
//! 原版容器条目以 `key = djb2(name_lowercase)` 索引，但 `key` 本身不含名称信息；
//! 导出产物（`build_assets` 的 `raw/`、`unpak --out`）需按**业务名**命名而非哈希，
//! 本模块提供三级反查权威源 + 同名内容合并的无状态解析器。
//!
//! 命名权威源优先级（用户决策 2026-10，S6.5）：
//! 1. **`mapinfo.ini`**（容器内清单，F-210）——逐 `filename=` 行 `name_key` 闭合，0 失配；
//! 2. **exe UTF-16 资源名**（S1.1 命中 85/88 glpics + cards/music/goods 条目）——
//!    由调用方（ti-tools）扫描 exe 后注入 [`ResourceNames::register_exe_names`]；
//! 3. **`EXTRAS`** 多键结构证实名（`card1.pcx`/`card2.pcx`/`Thumbs.db` 等，
//!    `ti-tools/tests/id_keys.rs` 的 `EXTRAS` 表）；
//! 4. 全部未中 → 回退 `unknown-{key:08x}.bin`（glpics 余 3 条名称哈希型，F-209）。
//!
//! 同名重复处理（用户决策）：**内容相同时合并**——同名两条目字节全同则
//! 只出一个文件（后者标 `skipped`，不产生第二个路径）；内容不同时首见生效、
//! 其余加 `_2`/`_3`… 序号后缀。
//!
//! 本模块纯函数、无 IO：[`mapinfo_filenames`] 只读已解析的 `mapinfo.ini` 字节，
//! 各 `register_*` 由调用方注入，[`display_names`] 一次性产出「下标 → 显示名 +
//! 是否合并跳过 + 来源」三组对齐数组，[`export_csv`] 序列化为带 `source` 列的
//! CSV 行（供 `unpak` 落盘与覆盖率统计）。

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::Package;
use crate::ini::Ini;
use crate::lucency::name_key;

/// `mapinfo.ini` 的 key 值（`djb2("mapinfo.ini")`，各容器一致，F-210）。
pub const MAPINFO_KEY: u32 = 0x3753_DC98;

/// 一个条目的命名来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NameSource {
    /// 容器内 `mapinfo.ini` 的 `filename=` 行（权威，优先级最高）。
    Mapinfo,
    /// exe UTF-16 资源名扫描命中（`music\glmusic.mod` 等）。
    Exe,
    /// 多键结构 / 内容格式证实名（`card1.pcx`/`card2.pcx`/`Thumbs.db`）。
    Extra,
    /// 三级源均未命中，回退 `unknown-{key:08x}.bin`。
    Unknown,
}

impl NameSource {
    /// CSV / 诊断输出用的小写标签。
    #[must_use]
    pub const fn as_tag(self) -> &'static str {
        match self {
            Self::Mapinfo => "mapinfo",
            Self::Exe => "exe",
            Self::Extra => "extra",
            Self::Unknown => "unknown",
        }
    }
}

/// key → 业务名的反查表（无状态，纯数据）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceNames {
    /// 命名权威源表：key → (业务名, 来源)。同名 key 首见生效（后登记者不覆盖）。
    named: BTreeMap<u32, (String, NameSource)>,
    /// `mapinfo.ini` 中出现过但未在容器条目内闭合的 filename（诊断用，不进 [`ResourceNames::name_of`]）。
    mapinfo_unresolved: BTreeSet<String>,
}

impl ResourceNames {
    /// 构造空反查表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注入 `mapinfo.ini` 条目（权威源，优先级最高）。
    ///
    /// `filenames` = 容器内 `mapinfo.ini` 解析出的全部 `filename=` 值（含重复行）。
    /// 同名重复按「首见生效」登记：同一 name 多次出现只产生一条 `key → name` 映射。
    pub fn register_mapinfo(&mut self, filenames: &[String]) {
        for name in filenames {
            self.insert_if_absent(name_key(name), name.as_str(), NameSource::Mapinfo);
        }
    }

    /// 注入 exe UTF-16 资源名（第二级权威源，不覆盖已有任意来源的条目）。
    ///
    /// 每个名字同时登记**叶名**与**全路径**两个候选：exe 内资源名形如
    /// `music\glmusic.mod`，容器内 key 仅由叶名 `glmusic.mod` 经 [`name_key`] 计算，
    /// 但全路径的 key 也可能是某条目的合法索引，两个都登记以便命中。
    pub fn register_exe_names(&mut self, names: &[String]) {
        for full in names {
            let leaf = full.rsplit('\\').next().unwrap_or(full.as_str());
            self.insert_if_absent(name_key(leaf), leaf, NameSource::Exe);
            self.insert_if_absent(name_key(full), full, NameSource::Exe);
        }
    }

    /// 注入多键结构 / 内容格式证实名（第三级权威源，`EXTRAS` 表，不覆盖已有条目）。
    pub fn register_extras(&mut self, names: &[&str]) {
        for name in names {
            self.insert_if_absent(name_key(name), name, NameSource::Extra);
        }
    }

    /// 直接按 key 登记业务名（第四级权威源，孤儿条目内容推断名，不覆盖已有条目）。
    ///
    /// 用于 `glpics.pak` 余 3 条名称哈希型孤儿条目（`0x0D4EF1BE`/`0x20769D35`/`0x6171A968`，
    /// F-209）：这些 key 在 exe 中无 UTF-16 资源名对应，也不闭合 `djb2(name)`，
    /// 只能按内容（PCX 尺寸/像素特征）推断业务名后直接登记。
    pub fn register_key_name(&mut self, key: u32, name: &str) {
        self.insert_if_absent(key, name, NameSource::Extra);
    }

    /// 仅当该 key 尚未登记时插入（保证首见生效，不覆盖更高优先级来源）。
    fn insert_if_absent(&mut self, key: u32, name: &str, source: NameSource) {
        self.named.entry(key).or_insert_with(|| (name.to_string(), source));
    }

    /// 取某 key 的业务名（含来源）。未命名返回 `None`。
    #[must_use]
    pub fn name_of(&self, key: u32) -> Option<(&str, NameSource)> {
        self.named.get(&key).map(|(n, s)| (n.as_str(), *s))
    }

    /// 是否已命名（三级源任一命中）。
    #[must_use]
    pub fn is_named(&self, key: u32) -> bool {
        self.named.contains_key(&key)
    }

    /// 已命名 key 的总数。
    #[must_use]
    pub fn named_count(&self) -> usize {
        self.named.len()
    }

    /// 登记某容器内 `mapinfo.ini` 未闭合的 filename（诊断用，不进 [`ResourceNames::name_of`]）。
    pub fn record_unresolved(&mut self, name: &str) {
        self.mapinfo_unresolved.insert(name.to_string());
    }

    /// 全部未闭合的 mapinfo filename（字典序，借用本表）。
    #[must_use]
    pub fn unresolved_mapinfo_names(&self) -> Vec<&String> {
        self.mapinfo_unresolved.iter().collect()
    }

    /// 一步式：从已解析的 `Package` 读 `mapinfo.ini` 并注入本表（见 [`mapinfo_filenames`]）。
    ///
    /// 容器不含 `mapinfo.ini`（如 `cards.lpk`/`music.lpk`/`goods.lpk`）时为空操作，返回 `self`。
    pub fn load_from_package(&mut self, pkg: &Package, buf: &[u8]) -> &mut Self {
        let filenames = mapinfo_filenames(pkg, buf);
        self.register_mapinfo(&filenames);
        self
    }
}

/// 从已解析的 `Package` + 容器原始字节中读出 `mapinfo.ini` 的全部 `filename=` 值。
///
/// 仅当容器内含 `mapinfo.ini` 条目（key [`MAPINFO_KEY`]）且条目可解压、INI 可解析时
/// 返回；否则返回空表（纯函数，不报错——`cards.lpk`/`goods.lpk`/`music.lpk`
/// 本身不含 mapinfo，空表是合法结果）。
#[must_use]
pub fn mapinfo_filenames(pkg: &Package, buf: &[u8]) -> Vec<String> {
    let Some(idx) = pkg.find_key(MAPINFO_KEY) else {
        return Vec::new();
    };
    let Ok(raw) = pkg.read_entry(buf, idx) else {
        return Vec::new();
    };
    let Ok(ini) = Ini::parse_utf16le("mapinfo.ini", &raw) else {
        return Vec::new();
    };
    ini.entries
        .iter()
        .filter(|e| e.key.eq_ignore_ascii_case("filename"))
        .map(|e| e.value.clone())
        .collect()
}

/// 一条导出项：显示名 + 是否被合并跳过 + 来源标签。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportEntry {
    /// 导出显示名（含同名内容合并 / 内容不同序号后缀处理）。
    pub display: String,
    /// 是否被同名内容合并跳过（同内容只落第一个文件）。
    pub skipped: bool,
    /// 命名来源标签（`mapinfo`/`exe`/`extra`/`unknown`）。
    pub source: &'static str,
}

/// 把 `ResourceNames` + 容器条目列表一次性构建为「下标 → [`ExportEntry`]」对齐数组。
///
/// 处理规则（与 [`ResourceNames::name_of`] 配合）：
/// - 已命名条目取业务名；未命名回退 `unknown-{key:08x}.bin`；
/// - 同名（显示名相同）且内容字节全同：首见保留原名，其余标 `skipped=true`；
/// - 同名但内容不同：首见保留原名，其余加 `_2`/`_3`… 序号后缀。
///
/// 纯计算，需 `pkg`/`buf` 仅为读条目内容做合并判定；调用方负责把
/// `skipped=true` 的条目从落盘与 manifest 中剔除。
#[must_use]
pub fn export_entries(pkg: &Package, buf: &[u8], names: &ResourceNames) -> Vec<ExportEntry> {
    // 1) 按条目顺序算初始显示名 + 来源
    let n = pkg.entries.len();
    let mut per: Vec<(String, NameSource)> = Vec::with_capacity(n);
    for e in &pkg.entries {
        let entry = names.name_of(e.key);
        match entry {
            Some((d, s)) => per.push((d.to_string(), s)),
            None => per.push((format!("unknown-{:08x}.bin", e.key), NameSource::Unknown)),
        }
    }

    // 2) 按显示名分组，组内比内容
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (i, (d, _)) in per.iter().enumerate() {
        groups.entry(d.clone()).or_default().push(i);
    }

    let mut out: Vec<Option<ExportEntry>> = vec![None; n];
    for (display, idxs) in &groups {
        if idxs.len() < 2 {
            let i = idxs[0];
            let (d, s) = &per[i];
            out[i] = Some(ExportEntry { display: d.clone(), skipped: false, source: s.as_tag() });
            continue;
        }

        // 组内内容是否全同
        let mut bytes: Vec<Option<Vec<u8>>> = Vec::with_capacity(idxs.len());
        for &i in idxs {
            let b = pkg.read_entry(buf, i).ok();
            bytes.push(b);
        }
        let mut all_same = bytes.iter().all(Option::is_some);
        if all_same {
            let first = bytes[0].clone().unwrap_or_default();
            for b in &bytes[1..] {
                let cur = b.clone().unwrap_or_default();
                if cur != first {
                    all_same = false;
                    break;
                }
            }
        }

        let source = per[idxs[0]].1;
        for (k, &i) in idxs.iter().enumerate() {
            if all_same {
                // 内容全同 → 首见保留，其余跳过（用户决策「内容相同时合并」）
                out[i] = Some(ExportEntry {
                    display: display.clone(),
                    skipped: k != 0,
                    source: source.as_tag(),
                });
            } else {
                // 内容不同 → 首见保留原名，其余加序号后缀
                let display_name =
                    if k == 0 { display.clone() } else { format!("{display}_{}", k + 1) };
                out[i] = Some(ExportEntry {
                    display: display_name,
                    skipped: false,
                    source: source.as_tag(),
                });
            }
        }
    }

    // 3) 收尾：所有条目必然有值（groups 覆盖全部下标）
    let mut all = Vec::with_capacity(n);
    for slot in out {
        all.push(slot.unwrap_or(ExportEntry {
            display: String::new(),
            skipped: false,
            source: NameSource::Unknown.as_tag(),
        }));
    }
    all
}

/// 把 [`export_entries`] 结果序列化为 CSV（表头 `idx,display,source`，跳过合并项）。
///
/// 显示名不含逗号（资源名实测全是 `[A-Za-z0-9_.\\-]`），无需引号转义；
/// 若出现含逗号/换行的名称则**显式跳过该行**（不静默错列，防御性设计）。
#[must_use]
pub fn export_csv(pkg: &Package, entries: &[ExportEntry]) -> String {
    let mut out = String::from("idx,display,source\n");
    for (i, item) in entries.iter().enumerate().take(pkg.entries.len()) {
        if item.skipped {
            continue;
        }
        if item.display.contains(',') || item.display.contains('\n') {
            // 资源名实测不含这些字符；出现时跳过该行而非静默错列
            continue;
        }
        let _ = writeln!(out, "{i},{},{}", item.display, item.source);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lucency::MAGIC;
    use crate::lucency::Package;
    use std::io::Write as _;

    fn build_pkg(keys: &[u32]) -> Vec<u8> {
        let mut idx = Vec::new();
        let mut off = 0u32;
        for k in keys {
            idx.extend_from_slice(&k.to_le_bytes());
            idx.extend_from_slice(&off.to_le_bytes());
            idx.extend_from_slice(&10u32.to_le_bytes());
            idx.extend_from_slice(&5u32.to_le_bytes());
            off += 5;
        }
        let mut e = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        let _ = e.write_all(&idx);
        let body = e.finish().unwrap_or_default();
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&[0u8; 0x36 - 16]);
        out[0x1A..0x1C].copy_from_slice(&2004u16.to_le_bytes());
        out[0x26..0x2A].copy_from_slice(&u32::try_from(keys.len()).unwrap_or(0).to_le_bytes());
        out[0x2E..0x32].copy_from_slice(&u32::try_from(body.len()).unwrap_or(0).to_le_bytes());
        out.extend_from_slice(&body);
        out
    }

    fn mk_names(names: &[(u32, &'static str, NameSource)]) -> ResourceNames {
        let mut r = ResourceNames::new();
        for (key, name, source) in names {
            match source {
                NameSource::Mapinfo => r.register_mapinfo(&[name.to_string()]),
                NameSource::Exe => r.register_exe_names(&[name.to_string()]),
                NameSource::Extra => r.register_extras(&[name]),
                NameSource::Unknown => {}
            }
            let _ = key;
        }
        r
    }

    #[test]
    fn unresolved_mapinfo_recorded_separately() {
        let mut r = ResourceNames::new();
        r.record_unresolved("zzz.pcx");
        assert!(r.unresolved_mapinfo_names().iter().any(|s| s.as_str() == "zzz.pcx"));
    }

    #[test]
    fn unknown_key_has_no_name() {
        let mut r = ResourceNames::new();
        r.register_mapinfo(&["50.pcx".to_string()]);
        assert!(r.name_of(0x1234_5678).is_none());
        assert!(!r.is_named(0x1234_5678));
    }

    #[test]
    fn export_entries_falls_back_to_unknown_for_unnamed_key() {
        let keys = [name_key("50.pcx"), 0x1234_5678];
        let buf = build_pkg(&keys);
        let Ok(pkg) = Package::parse("t.pak", &buf) else {
            return;
        };
        let names = mk_names(&[]);
        let entries = export_entries(&pkg, &buf, &names);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].display, format!("unknown-{:08x}.bin", name_key("50.pcx")));
        assert_eq!(entries[0].source, "unknown");
        assert_eq!(entries[1].display, "unknown-12345678.bin");
    }

    #[test]
    fn export_entries_prefers_named_over_unknown() {
        let keys = [name_key("50.pcx"), name_key("100.pcx")];
        let buf = build_pkg(&keys);
        let Ok(pkg) = Package::parse("t.pak", &buf) else {
            return;
        };
        let mut names = ResourceNames::new();
        names.register_mapinfo(&["50.pcx".to_string(), "100.pcx".to_string()]);
        let entries = export_entries(&pkg, &buf, &names);
        assert_eq!(entries[0].display, "50.pcx");
        assert_eq!(entries[0].source, "mapinfo");
        assert_eq!(entries[1].display, "100.pcx");
    }

    #[test]
    fn export_csv_lists_non_skipped_entries_in_idx_order() {
        let keys = [name_key("50.pcx"), 0x1234_5678];
        let buf = build_pkg(&keys);
        let Ok(pkg) = Package::parse("t.pak", &buf) else {
            return;
        };
        let mut names = ResourceNames::new();
        names.register_mapinfo(&["50.pcx".to_string()]);
        let entries = export_entries(&pkg, &buf, &names);
        let csv = export_csv(&pkg, &entries);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "idx,display,source");
        assert_eq!(lines[1], "0,50.pcx,mapinfo");
        assert_eq!(lines[2], "1,unknown-12345678.bin,unknown");
    }

    #[test]
    fn register_key_name_assigns_orphan_key() {
        let mut r = ResourceNames::new();
        r.register_key_name(0x0D4E_F1BE, "boat_scene.pcx");
        assert!(r.is_named(0x0D4E_F1BE));
        let entry = r.name_of(0x0D4E_F1BE);
        assert_eq!(entry.map(|(_, s)| s), Some(NameSource::Extra));
        assert!(entry.is_some_and(|(n, _)| n == "boat_scene.pcx"));
    }

    #[test]
    fn register_key_name_does_not_override_existing() {
        let mut r = ResourceNames::new();
        r.register_mapinfo(&["50.pcx".to_string()]);
        let key = name_key("50.pcx");
        r.register_key_name(key, "override.pcx");
        let entry = r.name_of(key);
        assert!(entry.is_some_and(|(n, _)| n == "50.pcx"));
    }
}
