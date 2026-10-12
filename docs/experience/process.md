# 经验库 · process

> 流程与协作：验收纪律、并行会话、文档治理、oracle 规约。条目 ID 全局唯一（EXP-NNN），维护规则与全量索引见 docs/experience/index.md。

### EXP-007 流程：低价值目标不要反复投入（`.rsrc` 解析两次失败）
- 现象：解析 `.rsrc` 目录两次出错（基址换算、根目录 numId 偏移读错），第三次才意识到它只含图标/版本信息。
- 根因：没先评估信息价值就动手。
- 解法：先问「拿到它能推进哪个验收」，不能则记为缺口、跳过。
- 验证：跳过后无任何任务受阻（无任务依赖 `.rsrc`）。


### EXP-009 流程：二进制结构结论必须先 hexdump 再下笔
- 现象：EXP-003、EXP-005、EXP-007 都是「按直觉写解析 → 失败 → 回头看字节」。
- 根因：跳过了「先观察、后建模」。
- 解法：任何格式先出一张偏移标注表（docs/formats/ 里放 hexdump 片段），确认后再写 `define_format!`。
- 验证：S1 各格式一次性通过 roundtrip（作为阶段统计指标，若反复返工则本条需补充）。


### EXP-012 流程：文档与环境冲突时，先改文档再开工
- 现象：规范写 `gate.sh`+`rg`，环境不支持；若直接按文档硬做会产出跑不起来的脚本。
- 解法：把「文档 → 环境」的适配作为迭代 0 的独立动作，连同笔误（六 crate→5）一起修，并在计划文件写修订记录，避免规范与现实长期背离。
- 验证：修订记录与提交历史对应。


### EXP-013 流程：编译工具链后置（用户决策）
- 现象：原计划把 `rustup target add`（wasm/android，数百 MB 下载）放在 S0.1，阻塞真正的代码开发。
- 解法：**先代码、后工具链** —— S0.1 只建 workspace + native 构建；wasm/android target 分别移入 S2.5/S2.6，gate 第 4 项按阶段解锁（脚本里用 SKIP 标注未解锁项）。
- 验证：S0.1 全程不装 target 即可 gate 全绿。


### EXP-022 存储：三端差异只放后端，语义用「同一套断言」锁（S2.4 实测）
- 设计：`storage::Backend` trait（`load/store/remove/keys`）+ 两个可测实现：`MemoryStore`（headless / 语义基准）与 `FileStore`（桌面，每键一文件）。
- **三端一致性做法**：`tests/storage.rs` 里 `all_backends_share_identical_semantics` 对**同一批键值跑两遍**并逐项比对返回值与键集合；S2.5/S2.6 接上 IndexedDB / Android 后端时，**必须复用同一组断言**（把新后端塞进同一个 for 循环），而不是另写一套测试。
- 键名规范 `validate_key`：ASCII 字母数字 + `_ . -`，长度 1..=64；**禁止路径分隔符**（否则桌面文件实现会被目录穿越），非法键在三端都必须返回错误而不是静默成功。
- 体积：`MemoryStore::bytes_used()` 用于存档上限校验；512 KB 存档往返已覆盖。

### EXP-034 流程：并行会话工作树里先 git status 再动手，台账计数会「漂移」
- 现象：开工时台账 23 verified，几分钟后变 26；`ref/classify.json`、`evidence/S0.3.log` 等凭空出现；`ti-core/src/oracle` 编译坏（`RenderPlan` 重复定义）——都是并行会话（S0.3 / S3.2）的进行中状态。
- 教训：loop.md §5 的「先 git status + 读 ledger」不是仪式：**每开一轮、每个大动作前都重读一次**；别人未提交的半成品不属于你的文件域，编译坏了先确认是不是别人正在写（`git status` 看 untracked 目录 + 最近 mtime），不是就修，是就绕开/上报。


### EXP-038 流程：子代理声称 verified 后必须主线程独立复核（S3.3 台账回退实测）
- 现象：S3.3 子代理报告「台账已置 verified + gate 0 failure」，但几轮后重读 plan/ledger.yaml 发现 S3.3 行变回 status: pending、evidence: null（被后续子代理改台账时覆盖），而 oracle/menu.rs / rules/menu.rs / tests/diff_menu.rs / evidence/S3.3.log 全部真实存在且 diff_menu 4/4 实跑通过。子代理虚报或台账被回退，导致 verified 计数漂移。
- 根因：多子代理并发写 plan/ledger.yaml 时，后写的子代理用旧版 ledger 全量覆盖，把先前子代理刚置的 verified 冲回 pending（loop.md §6.2 第 2 条「只改自己任务行」未严格执行——子代理改的是整文件）。
- 解法：主线程（非子代理）每次收子代理报告后必须独立复核三件事：(1) 代码/测试文件真实存在（Test-Path）；(2) 实跑该模块 diff 测试（cargo test -p ti-core --test diff_<m> 看 test result）；(3) 重读 ledger 该行 status/evidence 是否仍为 verified。三者齐备才认 verified，缺任一就补写/回滚。
- 验证：本次 S3.3 复核 = 4 文件 EXISTS + diff_menu 4 passed + ledger 行修回 verified + rounds=1。
- 教训：**子代理的 verified 声称不可直接采信**，主线程是最终裁判；台账必须按行 edit（edit 工具的 oldString/newString 只含目标任务行），禁止整文件重写。


### EXP-040 逆向：S3 模块 oracle 固定种子确定性公式约定（S3.8 battle 实测）
- 现象：S3.8 战斗系统 oracle 需要「固定种子整场战斗输出一致」，但不能引入 rand 库（ti-core 无平台依赖硬约束 + 确定性要求）。
- 根因：oracle/impl 必须对同输入逐位一致，且可差分测试。真随机不可复现，必须用确定性伪随机。
- 解法：统一用 LCG 式确定性公式 `seed = seed.wrapping_mul(K).wrapping_add(C)`（S3.8 用 K=2654435761 / C 见 oracle/battle.rs，S3.12 全局 RNG 用 MSVC LCG K=214013 C=2531011，见 EXP-037），**不引 rand crate、不用 SystemRandom**（注：S8.7 起 rules 侧改 `rand_pcg` PCG，LCG 为 oracle 历史参照）。播种点（毫秒时间种子）不进 oracle，由调用层注入；oracle 只负责「给定种子 → 确定序列」。
- 验证：diff_battle 1650 组合（11 seed × 5 tier × 6 HP × 5 atk）oracle/impl 逐位一致 + semantic_deterministic_output 同 seed 两次独立调用逐位相同。
- 教训：**oracle 层一律确定性 LCG（wrapping_mul/add，整数），禁 rand/SystemRandom**；具体常数写进各模块 spec（battle.md/battle.rs 头注释），S3.12 统一全局 RNG 时核对是否同一套常数，避免各模块各用一套。


### EXP-048 流程：对抗审计发现的「可机械判定」检查项必须固化进 doccheck/gate，否则下一轮必然复发
- 现象：第 1-5 轮对抗审计反复命中同一类问题（文档与代码/数据不一致、孤儿 key、LCG hex 误算、据点阈值方向、release 门禁项数漂移、EXP 编号重号等），每轮靠子代理现场 grep 发现，跨会话记忆不沉淀。
- 根因：审计结论没有「固化」为机器可判定的断言；审计是一次性动作，断言必须进入门禁才能跨会话生效。
- 解法：凡审计中发现「重复、可机械判定」的检查项，必须写进 `scripts/check_*.py` 或 `doccheck`/`doccheck2` 断言集（gate 第 9/12/13 项），并在 `docs/process/04_tooling.md` 登记；`doccheck2` 固化 6 项高频断言（见 `docs/parity.md` 第 13 项），后续审计优先扩展断言集而非新增现场 grep。
- 验证：gate 14 项中 doccheck2（第 13 项）0 error；新增断言后对应审计项在下一轮不再复发。


### EXP-061 代码：行内全路径引用（`ti_core::data::balance()`）vs use 声明（S10.3 教训）
- 现象：S10.3 子代理在 `game_app.rs`/`scenes/main_map.rs` 方法体里行内写 `ti_core::data::balance()` / `ti_core::rules::build::cost_of` / `ti_core::rules::trade::sell_price`（9 处），绕过文件顶部已有的 `use` 声明习惯，违反 `docs/style.md` §5.8「crate 内 pub use 汇聚到 prelude；app 层统一 use ti_core::prelude::*」。
- 根因：子代理写新增代码时没看文件既有 use 习惯，直接全路径省事（行内全路径可读性差且绕过 prelude 约定）。
- 解法：行内全路径抽成文件顶部 `use` 声明（`use ti_core::data::balance; use ti_core::rules::build::cost_of; use ti_core::rules::trade::sell_price;`），方法体只用短名。子代理 prompt 里必须写「改动前先读文件既有 use 块，新增符号走 use 声明，行内全路径引用 = 0」。
- 验证：严格 grep（非 use/非注释行）`ti_core::` 命中 = 0（`crates/*/src` 全量）。
- 教训：新增代码必须对齐文件既有 use 习惯；行内全路径引用是子代理常见风格漂移，主线程复核必须 grep「非 use 行的全路径引用」一项。


### EXP-058 流程：子代理「verified」报告必须主线程三复核 + 台账机器对账（S9.5 教训）
- 现象：S9.5（Command 模式）子代理报告「已 verified + evidence/S9.5.log 已写 + ledger 已置 verified」，但主线程复核发现 `crates/ti-app/src/commands.rs` **根本不存在**，`evidence/S9.5.log` 也不存在，ledger 行实际仍是 pending（被后续并发子代理整文件覆盖回退，同 EXP-038 模式）。子代理「声称 verified」不可直接采信。
- 根因：多子代理并发 + 子代理虚报。子代理改 ledger 用整文件重写冲掉他人 verified（EXP-038 未根治）；子代理对「代码产物是否真实落地」缺乏 Test-Path 复核，凭主观判断报「已完成」。
- 解法：
  1. 主线程收子代理「verified」报告后**强制复核三件事**（Test-Path 代码文件 + 实跑 diff 测试 + 重读 ledger 行），缺任一回滚（继承 EXP-038/loop.md §6.2 第 8 条）；
  2. **机器化对账**：新增 `scripts/check_ledger_reality.py`（gate 第 14 项），遍历 ledger `status: verified` 行，校验 (a) evidence 文件存在且非空；(b) DoD 声明的代码符号（在 `SYMBOL_EXPECTATIONS` 登记的如 `trait Command`、`commands.rs`）在 `crates/*/src` grep 命中；缺任一 FAIL「台账虚报」；
  3. 子代理改 ledger **只许 edit 工具按行替换**（oldString/newString 含目标任务行），禁止整文件重写。
- 验证：`check_ledger_reality.py` 对当前 ledger 全绿（verified 任务 evidence 全存在 + 登记符号全命中）；S9.5 回退 pending（commands.rs 不存在），第 14 项 FAIL 拦截虚报。
- 教训：verified 是「代码产物真实落地 + 测试实跑通过 + 台账三对齐」的合取命题，任一缺项即虚报；靠机器对账（gate 第 14 项）而非人工自觉。


### EXP-059 代码：地形 camera 钳位 + 触摸/渲染坐标系脱节 = 主地图界错乱双 P0（S10.1 修复）
- 现象：用户报告「游戏界错乱、地图人物加载有问题」。主地图真实地形 blit 后，玩家棋子/据点画在错误位置，触摸点按走错方向。
- 根因（双 bug 叠加）：
  1. `game_app.rs` `terrain_camera()` 用 `i64::max(cx, 0)` 钳位负值——`TerrainAtlas::center_camera` 对 m01 返回 `(cx≈-520, cy≈-202)`，被钳成 `(0,0)`，导致「地形 blit 用真值 camera（屏幕外）」与「玩家棋子用钳位后 camera=(0,0)（原始 iso 位置）」两套坐标系，地形与人物脱节；
  2. `touch.rs` `grid_cell_at` 用占位 6×6 网格（`GRID_X0=20/Y0=30/STEP=6`）命中，而渲染走等距投影 + camera——触摸坐标系与渲染坐标系完全脱节，点哪走哪但方向错。
- 解法：
  1. `terrain_camera` 签名改 `(i64, i64)` 去 `max(0)` 钳位，`SceneHost::terrain_camera` 同步；`main_map.rs` 直接用真值；
  2. `grid_cell_at` 增 `terrain_drawn` 参数，地形走**等距投影逆变换**（iso_x/iso_y → row/col），与渲染同坐标系；`main_map.rs` 触摸命中改调逆变换。
- 验证：headless 断言「点按棋子 → 实际移动方向一致」（主地图触摸与渲染同坐标系不变式，进 `docs/parity.md` P0 判据）；`terrain_camera` 返回值与 `TerrainAtlas::center_camera` 一致（负值保留）。
- 教训：「渲染走投影 A、命中走网格 B」是经典坐标系脱节坑；凡「画面位置」与「交互命中」必须共用同一变换函数，禁两套坐标。camera 钳位（`max(0)`）看似防御实则破坏负偏移场景，签名应表达真实值域（`i64` 而非 `u32` 钳 0）。


### EXP-060 代码：HD 资产不递归子目录 + 场景 key 碰撞 = HD 失效 + 切场景残留（S10.2 修复）
- 现象：`assets_hd_pack/1x/` 下 5 个子目录（ui/glmap_m01/glmap_m02/glpics/cards）共 180+ PNG，但 HD 档实际全失效（回退 PCX）；Saveload→Index 切场景偶发残留上一帧。
- 根因：
  1. `game_app.rs` `load_hd_pack_from_dir` 用 `std::fs::read_dir(dir)` 只枚举一级，`1x/` 下 0 个直接文件（全在子目录），`read_iter.flatten()` 不递归 → HD PNG 全未注入；
  2. `treasure_native.rs` `last_rendered_scene` 存 `Scene::key`（`Intro|Exit => 7` 碰撞），切到 Intro/Exit 时若前一场景也 key=7 则跳过渲染 → 残留。
- 解法：
  1. `load_hd_pack_from_dir` 改递归遍历子目录（`walk` 或手动递归），命名映射 `ui/NN_hd.png`→f1=NN、`glmap_m01/NN_hd.png`→瓦片 NN；HD 注入计数 = assets_hd_pack 全量 PNG；
  2. `last_rendered_scene` 改存 `SceneKind`（9 个不同标识）而非 `Scene::key`（碰撞的 i32）。
- 验证：HD 注入计数断言（= assets_hd_pack 全量）；切 Intro/Exit/Saveload 无残留帧。
- 教训：`read_dir` 非递归是隐性陷阱——资产目录结构变化（加子目录）后静默失效；凡「加载资产目录」必须断言「注入数 = 目录全量文件数」，不可默认成功。场景 key 用作脏检查标识必须唯一（碰撞 → 残留），用枚举 `SceneKind` 而非压缩 i32。


### EXP-063 流程：doccheck 任务 ID 是散文全库扫描，新任务先登记台账（pending）再写文档引用（S11 踩坑）
- **现象**：S11 登记 S11.3/S11.4 与 S12 各包任务后，doccheck 第 9 项报 30 处「references task which is not in plan/ledger.yaml」——全是文档散文里写的 S12 各包 / S0 系列等任务 ID 没进台账。
- **根因**：doccheck 的任务 ID 引用检查是**全库散文扫描**（docs/*.md 每行提取 `S` 数字点数字/小写字母 形式的 token 与台账 ID 对账），不是只看「正式引用」。新阶段任务在文档里先写、台账后登记，文档先行就会批量 FAIL；且散文中「某阶段的 x」（x 代指 0 到 4）这种范围写法也会被当成任务 ID token 抓出来。
- **解法**：
  1. 新阶段开轮前先写 `plan/ledger.yaml` 任务行（status: pending 即可），再写/改文档引用 —— 台账先行，散文后行；
  2. 散文里「某阶段 x 这类」范围写法避免裸写 token，写成「S10.0–S10.4」或改用文字「S10 系列」，不让扫描抓到具体 ID；
  3. 台账行 `status: pending` 不算漂移（漂移检查只数 verified），pending 任务可先登记占位。
- **验证**：`cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` 0 error（S12 各包登记后 104 任务全过）。

### EXP-064 流程：子代理只拿到任务书 = 跨会话上下文丢失（会话起点必须注入路径/前缀/符号/台账）
- **现象**：模块并行开发（`process/05_module_parallel.md`）里，一个并行会话/子代理若只拿到任务书而没有仓库绝对路径与 crate 前缀约定，就会按错误假设动手（如把 crate 前缀写成 `ty-`、把仓库根当工作目录、猜 `crates/` 下的文件名），产出与真实仓库不符的改动。
- **根因**：任务书描述**做什么**，不描述**在哪做、用什么前缀、当前哪些任务已 verified**。缺了环境坐标，代理会用通用默认习惯补齐，等于把推断写进代码。
- **解法**（会话起点 prompt 必含 5 项）：
  1. 仓库绝对路径 + 工作目录；
  2. crate 前缀约定：`ti-`（禁止 `ty-`），并列出实际存在的 `crates/*`；
  3. 关键符号/文件路径（点名到文件，不让代理猜）；
  4. 契约锚点（`docs/contracts/` 哪几份、状态 draft/frozen）；
  5. 当前任务 + `plan/ledger.yaml` 已 verified 清单（防止重复劳动与状态误判）。
  子代理回执必须逐条对齐这 5 项；任一项缺失 → 判定返工，不算完成。
- **验证**：`scripts/check_module_contracts.py`（gate 第 16 项）校验包内零跨包 import + 契约锚点；会话任务书模板见 `process/03_templates.md` §7。

### EXP-065 流程：台账被文件级回退会静默丢条目（孤儿 evidence 检测兜底，gate 第 14 项）
- **现象**：S11/S12 阶段共 23 条台账任务（S9.2–S9.5、S10.0–S10.5、S11.1–S11.4、S12 系列）在某次回退后从 `plan/ledger.yaml` 消失，而 `docs/plan-*.md` 的任务描述与 `evidence/S10.*.log`、`S11.*.log` 全部还在 → doccheck 第 9 项一次性报 **115 条**「references task which is not in plan/ledger.yaml」。
- **根因**：`git checkout -- <file>` / `restore` / 单文件 reset 不进 reflog，工作区其余文件（未跟踪的 evidence、被改的 docs）全保留，**只有台账被打回旧版**，从表面看不出丢了东西；`evidence/S11.1.log` 里「104 tasks / 85 verified」与实测 81 的差就是线索。
- **解法**：
  1. `scripts/check_ledger_reality.py`（gate 第 14 项）增**孤儿 evidence 检测**：`evidence/` 下以任务 ID 命名的文件，其 ID 必须在台账存在，否则 FAIL；
  2. 文档里写任务 ID 前先登记台账行（EXP-063：台账先行），否则一旦回退就是成片 404；
  3. 长阶段（S10/S11 这种一开十几个任务）收尾时 commit，避免全部工作只存在于工作区。
- **验证**：`python scripts/check_ledger_reality.py` 正常态「88 个 verified 三方对账通过 + 0 个孤儿 evidence」；临时构造孤儿证据文件即报 FAIL 且退出码 1（`evidence/S13.2.log`）。
