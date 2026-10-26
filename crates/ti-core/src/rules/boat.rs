//! S21.10 船系统正式实现（数值从 `assets/data/balance.json → boat.levels` 注入）。
//!
//! 业务规格（唯一事实源）：[`docs/business/economy.md`](../../../docs/business/economy.md)
//! §船系统；战斗加成口径见 [`docs/business/combat.md`](../../../docs/business/combat.md)
//! §建造加成（双成长轴：建筑 Σ + 船累计值）。
//!
//! 设计要点：
//! - **与建造链平行的独立成长轴**：船不进 [`crate::rules::build::BuildManager`]，
//!   不走 `base_cost × level` 公式；造价 = 目标等级的 `cost`（一次性付清、立即生效）；
//! - **单向升级**：只能升不能降（无降级接口），满级后入口置灰（无操作）；
//! - **货舱容量 = 单次贸易数量上限**：`effective = min(goods.qty_max, cargo)`，
//!   钳制落在 [`crate::rules::trade::TradeLedger`]（`set_cargo_cap`）；
//! - **战斗加成**：`atk`/`hp` 是**累计值**（直接加，不经 `× level`），
//!   与建筑加成叠加（[`crate::rules::battle::CombatBonus`]）；
//! - 纯逻辑、零 IO、确定性（同输入同输出）；数值不写死在 Rust（全部来自
//!   [`crate::data::BoatCfg`]）。

use crate::data::{self, BoatCfg};
use crate::rules::trade::TradeLedger;

/// 船系统错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoatErr {
    /// 金不足（`gold < cost`）。
    InsufficientGold,
    /// 已满级（无操作；`docs/business/economy.md` §船系统规则 1）。
    MaxLevel,
}

/// 船会话：船等级 + 数值配置。
///
/// 等级链来自 `balance.boat.levels`（4 级：小舢板 → 双桅 → 三桅 → 巡防）；
/// 本结构体不硬编码任何业务数值。
#[derive(Debug, Clone)]
pub struct BoatSession {
    /// 数值配置（`balance.boat.levels`）。
    cfg: BoatCfg,
    /// 当前等级（1 起，单向递增）。
    level: u32,
}

impl BoatSession {
    /// 新建船会话（注入配置 + 起始等级）。
    ///
    /// 等级越界（`< 1` 或 `> 最高级`）→ 钳到合法区间（不 panic、不静默产生越界状态）。
    #[must_use]
    pub fn new(cfg: BoatCfg, level: u32) -> Self {
        let max = cfg.max_level();
        let level = if level < 1 {
            1
        } else if max > 0 && level > max {
            max
        } else {
            level
        };
        Self { cfg, level }
    }

    /// 从 `balance.boat` 新建（1 级小舢板；数值单一事实源）。
    ///
    /// # Errors
    ///
    /// [`crate::data::LoadError`]：`balance.json` 缺失 / 解析失败（硬错误，不 fallback）。
    pub fn from_balance() -> Result<Self, data::LoadError> {
        Ok(Self::new(data::balance()?.boat.clone(), 1))
    }

    /// 当前等级（1 起）。
    #[must_use]
    pub fn level(&self) -> u32 {
        self.level
    }

    /// 最高等级（等级链末级；表空 → 0）。
    #[must_use]
    pub fn max_level(&self) -> u32 {
        self.cfg.max_level()
    }

    /// 数值配置只读快照（存档恢复 / 成本查询用；`balance.boat` 单一事实源）。
    #[must_use]
    pub fn cfg(&self) -> &BoatCfg {
        &self.cfg
    }

    /// S21.11 读档恢复船等级（按存档 `boat_level` 重建会话；越界钳到合法区间）。
    ///
    /// 与 [`Self::upgrade`]（单向升级链）**语义分开**：读档 = 还原历史状态，不是降级操作；
    /// 建造升级仍只能走 `upgrade`（单向）。
    #[must_use]
    pub fn restored(cfg: BoatCfg, level: u32) -> Self {
        Self::new(cfg, level)
    }

    /// 当前等级的名称 key（表现层取文本用；缺表项 → 空串）。
    #[must_use]
    pub fn name_key(&self) -> &str {
        self.cfg.level(self.level).map_or("", |l| l.name_key.as_str())
    }

    /// 货舱容量（= 单次贸易数量上限；缺表项 → 0）。
    #[must_use]
    pub fn cargo(&self) -> u32 {
        self.cfg.level(self.level).map_or(0, |l| l.cargo)
    }

    /// 战斗 atk 加成（**累计值**，直接加）。
    #[must_use]
    pub fn atk(&self) -> u32 {
        self.cfg.level(self.level).map_or(0, |l| l.atk)
    }

    /// 战斗 hp 加成（**累计值**，直接加）。
    #[must_use]
    pub fn hp(&self) -> u32 {
        self.cfg.level(self.level).map_or(0, |l| l.hp)
    }

    /// 升到下一级的造价（已满级 → `None`）。
    #[must_use]
    pub fn next_cost(&self) -> Option<u32> {
        self.cfg.level(self.level.saturating_add(1)).map(|l| l.cost)
    }

    /// 是否已满级（满级后入口置灰，升级无操作）。
    #[must_use]
    pub fn is_max_level(&self) -> bool {
        self.next_cost().is_none()
    }

    /// 金是否够支付下一级造价（满级 → `false`）。
    #[must_use]
    pub fn can_upgrade(&self, gold: u32) -> bool {
        self.next_cost().is_some_and(|cost| gold >= cost)
    }

    /// 升级到下一级（扣款 + 等级 +1；**单向**，不提供降级）。
    ///
    /// 成功：扣 `next_cost()` 的金币、等级 +1；货舱容量与战斗加成**立即生效**
    /// （下次查询即新值）。失败：等级与金币都不变（状态不变式）。
    ///
    /// # Errors
    ///
    /// - [`BoatErr::MaxLevel`]：已满级（无操作）；
    /// - [`BoatErr::InsufficientGold`]：金币不足。
    pub fn upgrade(&mut self, gold: &mut u32) -> Result<u32, BoatErr> {
        let cost = self.next_cost().ok_or(BoatErr::MaxLevel)?;
        if *gold < cost {
            return Err(BoatErr::InsufficientGold);
        }
        *gold -= cost;
        self.level = self.level.saturating_add(1);
        Ok(self.level)
    }

    /// 单次贸易数量上限（= `min(goods.qty_max, cargo)`；`docs/business/economy.md`
    /// §船系统规则 2）。
    #[must_use]
    pub fn effective_qty_max(&self, goods_qty_max: u32) -> u32 {
        goods_qty_max.min(self.cargo())
    }

    /// 把当前货舱容量接到贸易账本（数量钳制 = `min(goods.qty_max, cargo)`）。
    ///
    /// 造船成功后由调用方立即调用（立即生效）。
    pub fn apply_cargo_to(&self, ledger: &mut TradeLedger) {
        ledger.set_cargo_cap(self.cargo());
    }
}
