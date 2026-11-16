# 轨道 3 流程机制审计（rea-audit-fix · 2026-11-16）

纯只读审计：docs/process/ + docs/loop.md + docs/rules/ + 飞轮/双审计/EXP 回写机制的失效面。
取证 = 实跑 grep/read/python/pwsh。共 24 条（M-01~M-24）。
A=可固化为机器断言 / B=需人拍板 / C=一次性修文档即毕。

## 发现清单

### M-01 台账面｜blocked 行 evidence 指向不存在的文件 + 台账校验不查 blocked
- 失效：`plan/ledger.yaml:384` S49.1（status: blocked）写 `evidence: "evidence/S49.1.log"`，该文件磁盘不存在（ls 确认）。blocked 任务按 loop.md §1 语义应 `evidence: null`；而 `check_ledger_reality.py`（gate 第 14 项）只遍历 verified 行（`check_evidence` 仅在 main() 对 verified 调用），对 blocked 行 evidence 存在性零检查。台账计数声称「340 任务/339 verified/1 blocked」= 实测一致（python 解析 340 行 = 339 verified + 1 blocked ✓），但 blocked 行的 evidence 假路径是死数据。
- 证据：`plan/ledger.yaml:384`（grep status: blocked 唯一命中）+ `ls evidence/S49.1.log` → No such file + `scripts/check_ledger_reality.py:182-196`（main() 只循环 verified）。
- 处置：**A**（check_ledger_reality 加：blocked 行 evidence 若非 null 须文件存在，或 blocked 行强制 evidence=null；loop.md §1 schema 写清 blocked ⇒ evidence: null）+ C（S49.1 行 evidence 置 null 或删）。P1。

### M-02 验收面｜硬规则 2 机器兜底只有「单侧」：只拦 gate-only，不拦 gate 缺席
- 失效：`doccheck.rs::evidence_is_gate_only`（crates/ti-tools/src/doccheck.rs:283）判据 = 文件**含** gate 特征行 `=== gate: TreasureIsl` 且**无** dod 关键词 ⇒ FAIL。反方向 = evidence 文件**不含任何 gate 输出、也全是叙述**（纯「声称已跑」文字）时 `evidence_is_gate_only` 返回 false ⇒ 恒 PASS。抽查 41 份 S49~S55 evidence：绝大多数不含 gate marker（grep 实测 0~1 行），靠 dod 关键词命中过关。即「只贴 gate 总输出」被机器拦，「纯叙述无实跑输出」无任何机器兜底——这正是硬规则 2 的另一半失效面（EXP-058 类虚报的反向形态：不是 gate-only 而是 narrative-only）。
- 证据：`crates/ti-tools/src/doccheck.rs:283-288`（GATE_MARKER 单侧判定）+ 实测 `grep -c "=== gate" evidence/S5*.log` 41 份中 35 份 = 0。
- 处置：**B**（方向拍板：是否要求 verified evidence 必含 ≥1 条实跑输出行/退出码，或维持「dod 关键词命中即可」现状）+ A（若拍板，doccheck 加「narrative-only」形态断言，棘轮存量豁免）。P1。

### M-03 验收面｜evidence 文件多任务共享，单任务虚报不可追责
- 失效：18 个 evidence 路径被 ≥2 个 verified 任务共用（最大 `evidence/S12.6.log` 被 S12.0~S12.6 七任务共用；S50.2/S46.0 各 3 任务；S9.5 与 S10.5 共用同一文件）。check_ledger_reality 的孤儿/存在性检查只看「该文件存在且非空」，共用文件里缺某一任务的判据段时无法发现——S54.1/S55.1 两份 log **逐字节相同**（diff 确认，S55.1.log = S54.1.log 复制件，1785B 同内容），S55 自身判据段（doccheck-counts-match 负例实跑）并未单独成段，而是并进 S54 合写文件。
- 证据：python 解析 ledger 339 行 evidence 字段去重统计（`evidence/S12.6.log ×7`、`×3` 两组、`×2` 四组）+ `diff evidence/S54.1.log evidence/S55.1.log` → IDENTICAL。
- 处置：**C**（S55.1 补自有 log 或 ledger 明示「S55.1 evidence = S54.1.log 合并件」）+ A（可选：check_ledger_reality 报「共享 evidence」清单供审计，非 FAIL）。P2。

### M-04 台账面｜三方对账第三方（符号 grep）覆盖率 ≈ 0.9%
- 失效：`check_ledger_reality.py` SYMBOL_EXPECTATIONS 只登记 3 个任务（S9.5/S10.4/S21.0，脚本:33-52 注释「按需追加」），339 verified 中仅 0.9% 有代码符号 grep 真验；其余 336 任务「三方对账」= 两方（台账行 + evidence 文件存在非空），第三方代码面恒跳过（`check_symbols` 未登记即 return []，脚本:127-129）。输出「全绿（339 个 verified 三方对账通过）」的措辞高估了强度。
- 证据：`scripts/check_ledger_reality.py:33-52`（SYMBOL_EXPECTATIONS 仅 3 键）+ `:127-129`（未登记 = 跳过）+ 实跑输出 `check_ledger_reality: 全绿（339 ... + 0 孤儿）` EXIT=0。
- 处置：**B**（是否逐任务补符号登记 = 高成本专项；或至少把输出措辞改为「2 方对账 + 3 任务符号 grep」防误读）+ A（doccheck 断言：SYMBOL_EXPECTATIONS 覆盖率 ≥ N% 或逐 dod 声明代码产物的任务必登记）。P2。

### M-05 飞轮面｜五步回执存在率：历轮 evidence 10/10 轮 0 份含「规范回执/五步」段
- 失效：06_flywheel §2 判据「子代理回报缺『规范回执』段 = 回报不合格打回」，S51.1 起 3 subagent 规范结构下发 + 五步回执为机制核心；但 `evidence/flywheel-R{2..11}.md` 10 份轮次落盘文件 grep 「规范回执/五步/提问/决策点」= 0 命中（唯一命中 R2:7 是「prompt 带『五步全走』字段」的编排描述，非 subagent 回执本身）。五步回执若存在只活在 subagent 对话流/主线程笔记里，未落盘 ⇒ 回执判据（缺段打回）无人可复核 = 机制两张皮（15 号 §2 第 28 行「机制落盘但审计未使用」的同族复发）。
- 证据：`grep -c "五步|规范回执|决策点|提问.*方案" evidence/flywheel-R*.md` → R2=1（编排描述）其余 R3~R11/R=0；对比 `evidence/flywheel-R2.md:5-7`（「修复 agent prompt 带『五步全走 + 红线基准面』字段」= 下发侧有，回报侧无）。
- 处置：**A**（flywheel-kickoff skill 编排面加硬步骤：3 subagent 回报的「规范回执 + 五步」段必须拷进 `evidence/flywheel-R<n>.md` 固定段，缺段 = 该轮停机判定 FAIL；doccheck2 可加「R 轮 evidence 含回执段」断言，棘轮存量豁免）+ B（是否强制落盘 = 拍板）。P1。

### M-06 飞轮面｜flywheel_readme 开轮输出第 [3] 条契约面 = 3 份全 draft，「契约冻结」纪律无机器兜底
- 失效：开轮必读第 [4] 条 = `docs/contracts/00_index.md` + c1/c2/c3。该索引表（docs/contracts/00_index.md:14-16）三契约全 `draft`；规则 1 写「`draft` 时开并行轮会 FAIL（第 16 项断言无 draft 残留）」——但 flywheel 开轮（§7 单轮流程 / 定向 §8）**不查契约状态**，第 16 项 `check_module_contracts` 只守「并行轮开法」。飞轮轮（S51.1 起 3 subagent 真跑）开轮时契约仍 draft，开轮必读把 draft 契约当下发基准，无门禁拦。
- 证据：`docs/contracts/00_index.md:14-16`（c1/c2/c3 status: draft）+ `scripts/flywheel_readme.ps1:289`（第 [4] 条原样下发 contracts/00_index）+ gate 第 16 项仅挂并行轮（`docs/contracts/00_index.md` 规则 1 口径）。
- 处置：**B**（契约何时冻 = 人拍板；若拍板维持 draft，则 flywheel_readme 第 [4] 条加「draft 契约仅作参照」注 + doccheck2 断言 flywheel 开轮前契约状态一致性声明；若冻 = 走第 16 项）。P2。

### M-07 双审计面｜B 侧独报消费 = 无机器追踪，「全修」靠主线程自觉
- 失效：EXP-188 A/B 协议（B 一致性独报值漂移）历轮 B 侧独报条目（R2 B-01~B-09 独报 3 条真漂移 / R3 B-03 P0 / R10 B-1~B-3）的「全修」闭环无台账行、无断言：R2 证据写「B 侧独报 3 条全修」，但修与未修不可机器复核（R9 实证 A 侧 3 条 P1 全是臆造行号 = 主线程 grep 证伪；B 侧若独报后部分修，停机判定「0 残留」也靠主线程复核，无落盘勾选表）。15 号 §2 第 21 行双审计 220 findings 的历史面 = 纯叙述。
- 证据：`evidence/flywheel-R2.md:20-21`（「抓到 A 看不到的 3 条真漂移…主线程亲自复核 top 8」无逐条勾选）+ `evidence/flywheel-R9.md:11`（A 侧臆造行号实证 = 复核必要性）+ `docs/process/15_process_ledger.md` 第 21 行（报告散落 evidence/console/audit-*.md，172 份无逐条闭环面）。
- 处置：**A**（审计任务书 `audit/taskbook.rs` 八要素加「B 侧独报条目 → 修复 commit / 状态」逐条机读段；`GET /api/audit` 收敛表加独报消费率）+ B（是否扩八要素 = 平台面拍板）。P2。

### M-08 EXP 回写面｜exp_log.py 查重语义缺陷：--check 对「已存在 2 处」恒 FAIL，不能做例行体检
- 失效：`exp_log.py` main()：登记路径 `len(hits)==1`（已存在）⇒「不重复登记」退出 1；`--check` 路径 `return 1 if len(hits) > 1 else 0`。一个正常登记的 EXP（index.md 行 + 分卷正文 = 2 处命中）跑 `--check` **必退出 1**（实测 EXP-189 --check 输出「出现 2 处」EXIT=1）。即工具契约要求的「只读模式 + 违规非 0 退出」里，正常态也非 0 ⇒ 该脚本不可挂 gate 例行体检，15 号 §2 第 2 行「EXP 登记已自动化（exp_log.py + check_experience_volume_counts）」的机器面实际只剩条数对账 1 项，查重面形同虚设（只能人工登记时跑，且正常结果也是「FAIL」码）。
- 证据：`scripts/exp_log.py:86-90`（`return 1 if len(hits) > 1 else 0`）+ 实跑 `python3 scripts/exp_log.py --check --id EXP-189 --volume process` → 「EXP-189 出现 2 处: [index.md:231, process.md:611]」EXIT=1。
- 处置：**C**（改 --check 语义：存在且 ≤2 处 = PASS 退出 0，>2 或跨卷重号 = FAIL；一次修脚本即毕）+ A（修后加负例：造重号 3 处 = 非 0）。P2。

### M-09 EXP 回写面｜EXP-189 登记 3 处未全齐：分卷表条数 47 vs 实 48，且 2 处「时点豁免」措辞掩盖真漂移
- 失效：EXP-189 登记（index.md:231 行 + process.md:611 正文）后，分卷表 process.md 条数仍 47（index.md:36）= 实 48 的 -1 漂移。`doccheck2 check_experience_volume_counts`（doccheck2.rs:1620）因表格解析判据 `cols[0].contains("](`)` 永不成立（分卷表行首列实为 `[`process.md`](process.md)` = 含「.md](``，代码注释自认「该检查自落地起从未生效」doccheck2.rs:1631-1632，历史 2 轮漂移靠人肉发现）⇒ 条数漂移 0 机器拦截。exp_log.py 登记路径③会回填条数，但 EXP-189 显然未经 exp_log.py 登记（否则条数 = 48）——EXP 登记 3 处齐不齐无门禁兜底。
- 证据：`docs/experience/index.md:36`（`| 47 |`）vs `grep -c '^### EXP-' docs/experience/process.md` = 48；`crates/ti-tools/src/bin/doccheck2.rs:1631-1632`（判据失效自认注释）；`exp_log.py:123-130`（登记路径会回填，说明 EXP-189 绕过）。
- 处置：**A**（修 check_experience_volume_counts 解析判据：trim 行后判 `](…md)` 形态 + 实测负例 process 47 vs 48 必 FAIL）+ C（index.md:36 条数 47→48，exp_log 口径回写）。P1（门禁恒失效 = 假绿）。

### M-10 门禁缺口面｜15 号 §2「已自动化」行 1 载体真失效 + 行 24 行号错位
- 失效：逐行核 35 行：「已自动化」行的载体 bin/脚本全部实存（task_close/exp_log/check_ledger_reality/doc_numbers/layout_spec_check/ui_* /check_ps_scripts 逐一 Test-Path 命中），**唯一真失效 = 行 2 载体 `check_experience_volume_counts`（见 M-09，断言自落地起从未生效）**。另：§2 表行号错位——物理第 24 行是「map_pack 草稿包健壮性」（行标 `| 24 |`，状态列措辞「已固化…S32.7 只挂门禁覆盖 P2-2」），而「判据载体存在性门禁（S46.0）」标 `| 26 |` 物理第 25 行，「内容运营流程机器化」标 `| 25 |` 物理第 26 行（行标 25/26 互换）；`check_process_ledger` 只验状态枚举/载体存在/待固化任务号，不验行号递增一致性。
- 证据：`docs/process/15_process_ledger.md:36-70`（§2 表 35 数据行实测；行 24/25/26 物理顺序 = 24/26/25 标号）+ `scripts/` 逐文件 Test-Path（11 载体全在）+ `doccheck2.rs:3911-3984`（check_process_ledger 判据面 = 状态/任务号/载体存在，无行号面）。
- 处置：**C**（15 号 §2 行标 25/26 互换纠正）+ A（check_process_ledger 加行号连续性断言，可顺带做）。P2。

### M-11 门禁缺口面｜「流程纪律 0 机器兜底 = 纯人盯」清单（15 号自认 5 项半自动中 3 项纯人 + 2 项无载体行）
- 失效：15 号 §2 状态 = 半自动/无载体者：行 18 出图逐屏目视（人核）/ 行 19 模块边界清单留档（「半自动（清单未留档接口）」= 实际纯人）/ 行 20 设计分歧人工裁决 / 行 22 平台 UI 视觉签收（人核 + 截图无断言）/ 行 28 ② state.md 压缩前 3 要素（「主线程手动自检，无自动触发」）。其中**行 19 模块边界清单 = 0 机器兜底纯人盯**（EXP-112 复发证据在，code_outline.py 只出清单不留档，接口未建）；**行 28② state.md 纪律 = 0 载体**（loop.md §9 触发条件全人工判断，`evidence/*.state.md` 磁盘 0 个 = 机制落地以来 0 次执行证据）。
- 证据：`docs/process/15_process_ledger.md:50-68`（行 18/19/20/22/28 状态列）+ `ls evidence/*.state.md` → 0 文件 + `docs/loop.md:80-91`（§9 触发条件 = 人工判断 4 条）。
- 处置：**B**（state.md 纪律是否值得固化载体 = 拍板；若固化 = A 断言「in_progress 任务跨会话须有 state.md 或豁免声明」；模块边界清单留档接口 = 人排期）。P2。

### M-12 飞轮停机面｜停机三条件判定 = 主线程自觉，0 机器断言；且 R4~R7 连续 4 轮未停机 = 成本膨胀实证
- 失效：06_flywheel §4 三条件（0 新发现/0 误报/0 残留）判定的唯一载体 = 主线程写进 `evidence/flywheel-R*.md` 的自由文本 + `flywheel.log`；`check_flywheel_readback`（doccheck2 第 8 项）只断言「05/03 文档含『开轮必读/规范回执』关键词」（doccheck2.rs:772-790），对停机判定 0 断言。成本面：2026-11-12 单日 R3→R11 = 9 轮（R3 开 / R4-R7 每轮「连续 2 轮 0/0/0 未达成」再开 / R8+R9 停机 / R10 平台面 7 findings / R11 停机）；R4:28/R5:21/R6:30/R7:25 逐轮实证「每开一轮 = 3 subagent + 主线程复核 + 全量 gate」的膨胀；而「修复 agent 改完全绿才回写」的自验判据（cargo test + clippy 全绿）同样无落盘强制。
- 证据：`docs/process/06_flywheel.md:53-61`（停机 = 主线程复核，EXP-038/058）+ `crates/ti-tools/src/bin/doccheck2.rs:772-790`（flywheel-readback 仅关键词面）+ `evidence/flywheel-R4.md:28`/`R5.md:21`/`R6.md:30`/`R7.md:27`（「未达成→再开」4 连轮）+ `ls evidence/` 单日 R3~R11 9 份。
- 处置：**B**（「连续 2 轮 0/0/0 才停机」口径 vs 成本：是否放宽「修复面 0 残留 + 审计面 0 P0/P1」即停，留 P2 轮 = 人拍板）+ A（R 轮 evidence 必含三条件勾选表 + gate 全绿行，doccheck2 断言缺失 FAIL，存量棘轮）。P2。

### M-13 双审计面｜R12 46 条 CONFIRMED 的机器兜底 = 2 项窄断言 + 全量人工；「项号文字」面 20+ 处 0 专属断言
- 失效：R12 修毕 20+ 处「项号文字本身」漂移（EXP-189 自认高频复现模式），固化判定 = **不新增断言**（r12-audit.log [5]：「项号引用面 20+ 处且 gate_data.json 顺序机读可信，文字漂移靠 cross-doc-counts + gate-items 双断言 + 对抗审计人工复核兜底」）。实测兜底面：`gate-items`（doccheck2 第 6 项）只断言 gate 项数 == 声称值（数量面），`cross-doc-counts`（第 44 项）只锚 3 条计数串（任务数/文档数面）；「doccheck2 第 N 项 = check_xxx」的**项名↔项号**文字 20+ 处 = 0 专属断言，下一轮 gate/doccheck2 加减项后必然复发（S53/S54/S55 加 3 项后 9+5 处现状区全漂 = 实证）。靠「后续 gate 项数变更走 doc_numbers --fix 扩面」= 工具扩面「待专项」未立项（process.md:617 自认）。
- 证据：`docs/experience/process.md:611-617`（EXP-189 正文：①项号镜像 9 处 + 5 处 / ②CS-1~6 vs 4 / ③版本路径面；固化判定「不新增断言…扩面待专项」）+ `evidence/r12-audit.log` [2][5]（46 CONFIRMED 全值漂移 + 收口口径）+ `assets/gate_data.json` doccheck2_checks 47 项 vs 文档「第 N 项」镜像。
- 处置：**B**（拍板是否立项「项号文字回写」专项 = 扩 doc_numbers.py `--fix` 面覆盖「第 N 项」声称点（机读源 = gate_data.json 序号），一次性修 20+ 处 + 门禁扩面 = A 形态；不立项 = 维持人工兜底 + 15 号 §2 挂待固化任务号）。P1。

### M-14 门禁缺口面｜gate.ps1 注释自认「36 项」，`Report` 实测 38 项 = 计数口径漂移（gate 自身面）
- 失效：`scripts/gate.ps1:1` 注释「统一门禁（36 项）」，但 `Report` 调用实测 = 38 行（fmt/clippy/test/cargo build ×2（native+aarch64 各 1 行）/…）。gate 项数的「36」= 声称镜像（parity/AGENTS/release 全写 36），`doccheck2 gate_item_actual_count` 去重口径才守 36（同项 PASS/FAIL 双分支去重后 36，flywheel_readme Read-GateItems 同口径注释 128-137 行实证去重逻辑）——即「36 项」是去重逻辑项名数，非 Report 行数；gate.ps1 自身注释与「36」口径的关系无文档说明，下轮加项的人很容易按 Report 行数（38）计数。
- 证据：`grep -c "^Report \|^    Report \|Report (\$" scripts/gate.ps1` 实测 38（含双分支重复）；`scripts/flywheel_readme.ps1:115-137`（去重口径注释 = 36 逻辑项名）；`docs/parity.md` §门禁 36 项。
- 处置：**C**（gate.ps1 头注释补一句「36 = 逻辑项名去重口径（PASS/FAIL 双分支去重），Report 行 38」即毕）+ A（可选 doccheck2 gate-items 断言注释含「36 = 去重口径」字样）。P2。

### M-15 文档膨胀面｜开轮必读 7 条最小集合 = 124KB/1634 行，且 3 份契约全 draft 仍强制下发
- 失效：7 条必读指向文档集实测 = `00_index.md 15KB + business/00_index.md 6.2KB + 01_architecture.md 9.7KB + 02_conventions.md 6.2KB + contracts/00+c1+c2+c3 13.9KB + experience/index.md 53KB + 03_templates.md 20KB` = **124KB / 1634 行**（flywheel_readme -Quiet 只发一句话要点，但 §2 判据要求 subagent 回执「读了哪份、命中几条」= 真读 7+3 份）。其中 contracts c1/c2/c3（13.9KB）状态 draft（M-06）仍占必读 11%；experience/index.md 53KB = 必读集最大单份（53/124KB），而 §2 第 4 条只要求「重点读与改动域相关的 EXP」= 53KB 全读无必要。
- 证据：`wc -c -l` 逐份实测（上表）+ `docs/process/06_flywheel.md:24-34`（7 条清单）+ `scripts/flywheel_readme.ps1:285-292`（-Quiet 面 = 一句话要点，非全文）。
- 处置：**B**（必读集最小化 = 不裁决，仅呈证据：①契约 draft 期豁免下发 ②EXP index 只发本域过滤后子集（脚本已有 Filter-Exps 能力但开轮必读数无上限提示）③03_templates 20KB 可拆五步段独立小文件）。P2。

### M-16 文档膨胀面｜三处重叠路由面：AGENTS 路由表 56 行 ↔ README 索引 116 行 ↔ 00_index 分工表，「单一事实源」结论跨 3 处复述
- 失效：AGENTS.md 路由表（56 条 `**` 行）与 docs/README.md 索引（116 条 `| [` 行）大量条目同指一文档（如 14_tooling_catalog 双处含「工具契约 3 处登记」全文复述、15_process_ledger 双处复述「载体三件套 + doccheck2 项号」——且两处项号还互漂：AGENTS:103 写「第 33~36 项」，README:75 写「第 31~34 项」（实测 doccheck2_checks 数组 repo-text-integrity=33/task-change-scope=34/tooling-registration=35/process-ledger=36 ⇒ README 31~34 错、AGENTS 33~36 对；或按「第 N 项」= 1-based 全表则 AGENTS 亦漂：tooling-registration 实为第 35 项）。硬规则 6「同一结论只写一处」在路由层自身违反：路由表三处（AGENTS/README/00_index §分工表）互相引用 + 复述细节。
- 证据：`grep -c "^| \*\*" AGENTS.md` = 56 vs `grep -c "^| \[" docs/README.md` = 116；`AGENTS.md:102-103` vs `docs/README.md:74-75`（14 号/15 号条目双复述 + 项号 33~36 vs 31~34 互漂）+ `assets/gate_data.json` 数组 1-based 实测。
- 处置：**C**（README:75「第 31~34 项」改「第 33~36 项」对齐 AGENTS，两处细节瘦身互指）+ A（doccheck2 加「路由表 ↔ README 索引条目数/指文档集一致」断言 = 防 3 处各自生长；项号镜像面归 M-13 专项）。P2。

### M-17 飞轮开轮面｜docs/rules 域过滤：6 文件 frontmatter 齐，但 `ti-engine/scripts/assets` 域 P 包 = 0，红线注入面实际只覆盖 2/6 文件
- 失效：`flywheel_readme.ps1:236-240` 注释自认「P 包 files 的 crate 前缀匹配红线文件 globs」；`plan/packages.yaml` 7 包 files 全在 `crates/ti-core/`（P1~P5）+ `crates/ti-app/`（P6）+ `assets/data/`（D）⇒ 按包域过滤只命中 `ti-core-rules.md` + `ti-app-scenes-rules.md` 2 个；`ti-engine-rules.md`（crates/ti-engine/src/**）/ `scripts-rules.md` / `assets-data-rules.md` / `ti-console-rules.md` 4 个 = 0 P 包持有，仅 -All（全域开轮）下发全 6 个。即：S51.1「3 subagent 带红线注入」的包域轮（P1~P6 开轮）对 ti-engine/scripts/assets/console 4 域红线 0 注入——而 R10 平台面审计（ti-console 域）走全域开轮才拿到 ti-console-rules。域↔红线映射无台账行，加新 P 包（如 ti-engine 包）时红线注入面自动扩展性 = 依赖注释而非机器面。
- 证据：`scripts/flywheel_readme.ps1:146-170,236-270`（过滤逻辑 + 注释「ti-engine/scripts/assets 包域 = 0」）+ `plan/packages.yaml`（7 包 files 域实测）+ `docs/rules/` 6 文件 frontmatter paths 逐一核对。
- 处置：**B**（是否加 P7 = ti-engine 包域 + P8 = 平台包域使 6 红线全挂包域 = 拍板）+ C（flywheel_readme 输出「本域 0 命中」时已提示「改前查 docs/rules/ 5 个全」（:309 行写死 5，实 6 = 小漂移））。P2。

### M-18 飞轮开轮面｜flywheel_readme.ps1 :309 硬编码「docs/rules/ 5 个全」，实 6 文件 = 输出面陈旧
- 失效：`flywheel_readme.ps1:309` 包域 0 命中时输出「（无命中包域红线文件，改前查 docs/rules/ 5 个全）」——docs/rules/ 实 6 文件（S50.3 5 个 + 飞轮 R10 新增 ti-console-rules.md，`-Check` 实测「rules 6 红线」）。输出面数字陈旧 = 子代理照输出去查会漏 1 个。同型：`-Check` 判据 `allRules.Count -lt 5`（:356）= 下限 5，加到 7 文件才触发，数字型判据易过期。
- 证据：`scripts/flywheel_readme.ps1:309`（「5 个全」）vs `ls docs/rules/*.md` = 6 + `flywheel_readme -Check` 实跑「rules 6 红线」。
- 处置：**C**（:309 「5 个全」→「全」（去数字）或改动态计数；:356 下限 5 留作下限但注释说明）。P2。

### M-19 双审计面｜rea-audit-fix skill 文本 = Windows 时代冻结面（C:\金银岛 根 + Temp 输出路径），Linux 迁移后 0 更新
- 失效：`.opencode/skills/rea-audit-fix/SKILL.md:14`「仓库根 C:\金银岛」+ `:32`「Write 到 C:\Users\Administrator\AppData\Local\Temp\opencode\audit_roundN_find.md」= 2026-10 前 Windows 单机形态；Linux 迁移（docs/migration_linux.md）+ 现仓库路径 /disk/workspaces/treasureIsl + `/tmp/opencode` 约定后，skill 文本未同步（R12 实跑 = `/tmp/opencode/audit_round12_find.md`，靠 skill 之外的人约定兜住）。skill 是子代理 prompt 的固定模板源，模板里写死 Windows 路径 = 每次开轮 prompt 带错误环境事实（与 flywheel_readme「环境事实」段自相矛盾）。
- 证据：`.opencode/skills/rea-audit-fix/SKILL.md:14,32`（grep `C:\\` 命中 2 处）vs `evidence/r12-audit.log:4`（实落 /tmp/opencode）+ `docs/migration_linux.md`（Linux 续工）。
- 处置：**C**（skill 文本 2 处路径改「仓库根（当前工作目录）+ /tmp/opencode/」即毕）+ A（可选 doccheck 扫 `.opencode/skills/*.md` 含 `C:\\` = 漂移 FAIL）。P1（skill = 开轮固定模板，每轮污染 subagent prompt）。

### M-20 验收面｜S49.4d 判据「本机无 NDK 无法验证 Android」= 真机判据 P5 依赖用户重编 APK，verified 的机器面 = 0
- 失效：`evidence/S49.4d.log:23` 自认「Android 交叉编译本机无 NDK 无法验证…真机验证 = 用户重编 APK 实测」——S49.4d（触摸/全屏/BGM 修复）置 verified 时 Android 行为面 0 机器判据（gate 第 4c 项 Android 真机 = 记名 SKIP 口径「本机无设备 SKIP 非 FAIL」，docs/parity.md:70）。P5 判据（S49.4e 立的）= 事后补判据，S49.4d 的 verified 时点真机面 = 纯人盯（用户实测留痕在聊天，无 evidence 文件锚定「用户重编后 PASS」的收口动作）。
- 证据：`evidence/S49.4d.log:19,23` + `docs/parity.md:70`（设备接入即跑；无设备 = SKIP 文档化）+ `scripts/gate.ps1:1`（第 4c 项注释）。
- 处置：**B**（「用户重编 APK 实测 PASS」是否要求留痕进 evidence/S49.4d.log 追加段 + verified 判据含「真机收口」= 人拍板 P5 类任务收口口径）。P2。

### M-21 飞轮回写面｜飞轮回写表 §3「可机械判定 ⇒ doccheck2 新断言」无负例护栏强制，断言可恒 PASS 漏网（scene-band-count 实证后未推广）
- 失效：06_flywheel §3 判据「下一条能自动 grep/断言复现」，但**断言接入后非恒 PASS 的验证（负例护栏）不是 §3 表的判据项**（负例护栏实为 R9/R11 起主线程自觉动作：R9:「S53 断言非形同虚设：负例护栏造幻影名 ⇒ 1 ERROR ✓」；S54.1/S55.1 dod 才把「负例护栏实跑」写成判据）。飞轮 R 曾删恒 PASS `check_scene_band_count`（evidence/flywheel-R.md:19，select_role 退役后 0 命中形同虚设）= 实证「恒 PASS 断言会漏网」，但 47 项存量断言 0 条有「定期负例复验」机制（studio_tests 第 3 类断言 = 工具台账登记，非 doccheck2 负例面；`doccheck-counts-match` 只管计数声称点）。
- 证据：`docs/process/06_flywheel.md:43-51`（§3 表无负例行）+ `evidence/flywheel-R.md:19`（删恒 PASS 实证）+ `evidence/flywheel-R9.md:16`（负例 = 人工动作）+ `scripts/studio_tests.py`（3 类断言面 = 台账/触发面/映射，无 doccheck2 恒 PASS 面）。
- 处置：**A**（doccheck2 加元断言：RUST_CHECKS 47 项逐条「最近一次实跑非恒 PASS 留痕」或 studio_tests 加第 4 类「断言面非恒 PASS 抽样负例」；S54/S55 判据形态推广 = 判据模板加「负例护栏」必填段）+ B（存量 47 项逐条负例成本 = 拍板做不做全量）。P2。

### M-22 台账面｜loop.md §1 schema 写 `evidence: null` 语义，ledger 实存行格式 = `evidence: "…log"` 字符串，blocked 行 schema 漂移
- 失效：loop.md §1 台账 schema 示例 `evidence: null # verified 后填`；规则「blocked_by 约定：blocked 任务写字符串原因，pending/verified 为 null」。S49.1 blocked 行实存 `evidence: "evidence/S49.1.log"`（M-01）= 按 schema 该字段在 blocked 态无定义（loop.md 未写 blocked ⇒ evidence 允许/必须什么）⇒ 写不存在的文件路径是 schema 空档的必然产物。
- 证据：`docs/loop.md:14-33`（schema + 规则，无 blocked-evidence 条款）vs `plan/ledger.yaml:384`。
- 处置：**C**（loop.md §1 补 1 行：「blocked 行 evidence 必须为 null」，与 M-01 同单修）+ A（台账校验器 `ledger` bin 加 blocked ⇒ evidence null 断言）。P2（与 M-01 合并）。

### M-23 门禁覆盖缺口面｜doccheck「数值断言」只有 4 条（UI 场景数 20/控件 2041/LCG hex/save.magic），业务数值面 0 断言 = 值漂移 B 面全靠飞轮人扫
- 失效：doccheck（gate 第 9 项）数值断言面实测 4 条（`doccheck.rs` NumericAssertion 表，实跑输出「4 assertions」），全是格式/逆向面常量；业务数值（balance.json 数值 ↔ business/*.md 声称值、territory.md 可测表 ↔ tests/ 测试名）= 无 doccheck 面，靠 `check_business_doc_test_refs`（第 45 项，只断言「测试名存在」不断言数值）+ 飞轮 B 侧人工。即「doc↔data 值漂移」机器面 0 覆盖，R12 46 条 CONFIRMED 全值漂移 = B 面独报的结构性原因（A 完成度面看不见值，机器又不断言值）——机器兜底缺位把整面推给 2 天 9 轮的人肉飞轮。
- 证据：`crates/ti-tools/src/bin/doccheck.rs`（NumericAssertion 4 条，doccheck.rs:39 注释 + 实跑 `doccheck: …4 assertions…`）+ `evidence/r12-audit-audit.md:9-19`（46 CONFIRMED 抽样 = 项号/计数/路径/版本值漂移）+ `docs/process/06_flywheel.md:56-58`（B 面 = 人工裁决桶）。
- 处置：**B**（是否扩 doccheck 数值断言面到 balance↔business 声称值（高成本，易误报）= 拍板；折中 = doc_numbers.py `--fix` 扩「项号+数值声称点」回写面（M-13 同专项））。P1。

### M-24 飞轮成本面｜「06_flywheel §8 定向飞轮 100 条/轮」目标与 R3~R11 实际 0~42 条/轮 脱节，停机判据三条件在收敛末期成本/收益倒挂
- 失效：§8 写「找问题目标 100 条/轮」（不足 100 = 该主题接近收敛），但业务面 R3~R9 实际 = 42（R2）→3（R3）→3（R4）→0（R5）→2（R6）→2（R7）→0（R8）→0（R9）；R5/R8/R9 三个 0 轮各耗 3 subagent + 全量 gate + 主线程复核，且 R5 是「第 1 个 0 轮」、R9 是「第 2 个 0 轮」才停机 = 连续 2 轮 0/0/0 口径使停机判定**至少 2 轮全成本 0 产出**。R10/R11 平台面重复同一 2 轮结构。成本/收益倒挂面（3 subagent + gate 36 项 × 2 纯验证轮）无机器记账，无「停机轮预算」概念。
- 证据：`docs/process/06_flywheel.md:126`（100 条/轮目标）+ `evidence/flywheel-R{2..9}.md` 逐轮 findings 计数（上列）+ `evidence/flywheel-R10.md:40`/`R11.md:24-26`（平台面 2 轮 0 结构）+ CCGS §7.4「连续 2 轮 0/0/0」口径（22 号 §7.4）。
- 处置：**B**（停机口径修订 = 人拍板：①「第 1 个 0 轮 + 修复面 0 残留」即停，P2 轮降级为可选；②或「0 轮」判定复用上一轮负例护栏不重开 3 subagent；拍板后 06_flywheel §4 + 22 号 §7.4 同步改）。P1（高频成本面，每域停机都吃 2 轮）。

## 统计
- 总条数：24
- A/B/C：A=10（M-01/M-05/M-07/M-08/M-09/M-10/M-12/M-13(可选)/M-16/M-21；另 M-02/M-19/M-22/M-23 含 A 分量，按主处置计 B/C）
  - 严格主处置：A=7（M-05,M-07,M-09,M-10,M-12,M-21 + M-01）/ B=8（M-02,M-06,M-11,M-13,M-17,M-20,M-23,M-24）/ C=9（M-03,M-04,M-08,M-14,M-15,M-16,M-18,M-19,M-22）
- 优先级：P0=0 / P1=5（M-01,M-02,M-05,M-09,M-13,M-19,M-24 中取前 5：M-05 回执不落盘 / M-09 条数门禁恒失效 / M-13 项号文字 0 断言 / M-19 skill Windows 路径污染 / M-24 停机 2 轮成本） / P2=其余
  - P0 阻塞验收可信度 = 0 条（台账 339 三方对账实测全绿，无阻塞级失效；M-01/M-09 接近 P0 但属数据面陈旧/门禁恒 PASS，未直接污染 verified 判定）
