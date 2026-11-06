# 审计任务书 · S44.3 · r2 · a 侧

> 你是一个**只读**对抗审计子代理（A：完成度审计）。不得改文件、不得改台账、不得跑构建。
> 被审任务 = S44.3。本轮 = r2 收敛验证轮。

## 角色声明
你是对抗审计子代理 a（A：完成度审计）。

## 只读约束
不得改文件、不得改台账、不得跑构建（仅允许只读命令）。

## 被审任务
台账 id = `S44.3`；dod = 改动面=平台（crates/ti-console/src/content/ui_wizard.rs + web index.html UI 向导）+ 数据（evidence/console/content-ui-<id>.json gitignored）| 业务逻辑/游戏产品 Rust 零改动（crates/{ti-core,ti-app,ti-engine,ti-render,ti-platform,ti-formats} 全目录 0 diff；平台本体 ti-console/ti-devmeta 允许改——S44 是平台功能开发，非内容运营）| 判据=向导 6 步每步判据 + 端到端用平台跑 index 变体（设计稿出图→变体器→rebuild→出图→签收→零 Rust）每步判据过 + git diff crates/{ti-core,ti-app,ti-engine,ti-render,ti-platform,ti-formats} = 空（游戏产品源码零改动；ti-console/ti-devmeta 平台本体允许改） + evidence/*.log 含「用户视觉验收」（doccheck2 第 35 项）+ 变体器同路径替换（资产名/坐标/scenes.json 不动）+ UI 原型截图用户确认留痕 + gate 全绿 + evidence/S44.3.log
证据路径 = evidence/S44.3.log。

## 本轮定位
本轮 = r2（收敛验证轮，复核上一轮 findings 是否闭环）。

## 审计面
A：DoD 逐条；B：架构/规范/文档/门禁/回归/安全/依赖

## 证据要求
必须引用**文件路径 + 行号**或**命令实跑输出**，禁止「看起来没问题」。

## 报告格式契约
报告正文**第一个 ``` 代码块**内必须含四行（缺任一行 ⇒ 判不可解析）：
```
总判定: PASS|FAIL
新发现: <整数>
误报: 0
残留: <整数>
```
- `新发现` 含 P2 建议（子代理报什么算什么）；`误报` = 0；`残留` = 上一轮未闭环数。

## 输出要求
把完整报告正文（含四行代码块 + findings 表）作为最终回复输出。

## 上一轮 findings（需复核）
- r1 a 侧：新发现 1，残留 0（evidence/console/audit-S44.3-r1-a.md）
- r1 b 侧：新发现 7，残留 0（evidence/console/audit-S44.3-r1-b.md）

