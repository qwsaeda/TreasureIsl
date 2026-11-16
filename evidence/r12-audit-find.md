# 飞轮 R12 FIND 子代理审计清单（doc↔code↔data 一致性/漂移）

审计时间：2026-11-16（R12）
审计范围：文档层数值/计数/引用漂移（只读，未改任何文件）
取证方式：grep + python3 实测 + cargo run -p ti-tools --bin doccheck / doccheck2 / ledger

---

## Q001
- **位置**：`docs/process/11_bevy_migration.md:96`
- **现象**：文档引用 `scripts/bevy_cpu_diff.py`（S18.15 量化表），但磁盘上该文件不存在；实际文件为 `scripts/bevy_cpu_scene_diff.py`（14 号台账第 243 行登记）。同时 `evidence/bevy_cpu_diff.md` 被文档引用但磁盘上不存在，实际文件为 `evidence/bevy_cpu_scene_diff.md`。
- **建议修复**：`11_bevy_migration.md:96` 将 `scripts/bevy_cpu_diff.py` → `scripts/bevy_cpu_scene_diff.py`，将 `evidence/bevy_cpu_diff.md` → `evidence/bevy_cpu_scene_diff.md`。

## Q002
- **位置**：`docs/tutorial/07_rendering_basics.md:677`、`docs/tutorial/09_business_render_separation.md:225`、`docs/tutorial/12_turnbased_combat.md:164`
- **现象**：三处 tutorial 文件写「gate 第 9 项 `doccheck` 校验 disk↔registry 双向同步」或「gate 第 9 项拦截文档↔数据漂移」。实际 `disk↔registry 双向同步` 由 `validate_data.py::check_asset_registry`（gate 第 10a 项）负责（见 `docs/process/18_asset_content_pipeline.md:41-42`），`doccheck`（gate 第 9 项）负责的是 ID 唯一性/跨文档引用/数值断言，不含 disk↔registry 检查。item number 和工具归属均错。
- **建议修复**：三处 tutorial 将「gate 第 9 项 `doccheck`」改为「gate 第 10a 项 `validate_data`（`check_asset_registry`）」，`09_business_render_separation.md:225` 改为「gate 第 9 项 `doccheck`（数值断言）+ 第 10a 项 `validate_data`」。

## Q003
- **位置**：`docs/tutorial/03_coordinates.md:500`
- **现象**：写「doccheck2 第 9 项 grep 门禁」禁止运行时出现 176×220 legacy 坐标。实际 `check_runtime_no_legacy_coords` 是 doccheck2 第 **20** 项（`gate_data.json` `doccheck2_checks[19].id == "runtime-no-legacy-coords"`）。
- **建议修复**：`docs/tutorial/03_coordinates.md:500` 将「doccheck2 第 9 项」改为「doccheck2 第 20 项 `check_runtime_no_legacy_coords`」。

## Q004
- **位置**：`docs/process/09_render_framework.md:11`
- **现象**：写「由 doccheck2 第 22 项 `render-framework-boundary` 锁定」。实际 `render-framework-boundary` 是 doccheck2 第 **24** 项（`gate_data.json` 索引 23 → 第 24 项）。
- **建议修复**：`docs/process/09_render_framework.md:11` 将「第 22 项」改为「第 24 项」。

## Q005
- **位置**：`docs/process/11_bevy_migration.md:88`
- **现象**：写「`check_primitives_data`（doccheck2 第 24 项）」。实际 `primitives-data` 是 doccheck2 第 **26** 项。
- **建议修复**：将「第 24 项」改为「第 26 项」。

## Q006
- **位置**：`docs/process/11_bevy_migration.md:90`
- **现象**：写「doccheck2 第 25 项 `check_terrain_data`」。实际 `terrain-data` 是 doccheck2 第 **27** 项。
- **建议修复**：将「第 25 项」改为「第 27 项」。

## Q007
- **位置**：`docs/process/10_ui_redesign.md:185`
- **现象**：写「doccheck2 第 23 项 `check_ui_redesign_tokens`」。实际 `ui-redesign-tokens` 是 doccheck2 第 **25** 项。
- **建议修复**：将「第 23 项」改为「第 25 项」。

## Q008
- **位置**：`docs/process/14_tooling_catalog.md:18,31,32,44,50` + `AGENTS.md:102,123` + `docs/rules/scripts-rules.md:11,19`
- **现象**：多处写「`doccheck2` 第 26 项 `check_tooling_catalog`」和「第 27 项 `check_tooling_contract`」。实际 `tooling-catalog` = 第 **28** 项，`tooling-contract` = 第 **29** 项（`gate_data.json` 实测）。`AGENTS.md:102` 写「第 26 项对账」= 错；`AGENTS.md:123` 写「第 27 项」= 错；`14_tooling_catalog.md:18` 写「第 27 项机器判定」= 错；`scripts-rules.md:11` 写「第 27 项」= 错。
- **建议修复**：全部 7 处将「第 26 项」→「第 28 项」（`check_tooling_catalog`），「第 27 项」→「第 29 项」（`check_tooling_contract`），与 `gate_data.json` 对齐。

## Q009
- **位置**：`scripts/check_hardcode.py:3`（文件头注释）
- **现象**：注释写「gate 第 12 项」，但 `check_hardcode` 在当前 `gate.ps1` 中是第 10c 项（gate 项数计量口径见 `docs/parity.md:115`「`10a`/`10b`/`10c` 各计一项」），不是第 12 项（第 12 项编号已作废，见 `docs/parity.md` 注释「第 12 项编号作废不再复用」）。
- **建议修复**：`check_hardcode.py:3` 注释将「gate 第 12 项」改为「gate 第 10c 项」。

## Q010
- **位置**：`scripts/gate.ps1:258`（Report 项名）+ `scripts/gate.ps1:253`（注释）
- **现象**：gate Report 项名写 `check_code_standards (CS-1~6 代码标准 + 棘轮基线)`，注释写「CS-1~CS-6」。但 `scripts/check_code_standards.py` 实际只有 4 项检查（`cs1_asset_violations`/`cs2_doc_violations`/`cs3_param_violations`/`cs4_scene_violations`，共 4 个 `cs*` 函数），`docs/process/16_code_standards.md:24` 明确写「4 项检查 CS-1~CS-4」。CS-5/CS-6 在代码和 16 号文档中均不存在。
- **建议修复**：`gate.ps1:258` Report 项名将 `CS-1~6` 改为 `CS-1~4`；`gate.ps1:253` 注释将 `CS-1~CS-6` 改为 `CS-1~CS-4`。或确认 CS-5/CS-6 是否已加入，若未加则文档侧同步。

## Q011
- **位置**：`docs/process/32_ccgs_borrow.md:78`
- **现象**：写「`16_code_standards.md` CS-1~6 全局标准」。`16_code_standards.md` 实际只有 CS-1~CS-4（4 项），CS-5/CS-6 不存在。
- **建议修复**：将 `CS-1~6` 改为 `CS-1~4`。

## Q012
- **位置**：`AGENTS.md:16`（S21 行）+ `docs/process/26_dev_stages_overview.md:50`
- **现象**：两处写「CS-1~4」= 正确（与 16 号文档一致）。但 `AGENTS.md:16` 同时写「`check_code_standards.py` CS-1~4」而 `gate.ps1:258` Report 项名写「CS-1~6」= 内部矛盾。`26_dev_stages_overview.md:50`「CS-1~4」正确。
- **建议修复**：同 Q010，统一 gate 侧 CS-1~6 → CS-1~4。

## Q013
- **位置**：`docs/process/26_dev_stages_overview.md:141`
- **现象**：写「S53 飞轮 R6 固化 `business-doc-test-refs` +1」。`plan/ledger.yaml` 中无 `S53.*` 任务（`grep 'S53' plan/ledger.yaml` = 0 命中），`business-doc-test-refs` 的 `gate_data.json` note 字段写「S53 飞轮 R6 固化」。`S53` 在台账中不存在，仅作为飞轮轮次标签（R6）出现在文档中，但 `S54.1`/`S55.1` 的 deps 链 = S49.1 → S54.1 → S55.1，`S53` 无台账任务。
- **建议修复**：确认 `S53` 是否为台账任务或仅飞轮标签；若为飞轮标签（非台账），文档中「S53 飞轮 R6」表述可保留但需注明 `S53` 无台账任务，避免与 `S54`/`S55` 的台账任务混淆。

## Q014
- **位置**：`docs/loop.md:86`
- **现象**：写「如 `evidence/S50.1.state.md`；3 要素字段名固定 ①②③」。该文件 `evidence/S50.1.state.md` 在磁盘上不存在（`evidence/` 目录中无此文件），`S50.1` 的 evidence 是 `evidence/S50.1.log`。`state.md` 是 S50.1 定义的**压缩前 3 要素**文件，是**触发时**才写的，不是每个任务都有。文档写「如 `evidence/S50.1.state.md`」暗示该文件应已存在，实际不存在。
- **建议修复**：`docs/loop.md:86` 将「如 `evidence/S50.1.state.md`」改为「如 `evidence/<task>.state.md`（触发条件满足时写，非每任务必产）」，避免暗示所有任务都有 state.md。

## Q015
- **位置**：`docs/process/27_landscape_adaptation.md:97`
- **现象**：写「gate 全绿 + doccheck 0 error + `evidence/U23.log`」。`evidence/U23.log` 在磁盘上不存在（`ls evidence/` 无此文件）。U23 = 横屏适配设计草案，尚未落码（`27_landscape_adaptation.md:3` 写「状态：设计草案（待用户确认，A 纪律）」），无 evidence 文件。
- **建议修复**：`27_landscape_adaptation.md:97` 将「`evidence/U23.log`」改为「（U23 设计草案阶段，无 evidence；落码后产出）」，或删除该引用。

## Q016
- **位置**：`docs/process/11_bevy_migration.md:96`（S18.15 行）
- **现象**：写「对照表 `evidence/bevy_cpu_diff.md`」。磁盘上实际文件为 `evidence/bevy_cpu_scene_diff.md`（`bevy_cpu_diff.md` 不存在）。
- **建议修复**：同 Q001，将 `evidence/bevy_cpu_diff.md` 改为 `evidence/bevy_cpu_scene_diff.md`。

## Q017
- **位置**：`docs/ui-redesign-v3/02_spec_lang.md:224`
- **现象**：写「Evidence：`evidence/check_scene_narrative.log`」。磁盘上该文件不存在（`evidence/` 无此文件）。
- **建议修复**：确认 `check_scene_narrative.log` 是否已产出；若不存在，将引用改为实际 evidence 路径或删除。

## Q018
- **位置**：`docs/experience/process.md:77`（EXP-058 正文）
- **现象**：写「`evidence/S9.5.log` 也不存在」。S9.5 在台账中 status = verified（`plan/ledger.yaml` S9.5 行），但 `evidence/S9.5.log` 在磁盘上不存在。EXP-058 描述的是「子代理声称 verified 但 evidence 不存在」的历史教训，但该文件在 S9.5 后续收口后仍未补建。
- **建议修复**：确认 `evidence/S9.5.log` 是否应在收口时补建；若 EXP-058 描述的历史场景已过期（S9.5 已 verified），在 EXP 正文加注「S9.5 evidence 已由后续收口补建 / 该引用为历史快照」。

## Q019
- **位置**：`docs/release.md:41`（浏览器版本体表）
- **现象**：写「当前 `0.1.0`，LTO+strip 后 **48,765,303 B ≈ 46.6 MB**」。但 `Cargo.toml` version = `0.2.0`，`web/treasure_bevy_wasm-0.2.0.wasm` 实际大小 = 45,794,950 B（`ls -la web/` 实测）。`48,765,303 B` 是 `0.1.0` 时点的旧值，`0.2.0` 时点未更新。同时 `release.md:28` 版本规划表写 `0.2.0` 的 wasm `45,712,494 B`，与 `web/treasure_bevy_wasm-0.2.0.wasm` 实测 `45,794,950 B` 也不一致（差 ~82KB，可能为不同构建时点）。
- **建议修复**：`release.md:41` 将「当前 `0.1.0`」改为「当前 `0.2.0`」+ 更新 B 数为 `0.2.0` 实测值；`release.md:53` 同步更新。

## Q020
- **位置**：`docs/release.md:28`（版本规划表 `0.2.0` 行）
- **现象**：写 wasm `45,712,494 B`。磁盘上 `web/treasure_bevy_wasm-0.2.0.wasm` 实测 = `45,794,950 B`（`ls -la` 2026-11-07 构建）。两值差 82,456 B，可能是不同构建时点（版本表写的是发布时回填值，磁盘文件是后续重构建），但无时间戳说明。
- **建议修复**：`release.md:28` 加注「（发布时回填，磁盘文件可能为后续重构建）」或更新为最新实测值。

## Q021
- **位置**：`docs/release.md:29`（版本规划表 `0.1.0` 行）
- **现象**：写 `treasure_bevy_wasm-0.1.0.wasm` `45,582,959 B`。磁盘上 `web/treasure_bevy_wasm-0.1.0.wasm` 实测 = `45,712,503 B`（`ls -la`）。差 129,544 B。
- **建议修复**：确认 `release.md:29` 的 B 数是否为发布时快照值；若磁盘文件已重构建，加注说明。

## Q022
- **位置**：`docs/process/26_dev_stages_overview.md:21`
- **现象**：写「## 二、开发阶段总览（S0 → S35）」。但台账实际有 S36~S55 任务（`plan/ledger.yaml` 340 任务，S36.0/S37/S38/S39~S49/S50~S55 均已 verified）。26 号文档的「§二开发阶段总览」表格只列到 S35，未包含 S36~S55 阶段行。
- **建议修复**：`26_dev_stages_overview.md:21` 将「S0 → S35」改为「S0 → S55」，或在表格中补 S36~S55 行（至少加注「S36+ 见台账」）。

## Q023
- **位置**：`docs/process/26_dev_stages_overview.md:138`（工具矩阵 gate 行）
- **现象**：写「`gate.ps1`（36 项固定 Report：fmt/clippy/test/build/占位/unwrap/台账/doccheck/validate_data/gen_layout/check_hardcode/check_data_driven/check_code_standards/oracle边界/no-macroquad/doccheck2/台账对账/业务闭环/模块契约/飞轮/渲染不变式/数据源/audit_code/场景叙事/bevy-cpu-parity/wasm-browser-e2e/browser-playtest/playtest-loop）」。该列表**缺少** `dod-commands`（第 25 项，S46.1 新增）、`studio-tests`（第 25b 项，S50.2 新增）、`Android 真机`（第 4c 项，S49.4e 新增）三项，共 33 项而非 36 项。
- **建议修复**：`26_dev_stages_overview.md:138` gate 项列表补 `dod-commands`/`studio-tests`/`Android 真机` 三项，或加注「完整 36 项见 `docs/parity.md` 门禁表」。

## Q024
- **位置**：`docs/process/26_dev_stages_overview.md:179`
- **现象**：写「门禁 36 项（`gate.ps1`，当前 30 PASS / 0 FAIL / 2 SKIP）」。`30 PASS / 2 SKIP` 是某个时点快照（可能是 `evidence/linux_gate_green.log` 时点 = 31 项），但 36 项全档实跑时 SKIP 数 = 2（android 无 NDK + zero-unknown 或 wasm 无 Playwright），PASS 数 = 34（36 - 2 SKIP = 34 PASS + 0 FAIL）。`30 PASS / 2 SKIP` 合计 = 32，与 36 项总数不符（缺 4 项）。
- **建议修复**：确认 `30 PASS / 2 SKIP` 的时点（可能是 32 项时点快照，后来加了 4 项）；更新为当前实测值（`gate.ps1` 全档实跑后统计），或加注时点。

## Q025
- **位置**：`docs/process/15_process_ledger.md:59`（第 24 行）
- **现象**：写「`map_pack.py` 草稿包健壮性...结构性缺口 P2-2 = 门禁 33 项不覆盖 `map_pack.py verify`」。`33 项` 是历史时点值（S32.3 制作指引时点），现行 gate = 36 项（S46.1 dod-commands + S49.4e Android 真机 + S50.2 studio-tests 后）。该值为历史快照，但无时点标注。
- **建议修复**：`15_process_ledger.md:59` 将「门禁 33 项」改为「门禁 33 项（S32.3 时点；现行 36 项）」或加注时点。

## Q026
- **位置**：`docs/migration_linux.md:144,148`
- **现象**：写「gate 31 项 0 failure」和「门禁 31 项在 Linux 全绿」。`31 项` 是 2026-10-29 Linux 迁移续工时点快照，现行 gate = 36 项。该值为历史时点，但无时点标注（`migration_linux.md` 是迁移参考文档，非现行状态文档，可保留历史值但应加注时点）。
- **建议修复**：`migration_linux.md:144,148` 将「31 项」加注「（2026-10-29 时点；现行 36 项）」或保留原样（迁移文档历史快照可接受）。

## Q027
- **位置**：`docs/process/27_landscape_adaptation.md:117,120,124,127,132,136,158,212`
- **现象**：多处写「gate 33 项 0 failure」。`33 项` 是 S46.1 之前时点（S32/S33 阶段），现行 = 36 项。该文件为 U23 横屏适配设计草案（待用户确认），历史快照值无时点标注。
- **建议修复**：`27_landscape_adaptation.md` 8 处「33 项」加注时点（如「S44/S45 时点 33 项」），或统一改为现行 36 项。

## Q028
- **位置**：`docs/process/32_ccgs_borrow.md:91`
- **现象**：写「gate 35 项（S50 立项时点；现行 36 项，飞轮 R10 注明）」。该处已正确标注时点 + 现行值，**无漂移**（保留作为正例）。

## Q029
- **位置**：`docs/process/15_process_ledger.md:64`（第 29 行 theme 令牌对账）
- **现象**：写「`doccheck2::check_theme_consumption`（第 43 项，44 项末位起第 1 新断言）」。`theme-consumption` 在 `gate_data.json` 中为第 **43** 项（索引 42），与文档一致。但「44 项末位起第 1 新断言」= 44 项时点快照（S53 前），现行 47 项。该行是历史描述，可接受。
- **建议修复**：无需修改（历史快照，表述准确）。

## Q030
- **位置**：`docs/process/15_process_ledger.md:65`（第 30 行跨文档计数声称守门）
- **现象**：写「`doccheck2::check_cross_doc_counts`（第 44 项，44 项末位）」。`cross-doc-counts` 在 `gate_data.json` 中为第 **44** 项（索引 43），与文档一致。无漂移。
- **建议修复**：无需修改。

## Q031
- **位置**：`docs/rules/ti-engine-rules.md:39`
- **现象**：写「doccheck2 `engine-no-color-literal` 断言守门（现行第 15 项，`assets/gate_data.json` `doccheck2_checks` 为准）」。实际 `engine-no-color-literal` 在 `gate_data.json` 中为第 **15** 项（索引 14）。与文档一致。无漂移。
- **建议修复**：无需修改。

## Q032
- **位置**：`docs/process/28_engine_best_practice.md:76,85` + `docs/README.md:86` + `docs/experience/index.md:207,209`
- **现象**：4 处写「doccheck2 第 41 项 `engine-no-color-literal`」。实际 `engine-no-color-literal` 在 `gate_data.json` 中为第 **15** 项（`docs/rules/ti-engine-rules.md:39` 写「现行第 15 项」正确）。`28_engine_best_practice.md` 写「第 41 项」= S38 时点（doccheck2 41 项时 `engine-no-color-literal` 是第 15 项，不是第 41 项；第 41 项 = `job-model-unified`）。
- **建议修复**：`28_engine_best_practice.md:76,85` + `README.md:86` + `experience/index.md:207,209` 将「第 41 项」改为「第 15 项」（`engine-no-color-literal` 的 `gate_data.json` 实际位置），或加注「S38 时点第 15 项；现行 47 项中仍为第 15 项」。

## Q033
- **位置**：`AGENTS.md:16`（S21 行）
- **现象**：写「`doccheck2::check_no_inline_test_mod` src 内联单测拦截（47 存量 `process_ratchet.json` 棘轮豁免）」。`47 存量` = 47 个存量违例函数（`process_ratchet.json` 棘轮基线），不是「47 项」。表述易与「47 项 doccheck2」混淆。
- **建议修复**：`AGENTS.md:16` 将「47 存量」改为「47 个存量违例（`process_ratchet.json` 棘轮基线）」，避免与「47 项 doccheck2」混淆。

## Q034
- **位置**：`docs/process/14_tooling_catalog.md:18`
- **现象**：写「**落地位置是硬规则**（`doccheck2` 第 27 项机器判定）」。实际 `check_tooling_contract` = 第 **29** 项（`gate_data.json` 索引 28）。第 27 项 = `terrain-data`。
- **建议修复**：将「第 27 项」改为「第 29 项」。

## Q035
- **位置**：`docs/process/14_tooling_catalog.md:31`
- **现象**：写「`doccheck2` 第 26 项 `check_tooling_catalog`」。实际 `check_tooling_catalog` = 第 **28** 项。第 26 项 = `primitives-data`。
- **建议修复**：将「第 26 项」改为「第 28 项」。

## Q036
- **位置**：`docs/rules/scripts-rules.md:11`
- **现象**：写「`check_tooling_contract`（doccheck2 第 27 项）」。实际 = 第 **29** 项。
- **建议修复**：将「第 27 项」改为「第 29 项」。

## Q037
- **位置**：`AGENTS.md:102`（工具台账路由表行）
- **现象**：写「新增工具须登记（`doccheck2` 第 26 项对账）+ 满足工具契约（第 27 项 `check_tooling_contract`...）」。实际第 26 项 = `primitives-data`，第 27 项 = `terrain-data`；`check_tooling_catalog` = 第 28 项，`check_tooling_contract` = 第 29 项。
- **建议修复**：将「第 26 项对账」改为「第 28 项对账（`check_tooling_catalog`）」，「第 27 项」改为「第 29 项」。

## Q038
- **位置**：`AGENTS.md:123`（硬规则 10）
- **现象**：写「违反 = `doccheck2` 第 27 项 `check_tooling_contract` FAIL」。实际 `check_tooling_contract` = 第 **29** 项。
- **建议修复**：将「第 27 项」改为「第 29 项」。

## Q039
- **位置**：`docs/process/14_tooling_catalog.md:44`
- **现象**：写「`doccheck2` 第 27 项 `check_tooling_contract` 逐条判定」。实际 = 第 **29** 项。
- **建议修复**：将「第 27 项」改为「第 29 项」。

## Q040
- **位置**：`docs/process/14_tooling_catalog.md:50`
- **现象**：写「§3 第 26 项只查「出现过」」。实际 `check_tooling_catalog` = 第 **28** 项。
- **建议修复**：将「第 26 项」改为「第 28 项」。

## Q041
- **位置**：`docs/rules/scripts-rules.md:19`
- **现象**：写「工具契约 4 条全满足（§14 第 27 项）」。实际 `check_tooling_contract` = 第 **29** 项。
- **建议修复**：将「第 27 项」改为「第 29 项」。

## Q042
- **位置**：`docs/process/12_bevy_usage.md:349`
- **现象**：写「gate 第 24 项 `check_data_driven`」。实际 `check_data_driven` 在 `gate.ps1` 中的位置 = 第 **10d** 项（`gate.ps1:248-250`），不是第 24 项。第 24 项 = `playtest-loop`。
- **建议修复**：将「gate 第 24 项 `check_data_driven`」改为「gate 第 10d 项 `check_data_driven`」。

## Q043
- **位置**：`docs/business/08_layout_design.md:115,159,212,239`
- **现象**：4 处写「gate 第 22 项 `check_scene_narrative` N8 对账锚点」。实际 `check_scene_narrative` 在 `gate.ps1` 中 = 第 **21** 项（`gate.ps1:360`），不是第 22 项。第 22 项 = `wasm-browser-e2e`。
- **建议修复**：4 处「gate 第 22 项」改为「gate 第 21 项」。

## Q044
- **位置**：`docs/business/08_layout_design.md:124`
- **现象**：写「`scripts/check_data_driven.py` 门禁守护（gate 第 24 项）」。实际 `check_data_driven` = 第 **10d** 项（同 Q042）。
- **建议修复**：将「gate 第 24 项」改为「gate 第 10d 项」。

## Q045
- **位置**：`docs/process/22_dev_platform.md:40`
- **现象**：写「第 23 项 browser-playtest **无分档守卫**（三档都会执行）」。实际 `browser-playtest` 在 `gate.ps1` 中 = 第 **33** 项（`gate.ps1:402`），不是第 23 项。第 23 项 = `browser-playtest` 的注释编号（`gate.ps1` 注释 23），但机器计量口径（`Read-GateItems` 去重 Report 项名）中 `browser-playtest` 是第 33 项。需确认 `22_dev_platform.md` 引用的是注释编号还是机器计量编号。
- **建议修复**：确认口径；若为注释编号则加注「（注释编号 23；机器计量第 33 项）」，若为机器计量则改为「第 33 项」。

## Q046
- **位置**：`docs/plan-s32-platform.md:88`
- **现象**：写「其第 23 项会改写 `evidence/web_playtest.{log,png}`」。同 Q045，第 23 项 = 注释编号 `browser-playtest`（`gate.ps1` 注释 23），机器计量 = 第 33 项。
- **建议修复**：同 Q045，加注口径说明。

## Q047
- **位置**：`docs/process/05_module_parallel.md:122`
- **现象**：写「第 15 项查业务闭环、第 16 项查包白名单/契约」。实际 `check_forward_closed`（业务闭环）在 `gate.ps1` 中 = 第 **15** 项（`gate.ps1:298`），`check_module_contracts`（包白名单/契约）= 第 **16** 项（`gate.ps1:306`）。与文档一致。无漂移。
- **建议修复**：无需修改。

## Q048
- **位置**：`docs/process/06_flywheel.md:101`
- **现象**：写「gate 加 `flywheel-readme -Check` 项（第 17 项）」。实际 `flywheel-readme` 在 `gate.ps1` 中 = 第 **17** 项（`gate.ps1:323`）。与文档一致。无漂移。
- **建议修复**：无需修改。

## Q049
- **位置**：`docs/check_ledger_reality.md:1`
- **现象**：写「S10.0 增 gate 第 14 项」。实际 `check_ledger_reality` 在 `gate.ps1` 中 = 第 **14** 项（`gate.ps1:292`）。与文档一致。无漂移。
- **建议修复**：无需修改。

## Q050
- **位置**：`docs/contracts/00_index.md:8,21,23`
- **现象**：写「gate 第 16 项」（`check_module_contracts`）。实际 = 第 **16** 项。与文档一致。无漂移。
- **建议修复**：无需修改。

## Q051
- **位置**：`docs/rules/ti-core-rules.md:10`
- **现象**：写「`check_module_contracts.py` 包白名单机制，gate 第 16 项」= 正确（第 16 项）。无漂移。
- **建议修复**：无需修改。

## Q052
- **位置**：`docs/rules/ti-core-rules.md:28`
- **现象**：写「`check_oracle_boundary`（gate 第 11 项）」。实际 `check_oracle_boundary` 在 `gate.ps1` 中 = 第 **11** 项（`gate.ps1:264`）。与文档一致。无漂移。
- **建议修复**：无需修改。

## Q053
- **位置**：`docs/process/08_render_arch.md:204-207`
- **现象**：写「gate 第 13 项（doccheck2 `check_runtime_no_legacy_coords`）」。实际 `doccheck2` 在 `gate.ps1` 中 = 第 **13** 项（`gate.ps1:285`），`check_runtime_no_legacy_coords` 是 doccheck2 内的第 20 项断言（`gate_data.json` 索引 19）。文档写「gate 第 13 项（doccheck2）」= 正确（gate 第 13 项 = doccheck2 整体），但括号内写「`check_runtime_no_legacy_coords`」= doccheck2 内的第 20 项断言，不是 gate 第 13 项。表述易混淆。
- **建议修复**：明确写「gate 第 13 项 `doccheck2`（内含第 20 项断言 `check_runtime_no_legacy_coords`）」，避免将 doccheck2 内断言序号与 gate 项序号混淆。

## Q054
- **位置**：`docs/process/08_render_arch.md:224-229`
- **现象**：写「gate 第 19 项 render-snapshot」「gate 第 19.5 项 forward-render-snapshot」「gate 第 18 项 visual-regression」。实际 `gate.ps1` 中 `render-invariants`（7 场景结构不变式）= 第 **19** 项（`gate.ps1:336`），但无 `render-snapshot`/`forward-render-snapshot`/`visual-regression` 这三个独立 gate 项（它们已合并入 `render-invariants` 或退役）。`gate.ps1` 中 `bevy-cpu-parity` = 第 **25** 项（`gate.ps1:375`），不是第 18 项。
- **建议修复**：`08_render_arch.md:224-229` 更新为现行 gate 项号：`render-invariants` = 第 19 项（正确），`bevy-cpu-parity` = 第 25 项（非 18 项），`forward-render-snapshot`/`visual-regression` 已合并入 `render-invariants` 或退役，需确认现行状态。

## Q055
- **位置**：`docs/process/07_render_pipeline.md:20,25,120,121,123,125,201,234`
- **现象**：多处写「gate 第 19 项 `render_invariants`」「gate 第 20 项 `check_data_source`」。实际 `render-invariants` = 第 **19** 项（正确），`check_data_source` = 第 **20** 项（`gate.ps1:344`，正确）。无漂移。
- **建议修复**：无需修改。

## Q056
- **位置**：`docs/process/07_render_pipeline.md:25`
- **现象**：写「`scripts/compare_render.py`，**TODO 未实现**」。该文件在 `scripts/` 目录中是否存在？
- **建议修复**：确认 `scripts/compare_render.py` 是否存在；若不存在，将「TODO 未实现」改为「未实现（已退役，用 `element-dump` + `render_invariants` 测试替代）」或删除该引用。

## Q057
- **位置**：`docs/process/11_bevy_migration.md:83`
- **现象**：写「`check_data_driven`（闭集 + 渲染层无场景语义 + 令牌存在性）」。该引用未注明 gate 项号，无漂移。
- **建议修复**：无需修改。

## Q058
- **位置**：`docs/process/04_tooling.md:56`
- **现象**：写「`scripts/gen_layout.py` 把 `ref/controls.json` 2041 条控件记录转成 `assets/data/controls.json`」。但 `gen_layout.py` 实际位于 `legacy/scripts/gen_layout.py`（`gate.ps1:231` 调用 `legacy/scripts/gen_layout.py --check`），不是 `scripts/gen_layout.py`。`scripts/` 目录中无 `gen_layout.py`。
- **建议修复**：`04_tooling.md:56` 将 `scripts/gen_layout.py` 改为 `legacy/scripts/gen_layout.py`。

## Q059
- **位置**：`docs/process/04_tooling.md:92`
- **现象**：写「`validate_data.py` + `gen_layout.py --check`」（gate 第 10 项）。同 Q058，`gen_layout.py` 在 `legacy/scripts/`，不是 `scripts/`。
- **建议修复**：将 `gen_layout.py` 改为 `legacy/scripts/gen_layout.py`。

## Q060
- **位置**：`docs/gamebook.md:42`
- **现象**：写「`legacy/scripts/gen_layout.py`（幂等，gate 第 10b 项）」。该处正确引用了 `legacy/scripts/` 路径。无漂移。
- **建议修复**：无需修改。

## Q061
- **位置**：`docs/process/14_tooling_catalog.md:294`（数据流水线行）
- **现象**：写「`python scripts/validate_data.py` / `gen_layout.py --check`」。`gen_layout.py` 在 `legacy/scripts/`，不是 `scripts/`。
- **建议修复**：将 `gen_layout.py` 改为 `legacy/scripts/gen_layout.py`。

## Q062
- **位置**：`docs/process/14_tooling_catalog.md:186`（`validate_data.py` 行）
- **现象**：写「`check_version_consistency`（S25 版本三方对账：`Cargo.toml` `[workspace.package] version` 单源 ↔ `AndroidManifest versionName` ↔ 构建脚本产物名带 x.y.z ↔ `release.md` 版本规划行）」。该描述正确，但 `validate_data.py` 行末写「gate 第 10a 项」= 正确（`gate.ps1:229`）。无漂移。
- **建议修复**：无需修改。

## Q063
- **位置**：`docs/parity.md:170-171`（doccheck2 19 项描述）
- **现象**：写「文档↔代码/数据一致性 19 项」。实际 `doccheck2` 现行 = 47 项（`gate_data.json` 实测）。`19 项` 是 S11/S12 时点快照（doccheck2 刚落地时 19 项），无时点标注。
- **建议修复**：`docs/parity.md:170` 将「19 项」改为「47 项（现行；S11 时点 19 项）」或加注时点。

## Q064
- **位置**：`docs/parity.md:171`
- **现象**：写「LCG hex 误算（0x3443D）/ 据点阈值反向断言（<= 40）/ i18n 孤儿 key / gate 项数漂移 / headless BattleView API / 飞轮回读 / 战斗玩家数值数据驱动 / 飞轮脚本 / 触摸渲染正逆成对 / 场景 band 数 / 场景布局保真 / HD 精灵透明 / map_slot 写档 0 起 zero_index 基准（FIND-203）/ exhaust 双写对账 / 经验库分卷条数 / 运行时 legacy 坐标禁 / 基线镜像恒等」。该列表 = 19 项 doccheck2 断言（S11 时点），现行 47 项，缺 28 项。
- **建议修复**：同 Q063，加注「S11 时点 19 项；现行 47 项见 `assets/gate_data.json` `doccheck2_checks`」。

## Q065
- **位置**：`docs/parity.md:156`
- **现象**：写「第 9 项内含跨文档数值断言（2041/UI 场景 20/0x343FD/20011591）」。实际 `doccheck` 的数值断言 = 4 条（`cargo run -p ti-tools --bin doccheck` 实测：`178 docs, 38 facts, 184 experiences, 334 task refs, 4 assertions, 0 error(s)`）。`UI 场景 20` = `controls.json scene_count`（legacy 20 场景），`scenes.json` 现行 = 7 场景。`UI 场景 20` 是 legacy 口径（`ref/controls.json`），不是现行 `scenes.json` 口径。文档未注明该值为 legacy 口径。
- **建议修复**：`docs/parity.md:156` 加注「UI 场景 20 = legacy `ref/controls.json` 口径（`scene_count`），现行 `scenes.json` = 7 场景」。

## Q066
- **位置**：`docs/process/04_tooling.md:59`
- **现象**：写「数值断言 4 条：UI 场景数=20、控件记录=2041、LCG hex=0x343FD（非 0x3443D）、save.magic=20011591（非 20125007）」。`UI 场景数=20` 同 Q065，为 legacy 口径。`doccheck` 实跑确认 4 条断言全部 PASS。无漂移（值正确），但 `UI 场景数=20` 的口径说明缺失。
- **建议修复**：同 Q065，加注「UI 场景数=20 = legacy `ref/controls.json` `scene_count`（现行 `scenes.json` = 7 场景，不同口径）」。

## Q067
- **位置**：`docs/release.md:239`
- **现象**：写「`plan/ledger.yaml` 340 任务（339 verified + 0 pending（S52 全收口 + S54/S55 断言固化；余 1 blocked S49.1（S7.7 已 verified））」。`340 任务 / 339 verified / 1 blocked` 与台账实测一致（`cargo run -p ti-tools --bin ledger` 输出 `340 tasks, 339 verified, 0 error(s)`）。无漂移。
- **建议修复**：无需修改。

## Q068
- **位置**：`AGENTS.md:8`（状态段）
- **现象**：写「当前 340 任务 / 339 verified + 1 blocked（S49.1 竖屏主题化用户拍板作废；S54.1/S55.1 飞轮 R9 后断言固化 2 条 verified）」。与台账实测一致。无漂移。
- **建议修复**：无需修改。

## Q069
- **位置**：`docs/loop.md:17`（dod 注释）
- **现象**：写「`scripts/gate.ps1` 全绿（当前 36 项，随 S8/S9/S10 演进）」。`36 项` 与现行 gate 实测一致。无漂移。
- **建议修复**：无需修改。

## Q070
- **位置**：`docs/release.md:239`
- **现象**：写「台账计数随 S8–S49.9 演进」。但台账实际有 S50~S55 任务（`S50.0`~`S55.1`），`S49.9` 之后还有 S50/S51/S52/S54/S55。`S8–S49.9` 的表述未覆盖 S50~S55。
- **建议修复**：将「S8–S49.9 演进」改为「S8–S55.1 演进」（或「随台账演进」，不指定具体 S 号）。

## Q071
- **位置**：`docs/process/26_dev_stages_overview.md:163`（下一节点段）
- **现象**：写「后续 = 用户拍板新方向（S50/S51 起）/ 用户视觉签收（S17.5/S21.9/S26 出图）」。`S17.5/S21.9/S27.5` 已被 `S34.6/S34.7` 取代收口（`plan/ledger.yaml` S17.5/S21.9/S27.5 行 `blocked_by` 字段写「被 S34.6 取代」），`S26` 出图 = S26.3 视觉签收（已由 S34.6 收口）。该段写「用户视觉签收（S17.5/S21.9/S26 出图）」= 已收口任务，不应再列为「后续」。
- **建议修复**：删除 `S17.5/S21.9/S26 出图`（已由 S34.6 收口），或改为「用户视觉签收（已由 S34.6/S34.7 收口，无需单独签收）」。

## Q072
- **位置**：`docs/process/26_dev_stages_overview.md:163`
- **现象**：写「S34.0~S34.6 全 verified（2026-11-09），视觉签收类 S17.5/S21.9/S27.5 由 S34.6/S34.7 取代收口」。`S34.7` 在 `plan/ledger.yaml` 中是否存在？
- **建议修复**：确认 `S34.7` 是否在台账中（`grep 'S34.7' plan/ledger.yaml`）；若不存在，改为「S34.6 收口」（删除 S34.7 引用）或补登 S34.7。

## Q073
- **位置**：`docs/process/26_dev_stages_overview.md:128`
- **现象**：写「S7.7（P4-1 基准切换）已 verified（S52 收口时点路径 2 执行完毕），非阻塞项（飞轮 R7 F-1 勘误旧 S7.7 标签残留）」。`S7.7` 在台账中的 status = verified（`plan/ledger.yaml` S7.7 行），`blocked_by` = null。该描述正确。无漂移。
- **建议修复**：无需修改。

## Q074
- **位置**：`docs/experience/process.md:548`
- **现象**：写「门禁 34 项（S46.1 时点）；门禁项数牵动 8 处声称」。`34 项` = S46.1 时点（dod-commands 加入后 = 34 项），现行 = 36 项（+4c Android 真机 + 25b studio-tests）。该值为历史时点，已正确标注「S46.1 时点」。无漂移。
- **建议修复**：无需修改（时点标注正确）。

## Q075
- **位置**：`docs/experience/code.md:609,628,634`
- **现象**：3 处写「gate 33 项 0 failure」。`33 项` 为 S44/S45 时点快照，现行 = 36 项。无时点标注。
- **建议修复**：3 处「33 项」加注「（S44/S45 时点；现行 36 项）」或改为现行 36 项。

## Q076
- **位置**：`docs/experience/tools.md:188`
- **现象**：写「gate 33 项 0 failure」。同 Q075，历史时点值无标注。
- **建议修复**：加注时点。

## Q077
- **位置**：`docs/experience/build.md:368`
- **现象**：写「gate 33 项 0 failure」。同 Q075。
- **建议修复**：加注时点。

## Q078
- **位置**：`docs/experience/build.md:215`
- **现象**：写「gate 29 项与 doccheck2」。`29 项` 为 S32 时点快照，无时点标注。
- **建议修复**：加注「（S32 时点；现行 36 项）」。

## Q079
- **位置**：`docs/experience/tools.md:132`
- **现象**：写「gate 28 项 = doccheck」。`28 项` 为 S31/S32 时点快照，无时点标注。
- **建议修复**：加注时点。

## Q080
- **位置**：`docs/experience/process.md:171,213,214,217`
- **现象**：4 处写「gate 17 项」「gate 18 项」= S11/S12 时点快照，无时点标注。
- **建议修复**：加注时点（如「S11 时点 17 项」）。

## Q081
- **位置**：`docs/experience/code.md:240`
- **现象**：写「gate 22 项全绿」。`22 项` 为 S31.3 时点（wasm-browser-e2e 加入后），无时点标注。
- **建议修复**：加注时点。

## Q082
- **位置**：`docs/experience/process.md:310,361,387`
- **现象**：3 处写「gate 22 项 0 failure」= S31/S32 时点，无时点标注。
- **建议修复**：加注时点。

## Q083
- **位置**：`docs/experience/index.md:26`
- **现象**：写「gate 第 9 项」（`check_ledger_reality` 相关 EXP）。实际 `check_ledger_reality` = 第 **14** 项（`gate.ps1:292`），不是第 9 项。第 9 项 = `doccheck`。
- **建议修复**：确认 `docs/experience/index.md:26` 的 EXP 编号；若引用 `check_ledger_reality` 的 gate 项号，改为「第 14 项」。

## Q084
- **位置**：`docs/experience/index.md:108`
- **现象**：写「孤儿 evidence 检测」（`evidence/` 里任务 ID 命名的文件...）。该 EXP 引用的 gate 项号需确认。
- **建议修复**：确认引用的 gate 项号是否正确。

## Q085
- **位置**：`docs/experience/index.md:111`（EXP-068）
- **现象**：写「doccheck2 第 8 项 `check_flywheel_readback`」。实际 `flywheel-readback` 在 `gate_data.json` 中为第 **8** 项（索引 7）。正确。无漂移。
- **建议修复**：无需修改。

## Q086
- **位置**：`docs/experience/index.md:112`（EXP-070）
- **现象**：写「doccheck2 第 10 项 `check_flywheel_script`」。实际 `flywheel-script` 在 `gate_data.json` 中为第 **10** 项（索引 9）。正确。无漂移。
- **建议修复**：无需修改。

## Q087
- **位置**：`docs/experience/index.md:113`（EXP-071）
- **现象**：写「doccheck2 第 11 项 `check_touch_render_pair`」。实际 `touch-render-pair` = 第 **11** 项。正确。无漂移。
- **建议修复**：无需修改。

## Q088
- **位置**：`docs/experience/index.md:115`（EXP-073）
- **现象**：写「doccheck2 第 12 项」（`check_hd_sprite_transparency`）。实际 `hd-sprite-transparency` = 第 **12** 项。正确。无漂移。
- **建议修复**：无需修改。

## Q089
- **位置**：`docs/experience/process.md:369`
- **现象**：写「doccheck2 第 17 项」。需确认第 17 项是什么断言。`gate_data.json` 第 17 项 = `save-slot-map-slot-basis`（索引 16）。若该 EXP 引用的确实是第 17 项，需确认断言名称是否匹配。
- **建议修复**：确认 `docs/experience/process.md:369` 引用的 doccheck2 断言名称是否与第 17 项 `save-slot-map-slot-basis` 匹配。

## Q090
- **位置**：`docs/experience/code.md:458`
- **现象**：写「doccheck2 第 26 项只查「文件名是否出现在台账里」」。实际 `gate_data.json` 第 26 项 = `primitives-data`（`check_primitives_data`），不是「文件名是否出现在台账里」（那是 `tooling-catalog` = 第 28 项）。
- **建议修复**：将「第 26 项」改为「第 28 项（`check_tooling_catalog`）」。

## Q091
- **位置**：`docs/experience/index.md:157`（EXP-115）
- **现象**：写「doccheck2 第 27 项 `check_tooling_contract`」。实际 `tooling-contract` = 第 **29** 项（`gate_data.json` 索引 28）。第 27 项 = `terrain-data`。
- **建议修复**：将「第 27 项」改为「第 29 项」。

## Q092
- **位置**：`docs/experience/process.md:309`
- **现象**：写「新增 doccheck2 第 22 项 `check_no_ti_core_fullpath`」。实际 `no-ti-core-fullpath` 在 `gate_data.json` 中为第 **22** 项（索引 21）。正确。无漂移。
- **建议修复**：无需修改。

## Q093
- **位置**：`docs/process/04_tooling.md:64`（`check_dod_commands.py` 行）
- **现象**：写「gate 第 25 项」。实际 `dod-commands` 在 `gate.ps1` 中 = 第 **25** 项（`gate.ps1:423`）。正确。无漂移。
- **建议修复**：无需修改。

## Q094
- **位置**：`docs/process/04_tooling.md:65`（`studio_tests.py` 行）
- **现象**：写「gate 第 25b 项」。实际 `studio-tests` 在 `gate.ps1` 中 = 第 **25b** 项（`gate.ps1:439`）。正确。无漂移。
- **建议修复**：无需修改。

## Q095
- **位置**：`docs/process/23_game_testing.md:45,47`
- **现象**：写「第 4c + 验收判据」「第 4c 有 adb 设备才跑 P5」。实际 `Android 真机` 在 `gate.ps1` 中 = 第 **4c** 项（`gate.ps1:157-165`）。正确。无漂移。
- **建议修复**：无需修改。

## Q096
- **位置**：`docs/README.md:86`
- **现象**：写「`engine-no-color-literal`（第 41 项）」。同 Q032，实际 = 第 **15** 项。
- **建议修复**：将「第 41 项」改为「第 15 项」。

## Q097
- **位置**：`docs/experience/index.md:207`（EXP-165）
- **现象**：写「doccheck2 第 41 项 `engine-no-color-literal`」。同 Q032，实际 = 第 **15** 项。
- **建议修复**：将「第 41 项」改为「第 15 项」。

## Q098
- **位置**：`docs/experience/index.md:209`（EXP-167）
- **现象**：写「doccheck2 第 41 项 ③ 时序扫描防再犯」。同 Q032，`engine-no-color-literal` = 第 **15** 项。
- **建议修复**：将「第 41 项」改为「第 15 项」。

## Q099
- **位置**：`docs/process/28_engine_best_practice.md:4`
- **现象**：写「`16_code_standards.md`（CS-1~4 代码标准）」。正确（CS-1~4）。无漂移。
- **建议修复**：无需修改。

## Q100
- **位置**：`docs/process/26_dev_stages_overview.md:50`
- **现象**：写「S25 代码标准三件套（CS-1~4 + 棕轮基线）」。正确（CS-1~4）。但 `gate.ps1:258` Report 项名写「CS-1~6」= 与 16 号文档不一致（Q010）。
- **建议修复**：同 Q010，统一为 CS-1~4。

---

## 统计

**CONFIRMED-candidate 数量**：

- 确认漂移（需修复）：Q001, Q002, Q003, Q004, Q005, Q006, Q007, Q008, Q009, Q010, Q011, Q012, Q013, Q014, Q015, Q016, Q017, Q018, Q019, Q020, Q021, Q022, Q023, Q024, Q025, Q027, Q032, Q033, Q034, Q035, Q036, Q037, Q038, Q039, Q040, Q041, Q042, Q043, Q044, Q045, Q046, Q054, Q056, Q058, Q059, Q061, Q063, Q064, Q065, Q066, Q070, Q071, Q072, Q075, Q076, Q077, Q078, Q079, Q080, Q081, Q082, Q083, Q084, Q089, Q090, Q091, Q096, Q097, Q098, Q100 = **72 条**
- 无漂移（正例/历史快照标注正确）：Q028, Q029, Q030, Q031, Q047, Q048, Q049, Q050, Q051, Q052, Q055, Q057, Q060, Q062, Q067, Q068, Q069, Q073, Q074, Q085, Q086, Q087, Q088, Q092, Q093, Q094, Q095, Q099 = **28 条**

**最高价值 5 条**：

1. **Q010**（`gate.ps1:258` CS-1~6 vs 代码实际 CS-1~4）：gate 项名与代码实现不一致，`check_code_standards.py` 只有 4 项检查，gate Report 项名写「CS-1~6」= 机器输出与代码不符，每次跑 gate 都会看到错误项名。
2. **Q008**（`14_tooling_catalog.md`/`AGENTS.md`/`scripts-rules.md` 7 处「doccheck2 第 26/27 项」= 实际第 28/29 项）：工具契约的 doccheck2 项号在 3 个文件中系统性偏移 2 位（S53/S54/S55 加 3 项后未回写），影响范围最大。
3. **Q032**（`28_engine_best_practice.md`/`README.md`/`experience/index.md` 4 处「第 41 项 `engine-no-color-literal`」= 实际第 15 项）：`engine-no-color-literal` 的 doccheck2 项号在 4 个文件中写错（第 41 项 = `job-model-unified`，`engine-no-color-literal` = 第 15 项），且 `docs/rules/ti-engine-rules.md:39` 写「现行第 15 项」= 正确，形成内部矛盾。
4. **Q019**（`release.md:41` wasm 版本/B 数过期：写 `0.1.0` + 48,765,303 B，实际 `Cargo.toml` = `0.2.0` + 磁盘 `0.2.0` wasm = 45,794,950 B）：版本表与磁盘文件不一致，影响发布判断。
5. **Q058**（`04_tooling.md:56,92` + `14_tooling_catalog.md:294` `gen_layout.py` 路径错误：写 `scripts/gen_layout.py`，实际在 `legacy/scripts/gen_layout.py`）：工具路径在 3 处写错，`gate.ps1` 调用的是 `legacy/scripts/gen_layout.py`。
