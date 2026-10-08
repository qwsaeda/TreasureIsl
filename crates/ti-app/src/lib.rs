//! 组装层：场景注册、主循环、三端入口。
//!
//! 本 crate 目前包含三块：
//! - [`error`]：跨层错误聚合（`docs/style.md` §5.5），任何子系统的错误在这里
//!   汇聚成一个类型，入口处统一打印；
//! - [`glyph_raster`]：真实 `GoldLand.Fnt` 字形 → 墨点位图 + 豆腐块回退
//!   （`treasure_wasm` 壳与验收测试共用，S2.5 补实）；
//! - [`audio_prep`]：`music.lpk` 解包 → `parse_wav` → `Mixer` → 可播放 WAV 字节
//!   （S2.5 补实）与 [`shell_ui`]：壳的菜单表/状态行文本。
//!
//! 三端入口 bin 见 `src/bin/`（`treasure-wasm` 需 `wasm-shell` feature）。

pub mod audio_prep;
pub mod error;
pub mod glyph_raster;
pub mod shell_ui;

pub use error::{Error, Result};
pub use shell_ui::{MENU, MenuItem, status_texts};
