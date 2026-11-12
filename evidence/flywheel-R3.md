# 飞轮 R3（2026-11-12，S52 收口后全域轮）

## 开轮前置：flywheel_readme 基准修正（开轮即抓 2 真漂移）
- **漂移①（P1）**：`flywheel_readme.ps1` doccheck2 清单段写死「52 项」，实跑 44 项（脚本数所有 `fn check_` 定义 52，含 8 个未注册子 helper）
- **漂移②（P1）**：gate 项清单第 20/21 项 `no-macroquad` 重复（PASS/FAIL 双分支各 1 行，未按项名去重）
- 修法 = `Read-Doccheck2Checks` 改读 `gate_data.json doccheck2_checks`（44，单一事实源）+ `Read-GateItems` 按完整项名去重（36，与 doccheck2 gate_item_actual_count 同口径）
- 负例护栏：gate +1 Report → readme 36→37 ✓ / gate_data +1 断言 → 44→45 ✓

## 找问题 agent（只读）发现 3 条
| 发现 | 类别 | 裁决 |
|---|---|---|
| F-R3-01 `15_process_ledger.md:67` + `AGENTS.md:54` + `evidence/S52.1.log:2` 声称「territory_v2 42 内联迁移」实测 18+4=22（虚报计数）| P1 值漂移 | **真漂移**（主线程 git show e1cf0f8^ 复核）|
| F-R3-02 `check_hardcode.py` territory_v2.rs 整文件 CJK 豁免过宽 | P2 需人裁决 | **需人裁决**（用户拍板 = 维持文件级 + 文档化约束 + S53 行级收窄留账）|
| F-R3-03 `territory_v2_rules.rs` 文件头 doc 漏 `ai_turn_v2`（3 死代码只描述 2 个）| P2 需人裁决 | **真漂移**（头 doc 补 1 行披露）|

## A/B 双审计对抗（只读，独立并行不读对方结论）
**A 侧（完成度）**：F-R3-01 真漂移 ✓ / F-R3-02 需人裁决 ✓ / F-R3-03 真漂移 ✓
- A 侧补找盲区（找问题 agent 漏报面）：
  - **B-03 P0**：`board.rs:565 String::from("无主")` 玩家可见 CJK 硬编码（绕过 check_hardcode 双通道：`String::from` 不走 STR_LIT 字面量捕获 + game_app/ 目录 cjk 豁免）→ i18n 化 `role.name.none`
  - B-01：4 处「42」内联计数漂移（同 F-R3-01 同值过期）
  - B-02：check_hardcode `--list` 子命令缺失（14号/04号声称可用但脚本无）
  - B-04 P3：`flywheel_readme.ps1:179`「RUST_CHECKS 42 项」注释陈旧
**B 侧（一致性）**：三方一致性交叉验 ③文档↔④数据↔①代码：
- F-R3-01/02/03 三方对账 = 全确证
- **B 侧独报（A 侧完成度看不到的值漂移面）**：
  - **N-A2 真漂移**：`docs/business/progression.md` §跨图推进 段整体 = S15.2 时点 3 图规格（m03 通关 / u8 掩码 / bit 3 CLEARED / 10 个旧测试名 0 命中 tests/），现行代码 = S33/S37 20 图（m20 通关 / u32 / bit 19 / 8 个现行测试名）→ 业务文档滞后 1 代
  - N-A4：check_hardcode.py EXEMPT 表 territory_v2.rs 重复行（实查 = 仅 1 处 L71，B 侧误报，无需去重）
  - N-A5/B-04：`flywheel_readme.ps1:179`「RUST_CHECKS 42」= 旧值残留（S52 轮已改「42 RUST + 2 grep = 44」对齐）
- i18n 白名单 5 条全部真消费点核验（无白名单孤儿遮蔽）✓
- cross-doc-counts release.md 声称点逐字对账 ✓
- gate_data.json doccheck2_checks 44 = 实跑 44 ✓
- **B 侧误报 1 条**：`01_game_logic_overview.md:171` 状态图「M03_unlocked→M04」被 B 侧报为 m03 通关滞后，实查 = 20 级解锁链正常节点名（通关点已正确 CLEARED via m20/bit19），overview 状态图已是现行 20 图规格 = 误报

## 修复落地（写码面，用户拍板 8 项处置）
| 修复 | 落点 | 状态 |
|---|---|---|
| F-R3-01 四处「42 内联」→「18 内联迁移 + 4 新增共 22」 | `15_process_ledger.md:67` + `AGENTS.md:54` + `evidence/S52.1.log:2` + `territory_v2_rules.rs` 文件头 | ✓ 全改，grep 0 残留 |
| F-R3-01 固化断言 | `doccheck2.rs check_cross_doc_counts` 新增 claims 锚点「territory_v2 18 内联迁移」（负例：改 42 必 FAIL 已实跑）| ✓ |
| B-03 无主 i18n 化 | `board.rs:565 String::from("无主")` → `crate::i18n::sget("role.name.none")` + `strings.json` 新增 `role.name.none: "无主"`（indent=1）| ✓ board.rs 0 玩家可见 CJK 字面量 |
| F-R3-03 头 doc 补披露 | `territory_v2_rules.rs` 文件头补「ai_turn_v2 全库 0 引用」| ✓ |
| F-R3-02 文件级豁免文档化 | `14_tooling_catalog.md` check_hardcode 行注明「文件级宽豁免，命中 0 行，S53 行级收窄留账」| ✓ |
| N-A2 业务文档滞后 | `progression.md` §跨图推进 段全段重写（3图→20图/u8→u32/bit3→bit19/10旧测试名→8现行测试名）+ `cross_map_demo.rs` 打印文案 3图→20图 | ✓ |
| B-02 --list 子命令 | `check_hardcode.py` 补 `--list` 按文件分组汇总（对齐 14号/04号台账声称）| ✓ |

## 停机判定（06_flywheel §4）
- 找问题 = 3 findings 全裁决（1 P1 + 2 P2）
- 审计 A/B：A 侧 0 独报盲区外漂移；B 侧独报 N-A2（P1 业务文档滞后，已修）+ N-A4/N-A5（实查误报/已对齐）
- **A/B 连续 2 轮 0/0/0 未达成**（R3 第 1 轮 B 侧独报 N-A2）→ **R4 必开**（N-A2 修复后复验，确认 0 新漂移 + 0 误报才停机）
- 主线程亲自复核（EXP-038/058）：B-03（board.rs:565 读文件 + 渲染面 main_map.rs:1947 + strings.json 缺 key）/ N-A2（progression.md 全段 + cross_map_demo.rs MapSlot::all()=20 + SaveSlot.u32 实查）/ F-R3-01（git show e1cf0f8^ #[test]=18）全主线程实跑核实，非子代理声称
