//! S8.5 控件层：`assets/data/controls.json`（2041 条控件记录）→ 帧缓冲绘制 + hit 边界。
//!
//! **状态：legacy 保留（生产渲染路径不使用）**
//!
//! S15 UI 重设计后，7 个场景的渲染全部走 `scenes::chrome`（羊皮纸/木纹/金色系 + `scenes.json` 坐标），
//! 不再调用本模块。保留原因：
//! 1. `controls.json` 是 `scripts/validate_data.py` 的校验对象（与 `legacy/ref/controls.json` 一致性）；
//! 2. `tests/widgets.rs` 与 `tests/game_app_flow.rs` 用本模块验证数据驱动渲染/hit 边界契约。
//!
//! 如需彻底移除，须同步：删 `lib.rs` 的 `pub mod widgets;` + 删 `tests/widgets.rs` + 删 `tests/game_app_flow.rs`
//! 中相关用例 + 确认 `gen_layout` 脚本不再产出 `controls.json`。
//!
//! 单一事实源（`docs/impl/14_asset_enhancement.md`）：
//! - 控件表 = `scripts/gen_layout.py` 从 `ref/controls.json`（逆向控件记录）生成的
//!   `assets/data/controls.json`（20 场景 / 2041 记录 / 44 种 f5 控件类型）；
//! - 本模块**只读**该表并提供「画控件 + hit 测试」两个窄能力，**不**在 `.rs` 里
//!   硬编码控件位置（消除旧 `render_*` 里手写的 `fill_rect` 魔法数）。
//!
//! 判据（S8.5 `DoD`）：
//! - 只画**可见**控件（`flags & 1 != 0` 且 `w > 0 && h > 0` → 「draw 面积 > 0」）；
//! - hit 边界 = 控件矩形（`x/y/w/h`），供触摸层做「点在哪个控件上」；
//! - 控件类型 `f5` 决定默认色板索引（`kind % 调色板长度`，避免硬编码每种类型的色）。
//!
//! 色板索引计算：`usize::try_from(rec.kind)`（`kind` 为 u32，32/64 位平台均安全）取模 `palette.len()`，
//! 避免 `as u32` 中转在 64 位指针平台触发 `cast_possible_truncation`。
use ti_core::data::{self as ti_data, ControlRec, ControlsCfg};

/// 控件层：封装 `controls.json` 的读取 + 绘制 + hit 测试。
pub struct WidgetLayer {
    cfg: &'static ControlsCfg,
    /// 控件默认色板（8 色，按 `kind % 8` 取；避免逐类型硬编码颜色）。
    palette: [u16; 8],
}

impl WidgetLayer {
    /// 从 `controls.json` 建控件层（数据表缺失 → `None`，调用方回退旧渲染路径）。
    ///
    /// 默认色板取 `menu_shell` 的 `COLOR_*` 布局色（复用同一套色，S8.4 已抽常量）。
    #[must_use]
    pub fn new() -> Option<Self> {
        use crate::menu_shell::{
            COLOR_BG, COLOR_LANDMARK, COLOR_PLAYER, COLOR_SELECTED, COLOR_TITLE_BAR,
        };
        let cfg = ti_data::controls().ok()?;
        Some(Self {
            cfg,
            palette: [
                COLOR_TITLE_BAR,
                COLOR_SELECTED,
                COLOR_LANDMARK,
                COLOR_PLAYER,
                COLOR_BG,
                COLOR_TITLE_BAR,
                COLOR_SELECTED,
                COLOR_LANDMARK,
            ],
        })
    }

    /// 控件表（只读，`assets/data/controls.json`）。
    #[must_use]
    pub fn cfg(&self) -> &ControlsCfg {
        self.cfg
    }

    /// 取某场景的**可见**控件（迭代器；未命中 → 空）。
    pub fn visible(&self, scene_id: &str) -> impl Iterator<Item = &ControlRec> {
        self.cfg.visible_records(scene_id)
    }

    /// 某场景可见控件数（S8.5 判据：>0）。
    #[must_use]
    pub fn visible_count(&self, scene_id: &str) -> usize {
        self.visible(scene_id).count()
    }

    /// 控件默认色（按 `kind % 8` 取色板，避免逐类型硬编码）。
    ///
    /// 色板长度恒 8（`new()` 构造），`rec.kind % 8` 值域 0..8，`u32` 足够；
    /// `usize` 直接取模（32/64 位平台均为合法整数，无需 `u32` 中转截断）。
    #[must_use]
    pub fn color_of(&self, rec: &ControlRec) -> u16 {
        let idx = (usize::try_from(rec.kind).unwrap_or(0)) % self.palette.len();
        self.palette.get(idx).copied().unwrap_or(self.palette[0])
    }

    /// 把某场景的可见控件画到帧缓冲（`MenuShell::fill_rect` 逐控件；`w/h == 0` 自动跳过）。
    ///
    /// 逐位对照：控件矩形 `(x, y, w, h)` 直接落 `fill_rect`，无偏移/无缩放
    /// （`controls.json` 的坐标已是逻辑像素）。
    pub fn draw(&self, scene_id: &str, shell: &mut crate::menu_shell::MenuShell) {
        for rec in self.visible(scene_id) {
            shell.fill_rect(
                i32::try_from(rec.x).unwrap_or(0),
                i32::try_from(rec.y).unwrap_or(0),
                i32::try_from(rec.w).unwrap_or(0),
                i32::try_from(rec.h).unwrap_or(0),
                self.color_of(rec),
            );
        }
    }

    /// hit 测试：点 `(x, y)` 落在哪个可见控件上（后画的在上面 → 倒序遍历取首个命中）。
    ///
    /// 边界规则：左闭右开（`x <= px < x+w` 且 `y <= py < y+h`），与 `fill_rect` 像素判定一致。
    #[must_use]
    pub fn hit(&self, scene_id: &str, px: i32, py: i32) -> Option<&ControlRec> {
        let ux = u32::try_from(px).ok()?;
        let uy = u32::try_from(py).ok()?;
        // 后画的在上面 → 倒序遍历取首个命中（收集到 Vec 后 rev()，控件数小）。
        let recs: Vec<&ControlRec> = self.visible(scene_id).collect();
        recs.into_iter().rev().find(|r| {
            ux >= r.x && ux < r.x.saturating_add(r.w) && uy >= r.y && uy < r.y.saturating_add(r.h)
        })
    }
}
