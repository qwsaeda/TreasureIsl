//! UI 图标集（14 枚，48×48 逻辑盒，代码矢量绘制 = 矩形组合，零外部资产）。
//!
//! 设计（`docs/process/10_ui_redesign.md` §5）：图标是**框架层**的几何绘制，游戏层只决定
//! 「哪个界面元素用哪枚 id + 什么 tint」——映射走数据（`assets/data/ui_icons.json`），
//! 框架不认识新游戏/存档等游戏语义（零游戏语义契约，`09_render_framework.md` §1）。
//!
//! 绘制原语只有 `Canvas::fill_rect`（无抗锯齿、无贴图），故全部图标 = 若干条对齐矩形组合；
//! 风格统一 = 4px 笔画 + 直角端头 + 同色单层填充（不做双色，保持在选中/未选中底上都可读）。

use crate::canvas::Canvas;
use crate::theme::Theme;

/// 图标逻辑边长（正方形盒）。
pub const ICON_SIZE: i32 = 48;

/// 图标 id（共 14 枚，与 `assets/data/ui_icons.json` 的 `icons[].id` 一一对应）。
pub const ICON_COMPASS: u32 = 0;
/// 宝箱（读档）。
pub const ICON_CHEST: u32 = 1;
/// 船锚（载入游戏）。
pub const ICON_ANCHOR: u32 = 2;
/// 齿轮（选项设置）。
pub const ICON_GEAR: u32 = 3;
/// 书（帮助说明）。
pub const ICON_BOOK: u32 = 4;
/// 门（离开游戏）。
pub const ICON_DOOR: u32 = 5;
/// 金币（贸易/持有量）。
pub const ICON_COIN: u32 = 6;
/// 剑（战备/战斗）。
pub const ICON_SWORD: u32 = 7;
/// 城堡（建造）。
pub const ICON_CASTLE: u32 = 8;
/// 灯塔（据点）。
pub const ICON_BEACON: u32 = 9;
/// 心形（HP）。
pub const ICON_HEART: u32 = 10;
/// 三角旗（回合标记）。
pub const ICON_FLAG: u32 = 11;
/// 折叠地图（选图）。
pub const ICON_MAP: u32 = 12;
/// 角色首字母底环（字母由调用方叠画）。
pub const ICON_MONOGRAM_RING: u32 = 13;

/// 图标总数（注册表完备性判据：`ui_icons.json` 须恰好登记 14 条）。
pub const ICON_COUNT: u32 = 14;

/// 图标 id 是否在集内（越界 id 静默不画 —— 调用方取注册表时判定）。
#[must_use]
pub fn is_valid(kind: u32) -> bool {
    kind < ICON_COUNT
}

/// 绘制 1 枚图标：`tint` = 图标墨色（默认 `theme.gold`，选中态传 `theme.text_on_parch`）。
///
/// 未知 id = 0 墨点（不 panic、不留坑：调用方应先用 [`is_valid`] 过滤）。
pub fn icon(host: &mut dyn Canvas, theme: &Theme, kind: u32, x: i32, y: i32, tint: u16) {
    if !is_valid(kind) {
        return;
    }
    let (x, y) = (x, y);
    match kind {
        ICON_COMPASS => {
            // 罗盘：外环 4 边 + 竖针 + 中心轴点
            ring(host, x, y, 32, 4, tint);
            host.fill_rect(x + 22, y + 12, 4, 24, tint);
            host.fill_rect(x + 16, y + 22, 16, 4, tint);
        }
        ICON_CHEST => {
            // 宝箱：箱盖 + 箱体 + 金锁
            host.fill_rect(x + 6, y + 12, 36, 10, tint);
            host.fill_rect(x + 6, y + 24, 36, 18, tint);
            host.fill_rect(x + 21, y + 22, 6, 12, tint);
        }
        ICON_ANCHOR => {
            // 船锚：环 + 竖杆 + 横杆 + 锚爪
            host.fill_rect(x + 20, y + 6, 8, 8, tint);
            host.fill_rect(x + 22, y + 14, 4, 24, tint);
            host.fill_rect(x + 12, y + 20, 24, 4, tint);
            host.fill_rect(x + 12, y + 32, 24, 4, tint);
            host.fill_rect(x + 16, y + 36, 4, 8, tint);
            host.fill_rect(x + 28, y + 36, 4, 8, tint);
        }
        ICON_GEAR => {
            // 齿轮：四齿 + 环四边（中心留空 = 孔）
            host.fill_rect(x + 22, y + 4, 4, 10, tint);
            host.fill_rect(x + 22, y + 34, 4, 10, tint);
            host.fill_rect(x + 4, y + 22, 10, 4, tint);
            host.fill_rect(x + 34, y + 22, 10, 4, tint);
            ring(host, x, y, 20, 4, tint);
        }
        ICON_BOOK => {
            // 书：两页 + 书脊
            host.fill_rect(x + 6, y + 10, 18, 28, tint);
            host.fill_rect(x + 24, y + 10, 18, 28, tint);
            host.fill_rect(x + 22, y + 8, 4, 32, tint);
        }
        ICON_DOOR => {
            // 门：门框 + 门板 + 把手
            ring(host, x, y, 24, 4, tint);
            host.fill_rect(x + 12, y + 10, 4, 32, tint);
            host.fill_rect(x + 32, y + 10, 4, 32, tint);
            host.fill_rect(x + 26, y + 24, 4, 4, tint);
        }
        ICON_COIN => {
            // 金币：外圆三段 + 内环两段 + 币心留空
            host.fill_rect(x + 14, y + 6, 20, 4, tint);
            host.fill_rect(x + 8, y + 10, 32, 4, tint);
            host.fill_rect(x + 6, y + 18, 36, 12, tint);
            host.fill_rect(x + 8, y + 34, 32, 4, tint);
            host.fill_rect(x + 14, y + 38, 20, 4, tint);
            host.fill_rect(x + 20, y + 18, 8, 4, tint);
            host.fill_rect(x + 20, y + 26, 8, 4, tint);
        }
        ICON_SWORD => {
            // 双刃剑：剑身 + 护手 + 柄 + 柄头
            host.fill_rect(x + 22, y + 6, 4, 28, tint);
            host.fill_rect(x + 14, y + 34, 20, 4, tint);
            host.fill_rect(x + 22, y + 38, 4, 8, tint);
            host.fill_rect(x + 20, y + 44, 8, 4, tint);
        }
        ICON_CASTLE => {
            // 城堡：双塔 + 城垛 + 城基
            host.fill_rect(x + 8, y + 18, 8, 24, tint);
            host.fill_rect(x + 32, y + 18, 8, 24, tint);
            host.fill_rect(x + 8, y + 12, 8, 6, tint);
            host.fill_rect(x + 16, y + 12, 8, 6, tint);
            host.fill_rect(x + 24, y + 12, 8, 6, tint);
            host.fill_rect(x + 32, y + 12, 8, 6, tint);
            host.fill_rect(x + 8, y + 30, 32, 12, tint);
        }
        ICON_BEACON => {
            // 灯塔：塔身 + 灯室 + 光束
            host.fill_rect(x + 20, y + 18, 8, 24, tint);
            host.fill_rect(x + 18, y + 14, 12, 4, tint);
            host.fill_rect(x + 30, y + 8, 12, 4, tint);
            host.fill_rect(x + 20, y + 40, 8, 4, tint);
        }
        ICON_HEART => {
            // 心形：双叶 + 身
            host.fill_rect(x + 10, y + 10, 12, 12, tint);
            host.fill_rect(x + 26, y + 10, 12, 12, tint);
            host.fill_rect(x + 10, y + 18, 28, 8, tint);
            host.fill_rect(x + 14, y + 26, 20, 8, tint);
            host.fill_rect(x + 18, y + 34, 12, 6, tint);
        }
        ICON_FLAG => {
            // 三角旗：旗杆 + 旗面（阶梯三角）
            host.fill_rect(x + 10, y + 6, 4, 36, tint);
            host.fill_rect(x + 14, y + 8, 24, 8, tint);
            host.fill_rect(x + 14, y + 16, 18, 6, tint);
            host.fill_rect(x + 14, y + 22, 12, 4, tint);
        }
        ICON_MAP => {
            // 折叠地图：纸面 + 两道折痕
            host.fill_rect(x + 6, y + 10, 36, 28, tint);
            host.fill_rect(x + 8, y + 12, 2, 24, theme.menu_bg);
            host.fill_rect(x + 23, y + 12, 2, 24, theme.menu_bg);
        }
        ICON_MONOGRAM_RING => {
            // 角色首字母底环（字母由调用方按数据用正文墨叠画）
            ring(host, x, y, 28, 4, tint);
        }
        _ => {}
    }
}

/// 四边矩形环：在 48×48 盒内居中画边长 `size`、笔画宽 `t` 的环。
fn ring(host: &mut dyn Canvas, x: i32, y: i32, size: i32, t: i32, tint: u16) {
    let o = (ICON_SIZE - size) / 2;
    host.fill_rect(x + o, y + o, size, t, tint);
    host.fill_rect(x + o, y + o + size - t, size, t, tint);
    host.fill_rect(x + o, y + o + t, t, size - 2 * t, tint);
    host.fill_rect(x + o + size - t, y + o + t, t, size - 2 * t, tint);
}
