# S32.0 第 3 轮审计 · A（完成度）

> 落盘说明：本文件为**第 3 轮审计 findings 的落盘凭证**（会话内审计报告的结构化汇总）。
> 审计方式：只读，不改任何文件。实跑：`ledger` / `doccheck` / `doccheck2` / `check_code_standards.py --check`。
> 审计对象：S32.0 第 3 版（22 号 + WBS + 台账）。

## 总判定（第 3 轮）

**FAIL** —— 第 2 轮 16 条 findings **全部 CLOSED**（逐条给行号证据）；新发现 **P0×0 / P1×1 / P2×7 = 8 条**。

## 新发现（8 条）

| ID | 严重度 | 问题 | 修复落点（第 4 版已落） |
|---|---|---|---|
| N-1 | P1 | §3.1「唯一写点 = `evidence/console/`」被 `gate.ps1` 自身写点证伪：第 23 项 browser-playtest **无分档守卫**，实测改写 git 跟踪的 `evidence/web_playtest.log` + `.png` | §3.1 写点表逐条列出；§8 把该两条列为「门禁既有副作用路径：只记录不判定」 |
| N-2 | P2 | §3.1 漏列 `task_close.py` 的 `evidence/<task>.log` 写点（非 `--check` 路径也写） | §3.1 写点表补 `evidence/<task>.log` |
| N-3 | P2 | §8 冻结路径漏 `crates/ti-formats`（游戏侧 crate，被 ti-core/ti-app/ti-platform 依赖） | §8 冻结路径补 `crates/ti-formats` |
| N-4 | P2 | §8「绝对为空」判据当前不可满足（并行会话在途改动会红），且与 §10.4 的会话相对口径矛盾 | §8 改**增量口径**（动作前后 `git status --porcelain` 快照之差为空） |
| N-5 | P2 | 台账 `rounds` 与实际轮次不符；r1 报告抬头承诺「第 4 轮」与「3 轮未过拆子任务」冲突；第 2 轮报告未落盘 | 按 `docs/loop.md` §4 拆 `S32.0a` / `S32.0b`；r1 抬头改「后续轮次」；r2/r3 报告落盘 |
| N-6 | P2 | §10.1 的 token 守卫（非回环无 token 拒启动 / 写操作必带 token）在台账无验收任务 | S32.1 / S32.4 DoD 补鉴权负例判据 |
| N-7 | P2 | §4.2「已装包 id 枚举」判据（`manifest.json` 存在者）会把骨架 `_template` 算成包（实测 `verify _template` = 11 项不合规；**注：该实测为第 3 轮时点，现测 7 项 —— `_template` 无 `map.json`，报的是「缺 map.json + I1/I2/I3」四条结构检查**） | §4.2 改为「已装包集合取自 `map_pack.py list`」（工具自身已排除 `_template`） |
| N-8 | P2 | r1-a 报告内部严重度分档与自身表格不符（写 P1×6/P2×6，实为 P1×7/P2×5，总数 12 不变） | r1-a 勘误 |

## 补充核对（实跑通过项）

- `doccheck` 0 error / `doccheck2` 0 error（38 项）/ `ledger` 0 error / `check_code_standards.py --check` PASS；
- §11.2「既有门禁自动覆盖」论证属实（占位/unwrap 扫描覆盖 `crates/**/*.rs`，CS 扫描遍历 `crates/`）；
- §3.4「ti-tools 间接拖入游戏栈」属实（`ti-platform(ogg)` → `ti-core/ti-render/macroquad`）；
  `crates/ti-tools/src/ledger.rs` 已是纯 std 手写 flow-mapping 解析器 ⇒ 「ti-devmeta = 纯 std」可行；
- 待建项归属清晰（`crates/ti-devmeta/`、`crates/ti-console/`、`console_sources.json`、`console_commands.json`
  均已归到具体任务）⇒ 不构成虚报。
