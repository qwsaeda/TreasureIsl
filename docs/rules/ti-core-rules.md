---
paths:
  - "crates/ti-core/src/rules/**"
  - "crates/ti-core/src/domain/**"
  - "crates/ti-core/tests/**"
---
# ti-core 编码红线（P1~P5 包共享域，CCGS `rules/*.md` `paths:` 模式借鉴，S50.3）

> 本文件 = `docs/rules/` 路径作用域红线速查面（给 LLM 子代理开轮读，**非机器拦截**；
> 机器拦截仍走 `check_module_contracts.py` 包白名单机制，gate 第 16 项）。
> 单一事实源 = 各条红线指向的既有规范文档，本文件只做路径域速查 + 反例，不重述结论。

## 红线（4~6 条，违反 = 返工）

1. **零 UI 直引 / 零渲染调用**：`rules/*` 与 `domain/*` 不得 `use` 任何 `ti_render` / `ti_app` / `bevy`
   渲染原语（`draw_*` / `blit_*` / `Sprite` / `Text` / `Node`）；业务层纯逻辑 + 数据（结构 + 规则表），
   渲染归 `ti-render` / `ti-engine`。反例：`battle.rs` 里写 `draw_hp_bar(...)`。
2. **数据驱动 key 单源**：数值/文本/布局常量不得硬编码在 `rules/*`，须走 `assets/data/*.json`
   的 key（`balance.json` / `strings.json` / `board.json` / `scenes.json`）；新常量先落数据 key 再在
   `rules/*` 读。反例：`progression.rs` 写 `let dice_max: u32 = 6;`（应 = `board.json` `dice.max`）。
3. **业务先定案再动码**：改 `rules/*` 业务规则前，对应 `docs/business/*.md` 单一事实源须先定案
   （业务规则改动面 = 先 `docs/business/` 定案再动代码，硬规则 7）。反例：直接改 `territory.rs`
   占领结算公式而不改 `business/territory.md` §落格分叉。
4. **跨包协作只经两条通道**：包内零跨包 import，只用已冻结契约公共类型
   （`ti_core::domain::*` / `rules::rng::GameRng`）与 `assets/data/*.json` 的 key（`05_module_parallel.md`
   §3 包依赖铁律 1/2）。反例：`P2 economy` 直接 `use` `P1 combat` 内部 `CombatPhase`（应走 `domain` 公共面）。
5. **oracle 参照物边界**：`oracle/*` 是逆向参照实现（唯一源 `ti-core/src/oracle`），`rules/*` 不得
   反向依赖 `oracle` 内部函数（`shell` 层 0 个 `oracle_*` 函数调用，gate 第 11 项
   `check_oracle_boundary`）；oracle 仅作 diff 基准，不是生产路径。反例：`battle.rs` 调
   `oracle::battle::step(...)` 取结果（应自己实现 `rules::battle::step`，diff 测试对照 oracle）。

## 正确 / 反例（1~2 条，给子代理开轮速查）

- 正确：`trade.rs` 读 `balance.json` `trade.price_multiplier` 做结算，纯函数 + 返回 `TradeResult` 结构，
  渲染层（`ti-app/scenes`）再决定怎么画。
- 反例：`trade.rs` 里 `eprintln!("你赚了 {} 金币", profit)`（中文面量 + 渲染/IO 混入业务层，
  违反红线 2 + check_hardcode 严格模式 0 命中）。
