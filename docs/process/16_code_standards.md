# 代码标准（执行矩阵 · 机器强制）

> **定位**：`02_conventions.md` 管「该不该这么写」（架构/流程），`style.md` 管语法层（trait/clippy）。
> 本文 = **执行矩阵**：把「架构 / 内聚 / 耦合 / 命名 / 重构」每个维度映射到
> **机器检查** 或 **人工纪律**，并声明豁免与基线。
>
> **纪律（15_process_ledger §1 同源）：标准不被任何脚本/门禁执行 = 标准不存在。**
> 本文每一行必须有「机器执行」或「人工执行载体」，无执行体的条目禁止入表。

## 1. 执行矩阵

| 维度 | 标准条款 | 机器执行 | 存量豁免 / 基线 |
|---|---|---|---|
| **单一事实源 / 耦合** | 资产路径字面量只允许驻留 `scenes/chrome.rs`（资产表单一读点）/ `ti-core` data 层 / `ti-tools` 工具链（CS-1；`02_conventions` §1 之资产面） | `scripts/check_code_standards.py --check`（gate 项，全档执行） | `process_ratchet.json::code_standards.cs1_asset_grandfathered`（当前 **0**） |
| **职责声明 / 内聚** | `src/` 每个 `.rs` 文件首部必须有 `//!` 模块文档（负责什么 / 不负责什么；`02_conventions` §9.1「一个文件 = 一个职责」）（CS-2） | 同上 | `cs2_doc_grandfathered`（存量 10 = oracle 分卷） |
| **函数命名** | `fn` 参数名禁单字母（例外仅坐标 `{x, y}`；`02_conventions` §3「禁无意义缩写」）（CS-3） | 同上 | `cs3_param_grandfathered`（存量 136） |
| **架构 / 窄接缝** | 场景层（`scenes/`）禁构造持久会话（`BoatSession`/`BoardSession`/`MapSession`/`BattleSession`）——会话归 `game_app` 层，场景只经 `SceneHost` 窄接缝读写（一次性规则对象 `BuildManager`/`TradeLedger` 按次结算即弃，不在此列）（CS-4） | 同上 | `cs4_scene_grandfathered`（当前 **0**） |
| **职责拆分 / 内聚（规模面）** | 文件 ≤900 行 / 函数 ≤140 行 / 同骨架重复 ≥N 次 | `scripts/audit_code.py`（gate 独立项） | `scripts/code_budget.json` 棘轮基线 |
| **lint 红线** | `cargo fmt` 0 diff / `clippy -D warnings` 0 警 / 库代码 0 `unwrap`·`expect` / 0 `TODO`·`FIXME`·`unimplemented!`·`panic!` | gate 项 1~2 + 占位/unwrap 扫描项 | —（零豁免） |
| **重构纪律** | 批量编辑 → 一次编译；报错一次性收口；禁「改一处编一次」探针节奏（`02_conventions` §9.2） | 人工（飞轮审计抽查，`06_flywheel.md`） | — |

## 2. 工具契约（`check_code_standards.py`）

- **4 项检查 CS-1~CS-4**（上表前 4 行），只读：从不写 `.rs`；`--check` = 门禁判定（违例超基线 ⇒ 非 0 退出）；
  `--report` = 信息性明细（恒 0）；`--update-baseline` = 写回豁免基线（仅 `process_ratchet.json`）。
- **棘轮只收紧不放宽**：豁免 = 存量违例点（`reason` + `fix_in` 任务号，15_process_ledger §2 纪律）；
  **新增代码零豁免**——新文件/新函数/新参数命中任何 CS 项 = gate FAIL。
- 豁免清零后对应条目可人工删除（收紧方向，允许）；`--update-baseline` 本身只增不删（防误删）。
- 规模维度（文件/函数行数、重复模式）归 `audit_code.py` + `code_budget.json`，**本工具不重复**（单一事实源纪律）。

## 3. 工作流

1. **写码前**：对照 §1 矩阵（新代码零豁免：不硬编码资产路径 / 文件带 `//!` 职责声明 / 参数非单字母 / 场景不构造会话）。
2. **提交前**：`gate.ps1`（或 `-Fast`）自动拦新增违例；被拦 = 先改码，不是加豁免。
3. **确需豁免**（结构性债务）：`--update-baseline` 生成条目 → 人工补 `fix_in` 任务号 → 挂 `plan/ledger.yaml`。
4. **门禁结构变化**（gate 增删项）：同步 `doccheck2::check_gate_item_count` 的 `expected` 阈值 + docs「gate N 项」声称（FIND-304-B）。

## 4. 与相邻文档的关系

- `02_conventions.md` §1/§3/§9 = 本文条款的**条文源**（本文只加「机器执行」列，不复述条文）。
- `style.md` §5.6.x = trait/标准库选用语法规范（本文不覆盖）。
- `14_tooling_catalog.md` = 工具登记（本文工具三处登记之一）。
- `15_process_ledger.md` §2 = 豁免必须挂任务号的纪律源。
