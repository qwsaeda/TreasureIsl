# AGENTS.md — 路由表

项目：Windows Mobile 游戏《金银岛 TreasureIsl》(ARM 原生) 的**逆向 + Rust 现代化移植**，目标 WASM(浏览器) + Android 双端。
背景计划不在此处，按任务路由读文档，**不要凭默认习惯开工**。

## 当前状态

- 根目录：`TreasureIsl.exe`（ARM PE32）、`TreasureIsl.exe.i64`（IDA Professional 9.4，已自动分析、**无自定义命名**）、`legacy/`（逆向产物归档：`exe`/`i64`/`data`/`ref`/`docs`）、`Cargo.toml`（6 workspace member = 6 开发 crate）、`scripts/`、`docs/`、`android/`、`plan/ledger.yaml`、`evidence/`、`ref/`。
- 工程已 bootstrap：`crates/{ti-formats,ti-core,ti-platform,ti-app,ti-tools}`；进度只由 `plan/ledger.yaml` 驱动。
- **S15 业务场景穷举测试阶段**：S0–S14 全 verified（含 S9.1–S9.5 全 verified、S10.0–S10.5 全 verified、S11.1–S11.4 全 verified、S12.0–S12.8 全 verified、S13.1–S13.6 全 verified、S14.0 verified）；S15.1 verified，S15.1 verified、**S15.2 verified**（跨图推进规则定稿 + 落地，S15.2 残留 2 项已闭环 = 通关门禁 `is_cleared` 逐图 + 存档 `unlocked_mask` 逐图写入，见 `evidence/S15.2.log [8]`）。S16（试玩闭环 + 发布 + 设计 sheet）待开工（`docs/plan-s16-loop.md`，WBS S16.1 起）。
  逆向产物已物理归档 `legacy/`（S11.1 verified）；oracle 参照实现唯一源 = `ti-core/src/oracle`（`pub mod oracle` 公开导出，测试直接 `use ti_core::oracle::`）；逐字节重复 crate `legacy/oracle-rust` 已于 S15.8 删除（C4 反向守卫防重引入）。
  模块并行开发 loop 已固化（`docs/process/05_module_parallel.md`）；飞轮流水线已落地（`docs/process/06_flywheel.md` + gate 第 17 项）。

## 路由表

> 全库索引见 `docs/README.md`。本表只列「开工前必读」的入口。

| 你要做的事 | 读 |
|---|---|
| **理解游戏怎么设计/架构/模块化（第一入口）** | `docs/00_design.md`（正向设计总览：核心循环 + 数据分层 + crate 架构 + 9 场景 + 模块包 + 窄接口；**不读 legacy/**） |
| **新开工 / 参与开发（先读这个）** | `docs/process/00_index.md`（分工表：谁做什么 + 标准工作流）+ `docs/business/00_index.md`（业务唯一事实源） |
| **任何开工动作之前** | `docs/loop.md` + `plan/ledger.yaml` + `docs/README.md` |
| 判断「做完了没有」/ 验收 | `docs/parity.md`（P0~P4 判据 + **门禁项数/口径唯一事实源**） |
| 写 Rust（分层/数据驱动/现成库） | `docs/process/01_architecture.md` + `docs/process/02_conventions.md`（新）；语法层细节 `docs/style.md`（**标准库 trait 选用矩阵 + 重构操作规程 = §5.6.6**：From/Into/TryFrom/Display/Default/AsRef/include_str! 等何时用 + 手写转换补 trait 的五步规程） |
| 套用模版 / 写新场景·控件·模块 | `docs/process/03_templates.md`（§7 = 模块包任务书模版） |
| 用脚本/codegen 批量处理 | `docs/process/04_tooling.md` |
| **查业务规则（战斗/经济/地图/存档/UI 流）** | `docs/business/` 各系统文档（不依赖逆向产物） |
| **界面版面 / 配色 / 控件坐标（1080 单一事实源）** | `docs/business/08_layout_design.md`（改 scenes.json 坐标/配色前必读；表值 = scenes.json 同值） |
| **模块并行开发（契约→并行→集成）** | `docs/process/05_module_parallel.md`（一轮 4 步 + 纪律 5 条）+ `docs/contracts/00_index.md` + `plan/packages.yaml`（**包划分的机读唯一源**） |
| **对抗流水线飞轮（找问题→审计→修复→回写；开轮编排 = `flywheel-kickoff` skill）** | `docs/process/06_flywheel.md`（3 角色 + 开轮必读 6 条 + 停机条件）+ `scripts/flywheel_readme.ps1 -Check` |
| **界面全量重设计 S17（设计令牌 v2 / 8pt 版面 / 图标 14 项 / 动效 4 参数，航海夜·精修）** | `docs/process/10_ui_redesign.md`（S17 规格单一事实源；改 UI 视觉量前必读，与 `08_layout_design.md` 场景坐标实例互锁） |
| **UI 重设计 v3（真游戏化版面：场景叙事 DSL + 打破 7 场景同模板）** | `docs/ui-redesign-v3/README.md`（01 对标拆解 / 02 场景叙事 DSL 规格 / 03 七场景重设计提案 / 04 index 落地；gate 顶层固定第 23 项（gate.ps1 内注释编号 21）`check_scene_narrative` 锚点 `assets/data/ui_schema/scene_manifest_v3.json`；v3 取代 v2 的「7 场景同模板」前提，色板/字体/动效令牌仍沿用 `10_ui_redesign.md`） |
| **改渲染代码（sprite_atlas/terrain_atlas/game_app render/越界/透明/z-order）前必读** | `docs/process/08_render_arch.md`（**运行时架构契约**：分层 / 绘制原语 / z-order=调用顺序 / 单一 1080 坐标系 / grep 门禁；gate 第 9 项 doccheck2 `check_runtime_no_legacy_coords` 锚点）<br>`docs/process/07_render_pipeline.md`（legacy 像素管线契约，已降级为逆向参照） |
| **改 `crates/ti-render/`（渲染框架 crate）代码前必读** | `docs/process/09_render_framework.md`（**ti-render crate 边界契约**：职责与禁止项 / 模块结构 / Theme schema / 迁移映射 / 数据分离原则；注意本 crate **无 z-order 枚举**，层序在 `chrome::scene_shell` 调用序） |
| **S18 渲染引擎迁移（Bevy 0.20）· 改引擎层/加 bevy 依赖前必读** | `docs/process/11_bevy_migration.md`（**迁移裁决单一事实源**：差距定稿 / 定案推翻签核（macroquad 定案 + 800KB wasm 预算）/ 新预算 / 架构形状（新 crate `ti-engine` + Canvas 后端替换 + 双渲染期）/ WBS S18.0–S18.8 / 风险） |
| 可玩实现手册（已验证 rules 接进交互壳的接线规格） | `docs/gamebook.md` |
| 当前可玩范围 / 平台产物 / 已知限制 / 如何运行 | `docs/release.md`（纯 UTF-8，编辑 CJK 内容时勿用 `Set-Content`，见 EXP-051/091） |
| 防虚报：台账 verified ↔ evidence ↔ 代码符号三方对账 | `docs/check_ledger_reality.md`（EXP-058；gate 第 14 项） |
| S10 现代化收口总计划（P0 修复/占位清零/GPU 渲染） | `docs/plan-s10-refactor.md`（历史阶段文档） |
| S11 正向开发收口（逆向归档 legacy/ + 业务闭环） | `docs/plan-s11-forward.md`（历史阶段文档，§7 交接快照） |
| S12 模块化并行 WBS | `docs/plan-s12-parallel.md` |
| S16 试玩闭环 + 发布 + 设计 sheet（WBS，S15.2 后开工） | `docs/plan-s16-loop.md`（S16.1 浏览器真运行时试玩 web_playtest / S16.2 试玩 agent 闭环 loop / S16.3 一键发布 publish-web / S16.4 内容设计 sheet 模版；对标 OhMyGame 只吸收概念） |
| 逆向 exe 或 `data/` 格式（**仅参照物**，不作为验收基准） | `legacy/docs/facts/index.md` |
| 历史阶段计划（已完成阶段，查背景用） | `docs/plan-archive.md`（S0~S8 逐阶段 1~2 行摘要 + 原文件位置） |
| **游戏开发入门教程** | `docs/tutorial/README.md`（10 章 + 进度追踪，紧密绑定项目代码） |
| 避免重复踩坑 | `docs/experience/index.md` ← **每次踩坑后必须回写** |
| 容器格式（pak/lpk）字段与偏移 | `docs/formats/lucencypackage.md` |
| 现代化改造 / 移植 / 多端适配 | `docs/modernization.md` |
| **HD 资产增强 / 超分 / 矢量化 / 多分辨率打包（S7）** | `docs/impl/14_asset_enhancement.md`（方案规格，历史阶段） |
| **AI 生图（UI 效果图/概念图，S18.7 前置）** | `scripts/agnes_gen.py`（Agnes Image 2.5 Flash；`AGNES_API_KEY` 环境变量；用法见 `docs/process/04_tooling.md` §6） |
| Android 壳落地 + Mac 出 APK（S5.5） | `android/README.md` |
| 引入新依赖 / 改 workspace 结构 | `docs/process/01_architecture.md` §4（现成库清单，先查再引）→ `docs/style.md` §5.9 |
| **报「特定区域贴图错位/覆盖菜单」先逐元素定位** | `cargo run -p ti-app --features native-shell --bin element-dump -- <scene>`（§7 元素级验证法：逐精灵落点/越界判定/像素贡献，定位到具体记录号 + f1 精灵） |
| 推送远端 / 分支与 commit 规范 / **完成即自动推送**（gate 全绿 + verified → commit → 自动 push，无需用户逐次指令） | `docs/git.md` |

## 硬规则（违反 = 返工）

1. **不许留占位**：`TODO/FIXME/unimplemented!/todo!/panic!` 及库代码中的 `unwrap/expect` 一律为 0 命中（`tests/` 除外），门禁在 `scripts/gate.ps1`。
2. **验收才算完成**：任务的验收命令**实跑输出**写入 `evidence/<task>.log` 后，才在 `plan/ledger.yaml` 置 `verified`。⛔ **只贴 gate 总输出不算验收证据** —— `doccheck` 会检查 evidence 是否包含任务自己的判据（第 9 项）。
3. **进度只由台账驱动**：循环 = 取下一个非 `verified` 任务 → 实现 → 验收 → `scripts/gate.ps1` → 更新台账 → 终止条件见 `docs/loop.md`（推进至 S4.4）。
4. **结论必须实测**：二进制/格式相关结论写进 `legacy/docs/facts/<分卷>.md` 前必须有脚本证据；推测一律标注 `推测`。
5. **选型已定案**：Rust edition 2024 + macroquad（见 `docs/style.md`）。不要重新讨论语言/框架。
6. **文档拆分 + 单一事实源**：细节进 `docs/`，本文件只做路由与硬规则。**同一结论只写一处**，其他文档引用 ID（`F-xxx`/`EXP-xxx`/`P0-x`）而非复述。新增文档必须登记进 `docs/README.md` 与本路由表。ID 唯一性与跨文档引用有效性由 `doccheck` 门禁（第 9 项）自动校验。

## 命令

```sh
# 常用命令一键入口（16 项命令固化，-List 列全部；含 push 前置 gate 全绿检查）
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 -List
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 run-native     # 桌面版（HD 真彩，F4 切画质）
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 gate         # 全门禁
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 test         # 全 workspace 测试
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 element-dump index 3  # 元素级验证（场景 + 记录号）

# 每轮循环的门禁：gate.ps1 23 项顶层固定 Report（fmt/clippy/test/build/占位/unwrap/zero-unknown/台账/doccheck/validate_data/gen_layout/check_hardcode/oracle边界/doccheck2/台账对账/业务闭环/模块并行/飞轮/视觉回归/渲染快照/正向渲染快照/数据源/场景叙事约束check_scene_narrative；wasm/android 构建为条件子项不计顶层；zero-unknown 亦条件项（verify.rs 在位才计，QA145））
# 全绿才可把任务置 verified，输出 tee 到 evidence/<id>.log
powershell -ExecutionPolicy Bypass -File scripts/gate.ps1

# 单项调试
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace
cargo run -q -p ti-tools --bin ledger  -- plan/ledger.yaml   # 台账校验（gate 第 8 项）
cargo run -q -p ti-tools --bin doccheck                       # 文档一致性（gate 第 9 项，含跨文档数值断言 + release 台账计数漂移）
python scripts/check_ledger_reality.py                         # 台账三方对账（gate 第 14 项，verified↔evidence↔代码符号，EXP-058）

# S2.5：构建浏览器产物到 web/（wasm + miniquad gl.js + 真实资源）
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
# 本地预览（wasm 需 HTTP，不能 file://）
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080

# S2.7：native + wasm 两端冒烟（本机无 bash，务必用 ps1）
powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1

# S1.4b：控件记录全量导出（20 UI 场景 → ref/controls.json，44 种 f5 控件类型）
cargo run -q -p ti-tools --bin controls -- --out ref

# spec 骨架半自动生成（模板化逆向→开发：函数清单表 + 逐函数带行号证据行表，人工只补「语义」列）
cargo run -q -p ti-tools --bin specskel -- <module>              # 打印到 stdout
cargo run -q -p ti-tools --bin specskel -- <module> --out PATH   # 覆盖已有 spec 文件
cargo run -q -p ti-tools --bin specskel -- <module> --report     # 只打印统计摘要
# <module> = ref/functions_<module>.csv 的 <module>（main_loop/render/menu/selectrole/selectmap/
# mainmap/trade/build/battle/cards/saveload/messages/rng）；自动识别两种列格式
# （常规 addr|address,size,name 与 RNG 专用 kind,site,func）

# 反编译导出（S0.2）；IDA 9.x 把 7.5 的 idat64.exe 合并为 idat.exe（32/64 同一程序）
"D:\soft\IDA Professional 9.4\idat.exe" -A -Lref\ida_run.log -Slegacy\scripts\ida_dump.py TreasureIsl.exe
```

- 构建范围随阶段解锁：native（已解锁）→ wasm（**S2.5 已解锁**）→ android（S2.6 在 **Mac** 装 NDK/SDK + cargo-apk 编译 APK，本机 Windows 不产出，gate SKIP）；gate 按已装 target 自动判定并打印 SKIP。
- **桌面/浏览器可玩（S5.1–S5.4）已收口**：菜单→选人→选图→主地图→贸易/建造→战斗→存读档全链路可玩，P3-3 全流程 + P4-1 渲染对齐已机器判定；S0–S14 全 verified，当前活动任务 S16.1（S15.2 已 verified，见 `plan/ledger.yaml`；WBS 详见 `docs/plan-s16-loop.md`）；Android 为后续移动端交付，开发本身不依赖 Android 工具链。
- S5.5 Android 壳已落地（`android-shell` feature + `treasure-android` bin + `android/README.md`）；APK 实际产出在 Mac，本机 host 形态可编译校验。

## REA 逆向 MCP（2026-10 接入）

- 注册：OpenCode `mcp.rea`（`~/.config/opencode/opencode.json`），`environment` 带 `REA_ANALYSIS_PROVIDER=ida` + `REA_IDA_MCP_CONFIG=C:\Users\Administrator\.rea\ida-mcp.json`；headless supervisor = `D:\ProgramData\miniconda3\Scripts\idalib-mcp.exe`（IDA 9.4 + idapro 0.0.9 + ida-pro-mcp 2.0.0）。
- **三层定位**（F-319 波动性可用 + F-321 结构化能力 + EXP-041 分层修正）：
  - ① **批量/离线主路径**（不可替代）：`ref/pseudocode/*.c` grep（S0.2 已全量导出 1827 份）+ `scripts/pe_rva.py` + `idat.exe -A -S`；
  - ② **REA 结构化能力**（补洞，未利用）：CFG/调用图/类型恢复/值追踪/重建验证 —— 冒烟优先，失败即退主路径，见 F-321 + `docs/style.md` §5.11.4；
  - ③ **稳定性**：F-319 波动性可用（S3.2 实测可用 EXP-039 / S3.12 实测失败 EXP-036），逐工具实测表见 `legacy/docs/facts/tools.md`。
