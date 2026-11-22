# 成就系统（正向设计，2026-11-22）

> 单一事实源：本文定义成就的业务规则与触发条件；数值/文案落到 `assets/data`。
> release.md §后续「成就系统」的正式规格。不依赖逆向产物。

## 1. 目的

给玩家长期目标与即时正反馈：在关键玩法节点（击败 BOSS / 占领领地 / 通关 / 财富 / 造船）
解锁成就，持久记录到存档，可随时查看清单。纯增量系统，不改既有玩法数值。

## 2. 成就清单（5 个，位掩码 bit 0..=4）

| bit | id | 名称 key | 触发条件 |
|---|---|---|---|
| 0 | `first_boss` | `achievement.first_boss` | 首次击败任意 BOSS |
| 1 | `conqueror` | `achievement.conqueror` | 首次占领领地格（副本胜利/攻占） |
| 2 | `clear_m01` | `achievement.clear_m01` | 通关 m01（解锁链首图 CLEARED 位） |
| 3 | `tycoon` | `achievement.tycoon` | 金币峰值达到 10000 |
| 4 | `master_ship` | `achievement.master_ship` | 船升到满级（level 4） |

## 3. 判定规则（纯函数）

`AchievementStats { boss_defeats, owned_cells, cleared_m01, peak_gold, boat_level }`
由 app 层从运行状态汇总；`rules::achievements::eval(mask, &stats) -> (new_mask, newly_unlocked)`：
- 对每项未解锁成就，若对应 stats 达标 ⇒ 置位 + 记入 newly。
- 单调递增：已置位永不清除（只增不减）。
- 阈值固定为常量（`TYCOON_GOLD=10000` / `MASTER_SHIP_LEVEL=4`）定义在规则层
  `rules::achievements`（成就阈值不随 balance 调，避免 balance schema 扩散；调整阈值须改规则层+本文档）。

## 4. 持久化

成就位掩码 `u64` 存入 `SaveSlotV4Ext.achievements`（`#[serde(default)]`，旧档 v4=None 或
无此字段 ⇒ 0，零迁移成本）。存档往返须还原；与 `unlocked_mask`（跨图解锁）语义独立。

## 5. 表现

- 解锁瞬间：主地图 toast 提示（i18n `achievement.unlocked_fmt` + 名称 key）。
- 查看：主地图 P 键开「成就面板」，列出 5 项 + 已解锁/未解锁态（复用已签收羊皮卷轴风格）。
- 视觉 A 纪律：复用已签收候选 A 羊皮卷轴，非新视觉设计；新增仅内容文案（数据驱动）。

## 6. 测试判据

- 规则层：eval 单项达标置位 / 单调不减 / 多项同解锁 / 阈值来自 balance。
- 存档：写入位掩码 → 往返还原；旧档（无字段）= 0。
- app e2e：P 键开面板 / 状态展示；解锁钩子（至少 first_boss / tycoon）触发 toast + 置位。

## 7. 边界

- 成就不影响任何玩法数值/胜负判定（纯展示 + 记录）。
- 热座多席：成就为账号级（不按席位），全档共享一个掩码。
- 成就 id/文案新增 = 加数据（i18n + 本表），不改规则结构。
