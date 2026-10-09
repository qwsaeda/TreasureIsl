//! 用户可配置项（S4.1）：缩放模式、横竖屏、虚拟方向键、手势识别、按键自定义。
//!
//! 设计原则：
//! - 所有配置均为纯数据结构，不依赖后端；
//! - [`GestureTracker`] 把事件流折叠为高层手势，供上层场景直接消费；
//! - 默认值来自原版 `WinCE` 触摸设备的行为（`docs/facts/index.md`）。

use crate::backend::KeyCode;
use crate::gfx::Viewport;
use std::collections::BTreeMap;

/// 缩放模式（决定屏幕利用率与像素保真度的取舍，见 `docs/modernization.md` §3）。
///
/// 176×220 是 4:5 竖屏内容，在 1080×2400 手机上等比铺满只有 **56%** 屏幕利用率；
/// 下面 4 种模式覆盖「像素完美但留黑边」到「铺满但变形/裁剪」的连续取舍。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaleMode {
    /// 等比铺满（最小缩放比），溢出留黑边。像素方形，176×220 内容利用率最高。
    #[default]
    Smooth,
    /// 整数倍等比（向下取整，≥1×）。像素锐利不糊，但黑边比 `Smooth` 略多。
    Integer,
    /// 非等比拉伸铺满。利用率 100%，代价是像素非方形（画面轻微变形）。
    Stretch,
    /// 等比放大铺满 + 对称裁剪。利用率 100% 且像素方形，代价是裁掉溢出内容
    /// （4:5 内容在 20:9 屏上会裁掉约 44% 宽度）。
    ZoomCrop,
}

impl ScaleMode {
    /// 全部模式（设置菜单枚举用）。
    pub const ALL: [Self; 4] = [Self::Smooth, Self::Integer, Self::Stretch, Self::ZoomCrop];

    /// 可读标签（设置菜单显示）。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "等比",
            Self::Integer => "整数倍",
            Self::Stretch => "拉伸",
            Self::ZoomCrop => "裁剪",
        }
    }

    /// 本模式是否保持像素方形（等比）。
    #[must_use]
    pub fn keeps_square_pixels(self) -> bool {
        matches!(self, Self::Smooth | Self::Integer | Self::ZoomCrop)
    }

    /// 本模式是否铺满屏幕（无黑边）。
    #[must_use]
    pub fn fills_screen(self) -> bool {
        matches!(self, Self::Stretch | Self::ZoomCrop)
    }

    /// 按模式计算缩放比（等比模式有意义；非等比模式由 [`Viewport::with_mode`] 处理）。
    ///
    /// `raw` 是等比 fit 算出的连续缩放比；`Integer` 向下取整且不小于 1.0。
    #[must_use]
    pub fn apply(self, raw: f32) -> f32 {
        match self {
            Self::Integer => raw.floor().max(1.0),
            Self::Smooth | Self::Stretch | Self::ZoomCrop => raw,
        }
    }
}

/// 屏幕方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// 竖屏（原版 176×220，宽 < 高）。
    #[default]
    Portrait,
    /// 横屏（宽 > 高）。
    Landscape,
}

impl Orientation {
    /// 返回该方向下逻辑宽高（竖屏 176×220，横屏 220×176）。
    #[must_use]
    pub fn logical_size(self) -> (f32, f32) {
        match self {
            Self::Portrait => (176.0, 220.0),
            Self::Landscape => (220.0, 176.0),
        }
    }
}

/// 手势类型（由 [`GestureTracker`] 产出）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    /// 单点快速触碰（位移 < 阈值）。
    Tap,
    /// 滑动（位移 ≥ 阈值，方向由起点→终点决定）。
    SwipeUp,
    /// 向右滑动。
    SwipeRight,
    /// 向左滑动。
    SwipeLeft,
    /// 向下滑动。
    SwipeDown,
}

/// 手势识别器：累积指针事件，`PointerUp` 时产出 [`Gesture`]。
///
/// 逻辑坐标；`swipe_threshold` 在构造时设定（单位：逻辑像素）。
#[derive(Debug, Clone)]
pub struct GestureTracker {
    /// 滑动判定的最小位移（逻辑像素）。
    pub swipe_threshold: f32,
    down: Option<(f32, f32)>,
}

impl GestureTracker {
    /// 新建追踪器，滑动阈值 = `swipe_threshold`（逻辑像素，默认 20.0）。
    #[must_use]
    pub fn new(swipe_threshold: f32) -> Self {
        Self { swipe_threshold, down: None }
    }

    /// 默认追踪器（`swipe_threshold = 20.0`）。
    #[must_use]
    pub fn default_threshold() -> Self {
        Self::new(20.0)
    }

    /// 处理一个指针事件，返回识别出的手势（`None` = 尚未完成）。
    ///
    /// `PointerDown` 记录起点；`PointerUp` 计算位移并返回手势。
    /// 中间 `PointerMove` 事件直接忽略（起点/终点已足够）。
    #[must_use]
    pub fn feed(&mut self, x: f32, y: f32, is_down: bool) -> Option<Gesture> {
        if is_down {
            self.down = Some((x, y));
            return None;
        }
        let (sx, sy) = self.down.take()?;
        let dx = x - sx;
        let dy = y - sy;
        if dx.abs() < self.swipe_threshold && dy.abs() < self.swipe_threshold {
            Some(Gesture::Tap)
        } else if dx.abs() >= dy.abs() {
            Some(if dx > 0.0 { Gesture::SwipeRight } else { Gesture::SwipeLeft })
        } else {
            Some(if dy > 0.0 { Gesture::SwipeDown } else { Gesture::SwipeUp })
        }
    }
}

impl Default for GestureTracker {
    fn default() -> Self {
        Self::default_threshold()
    }
}

/// 虚拟方向键：4 个逻辑坐标矩形，触摸时识别对应方向。
///
/// 默认布局（176×220 逻辑坐标）：
/// - 上：`(68, 152, 40, 24)`
/// - 下：`(68, 192, 40, 24)`
/// - 左：`(24, 172, 32, 24)`
/// - 右：`(120, 172, 32, 24)`
#[derive(Debug, Clone)]
pub struct VirtualKeyPad {
    /// 各方向按钮矩形（逻辑坐标）。
    pub up: [f32; 4],
    /// 下方向按钮 `[x, y, w, h]`。
    pub down: [f32; 4],
    /// 左方向按钮 `[x, y, w, h]`。
    pub left: [f32; 4],
    /// 右方向按钮 `[x, y, w, h]`。
    pub right: [f32; 4],
}

impl VirtualKeyPad {
    /// 默认布局。
    #[must_use]
    pub fn default_layout() -> Self {
        Self {
            up: [68.0, 152.0, 40.0, 24.0],
            down: [68.0, 192.0, 40.0, 24.0],
            left: [24.0, 172.0, 32.0, 24.0],
            right: [120.0, 172.0, 32.0, 24.0],
        }
    }

    /// 判定逻辑坐标 `(x, y)` 命中哪个方向；未命中返回 `None`。
    #[must_use]
    pub fn hit(&self, x: f32, y: f32) -> Option<&'static str> {
        if Self::in_rect(x, y, &self.up) {
            Some("up")
        } else if Self::in_rect(x, y, &self.down) {
            Some("down")
        } else if Self::in_rect(x, y, &self.left) {
            Some("left")
        } else if Self::in_rect(x, y, &self.right) {
            Some("right")
        } else {
            None
        }
    }

    fn in_rect(x: f32, y: f32, r: &[f32; 4]) -> bool {
        x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3]
    }
}

impl Default for VirtualKeyPad {
    fn default() -> Self {
        Self::default_layout()
    }
}

/// 按键自定义映射：动作名 → 原生键码。
///
/// 使用 `BTreeMap` 保证序列化/迭代顺序稳定。
#[derive(Debug, Clone, Default)]
pub struct KeyBindings {
    map: BTreeMap<&'static str, KeyCode>,
}

impl KeyBindings {
    /// 默认绑定（原版键位）：`up/down/left/right/confirm/cancel`。
    ///
    /// 键码值为 macroquad `KeyCode` 的 u32 数值（见 `macroquad::KeyCode` 文档），
    /// 平台层只做透传，不做语义解释。
    #[must_use]
    pub fn defaults() -> Self {
        let mut b = Self { map: BTreeMap::new() };
        b.set("up", KeyCode(0x26)); // Up
        b.set("down", KeyCode(0x28)); // Down
        b.set("left", KeyCode(0x25)); // Left
        b.set("right", KeyCode(0x27)); // Right
        b.set("confirm", KeyCode(0x0D)); // Enter
        b.set("cancel", KeyCode(0x1B)); // Escape
        b
    }

    /// 绑定动作 `action` 到 `code`。
    pub fn set(&mut self, action: &'static str, code: KeyCode) {
        self.map.insert(action, code);
    }

    /// 查询动作 `action` 对应的键码；未绑定返回 `None`。
    #[must_use]
    pub fn get(&self, action: &str) -> Option<KeyCode> {
        self.map.get(action).copied()
    }

    /// 绑定对数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// 上采样滤波模式（176×220 放大到现代屏幕时的纹理采样方式）。
///
/// 原版是低分辨率硬像素，**最近邻**能保住像素锐利感；但 4x 以上放大时
/// 锯齿明显，`Linear`（双线性）更平滑。两者可运行时切换（S6.1）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    /// 最近邻（像素锐利，保住原版点阵风格；像素游戏的默认选择）。
    #[default]
    Nearest,
    /// 双线性线性插值（平滑，高倍率下锯齿更少）。
    Linear,
}

impl FilterMode {
    /// 全部模式（设置菜单枚举用）。
    pub const ALL: [Self; 2] = [Self::Nearest, Self::Linear];

    /// 可读标签（设置菜单显示）。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Nearest => "像素锐利",
            Self::Linear => "平滑",
        }
    }

    /// 取下一个模式（循环，`ALL` 顺序）。
    #[must_use]
    pub fn next(self) -> Self {
        let all = Self::ALL;
        let i = all.iter().position(|&m| m == self).unwrap_or(0);
        all[(i + 1) % all.len()]
    }

    /// 对应 miniquad 的 `FilterMode`（经 `macroquad::miniquad` 再导出）。
    #[must_use]
    pub fn to_miniquad(self) -> macroquad::miniquad::FilterMode {
        match self {
            Self::Nearest => macroquad::miniquad::FilterMode::Nearest,
            Self::Linear => macroquad::miniquad::FilterMode::Linear,
        }
    }
}

/// 边框装饰样式（S6.2）：letterbox / pillarbox 黑边的填充方式。
///
/// 176×220 是 4:5 竖屏内容，在 20:9 屏上等比铺满只有 36~78% 利用率，
/// 剩余区域此前是纯黑（GPU clear 色）。本枚举让边框不只是「不刺眼的黑」，
/// 而是与游戏调色板一致的主题色，减少大面积死黑的突兀感。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyle {
    /// 纯黑（原版 letterbox 观感，零成本）。
    Black,
    /// 深蓝（与游戏主背景 `0x0010` 同色系，弱化黑边突兀感）。默认。
    #[default]
    DeepBlue,
    /// 与主画面同色（视觉上边框「消失」，画面居中悬浮）。
    MatchBackdrop,
}

impl BorderStyle {
    /// 全部样式（设置菜单枚举用）。
    pub const ALL: [Self; 3] = [Self::Black, Self::DeepBlue, Self::MatchBackdrop];

    /// 可读标签（设置菜单显示）。
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Black => "纯黑",
            Self::DeepBlue => "深蓝",
            Self::MatchBackdrop => "同背景",
        }
    }

    /// 取下一个样式（循环，`ALL` 顺序）。
    #[must_use]
    pub fn next(self) -> Self {
        let all = Self::ALL;
        let i = all.iter().position(|&m| m == self).unwrap_or(0);
        all[(i + 1) % all.len()]
    }

    /// 本样式对应的 RGB565 色值。
    ///
    /// - `Black` → `0x0000`
    /// - `DeepBlue` → `0x0010`（与 `GameApp::render_*` 的 `fb.clear(0x0010)` 一致）
    /// - `MatchBackdrop` → `0x0000`（由调用方按当前帧缓冲左上角像素替换，见
    ///   [`BorderStyle::resolve`]）
    #[must_use]
    pub fn rgb565(self) -> u16 {
        match self {
            Self::Black | Self::MatchBackdrop => 0x0000,
            Self::DeepBlue => 0x0010,
        }
    }

    /// 按当前帧缓冲首像素解析实际填充色。
    ///
    /// `MatchBackdrop` 取帧缓冲第 0 像素（场景背景色），其余用 [`Self::rgb565`]。
    #[must_use]
    pub fn resolve(self, fb_first_pixel: u16) -> u16 {
        match self {
            Self::MatchBackdrop => fb_first_pixel,
            Self::Black | Self::DeepBlue => self.rgb565(),
        }
    }
}

/// 用户全部可配置项（S4.1）。
#[derive(Debug, Clone)]
pub struct Settings {
    /// 缩放模式。
    pub scale_mode: ScaleMode,
    /// 上采样滤波（最近邻/平滑，S6.1）。
    pub filter_mode: FilterMode,
    /// letterbox / pillarbox 边框装饰样式（S6.2）。
    pub border_style: BorderStyle,
    /// 屏幕方向（竖屏/横屏）。
    pub orientation: Orientation,
    /// 是否启用虚拟方向键（触摸屏时自动开启）。
    pub virtual_keypad: bool,
    /// 虚拟方向键布局。
    pub keypad: VirtualKeyPad,
    /// 按键自定义映射。
    pub key_bindings: KeyBindings,
}

impl Settings {
    /// 默认设置（`Smooth` 缩放、`Portrait` 方向、虚拟键开、默认按键）。
    #[must_use]
    pub fn default_settings() -> Self {
        Self {
            scale_mode: ScaleMode::default(),
            filter_mode: FilterMode::default(),
            border_style: BorderStyle::default(),
            orientation: Orientation::default(),
            virtual_keypad: true,
            keypad: VirtualKeyPad::default(),
            key_bindings: KeyBindings::defaults(),
        }
    }

    /// 按当前 [`scale_mode`](Self::scale_mode) 计算视口缩放比。
    ///
    /// `raw_scale` 是连续 fit 比例（同 `Viewport::fit` 的结果）。
    #[must_use]
    pub fn effective_scale(&self, raw_scale: f32) -> f32 {
        self.scale_mode.apply(raw_scale)
    }

    /// 用本设置构造一个视口。
    ///
    /// `screen_w/screen_h` 为当前窗口尺寸（像素），
    /// 逻辑分辨率取 [`orientation::logical_size`](Orientation::logical_size)。
    /// 缩放模式由 [`scale_mode`](Self::scale_mode) 决定（拉伸/裁剪会突破等比）。
    #[must_use]
    pub fn viewport(&self, screen_w: f32, screen_h: f32) -> Viewport {
        let (lw, lh) = self.orientation.logical_size();
        let integer = self.scale_mode == ScaleMode::Integer;
        if self.scale_mode == ScaleMode::Stretch || self.scale_mode == ScaleMode::ZoomCrop {
            // 拉伸/裁剪固定作用于 176×220 原版逻辑尺寸
            return Viewport::with_mode(screen_w, screen_h, self.scale_mode, integer);
        }
        let raw = (screen_w / lw).min(screen_h / lh);
        let scale = self.effective_scale(raw);
        let offset_x = (screen_w - lw * scale) * 0.5;
        let offset_y = (screen_h - lh * scale) * 0.5;
        Viewport::from_scale(scale, offset_x, offset_y, lw, lh)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::default_settings()
    }
}
