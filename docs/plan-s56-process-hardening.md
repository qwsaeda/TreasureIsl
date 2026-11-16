# S56 流程强化（15 号行 36 载体专项：doccheck2 项号文字 + 数值声称点回写面）

> 立项 = 2026-11-16 轨道 3（rea-audit-fix 开发流程审计）M-13/M-23 拍板「折中面固化」+ 15 号 §2 行 36 待固化任务号。
> 前置 = R12 修 20+ 处「项号文字」漂移（EXP-189 教训：gate_data.json 数组顺序 = 机读唯一源，文档「第 N 项 X」= 手工镜像必漂移）。

## WBS

- [ ] **S56.0**：`scripts/doc_numbers.py` 扩「声称点对账」面
  - ① 扫全库（AGENTS.md + docs/**/*.md，排除 experience 叙事卷/plan-* 时点快照）`doccheck2 第 N 项` 声称点；
  - ② 对照 `assets/gate_data.json` `doccheck2_checks` 1-based 位实测，声称 (N, check_name) 不匹配 ⇒ 报告；
  - ③ `--fix` = 按实测位回写声称点（现状只读 `--check`）；`--check` 违规非 0；
  - 判据 = 现行全库 0 漂移（R12 已清零态）+ 负例护栏（临时改 1 处项号 ⇒ `--check` 必 FAIL）+ 工具契约 3 处登记（14 号 §3.1 + 04 速查 + README 索引，doccheck2 tooling-contract 第 29 项）。
  - evidence = `evidence/S56.0.log`。
