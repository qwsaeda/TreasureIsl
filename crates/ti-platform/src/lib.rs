//! 平台抽象层：绘图、输入、音频、存储、时钟的 trait 定义。
//!
//! 设计（`docs/style.md` §5.6）：
//! - 后端用**关联类型 + 泛型**注入，不用 `dyn`（热路径零虚调用）；
//! - 所有实现共享同一套 trait，因此 `HeadlessBackend`（测试）与
//!   `MacroquadBackend`（三端）可互换，回归测试不依赖窗口/声卡。

pub mod audio;
pub mod audio_asset;
pub mod backend;
pub mod error;
pub mod gfx;
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
pub use gfx::{FB_PIXELS, FramePresenter, Framebuf, UploadMode, Viewport};
pub use input::{HeadlessInput, VirtualButton, translate_event};
pub use settings::{
    Gesture, GestureTracker, KeyBindings, Orientation, ScaleMode, Settings, VirtualKeyPad,
};
pub use storage::{FileStore, MemoryStore, validate_key};
