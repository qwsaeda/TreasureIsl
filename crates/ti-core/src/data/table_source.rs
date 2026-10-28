//! 表注入接缝（S31.2，`docs/process/21_map_package_spec.md` §7 选 B）：
//! 壳层启动时选定地图包 → 直读包三段表文本 → 注入本模块；各表访问器（`board()` /
//! `scenes()` / `strings()`）解析前查注入源，**缺注入 = 内嵌 `include_str!` 默认包**
//! （§6 失败回退语义：0 行为变化，wasm 无包时照旧全内嵌）。
//!
//! 时序约束（硬规则）：注入必须在**首次表访问之前**完成（`OnceLock` 生成后不可替换，
//! 迟注入 = 静默忽略 + `eprintln` 告警）。壳层在 `GameApp` 构造 / `load_resources`
//! 前调用 [`install`]（首访前）；已首访后的调用只告警不改行为。
//!
//! 0 游戏语义：本模块只管「哪段文本」，解析 / 校验全在既有表模块（口径不双份）。
#![allow(clippy::doc_markdown)]

use std::sync::OnceLock;

use crate::data::map_pack::table_key;

/// 已注入的包表文本（`None` 段 = 该段走内嵌默认；`install` 幂等：首访后覆盖 = 忽略）。
#[derive(Debug, Default, Clone)]
pub struct PackTables {
    /// 注入来源包 id（日志 / 证据用；`None` = 纯内嵌默认包）。
    pub source_id: Option<String>,
    /// `board.json` 包文本（`Some` = 覆盖内嵌 `BOARD_JSON`；`None` = 内嵌）。
    pub board: Option<String>,
    /// `scenes.json` 包文本（`Some` = 覆盖内嵌 `SCENES_JSON`；`None` = 内嵌）。
    pub scenes: Option<String>,
    /// `strings.json` 包文本（`Some` = 覆盖内嵌 `STRINGS_JSON`；`None` = 内嵌）。
    pub strings: Option<String>,
    /// 已安装包标识列表（S31.2b：`select_map` 包列表 UI 数据源；
    /// 壳层注入时由 `map_pack::available_pack_ids` 填充；`None` = 未采集 = UI 走内嵌 3 槽）
    pub available_ids: Option<Vec<String>>,
}

static PACK: OnceLock<PackTables> = OnceLock::new();

/// 注入包表（**首次表访问前**调用；迟调用 = 忽略 + 告警，不 panic、不改已生成缓存）。
///
/// 返回 `true` = 已注入生效；`false` = 表访问已发生（`OnceLock` 已定型），保持内嵌默认。
#[must_use]
pub fn install(tables: PackTables) -> bool {
    let accepted = PACK.set(tables).is_ok();
    if !accepted {
        eprintln!("[table_source] 迟注入（表已首访）⇒ 忽略，保持内嵌默认包");
    }
    accepted
}

/// 当前注入源包 id（`None` = 未注入 / 纯内嵌默认包；壳层日志 + 出图证据用）。
#[must_use]
pub fn source_id() -> Option<&'static str> {
    PACK.get().and_then(|t| t.source_id.as_deref())
}

/// 已安装包标识列表（S31.2b：`select_map` 包列表 UI 数据源；`None` = 未注入 / 未采集，
/// UI 走内嵌 `MapSlot` 3 槽旧流，0 行为变化）
#[must_use]
pub fn available_ids() -> Option<Vec<String>> {
    PACK.get().and_then(|t| t.available_ids.clone())
}

/// 取某段表的生效文本（注入优先；未注入该段 = 内嵌默认文本）。
///
/// `key` = [`table_key`] 常量（`board` / `scenes` / `strings`）。返回 `Cow`：注入
/// （`PACK` 常驻堆文本）= `Owned` 克隆，内嵌默认 = `Borrowed`（避免 `'static` 越权）。
#[must_use]
pub fn table_text(key: &str, embedded: &str) -> std::borrow::Cow<'static, str> {
    let override_text = PACK.get().and_then(|t| match key {
        table_key::BOARD => t.board.as_ref(),
        table_key::SCENES => t.scenes.as_ref(),
        table_key::STRINGS => t.strings.as_ref(),
        _ => None,
    });
    match override_text {
        Some(s) => std::borrow::Cow::Owned(s.clone()),
        None => std::borrow::Cow::Owned(embedded.to_string()),
    }
}
