# 领地/大富翁 v2 业务逻辑落地 — 阶段计划（S33）

> 触发：用户 2026-10-30 五轮业务逻辑输入（战斗改兵力模型 + 人手/武器 + 驻军自定 + 攻占/破产/
> 多人同格 + 海事事件 + 商人 + 每图事件/商人 + 胜负条件），并明确「**每个阶段 = 目标 + 文档 +
> 规范 + 经验，后面阶段可以参考复用**」。
>
> 业务规则定案稿 = `docs/business/territory.md`（v2）+ `docs/business/combat.md`（兵力模型）+
> `docs/business/economy.md`（资源节）+ `docs/process/21_map_package_spec.md`（每图事件/商人）。
>
> **阶段纪律**：每阶段完成 = 目标达成 + 文档对齐 + 规范（`rules/*`）落地 + 经验（`docs/experience/`）记录；
> 后阶段**复用**前阶段的规范/经验（模块接口、公式、数据段、单测命名）。

## 阶段总览（每阶段 = 目标 / 文档 / 规范 / 经验 / 复用）

| 阶段 | 目标 | 文档（单一事实源） | 规范（落码） | 经验（记录） | 后阶段复用 |
|---|---|---|---|---|---|
| **S33.1 规则层基础** | 兵力模型 + 海事事件 + 商人（确定性、可测） | `combat.md` §兵力模型 / `territory.md` §6-7 / `economy.md` 资源节 | `rules/force.rs` / `rules/events.rs` / `rules/merchant.rs` | ①确定性 VS（不耗随机）②双方必损（pct）③一人一把（武装=min）④事件效果=百分比损耗+绝对增量双字段⑤商人上限（人手/船容量） | S33.3 攻占/多人同格 用 `force::resolve`；S33.5 VS 界面 用 `force`；S33.6 AI 用 `force` |
| **S33.2 数据层** | `balance.json` 加 `manpower/events/merchants` 段 + `map.json` 逐格地形/建筑/事件/商人 | `21_map_package_spec.md` §3.1 / `territory.md` §7 | `data/balance.rs` Raw 段 + `validate_data.py` 断言 | ⑥数据段增量（serde default 回退，缺省不破）⑦每图商人/事件不变量 | S33.3 规则读段；S33.4 存档 v5 引用 |
| **S33.3 领地规则重做** | 驻军自定 / 攻占（败交双倍）/ 破产+胜负 / 多人同格（后到者决定）/ 不可占领格 | `territory.md` §4 / §2 | `rules/territory.rs` 重做 | ⑧驻军守恒（驻军+舰队=总人手）⑨攻占败交双倍⑩多人同格只打前一个 | S33.4 存档驻军；S33.5 UI；S33.6 AI |
| **S33.4 存档 v5** | 席位人手/武器 + 领地驻军 + 迁移 v4→v5 | `territory.md` §8 | `rules/saveload` v5 | ⑪迁移链缺省（单席+默认资源+空领地） | S33.5 读档 |
| **S33.5 表现层** | 玩家攻占 UI（交钱/攻占二选一）+ VS 界面按格类（自动动效）+ 驻军显示 | `11_vs_spec.md` §4 / `combat.md` VS 界面 | `scenes/vs` 按格类 + `main_map` 攻占选项 + 驻军显示 | ⑫VS 按格类×地图主题 ⑬自动动效（无逐回合输入） | S33.6 复用 VS |
| **S33.6 AI** | AI 驻军自定（战后 1/3 守）+ 攻占判定（**兵力模型**：舰队战力 vs 守方战力）+ 自动回合 | `territory.md` §6 | `rules/territory.rs::ai_policy` + `territory_v2::ai_policy_v2` + `ai_turn_v2` + `ai_siege_*_fmt` toast | ⑭AI 同一分支表（策略只选分支，v2 攻占判定用 `force::resolve` 预测而非纯阈值） | 收官 |

## 依赖与顺序

```
S33.1（规则基础）→ S33.2（数据）→ S33.3（领地规则）→ S33.4（存档 v5）→ S33.5（表现）→ S33.6（AI）
```

- S33.1 自包含（已可编译 + 21 单测过），供 S33.3/S33.5/S33.6 复用。
- S33.2 为 S33.3+ 提供数据（`balance.manpower/events/merchants` + `map.json` 逐格）。
- 每阶段完成 = `gate` 相关项 + 该阶段单测全绿 + `evidence/S33.x.log` 记录。

## 阶段验收（每阶段判据）

- **S33.1**：`cargo test -p ti-core --lib {force,events,merchant}::` 全绿（21 项）+ `gate` 相关项 PASS + `evidence/S33.1.log`。
- **S33.2**：`validate_data.py` 全绿（新增段断言）+ `map_pack verify` 0 违规 + `evidence/S33.2.log`。
- **S33.3**：`cargo test -p ti-core --lib territory::` 全绿（§10 表新规则单测）+ `evidence/S33.3.log`。
- **S33.4**：存档 v4→v5 往返还原单测 + `evidence/S33.4.log`。
- **S33.5**：`--shot vs` 按格类出图（攻占/副本/BOSS/遭遇）+ 攻占 UI e2e + `evidence/S33.5.log`。
- **S33.6**：AI 回合 e2e（AI 摇骰/占领/攻占可见）+ `evidence/S33.6.log`。
