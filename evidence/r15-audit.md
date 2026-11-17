# R15 双 agent 对抗审计收口（2026-11-17）

## AUDIT 判定统计
- CONFIRMED 52 / REJECTED 6 / PARTIAL 8 / NO-LINE 34（FIND 57 CONFIRMED-candidate，AUDIT 去重 + 时点豁免撤回 5）

## 主线程复核 8 组全修
| 组 | 缺陷 | 修复 |
|---|---|---|
| Q094 | README L84「当前主待办 S32.4~S32.9 + S34.0~S34.6 + S35.1」无时点标注（台账 0 pending 后过期） | 补「2026-10 时点主待办；现行 2026-11-16 全 verified，进度唯一事实源 = 台账」 |
| Q040/Q041 | loop.md L27/L37「gate 第 8 项/第 35 项」旧时点值（去重实序 ledger=10 / dod-commands=32） | → 第 10 位/第 32 位 + 旧值时点标注 |
| Q096/097/099 | AGENTS 5 处 gate 项号（第 17/23/12/13/23 项）旧时点值 | → 去重序实值（flywheel=26 / check_ledger=20 / ledger=10 / doccheck=11）+ 旧值时点标注 |
| Q001/Q034 | 14 号「SYMBOL_EXPECTATIONS 15 任务（扩 11）」vs 脚本实 16 任务（扩 12） | → 16 任务/扩 12 |
| Q004 | plan-s56 S58.3「分母 19 面 19/19」vs 实测 20/20 | → 分母 20 面 20/20 |
| Q005 | 23 号 L10「evidence 755 文件」双计数矛盾（实测 git-tracked） | → 实测值 + R15 回写注 |
| Q018 | 32 号 L67/L97「现行 = 47 项」doccheck2 过期 | → 50 项（S57.1/S58.0/S58.2 各 +1）注 |

## 停机判定
- 本轮 CONFIRMED 52 条（去重 8 组独立缺陷）全修；**第 2 轮 0/0/0 = 可选复扫**（M-24 拍板①：第 1 个 0 轮即通过，第 2 个 0 轮可选）
- 高频复现模式（gate 项号旧时点值全库残留 = 9 处）：doccheck2 50 项无「gate 项号文字」断言覆盖（R12 EXP-189 同类，S58.0 枚举面只守「N 条流程强化」不守「第 N 项 gate」），EXP 回写登记后续轮扩 gate 项号文字断言面。
## gate 全量：36 项 0 failure（2 环境 SKIP）
