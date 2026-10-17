//! 平台后端 trait 契约（`docs/style.md` §5.6）。
//!
//! 坐标约定：[`Event`] 中的坐标一律是 **176×220 逻辑坐标**，
//! 屏幕坐标 ↔ 逻辑坐标的换算由后端负责（`docs/parity.md` P4-2）。

use crate::error::Error;

/// 16 位 RGB565 颜色（原版 gx.dll 显示格式，见 `legacy/docs/facts/index.md` F-102）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgb565(pub u16);

/// 逻辑坐标系下的一个矩形。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// 左上角 x（逻辑像素）。
    pub x: i32,
    /// 左上角 y（逻辑像素）。
    pub y: i32,
    /// 宽度（逻辑像素）。
    pub w: u32,
    /// 高度（逻辑像素）。
    pub h: u32,
}

/// 原生键码（平台各自的编码，仅作透传，不做语义解释）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCode(pub u32);

/// 一枚音频通道句柄（由 [`Audio::play`] 分配，用于 [`Audio::stop`]/音量控制）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel(pub u32);

/// 统一输入事件。坐标为 176×220 逻辑坐标。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// 指针按下（触摸/鼠标按下）。
    PointerDown {
        /// 逻辑坐标 x。
        x: f32,
        /// 逻辑坐标 y。
        y: f32,
    },
    /// 指针抬起。
    PointerUp {
        /// 逻辑坐标 x。
        x: f32,
        /// 逻辑坐标 y。
        y: f32,
    },
    /// 指针移动（拖动）。
    PointerMove {
        /// 逻辑坐标 x。
        x: f32,
        /// 逻辑坐标 y。
        y: f32,
    },
    /// 按键按下。
    KeyDown(KeyCode),
    /// 按键抬起。
    KeyUp(KeyCode),
    /// 退出请求（窗口关闭等）。
    Quit,
}

/// 绘图后端：清屏、位块传输、呈现。
///
/// 原版是 176×220 的整屏位块绘制（gx.dll `GXBeginDraw`/`GXEndDraw`），
/// 因此这里只暴露软件光栅化的最小接口，缩放/上传纹理由实现内部处理。
pub trait Gfx {
    /// 用纯色清空逻辑帧缓冲。
    fn clear(&mut self, color: Rgb565);

    /// 把行主序 RGB565 像素（长度 = `dst.w * dst.h`）贴到 `dst`。
    ///
    /// # Panics
    ///
    /// 实现必须校验 `src.len() == (dst.w * dst.h) as usize`，
    /// 不满足时返回错误路径而非 panic（本仓库 `panic = deny`）。
    fn blit(&mut self, dst: Rect, src: &[u16]);

    /// 把逻辑帧缓冲呈现到屏幕（wasm/桌面对应 swap）。
    fn present(&mut self);
}

/// 输入后端：把平台事件翻译成 [`Event`] 流。
pub trait Input {
    /// 取出并清空本帧事件。
    fn poll(&mut self, out: &mut Vec<Event>);
}

/// 音频后端：通道式播放（原版 Hekkus Sound System 也是通道模型）。
pub trait Audio {
    /// 播放一枚声音素材，返回通道句柄。
    fn play(&mut self, id: u32, volume: u8) -> Channel;

    /// 停止指定通道。
    fn stop(&mut self, channel: Channel);

    /// 设置通道音量（0..=255）。
    fn set_volume(&mut self, channel: Channel, volume: u8);
}

/// 存储后端：桌面=文件系统，wasm=IndexedDB，Android=应用内部存储。
pub trait Storage {
    /// 读取一个命名存储项。
    ///
    /// # Errors
    ///
    /// 项不存在或底层 IO 失败时返回错误；调用方（存读档）负责区分。
    fn read(&mut self, key: &str) -> Result<Vec<u8>, Error>;

    /// 写入一个命名存储项。
    ///
    /// # Errors
    ///
    /// 底层 IO 失败时返回错误。
    fn write(&mut self, key: &str, value: &[u8]) -> Result<(), Error>;
}

/// 时钟后端：单调毫秒（原版 `GetTickCount` 语义）。
pub trait Clock {
    /// 自进程启动以来的毫秒数。
    fn now_ms(&mut self) -> u64;
}

/// 后端聚合：把五件套以关联类型暴露给组装层。
pub trait Backend {
    /// 绘图实现。
    type Gfx: Gfx;
    /// 输入实现。
    type Input: Input;
    /// 音频实现。
    type Audio: Audio;
    /// 存储实现。
    type Storage: Storage;
    /// 时钟实现。
    type Clock: Clock;

    /// 可变访问绘图后端。
    fn gfx(&mut self) -> &mut Self::Gfx;
    /// 可变访问输入后端。
    fn input(&mut self) -> &mut Self::Input;
    /// 可变访问音频后端。
    fn audio(&mut self) -> &mut Self::Audio;
    /// 可变访问存储后端。
    fn storage(&mut self) -> &mut Self::Storage;
    /// 可变访问时钟后端。
    fn clock(&mut self) -> &mut Self::Clock;
}

#[cfg(test)]
mod tests {
    use super::{Event, KeyCode, Rect, Rgb565};

    #[test]
    fn event_coordinates_are_f32_logical_space() {
        let ev = Event::PointerDown { x: 88.0, y: 110.0 };
        assert_eq!(ev, Event::PointerDown { x: 88.0, y: 110.0 });
        let key = Event::KeyDown(KeyCode(0x26));
        assert!(matches!(key, Event::KeyDown(_)));
    }

    #[test]
    fn rect_and_color_are_copy_types() {
        let r = Rect { x: 0, y: 0, w: 176, h: 220 };
        let c = Rgb565(0xF800);
        let (r2, c2) = (r, c);
        assert_eq!(r, r2);
        assert_eq!(c, c2);
    }
}
