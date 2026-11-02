//! S32.5 双审计编排（22 号 §7）：任务书生成 + 报告归档 + 机械判定。
//!
//! 平台只做「任务书生成 + 归档 + 判定」，**不做**审计本身（不依赖任何模型 SDK，§7.3）。
//! - `template`：八要素任务书模板（§7.3 载体口径；模板逐条覆盖八要素清单，`cover_elements` 机器断言）；
//! - `judge`：机械判定（四行机器字段解析 + 值域 + 轮次完整性 + 收敛 0/0/0×2 + 3 轮升级）；
//! - `taskbook`：从台账生成任务书 `.prompt.md`；
//! - `archive`：粘贴归档报告 `.md`（防穿越 + 命名规则校验）。

pub mod archive;
pub mod judge;
pub mod taskbook;
pub mod template;
