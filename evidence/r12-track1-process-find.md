# R12 轨道 1 FIND — 开发流程面 doc↔code/data 值漂移（P001~P065）

只读审计（FIND 角色，未改任何仓库文件）。基准面（机器实测）：
- gate.ps1 机器 36 项（`Report` 行 trim 后去重首引号串口径 = `flywheel_readme.ps1 -Check`/doccheck2 `check_gate_item_actual_count` 同实现，实测 36）：1 fmt / 2 clippy / 3 test / 4 native build / 5 wasm build+clippy / 6 android build / 7 Android 真机(4c) / 8 clippy feature-gated / 9 占位 / 10 unwrap / 11 zero-unknown / 12 ledger / 13 doccheck / 14 validate_data / 15 gen_layout / 16 check_hardcode / 17 check_data_driven / 18 check_code_standards / 19 check_oracle_boundary / 20 no-macroquad / 21 doccheck2 / 22 check_ledger_reality / 23 check_forward_closed / 24 check_module_contracts / 25 flywheel-readme / 26 render-invariants / 27 check_data_source / 28 audit_code / 29 check_scene_narrative / 30 bevy-cpu-parity / 31 wasm-browser-e2e / 32 browser-playtest / 33 playtest-loop / 34 dod-commands / 35 studio-tests（20/21 no-macroquad 两变体名各计 1 项 = 36）。
- doccheck2 47 项（gate_data.json `doccheck2_checks` 1-based）：22 render-framework-boundary / 23 std-traits-matrix / 24 render-framework-boundary 后…=24 render-framework-boundary？——准确表：1 audio…15 engine-no-color-literal / 16 landscape / 17 save-slot / 18 exhaust / 19 experience-volume / 20 runtime-no-legacy-coords / 21 layout-doc-table / 22 no-ti-core-fullpath / 23 std-traits-matrix / 24 render-framework-boundary / 25 ui-redesign-tokens / 26 primitives-data / 27 terrain-data / 28 tooling-catalog / 29 tooling-contract / 30-41… / 42 rules-files / 43 theme-consumption / 44 cross-doc-counts / 45 business-doc-test-refs / 46 blocked-label / 47 doccheck-counts-match。
- 台账 = 341 任务行（339 verified + S56.0 pending + S49.1 blocked），实测 `plan/ledger.yaml`。
- doccheck 实测 = 179 docs / 38 facts / 186 experiences / 335 task refs / 4 assertions / 0 error。
- evidence git-tracked（排除 console/）= 543 文件；集成测试文件 = 106。
- playbook.ps1 命令 = 21（switch 21 个 + 内部 health-check 不在 -List 清单；-List 打印 20 行，与 22 不吻合）。
- R12 已修 = gate_data.json 内各「第 N 项」文字（本审计只报文档残留同类，不重复报 gate_data 已修项）。

---

## P001
docs/process/07_render_pipeline.md:20 / :120 / :201 / :234（共 4 处）
render_invariants 声称「gate 第 19 项」，机器实测第 26 项（render-invariants）；同段 check_data_source 第 20/121/125 项声称 = 实测第 27 项。
建议：统一改 26/27 或改引用「render-invariants/check_data_source 项」去项号化。

## P002
docs/process/07_render_pipeline.md:25 / :121
「第 20 项 数据源门禁」表行 + 「第 20 项 FAIL」纪律句 = 实测 27 项。
建议：20→27 两处。

## P003
docs/process/04_tooling.md:59 / :75
doccheck 声称「gate 第 9 项」（§已建表 + 「gate 项数」注释块两处）= 实测 13 项。
建议：9→13（带 S8.2 时点注或去项号）。

## P004
docs/process/04_tooling.md:61 / :82
check_oracle_boundary 声称「gate 第 11 项」（两处）= 实测 19 项。
建议：11→19。

## P005
docs/process/04_tooling.md:63 / :83
check_ledger_reality 声称「gate 第 14 项」（两处）= 实测 22 项。
建议：14→22。

## P006
docs/process/04_tooling.md:52 / :76 / :92
「gate 第 10 项 = 数据表门禁（validate_data+gen_layout+check_hardcode）」三处 = 实测 14/15/16 三项分列（validate_data=14、gen_layout=15、check_hardcode=16）。
建议：改「第 14~16 项」或去项号。

## P007
docs/process/05_module_parallel.md:122
「第 15 项查业务闭环、第 16 项查包白名单/契约」= 实测 check_forward_closed=23、check_module_contracts=24。
建议：15→23、16→24。

## P008
docs/process/06_flywheel.md:112 / docs/process/00_index.md:77 / docs/process/26_dev_stages_overview.md:105
「gate 第 17 项 = flywheel-readme」三处 = 实测 25 项。
建议：17→25。

## P009
docs/process/08_playtest_loop.md:60
「gate 第 23 项同款前置（serve）」语境 = 指 web_playtest/browser-playtest 前置 = 实测 32 项。
建议：23→32。

## P010
docs/process/12_bevy_usage.md:349 / docs/process/11_bevy_migration.md:83 / docs/business/08_layout_design.md:124
check_data_driven 声称「gate 第 24 项」三处 = 实测 17 项。
建议：24→17。

## P011
docs/business/08_layout_design.md:115 / :159 / :212 / :239（4 处）
check_scene_narrative 声称「gate 第 22 项」= 实测 29 项。
建议：22→29。

## P012
docs/README.md:41
check_ledger_reality「gate 第 14 项」= 实测 22 项。
建议：14→22。

## P013
docs/README.md:56
07_render_pipeline 行「gate 第 19/20 项锚点」= 实测 26/27 项。
建议：19/20→26/27。

## P014
docs/README.md:60
「gate 第 28 项 check_scene_narrative」= 实测 29 项。
建议：28→29。

## P015
docs/README.md:13 / :36
「doccheck 门禁第 9 项」两处 = 实测 13 项。
建议：9→13。

## P016
docs/process/00_index.md:154
「新增文档登记 docs/README.md（门禁第 9 项自动校验）」= doccheck 实测 13 项。
建议：9→13。

## P017
docs/process/09_render_framework.md:11
「doccheck2 第 22 项 render-framework-boundary」= 实测第 24 项（第 22 项 = no-ti-core-fullpath）。
建议：22→24。

## P018
docs/process/10_ui_redesign.md:185
「doccheck2 第 23 项 check_ui_redesign_tokens」= 实测第 25 项（第 23 项 = std-traits-matrix）。
建议：23→25。

## P019
docs/process/11_bevy_migration.md:90
「doccheck2 第 25 项 check_terrain_data」= 实测第 27 项 terrain-data（25 = ui-redesign-tokens；同文件 :88「第 26 项 primitives-data」正确，勿误改）。
建议：25→27。

## P020
docs/process/32_ccgs_borrow.md:82
「门禁面 = doccheck2 第 43 项断言 rules-files」无时点标注，= 实测第 42 项（43 = theme-consumption；R12 修过 gate_data 同类，此为文档侧残留）。
建议：43→42（或加「S50 立项时点 43」标注）。

## P021
docs/process/31_test_ingress_standard.md:42
示例「`web_playtest` gate=23」；test_manifest.json 实测 web_playtest gate=23 但机器 23 = check_forward_closed，web_playtest 覆盖的是 browser-playtest=32。31 号 schema 行（:36/:76）写值域 1..=36 但示例项号错。
建议：示例改「web_playtest gate=32」或改用语义名。

## P022
docs/process/31_test_ingress_standard.md:36 / :76
gate 值域声称 `1..=36`；validate_data.py 实测还接受条件项号字符串（`4c`，`[0-9]+[a-z]?` 正则），android_device_test gate='4c' 在册 = 文档值域窄于工具。
建议：值域表述补「或条件项号如 4c」。

## P023
assets/data/test_manifest.json（gate_full 条目）
name =「全量门禁 33 项」/ desc =「跑全量门禁 35 项」/ 无 gate 字段；机器实测 36 项，且 name≠desc 自相矛盾（33 vs 35）。
建议：name/desc 统一 36 项 + 加 gate 字段或 evidence 指针。

## P024
docs/process/23_game_testing.md:10
「`evidence/` 535 文件（R11 复验 534→535）」；git ls-files 实测 543（含 R12 新增 8）。
建议：doc_numbers.py 回写 543。

## P025
docs/process/23_game_testing.md:10
「106 个集成测试文件」与 crates/*/tests/*.rs 实测 106 ✓ —— 当前一致，列为观察点（doccheck2 无此断言，下次增减文件即漂移；S56.0 面可挂）。
建议：无需改；S56.0 声称点对账扩此点。

## P026
AGENTS.md:129
「常用命令一键入口（22 项命令固化）」；playbook.ps1 实测 -List 打印 20 条、switch 分支 21 个（内部 health-check 不列）。
建议：22→21（以 switch 分支为准）或去数字。

## P027
docs/process/14_tooling_catalog.md:118 / :254
「playbook.ps1 常用命令一键入口（16 条）」两处；S21.8 后脚本注释仍写 16，实测 21。
建议：16→21。

## P028
docs/process/27_landscape_adaptation.md:117 / :120 / :124 / :127 / :132 / :136 / :158 / :212（8 处）
S38 各任务收口句「gate 33 项 0 failure」无「时点」标记（同库其他历史计数均带时点/S 号标注）；现行机器 36。
建议：加「S38 时点（33）」或改「gate 全绿」。

## P029
docs/parity.md:108
计数链「S46.1 起 = 34 + 第 25 项 dod-commands；S49.4e 起 = 35 + 第 4c 项 Android 真机」与机器口径矛盾：dod-commands 现 = 第 34 位、Android 真机 = 第 7 位（4c 只是注释名）；且 S46.1/S49.4e 时点 dod-commands 在第 25/26 位也成立不了。
建议：计数链改「…= 35 + studio-tests 后 36；第 N 位以机器口径为准」。

## P030
docs/parity.md:115
「注释编号（1..20 + 19.5 + 4b/4c，含第 7 项 zero-unknown）」：zero-unknown 现机器 = 第 11 位（第 7 位 = Android 真机），注释面描述与现机器序号脱钩。
建议：注明「注释编号 ≠ 机器 1-based 位（零知识项位置随增项漂移）」。

## P031
docs/parity.md:123
「门禁第 25 项 dod-commands」= 实测第 34 项（第 25 位 = flywheel-readme）。
建议：25→34 或去项号。

## P032
docs/parity.md:218
「25b 元测试面」作为项号写 = 机器口径无 25b 位（studio-tests = 第 35 位；S50.2 时点 25b 为注释编号口径残留，无时点标注）。
建议：35 或「studio-tests 项」。

## P033
docs/process/22_dev_platform.md:40
「第 23 项 browser-playtest 无分档守卫」= 实测第 32 位。
建议：23→32。

## P034
docs/process/22_dev_platform.md:156 / :160 / :163（3 处）
「gate.ps1 第 17 项内部硬编码 powershell / 宿主解释器探测」语境 = flywheel-readme 项（S32.11 修复面）= 实测第 25 位。
建议：17→25。

## P035
docs/process/22_dev_platform.md:135
「S50.2 studio-tests 后 35→36」增量链与机器 36 终值 ✓ 但缺「第 35 位 = studio-tests」定位信息，易被误读为 25b；低危表述面。
建议：补「studio-tests = 机器第 35 项」。

## P036
docs/process/28_engine_best_practice.md:76 / :85
「engine-no-color-literal（第 15 项）」「机器守门（第 15 项）」= 实测 doccheck2 第 15 位 ✓ —— 正确，列为已核销观察点（R12 后复验通过，防后续删项复发误改）。
建议：无需改。

## P037
docs/process/08_render_arch.md:145 / :204
「doccheck2 第 20 项 runtime-no-legacy-coords」✓（gate_data.json 第 20 位）+「gate 第 13 项 doccheck2」✗（doccheck2 = gate 第 21 位，第 13 位 = doccheck）。
建议：gate 13→21。

## P038
docs/process/08_render_arch.md:205 / :207
「gate 第 13 项（doccheck2）」两处同 P037 误（且 :207 混写「gate 第 3 项 + 第 13 项」，3 = cargo test ✓、13 应为 21）。
建议：13→21。

## P039
docs/process/08_render_arch.md:224-229
基线镜像恒等段自称「历史记述——S18.7b 起已退役，以下 gate 项号 = 退役前时点口径」，但 :229「doccheck2 第 19 检查 baseline-mirror-identity」该检查已不在现行 47 项（退役）；「第 13 项」gate 位也非退役前时点（doccheck2 当时 = 第 13 位 ✓ 时点值，可接受，但「第 19 检查」无时点标注易被当现行）。
建议：:229 加「（S15 时点，已退役）」括注。

## P040
docs/process/08_render_arch.md:195
「gate 第 18/19/19.5 项的三套基线」无时点标记（S15.6 前旧位），与 :6 文件头「基线机制已删除」自相矛盾未注时点。
建议：加「S15 时点位」或改语义名。

## P041
docs/process/15_process_ledger.md:65 / :66
行 29/30 载体列「doccheck2::check_theme_consumption（第 43 项…）」「cross-doc-counts（第 44 项…）」= 实测 43/44 ✓ —— 正确核销（R12 修复后 15 号行未被漏改）。
建议：无需改。

## P042
docs/process/15_process_ledger.md:61
行 36 = S56.0 载体行（「doccheck2 项号/工具契约/gate 项名等项号文字漂移」）存在 ✓ 与 AGENTS.md 路由表 S56 行「15 号行 36」一致；行号 34/36 间无 35 行 = 行号非连续（34 后直接 36）。
建议：无碍（行号 = 台账任务粒度），若 S56 登记引用「行 35」需澄清。

## P043
docs/plan-s50-studio-interface.md:100-104 / :121
「doccheck2 加第 43 项断言 check_rules_files」（4 处）= S50 立项时点值（当时 43），现行 = 42（R12 删恒 PASS 项后 -1）；计划文档无时点标注，且该文件被 S56/路由表当现行引用。
建议：加「（S50 立项时点；现行 42）」或改 42。

## P044
docs/process/26_dev_stages_overview.md:141
「doccheck（179 docs/38 facts/186 experiences/335 task refs…）」与 doccheck 实测 179/38/186/335 ✓（轨道 3 回写后一致）—— 核销；同文件 2026-11-16 轨道 3 注「184→185 + 178→179 + 334→335 + 185→186」增量链 ✓。
建议：无需改（列为 S56.0 声称点对账观察点：该段每加 1 篇文档即漂移）。

## P045
docs/process/26_dev_stages_overview.md:138
「gate.ps1（36 项固定 Report）」总数 ✓；但同文件 :181 36 项清单面 = 29 个工具名 + 「…」截断，且「check_code_standards」排末位而机器 = 第 18 位、「flywheel」排在「模块契约」后而机器 = 第 25 位（24 后）✓ 序大致对但「台账 / doccheck」序 = 12/13 ✓；「渲染不变式」排 26 ✓；清单漏「check_data_driven 在 check_ledger_reality 前」实测 17 vs 22——清单 17 项 check_data_driven 排第 18 位（check_ledger_reality 后）。
建议：清单改「以 gate.ps1 机器位为准，勿手工重排」。

## P046
AGENTS.md:8
「当前 341 任务 / 339 verified + 1 pending（S56.0）+ 1 blocked（S49.1）」= ledger.yaml 实测 341/339/1/1 ✓ —— 核销（S56.0 轨道 3 登记后一致）。
建议：无需改；列为 S56.0 高频漂移点（每次 task_close 即 ±1）。

## P047
docs/process/24_apk_size_std.md + 25_versioning.md 版本面
Cargo.toml 0.2.0 = AndroidManifest versionName = release.md 0.2.0；磁盘 web/treasure_bevy_wasm-0.2.0.wasm = 45,794,950 B = release.md:41 ✓ —— 全核销，无漂移。
建议：无需改。

## P048
crates/ti-console/console_commands.json（24 id）vs docs/process/22_dev_platform.md §4.2 白名单表
24 id 两侧集合相等、enabled_by_default 标志一致、ui_theme 行在（R10/R11 修过）—— 核销，无缺失行。
建议：无需改（S56.0 面 = console_commands.json ↔ 22 号表 双向对账）。

## P049
docs/process/31_test_ingress_standard.md:48
程序白名单「cargo/python3/pwsh/powershell（镜像平台 run_test 白名单）」与 validate_data.py 实测白名单集合一致 ✓；但 §3 禁 shell 拼接面「;&&|>`$(」与 23 号/22 号 §10.3 表述一致 ✓。
建议：无需改（核销）。

## P050
docs/process/23_game_testing.md:45
「gate 第 4c 项 + 验收判据（android_device_test）」与 parity.md:82 一致（4c = Android 真机条件项名）✓；但 4c 机器位 = 第 7 位，「第 4c 项」为注释命名面，两处互洽、与 31 号值域表述（P022）需同步。
建议：统一注「4c = 条件项名（机器第 7 位）」。

## P051
docs/process/14_tooling_catalog.md:18 / :31-32 / :44 / :50
「doccheck2 第 28 项 tooling-catalog / 第 29 项 tooling-contract」✓（gate_data.json 28/29 位正确）—— 核销（R12 漏改高发区但本处幸存）。
建议：无需改。

## P052
docs/process/14_tooling_catalog.md:189 / docs/process/04_tooling.md:64 / :65
「dod-commands = gate 第 25 项」「studio_tests = gate 第 25b 项」（三处）；机器实测 dod-commands = 34、studio-tests = 35（parity.md 口径 25/25b 为 S46.1/S50.2 时点位残留，无时点标注）。
建议：25→34、25b→35（或去项号改语义名）。

## P053
docs/process/04_tooling.md:64
check_dod_commands 行「gate 第 25 项」= P052 同源，单独列防漏修。
建议：25→34。

## P054
docs/process/04_tooling.md:65
studio_tests 行「gate 第 25b 项」= P052 同源，单独列防漏修。
建议：25b→35。

## P055
docs/process/22_dev_platform.md:22 号 §10.3（平台 run_test 白名单段）
平台 `run_test` 命令白名单 = 24 个 console_commands id 中 `gate`/`smoke_ps1` 用 powershell（Linux 映射 pwsh）——与 31 号 §3 白名单「cargo/python3/pwsh/powershell」一致 ✓；但 22 号 §10.3 未声明「test_manifest command 首词白名单 = validate_data 同集合」的双向对账关系（studio_tests ② 只做 manifest↔console_commands 映射，不查首词∈白名单）。
建议：22 号 §10.3 补一句「test_manifest command 白名单以 31 号 §3 为准」。

## P056
docs/process/08_render_arch.md:229
「doccheck2 第 19 检查 baseline-mirror-identity」= 该检查已退役（47 项中无），段内 :224 注「历史记述」但 :229 的「第 19 检查」本身无退役标注（P039 同段，单独列防漏修 :229 行）。
建议：:229 加「（已退役，S15 时点口径）」。

## P057
docs/process/11_bevy_migration.md:88
「check_primitives_data（doccheck2 第 26 项）」✓ = gate_data.json 第 26 位（primitives-data）—— 核销（同文件 :90 第 25→27 错见 P019，两处并存勿改错）。
建议：无需改（仅 :90 修 27）。

## P058
docs/process/26_dev_stages_overview.md:105
「开轮必读 7 条〔S51.0〕+ 停机条件（gate 第 17 项锚点）」= P008 同源（flywheel-readme 实测 25），单独列防漏修 26 号。
建议：17→25。

## P059
docs/process/07_render_pipeline.md:123
「元素级验证…与第 19 项全帧 diff 互补」= render-invariants 实测 26（P001 同文件第 5 处）。
建议：19→26。

## P060
docs/process/07_render_pipeline.md:201 / :234（第 6/7 处）
「render_invariants 第 19 项」两处 = 实测 26。
建议：19→26。

## P061
docs/process/00_index.md:77
「06_flywheel…gate 第 17 项」= P008 同源第 2 处（00_index 与 06_flywheel:112 分属不同文件，单独列防漏修 00_index）。
建议：17→25。

## P062
docs/process/23_game_testing.md:29 / :65 / :77 / :203
「门禁 36 项」4 处 ✓ = 机器 36 —— 核销（R10 勘误后一致）。
建议：无需改。

## P063
docs/release.md:28（0.2.0 版本行）
wasm sha256 前缀 + 45,712,494 B（发布时点）vs 磁盘 45,794,950 B（release.md:41 收口值）—— 两处值不同（发布回填 vs 收口复测），:28 标注「APK 构建时回填」，属时点值非漂移；无「回填后更新」机制说明。
建议：:28 补「收口值以 :41 行 45,794,950 B 为准」或加时点注。

## P064
docs/process/27_landscape_adaptation.md:158 / :212
S38 时点「gate 33 项 0 failure」2 处 = P028 同源（该文件共 8 处，:117/:120/:124/:127/:132/:136 已列 P028，此处列 :158/:212 防漏修）。
建议：加 S38 时点注。

## P065
docs/loop.md:81-92（state.md 纪律段）
「压缩前 3 要素写 evidence/<task>.state.md」纪律存在但仓库 0 个 *.state.md 产物（15 号行 28 = S50.4 审计确认「无 state.md 产出」）；纪律从未被触发，无机器守门（15 号自认「半自动」）。
建议：低危；S56.0 面可加「state.md 存在性 = 0 触发」口径注。

---
### 统计
- 共 65 条（P001~P065）；其中：真漂移需修 = 46 条，核销（实测一致/已修对，防复发观察）= 17 条（P025/P036/P041/P044/P046/P047/P048/P049/P050/P051/P057/P062 + 同源拆条防漏修 3 条），低危表述/时点标注建议 = 2 类。
- 高频类 Top：①gate 工具项号文字漂移（P001-P016, P031, P033-P034, P037-P038, P052-P054, P058, P060-P061 ≈ 30 条，EXP-189 同模式，S56.0 声称点对账面的直接输入）②doccheck2 项号（P017-P020, P039, P043, P056 ≈ 8 条）③总数/计数面（P023, P024, P026-P027, P029-P032, P045, P046 ≈ 12 条）④值域/示例面（P021-P022, P055）⑤时点标注缺失（P028, P040, P063-P064）。

### Top-5 最高价值
1. P023 test_manifest gate_full name 33/desc 35 vs 机器 36 + 自身 name≠desc 矛盾（机读数据文件漂移，平台测试页直接展示，validate_data 值域检查管不到 name/desc 文本）。
2. P001~P002 07_render_pipeline 8 行「第 19/20 项」（单一文件最密集项号漂移，该文件是渲染面必读）。
3. P003~P006 04_tooling 7 行「第 9/10/11/14 项」（工具台账速查表，新人第一入口）。
4. P024 23 号 evidence 535 vs 实测 543（R12 后 8 个新文件即漂，EXP-189 计数面同模式）。
5. P026+P027 命令集「22 项/16 条」双版本 vs 实测 21（AGENTS.md + 14 号两处各自写错成两个不同数，工具契约面）。
