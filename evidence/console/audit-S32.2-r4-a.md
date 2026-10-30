# S32.2 第 4 轮审计 · A（完成度）

```
总判定: PASS
新发现: 1
误报: 0
残留: 0
```

> 落盘说明：r4 报告主线程补写件（r5-A 附注「轮次完整性凭证缺口」收口，同 r1 补写先例；
> B 侧同轮 = `audit-S32.2-r4-b.md`）。只读；实跑 12 条机器判据 + 本机 8 端点 + 越界 5 面；
> 未跑 gate.ps1 全量；并行会话在途面（CS-3 ti-engine / i18n 孤儿 key / maps manifest）剥离归属。

## DoD 逐条（S32.2，台账 L310，10 判据面）

1 台账任务数/资产数/已装包数与源文件抽样一致：**PASS**（/api/overview = 262/246/S7.7/4/75/
   repo_sources=10；`--bin ledger` 同值 0 漂移）
2 越界路径 400/404：**PASS**（raw 穿越 400 / 编码穿越 404 / 未登记 board.json +
   packages.yaml 404 / 登记面 200 / 符号链接逃逸单测 unix 实跑）
3 0 写盘：**PASS**（repo/ 0 写盘 API；唯一写点 .lock）
4 单测覆盖解析与路径边界：**PASS**（实测 24 条全过）
5 console_sources.json 单一事实源 + CS-1 0 新增：**PASS**（10 条 ↔ §5 10 行 1:1；
   CS-1 ti-console 面 0 命中）
6 不结构化解析 packages.yaml：**PASS**（0 YAML 解析器 + 直读 404 + probe false）
7 gate 等价机器判据：**PASS**（12 条全绿；CS-3 ti-engine 1 项 = 并行 S18.8d/e 线）
8 依赖图断言保持：**PASS**（0 游戏 crate）
9 双审计（r4 本轮）：**PASS（在途）**（r4 A 1 P2 = log 残留，主线程同批修）
10 evidence/S32.2.log：**PASS**（1 处 1 行级残留 = P2-1）

## r3 双侧 1 条 P2 闭环

- r3-B P2-1（15 号 L54 合计 206→207）：**CLOSED**（L54 现值 207；分项 170+5+4+21+7 = 207
  自洽；S32.0a r4~r15 小计 87 与盘上 14 份报告吻合）

## 新发现（1 条 P2，1 行级）

| ID | 严重度 | 位置 | 问题 | 修复落点 |
|---|---|---|---|---|
| P2-1 | P2 | evidence/S32.2.log §判据 1（L45~46） | 判据 1 段实测输出仍为 r1 前计数面（verified:245 + repo_sources:9；现 = 246/10）0 回填；§五 未提 r2/r3 各轮对 JSON 条数的影响 | 主线程同批：判据 1 段回填 246/10 + 时点快照口径注 + 判据 8 段 246 同步 |

## 总判定

**PASS（实质收敛轮，1 P2 行级已定位）**：DoD 10/10 产物面 PASS + r3 1/1 闭环 +
第 3 版 7 改动面 0 新矛盾 + 实跑 12 条全绿（红项全归并行线）。
P2-1 收口同批修后 r5 复核（r5 + r6 双侧 0/0/0 = 连续 2 轮实质收敛窗口，主线程 §7.5 收口）。

> 并行会话在途面（0 计入 S32.2 新发现）：i18n 孤儿 key WARN ×2（S8.x 数据化线）/
> 台账 verified 246 vs log 初稿 245（S32.1 verified 推进在途）/ maps m04 manifest（S32.12 线）/
> CS-3 ti-engine vs_duel.rs::ease_out 新增 1 项（S18.8d/e 线）/ 根 Cargo.toml S18.8d 注释段。
