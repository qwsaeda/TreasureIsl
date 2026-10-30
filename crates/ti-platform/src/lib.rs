//! 平台抽象层：绘图、输入、音频、存储、时钟的 trait 定义。
//!
//! 设计（`docs/style.md` §5.6）：
//! - 后端用**关联类型 + 泛型**注入，不用 `dyn`（热路径零虚调用）；
//! - 所有实现共享同一套 trait，因此 `HeadlessBackend`（测试）与
//!   渲染后端（三端）可互换，回归测试不依赖窗口/声卡。
//!
//! S18.8b（macroquad 退役上洗）：`macroquad` 依赖整体删除。
//! - `gfx`：GL 呈现层（纹理上传 / 全屏 quad / 边框填充）删；
//!   `FramePresenter` 保留为脏行状态机 + 计数（stats 观测口径不变）。
//! - `gpu_renderer`：GL 逐精灵 blit 删；保留图集抽象 `SpriteRef` + 计数
//!   （真实上屏 = ti-engine Bevy 壳，S18.8a 起）。
//! - `settings`：`FilterMode` 薄封装改挂 bevy `TextureFilter`（2 变体 1:1）。

pub mod audio;
pub mod audio_asset;
pub mod backend;
pub mod error;
pub mod gfx;
/// S10.4：GPU 化逐精灵 batch 渲染（图集抽象 + 计数；GL 呈现 S18.8b 起由 Bevy 壳承担）。
pub mod gpu_renderer;
/// S7.5：HD 资产 PNG 解码（feature `hd-8bit` 门控，默认不编译）。
#[cfg(feature = "hd-8bit")]
pub mod hd_png;
pub mod input;
pub mod mod_pcm;
#[cfg(feature = "ogg")]
pub mod ogg_encode;
pub mod settings;
pub mod storage;

pub use audio::{Clip, Mixer, parse_wav};
pub use audio_asset::{Asset, Kind};
pub use backend::{
    Audio, Backend, Channel, Clock, Event, Gfx, Input, KeyCode, Rect, Rgb565, Storage,
};
pub use error::{Error, Result};
pub use gfx::{FB_PIXELS, FramePresenter, UploadMode, Viewport};
pub use input::{HeadlessInput, VirtualButton, translate_event};
pub use settings::{
    Gesture, GestureTracker, KeyBindings, Orientation, ScaleMode, Settings, VirtualKeyPad,
};
pub use storage::{FileStore, MemoryStore, validate_key};
