# S45 测试接入标准固化 WBS（游戏测试平台管理）

> 定案 = [`process/31_test_ingress_standard.md`](process/31_test_ingress_standard.md)（schema 契约 + 登记流程 + 平台接入口径 + 门禁断言）。
> 执行纪律：S44 后独立（用户拍板 2026-11-06）——S44.2~S44.5 全 verified 后开工；按阶段门顺序验收 + 每任务双审计（22 号 §7）。
> 平台与游戏运行时解耦：不改玩法/渲染/数值/`web/` 产物（断言方式 = 22 号 §8）。

## 阶段总览

| 阶段 | 目标 | 任务 | 阶段门 |
|---|---|---|---|
| S45.0 定案 | 31 号标准 + WBS + 登记 | S45.0 | doccheck/doccheck2 0 error + gate 全绿 |
| S45.1 落地 | 23 号补全 + validate_data 强化 + 平台测试页展示标准 + 新增测试端到端 | S45.1 | 新测试按标准登记 → 平台自动接入 + 零平台代码 + gate 全绿 |

## 任务表

| id | 依赖 | 产物 | DoD（摘要） |
|---|---|---|---|
| S45.0 | S44.5 | 定案文档（31 号）+ WBS + 登记 | 31 号要素齐全（§1~§7：schema 契约/command 契约/登记流程/平台接入口径/门禁断言/登记）+ WBS 任务表 + 登记（README/路由表/00_index）+ 23 号 §7 补「测试接入标准」段；doccheck + doccheck2 0 error；gate 全绿；evidence/S45.0.log |
| S45.1 | S45.0 | 标准落地 + 新增测试端到端 | `validate_data::check_test_manifest` 强化（command 程序白名单 / 无 shell 拼接面 / id 格式 / name 必填）+ 23 号 §7 契约表 + 平台测试页展示标准（`/api/tests` 含 schema 契约摘要）+ **新增测试端到端**（按标准登记一条测试 → `/api/tests` 自动出现 + `?run` 白名单执行 → `git diff crates/ti-console/` = 空〔零平台代码〕）+ 登记违规负例（sh/bash/`;`/缺 name → validate_data FAIL）+ gate 全绿 + evidence/S45.1.log |

## 每任务审计要求（强制，镜像 S32）

1. 主线程实现 + 自验，命令实跑输出入 `evidence/S45.x.log`（含 DoD 自身判据）。
2. 并行起两个子代理（只读、互不可见）：A 完成度（DoD 逐条）/ B 一致性（架构/规范/安全/依赖）。
3. findings 修复 → 复审计；0 新发现 + 0 误报 + 0 残留，连续 2 轮才算通过；连续 3 轮未过 ⇒ 拆子任务。
4. 报告落 `evidence/console/audit-S45.x-r<N>-{a,b}.md`，抬头含四行机器可解析字段（22 号 §7.3）。

## 不做

- 不做重放断言（运行走 test_manifest command 本体）；不做任意 shell（程序白名单 + 逐参数非 shell）。
- 不内嵌测试框架；不重写既有测试（23 号 §6 不做什么）。
