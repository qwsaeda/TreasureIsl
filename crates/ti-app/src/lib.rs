//! 组装层：场景注册、主循环、三端入口。
//!
//! 本 crate 目前包含四块：
//! - [`error`]：跨层错误聚合（`docs/style.md` §5.5），任何子系统的错误在这里
//!   汇聚成一个类型，入口处统一打印；
//! - [`glyph_raster`]：真实 `GoldLand.Fnt` 字形 → 墨点位图 + 豆腐块回退
//!   （`treasure_wasm` 壳与验收测试共用，S2.5 补实）；
//! - [`audio_prep`]：`music.lpk` 解包 → `parse_wav` → `Mixer` → 可播放 WAV 字节
//!   （S2.5 补实）与 [`shell_ui`]：壳的菜单表/状态行文本；
//! - [`game_app`]：可玩交互核心（S5.1，把 `ti-core::rules` 的 13 个玩法模块接进表现层，
//!   见 `docs/gamebook.md`）；
//! - [`touch`]：触屏输入层（**直接触摸界面元素，不画虚拟按键**，见 `docs/modernization.md` §5）；
//! - [`i18n`]：文本访问层（`S.get("key")` → `assets/data/strings.json`，S8.3 数据化）。
//!
//! 三端入口 bin 见 `src/bin/`（`treasure-wasm` 需 `wasm-shell` feature）。

pub mod audio_prep;
pub mod commands;
pub mod error;
pub mod game_app;
pub mod glyph_raster;
pub mod hd_manifest;
pub mod i18n;
pub mod menu_shell;
pub mod scenes;
pub mod shell_ui;
pub mod sprite_atlas;
pub mod terrain_atlas;
pub mod touch;
pub mod ttf_font;
pub mod widgets;

#[cfg(feature = "android-shell")]
pub mod android_shell;

#[cfg(feature = "stats")]
pub mod stats;

pub use commands::{Command as CommandTrait, GameCommand};
pub use error::{Error, Result};
pub use game_app::{BattleSnapshot, BattleView, GameApp, GameInput, Scene};
pub use i18n::{sget, sgetf, try_sget};
pub use menu_shell::MenuShell;
pub use scenes::{
    EconomyHost, IndexScene, MainMapScene, MapHost, MenuHost, MessageScene, NavHost, RenderHost,
    SaveloadScene, Scene as SceneTrait, SceneHost, SceneKind, SceneRegistry, SceneTransition,
    SelectMapScene, SelectRoleScene, SelectionHost, SessionHost, SpriteHost, VsScene, markers,
    route_input, route_render, route_touch,
};
pub use shell_ui::{MENU, MenuItem, status_texts};
pub use sprite_atlas::SpriteAtlas;
pub use terrain_atlas::TerrainAtlas;
pub use touch::{TouchPad, TouchSample};
