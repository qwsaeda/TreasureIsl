//! 文案表：`assets/data/strings.json` → `Strings`（i18n 委托目标）。
//! 拆分自 `data.rs`（S11）；顶层 re-export `strings` / `pairs`，外部 API 不变。

use std::collections::BTreeMap;
use std::sync::OnceLock;

use super::{LoadError, SCHEMA_VERSION_KEY, STRINGS_JSON, Strings, map_pack, parse, table_source};

static STRINGS: OnceLock<Strings> = OnceLock::new();
/// 文案键值对快照（首次调用解析，之后命中缓存）；供 i18n 遍历用。
pub fn pairs() -> &'static [(&'static str, &'static str)] {
    static PAIRS: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
    PAIRS.get_or_init(|| {
        let table = STRINGS.get_or_init(load_strings_table);
        table.map.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    })
}

/// 加载文本表（首次解析，之后命中缓存）。
///
/// # Errors
/// JSON 语法错误、非 `string→string` 映射时返回 [`LoadError`]（不 fallback）。
pub fn strings() -> Result<&'static Strings, LoadError> {
    if let Some(v) = STRINGS.get() {
        return Ok(v);
    }
    let raw: BTreeMap<String, serde_json::Value> = parse(
        table_source::table_text(map_pack::table_key::STRINGS, STRINGS_JSON).as_ref(),
        "strings.json",
    )?;
    let mut map = BTreeMap::new();
    for (k, v) in raw {
        if k == SCHEMA_VERSION_KEY {
            continue;
        }
        match v {
            serde_json::Value::String(s) => {
                map.insert(k, s);
            }
            other => {
                return Err(LoadError::Schema(format!(
                    "strings.json: `{k}` should be text, got {other}"
                )));
            }
        }
    }
    Ok(STRINGS.get_or_init(|| Strings { map }))
}

/// 仅供 [`pairs`] 使用的「加载或回退空表」入口（`pairs` 非 Result 语义）。
fn load_strings_table() -> Strings {
    let text = table_source::table_text(map_pack::table_key::STRINGS, STRINGS_JSON);
    match parse::<BTreeMap<String, serde_json::Value>>(text.as_ref(), "strings.json") {
        Ok(raw) => {
            let mut map = BTreeMap::new();
            for (k, v) in raw {
                if k == SCHEMA_VERSION_KEY {
                    continue;
                }
                if let serde_json::Value::String(s) = v {
                    map.insert(k, s);
                }
            }
            Strings { map }
        }
        Err(_) => Strings::default(),
    }
}
