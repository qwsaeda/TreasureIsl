# S32.0a 第 7 轮审计 · B（一致性与风险）

> 落盘说明：第 7 轮 B 侧审计（r6 的 B 侧因额度中断未出报告，本轮是 B 视角首次完整覆盖）。
> 审计方式：只读；未跑 `scripts/gate.ps1`；实跑 doccheck / doccheck2 / ledger /
> check_code_standards / check_ledger_reality / doc_numbers / map_pack.py list。

```
总判定: FAIL
新发现: 3
误报: 0
残留: 0
```

> 说明：本份 3 条均为 **P2 级口径精化建议**（0 P0/P1，无阻断级）。
> 按 22 号 §7.3「计数口径」（P2 建议计入新发现），该侧新发现 = 3 ≠ 0 ⇒ 该轮未收敛，
> 四行块 `总判定` 如实写 **FAIL**；3 条建议由第 8 版文档吸收，第 8 轮复核。

## 一致性面逐项判定（a~g 全 PASS）

a) §7.4 两计数器口径 vs loop.md §4 vs `ledger.rs` 校验器：**PASS**（三方逐句对齐；当前台账最大 rounds=2，无 >=3 行）
b) 台账 13 行 deps 链 + loop §3 终止条件可满足性：**PASS**（P3 不入台账；S32.11/S32.12 与 S16.4 无互斥；S32.12 判据 m01~m03 与 m04 空骨架口径兼容 ⇒ 见 P2-1 的 m04 check 面精化）
c) §3.1 写点表穷举性：**PASS**（gate / task_close / exp_log / build_wasm / map_pack 逐脚本核对吻合；`.default` 条件写 ⇒ P2-2 微瑕疵）
d) 22 号 vs 既有单一事实源（parity 31 项 / 14 号工具契约 / 16 号 CS / 06 飞轮停机 / migration_linux 映射 / 21+18 号引用）：**PASS**
e) `check_process_ledger` 任务号勘误：**PASS**（旧 `contains("S2")` 确漏 S32.x；新通用模式对存量 20~23 行判定无漂移；15 号勘误注如实）
f) §7.3 四行位置口径 vs r4~r6 实际报告：**PASS**（r4/r5 五份全部可被解析器命中；r1~r3 六份无 fenced 块 ⇒ 正确落入祖父条款；r6-b 占位值域外值 ⇒ P2-3）
g) 台账 S32 行「；判据=」格式 vs `check_task_change_scope` 切段：**PASS**（13/13 行声明段命中面族，实跑 0 error）

## 新发现（3 条 P2）

| ID | 严重度 | 位置 | 问题 | 修复落点（第 8 版已落） |
|---|---|---|---|---|
| P2-1 | P2 | 台账 S32.12 DoD + WBS 任务表 | 「`map_pack.py check` 0 漂移」对全部 `installed_packs()`（含 m04 草稿）⇒ m04 若先入库会在**本任务文件域外**红；DoD 未声明 m04 在 check 面的处置 | S32.12 DoD 补「m04 草稿段不计；m04 已入库则先 `uninstall m04` 再跑 check」 |
| P2-2 | P2 | 22 号 §3.1 写点表 map_pack 行 | `.default` 被写成 install/uninstall 共同**无条件**写点，实测为条件写（install 缺时 / uninstall 仅卸默认包时） | 该行补「条件写」备注 |
| P2-3 | P2 | 22 号 §7.3 报告格式契约 | 四行契约未定义 `总判定` **值域**；r6-b 占位写 `中断（无报告）`（值域外值），若 S32.5 解析器不把值域异常判「不可解析」，占位轮的 0/0/0 会被误读成有效轮 | §7.3 补值域口径（非 PASS/FAIL ⇒ 该轮不计入收敛）+ S32.5 单测断言 |

## 实跑验证（未跑 gate.ps1）

`doccheck` 0 error / `doccheck2` 0 error（38 项）/ `ledger` 0 error（262 任务）/
`check_code_standards` PASS（CS-2 红项归属并行会话在途改动 `ti-engine/src/bin/android_entry.rs`，非 S32 引入，已标注）/
`check_ledger_reality` 全绿 / `doc_numbers` 0 漂移 / `map_pack.py list` = m01~m03 各 22 项 + m04 11 项（HEAD 既有 I7 漂移在位，= S32.12 立项前提）。

## 总判定

**FAIL（未收敛，0 阻断级）**：0 P0/P1；3 条 P2 建议由第 8 版吸收；并行会话红项已标注归属。
