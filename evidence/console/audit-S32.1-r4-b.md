# S32.1 第 4 轮审计 · B（一致性与风险）

```
总判定: PASS
新发现: 1
误报: 0
残留: 0
```

> 落盘说明：第 4 轮 B 侧审计落盘凭证（会话内报告全量落盘；1 条 P2 = N4-1 计数回填提示，
> 收口时同批吸收）。只读；实跑 13 条（12 绿 + 1 黄〔环境：`.lock` 并行会话占用，服务实例面
> 由 r1 实跑证据 + 路由 0 改动支撑，如实标注〕）；未跑 gate.ps1 全量；并行会话在途面
> （S18.8d/e android/LTO + strings i18n + S32.12 I7 修复 + .gitignore）逐条剥离归属。

## r3 B 侧 3 条闭环：3/3 CLOSED

| ID | 判定 | 证据 |
|---|---|---|
| N-1（P0，console_commands.json L26 JSON 非法） | **CLOSED** | 实跑 `json.load` = OK；L26 = `"writes": ["plan/ledger.yaml", "docs/release.md", "evidence/"],`（尾逗号在位 + 行尾 `#` 注释已并入 L2 `_comment`，JSON 内 0 行尾注释） |
| N-2（P2，L2 `_comment` 未含写面前缀口径） | **CLOSED** | L2 `_comment` 实测含「写面列（前缀形态条目按前缀匹配，不展开模板：close_run 写面第 3 项 `evidence/` = 运行时按台账 id 展开为 evidence/<task>.log）」，与机器侧 L26 第 3 项逐字对应 |
| N-3（P2，22 号 §4.2 L94 close_run 写面列未同步） | **CLOSED** | L94 写面列实测含「机器真源写面 = `[plan/ledger.yaml, docs/release.md, evidence/前缀]`，`evidence/` 为前缀形态：运行时按台账 id 展开，S32.6 对账按前缀匹配」，与前两条同口径三方一致 |

## 一致性面 a~h（8/8 PASS，第 4 版 3 处改动 0 新矛盾）

a) §4.1 vs ti-devmeta::http：**PASS**（0 异步栈：http.rs = std TcpListener + thread::spawn/
   thread::scope；两 crate 全源 grep async/tokio/axum 仅 2 处 doc 注释「不引 axum/tokio」；
   依赖面 = ti-devmeta 0 外部依赖 + ti-console 仅 serde_json）
b) §4.2 白名单 9 行 vs console_commands.json：**PASS**（JSON 合法后机器可解析：id 集合 9=9；
   写面语义 0 漂移（人读面 `[plan/ledger.yaml, docs/release.md, evidence/前缀]` ↔ JSON 第 3 项
   `"evidence/"` 前缀同口径）；默认启用列一致（close_run = false + `requires_flag: --enable-write`
   ↔ §4.2 L94「否」+ L103「当前仅 close_run 一条」）；「两份必须一致」机器侧现已成立
   （S32.6 可加对账断言））
c) §5 vs console_sources.json：**PASS**（9 路径 9 行逐条 1:1 展开；write_whitelist =
   `evidence/console/` ↔ §3.1 唯一写点一致）
d) 根 Cargo.toml S32.1 面：**PASS**（git diff 证实 members 纯插入 3 行〔L10 注释 + L11/12 两
   member 行〕；L21「S18.8d 收口」与 L27 一致（r2-A P2-01 闭环 0 回潮）；android/LTO/
   macroquad 删除面 = 并行 S18.8d/e 在途，剥离 0 破坏）
e) ti-tools 薄 re-export 面：**PASS**（lib.rs 5 符号 1:1；ti-tools/src/ledger.rs 已删；
   serve.rs 对 HEAD 0 改动；bin/ledger.rs 纯调 ti_devmeta::；实跑 --bin ledger =
   262 任务 / 244 verified / 0 error / evidence ok，行为不变）
f) 14 号 §1.4 第 4 条豁免注记 vs check_tooling_contract 机器扫面：**PASS**（豁免措辞「实扫面 =
   scripts/ 的 .py/.ps1 不含 crates/」与 doccheck2 实扫面（read_dir scripts + is_tool_script
   只吃 .py/.ps1 + `__` 前缀排除；stray-scan SKIP_DIRS 使 crates/ 无散落脚本）逐字相符
   ⇒ 0 误报；实跑 doccheck2 38 项 0 error）
g) §11.2 平台自身质量 vs 现状：**PASS**（workspace 成员自动覆盖〔Cargo.toml L11/L12 +
   lints.workspace〕；web/* 排除挂 S32.3 兜底；CS-1 0 命中（`grep assets/ crates/ti-console/src/`
   = 0 命中）；机器面盲区如实标注；信任项挂 S32.2 与现状 0 测试面一致）
h) gate 第 8/9 项调用面：**PASS**（--bin ledger 0 error / --bin doccheck 0 error（148 docs）/
   --bin doccheck2 38 项 0 error〔i18n 孤儿 4 条 WARN = 并行 strings 在途线，非 error〕/
   check_code_standards PASS / check_ledger_reality 全绿 / doc_numbers 0 漂移）

## 新发现（1 条 P2，非阻断，收口时同批吸收）

| ID | 严重度 | 位置 | 问题 | 修复落点 |
|---|---|---|---|---|
| N4-1 | P2 | 15 号 L54 第 21 行 S32.1 侧计数 | findings 计数止于「r1 12 + r2 3 + r3 5 = 20」，未含 r4 侧结果（r4 A/B = 0/1，主线程落盘报告后才完整）；当前 214 总额与 r3 时点一致（0 漂移），但 S32.1 收口前须回填 r4 计数行，否则 S32.5 机械判定读台账时缺一轮 | S32.1 收口（task_close）时按 §7.3 机器可解析字段回填 r4 A/B 的「新发现/误报/残留」+ 214 总额更新（主线程同批，0 阻断） |

## 实跑验证（13 条：12 绿 + 1 黄〔环境〕）

json.load ×2 OK / build exit=0 / test ti-devmeta 8/8 / test ti-console 0 用例 PASS /
clippy 0 warning / cargo tree 0 游戏 crate / ledger 262/244/0 error / doccheck 0 error /
doccheck2 38 项 0 error（4 WARN i18n = 并行在途）/ CS PASS / check_ledger_reality 全绿 /
doc_numbers 0 漂移。

## 总判定

**PASS（非收敛轮，1 条 P2 = 计数回填提示）**：r3 3/3 CLOSED + 一致性面 a~h 8/8 PASS
（第 4 版 3 处改动 0 新矛盾）+ 实跑 12 条全绿；新发现 1 条 P2（N4-1，收口时同批吸收）
⇒ 按 §7.3 计数口径本轮非 0 新发现轮；第 5 轮复核（r5 0/0/0 + r4 0 新发现 0 误报 0 残留
= 连续 2 轮达标后主线程独立复核收口）。
