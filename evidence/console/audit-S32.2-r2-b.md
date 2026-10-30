# S32.2 第 2 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 3
误报: 0
残留: 4
```

> 落盘说明：第 2 轮 B 侧审计落盘凭证（A 侧同轮 = `audit-S32.2-r2-a.md`，2 条 P2）。
> 只读；实跑 11 条机器判据 + 本机 8 端点 + 越界 5 面 + 15 号 L54 计数逐份复核；
> 未跑 gate.ps1 全量；并行会话在途面逐条剥离归属。

## r1 B 侧 4 条闭环：4/4 CLOSED

- NEW-1（P1 §10.2 assets 越界）CLOSED（guarded_file 数据驱动混合白名单 + 实跑未登记 404）
- NEW-2（plan/packages.yaml）CLOSED（404）
- NEW-3（repo_root 假设）CLOSED（main.rs 启动自检在位）
- NEW-4（22 号 L360 + 15 号 L56 回写）CLOSED（收口挂账 = 预期态，0 回潮）

## 一致性面 a~i：9 PASS / 1 FAIL（b 项 = P2-1 同源）

a) §4.1 vs routes.rs（0 异步栈）：**PASS**
b) §5 表 9 行 vs JSON 11 条 1:1：**FAIL = P2-1**（JSON↔§5 表行数漂移 +
   `dist/maps/index.json` 登记不存在文件；§5 表行 9 条本身与 JSON 前 9 条同构 0 矛盾）
c) §10.2 三条 vs 混合白名单：**PASS**（canonicalize 前缀 / 白名单面 = 数据驱动 /
   assets 只返回登记文件 = 文件项精确 + 目录项限定）
d) §8 冻结路径 9 项 vs FROZEN_PATHS + porcelain 禁用：**PASS**
   （ls-files --others 面；.gitignore 过滤实码 = 仅排除 .git/.gitignore 自身 = P2 注记 3）
e) §11.2 L360 信任项备注挂账 + CS-1 0 字面量：**PASS**
f) /api/whitelist 参数口径（§4.2 枚举/id 约束面 = 写操作 --cmd，只读 GET 不受约束）：**PASS**
   （P2 注记 2：§10.2 补一行口径注，非矛盾）
g) main.rs 启动自检（0 写盘 + 拒启语义）：**PASS**
h) 24 条单测 vs 判据 4：**PASS**（实测 24；log 23 计数差 = 修复同批 +1，非矛盾）
i) evidence/console/ 报告面：**PASS**（r1 两份补写件在盘 + r2 两份 = 4 份；r1 缺件 = P2-2，
   主线程补写后闭环）

## 新发现（3 条 P2，均 1 行级/挂账面，主线程同批已修 2 条 + 1 条挂 S32.4）

| ID | 严重度 | 问题 | 修复落点 |
|---|---|---|---|
| P2-1 | P2 | `dist/maps/index.json` 登记指向不存在文件（同 r2-A P2-1 同源；B 侧独立核实 = 确认） | 已修：JSON 11→10 条 + 22 号 §5 表扩行 |
| P2-2 | P2 | §10.2 第 2 条「5 目录」括注在混合白名单数据驱动后已降级为语义面，22 号 L306 未注口径演进 | 已修：§10.2 补「混合白名单落实面」注 |
| P2-3 | P2 | snapshot .gitignore 命中文件过滤实码 = 仅排除 .git/.gitignore 文件自身，未做全量 check-ignore 遍历；S32.4 执行面消费前建议补全（当前 0 实害，web/ 不在冻结面） | 挂 S32.4（snapshot.rs 头注已补挂账面） |

## 实跑验证（11 条全绿；红项归并行会话）

build EXIT=0 / test 24/24 / clippy 0 warning / cargo tree 0 游戏 crate /
ledger 262/245(→246 漂移归并行) / doccheck 0 error / doccheck2 0 error（38 项）/
CS PASS（S32.2 面 0 新增；CS-3 1 项 ti-engine 新增 = 并行线）/ 三方对账全绿 /
doc_numbers 0 漂移 / 本机 8 端点 + 越界 5 面实测。

## 总判定

**PASS（非收敛轮，3 P2 已同批吸收 2 + 挂账 1）**：r1 4/4 CLOSED + 一致性面 9 PASS/1 FAIL
（b = P2-1 同源）+ 实跑全绿 ⇒ 按 §7.3 计数口径本轮非 0 新发现；第 3 轮复核
（r3 0/0/0 + r4 0/0/0 = 连续 2 轮收敛后主线程独立复核收口）。
