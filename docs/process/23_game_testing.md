# 23 · 游戏测试标杆（Game Testing Benchmark）

> 用户 2026-11-01 拍板：「根本没有测试——测试等级、分类、类型出标杆，尤其业务常用游戏测试方案、
> Bevy 官方推荐；测试要加入平台看到步骤/阶段/截图证据」。
> 单一事实源：本文（分级 L1–L4 / 分类 / 类型 / 行业常用 / Bevy 官方 / 平台集成）。
> 测试执行证据 = `evidence/`（每任务 `.log` + 截图 `.png`，`task_close.py` 收口强制落盘，`docs/parity.md` P0–P4）。

## 0. 定位

本文把仓库**已有的**测试事实（106 个集成测试文件 + 门禁 36 项 + 试玩闭环 4 报告 + `evidence/` 535 文件（git-tracked，排除 `console/` 侧车；计数实测回写，飞轮 R10 勘误旧 82/327 快照，R10 B 侧 507→534，R11 复验 534→535））
归入一套**标准化标杆**：按**级**（L1 单元→L4 验收）、**类**（功能/回归/视觉/音频/性能/兼容）、
**型**（冒烟/CI/试玩/探索/基准）三维分类，并给出行业常用方案与 Bevy 官方推荐作为参照。
**不是**新增一套独立测试体系——是给既有测试**统一口径、编号、归类**，并让平台可见。

## 1. 测试分级（Levels，纵向分层）

| 级 | 名称 | 覆盖 | 现有对应（示例） |
|---|---|---|---|
| **L1** | 单元（Unit） | 单规则/单函数最小逻辑（纯函数、不碰表现层/IO） | `ti-core/tests/rules_{battle,trade,build,boat,saveload,territory}.rs`、`reference_diff_*`（oracle 差分，`ti-core/src/oracle` 同种子同输入→同输出）、`ti-tools/tests/pcx_ref.rs`（PCX 解码 vs `scripts/pcx_ref.py` 独立参考，P0-4） |
| **L2** | 集成（Integration） | 跨模块/子系统交互（规则↔数据↔存档/地图包） | `ti-core/tests/exhaust_{combat,economy,progression,menu}.rs`、`map_pack.rs`、`territory_rules.rs`、`ti-app/tests/main_map_pick_e2e.rs`、`render_invariants.rs` |
| **L3** | 系统（System） | 完整场景/全流程（一局可玩链路） | `tests/scripts/full_run.json`（seed=42，16 步）、`ti-app/tests/e2e_playthrough.rs`、`app_{ai_turn,dungeon,siege,territory}_e2e.rs`、`game_app_flow.rs`、`save_migration.rs` |
| **L4** | 验收（Acceptance） | 用户可玩/视觉/性能/兼容终验 | `web_playtest`（CDP 按键驱动 菜单→选人→选图→主地图）、`fuzz_playtest.rs`、`bevy-cpu-parity`（GPU 离屏读回 vs CPU 逐像素，通道 ≤8 LSB 超差 <0.5%）、`render_parity_{gpu,hd}.rs`、视觉签收 S17.5/S21.9/S27.5、**Android 真机验收 P5**（`scripts/android_device_test.py --run`，见 §2 兼容类） |

## 2. 测试分类（Categories，横向维度）

| 类 | 语义 | 现有对应 |
|---|---|---|
| **功能**（Functional） | 规则/业务行为正确 | `rules_*`、`exhaust_*`、`reference_diff_*` |
| **回归**（Regression） | 改动不破坏既有行为 | 门禁 36 项全量（`gate.ps1`，`docs/parity.md` §门禁口径） |
| **视觉**（Visual） | 渲染/版面/资产/像素 | `render_parity_*`、`render_invariants`、`bevy-cpu-parity`、`--scene=X --shot` 出图 `evidence/bevy_native_*.png` |
| **音频**（Audio） | 音乐/音效播放（S34 原版音频接入后） | S34 新增：BGM `glmusic.wav` 循环 + SFX `press/invalid/victory.wav` 一次，`registry.audio` 对账 + 无音频设备降级断言 |
| **性能**（Performance） | fps/加载/体积基准 | `stats`（fps）、wasm 体积 `evidence/S18.8a-wasm-size.log`（44.8MB，`11_bevy_migration.md` §3 预算口径） |
| **兼容**（Compatibility） | native/wasm/android 三端一致 | `smoke.ps1`（两端冒烟）、gate 条件项（wasm/android SKIP 判定）、`bevy_native_*.png` ↔ wasm 同缺陷排查、**Android 真机 P5**（`android_device_test.py --run`，见下补强注） |

> **S49.4e 补强（2026-11-09 真机反馈「全屏/菜单无响应/后台音乐不停」根因审计）**：本类原写
> 「三端一致」但**从未定义 Android 真机怎么测、测什么**——执行只 `smoke.ps1`（两端）+ gate
> （android **SKIP 判定**），全屏/触摸/音频生命周期在规范里零判据、零执行、零任务，真机缺陷
> 结构性测不到（只能靠用户实测）。**本类口径修订**：
>
> 1. **三端一致须含 Android 真机执行**，不只是「构建 SKIP」。Android 真机交互判据 = `docs/parity.md` §P5
>    （P5-1 全屏 / P5-2 触摸 / P5-3 音频生命周期 / P5-4 切后台不崩），载体 = `scripts/android_device_test.py`
>    （adb 驱动，设备接入即能跑）。
> 2. **执行方式**：`python3 scripts/android_device_test.py --check`（环境检查：adb + 设备授权 + 包已装）
>    → `--run`（P5 全判据）。设备接入 + app 安装（`adb install dist/TreasureIsl-*.apk`）后执行。
> 3. **判据证据**：`--run` 输出写入 `evidence/android_device_test.log`（gate 第 4c 项 + 验收判据），
>    全 PASS = P5 达标；任一 FAIL = 非 0 退出（如实上报，不静默）。
> 4. **无设备不静默**：gate 第 4c 项有 adb 设备才跑 P5，无设备记名 SKIP + 文档化（本注），
>    不阻塞桌面但**明确记录 Android 真机未测**（防「SKIP 判定」被误读为「已测」）。
> 5. **设备能力分级（2026-11-10 飞轮 r1 修正）**：P5-4 = 进程生命周期判据（任何 adb 设备可跑）；
>    P5-1 = 窗口驻留依赖判据（需 app 窗口驻留 dumpsys，MuMu launcher 抢前台 + 0fps = 不驻留 → 设备标定）；
>    P5-2/P5-3 = 渲染出帧依赖判据（触摸需截屏非黑 / BGM 需渲染管线起，仅 GPU 正常设备有效）。
>    无 GPU 设备（MuMu 转译层缺 `VK_KHR_swapchain_mutable_format` = EXP-181；redroid 纯 CPU pastel 软渲染
>    0fps = 宿主 i915 Vulkan 1.1 不可用 + 无 virtio-gpu gfxstream，2026-11-10 三路径实测）上 P5-2/P5-3
>    无帧可判：`android_device_test.py` 渲染链路健康前置检测（截屏 100% 黑 + swap chain Timeout ≥3）
>    自动记 **SKIP（设备渲染限制，非 app bug，exit 0）**；P5-1 窗口不驻留 = 设备标定 FAIL（需前台稳定设备）；
>    P5-4 照跑。**P5 达标口径** = P5-4 PASS + P5-1/P5-2/P5-3 在 GPU 正常 + 前台稳定设备（arm64 真机）上 PASS；
>    GPU 转译层设备（MuMu Adreno 640 伪装）/ 纯 CPU 设备（redroid pastel）= P5-2/P5-3 SKIP 记名 +
>    P5-1 设备标定，`evidence/android_device_test.log` 含设备 serial 可回溯。MuMu 实测（2026-11-10）：P5-2 PASS / P5-4 PASS / P5-1 设备标定 / P5-3 SKIP。

## 3. 测试类型（Types，执行方式）

| 型 | 语义 | 现有对应 |
|---|---|---|
| **冒烟**（Smoke） | 启动/场景加载不崩 | `--scene=<7 场景> --shot`、`cargo build` + `cargo test` 编译 |
| **CI 自动** | 每次提交门禁 | `gate.ps1` 36 项（分档 `-Fast/-Touch/full`） |
| **试玩**（Playtest） | 真实用户/agent 走全流程 | `web_playtest`、`fuzz_playtest`、`playtest-loop`（`08_playtest_loop.md`，4 报告聚合 3 轮收敛入 `evidence/S16.2.log`） |
| **探索**（Exploratory） | agent 自由找问题（找问题→审计→修复→回写） | `06_flywheel.md` 飞轮、试玩 agent 闭环（`08_playtest_loop.md`） |
| **基准**（Benchmark） | 体积/fps 量化基准 | wasm 体积、fps 统计、`audit_code.py` 规模棘轮 |

## 4. 行业常用游戏测试方案（参照）

1. **测试金字塔**（Test Pyramid）：Unit（多）→ Integration（中）→ E2E（少）；本项目 L1→L4 即其落地。
   反向提示：L4 验收测试**少而关键**（每条都贵），L1 单元**多而快**（毫秒级）。
2. **ISTQB 测试级别**：组件（L1）/ 集成（L2）/ 系统（L3）/ 验收（L4）；**测试类型** = 功能 + 非功能
   （性能/兼容/可用性/安全）。
3. **游戏 QA 常用流水线**（本项目按序映射）：
   构建验证（`cargo build`/`build_wasm`）→ 冒烟（`--shot`）→ 功能（`rules_*`）→ 回归（gate 36 项）
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

> 用户 2026-11-01 拍板「加入平台」。设计（**只读索引 + 白名单动作触发**，符合 `22_dev_platform.md` §3 设计原则）：
> 平台只读索引 `evidence/` + gate 输出；**测试实际运行** = `/api/tests?run=<id>` 白名单动作
> （23 号 §6.1，后台 spawn + 日志落 `evidence/console/logs/`）；不加载 wasm、不碰游戏运行时。

**页面**（第 7 页，左菜单「测试」，`/api/tests` 只读适配）：

| 区 | 内容 | 数据源 |
|---|---|---|
| 顶栏 | 门禁 36 项 pass/fail 总览 + playtest-loop 4 报告状态 | `evidence/gate*.log`、`evidence/S16.2.log` |
| 左侧树 | 按 级(L1–L4)→类→型 分组，点选看该组测试清单 | `23_game_testing.md` 分类表（机读 = `assets/data/test_manifest.json`，见 §7） |
| 中部 | 某测试的**步骤/阶段**（从对应 `evidence/<id>.log` 解析：判据步骤、`--shot` 预载/就绪/写盘行、gate 各阶段 Report 行） | `evidence/*.log` |
| 右侧 | **截图证据**（`evidence/bevy_native_*.png` / `wasm_browser_e2e.png` / 试玩截图，缩略图点大） | `evidence/*.png` |
| 详情卡 | **▶ 运行测试**（点运行 → 后台 spawn test_manifest 命令 → 实时回显日志尾部 + PASS/FAIL 判定） | `/api/tests?run=<id>` |

### 6.1 测试实际运行（S32 用户拍板「能实际运行测试」）

- **白名单 id**：`/api/tests?run=<id>`，`<id>` 必须 = `test_manifest.json` 登记的 `tests[].id`（枚举，非自由字符串，22 号 §10.3）。
- **命令来源**：该 test 的 `command` 字段（禁前端/用户传命令，防任意执行）。
- **程序白名单**：`cargo` / `python3` / `pwsh` / `powershell`（其余程序 = 400 拒绝；`sh -c`/`cmd /c` 字符串拼接禁用，22 号禁止 2）。
- **执行形态**：逐参数 `Command::new(program).args([...])`，`current_dir = repo_root`，后台 spawn（`Stdio::null`），日志落 `evidence/console/logs/test-<id>-<ts>.log`（gitignored）。
- **判定**：日志含 `test result: ok` = PASS、`FAILED`/`error[`/`panicked` = FAIL；前端每 2s 轮询尾部增量回显。
- **不可自动运行**：人工验收（如 `visual_signoff`「用户视觉验收」无白名单程序）、无 `command` 字段 → 按钮禁用并注明。

**不做什么**：不重放断言（运行测试走 test_manifest 命令本体）；门禁/试玩仍由 `gate.ps1`/`playtest.ps1` 触发，
平台只读其结果 + 白名单动作触发运行。

### 6.2 统一作业模型（`runner/jobs.rs`；S49.7）

> 用户 2026-11-09 拍板：「平台测试交互问题很大……**从架构上要统一设计，复用**」。
> 根因 = 平台四类「跑一条命令」动作各写一套 spawn/状态/日志（测试自写且**不记 exit_code**、
> 构建自写 + 自写锁、发布/门禁各拼状态路径）⇒ 「跑完了吗 / 成没成 / 证明在哪」三个问题没有统一答案。

**一个动作 = 一个作业**（同一实现，不各自 spawn）：

| 面 | 契约 |
|---|---|
| 作业种类 `kind` | `test` / `build` / `publish` / `gate`（`runner::jobs::JobKind`） |
| 作业条目 `JobSpec` | `{ kind, id, program, args, meta }`；`program` ∈ 白名单 `cargo`/`python3`/`pwsh`/`powershell`（22 号 §10.3）；**逐参数非 shell**（禁 `sh -c`/`cmd /c`） |
| 状态文件 | `evidence/console/jobs/<kind>-<id>.json`（统一目录，gitignored）：`{kind,id,running,pid,exit_code,started_ts,ended_ts,log,evidence[],note}` |
| 日志 | `evidence/console/logs/<kind>-<id>-<ts>.log`（**流式**写 ⇒ 构建/测试输出实时可见） |
| 收割 | spawn 后**收割线程**等子进程退出 → 写 `running=false` + `exit_code` + `ended_ts` ⇒ 「跑完了吗 / 成没成」有机器判据（不靠日志字符串猜） |
| 证明 | 作业产物（日志 + 报告 + 截图）登记进状态文件 `evidence[]`，落 `evidence/tests/<id>/<ts>/` ⇒ **证明与本次运行绑定** |
| 查询 | `GET /api/jobs?kind=<kind>`（列表 + 状态）/ `?kind=<kind>&id=<id>`（详情 + 日志尾 + 证明） |

**迁移对照**（S49.7 全迁）：

| 作业 | 迁移前 | 迁移后 |
|---|---|---|
| 测试 | `repo::data::run_test` 自写 spawn（**无 exit_code、无状态文件** ⇒ 前端无从显示通过/失败） | `jobs::spawn_job(kind=test)` |
| 构建 | `repo::build_ops::run_build` 自写 spawn + 自写 `build.lock` | `jobs::spawn_job(kind=build)` + 作业模型内互斥检查（互斥语义保留） |
| 发布 | `content::publish` → `spawn::spawn_script_logged`（各自拼状态路径） | `jobs::spawn_job(kind=publish)` |
| 门禁 | `runner::gate` → `spawn::spawn_script_logged`（各自拼状态路径） | `jobs::spawn_job(kind=gate)` |

### 6.3 平台测试页交互规范（分组分类 → 逐个运行 → 出结果出证明）

> 用户 2026-11-09 拍板：「应该**分组分类**，然后**一个一个测试运行出结果**，**出证明**」。
> 旧交互三处不合规：分组是「级→类」静态两点树（无业务域视角 / 无折叠 / 无筛选）；条目无「运行」入口；
> `evidence` 是清单里**静态声明**的基线路径（与本次运行无绑定）。

| 区 | 内容 | 机器判据 |
|---|---|---|
| 侧栏 | **业务域分组**（`group`，§7.2）→ 域内**分类**（`category`）计数；搜索框按 name/id/command 过滤 | 分组计数 = manifest 实际条目数（机器对账） |
| 主区 | 折叠分组列表（业务域 → 分类 → 条目）；每条 = 名称 + id + `level`/`type` 徽章 + 证据在位/断链 + **状态徽章**（未跑 / 跑中 / 通过 / 失败+`exit_code`+耗时） | 条目数 = manifest 条目数（无遗漏无重复） |
| 运行 | 每条「▶ 运行」+ 分组「运行本组」（**串行**：同目标目录会互踩 cargo 锁，同 S46.3「单构建互斥」纪律） | 点击后 ≤2s 内出现「跑中」徽章（**不刷新页面**） |
| 结果 | 轮询 `GET /api/jobs?kind=test&id=<id>` → `exit_code` + 耗时 + **日志尾实时回显** | `exit_code` 落盘（跑完即有） |
| 证明 | 每条列**本次运行**的证明（作业状态 `evidence[]`：日志 + 报告 + 截图）+ 清单里静态基线证据 | 证明路径存在且与本次作业 id/时间戳对应 |
| 禁跑 | `command` 缺/`null`（人工视觉验收 / gate 覆盖）→ 按钮禁用**并注明原因** | 禁用原因可见（不静默） |


## 7. 机读清单（`assets/data/test_manifest.json`）

测试分组表（级/类/型 ↔ 测试 id ↔ 证据路径 ↔ 门禁项号）落 `assets/data/test_manifest.json`，
供平台 `/api/tests` 与门禁分组对账；`validate_data.py` 增 `check_test_manifest` 兜底
（每条测试 id 唯一 + 证据路径在 `evidence/` 存在或门禁项号在 1..=36 + **命令真实性**：
`cargo test --test <X>` → `crates/<pkg>/tests/<X>.rs` 必须存在，`cargo run --bin <Y>` →
`crates/<pkg>/src/bin/<Y>.rs` 或 `Cargo.toml [[bin]] name+path` 必须存在；防清单指向
已删/改名测试——实测抓出 `pcx_parity` 退役路径、`reference_diff_battle` 不存在、`treasure-headless`
bin 名与源文件名不一致 3 类陈旧项）。

### 7.1 测试接入标准（S45，单一事实源 = [`31_test_ingress_standard.md`](31_test_ingress_standard.md)）

**新增测试接入平台的固化标准**（用户 2026-11-06「固化一个标准，新脚本按标准写测试代码接入平台」）：

| 面 | 契约 |
|---|---|
| schema | `tests[]` 字段：`id`（唯一，`^[a-z0-9_]+$`）/ `level`∈L1~L4 / `category`∈6 类 / `type`∈5 型 / `name`（必填）/ `command`（§3）/ `evidence` XOR `gate`（至少一者） |
| command | 首词 ∈ `cargo`/`python3`/`pwsh`/`powershell`（平台 `run_test` 白名单）；逐参数非 shell（禁 `;`/`&&`/`\|`/`>`/反引号/`$(`）；命令真实性（--test/--bin 路径存在） |
| 登记流程 | 写测试 → 登记 `test_manifest.json` 一行 → `validate_data` 全过 → 平台 `/api/tests` 自动出现（**零平台 Rust 改动**）→ `?run=<id>` 白名单执行 |
| 门禁断言 | `check_test_manifest`（schema + 程序白名单 + 无 shell 拼接面 + id 格式 + name 必填 + evidence↔gate） |

> 详情（含登记流程逐步 + 平台接入口径 + 零平台代码断言）= 31 号 §2~§6，本段只引不复述。

### 7.2 manifest 扩展（`group` / `timeout_s` / `prove`；S49.7）

| 字段 | 必填 | 契约 |
|---|---|---|
| `group` | ✓（S49.7 起） | **业务域**：`core`（核心规则/玩法）/ `asset`（资产与内容管线）/ `platform`（平台与工具）/ `device`（端侧与兼容）/ `perf`（性能与体积）—— 主分组维度（§6.3 侧栏） |
| `timeout_s` | ○ | 作业超时（秒；缺省 = 不限；超时 ⇒ 作业记 `exit_code=-1` + note `timeout`） |
| `prove` | ✓（可自动运行者） | 本次运行**证明落点**（`evidence/tests/<id>/`）；与 `evidence`（静态基线证据）并存不冲突（§6.3「证明」行） |

## 8. 执行入口

| 入口 | 命令 |
|---|---|
| 全量门禁（回归+CI） | `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1`（36 项，输出 tee 到 `evidence/<id>.log`） |
| 全 workspace 测试 | `cargo test --workspace` |
| 单场景冒烟出图 | `cargo run -p ti-engine --bin treasure-bevy-native -- --scene=<id> --shot`（`xvfb-run -a` headless） |
| 试玩闭环 | `powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 run-native` + `web_playtest`（`08_playtest_loop.md` §6） |
| 收口（证据→台账） | `python scripts/task_close.py <task>` |

## 9. 架构统一与复用原则（S49.7）

1. **一个动作 = 一个作业**：平台任何「跑一条命令」的动作（测试/构建/发布/门禁）**必须**走
   `runner/jobs.rs` 统一作业模型；新增动作**不得**自写 spawn / 状态文件 / 日志路径（doccheck2 机器化）。
2. **声明优先**：可枚举的测试/动作 = manifest 声明（登记即接入，**零平台代码**）；不可枚举 = 白名单 id 枚举。
3. **命令契约**：程序白名单 + 逐参数非 shell + 逐项真实性校验（31 号 §3）。
4. **结果即判据**：退出码落盘（`exit_code`）+ 收割线程记终态；**不靠日志字符串猜**通过/失败。
5. **证明与运行绑定**：本次运行产物登记进作业状态 `evidence[]`；清单里的静态 `evidence` 只是**基线**，二者不混。
6. **互斥显式**：会互相踩目标目录/锁文件的动作（构建/门禁）= 作业模型 + 显式互斥检查（busy 回显），不做隐式串行。
7. **0 游戏依赖**：平台本体（`ti-console`/`ti-devmeta`）不引游戏 crate；作业模型只管进程与文件。
8. **规范优先于实现**：测试相关改动先改本规范（分级/分类/类型/schema/交互/作业模型），再落码；判据以本规范为准。
