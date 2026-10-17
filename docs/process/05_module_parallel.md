# 模块并行开发（Module-Parallel Loop）

**定位**：把「真实程序员的开发节奏」固化成可复用 loop：
**划分模块边界 → 各写各的 → 最后集成联调**。单任务串行循环见 [`../../loop.md`](../../loop.md) §2；
本文是**并行轮**的唯一事实源（`docs/plan-s11-forward.md` §6 只作指针）。

**执行单元（2026-10 决策）**：**并发会话 + 任务分支**。
主线程做契约冻结与集成联调；每个并行包由**一个会话 + 一支 `task/*` 分支**领取，
从基线标签切出，包内自检后合回主干。
子代理在包内**只做审计/调研**（不得写业务代码、不得改台账）——
防虚报判据见 [`../../loop.md`](../../loop.md) §6.2 第 8 条。

## 1. 何时开并行轮

| 条件 | 判定 |
|---|---|
| 包数 ≥ 2 | 少于 2 个包走单任务循环（防「假并行」） |
| 包间无依赖环 | `plan/packages.yaml` 的 `depends_on` 必须无环（契约轮人工核对） |
| 包边界已冻结 | 白名单互斥 + 跨包面已写进 [`docs/contracts/`](../contracts/00_index.md) |
| 基线干净 | 从 `baseline-<轮次>` 标签切分支；工作树有未提交改动 = **不许开轮** |

## 2. 一轮 = 4 步

```
① 契约冻结轮（主线程，1 人）
     冻结跨包公共面 → docs/contracts/*.md 全部 status: frozen
     锁包白名单     → plan/packages.yaml（两两交集 = ∅）
     逐包下发任务书 → docs/process/03_templates.md §7 模版填充
        ↓
② 并行开发轮（N 会话并发，各 1 包，文件域不相交）
     只改白名单内的文件；包内自由设计
     包内自检：cargo fmt/clippy/test（本包）+ 越界 grep = 0 + 业务可测规则表逐条落
     只报「包内绿」，**不写台账 verified、不写 evidence**
        ↓
③ 集成联调轮（主线程）
     按 packages.yaml 依赖序合并分支
     全 workspace build + test → P3-3 全流程 16 步 → gate.ps1 全项
     契约漂移 → 打回该包（§7），禁止主线程代劳改他包文件
     全绿 → 逐包回写 evidence/<task>.log + 台账置 verified
        ↓
④ 记账与经验回写
     evidence 实跑输出入台账；踩坑写 docs/experience/ 新 EXP 条
        ↓
     下一轮（冻结下一批包）
```

## 3. 包划分（按 `crates/ti-core` 真实布局；机读副本 = `plan/packages.yaml`）

| 包 | 域 | 文件白名单 | 依赖 |
|---|---|---|---|
| **D 数据/工具**（先行） | 数值/文本/布局/控件表 + 校验脚本 | `assets/data/*.json`、`crates/ti-core/src/data/**`、`scripts/validate_data.py`（FIND-201 统一入口）、`legacy/scripts/gen_layout.py` | 无（契约轮内完成） |
| **P1 战斗** | 战斗会话与结算 | `crates/ti-core/src/rules/battle.rs`、`crates/ti-core/src/domain/combat.rs`、`crates/ti-core/tests/rules_battle.rs` | D |
| **P2 经济** | 贸易 + 建造 + 银行 | `crates/ti-core/src/rules/{trade,build}.rs`、`domain/economy.rs`、`tests/{rules_trade,rules_build}.rs` | D |
| **P3 地图/卡牌** | 网格移动、据点、抽牌、跨图推进 | `crates/ti-core/src/rules/{main_map,cards,cards_draw,progression}.rs`、`domain/map.rs`、对应 tests | D |
| **P4 流程菜单** | 主菜单/选角/选图/消息/渲染 | `crates/ti-core/src/rules/{menu,selection,messages,main_loop,render}.rs`、`domain/{flow,selection,messages,render}.rs` | D |
| **P5 存档 + RNG** | 存档 v2 + 迁移 + 可复现随机 | `crates/ti-core/src/rules/{saveload,saveload_json,rng}.rs`、`domain/persistence.rs`、`tests/{rules_saveload,rules_saveload_json,save_migration}.rs` | **P1–P4 状态结构稳定后** |
| **P6 表现层**（后续轮） | 场景/控件/渲染/输入 | `crates/ti-app/src/{scenes,widgets.rs,touch.rs,menu_shell.rs}`、`crates/ti-platform/src/**` | P1–P5 会话 API 冻结 |

**包依赖铁律**

1. 包内**零跨包 import**：只用已冻结契约里的公共类型（`ti_core::domain::*`、`rules::rng::GameRng`）
   与 `assets/data/*.json` 的 key；
2. 跨包协作只经两条通道：**契约里的公共 struct 签名** + **`balance.json` / `strings.json` 的 key**；
3. 公共 struct 冻结后要改 → 全票同步（主线程仲裁），集成轮发现漂移 = 该包打回；
4. 表现层不反向依赖玩法内部：`ti-app` 只经 `SceneHost` 与 `rules`/`domain` 公开面。

## 4. 并行纪律（防串行退化的 5 条硬规则）

1. **文件域不相交**：包白名单两两交集必须为空（`plan/packages.yaml` 机器可判，`check_module_contracts.py` 第 1 条）；
2. **契约全票**：`docs/contracts/*` 的 `frozen` 段只由主线程改；包内请求改契约 → 走仲裁，不许私改；
3. **包内可单独绿**：每包必须有一条**不依赖他人未冻结产物**的验收命令
   （`cargo test -p ti-core --test <本包测试>`），否则不许开轮；
4. **集成轮门禁**：③ 不过（全 workspace 绿 + P3-3 + gate 全项）**不许**置 verified；
   子代理/会话只能报「包内绿」，verified 由集成轮给；
5. **打回而非代劳**：集成失败定位到包 → 该包自己改；主线程不得进他包文件（否则下次不敢并行）。
6. **环境事实前置（EXP-064）**：下发子代理任务书时，prompt 必须附带一段「环境事实」
   （工作目录真实路径 + crate 前缀 `ti-` 不是 `ty-` + 关键命令 + 本包白名单 + 当前在途并行包清单），
   子代理**不得**靠摸目录/试错确认命名；缺少该段 = 任务书不合格，打回主线程补全。

> 并发副作用（正常现象，不算违规）：`cargo` 的 `target/` 锁会让并行构建排队；
> 多个会话会同时改 `plan/ledger.yaml` —— 只许**按行替换**自己的任务行，禁止整文件重写（EXP-038）。

## 4.5 开轮必读清单 + 规范回执（飞轮增强，EXP-068）

单一事实源：[`06_flywheel.md`](06_flywheel.md) §2（开轮必读 6 条）。本节只做并行轮侧的接入要求。

1. **下发侧**：下发子代理任务书时，prompt 头部必须附「开轮必读清单」6 条
   （内容指向 `06_flywheel` §2，不在此复述）+ 本轮在途 P 包白名单（第 6 条）。
2. **回报侧**：子代理回报必须含「规范回执」段：逐条列出读了哪些文档 +
   命中几条现行规范 + 是否读到与本包域相关的 EXP。
3. **打回判据**：缺开轮必读条目的任务书、或无规范回执的回报 = 不合格，打回；
   模板见 [`03_templates.md`](03_templates.md) §7「开轮必读 + 回执」段。
4. 飞轮角色（找问题/审计/修复）与并行包会话共用同一份清单；飞轮自身的
   回写规则与停机条件见 `06_flywheel` §3/§4，不在本文展开。

## 5. 台账与证据口径

| 角色 | 能写什么 | 不能写什么 |
|---|---|---|
| 包会话 | 白名单内代码 + `evidence/S12.<n>.log`（包内自检输出） | 台账 `verified`、`docs/business/`、契约 |
| 主线程（集成轮） | 台账 `verified`/`evidence`/`rounds`、契约、business/process 文档 | — |
| 子代理（审计） | 审计结论、`docs/experience/` 草稿 | 任何业务代码、台账 |

- 每包任务的 `dod` 必须含**包内判据**（本包测试命令 + 越界 grep = 0 + 业务规则表条目）；
- `evidence` 由集成轮统一 tee（含 gate 输出 + P3-3 输出），包会话的包内 log 作为附注。

## 6. 与其他流程的分工

| 流程 | 文件 | 何时用 |
|---|---|---|
| 单任务循环 | [`../../loop.md`](../../loop.md) §2 | 改动 ≤ 8 文件、无并行面（改门禁/改脚本/改单个模块） |
| **并行轮** | 本文 | ≥2 个互不相交文件域的模块同时推进 |
| 审计修复环 | [`../pipeline.md`](../pipeline.md) | 测试/审计发现漂移后的批量修复 |
| 门禁 | `scripts/gate.ps1` | 每一轮集成轮必跑；第 15 项查业务闭环、第 16 项查包白名单/契约 |

## 7. 打回与拆包

| 情况 | 处置 |
|---|---|
| 包内验收连续失败 3 轮 | 拆子包（`S12.<n>.a/.b`），原任务转 `blocked` + `blocked_by` 写缺口 |
| 集成轮发现契约漂移（公共 struct 被单方面改） | 该包打回，按 §4 第 2 条全票同步后重跑 |
| 包之间出现新依赖（白名单要重叠） | 停止并行 → 回契约轮重划包边界 → 重新冻结 |
| 一包长期阻塞其他包 | 把被依赖的那部分拆到先行包（走 D 包模式：契约轮内完成） |

## 8. 与台账的对应

- 每轮在 `plan/ledger.yaml` 登记：`S<轮>.0` 契约冻结 → `S<轮>.1..N` 各包 → `S<轮>.<N+1>` 集成联调；
- 终止条件见 [`../../loop.md`](../../loop.md) §3；并行轮不改变终止条件，只改变达成路径。