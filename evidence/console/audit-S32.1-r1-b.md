# S32.1 第 1 轮审计 · B（一致性与风险）

```
总判定: FAIL
新发现: 5
误报: 0
残留: 0
```

> 落盘说明：第 1 轮 B 侧审计落盘凭证（会话内报告全量落盘；A 侧同轮 = `audit-S32.1-r1-a.md`，
> 7 条 P2，同源同批）。只读；实跑 build/test/clippy/cargo tree/ledger/doccheck/doccheck2/
> check_code_standards/check_ledger_reality/doc_numbers；未跑 gate.ps1 全量；
> 工作树并行会话在途改动（S18.8 系列/S32.11/S32.12/门禁 31→33）逐条剥离归属。

## 一致性面 a~h 逐项判定（全 PASS，S32.1 自身面 0 矛盾）

a) 22 号 §4.1 HTTP 定案（std TcpListener + 每连接一线程，不引 axum/tokio）vs ti-devmeta::http：
   **PASS**（依赖面 0 tokio/axum/async；`thread::spawn` 每连接一线程逐字对齐；
   routes.rs 用 `std::thread::scope` = std 同步原语，0 异步栈偏离）
b) 22 号 §4.2 白名单 9 行人读视图 vs console_commands.json 机器真源：
   **PASS**（id 集合 9=9 逐一对齐；enabled_by_default 逐条一致；写面逐条一致；
   mappack_verify 行 JSON 无 path 参数 = 与「平台自扫描拼 path」口径一致 = S32.7 消费面；
   close_check/exp_check/audit_book 参数列 JSON 未建模 = P2-4 小漂移，S32.6 对账口径不受影响）
c) 22 号 §5 数据源表 vs console_sources.json：**PASS**（P2-1 修复后 gate_data.json 补全，
   8 路径 + write_whitelist 逐条对齐）
d) 根 Cargo.toml members 段：**PASS**（S32.1 面 = 纯插入 3 行〔1 注释 + 2 member 行〕，
   其余 member 行逐字节未动；workspace.dependencies 0 改动；git diff 混并行 S18.8d/e 已剥离）
e) ti-tools 薄 re-export 面：**PASS**（ledger.rs 逐字节拷贝 0 差异；lib.rs 5 符号 1:1 re-export；
   bin/ledger + bin/doccheck 仅 import 改道；serve.rs 本体 0 改动保留；http::run 与 serve 循环
   行为等价〔多线程 = 顺序特例〕）
f) 14 号 §1.1/§1.4 改动 vs 存量 §3.1~§3.4：**PASS**（存量 4 类/4 张表 0 改动，纯追加第 5 类 + §3.5；
   §1.4 第 1/2 条括注追加；第 4 条 cargo bin 豁免注记 = P2-3 修复落点，已补）
g) 22 号 §11.2 平台自身质量 vs 现状：**PASS**（cargo test 0 用例机器绿 = DoD 字面成立；
   「-Check 0 写盘断言」单测载体挂 S32.2 = P2-2，22 号 §11.2 信任项备注已改挂 S32.2）
h) gate 第 8/9 项调用面（`cargo run -p ti-tools --bin ledger/doccheck`）：**PASS**
   （调用面未变；实跑 ledger 262/244/0 error + doccheck 0 error + doccheck2 38 项 0 error 全绿）

## 新发现（5 条 P2，0 P0/P1；全部有文档面修复落点，第 2 版已吸收）

| ID | 严重度 | 位置 | 问题 | 修复落点（第 2 版已落 / 挂后续任务） |
|---|---|---|---|---|
| P2-1 | P2 | console_sources.json vs 22 号 §5 表 | §5 表第 4 行 `assets/gate_data.json` 未进 JSON 单一事实源（S32.2 repo 适配层严格只读 JSON 将漏消费） | 已修：console_sources.json 补 gate_data.json 行（第 2 版） |
| P2-2 | P2 | 22 号 §11.2 信任项备注 | 「S32.1 落码后由 ti-console 单测（-Check 实跑 0 写盘断言）兜底」= S32.1 骨架 0 测试面，该句落点失真 | 已修：改挂 S32.2（repo 适配层 0 写盘断言）（第 2 版） |
| P2-3 | P2 | 14 号 §1.4 契约第 4 条 | 字面把 cargo bin 也纳入「新增工具须含 --check/只读」，机械实扫面（scripts/ .py/.ps1）不含 crates/ ⇒ 文档声称 > 机器覆盖漂移 | 已修：第 4 条补 cargo bin 豁免注记（第 2 版） |
| P2-4 | P2 | console_commands.json 参数面 | close_check/exp_check/audit_book 在 §4.2 表有参数列但 JSON 未建模 enum_param；S32.6 对账若升级参数面将漂移 | 挂 S32.6（对账口径 = id 集合 + 默认启用列先行，参数枚举扩面随 S32.6 定） |
| P2-5 | P2 | 22 号 §10.4 锁文件规格 vs S32.1 落码面 | 规格「pid + 起始时间 + 动作 id + 30 分钟陈旧接管」在 S32.1 骨架 = 仅 pid 最小面，完整规格 S32.4 补 | 已修：§10.4 补「S32.1 骨架 = pid 最小面；完整规格 S32.4 并发面补全」在途标注（第 2 版） |

## 实跑验证（9 条全绿；红项归并行会话在途）

build -p ti-console exit=0 / test -p ti-devmeta --lib 8/8 / test -p ti-console 0 用例 PASS /
clippy ti-devmeta+ti-console 0 warning / cargo tree 0 游戏 crate / ledger 244 verified 0 error /
doccheck 0 error / doccheck2 0 error（38 项）/ check_code_standards PASS / check_ledger_reality 全绿 /
doc_numbers 0 漂移。i18n 孤儿 key WARN ×14 = 并行会话 strings 在途线，非 error 非 S32.1 面。

## 总判定

**FAIL（非收敛轮，0 阻断级）**：一致性面 a~h 全 PASS（S32.1 自身面 0 矛盾，d/e 项并行会话
在途改动已剥离归属）；新发现 5 条 P2（P2-1/P2-2/P2-3/P2-5 文档面已随第 2 版吸收，P2-4 挂 S32.6）
⇒ 按 §7.3 计数口径未达「0 新发现 + 0 误报 + 0 残留」；第 2 轮复核。
