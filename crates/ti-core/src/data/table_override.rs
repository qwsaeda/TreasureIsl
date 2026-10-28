//! 数据表注入层（S31.2，`docs/process/21_map_package_spec.md` §7 运行期加载契约·选 B）：
//! 启动期把「地图包」的表 JSON 文本注入，**替代编译期内嵌默认数据**（`include_str!`）。
//!
//! 契约：
//! - [`set`] 只能在**首次表访问之前**调用一次（`board()`/`scenes()`/`strings()` 任一触发
//!   `OnceLock` 生成后，`set` 返回 `false` 且被忽略 —— 已生成的缓存不会被替换）；
//! - 未注入 / 注入失败 = 全部表用内嵌默认数据（§6 失败回退语义，0 行为变化）；
//! - 注入内容本身解析失败仍 = 硬错误（各表访问器照常返回 [`super::LoadError`]，不 fallback）。
//!
//! 消费方：`board`/`layout`/`strings` 三个表模块的访问器在解析前查 [`board_text`] /
//! [`scenes_text`] / [`strings_text`]；壳层（`ti-engine`）在构造 `GameApp` 前完成注入。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};

/// 已注入的表文本（`None` 表 = 保持内嵌默认）。
#[derive(Debug, Default, Clone)]
pub struct TableOverride {
    /// `board.json` 全文（替换内嵌 `BOARD_JSON`）。
    pub board: Option<String>,
    /// `scenes.json` 全文（替换内嵌 `SCENES_JSON`）。
    pub scenes: Option<String>,
    /// `strings.json` 全文（替换内嵌 `STRINGS_JSON`）。
    pub strings: Option<String>,
    /// 注入来源标签（如包 id `m02`；可空 = 匿名注入）。
    pub source: Option<String>,
}

/// 注入的表文本（`Mutex` 持有：首访前可写，首访后各访问器只读取值）。
static PENDING: OnceLock<Mutex<Option<TableOverride>>> = OnceLock::new();
/// 首次表访问完成标记（完成后的 `set` 一律拒绝）。
static ACCESSORS_DONE: AtomicBool = AtomicBool::new();

fn pending() -> &'static Mutex<Option<TableOverride>> {
    PENDING.get_or_init(|| Mutex::new(None))
}

/// 注入表覆盖（**必须在首次表访问之前调用**）。
///
/// 返回 `true` = 已记录（后续访问器使用注入文本）；`false` = 表访问已发生，忽略（0 副作用）。
#[must_use]
pub fn set(board_json: Option<String>, scenes_json: Option<String>, strings_json: Option<String>) -> bool {
    if ACCESSORS_DONE.load(Ordering::SeqCst) {
        return false;
    }
    let guard: MutexGuard<'_, Option<TableOverride>> =
        pending().lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(TableOverride { board: board_json, scenes: scenes_json, strings: strings_json, source: None });
    true
}

/// 带来源标签的注入（壳层用：`source` = 包 id；语义同 [`set`]）。
#[must_use]
pub fn set_with_source(
    source: &str,
    board_json: Option<String>,
    scenes_json: Option<String>,
    strings_json: Option<String>,
) -> bool {
    if ACCESSORS_DONE.load(Ordering::SeqCst) {
        return false;
    }
    let guard: MutexGuard<'_, Option<TableOverride>> =
        pending().lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(TableOverride { board: board_json, scenes: scenes_json, strings: strings_json, source: Some(source.to_owned()) });
    true
}

/// 首次表访问前由表模块调用：标记完成 + 取走 `None` 语义的兜底。
///
/// 返回 `Some(覆盖文本)` = 该表已注入；`None` = 用内嵌默认（或已完成首访且无注入）。
pub(crate) fn take_text(kind: &str) -> Option<&'static str> {
    ACCESSORS_DONE.store(true, Ordering::SeqCst);
    let guard: MutexGuard<'_, Option<TableOverride>> =
        pending().lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().and_then(|o| match kind {
        "board" => o.board.as_ref(),
        "scenes" => o.scenes.as_ref(),
        "strings" => o.strings.as_ref(),
        _ => None,
    }).map(|s| s.as_str())
}

/// 注入来源标签（`Some` = 已注入且带标签；`None` = 未注入 / 无标签）。
#[must_use]
pub fn source() -> Option<&'static str> {
    let guard: MutexGuard<'_, Option<TableOverride>> =
        pending().lock().unwrap_or_else(|e| e.into_inner());
    guard.as_ref().and_then(|o| o.source.as_deref())
}
