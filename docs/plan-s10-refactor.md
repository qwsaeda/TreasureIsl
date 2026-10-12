# S10 现代化收口计划（Modern Refinement Flywheel）

**目标**：修复界错乱/加载 bug → 占位清零 → GPU 化渲染 → 对抗流水线 loop 至停机。
**前置决策**（2026-10 用户）：本轮全做完（阶段 0+1+2+3+4）；TTF 字形并入 GPU 阶段；停机条件 = 审计 0 新发现 + P3-3 16 步 0 失败 + gate 14 项全绿。

> 本计划取代旧「S9 骨架」定位：S9.1/S9.3/S9.4 已 verified（场景拆分/字段私有化/Builder+Type-State），
> S9.2（窄接口拆分）与 S9.5（Command 模式）回退 pending（S9.5 因 commands.rs 不存在被 `check_ledger_reality` 第 14 项拦截，EXP-058）。
> S10 在 S9 骨架之上推进「真正可玩 + 现代观感」。

## 1. 阶段 0 — 标准修正与防虚报（主线程）

- **0.1 台账回退 + S10 登记**（`plan/ledger.yaml`）：S9.5 加 `blocked_by: s10-flywheel-stage-0`（pending）；新增 S10.0–S10.5。
- **0.2 门禁第 14 项**（`scripts/check_ledger_reality.py` + `gate.ps1`）：台账 verified ↔ evidence ↔ 代码符号三方对账（`SYMBOL_EXPECTATIONS` 登记 DoD 声明的代码产物，如 S9.5 的 `trait Command`/`commands.rs`），机器化拦截子代理虚报。
- **0.3 经验回写**（`docs/experience/process.md` + `code.md` + `index.md`）：
  - EXP-058：子代理 verified 必须 Test-Path + 实跑复核 + 三方对账（S9.5 教训）；
  - EXP-059：地形 camera 钳位 + 触摸/渲染坐标系脱节 = 主地图界错乱双 P0（S10.1）；
  - EXP-060：HD 不递归 + 场景 key 碰撞 = HD 失效 + 切场景残留（S10.2）。
- **0.4 标准入档**：
  - `docs/modernization.md` §1：渲染目标改 GPU 逐精灵 batch + TTF 字形，176×220 CPU blit 降级为 `legacy-render` feature；
  - `docs/parity.md`：P0-5（触摸/渲染同坐标系）+ P0-6（HD 递归注入 100%）+ P4-4（GPU 观感）+ gate 14 项 + 停机条件；
  - `docs/style.md` §5.6.5：GPU 渲染约定（atlas + batch + alpha blend + TTF）。

## 2. 阶段 1 — P0 错乱修复（并行 2 子代理，文件域不相交）

| 子代理 | 任务 | 文件域 | 判据 |
|---|---|---|---|
| A（S10.1） | 地形 camera 去钳位 + 触摸等距逆变换 | `game_app.rs` L901-910、`scenes/mod.rs` L394、`touch.rs` L139-196、`scenes/main_map.rs` L144-148 | headless 断言点按棋子与实际移动方向一致（P0-5） |
| B（S10.2） | HD 递归加载 + 场景 key 碰撞 + 字体缺口临时占位 | `game_app.rs` L728-774（递归）、`treasure_native.rs` L354-370（SceneKind）、字体 14 字临时字形 | HD 注入计数 = assets_hd_pack 全量（P0-6）；切 Intro/Exit 无残留 |

## 3. 阶段 2 — 占位清零（1 子代理 + 主线程复核）

8 处硬编码/绕过全走 `balance.json`（`assets/data/balance.json` 增 `combat.player` / `selection.maps` / `trade.markup` 等字段）：

| # | 位置 | 现状 | 改为 |
|---|---|---|---|
| 1 | `game_app.rs:1010` | `Unit{hp:2000,atk:60}` 硬编码 | `balance.combat.player` |
| 2 | `game_app.rs:786` | 主地图网格/据点/初始位硬编码 | `balance.selection.maps` |
| 3 | `main_map.rs:99` | 卖价 markup=1.0 零差价 | `balance.trade.markup` 真实值 |
| 4 | `main_map.rs:114` | 建造走 oracle `base_cost()` 绕过 rules | 走 `rules::build` |
| 5 | `main_map.rs:83` | qty_max 未查 | `balance.trade.goods[i].qty_max` |
| 6 | `game_app.rs:387` | 键盘 trade/build 固定 gid=1/Castle | 接 widgets 商品/建筑枚举 |
| 7 | `game_app.rs:809` | balance 失败 fallback 1000 | 改硬错误（`LoadError` 传播） |
| 8 | `rules/cards.rs:59` | CARD_CATALOG 占位 | 走 `balance.cards` |

判据：`check_hardcode.py --list` ti-app 命中归零 + diff 测试全绿。

## 4. 阶段 3 — GPU 化渲染（1 子代理）

- 新建 `crates/ti-platform/src/gpu_renderer.rs`：macroquad `draw_texture` 逐精灵 + 纹理图集 + alpha 混合；
- TTF 字形（`macroquad::load_ttf` + Noto Sans CJK 子集）替代 14 缺口字符；
- HD 默认开（递归加载后实际生效）；
- legacy 176×220 CPU blit 保留为 `legacy-render` feature 回退；
- 新 `render_parity_gpu` 基准全重生成（联动 S7.7 解锁，删 S7.7 `blocked_by`）；
- 判据：新基准全绿 + 60fps（`stats` 实跑）+ TTF 14 字无豆腐块（P4-4）。

## 5. 阶段 4 — 对抗流水线 loop（主线程驱动）

```
loop:
  1. gate.ps1（14 项）全绿
  2. rea-audit-fix 双 agent 对抗审计（找文档/代码/数据漂移）
  3. 漂移三分类回写：
     - 可机械判定 → doccheck2 数值断言 / gate 项
     - 需修码     → evidence 修复 + 测试
     - 流程教训   → docs/experience/index.md 新 EXP 条
  4. 修复 → 重 gate → 再审计
停机：审计 0 新发现 + P3-3 16 步 0 失败 + gate 14 项全绿
```

**防虚报**：每轮子代理产出主线程一律 Test-Path + 实跑复核（EXP-058）。

## 6. 执行顺序

```
阶段 0（主线程，1 轮）
  ↓
阶段 1A + 1B 并行（2 子代理，文件域不相交）
  ↓
阶段 2（1 子代理 + 主线程复核）
  ↓
阶段 3（1 子代理，GPU 化）
  ↓
阶段 4（loop 对抗流水线，多轮直至停机）
```

## 7. 已定开放点

- 字体缺口：**TTF 替代**（并入阶段 3）✓
- 本轮范围：**全做完**（0+1+2+3+4）✓
- 停机条件：**审计 0 新发现 + P3-3 0 失败 + gate 全绿** ✓

## 8. 风险与对策

| 风险 | 对策 |
|---|---|
| 子代理虚报 verified（S9.5 教训） | 主线程 Test-Path + 实跑复核 + `check_ledger_reality` 第 14 项机器拦截（EXP-058） |
| GPU 化大改回退 P4-1 基准 | 新 `render_parity_gpu` 全重生成，legacy CPU blit 保留 feature 回退（P4-1 双路径） |
| 占位清零改 balance.json 影响 diff 测试 | balance 字段增补后跑 `cargo test -p ti-core --test diff_*` 全绿才置 verified |
| 对抗流水线不收敛（审计反复出新发现） | rounds≥3 拆子任务；漂移优先固化进 doccheck2/gate 而非现场 grep（EXP-048） |
