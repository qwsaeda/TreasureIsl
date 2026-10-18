//! 等距投影：逻辑格坐标 ↔ 屏幕坐标的纯数学变换。
//!
//! 等距（isometric）投影把 `(row, col)` 逻辑格映射到呈现层 `(x, y)`：
//! - `x = origin_x + (col - row) * tile_w`
//! - `y = origin_y + (row + col) * tile_h`
//!
//! 原点与 tile 尺寸全为参数，不硬编码——调用方从数据读取后传入。

/// 等距投影参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IsoProjection {
    /// 原点 x（屏幕坐标）。
    pub origin_x: i32,
    /// 原点 y（屏幕坐标）。
    pub origin_y: i32,
    /// tile 半宽（逻辑像素）。
    pub tile_w: i32,
    /// tile 半高（逻辑像素）。
    pub tile_h: i32,
}

impl IsoProjection {
    /// 创建等距投影参数。
    #[must_use]
    pub const fn new(origin_x: i32, origin_y: i32, tile_w: i32, tile_h: i32) -> Self {
        Self { origin_x, origin_y, tile_w, tile_h }
    }

    /// `(row, col)` 逻辑格 → 屏幕 `(x, y)`。
    ///
    /// 返回菱形**顶角**坐标；菱形中心 = `(x + tile_w, y + tile_h)`。
    #[must_use]
    pub const fn cell(&self, row: i32, col: i32) -> (i32, i32) {
        (self.origin_x + (col - row) * self.tile_w, self.origin_y + (row + col) * self.tile_h)
    }

    /// 从逻辑格坐标计算菱形中心。
    #[must_use]
    pub const fn cell_center(&self, row: i32, col: i32) -> (i32, i32) {
        let (x, y) = self.cell(row, col);
        (x + self.tile_w, y + self.tile_h)
    }

    /// tile 半宽/半高。
    #[must_use]
    pub const fn tile_size(&self) -> (i32, i32) {
        (self.tile_w, self.tile_h)
    }

    /// tile 是否为退化值（0 或负）——此时投影不可逆。
    #[must_use]
    pub const fn is_degenerate(&self) -> bool {
        self.tile_w == 0 || self.tile_h == 0
    }

    /// 从屏幕坐标反推最近的逻辑格（用于触摸命中）。
    ///
    /// 返回 `(row, col)`，可能越界（调用方校验）。
    /// tile 退化（`tile_w` 或 `tile_h` 为 0）时返回 `(0, 0)`：原公式分母
    /// `2 * tile_w * tile_h` 为 0 会整数除零 panic（F6），此处早退兜底。
    #[must_use]
    pub const fn screen_to_cell(&self, sx: i32, sy: i32) -> (i32, i32) {
        if self.is_degenerate() {
            return (0, 0);
        }
        let dx = sx - self.origin_x;
        let dy = sy - self.origin_y;
        // 解方程组：dx = (col - row) * tile_w, dy = (row + col) * tile_h
        // => col = (dx/tile_w + dy/tile_h) / 2, row = (dy/tile_h - dx/tile_w) / 2
        let row = (dy * self.tile_w - dx * self.tile_h) / (2 * self.tile_w * self.tile_h);
        let col = (dx * self.tile_h + dy * self.tile_w) / (2 * self.tile_w * self.tile_h);
        (row, col)
    }
}
