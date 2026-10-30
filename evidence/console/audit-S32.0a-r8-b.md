# S32.0a 第 8 轮审计 · B（一致性与风险）

```
总判定: FAIL
新发现: 2
误报: 0
残留: 0
```

> 落盘说明：第 8 轮 B 侧审计 findings 落盘凭证（会话内报告结构化汇总）。只读；未跑 gate.ps1
>（第 23 项 browser-playtest 无分档守卫会改写 git 跟踪文件）；实跑 doccheck / doccheck2 / ledger /
> check_code_standards / check_ledger_reality / doc_numbers / map_pack.py list + check。

## 第 7 轮 3 条 P2 建议：3/3 CLOSED

P2-1（S32.12 m04 处置）CLOSED / P2-2（§3.1 .default 条件写）CLOSED / P2-3（§7.3 值域口径 + S32.5 断言）CLOSED。

## 一致性面 a~g：全 PASS

a) §7.4 两计数器 vs loop §4 vs ledger.rs（rounds>=3 报 error）三方逐句对齐 ✓
b) 台账 13 行 deps 链 + loop §3 终止条件可满足（P3 不入台账；S32.11/12 与 S16.4 无互斥）✓
c) §3.1 写点表穷举（gate/task_close/exp_log/build_wasm/map_pack 逐脚本核对）✓
d) 22 号 vs 既有单一事实源（parity 31 项 / 14 / 16 / 06 / migration_linux / 21 / 18）无冲突无重复 ✓
e) check_process_ledger 勘误正确 + 15 号勘误注如实 ✓
f) §7.3 位置/值域/计数三口径 vs 15 份报告可解析性（r6-b 占位值域外不计轮）✓
g) 台账「；判据=」格式 vs check_task_change_scope 切段（13/13 行，实跑 0 error）✓

## 新发现（2 条 P2）

| ID | 严重度 | 问题 | 修复落点（第 9 版已落） |
|---|---|---|---|
| P2-8 | P2 | S32.12 DoD「m04 若先入库会报 m04 段漂移 ⇒ 先 uninstall m04」技术前提偏差：实测 `check` 对无 map.json 的 m04 不产生漂移（manifest.map 与 board maps.m04 均为空） | DoD 改精确口径：m04 无 map.json 时 check 恒通过；填完 map.json 且哈希不一致须先 sync；已 install 且报漂移才 uninstall |
| P2-9 | P2 | S32.12 deps=[] 与 S32.11 无依赖 ⇒ 可能在 S32.11 之前执行，而 S32.12 DoD 含「gate 全绿」（第 17 项在 Linux 上假 PASS 未根治前不可信） | S32.11 deps 改 `[S32.12]`（先修门禁假 PASS，再修地图包漂移；与「旁支先于平台本体」取件顺序一致） |

## 实跑验证

doccheck 0 error / doccheck2 0 error（38 项）/ ledger 0 error（262 任务）/ check_code_standards PASS /
check_ledger_reality 全绿 / doc_numbers 0 漂移 / map_pack.py list = m01~m03 各 22 项 + m04 11 项 /
map_pack.py check = 0 漂移（4 包，exit 0）✓。无归属并行会话在途改动的新红项。

## 总判定

**FAIL**（新发现 2（P2）⇒ 未达 0 新发现 + 0 误报 + 0 残留；第 9 轮复核；0 阻断级）。
