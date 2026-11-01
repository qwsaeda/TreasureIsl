# S34 main_map v6「古典羊皮航海图」+ VS v2 战斗动画 落码 WBS

> 触发：用户 2026-10-31 几十轮迭看拍板 v6「不规则岛 + 格子紧围岛」+ VS 全新重设计
> （`12_main_map_v6_vs_v2_spec.md` 单一事实源；`13_user_requirements_ledger.md` 用户要求总账）。
> 本 WBS 把 spec 的「待确认/落码」面拆成可验收阶段。
>
> **前置**：S33.6 verified（领地 v2 AI 完整，`rules::territory_v2` 可消费）。

## 阶段总览

| 阶段 | 目标 | 产出 | 判据 |
|---|---|---|---|
| **S34.0** | v6/VS v2 设计稿定案 + 引擎出图（用户视觉签收 A 纪律） | `evidence/S34.0.log` + 出图 | spec §4.1 用户确认留痕 + 引擎真实渲染出图 + gate 全绿 |
| **S34.1** | v6 布局 JSON 生成工具 + 31 格坐标导出 | `scripts/compose_main_map_v6.py`（已固化）+ `assets/data/main_map_v6_layout.json` | `--check` 0 违规 + 布局 JSON 在位 + validate_data 全绿 |
| **S34.2** | v6 渲染层终码：`render_v6_elem` 默认路径 + 满屏海洋回退 | `crates/ti-app/src/scenes/main_map.rs`（`render_v6_elem`） | `--shot main_map` 出图 0 硬错误 + render_intent main_map 非黑 ≥980‰ + gate 全绿 |
| **S34.3** | VS v2 元素批次：`assets/ui/vs_v2/*.png` + 批次规格 + ui_manifest 登记 | `assets/ui/vs_v2/` + `assets/ui_batches/vs_v2.json` | `ui_asset_batch check` 0 违规 + 9 元素在位 + gate 全绿 |
| **S34.4** | VS v2 动画原型落码：`vs_anim_proto.rs` 4 阶段（clash/duel/result_win/result_lose）+ i18n 文案 | `crates/ti-app/src/scenes/vs_anim_proto.rs` + `strings.json` `scene.vs.v2_elm_*` 键 | `TI_VS_ANIM_PROTO=<phase> --shot vs` 4 张出图 + 0 硬错误 + gate 全绿 |
| **S34.5** | VS v2 按格类 × 地图主题分层：5 类（攻占/副本/BOSS/遭遇/事件）各一张出图 | `vs_anim_proto.rs` 格类分派 + 主题资产 | `--shot vs` 5 类各出图可区分 + 格类×地图主题不同 + gate 全绿 |
| **S34.6** | 视觉签收 + 出图验收（用户过目） | `evidence/S34.6.log` + 出图 | 用户视觉验收通过（消息留痕，A 纪律）+ gate 全绿 |

## 依赖

```
S34.0（设计定案+出图）→ S34.1（布局JSON）→ S34.2（v6渲染终码）
S34.0 → S34.3（VS v2 元素）→ S34.4（动画原型）→ S34.5（格类分层）→ S34.6（视觉签收）
```

- S34.0 前置 = spec 用户确认（A 纪律留痕）；已落码部分（`render_v6_elem` 默认路径 + `vs_anim_proto.rs`）
  作为 S34.0 的「设计稿先行」产物，用户过目后才算 S34.0 verified。
- S34.1 依赖 S34.0（布局参数定案）。
- S34.2 依赖 S34.1。
- S34.3~S34.6 依赖 S34.0（VS v2 设计定案）。

## 验收纪律

每阶段 = 目标达成 + 判据实跑输出写入 `evidence/S34.x.log` 后置 verified（硬规则 2）。
gate 全绿（33 项 0 failure）是前置条件。

## 审计第 6 轮登记（对抗审计 FIND/AUDIT，2026-11-01；**S34 待接线/占位，非 bug，不擅动**）

下列 16 条为 S34 进行中原型占位 / 待接线项，**登记为已知待办**（第 7 轮 FIND 不再重复标记）：
- **VS v2 动画原型占位**（`vs_anim_proto.rs`，S34.4 才替换）：Q008 `DEMO_ATK_CREW=30/ARMS=18/DEF=20/12` 硬编码（≠ balance.manpower.initial_crew=20/initial_arms=5）；Q009 `tri(t,900,10)/(800,16)` 三角波 period/amp 手调（spec §2.6 无参数表锚定）；Q010/Q092 `crew.min(40)/40.0` 分母硬编码 40（spec §2.5 兵力条基准未声明）；Q044 战报 :222-224 只列单侧未拆武装/徒手 + 未列守方战损（spec §2.5 要求双侧）。
- **v6 主地图渲染待接线**（`main_map.rs`，S34 落码中）：Q043 `render_v2_elem` 只 blit `V2_VS_BG`（`V2_BG_SIEGE/DUNGEON/BOSS` 从未消费，spec §2.4 五类分层未接）；Q051 `:1160-1188` 用 4 拆分元素（`V6_ISLE_LAND/LIGHTHOUSE/TREASURE/VILLAGE`）+ `:1190 let _=V6_ISLAND`（spec §1.3 定义单一 `island.png`——结构漂移）；Q052 海洋生物 4 处硬编码 blit（150-115/935-100/330-85/975-85）无动画（spec §1.4 要求 ti-engine 动画 + §3a 禁硬编码坐标）；Q053 `V6_CENTER=(540,740)`（`:989/:1017`）vs layout `island.cy=890`（spec §1.3 540,890）——原型岛心偏上 150px；Q078 `:1152 ink=0x1E14` + `:1199-1200 0x4A23/0x6245` 硬编码 RGB565 非 theme 令牌（违反数据驱动）；Q079 `:1203 if t.owned {V6_TILE_OWNED}else{V6_TILE_DENY}` 按归属布尔选面与 SpaceKind 无关——功能格误示；Q080 `:1153-1158` 木框 8/3px 硬编码（spec §1.1 未定义羊皮框宽）；Q098 `on_hit :574-588` 只处理 key 4..=7，key 0..=3（赛道/环格/据点岛/v6 格）无动作——格点击进副本/交钱/攻占/升级未接线。
- **v6 资产命名/登记**：Q066 m04 有完整 31 格 pack 但不在 `balance.selection.maps`/`MapSlot::all()`（`selection.rs:101` 仅 3）⇒ 已装包但不可选；Q067 v6 `isle_land/treasure/village` 非 `isle_<isle_id>` 命名（pipeline:127）+ `isle_lighthouse` 与 v5 卡面同名冲突；Q068 `assets/ui/isle_lighthouse.png`（v5）+ `assets/ui/v6/isle_lighthouse.png`（v6）同 basename 双路径——资产二义性。
- **v6 资产常量**：Q085 `V6_PARCHMENT` 仅定义无消费（S34 spec §1.1 定案 v6 外框羊皮做旧，**待接线**；chrome.rs:452 已注记保留）。

**业务待办（需用户拍板，非本审计能修；登记不擅动）：**
- **Q003**（船货舱联动，S21.10）：`TradeLedger::new` 货舱上限取 `cfg.default_cargo_cap`（`trade.rs:91` = 99，1 级小舢板），`balance.trade` 无 `default_cargo_cap` 字段，`ti-app` 无 `BoatSession::cargo` 接线 ⇒ **船升级后货舱仍限 99**（`boat.levels[].cargo` 未接进 `effective_qty_max`）。需用户拍板：船升级货舱是否进单次贸易上限（`docs/business/economy.md` §船系统规则 2 已定案，S21.10 接线）。
- **Q100**（驻军玩家自定 UI）：`territory.md` §4.1 定案「驻军 = 玩家自定，战略取舍（非公式）」；现状 `board.rs:348`/`host.rs:452` 硬编码 `crew/3`（1/3 驻军），无玩家自定 UI。需用户拍板：驻军自定交互（进副本/占领/攻占后的驻军滑杆/输入）。

