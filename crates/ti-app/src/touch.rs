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
///
/// 列表类场景（`select_role` / `select_map` / `saveload`）的命中带**不再用常量**，
/// 改由 [`scene_list_bands`] / [`slot_bands`] 从 `ListLayout`（scenes.json）推导，
/// 与渲染 `item_rect` 公式正逆成对（EXP-071）。
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

    /// 主地图网格原点 x（与渲染 `px = col * 6 + 20` 互逆）。
    pub const GRID_X0: f32 = 20.0;
    /// 主地图网格原点 y（与渲染 `py = row * 6 + 30` 互逆）。
    pub const GRID_Y0: f32 = 30.0;
    /// 主地图网格格距（逻辑像素）。
    pub const GRID_STEP: f32 = 6.0;

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

    /// 场景列表命中带几何（渲染 `item_rect` 公式的浮点镜像，EXP-071/EXP-074）。
    ///
    /// `select_role` / `select_map` / `saveload` 等列表场景的触摸命中与渲染高亮
    /// 共用此公式（正逆成对），不再各自维护常量（EXP-048/060）。
    /// 命中带 = 文字基线 y 上移 `offset` 像素、高 `item_h`（与 `ListLayout::item_rect` 同公式）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn scene_list_bands(
        x0: f32,
        x1: f32,
        y0: f32,
        step: f32,
        item_h: f32,
        offset: f32,
        count: usize,
    ) -> Vec<RectF> {
        (0..count)
            .map(|i| {
                let y = y0 + i as f32 * step - offset;
                RectF { x0, x1, y0: y, y1: y + item_h }
            })
            .collect()
    }

    /// 点按落在哪条场景列表命中带（[`scene_list_bands`] 几何）→ 返回行索引。
    /// 与渲染 `item_rect` 公式严格互逆（EXP-071 正逆成对）。
    #[must_use]
    pub fn scene_list_index_at(bands: &[RectF], x: f32, y: f32) -> Option<usize> {
        bands.iter().position(|b| (b.x0..b.x1).contains(&x) && (b.y0..b.y1).contains(&y))
    }

    /// 场景列表命中带（逻辑坐标矩形，`y0 ≤ y < y1` 半开区间）。
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct RectF {
        /// 左边界（含）。
        pub x0: f32,
        /// 右边界（不含）。
        pub x1: f32,
        /// 上边界（含）。
        pub y0: f32,
        /// 下边界（不含）。
        pub y1: f32,
    }

    /// 存读档槽位命中带几何（渲染 `item_rect` 公式的浮点镜像，EXP-071/EXP-074）。
    ///
    /// 与 `saveload` 场景渲染共用同一 `ListLayout` 参数（正逆成对）；
    /// 命中区为整屏宽（`0..176`），行带 = 基线 y 上移 `offset`、高 `item_h`。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn slot_bands(y0: f32, step: f32, item_h: f32, offset: f32, count: usize) -> Vec<RectF> {
        (0..count)
            .map(|i| {
                let y = y0 + i as f32 * step - offset;
                RectF { x0: 0.0, x1: 176.0, y0: y, y1: y + item_h }
            })
            .collect()
    }

    /// 点按落在哪条存读档槽位命中带（[`slot_bands`] 几何）→ 返回槽位索引。
    /// 与渲染 `item_rect` 公式严格互逆（EXP-071 正逆成对）。
    #[must_use]
    pub fn slot_band_index_at(bands: &[RectF], x: f32, y: f32) -> Option<usize> {
        bands.iter().position(|b| (b.x0..b.x1).contains(&x) && (b.y0..b.y1).contains(&y))
    }

    /// 点按是否落在主地图某格（**占位 6×6 网格**）→ 返回 `(row, col)`（越界 → `None`）。
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

    /// 点按是否落在主地图某格（**等距投影**，与 `TerrainAtlas` 渲染同坐标系）。
    ///
    /// 坐标系契约（EXP-067，与 `terrain_atlas::blit_map_cam` + `center_camera` 严格互逆）：
    /// - 格点（菱形交点）：`iso_x = 32×(row+col)`、`iso_y = 16×(row+grid_w-1-col)`（`GridPos::iso_pos`）。
    /// - 渲染侧瓦片以**格点为中心锚点**向左上扩展（`blit_map_cam` 的 `- w/2, - h/2`），
    ///   屏幕像素 `(sx, sy) = (iso_x + cam.0, iso_y + cam.1)`；
    ///   `camera = center_camera(terrain)` 把 iso 包围盒中心对到屏幕中心 (88,110)，**通常为负**。
    /// - 触摸逆变换（`GridPos::iso_pos` 的严格逆，与 camera 取值无关）：
    ///   `u = (x-cam.0)/32 = row+col`，`v = (y-cam.1)/16 = row+grid_w-1-col`
    ///   → `row = round((u+v-(grid_w-1))/2)`，`col = round((u-v+(grid_w-1))/2)`，clamp 到 `[0, grid)`。
    /// - 命中区 = 每格 Voronoi 菱形（四邻格点围成），与瓦片尺寸无关、四象限互斥无空洞。
    ///
    /// 旧实现 bug（EXP-067 根因）：`px = x - cam` 把 camera 当平移量却混入了错误的归一项
    /// （`half = (grid_w-1-0.5)/2` 未对齐 `iso_y` 的 `grid_w-1` 语义），且 `camera=(0,0)` 测试
    /// 掩盖了负 camera 下的系统性推偏 → 主地图点格命中错位、界面错乱。
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::manual_midpoint,
        clippy::cast_sign_loss
    )]
    pub fn iso_grid_cell_at(
        x: f32,
        y: f32,
        grid_w: u32,
        grid_h: u32,
        camera: (i64, i64),
    ) -> Option<(u32, u32)> {
        if grid_w == 0 || grid_h == 0 {
            return None;
        }
        // 屏幕 → 等距画布（camera 是「画布原点→屏幕」偏移，center_camera 产负值，减之还原）
        let px = x - camera.0 as f32;
        let py = y - camera.1 as f32;
        // 严格逆 `GridPos::iso_pos`（iso_x = 32(row+col), iso_y = 16(row+W-1-col)）：
        //   u = px/32 = row+col，v = py/16 = row+W-1-col
        //   row = (u+v-(W-1))/2，col = (u-v+(W-1))/2
        // 命中区 = 每格 Voronoi 菱形（四邻格点围成），round() 归最近格，clamp 到 [0, grid)
        let w1 = grid_w.saturating_sub(1) as f32;
        let u = px / 32.0;
        let v = py / 16.0;
        let row = ((u + v - w1) / 2.0).round().clamp(0.0, grid_h.saturating_sub(1) as f32);
        let col = ((u - v + w1) / 2.0).round().clamp(0.0, grid_w.saturating_sub(1) as f32);
        // clamp 保证 row/col ∈ [0, grid)，f32→u32 不丢符号、不截断
        Some((row as u32, col as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::layout::{GRID_STEP, GRID_X0, GRID_Y0, MENU_ROW_STEP, MENU_ROW_Y0};
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
    #[allow(clippy::cast_precision_loss)]
    fn scene_list_bands_match_item_rect() {
        // EXP-071 正逆成对：渲染 `ListLayout::item_rect(i)` 的中心点必须命中
        // `scene_list_index_at` 的第 i 条命中带（select_role / select_map 同公式）。
        use ti_core::data::ListLayout;
        let ll = ListLayout {
            x: 20,
            y: 50,
            row_h: 30,
            item_h: 26,
            text_x: 60,
            w: 136,
            highlight_offset: 10,
            gold_x: 0,
        };
        let bands = super::layout::scene_list_bands(20.0, 20.0 + 136.0, 50.0, 30.0, 26.0, 10.0, 5);
        for i in 0..5usize {
            let r = ll.item_rect(i);
            let cx = (r.x + r.w / 2) as f32;
            let cy = (r.y + r.h / 2) as f32;
            assert_eq!(super::layout::scene_list_index_at(&bands, cx, cy), Some(i), "item {i}");
        }
        // 行带之外 → None
        assert_eq!(super::layout::scene_list_index_at(&bands, 88.0, 5.0), None, "上沿外");
        assert_eq!(
            super::layout::scene_list_index_at(&bands, 88.0, 50.0 + 5.0 * 30.0),
            None,
            "超出行数"
        );
    }

    #[test]
    #[allow(clippy::cast_precision_loss)]
    fn slot_bands_match_item_rect() {
        // EXP-071 正逆成对：saveload 槽位渲染 `item_rect` 中心点必须命中 `slot_bands`。
        use ti_core::data::ListLayout;
        let ll = ListLayout {
            x: 10,
            y: 40,
            row_h: 40,
            item_h: 26,
            text_x: 10,
            w: 166,
            highlight_offset: 10,
            gold_x: 100,
        };
        let bands = super::layout::slot_bands(40.0, 40.0, 26.0, 10.0, 5);
        for i in 0..5usize {
            let r = ll.item_rect(i);
            let cx = 88.0;
            let cy = (r.y + r.h / 2) as f32;
            assert_eq!(super::layout::slot_band_index_at(&bands, cx, cy), Some(i), "slot {i}");
        }
        assert_eq!(super::layout::slot_band_index_at(&bands, 88.0, 5.0), None, "上沿外");
    }
}
