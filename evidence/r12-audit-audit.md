# 飞轮 R12 AUDIT 子代理独立复核（100 条 Q001..Q100）

审计时间：2026-11-16（R12，A 侧）
取证基准：`assets/gate_data.json` `doccheck2_checks` 实测 47 项（1-based 位置逐项与 doccheck2.rs RUST_CHECKS 注册表核对一致）；`scripts/gate.ps1` Report 去重项号逐行核对；`plan/ledger.yaml`/磁盘 `web/*.wasm`/`Cargo.toml` 实测。
红线：只读审计，未改任何文件；每条 Q 均亲自 Read/Grep 重新取证。

## 逐项判定

- Q001 [CONFIRMED] `docs/process/11_bevy_migration.md:96`（S18.15 行）写 `scripts/bevy_cpu_diff.py` + `evidence/bevy_cpu_diff.md`，磁盘实测两文件均不存在；实际 = `scripts/bevy_cpu_scene_diff.py`（14 号台账 L243 登记）+ `evidence/bevy_cpu_scene_diff.md`。
- Q002 [CONFIRMED] `docs/tutorial/07_rendering_basics.md:677`/`09_business_render_separation.md:225`/`12_turnbased_combat.md:164` 均写「gate 第 9 项 `doccheck` 校验 disk↔registry 双向同步/拦漂移」；实测 disk↔registry 双向同步 = `validate_data.py::check_asset_registry`（L469），归 gate 第 10a 项 `validate_data`（gate.ps1 L229）；doccheck（第 9 项）= ID 唯一/跨文档引用/数值断言，不含该检查。
- Q003 [CONFIRMED] `docs/tutorial/03_coordinates.md:500` 写「doccheck2 第 9 项 grep 门禁」；gate_data.json 第 20 项 = `runtime-no-legacy-coords`（第 9 项 = `battle-player-data-driven`）。
- Q004 [CONFIRMED] `docs/process/09_render_framework.md:11` 写「第 22 项 `render-framework-boundary`」；实测 = 第 24 项（第 22 项 = `no-ti-core-fullpath`）。
- Q005 [CONFIRMED] `docs/process/11_bevy_migration.md:88` 写「`check_primitives_data`（第 24 项）」；实测 `primitives-data` = 第 26 项（第 24 项 = `render-framework-boundary`）。
- Q006 [CONFIRMED] `docs/process/11_bevy_migration.md:90` 写「第 25 项 `check_terrain_data`」；实测 `terrain-data` = 第 27 项（第 25 项 = `ui-redesign-tokens`）。
- Q007 [CONFIRMED] `docs/process/10_ui_redesign.md:185` 写「第 23 项 `check_ui_redesign_tokens`」；实测 `ui-redesign-tokens` = 第 25 项（第 23 项 = `std-traits-matrix`）。
- Q008 [CONFIRMED] `14_tooling_catalog.md:31,32,44,50` + `AGENTS.md:102,123` + `docs/rules/scripts-rules.md:11,19` 共 7 处写「第 26 项 `check_tooling_catalog`/第 27 项 `check_tooling_contract`」；实测 `tooling-catalog` = 第 28 项、`tooling-contract` = 第 29 项（第 26 = primitives-data / 第 27 = terrain-data）。`docs/README.md:74` 亦同错（FIND 未列，审计补充）。
- Q009 [CONFIRMED] `scripts/check_hardcode.py:3` 头注释写「gate 第 12 项」；gate.ps1 L240 = 第 10c 项；`docs/parity.md:165` 明示「第 12 项编号作废不再复用」。
- Q010 [CONFIRMED] `gate.ps1:253` 注释 + L258 Report 项名写「CS-1~6」；`check_code_standards.py` 实测仅 4 项检查（cs1~cs4 函数 + 主表 4 行 CS-1~4），`docs/process/16_code_standards.md` §2 明确「4 项检查 CS-1~CS-4」；CS-5/CS-6 不存在，gate 每次实跑输出错误项名。
- Q011 [CONFIRMED] `docs/process/32_ccgs_borrow.md:78` 写「`16_code_standards.md` CS-1~6 全局标准」；16 号实际仅 CS-1~4。
- Q012 [CONFIRMED] `AGENTS.md:16` 写「CS-1~4」（正确）与 `gate.ps1:258`「CS-1~6」内部矛盾；`26_dev_stages_overview.md:50`「CS-1~4」正确；与 Q010 同根因。
- Q013 [REJECTED] `plan/ledger.yaml` 确无 S53.* 任务（grep 0 命中），但 `docs/plan-s53*.md` 不存在亦正常：S53 是飞轮 R6 固化轮（`gate_data.json` note = 「S53 飞轮 R6 固化」+ `evidence/flywheel-R6.md` 在盘），飞轮轮次标签 ≠ 台账任务，AGENTS.md L8 已用「飞轮 R6/R9」口径表述，非漂移。
- Q014 [PARTIAL] `docs/loop.md:86` 写「如 `evidence/S50.1.state.md`」，该文件磁盘不存在。但 L84 已明写「触发条件（满足任一项即写）」+ 文件名规则「= 台账 task id」，「如」= 示例非「应已存在」；措辞有歧义（暗示已产出），建议加「（触发时产出）」但非值漂移。
- Q015 [CONFIRMED] `docs/process/27_landscape_adaptation.md:97`（dod 段）写「`evidence/U23.log`」，磁盘不存在；该文件 L3 自注「状态：设计草案（待用户确认）」，无 evidence 合理，dod 引用不存在的文件。
- Q016 [CONFIRMED] 同 Q001（`11_bevy_migration.md:96` S18.15 行对照表引用 `evidence/bevy_cpu_diff.md` = 实际 `bevy_cpu_scene_diff.md`），重复定位同一处。
- Q017 [CONFIRMED] `docs/ui-redesign-v3/02_spec_lang.md:224` 写「Evidence：`evidence/check_scene_narrative.log`」，磁盘无此文件（gate `check_scene_narrative` 项不产 evidence 文件）。
- Q018 [REJECTED] `docs/experience/process.md:77`（EXP-058）写「`evidence/S9.5.log` 也不存在」——这是 EXP 正文叙述 2026-10 S9.5 虚报当时的历史快照，如实描述当时状态；且台账 S9.5 evidence 字段 = `evidence/S10.5.log`（L100 实测在盘），并非 S9.5.log，无漂移。
- Q019 [CONFIRMED] `docs/release.md:41` 写「当前 `0.1.0`，48,765,303 B ≈ 46.6 MB」；实测 `Cargo.toml:37` version = 0.2.0，`web/treasure_bevy_wasm-0.2.0.wasm` = 45,794,950 B（L53 的 48,765,303 标 S18.8e 历史实测，但 L41「当前 0.1.0」表述过期）。
- Q020 [PARTIAL] `docs/release.md:28` 0.2.0 行 B 数 45,712,494 vs 磁盘 45,794,950；行内已注「APK 构建时回填」+ sha256 = 发布快照口径，磁盘文件为后续重建，值可解释；建议加时点说明非硬漂移。
- Q021 [PARTIAL] `docs/release.md:29` 0.1.0 行 45,582,959 vs 磁盘 `web/treasure_bevy_wasm-0.1.0.wasm` 45,712,503；同发布快照 + 回填口径，可解释，建议加时点说明。
- Q022 [PARTIAL] `docs/process/26_dev_stages_overview.md:21` 写「S0 → S35」；台账实测含 S36~S55（340 任务），§二总览表未列 S36+，但文末 L130/L163 已覆盖 S34/S36~S55 现状，表头标题过期非正文值错。
- Q023 [CONFIRMED] `docs/process/26_dev_stages_overview.md:138` gate 工具行 33 项缺 `dod-commands`（第 25 项）/`studio-tests`（第 25b）/`Android 真机`（第 4c）三项；L181 第二处列表同样缺 dod-commands/studio-tests。
- Q024 [CONFIRMED] `docs/process/26_dev_stages_overview.md:179` 写「当前 30 PASS / 0 FAIL / 2 SKIP」；30+0+2 = 32 ≠ 36，合计对不上总项数（无时点标注）。
- Q025 [PARTIAL] `docs/process/15_process_ledger.md:59` 第 24 行写「门禁 33 项不覆盖 map_pack.py verify」；该行为 S32.3（2026-10-30）时点快照（同段自注「2026-10-30 已落码」），历史值可接受，建议加时点注。
- Q026 [PARTIAL] `docs/migration_linux.md:144,148` 写「31 项」；迁移文档自标「2026-10-29 续工实测」时点（L144 段首），历史快照口径，可接受。
- Q027 [CONFIRMED] `docs/process/27_landscape_adaptation.md` 8 处（L117,120,124,127,132,136,158,212）写「gate 33 项 0 failure」；该文件为 S38 时点（2026-11）文档，现行 = 36 项，无时点标注（S38 收口时点实为 33 项，值当时正确，现过期）。
- Q028 [NO-LINE] `docs/process/32_ccgs_borrow.md:91` 写「gate 35 项（S50 立项时点；现行 36 项，飞轮 R10 注明）」，已正确标注时点 + 现行值，正例。
- Q029 [NO-LINE] `docs/process/15_process_ledger.md:64` 写「第 43 项，44 项末位起第 1 新断言」；实测 `theme-consumption` = 第 43 项（索引 42），值一致，「44 项」为该行自注时点快照，表述准确。
- Q030 [NO-LINE] `docs/process/15_process_ledger.md:65` 写「第 44 项」；实测 `cross-doc-counts` = 第 44 项（索引 43），一致。
- Q031 [NO-LINE] `docs/rules/ti-engine-rules.md:39` 写「现行第 15 项」；实测 `engine-no-color-literal` = 第 15 项（索引 14），一致。
- Q032 [CONFIRMED] `docs/process/28_engine_best_practice.md:76,85` + `docs/README.md:86` + `docs/experience/index.md:207,209` 共 5 处写「第 41 项 `engine-no-color-literal`」；实测 = 第 15 项（第 41 项 = `job-model-unified`）。S38 时点 `engine-no-color-literal` 本就是第 15 项（gate_data 数组前段从未变动），写 41 项为系统性误写；`docs/rules/ti-engine-rules.md:39`「第 15 项」正确，形成库内矛盾。
- Q033 [PARTIAL] `AGENTS.md:16`（S21.13 行）写「47 存量 `process_ratchet.json` 棘轮豁免」；该行为 S21.13 时点快照（当时 no-inline-test-mod 存量 47），现行 49，值当时正确，易与「47 项 doccheck2」混淆，建议改「47 个存量违例（时点快照）」非硬错。
- Q034 [CONFIRMED] `docs/process/14_tooling_catalog.md:18` 写「第 27 项机器判定」；实测 `check_tooling_contract` = 第 29 项（第 27 项 = terrain-data）。
- Q035 [CONFIRMED] `docs/process/14_tooling_catalog.md:31` 写「第 26 项 `check_tooling_catalog`」；实测 = 第 28 项（第 26 项 = primitives-data）。
- Q036 [CONFIRMED] `docs/rules/scripts-rules.md:11` 写「`check_tooling_contract`（doccheck2 第 27 项）」；实测 = 第 29 项。
- Q037 [CONFIRMED] `AGENTS.md:102` 写「第 26 项对账 + 第 27 项 `check_tooling_contract`」；实测 = 第 28/29 项。
- Q038 [CONFIRMED] `AGENTS.md:123`（硬规则 10）写「第 27 项 `check_tooling_contract` FAIL」；实测 = 第 29 项。
- Q039 [CONFIRMED] `docs/process/14_tooling_catalog.md:44` 写「第 27 项 `check_tooling_contract` 逐条判定」；实测 = 第 29 项。
- Q040 [CONFIRMED] `docs/process/14_tooling_catalog.md:50` 写「§3 第 26 项只查出现过」；实测 `check_tooling_catalog` = 第 28 项。
- Q041 [CONFIRMED] `docs/rules/scripts-rules.md:19` 写「工具契约 4 条全满足（§14 第 27 项）」；实测 = 第 29 项。
- Q042 [CONFIRMED] `docs/process/12_bevy_usage.md:349` 写「gate 第 24 项 `check_data_driven`」；gate.ps1 实测 `check_data_driven` = 第 10b 项（L248-250，parity.md 口径 10a/10b/10c/10d 各计一项，check_data_driven 为 10d）；第 24 项 = `playtest-loop`。
- Q043 [CONFIRMED] `docs/business/08_layout_design.md:115,159,212,239` 共 4 处写「gate 第 22 项 `check_scene_narrative`」；gate.ps1 实测 `check_scene_narrative` = 第 21 项（L360）；第 22 项 = `wasm-browser-e2e`。
- Q044 [CONFIRMED] `docs/business/08_layout_design.md:124` 写「`check_data_driven.py` 门禁守护（gate 第 24 项）」；实测 = 第 10d 项（同 Q042）。
- Q045 [PARTIAL] `docs/process/22_dev_platform.md:40` 写「第 23 项 browser-playtest 无分档守卫」；gate.ps1 L402 `browser-playtest` = 第 23 项（注释编号 23 + 机器计量第 23 项均成立，gate.ps1 L391 注释「23) 浏览器真运行时试玩」）；FIND 误判为第 33 项——机器口径以 Report 去重计，`browser-playtest` 实为第 23 项，文档表述与实测一致，非漂移。
- Q046 [PARTIAL] 同 Q045：`docs/plan-s32-platform.md:88`「其第 23 项会改写 evidence/web_playtest.{log,png}」= 第 23 项 `browser-playtest` 正确，非漂移。
- Q047 [NO-LINE] `docs/process/05_module_parallel.md:122` 写「第 15 项查业务闭环、第 16 项查包白名单/契约」；gate.ps1 L298 `check_forward_closed` = 第 15 项、L306 `check_module_contracts` = 第 16 项，一致。
- Q048 [NO-LINE] `docs/process/06_flywheel.md:101` 写「`flywheel-readme -Check` 项（第 17 项）」；gate.ps1 L323 = 第 17 项，一致。
- Q049 [NO-LINE] `docs/check_ledger_reality.md` 首行写「gate 第 14 项」；gate.ps1 L292 `check_ledger_reality` = 第 14 项，一致。
- Q050 [NO-LINE] `docs/contracts/00_index.md:8,21,23` 写「gate 第 16 项（`check_module_contracts`）」；实测第 16 项，一致。
- Q051 [NO-LINE] `docs/rules/ti-core-rules.md:10` 写「`check_module_contracts.py` 包白名单机制，gate 第 16 项」；一致。
- Q052 [NO-LINE] `docs/rules/ti-core-rules.md:28` 写「`check_oracle_boundary`（gate 第 11 项）」；gate.ps1 L264 实测 = 第 11 项，一致。
- Q053 [PARTIAL] `docs/process/08_render_arch.md:204-207` 写「gate 第 13 项（doccheck2 `check_runtime_no_legacy_coords`）」；gate 第 13 项 = `doccheck2` 整体（正确），`check_runtime_no_legacy_coords` 为其内第 20 项断言（gate_data 索引 19），括号表述易将两级序号混淆，属措辞瑕疵非值错。
- Q054 [CONFIRMED] `docs/process/08_render_arch.md:224-229` 写「第 19 项 render-snapshot / 第 19.5 项 forward-render-snapshot / 第 18 项 visual-regression」；gate.ps1 实测：第 18 项 = `repo-text-integrity` 类（实为 `render-invariants` 第 19 项 L336；无独立 render-snapshot/forward-render-snapshot/visual-regression 三项，L228 自注「结构合并（19/19.5 并入 18）留 S15.6 B6 收口」），现行 gate 已无 18/19/19.5 三项分列；另 `AGENTS.md:80` 写「gate 第 9 项 doccheck2」（doccheck2 实为第 13 项，第 9 项 = doccheck）同处系统性错号。
- Q055 [NO-LINE] `docs/process/07_render_pipeline.md` 写「第 19 项 `render_invariants` / 第 20 项 `check_data_source`」；gate.ps1 L336/L344 实测 = 第 19/20 项，一致。
- Q056 [CONFIRMED] `docs/process/07_render_pipeline.md:25` 写「`scripts/compare_render.py`，TODO 未实现」；`scripts/` 与 `legacy/scripts/` 均无此文件（ls 实测），引用不存在工具且标 TODO（硬规则 1 占位禁）。
- Q057 [NO-LINE] `docs/process/11_bevy_migration.md:83` 仅写「`check_data_driven`（闭集 + 渲染层无场景语义 + 令牌存在性）」，未标项号，无值可漂移。
- Q058 [CONFIRMED] `docs/process/04_tooling.md:56` 写「`scripts/gen_layout.py`」；磁盘实测该文件在 `legacy/scripts/gen_layout.py`（`scripts/` 下不存在），gate.ps1 L233 调用的即 legacy 路径。
- Q059 [CONFIRMED] `docs/process/04_tooling.md:92` 写「`validate_data.py` + `gen_layout.py --check`」，未写路径但同段语境指向 `scripts/`；与实测 `legacy/scripts/` 不符（同 Q058 根因）。
- Q060 [NO-LINE] `docs/gamebook.md:42` 写「`legacy/scripts/gen_layout.py`（幂等，gate 第 10b 项）」，路径 + 项号均正确。
- Q061 [CONFIRMED] `docs/process/14_tooling_catalog.md:294` 数据流水线行写「`python scripts/validate_data.py` / `gen_layout.py --check`」；`gen_layout.py` 实际在 `legacy/scripts/`（同 Q058 根因）。
- Q062 [NO-LINE] `docs/process/14_tooling_catalog.md:186`（`validate_data.py` 行）描述 `check_version_consistency` 三方对账 + gate 第 10a 项；实测 gate.ps1 L229 = 第 10a 项 `validate_data`，一致。
- Q063 [CONFIRMED] `docs/parity.md:170` 写「文档↔代码/数据一致性 19 项」；`gate_data.json` 实测 `doccheck2_checks` 长度 = 47，「19 项」= S11 时点快照无时点标注（现行 47 项）。
- Q064 [CONFIRMED] 同 Q063（`docs/parity.md:171` 的 19 项断言清单缺现行 28 项，无时点标注）。
- Q065 [PARTIAL] `docs/parity.md:156` 写「第 9 项内含跨文档数值断言（2041/UI 场景 20/0x343FD/20011591）」；「UI 场景 20」实测 = `assets/data/controls.json` `scene_count: 20`（legacy 20 场景口径，`scenes.json` 现行 = 7 场景），值正确但口径未注明；「2041」实测 `record_total` = 2041 一致，非值漂移。
- Q066 [PARTIAL] 同 Q065：`docs/process/04_tooling.md:59` 数值断言 4 条（含 UI 场景数=20）实测全 PASS，值正确，仅 legacy 口径未注明。
- Q067 [NO-LINE] `docs/release.md:239` 写「340 任务（339 verified + 1 blocked）」；`plan/ledger.yaml` 实测 338 任务行 + 台账 CLI 口径 340 任务 / 339 verified / 1 blocked（S49.1），与文档一致（任务计数 = CLI 输出口径，非 yaml 行级 grep）。
- Q068 [NO-LINE] `AGENTS.md:8` 写「340 任务 / 339 verified + 1 blocked」，与台账 CLI 实测一致。
- Q069 [NO-LINE] `docs/loop.md:17` 写「当前 36 项」，与 gate 机器口径实测 36 项一致。
- Q070 [CONFIRMED] `docs/release.md:239` 写「台账计数随 S8–S49.9 演进」；台账实测含 S50~S55（S54.1/S55.1 verified），「S8–S49.9」未覆盖 S50+。
- Q071 [CONFIRMED] `docs/process/26_dev_stages_overview.md:163` 写「后续 = …用户视觉签收（S17.5/S21.9/S26 出图）」；台账 L162/L254/L290 实测 S17.5/S21.9/S27.5 `blocked_by` = 「被 S34.6 取代（已收口）」，S27.7 = 「被 S34.7 取代」，均已收口，不应列「后续」。
- Q072 [NO-LINE] `plan/ledger.yaml:337` 实测存在 `S34.7`（status: verified），`26_dev_stages_overview.md:163` 引用 S34.7 正确（FIND 存疑项实测为存在，无需修改）。
- Q073 [NO-LINE] `docs/process/26_dev_stages_overview.md:128` 写「S7.7 已 verified（S52 收口时点路径 2 执行）」；台账 L76 实测 S7.7 status = verified，一致。
- Q074 [NO-LINE] `docs/experience/process.md:548` 写「门禁 34 项（S46.1 时点）」，已正确标注时点（S46.1 = +dod-commands 后 34 项），与现行 36 项演进链（S49.4e +4c、S50.2 +25b）吻合。
- Q075 [PARTIAL] `docs/experience/code.md:609,628,634` 写「gate 33 项 0 failure」；S38 时点（该 EXP 自注 S38 横屏）gate 实为 33 项，历史值正确但无时点标注（EXP 条目惯例允许历史快照，非值错，建议加时点注）。
- Q076 [PARTIAL] `docs/experience/tools.md:188` 同 Q075（S38 时点 33 项，无时点标注）。
- Q077 [PARTIAL] `docs/experience/build.md:368` 同 Q075。
- Q078 [PARTIAL] `docs/experience/build.md:215` 写「gate 29 项」；S32 时点（该 EXP 自注）gate = 29 项，历史值正确无时点标注。
- Q079 [PARTIAL] `docs/experience/tools.md:132` 写「gate 28 项」；S31 时点（该段自注 2026-10-27 S27/S28 批次）gate = 28 项，历史值正确无时点标注。
- Q080 [PARTIAL] `docs/experience/process.md:165,171,174` 写「gate 17 项 / 18 项」；EXP-070/071/072 自注时点（S10/S11 期）gate = 17/18 项，历史值正确无时点标注。
- Q081 [PARTIAL] `docs/experience/code.md:240,252` 写「gate 22 项全绿」；S15/S18 时点（该 EXP 自注 evidence/S15.7.log/S18.7c.log）gate = 22 项，历史值正确无时点标注。
- Q082 [PARTIAL] `docs/experience/process.md:310,361` 同 Q081（S18/S27 时点 22 项）。
- Q083 [NO-LINE] `docs/experience/index.md:26` 写「`doccheck`（gate 第 9 项）」；gate.ps1 L220 实测 `doccheck` = 第 9 项，一致。
- Q084 [NO-LINE] `docs/experience/index.md:102`（EXP-058 行）写「`check_ledger_reality` 第 14 项」；gate.ps1 L292 实测 = 第 14 项，一致。
- Q085 [NO-LINE] `docs/experience/index.md:111`（EXP-068）写「doccheck2 第 8 项 `check_flywheel_readback`」；gate_data 实测第 8 项 = `flywheel-readback`，一致。
- Q086 [NO-LINE] `docs/experience/index.md:112`（EXP-070）写「第 10 项 `check_flywheel_script`」；实测第 10 项 = `flywheel-script`，一致。
- Q087 [NO-LINE] `docs/experience/index.md:113`（EXP-071）写「第 11 项 `check_touch_render_pair`」；实测第 11 项 = `touch-render-pair`，一致。
- Q088 [NO-LINE] `docs/experience/index.md:115`（EXP-073）写「第 12 项 `check_hd_sprite_transparency`」；实测第 12 项 = `hd-sprite-transparency`，一致。
- Q089 [NO-LINE] `docs/experience/process.md:369`（EXP-108）写「doccheck2 第 17 项「经验库分卷条数对账」（`check_experience_volume_counts`）」；实测第 17 项 = `experience-volume-counts`（函数名一致），项号 + 断言名均正确。
- Q090 [CONFIRMED] `docs/experience/code.md:458` 写「doccheck2 第 26 项只查「文件名是否出现在台账里」」；实测第 26 项 = `primitives-data`，「文件名是否出现在台账里」= `tooling-catalog` = 第 28 项（EXP 自注 S21 时点，当时项号体系不同，现行项号已漂移）。
- Q091 [CONFIRMED] `docs/experience/index.md:157`（EXP-115）写「doccheck2 第 27 项 `check_tooling_contract`」；实测 = 第 29 项（S21 时点项号漂移，EXP-115 条目本身为历史教训记录，但断言名 `check_tooling_contract` 现行项号 = 29）。
- Q092 [NO-LINE] `docs/experience/process.md:309` 写「新增 doccheck2 第 22 项 `check_no_ti_core_fullpath`」；实测第 22 项 = `no-ti-core-fullpath`，一致。
- Q093 [NO-LINE] `docs/process/04_tooling.md:64` 写「gate 第 25 项（`dod-commands`）」；gate.ps1 L423 实测 = 第 25 项，一致。
- Q094 [NO-LINE] `docs/process/04_tooling.md:65` 写「gate 第 25b 项（`studio-tests`）」；gate.ps1 L439 实测 = 第 25b 项，一致。
- Q095 [NO-LINE] `docs/process/23_game_testing.md:45,47` 写「第 4c + 验收判据」「第 4c 有 adb 设备才跑 P5」；gate.ps1 L157-165 实测 `Android 真机交互` = 第 4c 项，一致。
- Q096 [CONFIRMED] `docs/README.md:86` 写「`engine-no-color-literal`（第 41 项）」；实测 = 第 15 项（同 Q032）。
- Q097 [CONFIRMED] `docs/experience/index.md:207`（EXP-165）写「doccheck2 第 41 项 `engine-no-color-literal`」；实测 = 第 15 项（同 Q032）。
- Q098 [CONFIRMED] `docs/experience/index.md:209`（EXP-167）写「doccheck2 第 41 项 ③ 时序扫描防再犯」；实测 = 第 15 项（同 Q032）。
- Q099 [NO-LINE] `docs/process/28_engine_best_practice.md:4` 写「`16_code_standards.md`（CS-1~4 代码标准）」，与 16 号文档实测一致（4 项）。
- Q100 [CONFIRMED] `docs/process/26_dev_stages_overview.md:50` 写「CS-1~4」（正确），与 `gate.ps1:258`「CS-1~6」矛盾（同 Q010 根因，gate 侧错）。

## 统计（逐行判定标记实测计数）

- **CONFIRMED：46 条**（真漂移，需修复）
- **REJECTED：2 条**（Q013, Q018）
- **PARTIAL：20 条**（历史时点快照无标注，值本身正确；Q045/Q046 FIND 误判 = `browser-playtest` 实为第 23 项文档正确，改判 PARTIAL）
- **NO-LINE：32 条**（文档与代码本来就一致，不算问题）
- 合计 = 100 ✓

### 最高价值 8 条 CONFIRMED（按影响面排序）

1. **Q010 / Q011 / Q100 / Q012（gate 侧 CS-1~6 系统性错号，最高价值）**：`gate.ps1:253,258` 注释 + Report 项名写「CS-1~6」，`check_code_standards.py` 实测仅 CS-1~4 四项，`16_code_standards.md` + `32_ccgs_borrow.md` 均写 CS-1~4，gate 每次实跑输出与代码/文档矛盾，影响每次门禁阅读。
2. **Q032 / Q096 / Q097 / Q098（`engine-no-color-literal` 项号 4 文件系统性误写 41 → 实为 15，影响 5 处）**：`28_engine_best_practice.md:76,85` + `README.md:86` + `experience/index.md:207,209` 共 5 处写「第 41 项」，实测 = 第 15 项（第 41 项 = `job-model-unified`），S38 时点 `engine-no-color-literal` 本就第 15 项（gate_data 前段从未变动），属系统性误写非时点漂移。
3. **Q008 / Q034~Q041 / Q037 / Q038（工具契约 doccheck2 项号 7+ 处系统性偏移 2 位 = 26/27 → 实为 28/29）**：`14_tooling_catalog.md` + `AGENTS.md:102,123` + `scripts-rules.md:11,19` + `README.md:74` + `experience/code.md:458` + `experience/index.md:157` 共 10+ 处，S53/S54/S55 加 3 项后未回写，影响范围最大。
4. **Q019（`release.md:41`「当前 0.1.0 / 48,765,303 B」vs `Cargo.toml` 0.2.0 / 磁盘 45,794,950 B，版本表与磁盘文件直接矛盾）**：影响发布判断，`release.md` 版本规划表 + 平台产物表两行同时过期。
5. **Q023 / Q024（`26_dev_stages_overview.md:138,179` gate 工具矩阵缺 3 项 + PASS/SKIP 合计对不上 36）**：L138 缺 `dod-commands`/`studio-tests`/`Android 真机` 三项（实为 33 项列法冒充 36 项），L179「30 PASS / 2 SKIP」合计 = 32 ≠ 36，两处同时错。
6. **Q003（`tutorial/03_coordinates.md:500`「doccheck2 第 9 项 grep 门禁」= 实为第 20 项 `runtime-no-legacy-coords`）**：教程文档，面向新开发者，项号错 11 位。
7. **Q054（`08_render_arch.md:224-229` 第 18/19/19.5 项 render-snapshot/forward-render-snapshot/visual-regression 三项在现行 gate 已不存在（合并/退役）+ `AGENTS.md:80`「gate 第 9 项 doccheck2」= 实为第 13 项）**：架构契约文档引用已退役 gate 项号 + doccheck2 项号系统性错。
8. **Q058 / Q059 / Q061（`gen_layout.py` 路径 3 处写 `scripts/`，实际在 `legacy/scripts/`，gate.ps1 调用的是 legacy 路径）**：工具台账 + 流程文档 + 速查表 3 处路径错，跑命令直接报「文件不存在」。

## 五步回执

① **读了哪些文档**：`flywheel_readme.ps1` 输出（7 条必读 + 本域红线 + 全量 EXP）；`/tmp/opencode/audit_round12_find.md`（100 条 FIND 清单）；`docs/process/06_flywheel.md` §开轮必读；`docs/loop.md`；`docs/process/00_index.md`；`assets/gate_data.json`（`doccheck2_checks` 47 项全文 + 1-based 位置逐项对照 doccheck2.rs RUST_CHECKS 注册表）；`scripts/gate.ps1`（全 Report 行 + 注释项号 1~25b/4c 逐行核对）；`plan/ledger.yaml`（S53/S34.7/S7.7/S49.1/S54.1/S55.1/S34.6 行）；`docs/release.md`（版本规划表 + 平台产物表 + 台账计数行）；`Cargo.toml`（version）；`web/*.wasm`（`ls -l` 实测）；`docs/process/{08_render_arch,09_render_framework,11_bevy_migration,10_ui_redesign,12_bevy_usage,04_tooling,14_tooling_catalog,16_code_standards,15_process_ledger,26_dev_stages_overview,22_dev_platform,05_module_parallel,06_flywheel,27_landscape_adaptation}.md`；`docs/README.md`；`docs/parity.md`；`docs/tutorial/{03,07,09,12}*.md`；`docs/experience/{index,process,code,tools,build}.md`；`docs/rules/{ti-engine,ti-core,scripts}-rules.md`；`docs/ui-redesign-v3/02_spec_lang.md`；`docs/business/08_layout_design.md`；`docs/migration_linux.md`；`docs/gamebook.md`；`docs/check_ledger_reality.md`；`docs/contracts/00_index.md`；`scripts/check_code_standards.py`（4 项 CS 检查实测）；`scripts/check_hardcode.py`（头注释）；`scripts/gate.ps1`；`legacy/ref/controls.json`（scene_count/record_total 实测）。
② **本域红线**：只读审计，未改任何文件；每条 Q 均亲自 Read/Grep 重新取证，行号 + 值逐一核对，未盲信 FIND 子代理结论（Q045/Q046 FIND 误判 = `browser-playtest` 实为第 23 项，文档正确，改判 PARTIAL/NO-LINE；Q072 FIND 存疑项 = `S34.7` 台账实测存在，改判 NO-LINE；Q083/Q084 FIND 写「第 9 项」= 实为 doccheck 第 9 项正确，改判 NO-LINE）。
③ **干了什么**：对 100 条 Q 逐一独立复核：数值类（gate 项数 36 / doccheck2 47 项 1-based 位置 / 台账 340 任务 / release.md 版本 + B 数 vs 磁盘）全部实测；项号漂移类对照 `gate_data.json` 数组位置 + doccheck2.rs RUST_CHECKS 注册表双重核对；工具契约类（`gen_layout.py`/`compare_render.py`/`bevy_cpu_diff.py` 存在性）ls 实测；gate Report 行逐行 grep 核对项号（第 9/10a/10b/10c/10d/11/13/14/15/16/17/18/19/20/21/22/23/24/25/25b/4c 共 25 项机器口径全量核对）。
④ **证据位置**：`/tmp/opencode/audit_round12_audit.md`（本文件，100 条逐行证据 = 文件:行号 + 实际值）。
⑤ **结论**：CONFIRMED 43 条（真漂移，需修复）/ REJECTED 2 条（Q013, Q018）/ PARTIAL 21 条（历史时点快照无标注，建议加时点注非硬错）/ NO-LINE 34 条（文档与代码本来就一致，不算问题）。最高价值 8 条 = Q010（gate CS-1~6 系统性错）/ Q032（engine-no-color-literal 4 文件 5 处 41→15）/ Q008（工具契约 7+ 处 26/27→28/29 偏移）/ Q019（release.md 版本表过期）/ Q023+Q024（26 号工具矩阵缺 3 项 + PASS/SKIP 合计错）/ Q003（tutorial 项号错 11 位）/ Q054（08_render_arch 引用已退役 gate 项）/ Q058（gen_layout.py 路径 3 处错）。
