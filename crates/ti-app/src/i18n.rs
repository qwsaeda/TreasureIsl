//! i18n 访问层：把 `assets/data/strings.json` 暴露成渲染层可直接用的 `sget`。
//!
//! 用法：`sget("scene.select_role.title")` → `&'static str`。
//!
//! 设计（`docs/process/02_conventions.md` §1，数值/文本数据化）：
//! - 文案唯一事实源是 `assets/data/strings.json`，不在 `.rs` 里写中文字面量；
//! - **缺失 key / 整表不可用 → 返回 key 本身**（界面上显示 `scene.select_role.title`
//!   这种裸 key，肉眼可见异常；不静默 fallback 成错误文本，也不 panic——门禁红线）；
//! - `sget` 非 `const`（`ti_core::data::strings()` 用 `OnceLock` 惰性摊平）。
//!
//! 实现：不自己做第二次缓存，直接委托 `ti_core::data::strings()` 返回的
//! `&'static Strings`；`sget` = `strings().get(key)`（缺失回 key 本身，与 `Strings::get` 一致）。
//! 改坏表时 `strings()` 返回 `Err`，`sget` 回退成返回 key 本身（可见异常，不 panic）。

use ti_core::data;

/// 简写取文本：`sget("scene.select_role.title")`。
///
/// - 命中：返回 strings.json 里的文本；
/// - 缺失 / 整表改坏：返回 `key` 本身（界面显示裸 key，肉眼可见异常，不 fallback 假文本）。
///
/// 刻意用自由函数（而非 `S::get` 关联函数）：迁移脚本（`scripts/migrate_strings.py`）
/// 生成的调用点嵌在任意表达式位置，自由函数解析最稳。
///
/// ```ignore
/// shell.draw_text_row(sget("scene.select_role.title"), x, y);
/// ```
#[must_use]
pub fn sget(key: &str) -> &str {
    match data::strings() {
        Ok(t) => t.get(key),
        // 改坏表 → 回 key 本身（可见异常，不 panic、不静默假文本）
        Err(_) => key,
    }
}

/// `sget` 的 `Option` 版本（供校验类代码使用；`None` = 未命中或整表不可用）。
#[must_use]
pub fn try_sget(key: &str) -> Option<&str> {
    data::strings().ok().and_then(|t| t.try_get(key))
}

/// 取格式化文案：`sgetf("hud.gold_fmt", [g])` → `String`（填好 `{0}` 占位符）。
///
/// 文案 key 值形如 `"金 {0}"`；缺失 / 整表改坏时返回 **key 本身**（不填值），
/// 让异常在界面上肉眼可见，不 panic、不静默 fallback（门禁红线）。
/// 供 `format!` 拼接的 HUD 文案用（`金{gold}` / `持有{n}` / `回合{n}` 等）。
#[must_use]
pub fn sgetf(key: &str, args: &[&dyn std::fmt::Display]) -> String {
    let tmpl = match data::strings() {
        Ok(t) => match t.try_get(key) {
            Some(v) => v.to_string(),
            None => key.to_string(),
        },
        Err(_) => key.to_string(),
    };
    fill_args(&tmpl, args)
}

/// 把 `{0} {1}` 位置占位符替换进 `args`（无占位符时原样返回，不报错）。
fn fill_args(tmpl: &str, args: &[&dyn std::fmt::Display]) -> String {
    let mut out = String::with_capacity(tmpl.len() + args.len() * 4);
    let mut rest = tmpl;
    while let Some(pos) = rest.find('{') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos + 1..];
        let Some(end) = tail.find('}') else {
            out.push_str(tail);
            return out;
        };
        let inner = &tail[..end];
        if let Ok(idx) = inner.parse::<usize>() {
            if idx < args.len() {
                out.push_str(&args[idx].to_string());
            }
        } else {
            out.push_str(inner);
        }
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    out
}

/// 整表是否成功加载（`false` = 界面会显示裸 key，属异常状态，启动时断言一次）。
#[must_use]
pub fn is_loaded() -> bool {
    data::strings().is_ok()
}
