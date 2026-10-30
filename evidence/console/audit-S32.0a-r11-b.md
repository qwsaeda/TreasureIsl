# S32.0a 第 11 轮审计 · B（一致性与风险）

```
总判定: FAIL
新发现: 3
误报: 0
残留: 0
```

> 落盘说明：第 11 轮 B 侧审计落盘凭证（会话内报告结构化汇总）。只读；未跑 gate.ps1；实跑
> doccheck / doccheck2 / ledger / check_code_standards / check_ledger_reality / doc_numbers /
> map_pack.py list + check。

## 第 10 轮 2 条 P2 闭环：2/2 CLOSED

N-1（WBS L38 S32.12 依赖列 = S32.11，与 L16/L65/L67 三处一致 + 台账 307/308 前向合法 + ledger 实跑 0 error）CLOSED；
N-2（15 号计数 152→149 修正）CLOSED，且 r10 后更新为 **153 可复算**（38+16+29+22+25+9+7+3+3+4，逐份落盘报告复算吻合）。

## 一致性面 a~g

a) §7.4 两计数器 vs loop §4 vs ledger.rs：**PASS**（三方逐句对齐；台账最大 rounds = 1）
b) 台账 13 行 deps 链（全前向 DAG，ledger 实跑 0 error）+ 终止条件可满足：**PASS**
c) §3.1 写点表 6 行逐脚本核对（含 gate 第 23 项无 Tier 守卫实测）：**PASS**
d) 22 号 vs 既有单一事实源：**FAIL**（门禁 31→33 项漂移在 S32 文档 3 处写死「31 项」的当前口径 ⇒ D-1）
e) check_process_ledger 勘误 + 15 号勘误注：**PASS**（并行会话已同步 doccheck2 gate-items 阈值 32→33，
   与代码现状自洽）
f) §7.3 三口径 vs 21 份报告：**PASS**（r1~r3 祖父 / r4~r10 四行齐全 / r2-b、r6-b 占位值域外）
g) 台账「；判据=」格式 vs check_task_change_scope：**PASS**（13/13，实跑 0 error）

## 新发现（3 条 P2，由第 12 版文档吸收）

| ID | 严重度 | 问题 | 修复落点 |
|---|---|---|---|
| D-1 | P2 | 门禁 31→33 项后，S32 文档 4 处写死「31 项」当前口径（22 号 §4.2 L111 / WBS L58 / 台账 S32.12 标题 L308 / 台账 S32.11 DoD「其余 30 项」），且均在 doccheck2 gate-items 与 doc_numbers 双机器兜底盲区 | 各改「项数以 parity.md 为准（当前 33 项）」抗漂移表述；根治 = S32 文档声称点纳入机器面（建议后续 doc_numbers 扩面，非 S32.0a 范围） |
| D-2 | P2 | 22 号 §4.3 行号锚 L261 已漂至 L279（并行会话插入第 33 项后）；漂移标注措辞本身有效，但 S32.11 实施若按文档时点值定位会落错行 | 锚值改「S32.11 开工时重实测行号回填，勿按文档时点值定位」 |
| D-3 | P2 | 22 号 §4.2 白名单表备注「build_wasm -Check 不写 evidence」「map_pack install 条件写」= 信任项，无机器断言（非 S32 判据，属备注信任） | 22 号 §11.2 补「信任项备注（无机器断言，如实标注）；S32.1 落码后由 ti-console 单测兜底」 |

## 实跑验证（本时点全绿；gate.ps1 第 33 项同步线 / ti-engine / ti-platform / android 线若跑红 = 并行会话在途，不计 S32）

doccheck 0 error / doccheck2 0 error（38 项，gate-items 阈值 33 已随并行会话同步）/ ledger 0 error（262 任务）/
check_code_standards PASS / check_ledger_reality 全绿 / doc_numbers 0 漂移（实测 33 项）/
map_pack.py list = m01~m03 各 22 项 + m04 11 项 / map_pack.py check = 0 漂移（4 包）。

## 总判定

**FAIL（非收敛轮）**：第 10 轮 2/2 CLOSED + 一致性面 a/b/c/e/f/g PASS（d FAIL）；新发现 3 条 P2
（D-1 与 A 侧 N-1~N-3 同源同批，不重复计数时按一侧计）⇒ 未达「0 新发现 + 0 误报 + 0 残留」；第 12 轮复核。
