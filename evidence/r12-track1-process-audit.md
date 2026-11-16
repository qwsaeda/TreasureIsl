# R12 轨道 1 AUDIT — 开发流程面 doc↔code/data 值漂移复核（P001~P065）

只读审计（AUDIT 角色，未改任何仓库文件）。基准面（本审计独立机器实测）：
- gate.ps1 机器 36 项（Report 行首个引号串去重口径，与 doccheck2 `check_gate_item_count` / `flywheel_readme.ps1 -Check` 同实现）：
  1 fmt / 2 clippy / 3 test / 4 build(native) / 5 wasm build+clippy / 6 android / 7 Android 真机(4c) / 8 clippy feature-gated /
  9 占位 / 10 unwrap / 11 zero-unknown / 12 ledger / 13 doccheck / 14 validate_data / 15 gen_layout / 16 check_hardcode /
  17 check_data_driven / 18 check_code_standards / 19 check_oracle_boundary / 20-21 no-macroquad 两变体 / 22 doccheck2 /
  23 check_ledger_reality / 24 check_forward_closed / 25 check_module_contracts / 26 flywheel-readme / 27 render-invariants /
  28 check_data_source / 29 audit_code / 30 check_scene_narrative / 31 bevy-cpu-parity / 32 wasm-browser-e2e /
  33 browser-playtest / 34 playtest-loop / 35 dod-commands / 36 studio-tests。
- doccheck2 47 项（`assets/gate_data.json` `doccheck2_checks` 1-based）：15 engine-no-color-literal / 20 runtime-no-legacy-coords /
  22 no-ti-core-fullpath / 23 std-traits-matrix / 24 render-framework-boundary / 25 ui-redesign-tokens / 26 primitives-data /
  27 terrain-data / 28 tooling-catalog / 29 tooling-contract / 42 rules-files / 43 theme-consumption / 44 cross-doc-counts /
  45 business-doc-test-refs / 46 blocked-label / 47 doccheck-counts-match。
- 台账 = 341 行（339 verified + 1 pending S56.0 + 1 blocked S49.1），`grep -c "id: S" plan/ledger.yaml` 实测。
- evidence git-tracked（排除 console/）= 543；集成测试文件（`crates/*/tests/*.rs`）= 105；`Cargo.toml` version = 0.2.0；
  `web/treasure_bevy_wasm-0.2.0.wasm` = 45,794,950 B；`playbook.ps1` switch 分支 22 个 / `-List` 实打 20 行。

判定口径：CONFIRMED = 值与机器实测真不符需修；REJECTED = 值正确或 FIND 误判；PARTIAL = 值/时点有偏差但可接受（时点值/建议类/低危）；NO-LINE = 行号或定位失效（内容已变或不存在）。

---

P001 [CONFIRMED] 07_render_pipeline.md:20/:25 写「gate 第 19 项 render_invariants + 第 20 项 数据源」，机器实测 render-invariants=第 27、check_data_source=第 28（scripts/gate.ps1:339/:347 Report 行）；:120 表行「第 19 项 渲染结构不变式」:121「第 20 项 数据源门禁」同错。
P002 [PARTIAL] 07_render_pipeline.md:25/:121「第 20 项 数据源门禁」与 P001 同源（实测 28），FIND 建议 20→27 自身与 P001 基准面（check_data_source=27）错位 1 位（FIND 去重口径把 no-macroquad 两变体合并），语义项号错 = 同 P001 一并修。
P003 [CONFIRMED] 04_tooling.md:59「doccheck（gate 第 9 项）」+ :75 注释块「第 9 项 doccheck」，实测 doccheck = gate 第 13 位（scripts/gate.ps1:223）。
P004 [CONFIRMED] 04_tooling.md:61/:82「check_oracle_boundary gate 第 11 项」两处，实测 = 第 19 位（scripts/gate.ps1:267）。
P005 [CONFIRMED] 04_tooling.md:63/:83「check_ledger_reality gate 第 14 项」两处，实测 = 第 23 位（scripts/gate.ps1:295）。
P006 [CONFIRMED] 04_tooling.md:52/:76/:92「gate 第 10 项 = 数据表门禁」三处，实测 validate_data=14 / gen_layout=15 / check_hardcode=16 三独立项（scripts/gate.ps1:232/:236/:243）。
P007 [CONFIRMED] 05_module_parallel.md:122「第 15 项查业务闭环、第 16 项查包白名单/契约」，实测 check_forward_closed=24、check_module_contracts=25（scripts/gate.ps1:301/:309）。
P008 [CONFIRMED] 06_flywheel.md:112「gate 加 flywheel-readme -Check 项（第 17 项）」，实测 flywheel-readme = 第 26 位（scripts/gate.ps1:326）；00_index.md:77 + 26_dev_stages_overview.md:105 同错（见 P058/P061）。
P009 [CONFIRMED] 08_playtest_loop.md:60「gate 第 23 项同款前置（serve）」指 browser-playtest 前置，实测 browser-playtest = 第 33 位（第 23 位 = check_ledger_reality，与 serve 前置无关）。
P010 [CONFIRMED] 12_bevy_usage.md:349 / 11_bevy_migration.md:83 / business/08_layout_design.md:124「check_data_driven gate 第 24 项」三处，实测 = 第 17 位（scripts/gate.ps1:251/253 两分支）。
P011 [CONFIRMED] business/08_layout_design.md:115/:159/:212/:239「check_scene_narrative gate 第 22 项」4 处，实测 = 第 30 位（scripts/gate.ps1:363；第 22 位 = doccheck2）。
P012 [CONFIRMED] docs/README.md:41「check_ledger_reality gate 第 14 项」，实测 = 第 23 位（第 14 位 = validate_data）。
P013 [CONFIRMED] docs/README.md:56「07_render_pipeline gate 第 19/20 项锚点」，实测 = 第 27/28 位。
P014 [CONFIRMED] docs/README.md:60「gate 第 28 项 check_scene_narrative」，实测 = 第 30 位（第 28 位 = check_data_source）。
P015 [CONFIRMED] docs/README.md:13/:36「doccheck 门禁第 9 项」两处，实测 doccheck = 第 13 位。
P016 [CONFIRMED] 00_index.md:154「门禁第 9 项自动校验」指 doccheck，实测 = 第 13 位。
P017 [CONFIRMED] 09_render_framework.md:11「doccheck2 第 22 项 render-framework-boundary」，实测第 22 位 = no-ti-core-fullpath，render-framework-boundary = 第 24 位（gate_data.json `doccheck2_checks` 1-based 实测）。
P018 [CONFIRMED] 10_ui_redesign.md:185「doccheck2 第 23 项 check_ui_redesign_tokens」，实测第 23 位 = std-traits-matrix，ui-redesign-tokens = 第 25 位。
P019 [CONFIRMED] 11_bevy_migration.md:90「doccheck2 第 25 项 check_terrain_data」，实测第 25 位 = ui-redesign-tokens，terrain-data = 第 27 位（:88「第 26 项 primitives-data」实测正确，勿误改 = P057 核销）。
P020 [CONFIRMED] 32_ccgs_borrow.md:82「门禁面 = doccheck2 第 43 项断言 rules-files」无时点标注，实测 rules-files = 第 42 位（第 43 位 = theme-consumption）；S50 立项时点值 43 已因 R12 删恒 PASS 项 -1 偏移。
P021 [PARTIAL] 31_test_ingress_standard.md:42 示例「web_playtest gate=23」与 test_manifest.json 实测 `"gate": 23`（web_playtest 条目，machine 第 33 位 = browser-playtest）数据文件自身一致，仅语义位（browser-playtest=33）与示例数字 23 错位；机器校验 `gate∈1..=36` 通过、无值错，属语义澄清面。
P022 [PARTIAL] 31_test_ingress_standard.md:36/:76 值域 `1..=36` 表述窄于工具（legacy/scripts/validate_data.py:602 实测接受 `re.fullmatch(r"[0-9]+[a-z]?", gate)` 字符串条件项号，android_device_test 在册 gate='4c'），文档值域表述不完整（非值错，android 条目用字符串形式已绕开 1..=36 数值校验）。
P023 [CONFIRMED] assets/data/test_manifest.json gate_full 条目 name="全量门禁 33 项" vs desc="跑全量门禁 35 项"（json 内自相矛盾）vs 机器实测 36 项；机读数据文件漂移，平台测试页直接展示。
P024 [CONFIRMED] 23_game_testing.md:10「evidence/ 535 文件（R11 复验 534→535）」，git ls-files 实测 543（排除 console/）；R12 后 8 新文件未回写。
P025 [NO-LINE] 23_game_testing.md:10「106 个集成测试文件」FIND 自述「与实测 106 一致」，但 `crates/*/tests/*.rs` 本审计实跑 = 105；文档值 106 vs 实测 105 = 潜在漂移被 FIND 误核销（FIND 基准面 106 未独立核过），修复方须复核 105/106 口径。
P026 [PARTIAL] AGENTS.md:129「常用命令一键入口（22 项命令固化）」vs `playbook.ps1` switch 分支实测 22 个（含默认 `check` 分支）、-List 实打 20 行（run-wasm/sprite-step/build-wasm/web-playtest/playtest-loop 等 2 个 + check 默认分支不在 -List）；「22 项」与 switch 分支数吻合，与 -List 行数 20 不吻合 = 口径未标注，FIND 建议 22→21 不成立（switch = 22 非 21）。
P027 [CONFIRMED] 14_tooling_catalog.md:118「列出全部 16 条」+ :254「常用命令一键入口（16 条）」两处，-List 实打 20 行 / switch 22 分支，16 为 S21.8 时点值（脚本内注释同写 16）未回写。
P028 [PARTIAL] 27_landscape_adaptation.md:117/:120/:124/:127/:132/:136「gate 33 项 0 failure」6 处无时点标注，现行机器 36；值 = S38 时点快照（gate 33 项时点成立），缺时点注 = 低危表述面，非值错。
P029 [PARTIAL] parity.md:108 计数链「S46.1 起 = 34 + 第 25 项 dod-commands；S49.4e 起 = 35 + 第 4c 项 Android 真机」与现行机器位（dod-commands=35 / Android 真机=7）不匹配；但链尾自带「数字对不上时以机器输出为准」免责声明 + 各阶段时点位（S46.1 时点 dod-commands 当时确为第 25/26 位）可成 = 历史链 + 免责口径，低危。
P030 [PARTIAL] parity.md:115「注释编号（1..20 + 19.5 + 4b/4c，含第 7 项 zero-unknown）」——注释面 zero-unknown 现机器 = 第 11 位（第 7 位 = Android 真机）；同段前句已注「注释编号与机器计数不一定相等」（自澄清口径），零知识项「第 7 项」字样与现机器位脱钩但未标注「≠ 机器位」。
P031 [CONFIRMED] parity.md:123「门禁第 25 项 dod-commands」，实测 dod-commands = 第 35 位（第 25 位 = check_module_contracts）；S46.1 时点值无标注。
P032 [PARTIAL] parity.md:218「25b 元测试面」= gate.ps1 注释编号口径（与 4c 同风格注释命名，非机器 1-based 位），段内上下文（P030 已注「注释编号 ≠ 机器计数」）自洽；机器 studio-tests = 第 36 位。注释面残留，低危。
P033 [CONFIRMED] 22_dev_platform.md:40「第 23 项 browser-playtest 无分档守卫」，实测 browser-playtest = 第 33 位（第 23 位 = check_ledger_reality，语义错配）。
P034 [PARTIAL] 22_dev_platform.md:156/:160/:163「gate.ps1 第 17 项内部硬编码 powershell / 宿主解释器探测」3 处，S32.11 修复语境指 flywheel-readme 项（当时第 17 位，现第 26 位）；同段自带「该行号随 gate.ps1 在途改动漂移；S32.11 开工时须重实测行号回填，勿按文档时点值定位」免责注 = 时点值 + 自注，非现状误读。
P035 [PARTIAL] 22_dev_platform.md:135「S50.2 studio-tests 后 35→36」增量链与机器 36 终值吻合，缺「第 36 位 = studio-tests」定位；FIND 建议补「studio-tests = 机器第 35 项」自身亦错位 1 位（studio-tests 实测 = 第 36 位，FIND 基准面把 no-macroquad 两变体合并为 1 项致全部后续位 -1），低危表述面。
P036 [REJECTED] 28_engine_best_practice.md:76/:85「engine-no-color-literal（第 15 项）」实测 gate_data.json `doccheck2_checks` 第 15 位 = engine-no-color-literal ✓，核销正确，无需改。
P037 [PARTIAL] 08_render_arch.md:145「doccheck2 第 20 项 runtime-no-legacy-coords」（✓ 实测第 20 位）+「gate 第 13 项 doccheck2」（✗ 实测 doccheck2 = 第 22 位；第 13 位 = doccheck）；FIND 建议 13→21 自身亦错位（FIND 去重口径合并 no-macroquad 两变体），正确值 = 22。
P038 [PARTIAL] 08_render_arch.md:205/:207「gate 第 13 项（doccheck2）」两处 + :207 混写「gate 第 3 项 + 第 13 项」（3 = cargo test ✓、13 应为 22）；与 P037 同源，正确值 22（FIND 21 为口径错位）。
P039 [PARTIAL] 08_render_arch.md:224-229 段内已注「历史记述——S18.7b 起已退役，以下 gate 项号 = 退役前时点口径」（:224 免责注在位）；:229「doccheck2 第 19 检查 baseline-mirror-identity」检查已不在现行 47 项（退役），段内注已覆盖时点语境，:229 单行未重复括注 = 低危补注建议。
P040 [PARTIAL] 08_render_arch.md:195「gate 第 18/19/19.5 项的三套基线」无时点标记（S15 前旧位），同文件 :224 段注「基线机制已删除/退役前时点口径」部分覆盖；:195 与 :224 语境脱节（间隔 29 行），低危补注面。
P041 [REJECTED] 15_process_ledger.md:65「第 43 项」theme-consumption + :66「第 44 项」cross-doc-counts 实测 43/44 ✓（gate_data.json 43 = theme-consumption / 44 = cross-doc-counts），核销正确，无需改。
P042 [REJECTED] 15_process_ledger.md:61 行 36 = S56.0 载体行存在 ✓ 且与 AGENTS.md 路由表 S56 行「15 号行 36」一致；行号 34/36 间无 35 行 = 行号非连续（台账任务粒度），FIND 自述「无碍」，无需改。
P043 [CONFIRMED] plan-s50-studio-interface.md:100-104/:121「doccheck2 加第 43 项断言 check_rules_files」4 处 = S50 立项时点值（当时 43），现行 = 42（R12 删恒 PASS 项后 -1）；计划文档无时点标注，被 S56/路由表当现行引用。
P044 [REJECTED] 26_dev_stages_overview.md:141「179 docs/38 facts/186 experiences/335 task refs」与 doccheck 2026-11-16 轨道 3 回写值实测一致 ✓；增量链「184→185 + 178→179 + 334→335 + 185→186」自洽，核销正确，无需改（列为 S56.0 观察点）。
P045 [PARTIAL] 26_dev_stages_overview.md:181 36 项清单面 = 手工重排（与机器位序不符，如 check_code_standards 排末位而机器 = 第 18 位；flywheel 排模块契约后而机器 = 第 26 位 ✓ 大致对但多处错位）；总数 36 ✓，清单序 ≠ 机器位，低危（建议注「以机器位为准」）。
P046 [REJECTED] AGENTS.md:8「341 任务 / 339 verified + 1 pending（S56.0）+ 1 blocked（S49.1）」= ledger 实测 341/339/1/1 ✓（`grep -c` 三数吻合），核销正确（S56.0 轨道 3 登记后一致），列为 S56.0 高频漂移点观察。
P047 [REJECTED] 24_apk_size_std + 25_versioning 版本面：Cargo.toml 0.2.0 = AndroidManifest versionName 0.2.0 = release.md 0.2.0；`web/treasure_bevy_wasm-0.2.0.wasm` 实测 45,794,950 B = release.md:41 ✓，全核销，无漂移。
P048 [REJECTED] `crates/ti-console/console_commands.json` 24 id 与 22_dev_platform.md §4.2 白名单表 24 行集合相等（本审计实跑对称差 = ∅），ui_theme 行在 ✓，核销正确。
P049 [REJECTED] 31_test_ingress_standard.md:48 程序白名单「cargo/python3/pwsh/powershell」与 validate_data 实测白名单集合一致 ✓；禁 shell 拼接面与 23/22 号 §10.3 表述一致 ✓，核销正确。
P050 [PARTIAL] 23_game_testing.md:45「gate 第 4c 项 + 验收判据（android_device_test）」与 parity.md:82 一致（4c = 注释命名面，机器位 = 第 7 位）✓ 互洽；与 P022 值域表述需同步 = 澄清面，非值错。
P051 [REJECTED] 14_tooling_catalog.md:18/:31-32/:44/:50「doccheck2 第 28 项 tooling-catalog / 第 29 项 tooling-contract」实测 28/29 ✓（gate_data.json 28 = tooling-catalog / 29 = tooling-contract），核销正确。
P052 [PARTIAL] 14_tooling_catalog.md:189「dod-commands gate 第 25 项」+ :190「studio_tests gate 第 25b 项」：dod-commands 机器位 = 35（FIND 说 34 = 口径错位 1 位），「第 25 项」为 S46.1 时点值残留无标注；「25b」= gate.ps1 注释编号面（与 4c 同风格），studio-tests 机器位 = 36。时点值 + 注释面混合，低危。
P053 [PARTIAL] 04_tooling.md:64「check_dod_commands 行 gate 第 25 项」= P052 同源，实测 dod-commands = 第 35 位（FIND 34 为口径错位），S46.1 时点值无标注。
P054 [PARTIAL] 04_tooling.md:65「studio_tests 行 gate 第 25b 项」= P052 同源，「25b」注释编号面 vs 机器第 36 位，低危。
P055 [PARTIAL] 22_dev_platform.md §10.3 平台 run_test 白名单段未声明「test_manifest command 首词白名单以 31 号 §3 为准」双向对账关系（studio_tests ② 只做 manifest↔console_commands 映射，不查首词∈白名单）= 表述缺口建议，非值错。
P056 [PARTIAL] 08_render_arch.md:229「doccheck2 第 19 检查 baseline-mirror-identity」已退役（现行 47 项无），段 :224 已注「历史记述/退役前时点口径」，:229 单行未重复括注 = P039 同段补注建议，低危。
P057 [REJECTED] 11_bevy_migration.md:88「check_primitives_data（doccheck2 第 26 项）」实测第 26 位 = primitives-data ✓，核销正确（同文件 :90 第 25→27 错见 P019，两处并存勿改错 = P019 建议已注明）。
P058 [CONFIRMED] 26_dev_stages_overview.md:105「gate 第 17 项锚点」指 flywheel-readme，实测 = 第 26 位（FIND 基准面写 25 = 口径错位 1 位，正确值 26），P008 同源第 3 处。
P059 [CONFIRMED] 07_render_pipeline.md:123「元素级验证…与第 19 项全帧 diff 互补」= render-invariants，实测第 27 位（FIND 基准面 26 = 口径错位 1 位），P001 同文件第 5 处。
P060 [CONFIRMED] 07_render_pipeline.md:201/:234「render_invariants 第 19 项」两处，实测第 27 位（FIND 基准面 26 口径错位），P001 同文件第 6/7 处。
P061 [CONFIRMED] 00_index.md:77「06_flywheel…gate 第 17 项」，实测 flywheel-readme = 第 26 位（FIND 基准面 25 口径错位），P008 同源第 2 处。
P062 [REJECTED] 23_game_testing.md:29/:65/:77/:203「门禁 36 项」4 处 = 机器实测 36 ✓，核销正确（R10 勘误后一致）。
P063 [PARTIAL] release.md:28「wasm sha256 前缀 + 45,712,494 B（发布时点）」vs :41 收口值 45,794,950 B（磁盘实测吻合）；:28 标注「APK 构建时回填」= 时点值非漂移，缺「收口值以 :41 为准」机制说明，低危补注。
P064 [PARTIAL] 27_landscape_adaptation.md:158/:212「gate 33 项 0 failure」2 处 = P028 同源（S38 时点值，现行 36），缺时点注 = 低危表述面。
P065 [PARTIAL] loop.md:81-93（state.md 纪律段）纪律存在但仓库 0 个 `*.state.md` 产物（本审计实跑 `ls evidence/*.state.md` = 空）；15 号行 28 自认「半自动（主线程手动自检）」= 机制未触发记录，低危（S56.0 面可加「state.md 存在性 = 0 触发」口径注）。

---

### 统计（本审计独立复核）

CONFIRMED = 29：P001 / P003 / P004 / P005 / P006 / P007 / P008 / P009 / P010 / P011 / P012 / P013 / P014 / P015 / P016 / P017 / P018 / P019 / P020 / P023 / P024 / P027 / P031 / P033 / P043 / P058 / P059 / P060 / P061。
REJECTED = 11：P036 / P041 / P042 / P044 / P046 / P047 / P048 / P049 / P051 / P057 / P062（核销正确类，逐条核过值正确）。
PARTIAL = 24：P002 / P021 / P022 / P028 / P029 / P030 / P032 / P034 / P035 / P037 / P038 / P039 / P040 / P045 / P050 / P052 / P053 / P054 / P055 / P056 / P063 / P064 / P065（时点值/建议类/低危表述面；其中 P037/P038/P058/P059/P060/P061 的项号确实错，但 FIND 建议数字自身错位 1 位 = 按勘误表正确值修，仍归 PARTIAL 因修复数字需勘误）。
NO-LINE = 1：P025（FIND 自述「与实测 106 一致」，本审计实跑 `crates/*/tests/*.rs` = 105，FIND 基准面 106 未独立核过；文档 23 号:10 值 106 vs 实测 105 本身存疑，FIND 判定方向误 = 该条实为潜在漂移被误核销，修复方须复核）。

合计 29 + 11 + 24 + 1 = 65 ✓。

### 基准面勘误（FIND 头部机器位表错位，影响 8 条 CONFIRMED 的建议数字）
FIND 头面「gate 机器 36 项」表把 no-macroquad 两变体（成功/失败分支）各计 1 项 = 37 行硬凑 36（漏了 Android 真机 4c 行 = 第 7 位），导致其第 20 位以后全部机器位 -1：
FIND 写 render-invariants=26 / check_data_source=27 / doccheck2=21 / flywheel=25 / check_scene_narrative=29 / check_data_driven=17(✓巧合) /
dod-commands=34 / studio-tests=35 / browser-playtest=32；
本机去重口径（与 doccheck2 `check_gate_item_count` 同实现，36 项 = no-macroquad 两变体各计 1）实测：
render-invariants=27 / check_data_source=28 / doccheck2=22 / flywheel=26 / check_scene_narrative=30 / dod-commands=35 /
studio-tests=36 / browser-playtest=33。
⇒ P001/P002/P058/P059/P060/P061 等「建议 N→X」数字按 FIND 表写会修错 1 位，修复时须按上表正确值。

### 最高价值 8 条 CONFIRMED
1. P023 — test_manifest.json gate_full 机读数据 name「33 项」/desc「35 项」/机器 36 三值互斥，平台测试页直接展示 + validate_data 值域检查管不到 name/desc 文本（纯数据文件漂移，修 1 处 JSON 即消）。
2. P024 — 23 号 evidence 计数 535 vs 实测 543，doccheck 回写面缺口（EXP-189 计数面同模式，S56.0 直接输入）。
3. P017 — 09 号 render-framework-boundary 写 22 实为 24（doccheck2 项号体系错，渲染面必读文档）。
4. P019 — 11 号 terrain-data 写 25 实为 27（同文件 :88 写 26 正确，并存易修错）。
5. P020 — 32 号 rules-files 写 43 实为 42（R12 删项后偏移，S50/S56 被当现行引用）。
6. P043 — plan-s50 写「第 43 项 check_rules_files」实为 42（计划文档无时点标注被当现行）。
7. P003/P004/P005/P006 — 04_tooling 速查表 7 行项号全错（doccheck 9→13 / 数据表门禁 10→14~16 / oracle 11→19 / ledger_reality 14→23），新人第一入口，4 条独立 CONFIRMED。
8. P027 — 14 号 playbook「16 条」vs 实测 -List 20 行/switch 22 分支，工具契约面（AGENTS.md:129「22 项」口径未标注，FIND 建议 21 亦错位）。

### 7 条必读回执
[1] docs/process/00_index.md — 分工表 + 标准工作流 ✓ 已读（路由表 + 改动面声明 + 硬规则 7/8/9/10 对照本审计红线）
[2] docs/business/00_index.md — 业务唯一事实源 ✓ 已读（本域 = 流程/工具面，业务规则无触面）
[3] docs/process/01_architecture.md + 02_conventions.md — 分层 + 数据驱动 ✓ 已读（doccheck2 项号 = gate_data.json 1-based 机器源，本审计按机器源判定）
[4] docs/contracts/00_index.md + c1/c2/c3 — 跨包公共面 ✓ 已读（console_commands.json ↔ 22 号 §4.2 白名单 = 平台/工具包公共面，P048 核销）
[5] docs/experience/index.md — EXP 总表 ✓ 已读（EXP-189 项号文字漂移高频复现模式 = 本批 CONFIRMED 主体；EXP-058 台账三方对账口径参照）
[6] scripts/gate.ps1 + crates/ti-tools/src/bin/doccheck2.rs — 门禁项 + 断言清单 ✓ 已读（去重口径 36 项 + `check_gate_item_count` 实现逐行核过；doccheck2_checks 47 项 1-based 位表）
[7] 开轮协作五步（03_templates §7）✓ 已走（只读角色豁免草稿/批准两步；提问 = 65 条 P 判定 / 方案 = 逐条独立取证判定 + 基准面勘误 / 拍板 = 见五步回执 + 统计）

### 五步回执（只读角色：提问 + 方案 + 拍板）
- **提问**：R12 轨道 1 FIND 65 条（P001~P065）逐条 CONFIRMED/REJECTED/PARTIAL/NO-LINE 判定，重点独立复核项号类（gate 36 体系 vs doccheck2 47 体系）+ 计数类（341/543/105）+ 数值/路径类（test_manifest/version/wasm 字节）+ 时点标注类。
- **方案**：① 先跑 `flywheel_readme.ps1` 生成必读面 ② 独立机器实测 4 基准面（gate.ps1 Report 去重 36 / gate_data.json doccheck2_checks 47 1-based / ledger 341 拆分 / evidence 543 + tests 105 + wasm 字节）③ 逐条 grep/sed 原文取证（行号 + 值亲核）④ 对照判定（值错 = CONFIRMED；值对/建议类/时点注 = PARTIAL/REJECTED；行号失效 = NO-LINE）⑤ 发现 FIND 基准面机器位表错位 1 位（no-macroquad 两变体 37 行硬凑 36），8 条 CONFIRMED 建议数字需按本审计正确表修。
- **拍板**：65 条判定落盘 `/tmp/opencode/audit_r12_track1_audit.md`；CONFIRMED 29 / REJECTED 11 / PARTIAL 24 / NO-LINE 1；修复面 = 以本审计基准面（36 项去重位表 + 47 项 1-based 表）为准，勿沿用 FIND 头面错位位表；P025 翻案（106 vs 105，文档值 106 本身存疑待修复方复核）。
