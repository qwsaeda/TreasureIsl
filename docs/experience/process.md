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
- 解法：统一用 LCG 式确定性公式 `seed = seed.wrapping_mul(K).wrapping_add(C)`（S3.8 用 K=2654435761 / C 见 oracle/battle.rs，S3.12 全局 RNG 用 MSVC LCG K=214013 C=2531011，见 EXP-037），**不引 rand crate、不用 SystemRandom**。播种点（毫秒时间种子）不进 oracle，由调用层注入；oracle 只负责「给定种子 → 确定序列」。
- 验证：diff_battle 1650 组合（11 seed × 5 tier × 6 HP × 5 atk）oracle/impl 逐位一致 + semantic_deterministic_output 同 seed 两次独立调用逐位相同。
- 教训：**oracle 层一律确定性 LCG（wrapping_mul/add，整数），禁 rand/SystemRandom**；具体常数写进各模块 spec（battle.md/battle.rs 头注释），S3.12 统一全局 RNG 时核对是否同一套常数，避免各模块各用一套。


