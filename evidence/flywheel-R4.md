# 飞轮 R4（2026-11-12，R3 修复面复验轮）

## 开轮前置
- 基线全绿：gate R3 全绿 + doccheck2 44 项 0 error + doccheck 0 + 三方对账 337
- R4 域 = R3 修复面复验 + 全库新引入面（B-03 i18n 化连锁 / N-A2 重写牵连 / 26号声称点 / 动态 key 面）

## A/B 双审计对抗（只读，独立并行不读对方结论）
**A 侧（完成度）**：
- R3 7 修复点全 0 残留复验 ✓（B-03 i18n 化 0 其它玩家可见 CJK 字面量 / N-A2 重写 0 3图残留 / F-R3-01 四处改后 0 第 5 处 42 残留 / F-R3-03 头 doc 在位 / B-02 --list 真可用+对齐台账 / F-R3-02 文件级豁免+台账留账 / flywheel_readme 基准 44 项对齐）
- **A 侧独报（2 P2）**：
  - **P2-A1（N-A2 修复面未收全）**：`docs/business/territory.md:215` 存档 v5 字段表 `unlocked_mask u8` vs 代码 `persistence.rs:34 pub unlocked_mask: u32`（S37 起 20 图口径）→ 业务文档 3 图残留第 3 处（progression.md 段已改，territory.md 字段表漏收）
  - **P2-A2（动态 isle.* key 前瞻盲区）**：`board.rs:519/728 sget(&format!("isle.{isle}"))` 动态拼接 key 不在 check_i18n_orphans 字面量扫描面（isle. 前缀已豁免），strings.json 缺 `isle.<类名>` = 静默空串（玩家可见面）→ 前瞻防护须加「balance 岛原型 10 类 × isle.* key 实存」断言

**B 侧（一致性）**：
- 三方一致性交叉验（③文档↔④数据↔①代码）：R3 修复面无新漂移 ✓
- **B 侧独报（1 P1）**：
  - **P1-B1（26号声称点值漂移）**：`docs/process/26_dev_stages_overview.md:141`「doccheck2（42 项…）」vs 实测 44 项（gate_data.json doccheck2_checks = 44）；且 cross-doc-counts 的 gate 上下文谓词（`line.contains("gate.ps1")||"门禁 36"||"固定 Report"`）对该行 = False（26号:141 是「文档一致性」工具表，无 gate 关键词）⇒ 该漂移不在 cross-doc-counts 拦截面 → P1 值漂移（gate 盲区）

## 修复落地（写码面，主线程裁决 3 项全修）
| 修复 | 落点 | 判据 |
|---|---|---|
| P2-A1 territory.md u8→u32 | `docs/business/territory.md:215` `unlocked_mask u8`→`u32`（S37 20 图口径，与 progression.md L279 同措辞 + u8 旧档高 24 位=0 兼容注） | 代码 persistence.rs:34 实查 u32 ✓ |
| P1-B1 26号 42→44 项 | `docs/process/26_dev_stages_overview.md:141`「doccheck2（42 项…）」→「44 项（gate_data.json 单一事实源）」 | 实测 44 ✓ |
| P2-A2 isle.* key 实存断言 | `doccheck2.rs check_isle_keys_present` 新增独立子函数（balance.territory 岛原型 10 类 × strings isle.<id> 实存对账，缺即 FAIL；check_i18n_orphans 尾部 `errors.extend(...)` 接线） | 负例护栏：造幽灵岛类 `r4_ghost_isle` 无 key ⇒ doccheck2 FAIL✓（断言非恒 PASS） |

## 停机判定（06_flywheel §4）
- R4 = 飞轮第 2 轮（R3 第 1 轮 B 侧独报 N-A2/N-A4/N-A5，本轮 R4 A/B 仍独报 P2-A1/P2-A2/P1-B1）
- **A/B 连续 2 轮 0/0/0 未达成**（R4 仍有 3 findings，修复面 0 残留但审计面有独报）→ 须再开 R5 复验轮
- R5 收敛口径：R5 的 A/B 双审计对「R4 修复面 3 项 + 全库」均 0 findings（0 新漂移 + 0 误报 + 0 残留）= 连续 2 轮（R4+R5 都 0/0/0）才停机
- 主线程复核（EXP-038/058）：P2-A1（territory.md:215 读 + persistence.rs:34 实查）/ P1-B1（26号:141 读 + gate_data.json 44 实测 + cross-doc-counts 谓词盲区确认）/ P2-A2（board.rs:519/728 动态 key 实查 + 负例护栏造幽灵岛类 FAIL✓）全主线程实跑核实
