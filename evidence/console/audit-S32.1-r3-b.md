# S32.1 第 3 轮审计 · B（一致性与风险）

```
总判定: FAIL
新发现: 3
误报: 0
残留: 0
```

> 落盘说明：第 3 轮 B 侧审计落盘凭证（会话内报告全量落盘；A 侧同轮 = `audit-S32.1-r3-a.md`，
> 同源根因（r2-B P2-1 修复引入的 JSON 回归）双方各自报告，去重后 = 1 P0 + 2 P2，
> 主线程第 4 版同批吸收）。只读；实跑 13 条（12 绿 + JSON 校验 1 FAIL = N-1）；
> 未跑 gate.ps1 全量；并行会话在途面（S18.8d/e/strings i18n/S32.11/12 旁支）剥离归属。

## r2 B 侧 2 条 P2 闭环

| ID | 判定 | 证据 |
|---|---|---|
| P2-1（close_run 写面前缀形态 + 注释） | **PARTIAL（意图对，语法回潮 = N-1）** | L26 前缀 `evidence/` + 行尾注释在位，但 `#` 行尾注释 JSON 非法（实跑 `JSONDecodeError line 26 col 72`）；根因 = 修 P2-1 时未复用 L2 `_comment` 先例 |
| P2-2（§10.4 L324 补 S32.4 限定） | **CLOSED** | L322 + L324 两层边界自洽（pid 最小面 / 完整规格 = S32.4），main.rs L130 同步 |

## 一致性面 a~h（7 PASS / 1 FAIL）

a) §4.1 vs ti-devmeta::http：**PASS**（0 异步栈，thread::spawn 每连接一线程）
b) §4.2 9 行 vs console_commands.json：**FAIL（被 N-1 拖累；字面内容 0 漂移）**
   （id 9=9 / 写面语义 0 漂移 / `requires_flag` ↔ §6.1 写操作开关一致；
   机器真源当前不可解析 ⇒「两份必须一致」机器侧不成立）
c) §5 数据源 9 路径 vs console_sources.json：**PASS**（gate_data.json 补后 0 漏；
   write_whitelist = evidence/console/ 一致）
d) 根 Cargo.toml S32.1 面：**PASS**（纯插入 3 行；L21 修正后与 L27 一致；
   android/LTO 面 = 并行 S18.8d/e 在途，剥离 0 破坏）
e) ti-tools 薄 re-export：**PASS**（逐字节 0 差异 + 5 符号 1:1 + serve.rs 0 改动；
   实跑 ledger 262/244/0 error 行为不变）
f) 14 号 §1.4 第 4 条豁免注记 vs check_tooling_contract 实扫面：**PASS**（scripts/ .py/.ps1
   不含 crates/，0 误报；实跑 38 项 0 error）
g) §11.2 vs 现状：**PASS**（0 用例机器绿 + 信任项备注挂 S32.2）
h) gate 第 8/9 项调用面：**PASS**（ledger/doccheck 0 error / doccheck2 38 项 0 error
  〔i18n 孤儿 14 条 WARN = 并行 strings 在途〕/ CS PASS / 三方对账全绿 / doc_numbers 0 漂移）

## 新发现（3 条：1 P0 + 2 P2，均 S32.1 自身面，主线程第 4 版同批吸收）

| ID | 严重度 | 位置 | 问题 | 修复落点 |
|---|---|---|---|---|
| N-1 | P0 | console_commands.json L26 | r2-B P2-1 修复行尾 `#` 注释 ⇒ JSON 非法（`JSONDecodeError line 26 col 72`）；机器真源失格 | 已修：注释并入 L2 `_comment` + L26 补尾逗号（`json.load` 复验 OK） |
| N-2 | P2 | 同文件 L2 `_comment` | 对账口径只提「id 集合 + 默认启用列」，未提写面列前缀形态；S32.6 实现者可能误按精确路径断言 | 已修：L2 补「写面列前缀形态条目按前缀匹配，不展开模板：close_run 第 3 项 evidence/ = 运行时按台账 id 展开」 |
| N-3 | P2 | 22 号 §4.2 L94 close_run 行 | 人读面写面列仍散文「脚本自身写台账+计数+证据日志」，未同步机器真源 3 项前缀形态 | 已修：写面列补 `[plan/ledger.yaml, docs/release.md, evidence/前缀]`（前缀形态口径） |

## 实跑验证（12 绿 / 1 FAIL = N-1，修复后全绿）

build ✅ / test ti-devmeta 8/8 ✅ / test ti-console 0 用例 ✅ / clippy 0 warning ✅ /
cargo tree 0 游戏 crate ✅ / ledger 0 error ✅ / doccheck 0 error ✅ / doccheck2 0 error（38 项）✅ /
CS PASS ✅ / 三方对账全绿 ✅ / doc_numbers 0 漂移 ✅ / JSON 校验 ❌（N-1，主线程第 4 版已修）。

## 总判定

**FAIL（非收敛轮，1 P0 阻断已修）**：P2-1 部分闭环（语法回潮 = N-1）/ P2-2 完整闭环；
新发现 3 条（1 P0 + 2 P2）全部主线程第 4 版同批吸收；第 4 轮复核（0 新发现 + 0 误报 + 0 残留
连续 2 轮 = r4+r5 后主线程独立复核收口）。
