# S32.6 对抗审计报告 · r1 · B 侧（一致性/风险审计）

> 只读审计：未改文件/台账，未跑构建。8 审计面（架构/单一事实源/硬规则/安全/对账断言/回归/依赖/文档漂移）。

```
总判定: PASS
新发现: 5
误报: 0
残留: 0
```

## findings 表（均 P3 观察，无 P1 阻断 / P2 应修）

| # | 严重度 | 位置 | 结论 |
|---|---|---|---|
| 1 | P3 | scripts/task_close.py:77-84 | run_command 用 shell=True——平台层 run_sync 确用 Command::new+args，但下游 task_close.py 实际 shell 执行白名单固定串；今日安全（静态 JSON + task 台账 id 精确匹配），建议 shell=False + shlex.split 硬化 |
| 2 | P3 | close.rs:102-123 | map_cmd_id 扁平拼接 {runner} {script} {args}：gate → `pwsh scripts/gate.ps1 -File scripts/gate.ps1`（-File 在脚本后，序错）；exp_check → `exp_log.py --check`（缺 --volume）。设计限制，非注入 |
| 3 | P3 | evidence/S32.6.log:103-105 | 证据写「5→9 条单测 + 7 测试名」；实测 close_tests.rs #[test] = 8 条（含 exp_volumes_six）。数字/名单漂移 |
| 4 | P3 | close_tests.rs:41-49,67-75 | close_check/exp_check 测试以真实仓库根调用，write_log 写进真实 evidence/console/logs/（gitignored 无害但非 hermetic）；tmp_root 建了仅清理（死代码） |
| 5 | P3 | 22 号 §4.2:87-91 + close.rs:23 | 对账断言比较 HUMAN_VIEW_IDS（Rust 镜像）vs console_commands.json，不直接读 §4.2 文档文本；文档↔机器靠常量按约定桥接（设计如此，残余漂移面） |

## 审计面证据（摘要）

1. 架构/crate 边界：cargo tree -p ti-console = {flate2, serde_json, ti-devmeta}，0 游戏 crate；close_api.rs 是 routes 子模块，只做 HTTP 前置，业务执行在 runner::close。PASS。
2. 单一事实源：22 号 §4.2 14 id == console_commands.json 14 id == HUMAN_VIEW_IDS 14 id；write_required=[close_run]；EXP_VOLUMES == exp_log.py VOLUMES（6 分卷）。PASS。
3. 硬规则 1：src/ 0 处 unwrap/expect/panic!/TODO。PASS。
4. 安全：--cmd 只接受白名单 id（未登记 400）；run_sync Command::new + args 逐段传（禁 shell 拼接）；exp_check id 匹配 EXP-\d+（禁注入）；exp_check 不执行完整构建。PASS。
5. 白名单对账断言：whitelist_audit 不一致 → Err → 500 不静默。PASS。
6. 回归：GET /api/audit / GET+POST /api/gate 原样；close 纯追加；gate_tests 8 + audit_tests 28 全绿。PASS。
7. 依赖/构建：cargo test -p ti-console 全绿（close_tests 8 / audit_tests 28）+ clippy 0 error + fmt 0 diff + audit_code PASS。PASS。
8. 文档漂移：22 号 §4.2 / 15 号第 1 条 / 14 号 ti-console 行已登记。PASS。

## 结论

8 审计面全通过（安全 4 项无注入向量、对账不静默、回归端点原样、依赖图 0 游戏 crate、测试/构建/结构审计全绿）；新发现 5 条均 P3 观察。总判定 PASS（无阻断/应修）。
