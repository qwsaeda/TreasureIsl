# S32.1 第 2 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 2
误报: 0
残留: 0
```

> 落盘说明：第 2 轮 B 侧审计落盘凭证（会话内报告全量落盘；A 侧同轮 = `audit-S32.1-r2-a.md`）。
> 只读；实跑 11 条全绿（build/test×2/clippy/cargo tree/ledger/doccheck/doccheck2/
> check_code_standards/check_ledger_reality/doc_numbers/鉴权负例）；未跑 gate.ps1 全量；
> `.lock` 被并行会话占用 ⇒ 服务实例面由 r1 实跑证据 + 路由 0 改动支撑（如实标注）。

## r1 B 侧 5 条 P2 闭环：5/5 CLOSED（P2-4 = 既定挂账 0 残留）

P2-1（console_sources 补 gate_data.json）CLOSED（L5 实测，9 路径 ↔ §5 表逐条一致）；
P2-2（§11.2 信任项备注挂 S32.2）CLOSED（L359 实测）；
P2-3（14 号 §1.4 第 4 条 cargo bin 豁免注记）CLOSED（L52 实测，与 check_tooling_contract
实扫面（scripts/ .py/.ps1）0 矛盾）；
P2-4（参数面 8 条未建模）CLOSED = 既定挂 S32.6（console_commands.json L2 _comment 对账
口径标注在位，0 越界宣称）；
P2-5（§10.4 锁规格在途标注）CLOSED（L322 实测 + main.rs L130 注释实证；本轮新 P2-2 对其
再精化 = 措辞精度，非回潮）。

## 一致性面 a~h：全 PASS（S32.1 自身面 0 矛盾，并行会话剥离归属）

a) §4.1 HTTP 定案 vs 实现：**PASS**（0 tokio/axum/async；thread::spawn 每连接一线程；
   routes.rs std::thread::scope = 同步原语 0 偏离）
b) §4.2 白名单 9 行 vs JSON：**PASS**（id 9=9 / enabled_by_default / 写面逐条一致；
   mappack_verify 无 path 参数 = S32.7 消费面口径一致；close_run 写面第 3 项本轮改前缀
   形态（新 P2-1 修复，见下））
c) §5 数据源表 vs console_sources.json：**PASS**（9=9 逐条对应；CS-1「Rust 侧 0 字面量」
   互证，src/ 实测 0 命中）
d) 根 Cargo.toml S32.1 面：**PASS**（纯插入 3 行〔1 注释 + 2 member 行〕；并行 S18.8d/e
   android 注释块 + LTO/profile + macroquad 删除已剥离归属；L21 误标 S18.8b = r2-A P2-01，
   主线程同批修正，非 S32.1 引入）
e) ti-tools 薄 re-export：**PASS**（ledger.rs 与 HEAD 版逐字节 0 差异；5 符号 1:1；
   bin/ledger + bin/doccheck 仅 import 改道；serve.rs 0 改动；实跑 ledger 输出不变）
f) 14 号 §1.1/§1.4 vs check_tooling_contract 机器扫面：**PASS**（实扫 = scripts/ .py/.ps1，
   不含 crates/；豁免注记 0 误报；存量 §3.1~§3.4 纯追加 0 破坏；实跑 38 项 0 error）
g) §11.2 平台自身质量 vs 现状：**PASS**（0 用例机器绿 = DoD 字面成立；信任项备注挂 S32.2；
   web/ 盲区 = S32.3 兜底口径如实标注）
h) gate 第 8/9 项调用面：**PASS**（--bin ledger 0 error / --bin doccheck 0 error /
   doccheck2 38 项 0 error〔i18n 孤儿 14 条 WARN = 并行会话 strings 在途线〕）

## 新发现（2 条 P2，0 P0/P1；均挂 S32.4/S32.6 后续面，主线程同批已吸收 1 条文档面）

| ID | 严重度 | 位置 | 问题 | 修复落点 |
|---|---|---|---|---|
| P2-1 | P2 | console_commands.json close_run 写面第 3 项 | `evidence/<task>.log` 含 `<task>` 占位符 = 机器真源内模板串，与其余 8 条精确路径风格不一致；S32.6 对账若按字面精确比对会撞模板串 | 已修（第 3 版）：改前缀形态 `evidence/` + 注释「<台账 id>.log 运行时按 task 展开，S32.6 对账按前缀匹配」 |
| P2-2 | P2 | 22 号 §10.4 L324 陈旧锁判定 | 全规格（起始时间 + 30 分钟接管）无「= S32.4」限定，与 L322「pid 最小面」边界的子项归属措辞精度不足（功能面 0 越界，纯文档精度） | 已修（第 3 版）：L324 补「（完整规格 = S32.4 并发面：起始时间 + 动作 id 字段 + 30 分钟接管判定；S32.1 骨架期锁文件仅 pid，接管判定暂不实现）」 |

## 实跑验证（11 条全绿；红项归并行会话）

build exit=0 / test ti-devmeta 8/8 / test ti-console 0 用例 PASS / clippy 0 warning /
cargo tree 游戏 crate 0 命中 / ledger 262/244/0 error / doccheck 0 error / doccheck2 0 error（38 项）/
check_code_standards PASS / check_ledger_reality 全绿 / doc_numbers 0 漂移 /
鉴权负例 --host 0.0.0.0 无 token 拒启 exit=1（实测）。

## 总判定

**PASS（非收敛轮）**：r1 5/5 CLOSED + 一致性面 a~h 全 PASS + 实跑 11 项全绿；
新发现 2 条 P2（均 1 行级文档/JSON 修订，主线程同批已吸收）⇒ 按 §7.3 计数口径
本轮未达 0 新发现；第 3 轮复核（0 新发现 + 0 误报 + 0 残留连续 2 轮 = r3 + r4 达标后
主线程独立复核收口）。
