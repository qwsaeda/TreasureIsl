//! S32.2 只读数据源适配层（22 号 §5/§10.2）：
//!
//! - `config`：数据源路径单一事实源 `console_sources.json` 的加载与解析
//!   （Rust 侧 0 个 `assets/xxx.json` 字面量 ⇒ CS-1 天然 0 命中，22 号 §11.2）；
//! - `guard`：repo 路径白名单 + `canonicalize` 防穿越（22 号 §10.2：
//!   比较对象 = `canonicalize(repo_root)`，被访问路径 canonicalize 后必须仍以该值为前缀）；
//! - `data`：`/api/overview` + `/api/tasks` + `/api/evidence` + `/api/assets` + `/api/session`
//!   的只读组装（ledger 解析走 `ti-devmeta::ledger`；JSON 源 = `serde_json::Value`，
//!   不结构化解析 `plan/packages.yaml`，不引 2nd 解析器）；
//! - `snapshot`：「0 游戏产物变更」内容级指纹快照模块（22 号 §8 载体，15 号第 23 行；
//!   纯计算：冻结路径逐文件 sha256 + 未跟踪文件清单，忽略 `.gitignore` 命中文件，
//!   禁用 `git status --porcelain` 口径）。
//!
//! 本模块 **0 游戏依赖**；只读面（`config`/`guard`/`data`/`snapshot`）自身 0 写盘（写点仅
//! `evidence/console/` = 平台归档目录，快照对比报告由调用方落盘）；`import` = S32.15 ② **唯一受控写点**
//! （`POST /api/map/import` 落 `assets/maps/<id>/` 四文件 + 同进程 `map_pack.py verify/sync`〔先判后回注〕，
//! 平台其余 0 写盘，北极星「0 游戏产物变更」的受控例外，22 号 §10.1 + `snapshot::import_probe` 豁免面）。

pub mod config;
pub mod data;
pub mod guard;
pub mod import;
pub mod search;
pub mod snapshot;

/// 百分号解码（query 参数；前端 `encodeURIComponent` 输出 `%XX`，服务端须解码后再匹配，
/// 否则含空格/中文/保留字符的查询按字面 `%XX` 匹配 = 0 命中；S32.8 检索 + whitelist/evidence 路由）。
///
/// 仅解 `%XX`（`+` 保持原样，与 JS `encodeURIComponent` 语义一致：空格→`%20`、`+`→`%2B`）；
/// 非法 `%XX` 序列（如 `%GG`/末尾孤立 `%`）原样保留（不崩）；UTF-8 多字节序列正确还原。
#[must_use]
pub fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((high << 4) | low);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// 十六进制字节值（`0-9a-fA-F`）；非十六进制 = `None`。
fn hex_val(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
