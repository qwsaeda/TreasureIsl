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
//! 坐标约定：本模块产出的坐标一律是 **1080×1920 呈现层坐标**（= `menu_shell::LOGICAL_W_I32/H_I32`
//! 单一权威呈现坐标系，`08_render_arch.md` 契约 1）；屏幕坐标由调用方用
//! [`Viewport::screen_to_logical`] 反变换后传入（见 [`TouchPad::feed_screen`]）。

use ti_core::data;
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
    use super::data;

    /// 主菜单项数（`spec/menu.md §a` case 0..=5）。
    pub const MENU_COUNT: usize = 6;

    /// 消息界面左右分区 x（呈现层宽中心 540 = 1080/2；左 = 关闭，右 = 下一页）。
    /// S15.6 QA024：旧值 88 是 176 时代半屏（176/2），1080 空间 92% 宽度皆误判「下一页」。
    #[allow(clippy::cast_precision_loss)]
    pub const MSG_SPLIT_X: f32 = (ti_render::framebuffer::LOGICAL_W_I32 / 2) as f32;

    /// 点按是否落在主菜单某行 → 返回菜单项索引。
    ///
    /// 命中区几何读 `scenes.json` 的 `index` 段（`list.*` 与渲染 `item_rect`
    /// 正逆成对，EXP-071），`MENU_COUNT` 常量兜底项数。
    /// S15.6 QA101：带形 = `scene_list_bands` 同式（卡 i 占「顶 y0 + i*step - offset、
    /// 高 `item_h`」纵带，与 `ListLayout::item_rect` 正逆成对，同一读点 `scenes.json`）。
    /// 旧式（按基线 y0 起 floor 分带）漏减 `highlight_offset`：每卡顶 40px 带为
    /// 触摸死区、卡 1 顶带误选菜单 0；卡间空档（gap）= 无命中（点空白无副作用）。
    /// `scenes.json` 不可用 → `None`（触摸无副作用）。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn menu_row_at(x: f32, y: f32) -> Option<usize> {
        let scene = data::scenes().ok()?.get("index")?.clone();
        let l = &scene.list;
        let bands = scene_list_bands(
            l.x as f32,
            l.x as f32 + l.w as f32,
            l.y as f32,
            l.row_h as f32,
            l.item_h as f32,
            l.highlight_offset as f32,
            MENU_COUNT,
        );
        scene_list_index_at(&bands, x, y)
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
    /// 命中区 x 范围由调用方传 `x0`/`x1`（渲染列表 `list.x`/`list.x+list.w`），
    /// 行带 = 基线 y 上移 `offset`、高 `item_h`。
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    #[inline]
    pub fn slot_bands(
        x0: f32,
        x1: f32,
        y0: f32,
        step: f32,
        item_h: f32,
        offset: f32,
        count: usize,
    ) -> Vec<RectF> {
        scene_list_bands(x0, x1, y0, step, item_h, offset, count)
    }

    /// 点按落在哪条存读档槽位命中带（[`slot_bands`] 几何）→ 返回槽位索引。
    /// 与渲染 `item_rect` 公式严格互逆（EXP-071 正逆成对）。
    #[inline]
    #[must_use]
    pub fn slot_band_index_at(bands: &[RectF], x: f32, y: f32) -> Option<usize> {
        scene_list_index_at(bands, x, y)
    }

    /// 点按是否落在主地图某格（`chrome::iso_cell` 的逆，EXP-067 正逆成对）。
    ///
    /// 几何派生自 `scenes.json` 的 `main_map.grid.{x0,y0,tile_w,tile_h}` 段
    /// （`chrome::scene_layout("main_map").extra_scalar`，与渲染 `iso_cell`/`iso_tile`
    /// 同一数据源，契约 4）；触摸入参为 1080 呈现坐标。旧 176 时代占位公式
    /// （`GRID_X0`=20 / `GRID_Y0`=30 / `GRID_STEP`=6，6×6 格）已删除——S15.6 QA022：
    /// 渲染棋盘走 scenes.json 网格（x0=540/y0=520/tw=25/th=13，S15.6 QB105 重锚），
    /// 占位公式使整个可见棋盘的点格命中全部失效。越出棋盘 → `None`（触摸无副作用）。
    #[must_use]
    pub fn grid_cell_at(x: f32, y: f32, grid_w: u32, grid_h: u32) -> Option<(u32, u32)> {
        let layout = crate::scenes::chrome::scene_layout("main_map");
        iso_grid_cell_at(x, y, grid_w, grid_h, &layout)
    }

    /// 点按是否落在主地图某格（**等距投影**，`chrome::iso_cell` 的严格逆，EXP-067 正逆成对）。
    ///
    /// 坐标系契约（S15.6 QA023 重锚：渲染 `chrome::iso_cell` 与触摸共享 `scenes.json`
    /// `grid.*` 数据源，**无 legacy 32/16 投影、无 (88,110) 摄像机**——legacy 像素层
    /// 退出运行时的直接后果，`08_render_arch.md` 契约 1）：
    /// - 渲染正变换（`chrome::iso_cell`）：`sx = x0 + (col - row) * tw`、`sy = y0 + (row + col) * th`
    ///   （`x0/y0/tw/th` = `layout.extra_scalar` 的 `grid.x0/y0/tile_w/tile_h`）。
    /// - 触摸逆变换（严格逆，round-trip `iso_cell(iso_grid_cell_at(p)) == p` 对每个格点成立）：
    ///   `u = (x - x0) / tw = col - row`，`v = (y - y0) / th = row + col`
    ///   → `row = round((v - u) / 2)`，`col = round((v + u) / 2)`。
    /// - 命中区 = 每格最近格点（Voronoi 菱形）；越出 `[0, grid_w) × [0, grid_h)` → `None`
    ///   （棋盘外的黑屏点按不触发行走）。
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::manual_midpoint,
        clippy::cast_sign_loss,
        clippy::many_single_char_names
    )]
    pub fn iso_grid_cell_at(
        x: f32,
        y: f32,
        grid_w: u32,
        grid_h: u32,
        layout: &data::SceneLayout,
    ) -> Option<(u32, u32)> {
        if grid_w == 0 || grid_h == 0 {
            return None;
        }
        // S17.8：几何读 `chrome::board_fit(layout, grid_w)`（v3 世界画布 G 依赖拟合，
        // 缺席回退 v2 静态 `grid.*`）——与渲染 `iso_cell_fit` 同一数据源，正逆成对不变。
        let bg = crate::scenes::chrome::board_fit(layout, grid_w);
        let x0 = bg.x0 as f32;
        let y0 = bg.y0 as f32;
        let tw = bg.tw as f32;
        let th = bg.th as f32;
        if tw <= 0.0 || th <= 0.0 {
            return None;
        }
        // 严格逆 `chrome::iso_cell`（`sx = x0 + (col-row)*tw` / `sy = y0 + (row+col)*th`）：
        // u = col - row，v = row + col → row = (v-u)/2，col = (v+u)/2，round 归最近格点。
        let u = (x - x0) / tw;
        let v = (y - y0) / th;
        let row = ((v - u) / 2.0).round();
        let col = ((v + u) / 2.0).round();
        if row < 0.0 || col < 0.0 {
            return None;
        }
        // round 后非负，f32→u32 不丢符号、不截断
        let row_u = row as u32;
        let col_u = col as u32;
        if row_u < grid_h && col_u < grid_w { Some((row_u, col_u)) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::data;
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
        // 视口 2x：屏幕 (1080,1920) → 逻辑 (540,960)
        let vp = Viewport::fit(2160.0, 3840.0);
        let mut pad = TouchPad::new();
        assert!(pad.feed_screen(&vp, 1080.0, 1920.0, true).is_none());
        let got = pad.feed_screen(&vp, 1080.0, 1920.0, false);
        assert!(
            matches!(got, Some(TouchSample { x, y, .. }) if near(x, 540.0) && near(y, 960.0)),
            "屏幕 (1080,1920) 应反变换为逻辑 (540,960)，实际 {got:?}"
        );
        let (px, py) = pad.pos();
        assert!(near(px, 540.0) && near(py, 960.0), "pad.pos() 应记住最近采样点");
    }

    #[test]
    #[allow(clippy::cast_precision_loss)] // 测试常量下标 ≤ 20，f32 表示无损
    fn menu_rows_cover_six_items() {
        // 坐标读 scenes.json index 段（list.y/row_h/item_h/highlight_offset），与渲染 item_rect 正逆成对
        let scene = match data::scenes() {
            Ok(v) => match v.get("index") {
                Some(v) => v,
                None => std::process::abort(),
            },
            Err(e) => {
                eprintln!("scenes.json 不可用: {e}");
                std::process::abort()
            }
        };
        let y0 = scene.list.y as f32;
        let step = scene.list.row_h as f32;
        let offset = scene.list.highlight_offset as f32;
        let item_h = scene.list.item_h as f32;
        for i in 0..6usize {
            let top = y0 + i as f32 * step - offset;
            // 卡 i 顶带（含旧死区的卡顶 40px 带）+ 卡底 → 命中卡 i
            assert_eq!(super::layout::menu_row_at(540.0, top + 4.0), Some(i), "row {i} 卡顶带");
            assert_eq!(
                super::layout::menu_row_at(540.0, top + item_h - 4.0),
                Some(i),
                "row {i} 卡底"
            );
            // 卡间空档（gap）→ 无命中（点在空白，触摸无副作用；QA101 带形）
            if i + 1 < 6 {
                let gap_mid = top + item_h + (step - item_h) / 2.0;
                assert_eq!(
                    super::layout::menu_row_at(540.0, gap_mid),
                    None,
                    "row {i} 卡间空档应无命中（QA101）"
                );
            }
        }
        // QA101 反例 1（死区）：卡 0 顶 40px 带 y ∈ [y0-offset, y0)——旧式 y<y0 → None，
        // 新式命中卡 0（渲染卡 0 顶 = y0-offset，带内点按必须可选）
        assert_eq!(
            super::layout::menu_row_at(540.0, y0 - 20.0),
            Some(0),
            "卡 0 顶带（旧死区）应命中卡 0（QA101）"
        );
        // QA101 反例 2（误选）：卡 1 顶 40px 带——旧式 floor 归 0 误选菜单 0，新式命中卡 1
        let top1 = y0 + step - offset;
        assert_eq!(
            super::layout::menu_row_at(540.0, top1 + 20.0),
            Some(1),
            "卡 1 顶带（旧误选反例）应命中卡 1（QA101）"
        );
        assert_eq!(super::layout::menu_row_at(40.0, y0 + 4.0), None, "左边界外");
        assert_eq!(super::layout::menu_row_at(1040.0, y0 + 4.0), None, "右边界外");
        assert_eq!(super::layout::menu_row_at(540.0, 10.0), None, "标题条区域");
        assert_eq!(
            super::layout::menu_row_at(540.0, y0 + 6.0 * step - offset + item_h + 4.0),
            None,
            "超出 6 项"
        );
    }

    #[test]
    #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn grid_cell_inverts_iso_cell() {
        // EXP-067 正逆成对（S17.8 v3 重锚）：渲染 `chrome::iso_cell_fit`（v3 世界画布
        // G 依赖拟合 `board_fit`，缺席回退 v2 静态 `grid.*`）的每个格点喂回 `grid_cell_at`
        // （同 G 的 `board_fit` 逆解），必须逆解回原格。
        use crate::scenes::chrome;
        let layout =
            data::scenes().ok().and_then(|s| s.get("main_map")).cloned().unwrap_or_default();
        let grid = 20u32;
        let (tw, th) = chrome::iso_tile_fit(&layout, grid);
        assert!(tw > 0 && th > 0, "main_map 棋盘 tile_w/h 应为正（触摸逆变换可用）");
        for r in 0..grid {
            for c in 0..grid {
                let (sx, sy) = chrome::iso_cell_fit(r as i32, c as i32, &layout, grid);
                let hit = super::layout::grid_cell_at(sx as f32, sy as f32, grid, grid);
                assert_eq!(hit, Some((r, c)), "格 ({r},{c}) round-trip 应逆解回原格，实际 {hit:?}");
            }
        }
        // 棋盘外的黑屏点按 → None（不触发行走，S15.6 QA022 边界语义）：画布中心上方一点。
        let bg = chrome::board_fit(&layout, grid);
        let center_x = (bg.x0 + bg.tw) as f32;
        let above_y = (bg.y0 - 50) as f32;
        assert_eq!(
            super::layout::grid_cell_at(center_x, above_y, grid, grid),
            None,
            "棋盘外点按应无命中"
        );
        // 越界格索引（同 G=15，格 (19,19) 超出 15×15）→ 逆解无命中。
        let small = 15u32;
        let oob = chrome::iso_cell_fit(19, 19, &layout, small);
        assert_eq!(
            super::layout::grid_cell_at(oob.0 as f32, oob.1 as f32, small, small),
            None,
            "超出 15×15 的格点应无命中"
        );
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
        let bands = super::layout::slot_bands(10.0, 10.0 + 166.0, 40.0, 40.0, 26.0, 10.0, 5);
        for i in 0..5usize {
            let r = ll.item_rect(i);
            let cx = 88.0;
            let cy = (r.y + r.h / 2) as f32;
            assert_eq!(super::layout::slot_band_index_at(&bands, cx, cy), Some(i), "slot {i}");
        }
        assert_eq!(super::layout::slot_band_index_at(&bands, 88.0, 5.0), None, "上沿外");
    }
}
