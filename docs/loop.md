# 循环协议（Loop Protocol）

> 注：本文的「基线」记述属**历史**：像素基线机制已于 **S18.7b 退役**（三套 golden PNG + 三个 diff 测试均已删除）；
> 现行机制 = 结构不变式断言 + CPU↔Bevy 内存对拍。

进度推进的唯一机制。任何会话开工前先读本文 + `plan/ledger.yaml`，结束时台账必须是合法状态。

## 1. 台账 `plan/ledger.yaml`

```yaml
version: 1
tasks:
  - id: S0.1              # 对应 docs/plan-s0-s2.md 中的任务 ID
    title: 建 cargo workspace + 门禁
    status: pending       # pending | in_progress | blocked | verified
    deps: []              # 依赖任务 ID，全部 verified 才可取用
    dod: "scripts/gate.ps1 全绿（当前 22 项，随 S8/S9/S10 演进）"       # 验收判据（抄自 WBS）
    evidence: null        # verified 后填 evidence/<id>.log 相对路径
    blocked_by: null      # blocked 时填原因
    rounds: 0             # 已失败验收轮数，>=3 必须拆子任务
```

规则：
- `status` 只能按 `pending → in_progress → verified` 单向流转；回退只能到 `in_progress`，并写 `blocked_by`。
- `blocked_by` 约定：`blocked` 任务写**字符串原因**（如 `android-build-on-mac-not-this-machine`），非 `null`；`pending`/`verified` 任务为 `null`；`in_progress` 任务默认 `null`，但**被外部阻塞**（如渲染线未提交 WIP 致 gate 非全绿、视觉基线未重生成）时允许写非 `null` 原因（S15.2 先例；台账校验器 `ledger` 不禁此组合，`status` 保持 `in_progress` 而非 `blocked`——渲染线收敛 + 复测后转 verified，EXP-078 归因口径）。
- `verified` 必须有 `evidence`，否则台账校验（gate 第 8 项）失败。
- `rounds >= 3` 的任务必须拆成子任务（新增 `Sx.y.a` 形式 ID）后才可继续。

## 2. 循环体（每轮迭代）

```
1. 读 ledger → 选第一个 deps 全部 verified 且 status != verified 的任务
2. 实现（按 WBS 产出物落地，禁止留占位实现）
3. 跑该任务的验收命令 → 输出存 evidence/<id>.log
4. 跑 scripts/gate.ps1（全局门禁，22 项）
5. gate 全绿 → ledger 置 verified + evidence；否则 rounds+=1，按规则拆解/标记 blocked
6. 输出进度报告：verified/total、阻塞项、下一轮目标
7. 回到 1
```

## 3. 终止条件（全部满足才算项目完成）

1. ledger 中全部任务 `verified`；
2. `docs/parity.md` 的 P0~P4 五级标准全部 pass（各自校验命令产出报告存 `evidence/`）;
3. 三端产物生成：桌面 exe、wasm 站点、APK；
4. `scripts/gate.ps1` 22 项全绿，占位扫描 0 命中。

## 4. 卡死与回退

| 情况 | 处置 |
|---|---|
| 验收连续失败 3 轮 | 拆子任务，原任务转 `blocked`，`blocked_by` 写具体缺口 |
| 发现计划与实测冲突 | 以实测为准 → 更新 `legacy/docs/facts/index.md`，再改 WBS/台账，**不许静默绕过** |
| 踩到新坑（工具/格式/平台） | 立即写入 `docs/experience/index.md`（格式见该文件），并在任务 evidence 中引用 |
| 需要暂停（外部依赖缺失） | 任务标 `blocked`，继续推进无依赖任务，不空转 |

## 7. 模块并行轮（2026-10 起，S12 生效）

任务 ≥ 2 个且**文件域不相交**时，不走 §2 串行取件，改走并行轮。
唯一事实源 = [`process/05_module_parallel.md`](process/05_module_parallel.md)（一轮 4 步：
契约冻结 → 多包并行 → 集成联调 → 记账；包白名单 = `plan/packages.yaml`；任务书模板 =
`03_templates.md` §7；机器判定 = gate 第 15/16 项）。

与 §2 的分工：§2 管「单任务何时开工」，§7 管「多任务怎么同时开工」。
§6 多会话并行规则（文件域 + 分支模型）在 §7 下仍全部适用。

## 8. 会话交接

- 会话结束前：ledger 合法、`evidence/` 无缺失、经验库已回写。
- 会话开始时：先 `git status` + 读 ledger + 读 `docs/experience/index.md` 最近条目，避免重复踩坑。

## 6. 多会话并行规则（2026-10 起）

多会话可同时推进**文件域不相交**的任务。基线提交：`b07353b`（各会话从该点开分支）。

### 6.1 分波次与文件域（当前波次）

| 波次 | 会话 | 任务 | 独占文件域（其他会话不得改） |
|---|---|---|---|
| 1 | A | S0.2 → S0.4 → S1.4b | `legacy/scripts/ida_dump.py`、`ref/`、`legacy/docs/facts/index.md` |
| 1 | B | S2.6 | `scripts/` 中 android 相关新文件、`scripts/gate.ps1` 的 android 行 |
| 1 | C | S2.7 | `scripts/smoke.ps1`（本机无 bash，见 EXP-011） |
| 2 | D | S0.3（等 S0.2 verified） | `ref/classify.json`、`crates/ti-tools` 的 classify bin |
| 3 | 按 S3 裂变 | S3.1 后：主链 / S3.10 / S3.12 三路并行，S3.6/7/8 三模块并行 | 每模块独立的 `legacy/docs/spec/<m>.md`、`oracle/<m>.rs`、`rules|<m>.rs`、`tests/diff_<m>.rs` |
| 4 | S4/S5 收口 | S4.1→S4.4 → S5.1→S5.6 → S6.1→S6.5（呈现层完善 + 资源命名修正） | `crates/ti-app`（GameApp/sprite_atlas/terrain_atlas/渲染）、`crates/ti-formats/src/resource_names.rs`（S6.5）、`docs/release.md`、`docs/modernization.md` §9 体积表、`evidence/` 对应 log |

波次推进以 ledger 为准：deps 未全 `verified` 的任务不许开工（loop 第 1 步）。

### 6.2 硬规则

1. **分支模型**：每会话开工前 `git checkout -b task/<id>`（从基线或最新 main 切出）；完成任务、gate 全绿后合回 `main`（fast-forward 或普通 merge，不许 `--force`）。合并冲突由发起合并的会话解决。
2. **ledger 只改自己任务的行**：状态流转、`evidence`、`rounds`、`blocked_by` 仅限本会话认领的任务 ID；不重排文件、不改他人条目。合并时 ledger 冲突按「两边各保留自己任务行的最终态」处理，必要时合并后跑 `cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` 校验。
3. **gate 的归属判定**：gate 22 项是全 workspace 检查，别人半成品代码可能导致 FAIL。工作途中可随时跑 gate 自检；**但只有当你任务的产出物已落地、且 FAIL 项与你文件域无关时，才可忽略该 FAIL**。置 `verified` 前必须：已合并（或 rebase 到最新 main）且 gate 22 项全绿。
4. **共享文件按「最后合并者负责对齐」**：`crates/*/src/lib.rs`、`mod.rs`、`Cargo.toml`、`AGENTS.md` 等不可避免的注册点，改动最小化（只加自己的一行注册），合并冲突就地解决后必须重跑受影响的测试。
5. **cargo 并发**：`target/` 目录锁会让并行构建排队，属正常现象，不要杀掉对方的 cargo 进程。
6. **经验与事实库**：`docs/experience/index.md`、`legacy/docs/facts/index.md` 可能被多方追加——只在文件**末尾追加**自己的条目（不改动既有行），冲突概率最低；已有的坑不许静默绕过。
7. **S3 模块 8 子步固定产出物**（loop §6.1 波次 3 起）：每模块必产 `legacy/docs/spec/<m>.md` + `ref/functions_<m>.csv`（只列本模块函数，**不改动原 `ref/functions.csv`**）+ `crates/ti-core/src/oracle/<m>.rs` + `rules/<m>.rs` + `tests/diff_<m>.rs` + `evidence/S3.x.log`。oracle 与 impl 用**同一批公开类型/常量**（枚举、整数像素、错误型），仅「自由函数 vs struct 方法」结构不同；diff 测试遍历状态×事件全组合 + ≥2 语义测试，用 `.expect("...")` 不用 `.unwrap()`。**公式/常量/类型单一源**（EXP-043）：纯函数（LCG/FNV/哈希）只写一份，oracle 侧不重复抄写公式，diff 对比的是「自由函数 vs struct 方法」行为而非「公式 A vs 公式 B」。（LCG 指 S8.7 前 oracle 参照；当前 rules 侧已库化 PCG + `fnv`。）
8. **子代理 verified 复核**（EXP-038 实测回退教训）：子代理报告「已 verified」后，主线程必须独立复核：(1) 该模块 5 类产出物文件 `Test-Path` 真实存在；(2) 实跑 `cargo test -p ti-core --test diff_<m>` 看 `test result: ok`；(3) 重读 ledger 该行 `status: verified` + `evidence` 非空。三者齐备才认账，缺任一补写/回滚并重跑 `scripts/gate.ps1`。子代理改 ledger **只许 edit 工具按行替换**，禁止整文件重写（会冲掉并行子代理刚置的 verified）。
