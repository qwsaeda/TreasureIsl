# S32.2 第 3 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 1
误报: 0
残留: 0
```

> 落盘说明：第 3 轮 B 侧审计落盘凭证（A 侧同轮 = `audit-S32.2-r3-a.md`，0 新发现）。
> 只读；实跑 12 条机器判据 + 本机 8 端点 + 越界 5 面 + 15 号 L54 计数逐份复核；
> 未跑 gate.ps1 全量；并行会话在途面逐条剥离归属。

## r2 5 条 + 附带闭环：5/5 CLOSED + 附带 2 在位

- r2-A/B P2-1（index.json + 行数漂移）CLOSED（JSON 现 10 条 0 index.json 命中 +
  §5 表 10 行逐条 1:1：plan/ledger.yaml、docs/、evidence/、gate_data、ui_manifest、
  registry、assets/maps/（§5「<id>/」行展开口径自洽）、.default、assets/ui/、dist/maps/）
- r2-A P2-2（r1 缺件）CLOSED（4 份在盘 = r1×2 补写件 + r2×2）
- r2-B P2-2（§10.2 降级注）CLOSED（L309 混合白名单落实面注在位）
- r2-B P2-3（gitignore 挂账）CLOSED（snapshot.rs L13~16 头注挂账在位）
- 附带①（§10.2 GET 参数注）在位（L307）；附带②（log 24 条计数）在位（L32 + L79）

## 一致性面 a~j：10/10 PASS（第 3 版 0 新矛盾）

a) §4.1 vs routes.rs：**PASS**（纯 std，0 异步栈）
b) §5 表 10 行 vs JSON 10 条：**PASS**（集合 1:1，逐条列出；顺序漂移 0 计数）
c) §10.2 三条 vs 混合白名单：**PASS**（canonicalize 前缀 / 数据驱动 10 条 /
   assets 只返回登记文件 = 实跑 board.json 404 + ui/bg.png 200）
d) §8 冻结路径 9 项 vs FROZEN_PATHS + porcelain 禁用 + 头注 0 回潮：**PASS**
e) §11.2 CS-1 口径 + L360 挂账（现文未改 = 预期态）：**PASS**
f) /api/whitelist vs §4.2 口径（§10.2 注记后 0 重复报面）：**PASS**
g) main.rs 启动自检（0 写盘 + 拒启 exit=1）：**PASS**
h) 24 条单测（实测）：**PASS**（config 4 + guard 7 + data 8 + snapshot 6 = 实测 24）
i) 报告面（r1 补写 2 + r2 两份 = 4 份在盘；r3 两份 = 主线程收口落盘口径）：**PASS**
j) JSON 语法（json.load 实跑）：**PASS**

## 新发现（1 条 P2，1 行级，主线程同批已修）

| ID | 严重度 | 位置 | 问题 | 修复落点 |
|---|---|---|---|---|
| P2-1 | P2 | 15 号 L54 合计 206 | 自分项加总算术差 1（S32.0a 170 + S32.11 5 + S32.12 4 + S32.1 21 + S32.2 r1 7 = **207** ≠ L54 现值 206；逐份实加佐证：S32.0a r4~r15 = 87 与盘上吻合，仅 L54 合计数值差 1） | 已修：L54 合计 206 → 207（主线程同批 1 行级修订） |

## 实跑验证（12 条全绿；红项归并行会话）

build EXIT=0 / test 24/24 / clippy 0 warning / cargo tree 0 游戏 crate /
ledger 262/246/0 error / doccheck 0 error / doccheck2 0 error（38 项）/
CS = S32.2 面 0 新增（CS-3 ti-engine 1 项 = 并行 S18.8d/e 线）/ 三方对账全绿 /
doc_numbers 0 漂移 / 本机 8 端点 200 + 越界 400/404 5 面 + 登记面 200 + whitelist 判定面。

## 总判定

**PASS（非收敛轮，1 P2 已同批修）**：r2 5/5 + 附带 2/2 闭环 + 一致性面 a~j 10/10 PASS +
实跑 12 条全绿 + 新发现 1 P2（L54 计数 1 行级，主线程已修）⇒ 按 §7.3 计数口径
本轮非 0 新发现轮；第 4 轮复核（r4 双侧 0/0/0 = 与 r3 A 侧 0/0/0 构成连续 2 轮
〔r3-B 1 P2 已吸收不计轮，r4 A/B 双侧 0/0/0 判定〕后主线程 §7.5 独立复核收口）。
