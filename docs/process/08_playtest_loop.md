# 试玩闭环 loop（Playtest Loop）

**定位**：运行时试玩版飞轮。[`06_flywheel.md`](06_flywheel.md) 管**静态**审计（找文档↔代码/数据漂移），
本文管**运行时**缺陷（玩法/交互/异常），是 S16.2 单一事实源。
开轮必读、停机口径、回写三分类全部复用 06_flywheel（不重造），本文只定 4 角色与 4 份运行时报告。

## 1. 4 角色（一轮并发/串行）

| 角色 | 输入 | 输出 | 禁区 |
|---|---|---|---|
| **试戏 agent** | 开轮必读（`flywheel_readme.ps1` 输出）+ 4 份运行时报告 | 「缺陷清单」：每条 = 报告 + `文件:行`（或场景/按键序列）+ 类别（崩溃/交互失灵/状态错/数值异常）+ 复现步骤 | 不改任何文件 |
| **归因 agent** | 缺陷清单 | 逐条裁决三分类（下表），附理由 | 不改任何文件 |
| **修复 agent** | 归因结论（仅已裁决定论项） | 改码/数据/资产 + 补测试 + 自验（`cargo test -p <改动 crate>` + `clippy -D warnings` 全绿） | 不得改归因结论 |
| **复测** | 修复后 | 4 报告重跑 + 缺陷清单逐条关闭记录 | — |

## 2. 4 份运行时报告（试戏 agent 的数据源）

| 报告 | 载体 | 覆盖 |
|---|---|---|
| `fuzz_playtest` | `cargo test -p ti-app --test fuzz_playtest`（32 seed × 2000 步确定性随机输入） | panic / 场景判别值越界 / 存档往返损坏 |
| `full_run` | `cargo run -q -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json` | 全流程 16 步场景流转（开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单） |
| `visual_regression` | `cargo test -p ti-app --test render_invariants` + gate `render-invariants` 项（7 场景结构不变式断言，S18.7b 起无像素基线） | 场景层+精灵叠加结构不变式 |
| `web_playtest` | `node scripts/web_playtest.cjs`（S16.1；serve 需先在跑） | 浏览器真运行时：L1 实例化活体 + L2 渲染墨点 + L3/L4 按键场景流转 + console 致命错误=0 |

**聚合 = 全 0 失败即试戏 agent 一轮无新缺陷**（4 份任一非 0 = 该轮有新发现，开修复）。

## 3. 归因三分类（回写目标，沿用 06_flywheel §3）

| 缺陷类型 | 回写目标 | 判据 |
|---|---|---|
| 确定性测试缺陷（可稳定复现的崩溃/越界/存档损坏） | 修码 + 补测试 + `evidence/S16.2.log` | 复测 4 报告 0 失败 |
| 可机械判定的漂移（数值/常量/文案面量） | `doccheck2.rs` 新断言 | 下一条能自动 grep/断言复现 |
| 设计分歧 / 环境抖动（无 GPU、VM fps、CDP 不可用降级） | `docs/experience/` 新 EXP + `evidence/<task>.log` | 争议未定论，记录裁决理由 |

回写必须进门禁视野（06_flywheel §3 判据照抄：新 doccheck2 断言同步 checks 计数；新 EXP 登记 `docs/experience/index.md`）。

## 4. 停机条件（照 06_flywheel §4 口径）

- 4 报告全 0 失败 **且** 连续 **3 轮收敛**（新运行时缺陷 = 0 / 归因 = 0 未决 / 修复 = 0 残留）→ 停机；
- 停机结论**必须主线程独立复核**（EXP-038/058：子代理声称不算数），证据入 `evidence/S16.2.log`。
- 「找不到缺陷」本身不是停机条件（可能是还没跑全 4 报告）；**0 新发现 ×3 轮**才是。

## 5. 与 06_flywheel 的关系

| | 06_flywheel（静态） | 本文（运行时） |
|---|---|---|
| 找什么 | 文档↔代码/数据漂移（硬编码/越界/未实现） | 玩法/交互/异常（panic/场景流转/存档） |
| 开轮必读 | §2 六条 + `flywheel_readme.ps1` | **同款复用**（不重造） |
| 停机 | §4 三判据（0 新发现 / 0 误报 / 0 残留） | §4 3 轮收敛（0 缺陷 / 0 未决 / 0 残留） |
| 回写 | §3 三分类 | §3 同款三分类 |
| 台账 | 飞轮不写 verified | 本文写 S16.2 verified（由集成轮给，EXP 口径） |

## 6. 执行命令

```sh
# 4 报告一次跑全（试戏 agent 输入）
cargo test -p ti-app --test fuzz_playtest
cargo run -q -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json
cargo test -p ti-app --test render_invariants
# web_playtest 需 serve 在跑（gate 第 23 项同款前置）：
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080 &
node scripts/web_playtest.cjs
```

gate 第 24 项 `playtest-loop` = 4 报告聚合断言（全 0 失败 + 收敛记录在 `evidence/S16.2.log`）；
playbook `playtest-loop` 命令 = 一条命令出 4 报告 + 三分类清单。
