# 11 · VS 对战场景重设计规格（v6：道具 + 现代化回合制）

> 触发：用户 2026-10-29「对战个界面应该有道具特效，现代化回合制的效果，现在的界面设计显示是个屎，多出几种 VS 界面」，
> 并在选型环节明确「**全出渲染图**」（AI 概念稿不算数，要引擎真实渲染结果）。
>
> 单一事实源分工：
> - **业务规则**（动作/道具/伤害结算）= `docs/business/combat.md` §战斗动作（本文只引用，不复述数值）；
> - **版面/资产/交互** = 本文；
> - **数值** = `assets/data/balance.json → combat.items` / `combat.defend_reduction_pct`；
> - **文案** = `assets/data/strings.json → combat.item.*` / `combat.action.*`；
> - **资产** = `assets/ui_manifest.json`（批次 `vs_v1`）。

## 1. 问题诊断（旧 vs 界面为什么「不像游戏」）

| 症状 | 现状（`scenes.json` `vs` 段） | 后果 |
|---|---|---|
| 没有角色 | 全场景只有 `bg_night_sea` + `bg_battle_sea` 两张背景图 | 看不出「在对战谁」 |
| 血条是两条裸矩形 | `player_hp` / `boss_hp` = 448×48 纯色条 | 无立绘、无分段、无角色感 |
| 没有回合结构 | 回合数只是屏心一行文字（`rounds {x:540,y:416}`） | 不像回合制，像数值面板 |
| 没有道具 | 玩家只能「攻击」（`GameInput::Attack` 唯一动作） | 无策略、无资源管理 |
| 没有演出 | 命中特效 = 8 枚 1–3px 火花（`particles.rs`） | 无打击感 |

## 2. 业务规则升级（S27.7 定案）

> **⚠️ 2026-10-30 被 v2 兵力模型取代**：战斗已从「攻击/道具/防御 HP 回合制」改为
> **人手 + 武器确定性 VS**（`docs/business/combat.md` §兵力模型）——攻占/遭遇 = 一次比较结算，
> VS 界面 = 英雄 + 水手**自动动效**出结果（非逐回合玩家输入）。本节「三动作/4 道具」**作废**，
> 仅 §4 的 5 套候选版式（场景版面）仍为选型参考；落地按新 `combat.md` + `territory.md` §4。

### 2.1 动作：三选一（已作废，见上注记）

`攻击` / `道具` / `防御`（规则见 `docs/business/combat.md` §战斗动作）：

- 攻击 = 原伤害模型（`damage = round(atk × coef) + rng`，下限 `flat`）；
- 防御 = 当回合敌方反击减伤 `defend_reduction_pct`（50%）；
- 道具 = 结算该道具效果，**不消耗随机数**（同 seed 同轨迹仍成立）。

### 2.2 道具：4 种

| id | 效果 | 携带上限 | 定位 |
|---|---|---|---|
| `keg` 火药桶 | 固定 300 伤害（不吃 atk/随机） | 3 | 低成长兜底爆发 |
| `medkit` 医疗包 | 回 25% 上限 HP | 3 | 拉长回合，配合防御打消耗 |
| `rum` 朗姆酒 | 3 回合 atk ×1.5 | 2 | 增益铺垫，收益随 atk 放大 |
| `smoke` 烟雾弹 | 敌方下一次反击落空 | 2 | 关键回合免伤 |

数值/上限/文案全在数据侧（`balance.combat.items` + `strings.json`），规则只消费效果
（`domain::combat::ItemEffect`）——**加道具不改 `rules::battle`**。

### 2.3 规则层落点

- `domain::combat::ItemEffect`（效果词汇）+ `CombatAction`（动作词汇）；
- `rules::battle::BattleSession::step_with(action)`；`step()` ≡ `step_with(Attack)`（旧 API 不破）；
- 结算顺序：玩家动作 → 敌 HP 归零即胜 → 敌方反击（`stagger` 落空 / `defend` 减伤）→ 回合末 buff 递减；
- 可测规则 6 条已落地于 `crates/ti-core/tests/rules_battle.rs`（见 `combat.md` 可测试规则表）。

## 3. 资产（S27.7 新增，批次 `assets/ui_batches/vs_v1.json`）

| 资产 | 尺寸 | 抠图 | 用途 |
|---|---|---|---|
| `assets/ui/boss_captain.png` | 600×900 | `magenta_clean` | 敌方船长全身立绘（A/B/D/E 版式） |
| `assets/ui/hero_back.png` | 600×900 | `magenta_clean` | 玩家背身立绘（A/B/D 版式） |
| `assets/ui/deck_battle.png` | 1080×1920 | `none` | 夜间甲板 3/4 俯视背景（A/D 版式） |
| `assets/ui/item_keg.png` | 256×256 | `magenta_clean` | 火药桶图标 |
| `assets/ui/item_medkit.png` | 256×256 | `magenta_clean` | 医疗包图标 |
| `assets/ui/item_rum.png` | 256×256 | `magenta_clean` | 朗姆酒图标 |
| `assets/ui/item_smoke.png` | 256×256 | `magenta_clean` | 烟雾弹图标 |

复用既有资产：`bg_battle_sea.png`（双船对轰，C 版式）、`bg_night_sea.png`（夜空，E 版式）、
`role_*.png`（玩家半身像，E 版式）、`helm.png`（舵轮 = 回合徽章 / 道具转盘）、
`card_panel.png`（圆角面板蒙版）、`deck_plaque.png`（黄铜指挥台）、`icon_*.png`（动作图标）。

## 4. 选型：5 套候选版式（A/B/C/D/E）

> 选型方式 = **引擎真实渲染图**（`--shot --scene=vs` 出图）逐张肉眼比对；
> 渲染器 = `crates/ti-app/src/scenes/vs_proto.rs`（选型期原型，落地后按选定版式重写）。

| 版式 | 结构 | 道具位 | 特点 |
|---|---|---|---|
| **A 甲板对峙** | 夜间甲板背景；左下玩家背身立绘 / 右上 BOSS 立绘；顶部两枚**圆形血环**（12 段） | 底部横向 4 格黄铜腰带（首格选中金环） | 对峙感强、血环现代 |
| **B 海岸 BOSS** | BOSS 立于中央（居中大图）+ 玩家小图在左下；顶部细血带 | 底部**扇形 5 张手牌**（旋转）+ 能量点 | 卡牌肉鸽味 |
| **C 海战对轰** | 双船侧舷对轰（`bg_battle_sea`）；两侧**竖向血槽** | 右下**道具转盘**（舵轮 + 4 枚） | 贴合船系统、零角色动画 |
| **D 甲板透视决斗** | 3/4 俯视甲板；英雄背身在左下、船长在右上；顶部木端血槽 | 底部**弧形道具按钮** | 空间感/打击感最强 |
| **E 对称回合制** | 左右对峙（我方半身像 / BOSS 全身像）+ 中缝；顶部左右分段血条；中央**舵轮回合徽章** | 底部**黄铜指挥台**：3 个大动作键（攻击/道具/防御）+ 上方 4 格道具条 | 层级最清楚、540×960 也不糊 |

### 4.1 选型期原型开关

```sh
# 环境变量 TI_VS_PROTO = a|b|c|d|e ；TI_SHOT_NAME = 输出文件名（不含扩展名）
$env:TI_VS_PROTO='e'; $env:TI_SHOT_NAME='bevy_native_vs_e'
cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot --scene=vs
```

- `TI_VS_PROTO` 未设置 = 正式版式（旧 `vs` 布局，不受影响）；
- `TI_SHOT_NAME` = 同场景多张对比图不互相覆盖（`--shot` 原口径只按场景名命名）。

## 5. 落地 WBS（选定版式后）

| 步 | 内容 | 判据 |
|---|---|---|
| 1 | 本文补「定案版式」小节（逐元素坐标表 + 资产接线 + 交互契约） | 坐标表 = 实现唯一事实源（镜像 `business/08_layout_design.md` §4.4） |
| 2 | 正式实现：`scenes.json` `vs` 段原语 + `vs.rs` 渲染（道具带/动作键/回合徽章/伤害数字） | `vs` 场景出图 = 选定版式 |
| 3 | 动作接线：`GameInput::UseItem(idx)` / `Defend` + `route_hit` 命中（bevy_picking `pick_key`） | 点道具/点动作键 = 对应动作（旧「点哪都是攻击」退役） |
| 4 | 回合演出时间轴（突进 → 命中闪白 → 屏震 → 伤害数字 → 血条插值 → 敌方反击） | 时间轴可复现（`--shot` 仍能定格） |
| 5 | 背包/经济接线：贸易港购买 + 存档持久化 + 携带上限 | 存档往返不变（`persistence.md`） |
| 6 | 文档对齐 + 门禁 | `08_layout_design` §4.4 镜像 + `doccheck` 0 error + `gate` 全绿 |

## 6. 已知取舍

- **圆形头像**：无「圆形裁切」运行时能力（blit 不支持遮罩）⇒ 版式 A 的血环用分段小片拼弧，
  头像仍是方形立绘；若定案 A 且要圆形头像，需为每个角色预生成圆形裁切资产。
- **伤害数字字号**：可用字号只有 96/44/28 三档（`theme.fonts`）⇒ 伤害数字用 96 档，
  不用真 120px 冲击字（要做需扩字号档 = 字体/排版改动）。
- **道具获取**：本轮先做「战斗内可用 + 携带上限」，购买/存档接线列入 WBS 步 5（经济面另开任务）。
