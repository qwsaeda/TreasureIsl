//! `ti-formats` 的 ID 域：控件类型码（`ref/controls.json` 实测值域）。
//!
//! 与 `docs/style.md` §5.3 的 `ids.rs` 规划一致（S1.4b 先落地控件域；
//! 其他 ID 域待各自任务落地）。值域与语义见 [`ControlKind`]；
//! 数据源为 `data/*.Map` 全部控件记录的 `f5` 字段实测分布。
//!
//! S1.4b 实测结论（F-311，见 `ref/event_ids.json`）：`.Map` 控件 6 字段
//! （f0..f5）中**无事件 id 字段**——`f3` 是屏幕 Y 坐标而非 F-306 的
//! 「事件 id」推测；事件 id 由 `sub_31424` 在运行时控件对象上读取
//! （`*(u8*)(*(u32*)(ctrl+12)+22) >> 2 + 1`），不在 `.Map` 记录内。
//! 控件事件 id 的全量语义随 S3.3+ 场景逻辑逐 id 解释。

/// `.Map` 控件记录的 f5 字段（控件类型码，S1.4b / F-311 确认）。
///
/// 全量实测（20 个 UI `.Map`，2041 条记录，`ref/controls.json`）：95% 为 0；
/// 非 0 值 44 种，集中在 20..1400 分档区间（100/200/300/…/1400），语义随
/// S3.3+ 场景逻辑逐档解释，届时在此扩展命名变体。
/// 注意：f5 是控件类型码，**不是**事件 id（事件 id 在运行时对象上，见
/// 模块级文档与 `ref/event_ids.json`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlKind(u32);

impl ControlKind {
    /// 普通贴图控件（实测 1939/2041 条，约 95%）。
    pub const NONE: Self = Self(0);
    /// 实测分档起点（百位档 100..1400）。
    pub const HUNDRED: Self = Self(100);
    /// 实测分档终点。
    pub const FOUR_THOUSAND: Self = Self(1400);

    /// 原始 u32 值。
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for ControlKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.0 == 0 { write!(f, "none") } else { write!(f, "{:?}", self.0) }
    }
}

impl From<ControlKind> for u32 {
    fn from(kind: ControlKind) -> Self {
        kind.0
    }
}

impl From<u32> for ControlKind {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl core::str::FromStr for ControlKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "none" {
            return Ok(Self::NONE);
        }
        s.parse::<u32>().map(Self).map_err(|e| format!("非法 ControlKind: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::ControlKind;

    #[test]
    fn known_measured_kinds_roundtrip() {
        // 实测出现过的 f5 取值（S1.4b 分布统计前 12 + 分档档头）
        for raw in [
            0u32, 1, 2, 3, 10, 11, 12, 20, 21, 22, 30, 100, 101, 102, 111, 112, 200, 201, 202, 222,
            300, 301, 302, 333, 401, 402, 444, 501, 502, 555, 601, 602, 666, 701, 702, 777, 801,
            802, 901, 902, 1300, 1400,
        ] {
            let k = ControlKind::from(raw);
            assert_eq!(k.get(), raw, "roundtrip {raw}");
        }
    }

    #[test]
    fn none_is_default_measured() {
        assert_eq!(ControlKind::NONE.get(), 0);
    }

    #[test]
    fn display_none() {
        assert_eq!(ControlKind::NONE.to_string(), "none");
        assert_eq!(ControlKind::from(301u32).to_string(), "301");
    }
}
