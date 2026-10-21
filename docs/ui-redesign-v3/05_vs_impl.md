# 05 · vs 战斗场景落地记录（v3.1 双写 · S17.7）

> 注：本文的「基线」记述属**历史**：像素基线机制已于 **S18.7b 退役**（三套 golden PNG + 三个 diff 测试均已删除）；
> 现行机制 = 结构不变式断言 + CPU↔Bevy 内存对拍。

> 03 提案 §3.5 的落地版（波次 4 首波）。vs 为 7 场景中「视觉冲击最大」的双打场景之一：
> v2 空场 544px 死区 + HUD 顶置双血条 → v3 中央竞技区 `battle_arena`（mood `tense_battle`，
> focus.anchor = `battle_arena`，bbox_pct {7,12,85,52} ≈ 44% 屏）。
> **双写纪律同 S17.6**：v2 键（player_hp/boss_hp/rounds/outcome/empty/hint）坐标一律不动，
> v3 键（`battle_arena`/`focus_anchor`）叠加；v3.4 收口再删 v2 键。

## 1. 版面草图（1080×1920 单坐标系）

```
y=0      ┌──────────────────────────────────┐
         │  title 条（v2，y8..104）          │
y=168    │  ▓▓ 玩家血条 64..512   ▓ 576..1024 BOSS 血条 ▓   │  ← v2 HUD 顶置（P-OT 后续波角化）
y=230    │ ╔══════════════════════════════╗ │
         │ ║  battle_arena 竞技区          ║ │  x76..994 × y230..1228
         │ ║   罗盘饰（y≈479 十字金饰）   ║ │
y=416    │ ║   回合数（v2 屏心文本，叠上）  ║ │
y=895    │ ║   水平饰线（parch_light 1px） ║ │
y=960    │ ║   结果/空态（v2 屏心文本）     ║ │
y≈999    │ ║   ▂▂▂ 顶层甲板 50% 宽（bevel_hi）   ▂▂▂ ║ │
y≈1070   │ ║   ▂▂▂▂ 中层甲板 70% 宽（menu_bg）    ▂▂▂▂ ║ │
y≈1141   │ ║   ▂▂▂▂▂ 底层甲板 90% 宽（parch_light）▂▂▂▂▂ ║ │  ← 三层阶梯 = 甲板纵深
         │ ╚══════════════════════════════╝ │
y=1840   │  hint 行（v2）                      │
y=1920   └──────────────────────────────────┘
```

z-order（调用序，`vs.rs`）：`chrome::scene_shell`（底/框/标题条）→ `chrome::battle_arena`（竞技区垫底）
→ v2 血条/回合/结果文本（叠上）→ `chrome::scene_hint`。

## 2. 坐标方案（scenes.json vs 段，双写键）

| 键 | 值 | 依据 |
|---|---|---|
| `battle_arena {x:76, y:230, w:918, h:998}` | 竞技区矩形 | = manifest vs `focus.bbox_pct {7,12,85,52}` × (1080,1920) 取整：7%→75.6→76、12%→230.4→230、85%→918、52%→998.4→998（N8 对账 ±1px 内） |
| `focus_anchor {type:"battle_arena", bbox:{x:76, y:230, w:918, h:998}}` | 视觉重心锚 | gate 第 21 项 `check_scene_narrative` N8 对账值（scenes.json ↔ manifest 一致性唯一事实源） |
| v2 键不动 | `player_hp{64,168,448,48}` `boss_hp{576,168,448,48}` `rounds{540,416}` `outcome/empty{540,960}` `hint{540,1840}` | 血条带 y=168..216 在竞技区顶 230 之上，0 重叠；回合/结果文本落在竞技区内（P-Hades-3 中央浮现式） |

## 3. 实现映射（代码落点）

| 层 | 落点 | 说明 |
|---|---|---|
| 原语 | `ti_render::shell::battle_arena` | 外框 2px `wood` 发丝 + 三层甲板阶梯（90/70/50% 宽，`parch_light`→`menu_bg`→`bevel_hi` 自暗向亮 = 纵深）+ 罗盘饰（`gold_bright` 十字 + `gold` 点，与 `title_art_block` 同源语言）+ 2/3 处 `parch_light` 水平饰线；纯静态 0 动效（快照可复现；色板 0 外部资产；`w/h ≤ 0` 不画） |
| 包装 | `chrome::battle_arena(host, "vs")` | 数据驱动读 `battle_arena` 键，缺失/非正 = 不画（不 panic、不占位，与 icon 列同判据） |
| 接线 | `vs.rs` render | z-order 见 §1；血条/回合/结果文本语义 0 变更 |
| 8pt | `doccheck2::ui_8pt_walk` 豁免 `battle_arena` | v3 键坐标由 DSL/manifest 接管（ui-redesign-v3/02 §10），76/230/918/998 非 8pt 整除属预期 |
| 镜像 | `08_layout_design §4.4` 两行 | 表值 = scenes.json 同值（gate 第 9 项 doccheck2 `layout-doc-table-cross-check` 对账） |
| 基线 | `assets/render_baseline/vs.png` + `assets/forward_baseline/vs.png` 重生成 | hd 集合（Index/SelectRole/MainMap）不含 vs，不动 |

## 4. 像素证据（render_baseline/vs.png 抽检）

| 点 | 采样 | 值 | 判定 |
|---|---|---|---|
| 底层甲板 (540,1150) | `parch_light` | (16,24,74) | 底色 (8,12,41) 上可见 |
| 中层甲板 (540,1100) | `menu_bg` | (24,32,99) | 逐级亮 ✓ |
| 顶层甲板 (540,1030) | `bevel_hi` | (33,44,140) | 逐级亮 ✓ |
| 水平饰线 (540,895) | `parch_light` | (16,24,74) | 在位 |
| 罗盘饰 (540,479) | `gold_bright` | (74,199,239) | 在位 |
| 血条区 (200,192) | `COLOR_PLAYER` | (66,182,181) | v2 血条未受竞技区影响（y=192 < 230） |

## 5. 设计裁决记录

- **弃用 `bevel_shadow` 做底层甲板**：`bevel_shadow` == `bg` == `parch_dark` = 0x0865（theme.rs DEFAULT），
  与深底同色不可见；改用 `parch_light`（0x10C9）做最暗阶梯，形成 3 级可见渐变。
- **不做命令盘（P-OT-3 4 按钮）**：vs 业务模型 = tap-to-attack 单指令（`GameInput::Attack`），
  4 按钮会是死 UI（占位/假控件，违反硬规则 1）。命令盘留给 v3.4 波次（战斗指令扩展业务功能落地时
  随 `ui_icons`/input 模型一并上）。
- **星空不入 vs**：mood `tense_battle` 的重心 = 战场甲板（近景）而非夜空（远景），`ambient_art`
  只给 index（epic_entry）；vs 的氛围由竞技区罗盘饰 + 阶梯承担。
- **血条顶置暂留**：v3.1 双写不动 v2 血条坐标；P-OT-2 血条角化是 v3.4 波次（与命令盘同波，
  涉 HUD 重排 + `portrait_zones` 调整，需单独基线波）。

## 6. 后续波（v3.4，未开工）

`vs` + `main_map` 视觉冲击双打：血条角化（顶 1/4 屏两侧）+ 中央 4 项命令盘（tap-to-attack
扩展为指令选择时）+ main_map 世界画布 76% 屏放大。deps：战斗指令扩展业务功能（P0 优先级由
S17.5 视觉验收后定）。
