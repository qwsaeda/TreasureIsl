# 经验库 · process

> 注：本文的「基线」记述属**历史**：像素基线机制已于 **S18.7b 退役**（三套 golden PNG + 三个 diff 测试均已删除）；
> 现行机制 = 结构不变式断言 + CPU↔Bevy 内存对拍。

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
     + 开轮必读 7 条（S51.0 起；EXP-070 原 6 条 + 第 7 条开轮协作五步）+ 飞轮回写规则 + 停机条件；
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
- **验证**：`powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Package P6` 输出含 7 条必读（S51.0 起；EXP-070 原 6 条 + 开轮协作五步）
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

### EXP-099 流程：`required-features` 门控的 bin 不被 gate 编译/lint ⇒ 官方排障工具静默失效
- **现象**：`crates/ti-app/src/bin/element_dump.rs:119-124` 的 `find(|&((zx0,zy0,zx1,zy1),_)| ...)`
  因 `Iterator::find` 传入 `&&T` 而把绑定推成 `&u32`，**4 × E0308 编译失败**；而
  `scripts/gate.ps1` 连续多轮 **22 项全绿**。`element_dump` 是 AGENTS.md 路由表登记的
  「报『特定区域贴图错位』先逐元素定位」官方工具——排障时才发现它根本编不过。
- **根因**：`treasure-native` / `treasure-headless` / `element-dump` / `sprite-step` / `render-png`
  五个 bin 都声明了 `required-features = ["native-shell" / "headless"]`。gate 第 1~3 项
  （`cargo fmt --check` / `cargo clippy --workspace --all-targets` / `cargo test --workspace`）
  **不带 feature**，cargo 会直接跳过这些 target ⇒ 它们长期处于「不在编译器视野内」状态。
  这是**盲区**而非 bug：门禁绿 ≠ 全仓可编译。（同类根因：EXP-052 讲门禁扫描边界、
  EXP-048 讲「机械可判定项必须固化进门禁」。）
- **解法**：
  1. 修 `element_dump.rs` 的解构（去掉 `&` 前缀，改 `zone.0` / `map_or("其他", |z| z.1)`）；
  2. 新增 **gate 第 4b 项**：`cargo clippy --workspace --all-targets
     --features ti-app/native-shell,ti-app/headless -- -D warnings`，把全部 feature 门控 bin
     纳入编译 + lint（`--all-targets` 才能覆盖 bin，`check` 不足以抓 clippy lint）；
  3. 同批修掉该命令暴露的 5 处存量问题：`sprite_step.rs` 的 `println!` 格式串
     `{0}` 缺占位符（会 panic）、`treasure_native.rs` 悬空 doc 注释 + f64→u64 cast、
     `treasure_headless.rs` `if_not_else`、`render_png.rs` 6 处 pedantic cast。
- **验证**：`cargo clippy --workspace --all-targets --features ti-app/native-shell,ti-app/headless -- -D warnings` 0 error；
  `cargo check -p ti-app --features native-shell --bin element-dump` 0 error；
  gate 第 4b 项 PASS，`docs/parity.md` 计数口径同步 21 → 22（`doccheck2::check_gate_item_count` 对账）。
### EXP-100 流程/代码：脚本化批量改 import 的正则陷阱——前缀带尾 `::` + `+` 漏单段模块
- **现象**：飞轮 F17 要把 43 处非 use 行 `ti_core::a::b::X` 收敛为顶部 `use` + `b::X`。
  脚本用 `re.compile(r'ti_core::((?:[a-z_0-9]+::)+[a-z_0-9]+)::')` 收集模块前缀：
  ① `(?:…::)+` 用 `+` 要求至少两段模块 → **单段模块 `ti_core::data::` 一处都匹配不到**；
  ② 前缀捕获带**尾 `::`** 时 `prefix.split('::')[-1]` 得**空串** → 替换成空字符串，
  把 `ti_core::rules::cards::CardState` 削成裸 `CardState`，43 处**全部编译失败**
  （E0425/E0422/E0433 一片）。另两次修复陷阱：③ 插入的 `use` 落在**函数/测试体内**
  而非文件顶部 import 区（`touch.rs` 的 `layout`/`tests` 嵌套 mod 还要各自 `use super::`）；
  ④ 新断言扫到自己源码里的字符串字面量，需排除断言文件本身。
- **根因**：正则捕获组设计未考虑「模块路径段数可变」与「尾分隔符归属」；批量文本改写
  没有把「编译器」当唯一裁判（本轮第一次跑就编译不过，说明缺少**逐文件 `cargo check`** 环节）。
- **正解**：捕获组用 `(?:[a-z_0-9]+::)*`（`*` 允许零次，即支持单段）；别名取**去掉尾
  `::` 后的** `split('::')[-1]`；插入点锚定**文件顶部首个 `use` 区**，嵌套 `mod` 补
  `use super::`；断言类检查排除自身实现；**改完立刻逐文件 `cargo check` + 最后全量
  `clippy --all-targets`（含 feature 门控 bin）双跑**，让编译器逐处判刑。
- **验证**：`cargo check -p ti-app --features native-shell,headless` 0 error；
  `cargo test --workspace` 0 failed；新增 doccheck2 第 22 项 `check_no_ti_core_fullpath`
  断言该风格不再回潮（本次命中 0）；gate 22 项全绿。

### EXP-101 流程/构建：`required-features` bin 只在「目标+feature 双命中」时被编译，裸 target 构建是门禁盲区
- **现象**：S15.10 期间 `build_wasm.ps1` 首跑爆 E0502（`treasure_wasm.rs::play_last_sfx`
  的 `last_sfx()` 可变借用与 `menu_selected()` 不可变借用冲突）；修完编译后又连爆
  wasm-shell 组合独享的 3 处旧伤（`cycle_quality` must_use 未消费、`load_ttf_from_bytes`
  按值传 `Vec` 未消费、`f64→u64` 截断在 wasm32 才触发 `cast_possible_truncation`）。
  这些全部**从未被 gate 拦住**：第 4 项 `cargo build --workspace --target wasm32-unknown-unknown`
  因 `treasure-wasm` 带 `required-features = ["wasm-shell"]` 被 cargo 直接跳过；
  第 4b 项 clippy 组合只覆盖 `native-shell,headless`。4 个文件在 wasm 端静默烂了多轮。
- **根因**：门禁对「feature 门控 bin」的覆盖 = 各 clippy feature 组合的**手工并集**
  （EXP-099 补了 native 侧，S15.9 补了 hd-8bit），但没有任何机制保证「**每个 bin 的
  required-features 都被某条 gate 行编译到**」。目标特有 bin（wasm32 的 treasure-wasm）
  天然双盲区：裸 target build 跳过它（required-features 不命中）；native clippy 组合
  不含它（wasm-shell 不在组合里）。wasm32 平台类型宽度还不同（`u64` 非默认宽度 →
  截断 lint 只在 wasm 触发），同一行代码 native 绿 wasm 红。
- **解法**：① gate 第 4 项 wasm 分支并入
  `cargo clippy --target wasm32-unknown-unknown -p ti-app --features wasm-shell
  --all-targets -- -D warnings`，一行报 build+clippy 复合（项数不变，gate 仍 22 项）；
  ② 4 处旧伤就地修复：借用改 `last_sfx().is_some()` 前置再读菜单、`Vec<u8>`→`&[u8]`、
  `cycle_quality` 对齐 native 的 `let _ =`、截断用 native 同款
  `#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]`；
  ③ 纪律（写进 gate.ps1 第 4b 注释）：**新增/改 feature 门控 bin 时，其
  required-features 必须同步进某条 gate 行**，否则等于没门禁。
- **验证**：`cargo clippy --target wasm32-unknown-unknown -p ti-app --features wasm-shell
  --all-targets -- -D warnings` 0 error（修复前 3 连 error）；gate 第 4 项改复合判定后
  全绿；`build_wasm.ps1` 由 FAIL 转 PASS（产出 web/ 正常）。

### EXP-107 流程/构建：交付壳换引擎时，「未被任何门禁行编译的产物」会静默烂掉（S18.8a 的 android 壳盲区 + Bevy 壳 wasm 分支首次纳入 lint）
- **现象 1（既有盲区，本切片发现）**：android 壳（`treasure_android.rs` + `android_shell.rs`）
  **在 S18.8a 改动前就已编译不过** —— `android_shell.rs` 仍调 `hd_manifest::load_hd_pngs_from_manifest`，
  而该函数早已改名 `collect_hd_pngs_sync`（同步闭包版）。它长期没被发现，因为该 bin 带
  `required-features = ["android-shell"]`，而门禁的 android 构建行 `cargo build -p ti-app
  --target aarch64-linux-android` **不开 feature** → cargo 直接跳过该 bin → 门禁全绿。
  同型盲区 = EXP-099 / EXP-101 的又一次复发。
- **现象 2（新盲区，本切片首次暴露）**：Bevy 交付壳（`crates/ti-engine`，**独立 workspace**）
  不在根 workspace 的 `--workspace` 覆盖范围内 → 它的 **wasm 目标分支从未被 clippy 检查过**。
  S18.8a 把门禁 wasm 项改指 `treasure-bevy-wasm`（build + `clippy -D warnings`）后，一次暴露 5 处：
  `parse_font_bytes` import 在 wasm 分支未用、`FrameTex` 资源与 `rgb565_to_rgba8` 函数在 S18.7a
  （CPU 帧缓冲路径退役）后成死代码、`filter_map(..).next()` clippy 告警、`spawn_draw_list` 的
  `needless_pass_by_value`。
- **根因**：门禁对「交付产物」的覆盖 = **手工维护的（target × feature）并集**，没有任何机制保证
  「每个交付产物都被某条 gate 行编译 + lint」。引擎/壳迁移（S18 双渲染期 → 单引擎）时，
  新产物落在**新 workspace** 里，最容易整块掉出覆盖；而被删的旧产物留下的
  `required-features` bin 又没人再开 feature 去编。
- **解法**：① 门禁 wasm 项**直接指向交付产物**（`--manifest-path crates/ti-engine/Cargo.toml
  --bin treasure-bevy-wasm`，release 构建 + `clippy -D warnings`），交付壳从此有 lint 覆盖；
  ② 死代码当场清理（`FrameTex` / 占位纹理 / `rgb565_to_rgba8`：S18.7a 后已无 CPU 帧缓冲上传路径）；
  ③ 门禁注释 + `docs/parity.md` 显式标注 android 壳盲区（该壳随 S18.8d 退役，不在本切片修）；
  ④ 台账三方对账的符号期望随代码退役迁移（`check_ledger_reality.py` 的 S10.4 期望
  `pub fn load_ttf` → `pub fn from_bytes`，并写明迁到 `ti_render::font::TtfFont` 的原因）。
- **验证**：gate 22 项 0 failure（wasm 项 PASS = Bevy wasm 构建 + clippy 双绿）；
  `--selftest` 16/16 步 0 失败；`--scene=index --shot` 非黑像素 2072562 与 S18.7h 实测**逐位一致**
  （`evidence/bevy_native_index.png` 无 diff = 删死代码后画面不变）。
- **纪律（写进 gate.ps1 第 4b 注释）**：**新增/迁移交付产物时，必须同时给它的
  （target × feature）组合加一条门禁行**；被删壳留下的 `required-features` bin 要么有门禁行覆盖，
  要么在同一批次内删掉 —— 否则等于没门禁。

### EXP-108 流程/门禁：检查的行识别条件写错 → 恒不触发 = 一个不存在的门禁（分卷条数对账从未生效）
- **现象**：S18.8a 修 EXP-102 ID 撞号时顺手核对经验库索引，发现 doccheck2 第 17 项
  「经验库分卷条数对账」（`check_experience_volume_counts`）**自落地起一次都没真正执行过**：
  它用 `cols[0].contains(".md](")` 判定「这是分卷表行」，而分卷表首列实际写作
  ``[`code.md`](code.md)`` —— "md" 与 "](" 之间还有一个**反引号** → 子串 ".md](" 永不成立
  → 每行都 `continue` → 该检查恒返回 0 error。后果是分卷表「条数」列长期漂移无人发现：
  `code.md` 声明 28 / 实际 33（+5）、`tools.md` 声明 12 / 实际 13（+1），
  全量索引表还缺 6 行（EXP-049 / EXP-050 / EXP-062 / EXP-102~EXP-106）。
- **根因**：检查的「行识别」用手写子串而非「解析出字段再验证」；条件写错时**静默跳过全部输入**
  （不报错、不告警），而门禁只统计 error 数 → 一个恒不触发的检查 = 一个不存在的门禁，
  还会给人虚假安全感（比没有检查更危险）。这与 EXP-099 / EXP-101 / EXP-107 同族，
  但更隐蔽：前几次是「某些组合没覆盖」，这次是**一次都没执行过**。
- **解法**：① 判据从 `.md](` 改为 `](`（首列含 markdown 链接即候选）+ 解析出的文件名必须存在，
  解析失败即报错（不再静默跳过）；② 补齐漂移：`code.md` 28→33、`tools.md` 12→13、
  索引表补 6 行 + 修 3 处卷标签（EXP-059/061/074 的正文在 process.md）；③ **负测纪律**：
  新写的门禁检查上线前必须**故意破坏一次**（把判据写错 / 注入已知坏数据）看它是否报红，
  否则等于没写 —— 负测就是门禁检查的单元测试。
- **验证**：把 `code.md` 的声明条数改成 99 → doccheck2 立刻报
  「分卷表条数列：code.md 声明 99，实际 33（Q016 类漂移…）」→ 改回 33 后 0 error；
  gate 22 项 0 failure。

### EXP-114 流程/UI：视觉改动缺「设计稿确认」门禁 ⇒ 返工全堆在收口阶段（S20.2 先落码后出图）
- **现象**：S20.2 v4 面板语言（深棕木底 + 黄铜细边 + 选中态 3px 黄铜边，message/saveload/select_role 三屏）
  直接落码——scenes.json 坐标调整 + chrome.rs 新原语 + theme 新令牌全部进代码，
  设计效果图（三屏离屏出图）到**收口时才给用户看**；
- **根因**：变更面声明只门控「改哪一块」，没门控「这个视觉方向是否被用户认可」。
  视觉意图未经用户过目就落码 ⇒ 一旦方向要调，返工成本 = 6 项 gate 失败（编译/clippy/三处 doc 同步）
  + 3 条 render_intent 判据重锚 + 文档镜像 4 处，全在收口阶段付；
- **解法**（A 纪律，S20.3 固化）：① `00_index.md` 新增 §「UI 视觉改动：设计稿先行」（三步：
  草案 → 设计稿/效果图用户确认（消息留痕）→ 落码 + `ui_preview` 验收闭环）；
  ② AGENTS.md 硬规则 9；③ 任务 dod 必含「用户视觉验收通过（消息留痕）」（同 S17.1/S18.5 口径）；
  确认后改坐标/配色 = 新视觉意图，重新过目；
- **验证**：S20.3 固化后 gate 全绿；此后 UI 视觉任务无设计稿确认记录 = 任务退回（§分工表 纪律 5）。


### EXP-141 流程/设计：AI 概念稿迭看「逐元素修补」低效 + 图生图 ref 缩太小角色仍漂移（main_map v6 / VS v2，11 轮未收敛）
- **现象**：main_map v6 + VS 战斗动画 v2 AI 概念稿迭看 11 轮，用户每次仍给「再调整」：
  岛大小/贴环、格子立体感、VS 对角构图、人物统一、失败界面、字体动感、随机拼凑感……逐轮单点补；
  用户中途明确「你能不能总体去设计，别顾脸顾屁股」；
- **根因**：① 设计迭代用「用户逐条反馈 → 我改 prompt 单点 → 重生成」逐元素修补，
  单帧改好另一帧又漂（VS clash/duel/result 人物始终对不齐）；AI 无跨图约束，除非单帧内稳定锚；
  ② 图生图 ref（`--image`）缩到 100px（<128KB 单参数上限）后 AI 只当弱参考，
  人物装扮/脸仍自由发挥 ⇒ 「人物又变了」反复出现；
  ③ 格子语义根本分歧未先对齐：我理解「浮海石柱」用户要「平面棋盘格（玩家站格）」
  ——第 10 轮用户才点破「你知道格子的作用不，玩家在上面」；
- **解法**：① 设计迭代第一步 = **把核心意图写成约束清单（先与用户对齐语义）**，
  再一次性整体出图，不逐元素补；② 角色一致性靠「单帧稳定锚」而非跨帧 ref：
  ref 缩图 ≥150px（或 JPEG 压缩到 <128KB）且 prompt 强约束装扮词（三角帽/白衬衫/破披风/少年漫大眼），
  或直接一帧产出后裁剪该角色作后续帧 ref；③ 格子语义 = 大富翁式平面棋盘格（玩家站格），
  每格形状差异大（随机拼凑感），格间海水，环外海洋；
- **验证**：EXP 回写；后续 AI 概念稿迭看按「意图清单先行 + 整体出图」重做。

### EXP-142 流程/设计：设计规范「只有意图没有可执行规格」——几十轮要求未真总结，对抗子代理审计抓出（main_map v6 / VS v2）

- **现象**：用户 2026-10-31「我几十轮的要求规范都总结文档了没有？界面和图片资源设计很重要，但你的规范都是屎，开流水线对抗子代理去总结规范」。
  检查 `12_main_map_v6_vs_v2_spec.md`：仅 86 行，VS v2 部分 11 行。
- **根因**：规范停在「文字描述层」——有意图（羊皮底/铜牌/岛/生物/4 帧），但：
  ① 无设计流程方法论（用户最新拍板「元素级生成+组合合成」未写入）；
  ② 无尺寸/坐标基准（除 4 个色值外全无数字，环半径/格径/岛 620px/舵轮罗盘尺寸均缺）；
  ③ 无资产引用（`assets/ui_batches/*.json`、`elements/*.png`、`vs_anim_v2/` 全未引）；
  ④ 无战斗类型分层（攻占/副本/BOSS/遭遇/事件 5 类，`combat.md` 有表但 12 spec 未接）；
  ⑤ 无机器验收判据（只有人工勾选待确认项）；⑥ 自称「终稿/最终契约」却仍待用户确认（自相矛盾）。
- **解法**：① 对抗子代理双审计（FIND-AUDIT 对照需求摘要 + 诊断 VS 重设计）抓出 8 类缺口；
  ② 重写规范 = 加 §1.0 设计流程方法论 + 全篇补尺寸/坐标/资产引用 + §2.4 战斗类型分层 +
  §2.5 兵力 HUD 数据绑定 + §2.6 时间轴 + §4.2 机器验收判据表；③ VS 旧概念稿（AI 插画海报）已否决，
  按 R1~R10（敌我分列 + 顶/底状态框 + 中央战场 + 结果横幅，参考 FGO/宝可梦/Pirates/黑旗/炉石）重设计，
  且「引擎真实渲染为准，AI 概念稿只作方向参考」。
- **验证**：EXP 回写；后续规范新增必带尺寸/资产/判据，设计迭代按「意图清单先行 + 元素级生成 + 组合合成」。
  **重写后对抗审计（飞轮主线程复核）**：8 缺口已全补，但重写引入 6 项一致性问题（creatures 未合成 /
  判据「中心距≥格径」几何不可满足 / 每局随机 19/12 与业务固定 20/10 冲突 / vs_theme 误引（schema 未定）/
  combat.md 事实源链断 / ship_player 误归 vs_v1），**主线程复核后已全修**：TILE_BASE 84→78（85.0<89.43 判据可满足）/
  格型按 `territory.md` §2 固定 20/10/1（每局随机=归属非格型）/ vs_theme 标「待扩展」/ combat.md 交叉引用本规范 §2 /
  ship_player 改引 `ui_manifest.json`+`main_map_v4.json`/ creatures 改口「ocean_bg 烘焙 + 动画层元素源」；
  doccheck 0 error。

### EXP-143 流程：迭看图/引擎渲染图归档规范（防平台看不到 + 防乱放）
教训：用户迭看「生图在平台看不到」「图片乱放都找不到」。根因 = ①我把正式设计稿放 evidence/ui_concept/{vs_anim_v2,main_map_v6}/ 子目录（平台 GET /api/evidence?sub=ui_concept 只枚举顶层一层，子目录稿不可见）；②新开 evidence/vs_v2_render/ 目录放引擎渲染迭看图（无归档规范）。修复 = 13_ui_asset_pipeline.md §2.1a 定案：迭看图/引擎渲染图统一落 ui_concept/ 顶层，命名 <theme>_<variant>.png（合成稿）/ <theme>_<variant>_engine.png（引擎渲染），AI 元素源仅子目录（raw_/中间产物），写后必跑 ui_thumb.py 生成侧车；散落图已归位 + 侧车生成。规范：写迭看图前查 §2.1a 命名表；写后跑 ui_thumb.py；不新开顶层目录。

### EXP-150 流程：三键取掉 = 删原语而非改覆盖逻辑
## 三键取掉 = 删原语而非改覆盖逻辑

**现象**：用户报「主地图底部三键（建造/贸易/出售）没有取掉」。

**根因链**：`RawScene` 用 `#[serde(flatten)] extra` → `deck_trade/deck_sell/deck_build` 顶层键进 `extra`；
main_map.rs 的「覆盖为羊皮底」逻辑读 `layout.extra.get(DECK_*)`，但**三键视觉 = scenes.json `primitives[]`
的 deck_plaque 图片原语（z=8）**，渲染顺序 = 羊皮底（先画）→ `render_primitives_filtered(min_z=3)`
后画盖住 ⇒ 三键可见。v6 裁决「不常驻」的实现失效（覆盖层被原语盖住）。

**教训**：UI 元素「去掉」= 删**数据原语**（scenes.json `primitives[]`），不是加覆盖层盖住——覆盖层会被
z 更高的原语反盖。删原语后同步：①`on_hit` 的 pick_key 5/6/7 死分支 ②`open_build_panel` 死函数
③`v6_deck_popup_note` 孤儿字符串键 ④测试断言（`main_map_pick_e2e` 的 5/6/7 应 0 个）。

**验证**：删 9 原语（3 plaque + 3 icon + 3 text）→ `--shot main_map` 出图底部只剩舵轮/罗盘；
`main_map_pick_e2e` 断言 5/6/7=0 全过。

### EXP-151 流程：三键取掉 = 删原语而非改覆盖逻辑
## 三键取掉 = 删原语而非改覆盖逻辑

**现象**：用户报「主地图底部三键（建造/贸易/出售）没有取掉」。

**根因链**：`RawScene` 用 `#[serde(flatten)] extra` → `deck_trade/deck_sell/deck_build` 顶层键进 `extra`；
main_map.rs 的「覆盖为羊皮底」逻辑读 `layout.extra.get(DECK_*)`，但**三键视觉 = scenes.json `primitives[]`
的 deck_plaque 图片原语（z=8）**，渲染顺序 = 羊皮底（先画）→ `render_primitives_filtered(min_z=3)`
后画盖住 ⇒ 三键可见。v6 裁决「不常驻」的实现失效（覆盖层被原语盖住）。

**教训**：UI 元素「去掉」= 删**数据原语**（scenes.json `primitives[]`），不是加覆盖层盖住——覆盖层会被
z 更高的原语反盖。删原语后同步：①`on_hit` 的 pick_key 5/6/7 死分支 ②`open_build_panel` 死函数
③`v6_deck_popup_note` 孤儿字符串键 ④测试断言（`main_map_pick_e2e` 的 5/6/7 应 0 个）。

**验证**：删 9 原语（3 plaque + 3 icon + 3 text）→ `--shot main_map` 出图底部只剩舵轮/罗盘；
`main_map_pick_e2e` 断言 5/6/7=0 全过。

### EXP-152 流程：wasm 背景音乐无声 = 浏览器 autoplay policy
## wasm 背景音乐无声 = 浏览器 autoplay policy（需用户手势解锁）

**现象**：用户报「背景音乐没有（多平台）」。

**根因**：bevy_audio 0.20 `spawn_audio` 在 Startup spawn BGM → cpal 0.17 wasm `Stream::play()`
调 `AudioContext.resume()`，但 Startup 无用户手势被浏览器拒（autoplay policy）⇒ wasm BGM 无声。
native 逻辑正确（bevy_audio + ALSA，无声卡时静默降级）。

**教训**：wasm 音频**不能**在 Startup 直接播——浏览器要求 `AudioContext.resume()` 在用户手势
（sticky activation）之后。bevy_audio 0.20 无内置 wasm autoplay 处理（`AudioOutput.stream` 私有，
无 resume API）；cpal 0.17 的 `Stream::play()` 会调 resume，但需手势。

**修法**：`spawn_audio` 在 wasm 下不直接 spawn BGM（插 `BgmUnlock` 资源），`unlock_bgm_system`
（Update）等首次 pointer/keyboard `get_just_pressed()` 后 spawn BGM（resume 成功）。native 照旧直播。

**验证**：`unlock_bgm_system` wasm 编译通过（`cargo build --target wasm32-unknown-unknown`）；
native clippy 全绿。注意 bevy 0.20 `ButtonInput::any_just_pressed()` 需参数，改 `get_just_pressed().next()`。

### EXP-170 流程：判据载体缺失 ⇒ 门禁项永久 SKIP ⇒ 判据从未执行
**现象**：parity.md P0-2「每个格式解析器 0 未知字节」的判据命令 `cargo run -p ti-tools -- verify --zero-unknown`
**从未存在**——`crates/ti-tools/src/bin/verify.rs` 在 git 全历史中从未出现（`git log --all -- crates/ti-tools/src/bin/verify.rs` 为空），
而门禁第 7 项写成 `if ((Test-Path 'crates/ti-tools/src/bin/verify.rs') -and (Tier 'late'))` 的**载体存在性守卫**
⇒ 载体缺失 = 永久 SKIP ⇒ 判据**从未执行过**。S0~S2 计划（`docs/plan-s0-s2.md` §DoD）把 `verify --zero-unknown` 写进
阶段 DoD，但**未创建对应台账任务** ⇒ 无任务承载 ⇒ 载体永不落地 ⇒ 项目 DoD「P0~P4 全 pass」中 P0-2 长期不可达，
而台账 308/308 `verified` 掩盖了该缺口（验收集合 ≠ 判据集合）。

**根因（3 条）**：① 门禁项用「载体存在性」当条件，载体缺失时**静默降级 SKIP**（而非 FAIL）⇒ 缺失被伪装成「未解锁」；
② 判据命令**从未实跑过**——载体落地后首次实跑即暴露：`cargo run -p ti-tools -- verify --zero-unknown` 在
「多 bin 且无 `default-run`」的 crate 上被 cargo 直接拒绝（`could not determine which binary to run`），须写
`--bin verify --`；③ 计划里的判据命令**未落台账任务** ⇒ 无归属、无排期、永不实现。

**对策（S46.0 落地）**：① 载体 = `crates/ti-tools/src/bin/verify.rs`：遍历 `legacy/data/` 真实文件逐族解析
（UI 场景 `.Map` / 地形图 `glmapm*.Map` / `GoldLand.Fnt` / `font.dat` / `settings.dat` / 容器 `.pak`+`.lpk` 逐条目 + PCX 条目解码）；
口径 = **结构后无残留字节**（`define_format!` 生成的 `read` 对残留报 `TrailingBytes` ⇒ **解析成功即达标**，不需额外统计逻辑）；
实测 **33 文件 / 305 容器条目 / 0 跳过 / 0 失败** ⇒ P0-2 判据**首次真实通过**（此前是「不可达」而非「通过」）。
② 门禁第 7 项调用改 `--bin verify --`（显式 bin）；载体在位 ⇒ 该项由**永久 SKIP 转为默认 full 档实跑 PASS**。
③ 反例已固化进单测：`crates/ti-tools/tests/verify_zero_unknown.rs`（4 条：实跑 exit 0 / `--json` ok=true failed=0 /
无参退出码 2 / 数据面在位前置）。

**复发防护**：① 新增判据时，**判据命令必须可实跑**（多 bin crate 一律 `--bin <name> --` 形式，禁止 `-- <name>` 裸形式）；
② 门禁项若判据依赖某个载体文件，载体缺失应 **FAIL 而非 SKIP**（分档锁定除外，如 tier 条件）；
③ 文档（parity.md / 阶段计划）里出现的判据命令，必须能对应到**一个台账任务号**——否则它只是愿望，不是判据。

### EXP-171 流程：判据命令「形式不可跑」——载体在位也白搭（check_dod_commands 首跑抓出 9 处）
**现象**：S46.1 复盘 EXP-170 时写扫描器 `scripts/check_dod_commands.py`（判据命令载体可达性自检），**首跑即抓出 9 处**
不可实跑判据（修复后 0）：① `docs/parity.md` P0-1 判据 **`cargo run -p ti-tools -- unpack --check-all`**
——ti-tools 是多 bin crate 且无 `default-run`，cargo 直接拒绝（`could not determine which binary to run`），
与 EXP-170 同根因，但**载体 unpak 在位**（是命令**形式**错，非载体缺失）⇒ 该判据长期不可实跑；
② `docs/process/04_tooling.md` 表 `--bin datacheck`（**拼错**，实为 `doccheck`）；③ `docs/check_ledger_reality.md`
示例 `cargo test -p ti-app --test rules_trade`（crate **张冠李戴**，rules_trade 在 ti-core）；④ `docs/process/11_bevy_migration.md`
任务表引用 `--bin treasure-native`（ti-app 旧壳已退役）；⑤~⑨ AGENTS/loop/parity/00_index 里的台账校验命令
（同义重复但均为正确形式，①②③④ 属真伤）。另抓出 doc_numbers `--fix` 的**误伤 1 处**：
parity.md 行 21「doccheck2 `check_touch_render_pair`（门禁 doccheck2 项）」的「doccheck2 项」是
**doccheck2 自身检查项数**（非门禁项数），因行含「门禁」语境词被 gate-total 正则误改写为「doccheck34 项」。

**根因（3 条）**：① EXP-170 对策只钉死了**载体存在性**与**显式 `--bin`** 两类，但文档里的判据命令
**没有机器逐条自检**——「人记得、不复查」⇒ 拼错 bin 名 / 张冠李戴 crate / 引用退役载体这类**形式错**漏网；
② 判据命令散落在 parity.md 表格 / 工具目录 / 任务表多处，**一无唯一清单、二无实跑兜底**；
③ doc_numbers 的 gate-total 语境匹配（`门禁|全绿|gate|Report` 启发式）对「门禁 xxx 项」的**归属有歧义**
——同段落谈「doccheck2 项」也被当 gate 总数改写 ⇒ `--fix` 回写本身需要反例护栏。

**对策（S46.1 落地）**：① `scripts/check_dod_commands.py`：扫描 AGENTS.md + docs/**/*.md
（排除 experience/**（叙事卷引反例）/ impl/**（历史规格）/ plan-*.md（阶段 WBS，命令多被 superseded）），
判据命令逐条解析——`cargo run -p <crate> --bin <bin>` 对 **`cargo metadata` 权威 bin 清单**、
`cargo test -p <crate> --test <name>` 对权威 test 清单、`python scripts/*.py`/`pwsh scripts/*.ps1` 对文件存在性；
多 bin crate 上 `cargo run -p <crate> -- <arg>`（无 `--bin`）= **歧义 FAIL**（EXP-170 同类）；
`<module>`/单字母泛指 = 占位符跳过（误报防护，实测 `<module>`/`z.py` 两类）。不可达判据非 0 退出。
② 固化为**门禁第 25 项**（该 EXP 时点门禁 34 项；现行 36 项，`doc_numbers` 声称面口径）；门禁项数牵动 8 处声称由 `doc_numbers --fix` 同步 + doccheck2 期望值 33→34（时点值，现行 36）。
③ 4 处真伤当场修复：P0-1 判据改 `--bin unpak -- --check-all`（parity.md）/ datacheck→doccheck /
ti-app→ti-core / treasure-native→treasure-bevy-native。
④ 负例护栏：临时注入 `cargo run -p ti-tools -- fakebin` ⇒ 检出 FAIL；恢复 ⇒ PASS。

**复发防护**：① 文档新增判据命令时，跑 `python3 scripts/check_dod_commands.py` 自检（gate 每轮全量兜底）；
② doc_numbers `--fix` 类回写工具改动须带**负例（故意改错看报红）**——正例全绿不能证明正则会伤人（EXP-108 同类）；
③ 「门禁 N 项」语境词启发式对**工具自身项数**有歧义：写「doccheck2 41 项」这类声明时**别在句首加「门禁」语境词**，
或改用「doccheck2 自身检查 41」表述。

### EXP-172 工具：回写工具自身覆盖盲区——doc_numbers `--fix` 漏英文横幅 + doccheck2.rs 期望值
**现象**：S46.1 门禁项数 33→34 时，`scripts/doc_numbers.py --fix` 自称「回写 8 处」并打印
「已回写 23 处声称点」，但**门禁仍红**（doccheck2 `gate.ps1 实测固定项 34，期望 33`）；
人工排查发现 `--fix` 漏了 2 处：① `scripts/gate.ps1` **英文横幅** `=== gate: TreasureIsl (N items) ===`
（`claim_sites` 只认中文「N 项」正则 `(\d+)\s*项`，英文 `items` 盲区）；② `crates/ti-tools/src/bin/doccheck2.rs`
**期望值硬编码** `let expected = 34usize;`（`gate_item_actual_count` 断言 gate.ps1 实测 == expected，
此值不在 8 个声称文件内）。两处均为 S46.1 手工补刀才转绿。

**根因（2 条）**：① `doc_numbers.py` 的声称点清单是**固定文件列表 + 单一中文正则**，未随门禁结构
演化同步（英文横幅与 Rust 硬编码是后加的声称形态，工具没跟上）；② 门禁项数变更的**真权威**
是 doccheck2.rs 的 `expected`（gate-item 检查的对照基准），但工具把「8 个文档声称」当全部，
把真正的基准值留给人肉 ⇒ 权威源与声称点脱节。

**对策（S46.2 落地）**：① 给 `doc_numbers.py` 加两个声称点——`GATE_BANNER_RE`（`(N items)`，仅 gate.ps1）
与 `DOCCHECK2_EXPECTED_RE`（`let expected = Nusize;`，仅 doccheck2.rs），纳入 `--fix` 同口径回写；
② header 注释「8 处」改「10 处」+ 覆盖面清单明示；③ 负例护栏：临时把横幅与期望改 33 ⇒
`--fix` 回写 34（实测 3 处含 release 计数）⇒ 再体检 0 漂移；④ 15 号第 12 项 + 14 号 doc_numbers 行同步。

**同轮相关发现（S46.1 实证，一并记住）**：`doccheck2::check_process_ledger` 对 15 号载体格的解析按空白分割，
载体写 `scripts/x.py（gate 第 N 项）`（反引号后**紧贴全角括号**）会被当成路径 `scripts/x.py`（gate` →
误报「载体不存在」；写法 = 反引号后加**空格**再写括号注释（`scripts/x.py` gate 第 N 项）。
这是「载体格注释粘括号」小坑，与 EXP-171 ③ 同类（工具对书写形式敏感）。

**复发防护**：① 回写/生成类工具（`--fix` 形态）改动后，必须做**负例**（故意把声称点改错看是否
报红/回写），正例全绿不能证明覆盖面（EXP-171 ③ 同类）；② 工具的「声称点清单」应与其**权威基准
来源**（doccheck2.rs expected / gate.ps1 实测）一起维护，别把基准值当外部常量人肉同步；
③ 门禁项数变更后的自检 = 跑 `python3 scripts/doc_numbers.py`（应 0 漂移）+ `cargo run -q -p ti-tools --bin doccheck2`（应 0 error，gate-items 项兜底）。

### EXP-175 流程：换 UI 构图丢菜单命中（pick_key）
S48.0 主题构图（compose.json landscape_primitives）菜单牌只写 kind/rect/z/params.asset，未带 params.pick_key/pick_index；S39 默认横屏菜单牌有 params.pick_key=1+pick_index=0..5（on_hit key=1 命中已接线）。主题构图丢命中 → 换 UI 后横屏菜单不可点击。修复：10 套 compose.json 菜单牌补 params.pick_key=1+pick_index=0..5 + 命中几何验证（6 菜单中心命中 (1,0)~(1,5)）。教训：换 UI 构图必须保留命中数据（pick_key/pick_index），compose 菜单牌须与 S39 同结构带 params.pick_key。

### EXP-176 流程：向导 layout 步与 theme 步 diff 断言冲突
S48.0 平台 UI 向导插 theme 步（ui_theme 写 scenes.json 构图）后，layout 步原判据=0 scenes.json diff（纯变体跳过）与 theme 步写构图冲突（theme 已写 → diff 非 0 → layout FAIL）。修复：resolve_execution/evaluate_criterion 区分 state.theme 非空（theme 已写构图）⇒ layout 改 python 校验 scenes.json 规范形（exit 0），theme 空保持 git diff 0。教训：向导加「写构图」的步时，后续断言 0 diff 的步须区分语义（theme 非空用规范形验证，非强制 0 diff）。

### EXP-178 流程：零 Rust vs 测试断言冲突（换 UI 改 scenes.json 默认）
S48.0 主题构图落 scenes.json index.landscape.primitives（最后 t10）→ 既有 landscape_drawlist 测试（断言 S39 羊皮藏宝图 index_bg_map_h 满屏 + 木牌 @1446）FAIL；改测试（ti-app 测试文件）= 破零 Rust（crates/ 全目录 0 diff）。修复：scenes.json index.landscape 恢复 S39 默认（landscape_drawlist 7 passed + 零 Rust 保持），10 套主题构图保留在 assets/themes/<tid>/compose.json（可 apply 切换，运行时默认 S39 不变）。教训：内容运营换 UI 若改 scenes.json 默认，须评估既有测试断言（版面/命中测试锁定默认构图）；正确 = 主题包保留 + scenes.json 默认不变（测试过 + 零 Rust），换主题=apply 主题包。

### EXP-186 流程：CCGS 借鉴分析里「已替代」误判（事前协作协议真没有等价物）
S50.4 收口时把 CCGS「5 步协作协议逐 agent 内嵌」判进「4 不借鉴」，理由写「loop.md §4 + 硬规则 2 已替代」。用户质疑后查全库：loop.md §4 实际是「卡死与回退」（验收失败拆子任务），硬规则 2 是「验收才算完成」（事后出示证据）——两者都不覆盖 CCGS 五步协议的**事前**面（提问→给 2-4 方案+利弊→你拍板→出草稿→你批准才落地）。现有事前约束只有 06_flywheel §1 角色禁区（角色级一刀切「不改任何文件」，不是「可以改但改前先问+给方案+拍板」）+ 03_templates §7 任务书的「开轮必读+规范回执」（只约束读什么，无方案选项/拍板/草稿批准面）+ A 纪律（只覆盖 UI 视觉类，非通用五步协议）= 全库 0 命中「先问再写/给 2-4 方案/草稿先过目/批准才落地」。教训：做对照分析判「不借鉴」前，必须查我方**是否真有等价载体**（grep 关键词验证），不能凭「我方已有协作机制」印象归进「已替代」——CCGS 的五步协议是事前约束，我方收口约束（台账/门禁/双审计）是事后约束，两者正交，**事前协作协议面 = 真缺口**。S51.0 起改形态借鉴（固化进任务书模版段，非逐 agent 内嵌，因我方无静态 agent 配置文件）。

### EXP-187 流程：i18n 孤儿 key 审计两类盲区（动态 key 拼接 + 业务规格预置文案）
全域飞轮 R 审计 76 条里 41 条 i18n 孤儿 key，主线程复核抓到 2 类误报盲区：
① 动态 key 拼接 = 静态扫描盲区：`chrome.rs space_name()` 对 9 个 SpaceKind 变体、`main_map.rs render_hud_button(label_key)` 经变量传 key、`select_map.rs format!("glmapm{N:02}.Map")` 拼接 —— collect_sget_keys 只扫字面量 sget("...")，这些 0 命中 = 误报孤儿，须加前缀豁免（scene.main_map.node./hud. + glmapm*.Map）。
② 业务规格预置文案 ≠ 孤儿：`isle.*.guard/.benefit` 20 key 代码当前 0 引用，但 territory.md §3 明确「守军/占领后每圈产出」列 = 副本场景（S33.6 后 10 专属场景）将消费的 i18n 文案。判孤儿前须查业务规格单一事实源是否记载该文案用途。
教训：i18n 孤儿裁决三步 = grep 字面量消费 → grep 动态拼接点（format!/变量传 key）→ 查 docs/business/ 是否规格预置。三步全 0 才真孤儿可删。

### EXP-188 流程：飞轮 R2 审计 A/B 双 agent 对抗 + 越界 import 合法化口径
飞轮 R2 首轮真按 S51 规范结构跑（3 subagent 带红线注入 + 五步回执 + A/B 双审计对抗）。
① A/B 双审计：A 侧 = 完成度审计（14 P1 功能缺口/测试盲区），B 侧 = 一致性审计（值漂移/口径过期/恒 PASS）。两路独立，B 抓到 A 看不到的 3 条（B-01/B-02 doccheck2 42→44 项数漂移 + B-04 lighthouse benefit 三方不一致 + B-08 world_canvas 锚点待验）。
② 越界 import 口径定案： 7 处  = **表现层→规则层正确分层**，红线 1「越界」指反向（rules→渲染）。 补 7 模块公共面 + 合法化注。
③ 测试盲区 ~14 条（rules 函数 0 集成测试）= 高成本留后续专项，15 号台账待固化登记。
④ 死代码 3 函数（ai_turn_v2/victory_seat/player_defeated）= pub + 内联测试引用，删 0 收益不删（棘轮豁免）。

### EXP-189 流程：R12 双 agent 对抗审计——「项号文字本身」漂移 = 高频复现模式
R12（rea-audit-fix 双 agent：FIND 100 条 Q001~Q100 + AUDIT 逐条复核 46 CONFIRMED / 2 REJECTED / 20 PARTIAL / 32 NO-LINE + 主线程独立复核 8 条最高价值 + 自查 3 条漏网）修 20+ 处漂移。三类系统性模式：
① **项号镜像漂移**：`gate_data.json` `doccheck2_checks` 数组顺序是机读唯一源，文档文字「doccheck2 第 N 项 X」是手工镜像；S53/S54/S55 加 3 项（44→47）后，tooling-catalog/tooling-contract 从 26/27 漂到 28/29（9 处现状区：14_tooling_catalog×4 / AGENTS×2 / scripts-rules×2 / README×1），engine-no-color-literal 被 5 处误写「第 41 项」实为 15（S38 时点该项本就 15 = 数组前段从未变动，纯系统性误写非时点漂移，且与 rules/ti-engine-rules.md「第 15 项」库内矛盾）。
② **gate 侧文字与代码不符**：gate.ps1 注释 + Report 项名写「CS-1~6」，check_code_standards.py 实测仅 CS-1~4 四项；32_ccgs_borrow 同误写。
③ **版本/路径/工具矩阵面**：release.md 浏览器行「当前 0.1.0 / 48,765,303 B」vs 磁盘 0.2.0 / 45,794,950 B（版本规划表第 28 行 0.2.0 行才是真值）；gen_layout.py 3 处写 scripts/ 实为 legacy/scripts/（gate 实调路径）；26_dev_stages 工具矩阵 36 项实列 33 项（漏 dod-commands/studio-tests/Android 真机）+ PASS/SKIP 合计错；11_bevy_migration S18.7g「第 24 项」实 26（AUDIT 漏网自查）；08_render_arch 契约 1 表项号改现行 gate 13 项 doccheck2 内子检查口径 + §5.1 已退役基线段加「历史记述」标注；AGENTS/README「doccheck 独立第 9 项」统一改「gate 第 9 项」（doccheck 就是 gate 第 9 项本身，「独立」措辞歧义）。
处置面 = 现状区（AGENTS/README/14/28/26/04/rules/experience index 表）全部改现行值；**历史叙事卷（EXP-113/115 正文、release 版本规划表 0.2.0/0.1.0 行、15 号 §2 台账行）保留时点快照不改**（带时点标注 = 正确形态，飞轮 R 已固化口径「历史计划文档里 N 项 = 该阶段时点快照，引用须带时点标注」）。
固化判定 = **不新增 doccheck2 断言**：项号引用面 20+ 处、gate_data.json 顺序机读可信，文字漂移靠 cross-doc-counts（第 44 项）+ gate-items（第 6 项）双断言 + 对抗审计人工复核兜底；后续 gate/doccheck2 项数变更 = doc_numbers --fix 扩「项号文字」回写面（15 号 §2 第 12 行已登记该工具，扩面待专项）。
证据 = evidence/r12-audit.log + r12-audit-find.md + r12-audit-audit.md。
教训：双 agent 审计价值面 = B 侧（一致性）独报的项号/计数类漂移 A 侧（完成度）结构性看不见——本轮 46 CONFIRMED 里 0 条是功能缺口，全 = 值/编号/路径漂移，与 S51.1 飞轮 R2「B 侧独报 3 条 A 看不到的值漂移」同构；主线程独立复核不可省（AUDIT 漏网 3 条 + 改判 2 条 = EXP-038 再次实证）。

### EXP-190 流程：流程机制审计：停机口径/回执落盘/条数门禁恒失效/skill 路径冻结
## 现象

开发流程机制面 0 机器兜底清单（15 号 §2 半自动行逐条核实现状 + 历轮 evidence 取证）：
1. 分卷条数门禁恒失效：`check_experience_volume_counts` 行识别判据 `cols[0].contains(".md](")` 对实际行首列 `` [`process.md`](process.md) ``（含反引号）永不成立 ⇒ 该断言自落地起从未生效，index.md:36 process 卷 47 vs 实 48 = 真漂移 0 拦截（EXP-108 当年修判据修错方向，代码注释自认「从未生效」）。
2. 五步回执 0 落盘：`evidence/flywheel-R{2..11}.md` 10 轮 grep「规范回执/五步/决策点」= 0 命中（唯一命中 = 编排描述非回执本身）⇒ 06 §2「缺回执段 = 打回」判据无落盘证据，机制与执行两张皮。
3. 停机口径成本倒挂：「连续 2 轮 0/0/0 才停机」使每域停机至少 2 个全成本 0 产出验证轮（R5/R8/R9 + R10/R11 实证；单日 R3~R11 = 9 轮），与 §8「100 条/轮」目标脱节。
4. re-audit-fix skill 文本冻结面：`C:\金银岛` 根 + `C:\Users\Administrator\...\Temp\opencode\` 输出路径 = Windows 时代写死，Linux 迁移后 0 更新（R12 靠 skill 外的人约定兜 /tmp/opencode）。
5. blocked 行 evidence 假路径：原竖屏主题任务（唯一 blocked，已删除）写 `evidence: "evidence/<已删任务>.log"` 该文件不存在 = loop.md §1 schema 无 blocked⇒evidence 条款的必然产物。
6. exp_log.py --check 语义缺陷：正常登记（index 行 + 分卷正文 = 2 处命中）跑 --check 必退出 1 ⇒ 工具契约「只读 + 违规非 0」正常态也非 0 = 假红，不可挂 gate 例行体检。

## 根因

机制落盘 ≠ 机制执行（15 号行 28 同族）：门禁断言的行识别判据无负例护栏（EXP-108 修判据时未实测跑一次破坏验证）；停机/回执等「人盯纪律」无落盘载体（evidence 固定段缺失 = 无人可复核）；skill 文本 = 开轮固定模板源，环境迁移（Windows→Linux）无同步机制；schema 空档（blocked 行 evidence 语义未定义）由台账行自由发挥填假路径。

## 解法

M-09：判据改 `](` + `.md)` 收口形态匹配 + 表头失锚报错 + 分卷表限定（防条目索引行首列 `[`xx](xx)` 3 列误命中，负例：EXP 行首列含 `](index.md)` 但第三列非整数 = 自然跳过）+ 负例护栏（临时改 47 vs 48 实跑必 FAIL 再还原）。
M-05/M-12：06_flywheel §4 补「停机三条件勾选表 + 五步回执必须拷进 evidence/flywheel-R<n>.md 固定段，缺段 = 停机判定无效/回报打回」。
M-24：judge.rs 收敛阈值 2→1（第 1 个 0 轮即 Pass，历史轮「连续 2 轮」= 修订前口径快照不回改）+ 单测改判 + 22 号 §7.4/plan-s32/s44/s45/README 文档面全同步（历史 verified 行保留时点措辞）。
M-19：skill 文本 2 处路径改「仓库根（当前工作目录）+ /tmp/opencode/（Linux）」。
M-01/M-22：原竖屏主题任务 evidence 置 null + loop.md §1 补「blocked ⇒ evidence: null」条款 + check_ledger_reality 加 blocked 行 schema 校验 + 输出措辞改「三方/两方对账」防 M-04 误读。
M-08：exp_log --check 改「0 或 >3 处命中 = FAIL；2~3 处 = PASS（分卷表条数行引用 +1）」。
M-13/M-23：15 号 §2 行 36 + S56.0 台账立项（doc_numbers.py 扩项号文字 + 数值声称点对账面，待开工）+ 现轮人工清零（R12 20+ 处 + 本轮 README 项号 31~34→33~36 等）。

## 验证

- `check_experience_volume_counts` 负例实跑（临时 47 vs 实 48 ⇒ 1 ERROR 精准点名 process.md；还原 ⇒ 0 error）。
- doccheck2 47 项 0 error〔R12 时点值；现行 51 项〕+ doccheck 0 error + check_ledger_reality 全绿（339 verified 两方对账 + 3 符号 grep + 1 blocked schema + 0 孤儿）。
- `cargo test -p ti-console --test audit_tests` 28 条全绿（converge_one_round_pass 改判 Pass{1}）。
- exp_log --check EXP-189/EXP-001 均 EXIT=0（正常态），EXIT=1 = 违规面（0 处或 >3 处）。
- gate.ps1 头注释补「36 = 去重口径 / Report 行 38」防 M-14 误计数。

### EXP-191 流程：流程强化轮 3 面教训（git 还原丢改动 / 证据文件名前缀撞孤儿检测 / 覆盖率分母设计）
- **现象**：S57 轮 ① `git checkout -- <file>` 把**本轮未提交的** S57.2a patch（check_ledger_reality.py）整文件还原 3 次 ② 多任务组共用证据 `evidence/S57.0.log` 前缀 `S57` 不在台账（台账行 = S57.0/S57.1/S57.2a/S57.2b）⇒ 孤儿 evidence 误报 ③ 符号覆盖率分子混入 3 个文档类手工登记任务 = 106% 假绿。
- **根因**：① `git checkout --` 无差别还原整个文件到 HEAD，对本轮工作区跨任务累积改动 = 丢面；须**先 cp 备份再还原 + 幂等 patch 脚本重做**（`/tmp/opencode/s572a_patch.py` 含「已应用，跳过」守卫 + 好副本 `clr_s572a_good.py`）② 孤儿 evidence 检测口径 =「文件名任务 ID 前缀」，证据文件命名必须带**完整任务 ID 前缀**（S57.0.log 非 S57 前缀 .log）③ 覆盖率分子把「手工登记的文档类任务」（S9.5/S10.4/S21.0 符号非代码产物）混入「改动面含代码」分母面 ⇒ 分母口径 = 声明面 ∩ 可提取符号子集（110 = dod 含 crates 路径/pub 项的可提取面），分子 = 自动提取命中面。
- **解法**：① 高风险 git 还原前先 `cp file /tmp/opencode/bak_file`；patch 脚本幂等可重放（重做 = 跑 patch 脚本，不重敲 edit）② 多任务共用证据文件命名 = 任务组首 ID 全前缀（S57.0.log）③ 覆盖率 = 自动提取面（分母 ∩ 可提取子集，防虚高/假绿）+ 手工面优先合并；28%（31/110）= S57.2a 收口时点基线棘轮（只收紧，S57.2b 阶梯 60%→100%）。
- **验证**：`python3 scripts/check_ledger_reality.py` 全绿（31/110 = 28% 基线）+ 负例实跑（删 S21.0 手工符号 ⇒ FAIL 1，备份还原法） + doccheck2 48 项 0 error〔R13 时点值；现行 51 项〕。

### EXP-192 流程：数值声称点同组多文档口径漂移（R13 三组同模式 + 平台前端硬编码面 + WBS 复选框不回写）
- **现象**：R13 双 agent 审计 8 组 CONFIRMED 缺陷里 3 组同模式 =「同一数值在 3+ 文档各自手写，改 1 处漏 2 处」：①「流程强化 8/7 条」26 号 vs AGENTS.md ②「doccheck2 47/48 项」26 号 L141 漏 S57.1 +1 ③「门禁 33/36 项」27 号 8 处 + ti-console web/index.html 3 处硬编码（R10 勘误只修 28 号/README/AGENTS 文档面，漏平台前端）。另 2 组新面：plan-s56 WBS 复选框 4 行 [ ] 未随台账 verified 回写 + process.md L548 乱码「（off门禁 34 项）」。
- **根因**：① 数值声称点无「枚举型」载体——doccheck2 cross-doc-counts 只守「N 任务（M verified + 0 pending（…）」锚定短语 + 26 号工具矩阵行 doccheck 计数，**「N 条流程强化」枚举面 0 断言**（人工数枚举易 off-by-one，S53 属飞轮 R6 断言固化不属流程强化第二批 = 7 非 8）② 门禁项数声称面 doc_numbers 覆盖 8 文档 + S57.2b 扩 2 文件 = 10 载体，**ti-console 前端 index.html 硬编码 3 处 0 覆盖**（R10 修平台 doc↔code 漂移时前端面漏网）③ WBS 文档（plan-sNN）复选框 = 纯人工回写面，无机器断言（task_close.py 只回写台账不联动 WBS 文档）④ 乱码 = 早期手敲 CJK 混入（Set-Content 类事故残留，EXP-051 同类）。
- **解法**：① 枚举型声称点（「N 条 X」）暂靠对抗审计人工面 + 文档写全枚举（「S54.1/S55.1/S56.0/S57.0~S57.2b 共 7 条」= 枚举随值，改任务自动暴露）；doccheck2 扩「枚举面」断言 = 后续轮 ② doc_numbers.py 扩 `crates/ti-console/web/index.html` 门禁项数声称面（3 处硬编码 ⇒ 1 个声称文件登记，--fix 可回写）= 后续轮 ③ WBS 复选框回写 = 收口轮人工面（plan-sNN 4 行 [ ]→[x]），后续 S58+ WBS 统一在台账 dod 写「WBS 复选框同步」判据 ④ 乱码行 = 修时点标注（现行值 + 时点值双注，防误读）。
- **验证**：R13 8 条全修 + gate 36 项 0 failure + doccheck2 48 项 0 error〔R13 时点值；现行 51 项〕+ doc_numbers 0 漂移（evidence/r13-audit.md）。

### EXP-193 流程：机器门禁自身结构失效盲区（wbs-checkbox-sync 加粗行 100% 跳过 = 断言恒 0 命中）
- **现象**：R14 审计 Q005 = S58.2 新增 doccheck2 第 50 项 `wbs-checkbox-sync` 上线后 plan-s56 S58 段 4 行 `[ ]` 真漂移（台账全 verified）未被抓，doccheck2 实跑 0 error「通过」——机器门禁自身结构失效。
- **根因**：解析器 `id_start=after_box.find('S')` / `id_end=after_box.find("**")` 对全库 plan-sNN 复选框**全是加粗格式** `- [ ] **Sxx（...）**` ⇒ after_box = `**S58.0（...）**`，`find('S')`=2 ≥ `find("**")`=0 ⇒ 每行 `id_start>=id_end` 跳过 = 断言恒 0 命中（与 S57.1 state-md-discipline 首版同类：新断言须双负例实跑验证触发面，「0 error」≠「断言生效」）。
- **解法**：解析器修 = 剥前导 `*` 后取 S 开头任务 id 到「下一 非 alnum/`.`/`-` 字符」；新断言上线判据必含**双负例实跑**（临时造 1 条违规 ⇒ 必 FAIL 精准点名 + 还原 0 error），0 error 不能当生效证据。
- **验证**：R14 修复后负例实跑（plan-s56 S58.0 `[x]`→`[ ]` ⇒ ERROR 精准点名 L24 S58.0 ✓ + 还原 0 error）+ 全库 plan-sNN 复选框 ↔ 台账 verified 全 0 漂移。

### EXP-194 流程：gate 项号文字旧时点值全库残留（EXP-189 同类第 3 次，R15 实抓 9 处）
- **现象**：R15 审计 gate 项号文字（「第 N 项」/「gate 第 N 位」）全库 9 处旧时点值无标注：AGENTS.md L14/L87/L144/L145/L146（第 17/23/12/13/23 项）+ loop.md L37（第 35 项）等；现行去重实序（36 项 BTreeSet 去重）dod-commands=32 / flywheel=26 / check_ledger_reality=20 / ledger=10 / doccheck=11，旧值 = 早期 12/25 项体系时点命名。
- **根因**：gate.ps1 加减项后文档「第 N 项」文字是手工镜像（gate_data.json 去重序是机读源），R12 已修 20+ 处但 gate 项号面**无机器断言**（doccheck2 50 项守 doccheck2 项号文字 S56.0，不守 gate 项号文字；EXP-189 同类 = 第 N 次加减 gate 项后第 N+1 次复发）。
- **解法**：① 本轮 9 处全修（去重序实值 + 旧值时点标注「旧第 N 项 = X 项体系时点值」）② 后续轮扩 doccheck2「gate 项号文字」断言面（扫全库「第 N 项」声称点 ↔ gate.ps1 完整描述串去重 1-based 位实测，与 S56.0 doccheck2 项号面同模式，载体待立项）。
- **验证**：R15 9 处全修 + gate 36 项 0 failure + doccheck2 50 项 0 error（evidence/r15-audit.md）。
### EXP-195 · gate 项号文字断言（S59.0 载体）+ WBS 表格形态对账（S59.1）三坑

- **坑①（S59.0 断言恒 0 命中根因）**：posmap 单行 item `{"short":"X","pos":N}` 的 Rust 手工解析，`"short":` = **8 字符**（`"short"` 7 + 冒号 1），用 `s_idx + 7` 取到的是**冒号**而非值引号 ⇒ `strip_prefix('"')` 失败 ⇒ short 值全解析成 `":"`（垃圾）⇒ short_pos 36 项但全空值 ⇒ `line.contains(short)` 永不命中 ⇒ 断言恒 0 命中（负例改坏不 FAIL）。**教训 = 手工 JSON 解析的字符 offset 须逐字符核实**（`"short":` = `" s h o r t "` 7 字符 + `:` = 8 位；`"pos":` = 6 位），且首跑须用负例实跑验证解析值（而非只看 len），`[dbg] short_pos = 36 项` 不等于「解析正确」，须打 first3 看值。
- **坑②（gate 项号声称点 claimed 提取过宽 = 63 误报）**：初版用「全行 contains(short)」匹配 gate 项名，把行末叙述里碰巧出现的 gate 短名（doccheck/doccheck2 等）与 claimed 数字误配 ⇒ 63 处误报。**修法 = 紧邻区 + 词边界**：short 名须出现在「项/位」后 ≤14 字符窗口内 **且** 命中后紧跟字符非 ASCII 字母数字（防 `doccheck` 短名误吃 `doccheck2`）。
- **坑③（语义误报：doccheck2 检查项号 ≠ gate 项号）**：`第 N 项 doccheck`（N = doccheck2 内部检查序号）与 `gate 第 N 项 <gate项名>`（N = gate 去重位号）形态相同但语义不同；历史经验条目（EXP-065）的「gate 第 9 项 doccheck」= 旧 9 项体系时点值。**修法 = 时点/历史条目加「旧 9 项体系时点值」标注豁免；现状规范文档（style.md）修实位；exp 历史条目豁免。**
- **坑④（S59.1 表格 WBS 状态标注过宽 = 3 误报）**：初版用「行内含 pending 字样」判 pending 状态，把描述列散字（"允许类目 pending"/"pending=0"）误判 ⇒ 3 处历史 WBS 表格误报。**修法 = 只认明确状态标注形态（`✅ verified`/`已 verified`/`状态 \`pending\``），散字不判 + 历史阶段文档（plan-s0-s2/plan-s3）整体豁免（状态列 = 当时快照）。**
- **收口判据**：S59.0 = doccheck2 第 51 项 `gate-item-number-text`（机读源 `assets/gate_item_posmap.json` + `scripts/gate_item_posmap.py` 回写 + 3 处工具契约登记）；S59.1 = `wbs-checkbox-sync` 扩 `check_wbs_table_row`（plan-sNN 表格 WBS ↔ 台账 verified 对账）；双负例实跑 + clippy -D 0 warning + gate 全绿。
#### S59.0 gate 项号断言 AUDIT R16 抓出 4 真逻辑漏洞（修毕）

- **A-02 P0（整行豁免掩盖 2 真漂移）**：首版豁免取**行内首个「项/位」后全扫**「时点值/旧第」⇒ 同行「当前 gate 声称 + 历史时点标注」整行被误豁免，掩盖 AGENTS:145（doccheck 第11位≠实位13）+ loop:37（dod-commands 第32位≠实位35）两条真漂移。修 = 豁免只认**声称点数字后 8 字符内**带时点标注；另加「演进叙述行」（整行带「起」+ ≥2 个「第 N 项」）+ 子项分组（「第 N/M 项」斜杠）+ exp 历史条目整文件豁免。
- **A-01 P1（窗口 14 字符过小）**：紧邻区窗口硬编码 14 字符，对 >14 字符 short 名（`no unwrap()/expect()...` = 45 字符）结构性不可匹配，64% gate 项号漂移漏检。修 = 窗口 = `max(short_len) + 4`（动态 49 字符）。负例 flywheel-readme（15 字符）改 26→25 精准命中 loop.md:127 验证。
- **A-03 P2（多声称行只核首个）**：`iter_gate_claims` 初版 `line.find("第")` 只取首个「第」，多声称行（`gate 第 13 项 doccheck、第 99 项 bogus`）后续声称漏核。修 = while 全行迭代所有「第」。
- **A-05 P2（重复短名假阳性）**：posmap 含重复短名（`cargo build` pos4/6、`no-macroquad` pos20/21），窗口匹配 break 取**首个**命中 ⇒ 声称后一个位置必报假阳性。修 = 窗口取**最靠后** occurrence（最近声称）。
- **坑·CJK 字节边界 panic**：`iter_gate_claims` 的 `search_from = abs + 1`（abs = 「第」首字节）落「第」3 字节中间 ⇒ `line[search_from..]` char boundary panic（整个 doccheck2 进程崩，后续所有文档不扫）。修 = `abs + "第".len()`（跳整字 3 字节）。**教训 = CJK 字符串按字节推进必跳整字，`+1` 落多字节中间即 panic；且 panic 会静默吞掉整条门禁（doccheck2 exit 101 而非报「0 项」）。**
- **坑·gate 前缀 take(14) 取错方向**：`line[..di].chars().take(14)` 取「第」**行首** 14 字符（`| \`check_dod_c`）而非「第」**前** 14 字符 ⇒ gate 前缀校验恒 false，断言恒 0 命中。修 = `line[..di].chars().rev().take(14).collect().rev()` 取「第」前 14 字符。**教训 = `chars().take(n)` 取迭代器**前 n**（行首方向），取「尾部 n」须 `.rev().take(n)` 再 rev。**
#### R16 复扫 2 的 4 结构性漏网面处置
- **F-04 P3 已修**：gate-item-number-text 扫描面 15→16 文件（加 `docs/process/06_flywheel.md`，gate 第 26 位 flywheel-readme 高频停机锚点）。
- **F-02 P2 部分修**：`iter_gate_claims` 推进逻辑改 `search_from = min(第+len, item_pos)`（防 item_pos 越位跳第二声称）；**残留局限 = 跨行 gate 项号**（"第 N 项" 在行尾、short 名在下行，如 parity:111「gate 第 26 项」+ L112 flywheel-readme）窗口只取本行 ⇒ 跨行 short 名不核（值对故无真漂移，登记为已知局限，后续轮扩跨行窗口）。
- **F-01 P1 已修**：子项后缀「第 4c/25b/10a 项」（数字后紧跟小写字母 = 条件子块编号）识别后跳过（gate.ps1 条件子项去重后归并 1 主项，文档子项号 vs posmap 主位号口径不同，不核；防 "4c" 被 digits 解析为 "4" + 窗口短路误命中 pos8 feature-gated）。
- **F-03 P2 判定误报**：32_ccgs:67「gate 第 25 项 dod-commands（S50 立项时点快照；现行=36项）」= 时点快照行（带「S50 立项时点快照」标注）⇒ 整行「起」+多声称豁免**正确**（该行本就该豁免，不回改历史位号）。
#### 已知局限清零（用户拍板 1-3 全做，竖屏禁做已删竖屏主题任务）
- **跨行 gate 项号（P3 登记局限）**：`check_gate_item_number_text` 窗口只取本行（`line[claim.item_pos..]`），「第 N 项」在行尾 + short 名在下行（如 parity:111「gate 第 26 项」+ L112 flywheel-readme）不核。当前唯一 case 值对（26=26）无真漂移，跨行扩窗 ROI 低 + 易引入新 bug ⇒ 登记已知局限，后续轮再扩（需预读下行 + 偏移换算）。
- **A-06 WBS 状态变体（已扩）**：`check_wbs_table_row` 状态词表加 `verified（`（半角括号）+ `未 verified`（pending 变体）⇒ 灵敏度提升；blocked 不在本面（由 blocked-label-consistency 第 46 项核）。台账 0 pending（全 350 verified）故无 pending↔verified 可负例，扩词表 = 形态覆盖（值对无真漂移）。
- **原竖屏主题任务删除（用户「有则删除」）**：台账竖屏主题行删除（351→350 任务，1 blocked→0）+ S54.1 deps 改 [S49.0] + 全库 6 文件竖屏任务 token 清零（plan-s49/plan-archive/process/loop/release/AGENTS 改「原竖屏主题作废已删除」叙述）+ 计数面 351/350/1→350/350/0 全同步 + doccheck2 锚定短语同步。台账 350 任务 / 350 verified / 0 pending / 0 blocked = 全清零。
#### R17 飞轮（S59 + 竖屏主题删除 + 全库值漂移复扫）处置
- **AUDIT-B 7 真漂移全修**：gate 项号文字（AGENTS doccheck 9→13 / check_ledger_reality 20→23〔旧第20时点值〕/ ledger 10→12〔旧第10〕/ 26号 flywheel 17→26）+ doccheck2 项数（26号 47/48→51 / plan-archive 44→47 加时点标注 / AGENTS 里程碑 44→47 加「S55 时点值；现行 51 项」标注）。修后 doccheck2 0 error。
- **AUDIT-B 2 断言盲区登记局限**：
  - B-18 `gate-item-number-text` 「第 N 项」后**无 posmap short 名**时（如 "gate 第 9 项）。"语境型）`continue` 不核 = 盲区。根治需"语境判定 gate 项"（"第 N 项" 可能指 doccheck2 项/子项，歧义）⇒ 登记局限，靠 AUDIT 人工 + 修后不复发。
  - B-19 `cross-doc-counts` 仅核「N 任务」+「gate N 项」，不核「doccheck2 N 项」声称 ⇒ B-01/B-06/B-07 doccheck2 项数漂移无断言覆盖。登记局限（后续轮 cross-doc-counts 扩 doccheck2 项数面）。
- **AUDIT-A（主线程代做，subagent LLM 后端故障 2 次）**：断言面完整（16 文档覆盖高频 gate 项号文件；41 文件含 gate 项号但余 25 = 历史/时点/tutorial）+ 红线代码 0 违规（engine-no-color-literal/CS-1/CS-4 全基线内）+ doccheck2 51 项 0 恒 PASS（现行全过）= 0 新发现。
- **FIND = 0 条（0/0）**：竖屏主题任务删干净（全库 0 token）+ 计数面 350/350/0/0 全同步 + 门禁全 0。

#### 双盲对抗找游戏逻辑 bug / 合理性问题（用户 2026-11-17 拍板「自己整理对抗找问题」）
- **方法**：双 subagent 对抗（A 逻辑 bug 实现≠规格 / B 合理性失衡，双盲不读对方结论）+ 主线程逐条独立复核（EXP-038 子代理声称不算数）→ 合并 → 只修真 bug + 补测试盲区 + 设计缺陷列清单给用户裁决。
- **A 侧 9 条复核判定**：
  - 真 bug 已修 = A-03（`settle_siege` 攻占败漏入账守方，金币凭空消失）+ A-04（AI/玩家交钱超付，`apply_pay_toll` 返全额 toll 不钳金，守方多收）= **金币守恒**，两路径（host.rs `pay_siege_toll` + board.rs `settle_ai_landing`）+ `settle_siege` 三处统一改「实付 = min(toll, 席位金)」+ 守方入实付。补 2 e2e（M-04 `siege_loss_credits_defender_gold` / M-05 `pay_toll_caps_at_seat_gold`）。
  - 误报 = A-01（`run_ai_turns` 循环上界 5 对 5 席全环恒够，`advance` 跳破产不耗额外迭代，边界安全）/ A-05（v4 席位无 crew 字段恒缺省 0，迁移补默认正确，v5 运行态 0 不经迁移路径）/ A-07（`previous_occupant` rposition-1 对简单序列 a→b→c 正确取前驱，重复到访非规格 case）/ A-09（BOSS 战 `boss_fort` 防御 = S34.6 D5 用户拍板迁移兵力模型，旧 HP `boss_tiers` 作废，设计选择非 bug）/ A-11（`force::resolve` 守方胜与 combat.md 一致）。
  - 设计缺陷（实现≠规格，列清单待裁决）= A-02（`attack_threshold 0.6` 连续梯度退化为 `>0.0` 布尔开关）/ A-06（玩家落自己格自动升级/领产出，规格 §4「玩家可选」三支未做选择 UI + 增减驻军分支缺失）/ A-08（`assault` 无格属前置校验，无主格/自格调攻占语义错，P3 防御性）。
- **B 侧 5 条复核判定**：
  - 死内容/死参/死数据 = B-01（4 combat items keg/medkit/rum/smoke 无获取路径，卡片/事件/商人均不产 item）/ B-04（`toll_aversion` 仅 v1 `ai_policy` 用，v2 `ai_policy_v2` 死参）/ B-05（`boss_tiers` 旧 HP 表仅 `difficulty_ladder`/`rules_battle` 测试引用，active combat 用 `force::resolve` 不用，vestigial）。
  - 设计缺陷（合理性，待裁决）= B-02（`upgrade_priority=0.3 < 0.5` 阈值 ⇒ AI 永不升级，领地永远 L1）/ B-03（雪崩：先占者收入优势 7.5×，20 图 campaign 在 ~5 图后无意义，缺 catch-up 机制）。
- **教训**：金币守恒类 bug 最易漏（扣款方扣了、收款方没入账 = 净消失；或超额入账 = 净产生），双盲对抗 A 侧独立抓出 3 处同类（`settle_siege` 攻占败 / AI 交钱 / 玩家交钱），须统一「实付 = min(应付, 实有)」+ 收方入实付的守恒契约；测试盲区 M-04/M-05 补 e2e 守恒断言。设计缺陷类（A-02/A-06/B-01/B-02/B-03）= 实现偏离规格或数值失衡，须用户裁决方向（自动化 vs 选择 UI / 死内容接线 / AI 升级 / 雪崩 catch-up），非机械可判。

### EXP-196 · 易变平衡逻辑禁写进 rules 函数体（数据驱动纪律）
- **坑**：双盲对抗 A-02/B-02 修复时，把 AI 决策公式（`攻方战力 ≥ attack_threshold × 守方战力` 战力比门控 / `金 ≥ 升级费 × (1.0 + upgrade_priority)` 金富余门控）写进了 `rules/territory_v2.rs::ai_policy_v2` **函数体**。用户点出「易变逻辑写进代码层」= 违背 `economy.md` L12「数值全在 balance.json，调平衡不改代码」+ `01_architecture.md` 数据驱动分层。AI 激进/保守本是易变平衡项，改它不该改代码。
- **准则**：rules 层函数体只实现「读数据参数 → 选分支 → 出结果」的**骨架**；任何**平衡公式的系数/门槛/形态**（战力比阈值、金富余缓冲、概率、等级曲线）一律进 `balance.json`，代码读字段选形态。判定 = 「这条若改了算不算改平衡？」是 ⇒ 进数据层，禁写死函数体。
- **落地**（A-02/B-02 修正）：`ai_policy_v2` 加 `siege_mode`（`ratio`/`win_only`）+ `upgrade_mode`（`gold_buffer`/`priority_gate`）两数据字段（`balance.territory.ai`，`territory_cfg.rs` `SiegeMode`/`UpgradeMode` 枚举 + `RawTerritoryAi` String serde + `convert` match 转换），函数体 `match ai.siege_mode` 选公式形态、系数读 `ai.attack_threshold`/`ai.upgrade_priority`。改 AI 行为 = 改 balance.json 两字符串，零代码。
- **门禁面**：新增易变平衡公式时，先落 `balance.json` 字段 + `territory_cfg.rs` 结构 + `convert`，再改 rules 函数体读它；禁在 rules 函数体新写 `f32/f64 字面量系数`（如 `× 1.3`/`>= 0.5`/`0.6`）进平衡公式。轻量守门 = grep `rules/` 新增浮点字面量进系数乘法/比较（`check_code_standards.py` CS 候选扩展面，或 doccheck2 断言；误报面 = 正常逻辑常量如 `round_pct`/损耗率，须白名单）。
- **关联**：EXP-066（战斗玩家数值数据驱动）同类；CS-1（资产路径单源）同数据驱动纪律族。

#### HP 模型（旧 HP 回合制 + 道具）彻底删除（S34.6 D5 收口，用户拍板「全删 HP 本体」）
- **背景**：S34.6 D5 迁移兵力模型后，旧 HP 回合制（`rules::battle::BattleSession` 逐回合 + `combat.items` 4 道具 + `combat.action` 3 动作）成残留；`combat.md`/`11_vs_spec.md` 早已标注作废，但代码/数据/i18n/测试/example 未清。双盲对抗 B 侧报 B-01（4 道具死内容）/B-05（`boss_tiers` 表）= 同残留族。
- **删除面**（系统重构，先调用方→后实现→最后数据/i18n）：
  - `rules/battle.rs` 整删（`BattleSession`/`CombatBonus`/`step`/`step_with`/`boss_hp`）+ `rules/mod.rs` 声明 + `oracle/battle.rs` 整删（无 reference_diff 消费方，纯死基准）+ `oracle/mod.rs` 声明。
  - `combat.items` 4 道具（`RawCombatItem`/`RawItemEffect`/`CombatItemDef`/`ItemEffect`/`CombatAction`/`item()`）+ `balance.json combat.items[]` + `combat.item.*` i18n 4 键（保留 `combat.anim.items` 道具动效表 = 渲染 `vs_anim_v4` 缺省回退，无道具逻辑消费 = 空表安全退化）。
  - ti-app 侧（前轮已清）：`BattleView` 7 HP 字段 + `BattleSnapshot` 5 HP 字段 + `start_battle`/`start_dungeon_battle` 走 `defender_force` + `displayed_*_hp` 全链 + `vs_proto`/`treasure_headless`/e2e 读兵力量。
  - 测试/example：`rules_battle.rs`/`battle_rules.rs`/`exhaust_combat.rs` 整删 + `rules_boat` 删 `combat_bonus_includes_boat`（船加成进 HP 公式无兵力意义）+ `exhaust_random_session`/`territory_rules` 战斗段改 `force::resolve` + 3 example（`full_run`/`cross_map`/`random_session`）`BattleSession` 改 `force::resolve` 一次性。
  - `boss_tiers` 表**保留**（`defender_force` 用 `boss_tier+1` 查 `boss_fort` 防御 = 难度阶梯数据，非 HP 模型本身；`difficulty_ladder` 测试依赖）。
- **判定**：`Unit`/`Side`/`defender_unit`/`CombatCfg.player` 保留（`CombatCfg.player: Option<Unit>` 数据字段仍活 + `defender_unit` 守军强度口径仍被注释引用；纯死代码清理不强行删活数据字段）。
- **教训**：迁移类重构（HP→兵力）须**同步清**「实现 + 数据 + i18n + 资产 + 测试 + example + 文档」，否则残留成死代码长期挂编译面；双盲对抗 B 侧抓的"死内容"（B-01/B-05）= 迁移未清残留的典型，须系统删除面摸排（subagent 摸排 + 主线程逐层验证）而非逐点删。

#### 双盲对抗设计缺陷处置（A-06/A-08 落地 + B-03 待裁，2026-11-19）
- **A-06（玩家落自己格无选择权，规格 §4「玩家可选 ①升级 ②领产出」）= 已修**：落自己格从「自动升级/领产出」改为置 `pending_own` 待决 + 提示二选一，主地图 `Confirm`=`confirm_own`（升级加固，金不足/满级退化领产出）/`Back`=`abandon_own`（领产出），对齐 `pending_siege`/`pending_dungeon` 的玩家选择权模式。金币守恒：升级扣费/领产出入账自己（`credit_seat_gold`）。补 e2e `own_landing_gives_player_choice`。规格 §4 第三支「增减驻军」未做（独立交互，留后续轮）。
- **A-08（`assault` 无格属前置校验，无主格/自格调攻占语义错）= 已修（防御性）**：`assault` 加 `debug_assert!(matches!(own.owner, Some(o) if o != seat_idx))` + 文档前置契约（仅适用他人格，生产调用方 `ai_policy_v2` 他人格产生 `Siege` + `pending_siege` 落他人格已保证；无主走副本、自己走升级）。`debug_assert` 零生产成本，测试/开发期抓误用；不改 `AssaultOutcome`（Win/Loss 两态不变，避免改枚举传染）。
- **B-03（雪崩：先占者收入优势 ~7.5×，20 图 campaign ~5 图后无意义，缺 catch-up）= 用户拍板接受现状**：核实无 catch-up 机制（领地收入 `production_base` 45~100/圈 + `lap_bonus` 150 + `first_capture_bonus` 150 累积优势真实）。**大富翁类游戏固有特性**（Monopoly 本就先占者胜）。用户 2026-11-19 拍板「接受大富翁滚雪球，先占者胜是玩法的一部分」= **设计选择保留，不加 catch-up 落后者加成**（避免破坏大富翁核心爽感），登记 `territory.md` B-03 段。对局胜负仍走 `victory_seat`/`bankrupt`。

### EXP-197 流程：i18n 孤儿 ≠ 死代码：渲染活路径消费的 key 禁删
- **现象**：X2 删 Landing::Battle 后 battle_fmt i18n 孤儿 1 条（可删）；但同轮 combat.item.* 4 key 看着像孤儿（combat.items 表已删 B-01）。
- **根因**：combat.item.* 的 i18n key 经 chrome.rs 的 VS_ITEM_KEG/MEDKIT/RUM/SMOKE 常量被 vs_proto.rs L46-49 + vs_anim_v4.rs L613 消费（vs_duel 原型 = F-05 保留作动效骨架参考，渲染 vs 道具图标）。doccheck2 i18n 孤儿判定 = 静态扫 i18n_fmt 字面量，渲染层经常量间接引用 ⇒ 假孤儿（EXP-187 同类盲区：动态拼接/常量中转）。
- **解法**：删 i18n key 前 grep 代码消费点（含渲染层常量中转）+ 查 F-ID 登记（F-05 保留判定）。battle_fmt = 0 消费真删；combat.item.* = 活路径保留（4 条 WARN 登记，不静默清）。
- **验证**：grep -rn combat.item chrome.rs vs_proto.rs vs_anim_v4.rs ⇒ 4 常量 + 渲染调用；doccheck2 i18n 孤儿 WARN 4 条 = 已知保留面（非 FAIL）。

### EXP-198 流程：业务规格审计 4 分类处置 + 数值口径勘误（真漂移 vs 笔误 vs 设计空洞 vs 死代码）
- **现象**：docs/business/ 14 疑点审计：1338 金（progression §跨图节奏）vs 实测 338（board_measure 落格净金）= 数值漂移；boss_tiers 3 档（文档旧表）vs 20 档（balance 实际）= 笔误滞后；卡牌 HP 空洞（combat.md 删 HP 但 progression 仍写卡牌扣血）= 设计空洞；Landing::Battle（v5 无遭遇格但代码留分支）= 死代码。
- **根因**：文档镜像（progression 难度阶梯表 / 数值锚点）是手工同步的 v4 历史值，v5/v3/兵力模型大改后未回写；「见 BOSS 时金」口径演变（落格净金 vs 含领地通道）文档没标口径，读者误读。
- **解法**：① 逐疑点 grep 代码坐实（rules/ tests/ balance.json 三方）→ 4 分类处置（真漂移改值 / 笔误改措辞 / 空洞记现状待拍板 / 死代码删+登记）；② 数值声称点一律跑 board_measure.py 实测回写，禁抄文档旧值；③ 口径演变必须标「M6 口径」+ 旧口径勘误注。
- **验证**：doccheck 0 error + doccheck2 cross-doc-counts 0 + board_measure --check PASS（业务规格难度档位与 balance 一致）。4 分类 = 本轮 15 条勘误 + H1②/X2/M6 三项拍板的处置法。

### EXP-199 流程：迁移类重构须同步清「实现+数据+i18n+资产+测试+example+文档」六面，否则残留成死代码
- **现象**：S34.6 D5 删 HP 回合制后，card_state.hp 暂存通道（卡牌扣血）+ balance.board.battle_damage/battle_gold_loss 字段 + toast.battle_fmt i18n + board.rs Landing::Battle 落格分支 = 全残留（v5 无遭遇格，死分支不可达）；审计才暴露。
- **根因**：迁移类重构（HP→兵力模型）只清了 rules 层主路径（BattleSession/combat.items），未同步清「表现层死分支 + 数据段死字段 + i18n 死文案 + 落格死 arm」——六面中清了一面（rules），其余五面挂编译面长期漂移。
- **解法**：迁移类重构收口时，按六面清单系统摸排：① 实现（死分支/死字段/死函数）② 数据（balance 段死键）③ i18n（strings 死文案）④ 资产（ui 死图，查 vs 渲染活路径）⑤ 测试（断言死字段）⑥ 文档（规格旧口径）；逐层 grep 消费点 + 查 F-ID 保留判定（EXP-197）。本轮 = 删 battle_* 字段 + Battle 分支 + battle_fmt i18n + data_tables 断言 + 文档勘误。
- **验证**：grep -rn battle_damage/battle_gold_loss/board.card_state 全库 0 活消费（仅历史注释）；cargo test 990/0 + gate 36 项 0 failure；doccheck2 i18n 孤儿面 = 已知保留（combat.item.* vs 渲染活路径，EXP-197）。

### EXP-200 流程：迁移类口径对齐须同步清 rules 层（EXP-199 六面：app 改了 host.rs 但 rules/cards.rs apply_card 仍写 state.hp）
- **现象**：H1② 拍板卡牌 Damage/Heal 改人手，只改了 app 层 host.rs::apply_card_effect（seat_crew），但 rules 层 rules/cards.rs apply_card 仍写 CardState.hp（HP 暂存）；rules 层 = 卡牌效果唯一事实源，app 层实际不消费 apply_card 的 HP 路径（只消费 CardEngine 卡池 + 自己改 seat_crew）⇒ 两条路径口径分裂，rules 层死代码长期挂编译面。
- **根因**：H1② 拍板时只改了「表现层 host.rs」（app 侧），没同步「规则层 rules/cards.rs」（rules 侧）；EXP-199 六面清理只清了「实现/数据/i18n/资产/测试/文档」，漏了「rules 层口径」——迁移类重构（HP→人手）须同步清 app + rules 两侧，rules 层 = 效果结算唯一事实源，app 层 = 表现，两侧口径须一致。
- **解法**：① rules/cards.rs CardState 加 crew 字段（删 hp/hp_max）+ apply_card Damage/Heal 写 state.crew（saturating_sub/add，无上限对齐 app）；② 测试 reference_diff_cards.rs（state_with/state_clamped 构造器 hp→crew + 断言）+ exhaust_progression.rs（v.hp 档位映射 crew + 不变式改 gold ≤ gold_max）；③ cards_draw.rs 陈旧注释「保留复用」改 H1② 人手口径；④ doccheck2 第 52 项 card-crew-alignment 守门（CardState 含 pub crew + apply_card Damage/Heal 写 state.crew + 全 crates 0 处 state.hp 卡牌暂存，防回归）。
- **验证**：cargo test 990/0 + clippy 0 + doccheck2 52 项 0 error（card-crew-alignment 负例护栏：改回 state.hp 必 FAIL 精准点名 + 还原 0 error）。EXP-199 六面扩「七面」= +rules 层口径（app↔rules 双侧一致）。

### EXP-201 流程：专项审计 FIND subagent harness 故障兜底 + 现状声称点回写纪律（集成测试数/evidence 数随删除漂移）
- **现象**：测试体系专项审计 FIND subagent 两次 harness 故障（OpenAI delta missing id/name，与内容无关）；主线程手动 FIND 覆盖 12 必扫面，抓 23号 §0 两处现状漂移 + 1 覆盖粒度缺口。
- **根因**：① subagent harness 故障为工具侧不稳（连续两轮专项审计 FIND 均失败），非 prompt 内容问题——飞轮 R18 + 本轮均靠主线程手动补跑且更可控；② 105→103 = HP 本体删除 commit 2170700 删 3 测试文件（battle_rules/exhaust_combat/rules_battle）后 23号未回写（其它文档如 combat.md 回写了）；③ 757→560 = R15 回写取「全量 763 左右」口径，但括注写「排除 console/」（现全量 763 - console 203 = 560），口径自相矛盾。
- **解法**：① FIND subagent 故障 ⇒ 主线程手动 FIND（按必扫面逐条 grep，不依赖 subagent 落盘）；② 105→103 + 757→560 实测回写 23号 §0（含根因 + 历史快照链注释）；③ Q003 粒度：manifest 补 oracle_diff_cards/main_map/rng 3 条（28→31，L1 9→12，四个 reference_diff_* 独立实跑 11/8/7/4 passed），平台可逐项单独跑；④ 现状声称点回写纪律：文档里的「实测数字」（集成测试数/evidence 数/门禁项数）须随删文件/口径变更同步回写（同类 EXP-189 项号文字漂移 / EXP-192 数值声称点漂移）。
- **验证**：validate_data OK（31 条 manifest 对账）+ doccheck 0 + doccheck2 52 项 0 + ledger 351/351/0 + 4 个 reference_diff_* 独立实跑全过。

### EXP-202 流程：测试体系专项审计 27 CONFIRMED：平台 prove/证明渲染/徽章刷新 3 条规范承诺未实现（待拍板功能补完 or 降级措辞）
- **现象**：测试体系专项审计（2026-11-19）：FIND subagent 这次成功落盘 28 条（前两轮 harness 故障已恢复），AUDIT 子代理独立裁决 27 CONFIRMED / 0 REJECTED / 1 PARTIAL（Q027 措辞）+ 主线程独立复核 top 8 全 CONFIRMED（EXP-038）。
- **根因**：① Q009 manifest gate 项号 23/21/22 与 posmap 机读源错位（应 33/31/32）却过 validate_data（只校 1..=36 形状不校语义）⇒ 补 doccheck2 第 53 项 manifest-gate-posmap 守门；② Q012 我审前引入的 3 条 oracle_diff_* 未登记 console_commands ⇒ studio_tests --check exit 1（gate 25b FAIL）⇒ 补 3 条真映射（不扩豁免）；③ Q022 我给的 evidence=S20.2.log 是 v4 面板日志无关 ⇒ 改 3 个专属实跑日志；④ Q021 exhaust 夹具 hp/hp_max vs H1② crew 语义背离 ⇒ 夹具+ExhaustCardState 字段重命名；⑤ Q013/Q014/Q015 = 23 号 §6.2/§6.3 承诺（prove evidence/tests/<id>/<ts> 落点 / 本次运行 evidence[] 渲染 / ≤2s 跑中徽章）在 run_test/前端未实现——规范超前于实现，属功能补完或降级措辞的需人裁决项。
- **解法**：FIX 24 条（manifest gate 项号 / 专属 evidence / console 映射 / 夹具字段 / 文档数值回写 105→103、757→560、体积、P5 证据 MuMu→redroid、项数 19→52、51→52、§6.1 判据矛盾、§引用偏移、JobSpec timeout_s、分组计数措辞、gate.ps1 注释编号、waiver 死 id / 注释残留 / manifest 边界声明）+ doccheck2 第 53 项守门（负例护栏实跑 gate 33→23 必 FAIL + 还原 0 error）。剩 Q013/Q014/Q015 登记待拍板（功能补完 vs 规范降级），不擅自改平台前端。
- **验证**：studio_tests --check exit 0（3 类 OK）+ validate_data OK（31 条 manifest）+ doccheck2 53 项 0 error + clippy 0 + 4 个 reference_diff_* 独立实跑全过 + Q009 负例护栏实跑。
