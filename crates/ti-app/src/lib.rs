//! 组装层：场景注册、主循环、三端入口。
//!
//! 本 crate 目前包含四块：
//! - [`error`]：跨层错误聚合（`docs/style.md` §5.5），任何子系统的错误在这里
//!   汇聚成一个类型，入口处统一打印；
//! - [`glyph_raster`]：真实 `GoldLand.Fnt` 字形 → 墨点位图 + 豆腐块回退
//!   （验收测试与 shell 共用，S2.5 补实）；
//! - [`audio_prep`]：`music.lpk` 解包 → `parse_wav` → `Mixer` → 可播放 WAV 字节
//!   （S2.5 补实）与 [`shell_ui`]：壳的菜单表/状态行文本；
//! - [`game_app`]：可玩交互核心（S5.1，把 `ti-core::rules` 的 13 个玩法模块接进表现层，
//!   见 `docs/gamebook.md`）；
//! - [`touch`]：触屏输入层（**直接触摸界面元素，不画虚拟按键**，见 `docs/modernization.md` §5）；
//! - [`i18n`]：文本访问层（`S.get("key")` → `assets/data/strings.json`，S8.3 数据化）。
//!
//! 三端入口 bin 见 `src/bin/`（`treasure-headless` 需 `headless` feature；
//! `treasure-android` 需 `android-shell` feature）。S18.8a 起**发货入口 = Bevy 壳**
//! （`crates/ti-engine/src/bin/treasure_bevy.rs`：`treasure-bevy-native` / `treasure-bevy-wasm`），
//! 本 crate 的 macroquad native/wasm 壳已退役。

pub mod audio_prep;
pub mod commands;
pub mod error;
/// S14 正向渲染导航辅助（forward 态 7 场景状态机导航，`render-png` bin 与
/// `tests/render_snapshot_forward.rs` 共用，见 `docs/business/08_layout_design.md` §4）。
pub mod forward_nav;
pub mod game_app;
pub mod glyph_raster;
#[cfg(feature = "hd-8bit")]
pub mod hd_manifest;
pub mod i18n;
/// S18.8b：`image` 原语的 CPU 侧贴图资源缓存（进程级；路径 = 相对 `assets/` 根）。
#[cfg(feature = "hd-8bit")]
pub mod image_cache;
pub mod menu_shell;
pub mod particles;
pub mod scenes;
pub mod shell_ui;
pub mod sprite_atlas;
pub mod terrain_atlas;
pub mod touch;
pub mod widgets;
pub mod world_camera;
pub mod z_layers;

// S18.8c：`android_shell` mod 随 macroquad 壳退役删除；S18.8d = Bevy android 壳
// （`android/android-activity` crate 经 include! 复用 ti-engine Bevy 壳源）

#[cfg(feature = "stats")]
pub mod stats;

pub use commands::{Command as CommandTrait, GameCommand};
pub use error::{Error, Result};
pub use game_app::{BattleSnapshot, BattleView, GameApp, GameInput, Scene};
pub use i18n::{sget, sgetf, try_sget};
pub use menu_shell::MenuShell;
pub use particles::{FxKind, ParticleFx};
pub use scenes::{
    EconomyHost, IndexScene, MainMapScene, MapHost, MenuHost, MessageScene, NavHost, RenderHost,
    SaveloadScene, Scene as SceneTrait, SceneHost, SceneKind, SceneRegistry, SceneTransition,
    SelectMapScene, SelectRoleScene, SelectionHost, SessionHost, SpriteHost, VsScene, markers,
    route_hit, route_input, route_render,
};
pub use shell_ui::{MENU, MenuItem};
pub use sprite_atlas::SpriteAtlas;
pub use terrain_atlas::TerrainAtlas;
pub use ti_render as render;
pub use touch::{TouchPad, TouchSample};
pub use world_camera::{WorldCam, clamp_zoom};
pub use z_layers::{Z_ENTITY, Z_FX, Z_HUD, Z_OVERLAY, Z_TERRAIN, Z_TEXT};
