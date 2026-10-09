//! 触屏输入层：**直接触摸界面元素**，不画虚拟按键/虚拟手柄。
//!
//! 理念（`docs/modernization.md` §5）：玩家手指点哪就是哪，不在画面上叠一层
//! 方向键按钮。映射规则：
//!
//! | 场景 | 点按元素 | 滑动 |
//! |---|---|---|
//! | 主菜单 | 点菜单行 = 选中该行并立即执行 | — |
//! | 选角/选图 | 点某行 = 选中该项并确认 | 下滑 = 返回 |
//! | 主地图 | 点格子 = 朝该格走；点自己所在格 = 触发据点事件 | 滑动 = 朝该方向走一步 |
//! | 战斗 | 点任意处 = 攻击一回合 | 下滑 = 退出 |
//! | 消息 | 左半屏 = 关闭；右半屏 = 下一页 | 上滑 = 下一页；下滑 = 上一页 |
//! | 存读档 | 点有档的槽 = 读档；点空槽 = 存档 | 下滑 = 关闭 |
//!
//! 坐标约定：本模块产出的坐标一律是 **176×220 逻辑坐标**（屏幕坐标由调用方用
//! [`Viewport::screen_to_logical`] 反变换后传入，见 [`TouchPad::feed_screen`]）。

use ti_platform::gfx::Viewport;
use ti_platform::settings::{Gesture, GestureTracker};

/// 一次完成的触摸：手势类型 + 释放点的逻辑坐标。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchSample {
    /// 识别出的手势（点按/四向滑动）。
    pub gesture: Gesture,
    /// 释放点逻辑 x（用于命中判定）。
    pub x: f32,
    /// 释放点逻辑 y（用于命中判定）。
    pub y: f32,
}

/// 触屏输入垫：把每帧指针采样转成逻辑坐标手势。
///
/// 后端只需每帧给出「当前指针位置 + 是否按下」，本垫负责反变换、手势识别与
/// 启用开关，产出 [`TouchSample`] 交给 `GameApp::poll_touch`。
#[derive(Debug, Clone)]
pub struct TouchPad {
    /// 手势识别器（滑动阈值 = 20 逻辑像素）。
    tracker: GestureTracker,
    /// 最近一次采样点的逻辑 x。
    x: f32,
    /// 最近一次采样点的逻辑 y。
    y: f32,
    /// 是否启用（设置项可关；关闭时 `feed*` 恒返回 `None`，不产生任何输入）。
    enabled: bool,
}

impl TouchPad {
    /// 新建触屏垫（默认阈值 20 逻辑像素，默认启用）。
    #[must_use]
    pub fn new() -> Self {
        Self { tracker: GestureTracker::default_threshold(), x: 0.0, y: 0.0, enabled: true }
    }

    /// 指定滑动阈值（逻辑像素）新建。
    #[must_use]
    pub fn with_threshold(swipe_threshold: f32) -> Self {
        Self { tracker: GestureTracker::new(swipe_threshold), x: 0.0, y: 0.0, enabled: true }
    }

    /// 启用/禁用触屏（禁用时鼠标点击不产生触摸手势）。
    pub fn set_enabled(&mut self, on: bool) {
        self.enabled = on;
    }

    /// 当前是否启用。
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 最近一次采样点的逻辑坐标。
    #[must_use]
    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    /// 喂一帧采样（**已是逻辑坐标**）；手势完成时返回 [`TouchSample`]。
    ///
    /// `is_down = true` 记录起点并返回 `None`；`is_down = false` 判定手势
    /// （点按/滑动）并返回带释放点坐标的样本。未启用时恒返回 `None`。
    pub fn feed(&mut self, lx: f32, ly: f32, is_down: bool) -> Option<TouchSample> {
        if !self.enabled {
            return None;
        }
        self.x = lx;
        self.y = ly;
        self.tracker.feed(lx, ly, is_down).map(|gesture| TouchSample { gesture, x: lx, y: ly })
    }

    /// 喂一帧采样（**屏幕坐标**）：内部按视口反变换到逻辑坐标（`docs/parity.md` P4-2）。
    ///
    /// 未启用时不做反变换直接返回 `None`（省一次运算）。
    pub fn feed_screen(
        &mut self,
        vp: &Viewport,
        sx: f32,
        sy: f32,
        is_down: bool,
    ) -> Option<TouchSample> {
        if !self.enabled {
            return None;
        }
        let (lx, ly) = vp.screen_to_logical(sx, sy);
        self.feed(lx, ly, is_down)
    }
}

impl Default for TouchPad {
    fn default() -> Self {
        Self::new()
    }
}

/// 触屏命中区布局（逻辑坐标，与 `GameApp::render` 各场景的绘制位置
/// **严格一致**；改了绘制坐标必须同步改这里）。
pub mod layout {
    /// 主菜单行首 y（= `MENU_TOP(96) - 12`，即选中高亮矩形顶边）。
    pub const MENU_ROW_Y0: f32 = 84.0;
    /// 主菜单行步进（= `ROW_H`）。
    pub const MENU_ROW_STEP: f32 = 20.0;
    /// 主菜单命中区左边界（= 高亮矩形 `x`）。
    pub const MENU_X0: f32 = 18.0;
    /// 主菜单命中区右边界（= 高亮矩形 `x + w`）。
    pub const MENU_X1: f32 = 158.0;
    /// 主菜单项数（`spec/menu.md §a` case 0..=5）。
    pub const MENU_COUNT: usize = 6;

    /// 选角/选图列表行首 y（= 绘制 `y - 10`）。
    pub const LIST_Y0: f32 = 40.0;
    /// 选角/选图列表行步进。
    pub const LIST_STEP: f32 = 30.0;
    /// 选角/选图命中区左边界。
    pub const LIST_X0: f32 = 20.0;
    /// 选角/选图命中区右边界（= `x + w`）。
    pub const LIST_X1: f32 = 156.0;

    /// 主地图网格原点 x（与渲染 `px = col * 6 + 20` 互逆）。
    pub const GRID_X0: f32 = 20.0;
    /// 主地图网格原点 y（与渲染 `py = row * 6 + 30` 互逆）。
    pub const GRID_Y0: f32 = 30.0;
    /// 主地图网格格距（逻辑像素）。
    pub const GRID_STEP: f32 = 6.0;

    /// 存读档槽位首 y（= 绘制 `y`）。
    pub const SLOT_Y0: f32 = 40.0;
    /// 存读档槽位步进。
    pub const SLOT_STEP: f32 = 40.0;
    /// 存读档槽位命中高度。
    pub const SLOT_H: f32 = 40.0;

    /// 消息界面左右分区 x（左 = 关闭，右 = 下一页）。
    pub const MSG_SPLIT_X: f32 = 88.0;

    /// 点按是否落在主菜单某行 → 返回菜单项索引。
    ///
    /// 命中区为整行宽（手指容错），行带高 = `MENU_ROW_STEP`。
    // allow 理由：`y < MENU_ROW_Y0` 已排除负值，屏高 220 限定上界，
    // `floor()` 后必 ∈ [0, 6]，故 f32→i32 转换不截断、不丢符号。
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn menu_row_at(x: f32, y: f32) -> Option<usize> {
        if !(MENU_X0..MENU_X1).contains(&x) || y < MENU_ROW_Y0 {
            return None;
        }
        let idx = ((y - MENU_ROW_Y0) / MENU_ROW_STEP).floor() as i32;
        let idx = usize::try_from(idx).ok()?;
        if idx < MENU_COUNT { Some(idx) } else { None }
    }

    /// 点按是否落在选角/选图列表某行 → 返回行索引（`< count`）。
    // allow 理由：同 [`menu_row_at`]，`y < LIST_Y0` 已排除负值。
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn list_index_at(x: f32, y: f32, count: usize) -> Option<usize> {
        if !(LIST_X0..LIST_X1).contains(&x) || y < LIST_Y0 {
            return None;
        }
        let idx = ((y - LIST_Y0) / LIST_STEP).floor() as i32;
        let idx = usize::try_from(idx).ok()?;
        if idx < count { Some(idx) } else { None }
    }

    /// 点按是否落在主地图某格 → 返回 `(row, col)`（越界 → `None`）。
    // allow 理由：`x < GRID_X0` / `y < GRID_Y0` 已排除负值，
    // `floor()` 后为非负数，f32→u32 转换不截断、不丢符号。
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn grid_cell_at(x: f32, y: f32, grid_w: u32, grid_h: u32) -> Option<(u32, u32)> {
        if x < GRID_X0 || y < GRID_Y0 {
            return None;
        }
        let col = ((x - GRID_X0) / GRID_STEP).floor() as u32;
        let row = ((y - GRID_Y0) / GRID_STEP).floor() as u32;
        if col < grid_w && row < grid_h { Some((row, col)) } else { None }
    }

    /// 点按是否落在存读档第 `slot` 个槽位（0 起）。
    // allow 理由：槽位下标 ≤ 4（`SaveManager` 默认 5 槽），远小于 f32 尾数 23 位。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn slot_at(x: f32, y: f32, slot: usize) -> bool {
        let step = slot as f32 * SLOT_STEP;
        (0.0..176.0).contains(&x) && (SLOT_Y0 + step..SLOT_Y0 + step + SLOT_H).contains(&y)
    }
}

#[cfg(test)]
mod tests {
    use super::layout::{
        GRID_STEP, GRID_X0, GRID_Y0, LIST_STEP, LIST_Y0, MENU_ROW_STEP, MENU_ROW_Y0, SLOT_H,
        SLOT_STEP, SLOT_Y0,
    };
    use super::{Gesture, TouchPad, TouchSample};
    use ti_platform::gfx::Viewport;

    /// 浮点近似相等（避免 `float_cmp` lint，容差 1e-3）。
    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn tap_recognized_within_threshold() {
        let mut pad = TouchPad::new();
        assert!(pad.feed(50.0, 90.0, true).is_none(), "按下时手势未完成");
        let got = pad.feed(52.0, 92.0, false);
        assert!(
            matches!(got, Some(TouchSample { gesture: Gesture::Tap, x, y })
                if near(x, 52.0) && near(y, 92.0)),
            "小幅移动应识别为点按 (52,92)，实际 {got:?}"
        );
    }

    #[test]
    fn swipe_direction_recognized() {
        let cases = [
            (50.0_f32, 90.0_f32, 50.0_f32, 30.0_f32, Gesture::SwipeUp),
            (50.0, 90.0, 50.0, 160.0, Gesture::SwipeDown),
            (50.0, 90.0, 10.0, 90.0, Gesture::SwipeLeft),
            (50.0, 90.0, 100.0, 90.0, Gesture::SwipeRight),
        ];
        for (sx, sy, ex, ey, want) in cases {
            let mut pad = TouchPad::new();
            let _ = pad.feed(sx, sy, true);
            let got = pad.feed(ex, ey, false);
            assert!(
                matches!(got, Some(TouchSample { gesture, .. }) if gesture == want),
                "({sx},{sy})→({ex},{ey}) 应识别为 {want:?}，实际 {got:?}"
            );
        }
    }

    #[test]
    fn disabled_pad_never_produces_input() {
        let mut pad = TouchPad::new();
        pad.set_enabled(false);
        assert!(!pad.is_enabled());
        assert!(pad.feed(10.0, 10.0, true).is_none());
        assert!(pad.feed(10.0, 10.0, false).is_none(), "禁用后不应有手势");
    }

    #[test]
    fn feed_screen_inverts_viewport() {
        // 视口 2x：屏幕 (176,220) → 逻辑 (88,110)
        let vp = Viewport::fit(352.0, 440.0);
        let mut pad = TouchPad::new();
        assert!(pad.feed_screen(&vp, 176.0, 220.0, true).is_none());
        let got = pad.feed_screen(&vp, 176.0, 220.0, false);
        assert!(
            matches!(got, Some(TouchSample { x, y, .. }) if near(x, 88.0) && near(y, 110.0)),
            "屏幕 (176,220) 应反变换为逻辑 (88,110)，实际 {got:?}"
        );
        let (px, py) = pad.pos();
        assert!(near(px, 88.0) && near(py, 110.0), "pad.pos() 应记住最近采样点");
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // 测试常量下标 ≤ 20，f32 表示无损
    fn menu_rows_cover_six_items() {
        for i in 0..6usize {
            let y = MENU_ROW_Y0 + i as f32 * MENU_ROW_STEP + 4.0;
            assert_eq!(super::layout::menu_row_at(88.0, y), Some(i), "row {i}");
        }
        assert_eq!(super::layout::menu_row_at(5.0, 90.0), None, "左边界外");
        assert_eq!(super::layout::menu_row_at(200.0, 90.0), None, "右边界外");
        assert_eq!(super::layout::menu_row_at(88.0, 10.0), None, "标题条区域");
        assert_eq!(
            super::layout::menu_row_at(88.0, MENU_ROW_Y0 + 6.0 * MENU_ROW_STEP),
            None,
            "超出 6 项"
        );
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // 测试下标 ≤ 5，f32 表示无损
    fn list_index_maps_five_roles() {
        for i in 0..5usize {
            let y = LIST_Y0 + i as f32 * LIST_STEP + 4.0;
            assert_eq!(super::layout::list_index_at(88.0, y, 5), Some(i), "item {i}");
        }
        assert_eq!(super::layout::list_index_at(88.0, LIST_Y0 + 5.0 * LIST_STEP + 4.0, 5), None);
        assert_eq!(super::layout::list_index_at(4.0, LIST_Y0 + 4.0, 5), None, "左边界外");
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // 测试坐标 ≤ 19，f32 表示无损
    fn grid_cell_inverts_render_formula() {
        // 渲染 px = col*6 + 20 / py = row*6 + 30 → 命中应还原 (row, col)
        for (row, col) in [(0u32, 0u32), (1, 2), (5, 7), (19, 19)] {
            let x = col as f32 * GRID_STEP + GRID_X0;
            let y = row as f32 * GRID_STEP + GRID_Y0;
            assert_eq!(super::layout::grid_cell_at(x, y, 20, 20), Some((row, col)));
        }
        assert_eq!(super::layout::grid_cell_at(5.0, 30.0, 20, 20), None, "网格左外");
        assert_eq!(super::layout::grid_cell_at(20.0, 10.0, 20, 20), None, "标题条区域");
        // 越界格 → None
        let x = 19.0 * GRID_STEP + GRID_X0;
        assert_eq!(super::layout::grid_cell_at(x, 30.0, 15, 15), None, "超出 15×15");
    }

    #[test]
    fn slot_bands_do_not_overlap() {
        assert!(super::layout::slot_at(10.0, SLOT_Y0 + 2.0, 0));
        assert!(super::layout::slot_at(170.0, SLOT_Y0 + SLOT_H - 1.0, 0));
        assert!(!super::layout::slot_at(10.0, SLOT_Y0 + SLOT_H + 1.0, 0), "槽 0 下沿外");
        assert!(super::layout::slot_at(10.0, SLOT_Y0 + SLOT_STEP + 2.0, 1));
        assert!(!super::layout::slot_at(200.0, SLOT_Y0 + 2.0, 0), "右侧边界外");
    }
}
