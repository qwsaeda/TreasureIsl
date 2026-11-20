# 飞轮 R19 · 定向轮（测试体系 + 平台面专项）（2026-11-20）

方法：开轮 gate 全绿（36 项 0 failure；初查 3 FAIL = 本机 8080 静态服务未起的环境面，拉起后清零）
+ FIND（subagent harness 故障第 3 次 ⇒ 主线程手动 FIND 覆盖 10 面 + 回归面）
+ AUDIT-A subagent 独立裁决（9 CONFIRMED / 0 REJECTED / 1 新缺口 R19-10）
+ AUDIT-B harness 故障（第 4 次）⇒ 主线程独立做 B 轴（值漂移/口径）。

## FIND 清单（主线程，9 CONFIRMED 候选 + 7 面 0 漂移）
- R19-01: 23号 §2「43.8 MiB，≤8MB 预算」同句自相矛盾（上轮 Q004 我引入）⇒ 改「超 ≤8MB 预算〔实态 5.5×，超标定态〕」
- R19-02: 23号 §0「103 个集成测试」实测 104（上轮新增 jobs_proof_tests.rs 未回写）⇒ 103→104
- R19-03/07: 23号 §0「evidence 560（排除 console）」实测 565（上轮 5 新 tracked）⇒ 560→565
- R19-04: persist_proof 写 evidence/tests/ = 平台新写点未登记 ⇒ 22号 §3.1 写点表补行 + 23号 §6.2 证明行标写点属性
- R19-05: evidence/tests/ gitignore 口径未定 ⇒ 拍板 = 运行时车（gitignored，与 console/jobs 同类）+ 静态基线 tracked 双轨
- R19-06: console_commands 3 条 oracle_diff_* writes 字段语义误用（手工重定向日志 ≠ 命令写面）⇒ writes=[]
- R19-08: gate.ps1 注释 posmap 项号 = 0 漂移（回归面复核通过）
- R19-09: close.rs HUMAN_VIEW_IDS 数组内嵌中文注释（风格面）⇒ 移注释到 const 上
- 0 漂移 7 面：persist_proof 非 test 作业 no-op / 证明链接可达（guarded_file 白名单）/ 三面对账 27=27=27 / 收割竞态（pid 回填在 reaper 前，无冲突）/ runner=cargo 无 script 可跑 / group 分布前后端一致 / writes 消费点（oracle_diff_* 未被向导引用，改 [] 无副作用）

## AUDIT-A（subagent，完成度轴）
9 CONFIRMED / 0 REJECTED / 0 PARTIAL + 新缺口 R19-10：
- R19-10: 23号 §2「达标动作 = S18.6 LTO 未收口」指向错任务 ID——S18.6 实做 = 像素基线/门禁重建（dod 无体积项）；LTO 达标动作实际 S18.8e（46.6MB 超预算 6 倍留痕，S18.8e 台账 dod + evidence 实证）⇒ 23号改指向 + 11号 S18.1 行加勘误注

## AUDIT-B（主线程代行，一致性轴——harness 故障第 4 次）
- B-1: 23号 104/565/43.8MiB/31 条/L1 12 声称点 vs 实测 = 全一致（修复后）
- B-2: 26号/AGENTS 无 560/103 残留（grep 0 命中）
- B-3: close_tests 12/12 + jobs_proof_tests 3/3（三面对账 27=27=27 回归通过）
- B-4: 可机械固化判定 = 不补新断言（文档现状计数面「随增随漂」无稳定机读基准；现有 doccheck-counts-match 守 doccheck 输出计数、gate-item-number-text 守 gate 项号文字、manifest-gate-posmap 守 manifest gate 语义——本轮漂移面 = 测试文件数/evidence 数两类，属「易变计数」，R12 先例 = 人工回写 + 审计复核兜底，不入断言）

## 修复（7 条，全自验通过）
R19-01/02/03(+07)/04/05/06/09/10 = 23号 3 处 + 11号勘误注 + 22号写点表 + .gitignore + console_commands writes + close.rs 注释迁移

## 验证
cargo test 993/0 + clippy 0 + doccheck 180/38/198/345 0 + doccheck2 53 项 0 + studio_tests 3 类 OK + validate_data OK + close_tests 12/12 + jobs_proof_tests 3/3

## 停机判定（06_flywheel §4：0 新发现 + 0 误报 + 0 残留）
- 找问题 = 9 CONFIRMED 全修（含 AUDIT-A 补报 R19-10）
- 审计 = A 9 CONFIRMED/0 REJECTED + B 全一致/0 漂移
- 修复 = 0 残留（R19-05 口径已拍板落地；R19-09 风格项已修；无 blocked/需人裁决遗留）
- **本定向轮（测试/平台面）收敛停机**（第 1 个 0 轮 + 修复面 0 残留口径，M-24）；
  全域面（业务/渲染/资产）= R3~R11/R16/R17 已停机，本轮无新全域扫描需求
