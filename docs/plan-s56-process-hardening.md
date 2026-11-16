# S56 流程强化（15 号行 36 载体专项：doccheck2 项号文字 + 数值声称点回写面）

> 立项 = 2026-11-16 轨道 3（rea-audit-fix 开发流程审计）M-13/M-23 拍板「折中面固化」+ 15 号 §2 行 36 待固化任务号。
> 前置 = R12 修 20+ 处「项号文字」漂移（EXP-189 教训：gate_data.json 数组顺序 = 机读唯一源，文档「第 N 项 X」= 手工镜像必漂移）。

## WBS

- [x] **S56.0（verified 2026-11-16）**：`scripts/doc_numbers.py` 扩「声称点对账」面
  - ① 扫全库（AGENTS.md + docs/**/*.md，排除 experience 叙事卷/plan-* 时点快照）`doccheck2 第 N 项` 声称点；
  - ② 对照 `assets/gate_data.json` `doccheck2_checks` 1-based 位实测，声称 (N, check_name) 不匹配 ⇒ 报告；
  - ③ `--fix` = 按实测位回写声称点（现状只读 `--check`）；`--check` 违规非 0；
  - 判据 = 现行全库 0 漂移（R12 已清零态）+ 负例护栏（临时改 1 处项号 ⇒ `--check` 必 FAIL）+ 工具契约 3 处登记（14 号 §3.1 + 04 速查 + README 索引，doccheck2 tooling-contract 第 29 项）。
  - evidence = `evidence/S56.0.log`。

## S57 流程强化第二批（轨道 3 P2 可选面，2026-11-16 用户拍板「三个都做」）

- [x] **S57.0（M-17，verified 2026-11-16）**：`plan/packages.yaml` 加 P7=引擎域（ti-engine）+ P8=平台域（ti-console/ti-devmeta）⇒ 6 红线全挂包域（原 P1~P6 只挂 ti-core + ti-app 2/6）；05_module_parallel §3 包表同步 + `flywheel_readme.ps1 -Package P7/P8` 红线注入面实跑验证。
- [x] **S57.1（M-11，verified 2026-11-16）**：`doccheck2::check_state_md_discipline` 第 48 项（loop.md §9 state.md 载体面：① §9 段 + 3 要素字段名锚点存在性 ② in_progress 跨会话任务（rounds≥1 或 dod 含「跨会话」）须 `evidence/<id>.state.md` 或 dod「state 豁免」标记）+ 双负例护栏（删 §9 段 / 造 in_progress 跨会话行）。
- [x] **S57.2a（M-04 第一批，verified 2026-11-16）**：`check_ledger_reality.py` SYMBOL_EXPECTATIONS 扩面——从 3 任务（0.9%）扩到「dod 明确声明代码产物的任务」自动提取面（解析 dod 里 `crates/<x>/src/<file>.rs` / `fn <name>` / `pub struct <Name>` 字样自动登记期望符号，人工只审白名单豁免）+ doccheck 断言覆盖率下限（≥30% 声明代码产物的 verified 任务有符号登记，棘轮只收紧）。
- [x] **S57.2b（M-04 第二批，verified 2026-11-16）**：`doc_numbers.py` 扩「符号覆盖率」回写面（实跑 check_ledger_reality 输出「覆盖率 P%（C/D）」↔ 14 号 + check_ledger_reality.md 声称点对账，`--fix` 回写 + 负例实跑必 FAIL）；**阶梯 60%→100% = 后续覆盖率提升轮**（提覆盖率 = 扩 SYMBOL_EXPECTATIONS 手工登记 / dod 精确面补全 → check 脚本 `if cov < N` 常量收紧；doc_numbers 回写面随阶梯值自动跟进，无需再改工具）。

## S58 流程强化第三批（2026-11-16 用户拍板「EXP-192 三缺口全做 + 覆盖率 100% 声明面」）

- [ ] **S58.0（EXP-192 缺口①）**：doccheck2 第 49 项 `process-reinforcement-count`——「N 条流程强化」枚举值 ↔ 台账 S54.1/S55.1/S56.0/S57.0~S57.2b 实测 7 条（26 号 L14/L155/L163 + AGENTS.md L8 声称点对账；EXP-192 教训 = 枚举面 0 断言致 7/8 条 off-by-one）。
- [ ] **S58.1（EXP-192 缺口②）**：`doc_numbers.py` 扩 `crates/ti-console/web/index.html` 门禁项数声称面（R13 Q079 = R10 平台面勘误漏前端 3 处硬编码「33 项」；扩面后 --fix 可回写，防再漏网）。
- [ ] **S58.2（EXP-192 缺口③）**：WBS 复选框回写判据面——plan-sNN.md 文档 `[x]` 勾选 ↔ 台账任务 verified 状态对账断言（R13 Q002 = plan-s56 4 行 [ ] 未随 verified 回写；载体 = doccheck2 第 50 项 `wbs-checkbox-sync` 或 check_ledger_reality 扩面，落地时定）。
- [ ] **S58.3（M-04 100% 声明面）**：符号覆盖率分母 19 面（代码/规则实现 声明）全登记——11 个无代码产物符号字样 dod 手工 `SYMBOL_EXPECTATIONS` 登记 / dod 精确面补全 ⇒ 19/19 = 100%（可提取面 8/8 已 100%；声明面全登记 = M-04 终态）。
