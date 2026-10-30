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
//! 本模块 **0 游戏依赖 + 0 写盘**（写点仅 `evidence/console/` = 平台归档目录，
//! 快照对比报告由调用方落盘，本模块自身只做只读计算）。

pub mod config;
pub mod data;
pub mod guard;
pub mod snapshot;
