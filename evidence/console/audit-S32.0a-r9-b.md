# S32.0a 第 9 轮审计 · B（一致性与风险）

```
总判定: FAIL
新发现: 2
误报: 0
残留: 0
```

> 落盘说明：第 9 轮 B 侧审计 findings 落盘凭证（会话内报告结构化汇总，全量落盘）。只读；未跑 gate.ps1；
> 实跑 doccheck / doccheck2 / ledger / check_code_standards / check_ledger_reality / doc_numbers /
> map_pack.py list + check。

## 第 8 轮 2 条 P2 闭环判定

- **P2-8（S32.12 m04 处置口径）CLOSED**：三段口径对照 `map_pack.py cmd_check` L415-431 实算成立
  （m04 无 map.json ⇒ 恒通过；填后哈希不一致须 sync；已 install 报漂移才 uninstall；实测 check = 0 漂移 4 包 exit 0）。
- **P2-9（依赖方向）NOT-CLOSED，升级 P1**：台账 S32.11 deps=[S32.12] 方向反（S32.12 DoD 含「gate 全绿」
  应依赖 S32.11）+ 后向引用致 ledger 实跑 1 error + WBS↔台账漂移。

## 一致性面 a~g

a) §7.4 vs loop §4 vs ledger.rs：**PASS**
b) 台账 13 行 deps 链 + 终止条件：**FAIL**（S32.11 deps 后向引用，12/13 行合法）
c) §3.1 写点表穷举：**PASS**（6 行逐脚本核对吻合）
d) 22 号 vs 既有单一事实源（parity 31 / 14 / 16 / 06 / migration_linux / 21 / 18）：**PASS**
e) check_process_ledger 勘误 + 15 号勘误注：**PASS**
f) §7.3 三口径 vs 15 份报告可解析性：**PASS**（r1~r3 祖父 / r4~r8 四行齐全 / r6-b 值域外不计轮 / 15 号 149 可复算）
g) 台账「；判据=」格式 vs check_task_change_scope：**PASS**（13/13，实跑 0 error）

## 新发现（1 P1 + 1 P2）

| ID | 严重度 | 问题 | 修复落点（第 10 版已落） |
|---|---|---|---|
| P1-1 | P1 | S32.11 deps=[S32.12] 方向反 + 校验器实跑 1 error + WBS↔台账漂移（三重） | S32.11 deps=[] / S32.12 deps=[S32.11]；WBS L38 依赖列 = S32.11 + 执行顺序说明改「S32.11（先修门禁）→ S32.12（后修地图包，DoD 含 gate 全绿须第 17 项可信）」；22 号 §4.3 补「当前未根治，在途」状态 |
| P2-4 | P2 | 22 号 §4.3 已知缺陷段未标当前态（「根治另立项」易被读成已根治） | 改「根治已立项（台账 S32.11，**当前未根治，在途**）」 |

（P2-5「S32.12 DoD gate 全绿依赖语义」随 P1-1 的 deps 修正自动自洽，不单列。）

## 实跑验证（唯一红项 = P1-1 的 ledger 校验 1 error，归属 S32 台账自身；
CS-2 违例归属并行会话 android 线在途，在基线内不红）

doccheck 0 error / doccheck2 0 error（38 项）/ ledger **1 error**（P1-1）/ check_code_standards PASS /
check_ledger_reality 全绿 / doc_numbers 0 漂移 / map_pack.py list = m01~m03 各 22 项 + m04 11 项 /
map_pack.py check = 0 漂移（4 包）。

## 总判定

**FAIL**（新发现 2（P1×1 + P2×1）⇒ 未达 0 新发现 + 0 误报 + 0 残留；修复后第 10 轮复核）。
