# S32.0 第 3 轮审计 · B（一致性与风险）

> 落盘说明：本文件为**第 3 轮审计 findings 的落盘凭证**（会话内审计报告的结构化汇总）。
> 审计方式：只读，不改任何文件；**未跑 `scripts/gate.ps1`**（其第 23 项会改写 git 跟踪的
> `evidence/web_playtest.*`，见新 P1-A）。实跑：`doccheck` / `doccheck2` / `ledger` /
> `check_code_standards.py --check` / `check_ledger_reality.py` / `doc_numbers.py`。
> 审计对象：S32.0 第 3 版（22 号 + WBS + 台账）。

## 总判定（第 3 轮）

**FAIL** —— 第 2 轮 findings P1 4/4、P2 12/12 **CLOSED**；新发现 **P0×0 / P1×2 / P2×19 = 21 条**。

## P1 findings（2 条）

| ID | 问题 | 修复落点（第 4 版已落） |
|---|---|---|
| P1-A | §3.1「`gate.ps1` → 只读（不改仓库文件）」= **事实错误**：第 23 项 browser-playtest 无 `Tier` 守卫（三档都执行），`scripts/web_playtest.cjs` 写 `evidence/web_playtest.log` + `.png`（均 git 跟踪）⇒ 违反「唯一写点」承诺，且 §8 冻结路径不含 `evidence/` ⇒ 抓不到 | §3.1 写点表逐条列明；§8 增「门禁既有副作用路径：只记录不判定 + 必须写进执行日志」 |
| P1-B | S32.10「不计入 loop 终止条件」在唯一事实源 `docs/loop.md` §3/§1 **无对应条款** ⇒ 终止条件不可满足 + `blocked_by` 约定破线 | **S32.10 移出台账**（P3 只留在 WBS，需要时另立项）；WBS 补「P3 不入台账的理由」 |

## P2 findings（19 条）

| ID | 问题 | 修复落点（第 4 版已落） |
|---|---|---|
| P2-A | §8「0 游戏产物变更」绝对口径与并行会话纪律冲突（当前实测即 FAIL） | §8 改增量口径（动作前后快照之差） |
| P2-B | `docs/README.md` 收敛口径陈旧（「连续 2 轮 0 新发现」） | README 改「0 新发现 + 0 误报 + 0 残留且连续 2 轮，见 22 号 §7.4」 |
| P2-C | 硬编 `pwsh` ⇒ 只有 PowerShell 5.1 的 Windows 机不可用 | §4.3 改为候选顺序 `pwsh` → `powershell` |
| P2-D | §4.2 与 `console_commands.json` 两份「唯一」白名单且无对账 | §4.2 标「人读视图」+ JSON 标「机器真源」+ S32.6 DoD 加对账断言 |
| P2-E | cargo bin 登记落点指向只收 `scripts/` 的 14 号 §3.2（无 cargo bin 先例） | §12 改指 `docs/process/04_tooling.md` §1 的 crates bin 行 + README 索引；S32.1 DoD 同步 |
| P2-F | token 落盘路径缺 Windows 口径；用户不知从哪取 token | §10.1 补 Windows 路径 + `ti-console token` 子命令打印 + 启动日志只打印路径 |
| P2-G | 保留策略删的是 git 跟踪的证据文件（仓库长期脏） | §10.4 运行期日志落 `evidence/console/logs/`（gitignored）+ 只删未入库文件 + 审计报告永不自动删 |
| P2-H | 单实例锁互斥范围表述过强（不互斥人工直跑、不跨 worktree）；陈旧锁判定过简 | §4.1 + §10.4 收窄措辞 + 陈旧判定加「超 30 分钟」 |
| P2-I | 「子进程环境变量白名单透传」无最小集 ⇒ 门禁跑不起来 | §10.3 列最小集（PATH/HOME/USERPROFILE/CARGO_HOME/RUSTUP_HOME/TI_CHROMIUM_PATH/TI_CONSOLE_TOKEN） |
| P2-J | 第 2 轮改写丢了「审计报告由谁写」 | §7.3 step 3 补「由主线程写回（平台亦可粘贴归档）」 |
| P2-K | §7.4 的「3 轮未过拆子任务」在台账规则下会红（`rounds >= 3` 校验 error） | §7.4 补「拆子任务后原行删除或 `rounds` 归零」+ 实际拆 `S32.0a`/`S32.0b` |
| P2-L | §3.1 说 `exp_log` 可写 `docs/experience/`，§4.2 只开放 `--check` | §3.1 写点表标注「当前不开放写入入口」 |
| P2-M | S32.1 DoD 未含 `AGENTS.md` 登记；且 AGENTS.md crate 清单已陈旧（6 → 实为 7） | S32.1 DoD 补 AGENTS.md 同步 + AGENTS.md 本轮已修正为 7 crate |
| P2-N | 仓库根 `node_modules/` 未被忽略（`git add -A` 会入库 18 MB） | `.gitignore` 补 `/node_modules/` |
| P2-O | WBS「严格按阶段」与 deps 图不一致（P2 三任务 deps = S32.3） | WBS 改「按阶段门顺序验收（同阶段任务可并行，见台账 deps）」 |
| P2-P | 「不做完整构建」与 P3 Compose 部署之间口径缺口（镜像是否含 wasm 构建） | WBS §P3 补边界：部署不含游戏构建，`web/` 用既有产物挂载 |
| P2-Q | 台账 S32.4 引用「22 号 §4.2」但 §4.2 当时没有「fast 档 ≠ 全绿证据」这句 | §4.2 补该口径（与 `gate.ps1` 的 Tier 语义对齐） |
| P2-R | §10.2「仓库根前缀」未规定比较对象要 canonicalize（仓库根可能是符号链接） | §10.2 明确比较对象 = `canonicalize(repo_root)` |
| P2-S | 平台 `web/` 静态资源（JS/CSS）不在任何既有门禁扫描面 | §11.2 补「覆盖边界」+ 由 S32.3 判据兜底 |

## 范围核查（无越界）

游戏编辑器/关卡工具/运行时调试器、游戏运行时（wasm/渲染/玩法/数值）、多租户/账号/计费、
LLM SDK/模型调用 —— 均**未混入**（22 号 §1/§9 + WBS §不做 逐条排除）。
