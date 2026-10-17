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

### EXP-068 流程：子代理开轮不读最新规范/EXP/契约 → 重复踩坑（开轮必读清单 + 规范回执，doccheck2 第 8 项固化）- **现象**：子代理跑对抗流水线时不读最新规范/EXP/契约，凭 prompt 摘要 + 自身惯例做，
  导致重复踩坑：crate 前缀写成 `ty-`（应为 `ti-`）、跨层几何公式自洽但不对齐渲染侧
  （未读 EXP-067 跨层几何正逆成对测试约定）、不知道 gate 第 16 项查什么就产出违规写法。
- **根因**：任务书只列「白名单 + 目标 + 验收」，无「开轮必读清单」；
  子代理回报无「规范回执」判据，主线程无法确认代理是否读过现行规范。
- **解法**：
  1. 新建 `docs/process/06_flywheel.md`（飞轮单一事实源）：3 角色（找问题/审计/修复）
     + 开轮必读 6 条 + 飞轮回写规则 + 停机条件；
  2. `05_module_parallel.md` 新增 §4.5「开轮必读清单 + 规范回执」，
     下发/回报双侧硬要求，缺项打回；
  3. `03_templates.md` §7 任务书模版加「开轮必读 + 回执」段（checkbox 清单 + 回执 4 项）；
  4. `doccheck2.rs` 新增第 8 项断言 `check_flywheel_readback` 固化（检查上述 4 份文档关键词）。
- **飞轮**：子代理开轮读规范 → 命中回执 → 发现漂移 → 固化/EXP/lint 回写
  （`06_flywheel` §3）→ 下轮子代理读到新规范/新 EXP，不重复踩坑。
- **验证**：`cargo run -q -p ti-tools --bin doccheck2 .` 8 项 0 error（含第 8 项）。

### EXP-070 流程：开轮必读靠 LLM 手动 Read 6 份文档 → 工具调用爆炸 + 漏读（脚本化一键生成，EXP-068 增强）
- **现象**：EXP-068 开轮必读靠 LLM 每次手动 Read 6 份文档 + 读 gate + 读 doccheck2 + 读 packages.yaml，
  N 个子代理各跑一遍，工具调用爆炸 + 容易漏读（本轮 P6 子代理没读 06_flywheel 的在途白名单）。
- **根因**：必读清单是「规格」不是「脚本」——文档写死了 6 条路径，但没固化成一个一次跑全的入口。
- **解法**：
  1. 新建 `scripts/flywheel_readme.ps1`：一键生成完整必读输出（环境事实 + 6 文档路径 + 白名单
     + 本域 EXP + gate 项 + doccheck2 断言），读 `plan/packages.yaml` / `docs/experience/index.md`
     / `scripts/gate.ps1` / `doccheck2.rs` 的实时内容，规范变更后脚本自动反映，无需改 prompt；
  2. gate 加 `flywheel-readme -Check` 项（第 17 项）验证脚本输出完整性；
  3. `06_flywheel.md` 加「§开轮必读脚本化」节，LLM 子代理只需跑脚本 + 回执，不再逐个 Read 工具调用；
  4. doccheck2 加第 10 项 `check_flywheel_script`（断言脚本存在 + gate/06_flywheel/index.md 含关键词）。
- **飞轮**：脚本读的是各文件的**实时内容**，规范/EXP/门禁变了脚本自动反映；
  LLM 子代理只需跑 `powershell -File scripts/flywheel_readme.ps1 -Package <本包>` + 回执，不再逐个工具调用。
- **验证**：`powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package P6` 输出含 6 条必读
   + P6 白名单 + 本域 EXP + gate 17 项 + doccheck2 10 项；`-Check -Quiet` exit 0；
   `cargo run -q -p ti-tools --bin doccheck2 .` 10 项 0 error。

### EXP-072 流程：表现层坐标/渲染改动后界面错乱，`cargo test` 全绿但漏到用户才发现（视觉回归基准图 + gate 第 18 项）
- **现象**：P6 表现层改触摸/渲染坐标后界面错乱，`cargo test` 全绿（gate 16/17 项通过）但没跑实机/截图验证，漏到用户才发现。
- **根因**：gate 16/17 项全是编译/测试/文档一致性，**没有视觉几何回归**；P4-1 `render_parity` 只锁 PCX 解码基准（`render_parity.rs` 5 项断言），不锁「场景渲染最终像素」；表现层坐标/布局改动的回归只能靠肉眼或实机截图。
- **解法**：
  1. `crates/ti-app/src/game_app.rs` 加 `render_to_framebuf()`（渲染 + 返回 176×220 RGB565 只读切片，测试专用）；
  2. `crates/ti-app/tests/visual_regression.rs`（5 测试）：3 关键场景（Index/SelectRole/MainMap）headless 渲染 → 逐像素与 `assets/hd_baseline/<场景>.png` 对比（RGB565→RGB8，允许 ±1 量化误差），基准图缺失时生成落盘，存在时 diff 断言；
  3. `scripts/visual_baseline.ps1`（幂等）：`-Check` 模式 = 基准图 3 张在位 + `cargo test` 全绿 → exit 0；
  4. gate 加第 18 项 `visual-regression`（`visual_baseline.ps1 -Check`）。
- **飞轮**：表现层任何坐标/布局改动 → gate 第 18 项自动 diff 基准图 → 漂移当场拦，不等用户发现；改完正常渲染后重跑 `visual_baseline.ps1`（无 -Check）重新生成基准图。
- **验证**：`cargo test -p ti-app --test visual_regression` 5 项全绿（首次生成 3 张基准图 + 二次 0 diff）；`powershell scripts/visual_baseline.ps1 -Check` exit 0；gate 第 18 项 PASS。

### EXP-074 代码：数据表坐标迁移必须对拍旧验收视觉（scenes.json 回归旧值 + 视觉基准掩背景，HD 化只换贴图不动逻辑布局）
- **现象**：用户报「系统性全场景界面错乱」——角色列表、地图选择、存读档、主菜单标题条整体偏移/挤压，但 `cargo test` 全绿。
- **根因**（P6 数据表迁移 4 处漂移，EXP-059/067/071 第 4 次变体）：
  1. **表值凭空重填**：P6 子代理把 ab0e5b5 硬编码坐标搬进 scenes.json 时未对拍旧值，`index.list.y=96` 行高/起点、`saveload.list.y=40` 等被重填漂移；
  2. **`item_rect` 语义漂移**：旧代码高亮矩形 = `fill_rect(x, 基线y - offset, w, item_h)`（文字在基线 y，高亮上移固定 `offset`），P6 误加 `-item_h/2` 居中性 → 高亮矩形整体下移 13px；
  3. **extra 解析器静默置 0**：`locked:{text_x:130}` 是标量字段而非 Rect 形状，`serde_json::from_value::<Rect>` 失败 → `unwrap_or_default()` → 坐标丢失，「已锁定」文字画到 x=0；
  4. **视觉基准自生成（循环论证）**：EXP-072 首版基准图由「已漂移代码」渲染生成，diff=0 只证明自洽不证明正确。
- **解法**：
  1. `ListLayout` 新增 `highlight_offset`（高亮上移像素）+ `gold_x`（次要列），废弃 `-item_h/2` 居中性；`item_text_pos` 回归「文字基线 y」语义；
  2. `SceneLayout.extra_scalar: BTreeMap<String,i32>` 抽取 `*_x`/`*_y` 标量，消费方 `extra_scalar.get("locked.text_x")`；
  3. `menu_shell` index 标题条回归 40px（旧视觉），`scenes.json index.title.h=40`；
  4. 视觉回归 `diff_frames_ui` **掩背景色**（RGB565=16 清屏色），只比「UI 像素分布」（高亮/文字/HUD 落在哪些逻辑像素），不比背景贴图——HD 化（`assets_hd_pack` 2x/4x PNG）与原版 PCX 背景必然不同，整帧 diff 会误报；基准图 = 回归旧视觉后重新生成。
- **关键澄清**（正向开发不矛盾）：渲染坐标系恒为 176×220 逻辑帧（`LOGICAL_WIDTH/HEIGHT`），HD/分辨率升级 = 贴图 2x/4x + `Viewport` 缩放上屏，**UI 布局坐标（fill_rect/draw_text 的 x,y）不随图片分辨率变**。数据表驱动 UI 坐标是正向架构，但**表值必须 == 旧验收视觉值**，子代理禁止把表当「新设计值」凭空重填。
- **飞轮**：数据表任何坐标迁移 → 先 `git show <旧commit>:<旧文件>` 对拍旧硬编码值 → 表值==旧值才合入；视觉基准图必须掩背景（锁 UI 排版层不锁贴图层），且基准由「回归旧视觉后的代码」生成而非「已漂移代码」自证。
- **验证**：`cargo test -p ti-app --test visual_regression` 5 项全绿（3 场景 UI 层 0 diff + 互异 + 确定性）；gate 第 18 项 PASS；scenes.json 逐场景 == ab0e5b5 硬编码坐标；`menu_shell` index 标题条 40px 实机确认。

### EXP-075 代码：HD 精灵「洋红框」= 资产软 alpha + blit 只判 a==0 + render_hd 漏调 blit 三重根因（视觉回归只锁 PCX 路未锁 HD 路）
- **现象**：native 窗口（`hd-8bit` 默认开）人物/立绘被洋红矩形包住，截图对比 ab0e5b5「对不上」。
- **根因（三重，缺一不可）**：
  1. **资产生成软 alpha**：`pack_hd_assets.py` 1x 档用 `Image.LANCZOS` 对 straight RGBA 直接缩采样，源图 `assets_hd/ui` 里 30.4 万 alpha=0 像素带着洋红 RGB（PCX 调色板背景色）被插值渗进邻域 → 产生 3.8 万 1..254 软 alpha + 1956 洋红像素；
  2. **blit 只判 `a == 0`**：`blit_sprite_rgba` 把 1..254 软 alpha 当不透明画上 → 洋红框；
  3. **`render_hd` 漏调 blit**：只 `sync_rgba_from_rgb565`（RGB565→RGBA8 扩色）不叠加注入的 HD PNG（`atlas.blit_scene_rgba` 从未被调），上屏画面退化成 RGB565 扩色（洋红脏边被扩到 8 位更刺眼）。
- **解法**：
  1. `pack_hd_assets.py` 加 `sanitize_alpha`（alpha 二值化 `>=128→255` + 透明区 RGB 清零），缩采样前后各调一次，重生成 `assets_hd_pack`（半透明 38666→0，洋红 1956→715）；
  2. `blit_sprite_rgba` 加 `ALPHA_TRANSPARENT_CUTOFF=128` 阈值判透明（`rgba[3] < 128` 跳过），F-311 不透明覆盖语义兜底；
  3. `render_hd` 补 `atlas.blit_scene_rgba(scene, &mut self.shell.fb_rgba)`，把 HD PNG 真正叠加到 RGBA 帧缓冲。
- **为什么一直没发现（流程缺口，必须说清）**：
  1. **视觉回归只锁了一条渲染路径**：EXP-072 的 `visual_regression.rs` 3 场景基准全用 `render_to_framebuf`（RGB565 `Framebuf`，PCX 精灵 blit 路），而 native 实机走的是 `render_hd` + `fb_rgba`（RGBA8 路）——**HD 路径的渲染像素从未被任何机器校验**；
  2. **HD 是 S7 新 feature，默认开但默认 feature 的实机路径零门禁**：`hd-8bit`/`gpu-render` 都是 Cargo feature 门控，gate 18 项里没有任何一项对「feature 开后的渲染输出」做像素级断言，全绿只代表 PCX 路对，不代表 HD 路对；
  3. **资产生成与 blit 是两处独立 bug，互相掩盖**：资产软 alpha（生成侧）+ blit 只判 a==0（渲染侧）+ render_hd 漏 blit（上屏侧）——三者各自独立，单修一个仍洋红；且 native 窗口是**人肉实机**才触发，headless 测试（`render_to_framebuf`）根本不进 `render_hd`，所以 32 轮 fuzz + 视觉回归 + gate 18 项全都绿着把这个 bug 漏到用户截图才发现；
  4. **「正向开发」假设没被像素验证**：S7 资产升级时假定「HD PNG 贴上去就对了」，但没人对 HD 路径做过一次「贴图后哪些像素是非背景色」的机器判定，假设直接进了默认构建。
- **飞轮**：任何 feature 门控的渲染路径（`hd-8bit`/`gpu-render`）新增时，必须同步给对应 `render_xxx` 加一条视觉基准或像素断言（与默认路径同判据）；`pack_hd_assets.py` 的 alpha 二值化由 doccheck2 `check_hd_sprite_transparency` 锚定，重跑 pack 后 1x 档半透明像素计数必须 = 0（验收判据写进 S13.5 台账 dod）。
- **验证**：headless 计数 `index/select_role/main_map` 洋红（rgba）= 0；`assets_hd_pack/1x/ui` 半透明总数 = 0（原 38666）；`cargo test -p ti-app --test visual_regression` 5 项全绿；doccheck2 第 12 项 `check_hd_sprite_transparency` 0 error；gate 18 项全绿。


### EXP-077 流程：无原版渲染 ground truth 时禁止向用户索要截图
- 现象：渲染问题（EXP-076 后续「横条盖徽章/贴图覆盖菜单/特定区域错位」）排查中，多次把「原版截图/原版稳态画面」当成可行证据来源向用户索要；用户反复指出「几十年的游戏去哪运行」「没有原版截图」。
- 根因：
  1. 没先确认「原版能否在本机运行」这一硬约束——原版 `TreasureIsl.exe` 是 **ARM PE32 (machine=0x01c0) + subsystem=4 + 导入 coredll** 的 Windows CE/Mobile 程序，本机 x64 Windows 无 QEMU/ARM 工具链、无 WinCE 镜像，**根本跑不起来**；
  2. 在零 ground truth 下仍用「静态反推 + 子代理审计」求解渲染位置，伪代码里 IDA 吞寄存器（a4/v34）导致二义，子代理结论互相矛盾却无法裁决；
  3. 用户已明确「你不行就调 REA 来做」「出流水线对抗 10000 次必须找到问题」，但本会话工具列表无 `rea_*`，未第一时间声明「REA 不可用，只能静态」，继续凑数。
- 解法（以后遇到「渲染位置/视觉错位」类问题，按序执行，禁止跳步）：
  1. **先声明 ground truth 现状**：本机无原版运行能力（ARM WinCE exe 跑不了），无原版截图，无 QEMU 镜像——把这一事实写进本条 EXP 与 `docs/process/07_render_pipeline.md`；
  2. **禁止向用户索要「原版截图/原版稳态画面」**（用户没有，且这是几十年前的 ARM WinCE 游戏）；
  3. **替代证据链**（按可行性排序）：
     - (a) **用户指认偏移量**：用户看我渲染的 PNG，指出「哪个精灵朝哪个方向偏了多少像素」——我据此反推是坐标映射/PCX 宽高/网格层/z-order 哪一条；
     - (b) **像素级对比工具**：`scripts/compare_render.py` 输入「用户提供的任何参考图 + 我的渲染 PNG」，自动 diff 报告（哪些区域不一致、偏移多少、疑似哪条精灵）；
     - (c) **QEMU 系统模式**：需 ARM WinCE 镜像（NK.bin + 文件系统），本机没有——若用户能提供镜像，用 `qemu-system-arm` 跑原版 dump 帧缓冲；
  4. **补网格层**（`sub_443FC` 行 30-103，`a1[247]`）：这是坐实缺失的渲染层，无论「位置不对」根因是什么都该补——原版先画 `a1[248]`列×`a1[249]`行 背景 tile（底层），再画控件（顶层）；Rust `blit_scene_frame` 只画控件链。
- 验证：本条 EXP 登记；后续渲染问题排查不再出现「请提供原版截图」类提问；网格层补入后 `blit_scene_frame` 输出与原版 `sub_443FC` 两段结构一致。





### EXP-078 流程：并行半成品致 gate FAIL 时，先归因再决定能否转 verified
- **现象**：收口 S10.2 时 gate 4 项 FAIL（fmt/clippy/no placeholders/check_hardcode），但 4 项命中文件全是工作区**未提交的并行任务半成品**（sprite_step.rs 编译错 E0061、e2e_render_alignment.rs 占位符、render_snapshot.rs/sprite_atlas.rs 既有 warning），非本轮 S10.2 改动引起。
- **根因**：多任务并行开发共用一个工作区，前序任务（sprite_step/e2e_render_alignment）改完未提交，其半成品污染 gate 全局扫描；本轮任务若盲目「gate 全绿就转 verified」会把自己的改动与并行半成品混在一起提交，违反 AGENTS.md 硬规则 1/2。
- **解法**：(a) 用 git stash 基线对照法归因——stash 本轮改动跑 gate，凡 stash 后仍 FAIL 的项即既有/并行任务遗留，stash 后新出现的项才是本轮引入；(b) 本轮引入项必须修干净（fmt/clippy）；(c) 并行半成品项**不替别人修**，在 evidence log 里写明归因（哪些 FAIL 命中哪些未提交文件、哪个并行任务），台账维持 in_progress，待并行任务收口重跑 gate 全绿后再转 verified；(d) commit 只暂存本轮改的文件（git add 显式列文件，不用 git add -A），不裹挟并行半成品。
- **验证**：git stash 前后 gate FAIL 项集合 diff + evidence/S10.2.log 末尾「gate FAIL 归因（非 S10.2 引起）」章节 + 台账 S10.2 维持 in_progress。

### EXP-088 流程：机制 ≠ 规则 —— `unlocked_maps` 有字段/UI 置灰但无「打 BOSS 解锁下一图」推进规则；定稿跨图前必须查「规则是否存在」
- **现象**：用户问「我们有『跨图推进』游戏业务吗」。排查发现：`unlocked_maps`（存档字段）+ `game_app.rs` 的 `map_unlocked()` 校验 + SelectMap 置灰是**机制**，但开档时 `game_app.rs:535` 把**三张图全部解锁**（`MapSlot::all().to_vec()`），全库**没有任何代码**会往 `unlocked_maps` 里加图，`docs/business` 也没有「击败 BOSS → 解锁下一张图」的规则 —— 「打满 3 图通关」从未被定义。
- **根因**：数据字段 / UI 校验存在 ≠ 业务规则存在；把「有字段」当成「有玩法」就会凭空实现一条规则（= 臆造业务）。机制（能力）与规则（玩法定义）必须分开核对。
- **解法**：「落地」任何业务前先分机制 / 规则两步：(a) grep 该字段的**写入端**（`.push` / 增长逻辑），无增长 = 规则不存在；(b) 把规则定稿进 `docs/business/progression.md`「跨图推进」节（初始仅 m01 / 击败 BOSS 解锁下一图 / 击败 m03 = 通关 / 失败不解锁可重试，附 6 条可测试规则）；(c) 规则层落地 `ti-core/src/rules/progression.rs`（`ProgressionState` 值语义 + 6 单测 + `cross_map_demo` 打满 3 图 demo）；(d) **app 层接线**（`game_app.rs:535` → 初始仅 m01 + 击败增长 `unlocked_maps`）属表现/渲染线，是独立任务，不臆造、单独登记。
- **验证**：`rules::progression` 6 单测绿；`cross_map_demo --seed 3` 跑出 m01 胜→解锁 m02→m02 胜→解锁 m03→m03 胜→通关；`grep "unlocked_maps" crates/ti-app/src/game_app.rs` 开档仍是 `MapSlot::all()`（app 层未接线 = 残留，已注明归属渲染线）。
- **闭环（2026-10）**：app 层接线已落地（`game_app.rs` 开档 = `ProgressionState::initial_unlocked()` 仅 m01；`battle_step` 胜利钩规则层结算解锁；`restore_save` 按 `min_progression_to_select` 链恢复，`SaveSlot` schema 不变）；验收 `crates/ti-app/tests/app_progression_wiring.rs` 4 条全绿。
- **口径更新（第 12 轮审计后）**：规则表 6 → 8 行、单测 6 → 10（新增 `from_unlocked` / `min_progression_to_select` 两条 API + 4 单测，`progression.md` §可测试规则 为单一事实源）。

### EXP-089 流程：「跑多少次都一样」是回归测试的**正确**行为；要展示「每次都不同」用新种子 demo
- **现象**：用户质疑 `cargo test exhaust_random_session`「这运行多少次都是一样的」。实情：它是**固定顶种子**的回归测试，「次次相同 = 可复现」是设计目标（稳定抓回归）；「想看每次都不同」属于 **demo** 诉求。另有两处误判引发质疑：把 m01 起点→BOSS 误报为 20 步（实为 Manhattan 10 = 5+5）；把「测试里的有界随机抽样」说成「一局完整游戏」（用户问「10 步就结束了？」）。
- **根因**：没区分「回归（要可复现）」与「演示（要变）」两种验证目的，用固定种子回归测试去证明「随机性 / 非固定」必然「次次相同」；且关键数值（步数）凭估算断言而未实测。
- **解法**：(a) 回归测试**保持固定种子**（确定性、可复现），「非固定 / 非脚本化」用**选择指纹 distinct**（400 会话 distinct=366）+ **受控变异**（把选择点临时写死常量 → 多样性断言 `distinct>=100` 变红）证明；(b) 「每次不同」交给**新种子 demo**：`random_session_demo` / `cross_map_demo` 无参时取 SystemTime 纳秒做种子（每次开一个新会话），带 `--seed N` 才复现；(c) 定稿规则 / 断言前先**实测数值**（m01=10 步、m03=6 步，不是 20 步），不靠猜（AGENTS.md 硬规则 4「结论必须实测」）。
- **验证**：`random_session_demo` 连跑 3 次（无参）→ 3 个不同种子 / 3 个不同会话（2431724108 panda/m02、2595131308 katharina/m01、2754479208 kidd/m02）；受控变异（固定 4 个选择点为常量）使 `exhaust_random_session` 的 `distinct>=100` 断言失败，还原后回绿。

### EXP-092 流程：已提交的乱码（UTF-8→GBK 误读）文件恢复 = 找最后干净 commit + 拼接干净新段；**绝不**逆向「解码回去」
- **现象**：`docs/experience/process.md` 标题 + 前 16 条（EXP-007~065）整段被 UTF-8 字节误读为 CP936(GBK) 再存回 UTF-8 成乱码（「经验库」→「缇忛獙搴」），且**已提交进 git 历史**（`838ca016c` 起）；文件呈「前段乱码 + 后段（EXP-068~089）干净」混合态。
- **根因**：早期会话用 PowerShell `Get-Content|Set-Content`（默认 GBK 解码 UTF-8，EXP-051/086 家族）重写文件，把乱码 commit 进了仓库；后续会话又直接向乱码文件追加干净新条目，造成混合态。
- **解法（恢复法，可复用）**：
  1. **不要逆向转换乱码**——mojibake 有损（`?`/`€` 等字节 GBK 无映射，反解必出 U+FFFD），别指望「解码回去」。
  2. **找最后干净 commit**：`git log -- <file>` + `git show <sha>:<path>`（**cmd 直写原始字节，勿经 PowerShell 重编码**）抓 blob，判标题字节是否干净 UTF-8（「经验库」= `E7 BB 8F E9 AA 8C E5 BA 93`）；本例干净基线 = `3630693bd`（前 16 条全干净）。
  3. **拼干净新段**：混合态里「乱码后追加」的干净段（本例 `### EXP-068` 起到结尾）直接从当前工作树截取。
  4. **重建** = 干净基线（LF→CRLF 归一 + 去 BOM）+ 单空行 + 干净新段；用 `[System.IO.File]::WriteAllBytes` 写回（**绝不用 `Set-Content`**，EXP-051）。
  5. **验证**：条目数、`mojibakeHits=0`、标题含「经验库」、无重复 ID、`doccheck` 0 错误。
- **验证**：`process.md` 已恢复（25 条全干净，`doccheck` 0 错误）；乱码原版先备份临时目录再覆盖。
