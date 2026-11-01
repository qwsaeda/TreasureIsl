# 23 · 游戏测试标杆（Game Testing Benchmark）

> 用户 2026-11-01 拍板：「根本没有测试——测试等级、分类、类型出标杆，尤其业务常用游戏测试方案、
> Bevy 官方推荐；测试要加入平台看到步骤/阶段/截图证据」。
> 单一事实源：本文（分级 L1–L4 / 分类 / 类型 / 行业常用 / Bevy 官方 / 平台集成）。
> 测试执行证据 = `evidence/`（每任务 `.log` + 截图 `.png`，`task_close.py` 收口强制落盘，`docs/parity.md` P0–P4）。

## 0. 定位

本文把仓库**已有的**测试事实（82 个集成测试文件 + 门禁 33 项 + 试玩闭环 4 报告 + `evidence/` 327 文件）
归入一套**标准化标杆**：按**级**（L1 单元→L4 验收）、**类**（功能/回归/视觉/音频/性能/兼容）、
**型**（冒烟/CI/试玩/探索/基准）三维分类，并给出行业常用方案与 Bevy 官方推荐作为参照。
**不是**新增一套独立测试体系——是给既有测试**统一口径、编号、归类**，并让平台可见。

## 1. 测试分级（Levels，纵向分层）

| 级 | 名称 | 覆盖 | 现有对应（示例） |
|---|---|---|---|
| **L1** | 单元（Unit） | 单规则/单函数最小逻辑（纯函数、不碰表现层/IO） | `ti-core/tests/rules_{battle,trade,build,boat,saveload,territory}.rs`、`reference_diff_*`（oracle 差分，`ti-core/src/oracle` 同种子同输入→同输出）、`ti-tools/tests/pcx_ref.rs`（PCX 解码 vs `scripts/pcx_ref.py` 独立参考，P0-4） |
| **L2** | 集成（Integration） | 跨模块/子系统交互（规则↔数据↔存档/地图包） | `ti-core/tests/exhaust_{combat,economy,progression,menu}.rs`、`map_pack.rs`、`territory_rules.rs`、`ti-app/tests/main_map_pick_e2e.rs`、`render_invariants.rs` |
| **L3** | 系统（System） | 完整场景/全流程（一局可玩链路） | `tests/scripts/full_run.json`（seed=42，16 步）、`ti-app/tests/e2e_playthrough.rs`、`app_{ai_turn,dungeon,siege,territory}_e2e.rs`、`game_app_flow.rs`、`save_migration.rs` |
| **L4** | 验收（Acceptance） | 用户可玩/视觉/性能/兼容终验 | `web_playtest`（CDP 按键驱动 菜单→选人→选图→主地图）、`fuzz_playtest.rs`、`bevy-cpu-parity`（GPU 离屏读回 vs CPU 逐像素，通道 ≤8 LSB 超差 <0.5%）、`render_parity_{gpu,hd}.rs`、视觉签收 S17.5/S21.9/S27.5 |

## 2. 测试分类（Categories，横向维度）

| 类 | 语义 | 现有对应 |
|---|---|---|
| **功能**（Functional） | 规则/业务行为正确 | `rules_*`、`exhaust_*`、`reference_diff_*` |
| **回归**（Regression） | 改动不破坏既有行为 | 门禁 33 项全量（`gate.ps1`，`docs/parity.md` §门禁口径） |
| **视觉**（Visual） | 渲染/版面/资产/像素 | `render_parity_*`、`render_invariants`、`bevy-cpu-parity`、`--scene=X --shot` 出图 `evidence/bevy_native_*.png` |
| **音频**（Audio） | 音乐/音效播放（S34 原版音频接入后） | S34 新增：BGM `glmusic.wav` 循环 + SFX `press/invalid/victory.wav` 一次，`registry.audio` 对账 + 无音频设备降级断言 |
| **性能**（Performance） | fps/加载/体积基准 | `stats`（fps）、wasm 体积 `evidence/S18.8a-wasm-size.log`（44.8MB，`11_bevy_migration.md` §3 预算口径） |
| **兼容**（Compatibility） | native/wasm/android 三端一致 | `smoke.ps1`（两端冒烟）、gate 条件项（wasm/android SKIP 判定）、`bevy_native_*.png` ↔ wasm 同缺陷排查 |

## 3. 测试类型（Types，执行方式）

| 型 | 语义 | 现有对应 |
|---|---|---|
| **冒烟**（Smoke） | 启动/场景加载不崩 | `--scene=<7 场景> --shot`、`cargo build` + `cargo test` 编译 |
| **CI 自动** | 每次提交门禁 | `gate.ps1` 33 项（分档 `-Fast/-Touch/full`） |
| **试玩**（Playtest） | 真实用户/agent 走全流程 | `web_playtest`、`fuzz_playtest`、`playtest-loop`（`08_playtest_loop.md`，4 报告聚合 3 轮收敛入 `evidence/S16.2.log`） |
| **探索**（Exploratory） | agent 自由找问题（找问题→审计→修复→回写） | `06_flywheel.md` 飞轮、试玩 agent 闭环（`08_playtest_loop.md`） |
| **基准**（Benchmark） | 体积/fps 量化基准 | wasm 体积、fps 统计、`audit_code.py` 规模棘轮 |

## 4. 行业常用游戏测试方案（参照）

1. **测试金字塔**（Test Pyramid）：Unit（多）→ Integration（中）→ E2E（少）；本项目 L1→L4 即其落地。
   反向提示：L4 验收测试**少而关键**（每条都贵），L1 单元**多而快**（毫秒级）。
2. **ISTQB 测试级别**：组件（L1）/ 集成（L2）/ 系统（L3）/ 验收（L4）；**测试类型** = 功能 + 非功能
   （性能/兼容/可用性/安全）。
3. **游戏 QA 常用流水线**（本项目按序映射）：
   构建验证（`cargo build`/`build_wasm`）→ 冒烟（`--shot`）→ 功能（`rules_*`）→ 回归（gate 33 项）
   → 性能（体积/fps）→ 兼容（三端）→ **试玩/用户研究**（`web_playtest`/`fuzz_playtest`/视觉签收）。
4. **证据驱动验收**（本项目特有）：每任务验收命令**实跑输出**写 `evidence/<task>.log` 后才置
   `verified`（`docs/loop.md` + 硬规则 2），`task_close.py` 一条命令收口（判据→证据→台账→三方对账）。

## 5. Bevy 官方推荐（参照）

1. **`bevy/tests/how_to_test_apps.rs`**（官方集成测试范式）：`let mut app = App::new();`
   → `app.add_plugins(game_plugin);` → `app.update();`（跑 Startup/Update 系统）→ 断言
   `world` 里的实体/资源。ECS 系统集成测试的标准模式。
2. **测试种类**（Bevy wiki "Testing"）：**单元** / **集成** / **Doctest** / **Compile-fail（trybuild）** /
   **Example-smoke**（引擎 examples 编译冒烟）/ **Benchmark**。
3. **本项目 Bevy 侧测试落地**（`12_bevy_usage.md` §9 坑位 + `20_bevy_picking.md`）：
   - 系统测试：`ti-engine/tests/pick_1080.rs`（命中几何无头判据）、`ti-app/tests/render_invariants.rs`
     （7 场景 × 非黑覆盖率/内容签名互异/暖色板/TTF 在位）；
   - App 冒烟：`--scene=X --shot` 离屏出图（`ShotPreload` 全量资产 Loaded 后读回像素，`treasure_bevy.rs`）；
   - GPU↔CPU 对照：`bevy-cpu-parity`（真实 GPU 离屏读回 vs CPU 期望，UI 层已排除）；
   - 音频契约：`ti-engine/tests/audio.rs`（registry.audio 4 项 + WAV RIFF 魔数 + 磁盘↔登记双向同步；
     实际播放需音频设备，headless 无设备降级不 panic，机器判据 = 资产契约，§2「音频」类）。

## 6. 平台集成（TI Dev Console「测试」页，S32 扩展）

> 用户 2026-11-01 拍板「加入平台」。设计（只读，符合 `22_dev_platform.md` §3 设计原则）：
> 平台只读索引 `evidence/` + gate 输出，零写盘；不加载 wasm、不碰游戏运行时。

**页面**（第 7 页，左菜单「测试」，`/api/tests` 只读适配）：

| 区 | 内容 | 数据源 |
|---|---|---|
| 顶栏 | 门禁 33 项 pass/fail 总览 + playtest-loop 4 报告状态 | `evidence/gate*.log`、`evidence/S16.2.log` |
| 左侧树 | 按 级(L1–L4)→类→型 分组，点选看该组测试清单 | `23_game_testing.md` 分类表（机读 = `assets/data/test_manifest.json`，见 §7） |
| 中部 | 某测试的**步骤/阶段**（从对应 `evidence/<id>.log` 解析：判据步骤、`--shot` 预载/就绪/写盘行、gate 各阶段 Report 行） | `evidence/*.log` |
| 右侧 | **截图证据**（`evidence/bevy_native_*.png` / `wasm_browser_e2e.png` / 试玩截图，缩略图点大） | `evidence/*.png` |

**不做什么**：不跑测试、不改 evidence、不重放断言（只读展示）；门禁/试玩仍由 `gate.ps1`/`playtest.ps1` 触发，
平台只读其结果。

## 7. 机读清单（`assets/data/test_manifest.json`）

测试分组表（级/类/型 ↔ 测试 id ↔ 证据路径 ↔ 门禁项号）落 `assets/data/test_manifest.json`，
供平台 `/api/tests` 与门禁分组对账；`validate_data.py` 增 `check_test_manifest` 兜底
（每条测试 id 唯一 + 证据路径在 `evidence/` 存在或门禁项号在 1..=33 + **命令真实性**：
`cargo test --test <X>` → `crates/<pkg>/tests/<X>.rs` 必须存在，`cargo run --bin <Y>` →
`crates/<pkg>/src/bin/<Y>.rs` 或 `Cargo.toml [[bin]] name+path` 必须存在；防清单指向
已删/改名测试——实测抓出 `pcx_parity` 退役路径、`reference_diff_battle` 不存在、`treasure-headless`
bin 名与源文件名不一致 3 类陈旧项）。

## 8. 执行入口

| 入口 | 命令 |
|---|---|
| 全量门禁（回归+CI） | `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1`（33 项，输出 tee 到 `evidence/<id>.log`） |
| 全 workspace 测试 | `cargo test --workspace` |
| 单场景冒烟出图 | `cargo run -p ti-engine --bin treasure-bevy-native -- --scene=<id> --shot`（`xvfb-run -a` headless） |
| 试玩闭环 | `powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 run-native` + `web_playtest`（`08_playtest_loop.md` §6） |
| 收口（证据→台账） | `python scripts/task_close.py <task>` |
