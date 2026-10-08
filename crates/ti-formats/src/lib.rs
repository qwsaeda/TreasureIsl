//! `TreasureIsl` 二进制资源格式的解析与序列化。
//!
//! 硬指标（见 `docs/parity.md` P0/P1）：
//! - **0 未知字节**：解析必须消费完输入的每一个字节；
//! - **roundtrip 逐字节一致**：`read` 与 `write` 由同一份字段表生成。
//!
//! 二者由 [`macros`] 中的 `define_format!` 统一保证，各格式只需声明字段表，
//! 不手写读写代码；字段读写能力由 [`field::Field`] 提供。

pub mod error;
pub mod field;
pub mod font;
pub mod ini;
pub mod lucency;
pub mod macros;
pub mod mapfile;
pub mod pcx;
pub mod save;
pub mod terrain;

pub use error::{Error, ParseKind, Result};
pub use field::Field;
pub use font::{Charset, Font, Glyph};
pub use ini::{Ini, IniEntry};
pub use lucency::{Entry, Package, name_key};
pub use mapfile::{MapHeader, MapScene};
pub use pcx::Image;
pub use save::{SaveEnvelope, Settings};
pub use terrain::{CellRecord, ConnectionRecord, TerrainMap};
