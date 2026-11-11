# 32 · CCGS 工作室制度借鉴（决策依据单一事实源）

> 本文 = CCGS（Claude-Code-Game-Studios）调研摘录 + 机制级对照 + 3 借鉴点 / 4 不借鉴的**决策依据**；
> WBS 单一事实源 = [`../plan-s50-studio-interface.md`](../plan-s50-studio-interface.md)（S50.0~S50.4）。
> 立项 = 2026-11-11 用户拍板「两者都做」（既出设计稿 + 又立台账 WBS）。

## 1. CCGS 调研摘录（实拉 GitHub 仓库证据，非营销描述）

- 仓库 = `Donchitos/Claude-Code-Game-Studios`（MIT，`is_template: true`；star 38k / fork 5k 量级；
  2026-02-12 创建，最近推送 2026-11-09，main <200 commit）
- 顶层结构实测：
  - `.claude/agents/` 49 个 `kebab-case.md`（frontmatter = `name/description/tools/model/maxTurns/memory/disallowedTools/skills`
    + 正文 3~18KB：角色定位 + 每 agent 内嵌 5 步协作协议 + `Reports to / Escalation / Sibling coordination` 层级写死在正文）
  - `.claude/skills/` 75 个目录（每目录 = 一个 slash 命令，`SKILL.md` frontmatter 含 `user-invocable/allowed-tools/model`，
    body 内嵌 `` !`bash ... resolve_config` `` 动态配置注入语法；`studio-tests/` 含 49 agent + 75 skill 对应测试 .md
    + `catalog.yaml` + `quality-rubric.md` = 对 skills/agents 本身的 QA）
  - `.claude/rules/` 14 个 `path-scoped` 红线 .md（frontmatter `paths:` glob 列表 + 该路径下编码标准，
    + `validate-write.sh` hook 写前机器拦截；14 个 = agent-memory / ai-code / data-files / design-docs /
    engine-code / gameplay-code / narrative / network-code / prototype-code / scene-files / shader-code /
    skill-authoring / test-standards / ui-code）
  - `.claude/hooks/` 18 个 .sh（15 事件挂载 + 2 helper + 1 opt-in）：
    - SessionStart：读 `production/session-state/active.md` 的 `<!-- CHECKPOINT -->` 块注入上下文
      （**状态文件即记忆**思想 = 压缩后恢复不靠对话）
    - PreToolUse `Bash`：`validate-commit.sh`（48KB！解析 shell 命令串里所有 `git commit` 段，防 here-doc 绕过）
    - PreToolUse `Write/Edit`：`validate-registry.sh`（registry 一致性）+ `validate-write.sh`（write-guard 越域写拦截）
    - PreToolUse `Read`：`validate-read.sh`（拒 40KB+ 文档整读）
    - PreCompact：追加 `production/session-logs/compaction-log.txt` 一行「compaction occurred at <ts>」
    - SubagentStart/Stop：`production/session-logs/agent-audit.log` 追加管道符格式审计行
    - 全部命令带 Git-for-Windows 检测包装（缺 Git Bash 自跳过，不硬失败）
  - 协作规则 = `.claude/docs/coordination-rules.md` 7 条 prose（垂直委派/横向咨询/冲突上报/变更传播/
    禁止越域改文件/按 section 读文档/不上游提 issue），`CLAUDE.md` 经 `@import` 每 session 自动加载
  - 模型分层 = `.claude/docs/model-tiers.md`（agent frontmatter `model:` 字段何时生效的表）；
    `rigor` 三档 `minimal→solo`（默认）/ `standard→lean` / `full→full` 展开 `review_mode`
  - 五步协作协议 = `CLAUDE.md` 根 + 每个 agent body 内嵌同一段（creative-director 359 行版）+
    `docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` 24 处命中

## 2. 机制级对照表（CCGS vs TreasureIsl 现行体系）

| 机制 | CCGS | 我们（TreasureIsl） |
|---|---|---|
| 角色分层 | 49 agent（总监 3 Opus / 主管 9 Sonnet / 专家 38 Sonnet/Haiku） | 22 号双审计 A/B + 飞轮 3 角色 + 临时 spawn 子代理，无 agent 配置文件 |
| 协作规则 | `coordination-rules.md` 7 条 prose | `05_module_parallel.md` §4 并行纪律 6 条（文件域不相交/契约全票/集成轮门禁/打回不代劳/环境事实前置/必读清单） |
| 路径作用域 | 14 个 `rules/*.md` `paths:` glob + `validate-write.sh` 机器拦截 | `plan/packages.yaml` 包白名单 + `check_module_contracts.py`（gate 第 16 项）；**无「路径下编码红线」文件层** |
| 过程 Hook | 15 事件（commit 前 / 写前 / 压缩前后 / 子代理审计） | 无过程 hook；收口时刻 = `gate.ps1` 35 项 + `check_ledger_reality` 三方对账 |
| 停机判据 | 无显式（5 步协议 = 问→方案→拍板→草稿→批准） | 06_flywheel §4 三条（找问题 0 / 审计 0 误报 / 修复 0 残留）+ 主线程复核（EXP-038/058） |
| 审计日志 | `agent-audit.log` 管道符格式 | `evidence/console/audit-<task>-r<N>-<role>.md` 四行机器可解析字段 |
| 防虚报 | `code-review/SKILL.md`「NOT ASSESSED — NO DATA」+ `validate-commit.sh` | `check_ledger_reality`（gate 第 14 项）+ `evidence/<task>.log` 必须含任务自己判据 |
| 成本控制 | 模型 pin + 3 档 rigor | -Fast/-Touch/full 三档 gate（99% 时间在 8 个 cargo 项上），无模型 pin |

**结论**：CCGS 强在**过程约束**（写码时有人/有 hook 盯着）；我方强在**收口约束**（做完必须出示证据 + 机器核验）。
两者正交，不冲突——S50 只补过程约束的 3 个缺口。

## 3. 3 个借鉴点（按价值排序）

### ① 状态文件即记忆（价值最高，S50.1）

CCGS `active.md` CHECKPOINT 块 + SessionStart 重读 + PreCompact 追加日志行——核心思想 = **状态文件即记忆，
不靠对话续接**。我方 06_flywheel §4「主线程复核停机结论」是事后机制，无「会话压缩前保存状态」动作。
**借鉴**：`docs/loop.md` 加「长会话压缩纪律」节：压缩前主线程把 3 要素（已读清单/在途包白名单/下一步动作）
写进 `evidence/<task>.state.md`，压缩后第一件事读该文件续接，禁止凭对话摘要续接。触发条件 = 已读规范 >20 份
或 单任务跨 2 个 P 包以上或主线程自觉「快要压缩」。

### ② 元测试面（价值中，S50.2）

CCGS `studio-tests/` = 对 skills/agents 本身的 QA（49+75 对应测试 + catalog + rubric），行业少见。
我方 `assets/data/test_manifest.json` + 平台 `/api/tests` 是 L1~L4 游戏测试（被测对象 = 游戏），
**没有「对 doccheck2 41 项断言 / 对 35 项 gate 本身」的元测试面**——`dod-commands`（gate 第 25 项）
只覆盖「判据载体缺失」，不覆盖「门禁断言自身被正确触发」。
**借鉴**：新增 `scripts/studio_tests.py`（工具契约 4 条全满足），3 类断言：
1. doccheck2 每个 `fn check_*` 在注册表被引用 + 在 `gate_data.json` `doccheck2_checks` 有条目（双向对账）
2. `test_manifest.json` 每条 command 首词 ∈ 程序白名单 + 在 `console_commands.json` 有 id 映射（缺映射 = 平台面未覆盖）
3. `scripts/*.py/.ps1` 每个文件在 14 号表格行 + 04 速查 + README 索引 三处均登记（工具台账三处登记机器核验）
并入 gate 第 25b 项（收口档实跑，开发档 SKIP）。

### ③ 路径作用域红线文件（价值中低，S50.3）

CCGS 14 个 `rules/*.md` 的 `paths:` frontmatter + 该路径下编码红线正文 + hook 写前拦截模式。
我方最接近 = `16_code_standards.md` CS-1~6 全局标准 + `code_budget.json` 棘轮基线 + `process_ratchet.json` 豁免表，
**缺「按目录域的红线文件」层**。
**借鉴**：新增 `docs/rules/` 目录 5 个红线文件（按 `plan/packages.yaml` 白名单域划分：
`ti-core` / `ti-engine` / `ti-app scenes` / `scripts` / `assets-data`），frontmatter `paths:` glob + 正文 4~6 条红线；
门禁面 = doccheck2 第 43 项断言「5 文件齐全 + `paths:` 非空 + glob 合法 + 正文非空」。
**注意**：这层是「给 LLM 子代理开轮读的红线速查面」，机器拦截仍走既有 `check_module_contracts.py`
（gate 第 16 项，白名单机制），不重复造 hook。

## 4. 4 个不借鉴点（决策留痕，不立任务）

| CCGS 特性 | 为什么不抄 |
|---|---|
| 49 agent 层级（总监/主管/专家 3 层 + 垂直委派/横向咨询） | 项目规模 = 2 主线程 + 2 审计 + 3 飞轮 + 临时 spawn 足够；49 角色 = 过度设计；台账本身已替代「制作人协调变更传播」 |
| 75 个 slash 命令（/brainstorm /dev-story /code-review ...） | gate 35 项已等价覆盖质量阶段；75 命令是过程脚手架，我方是收口脚手架，多脚手架反而让 LLM 走流程忘了「必须实测」 |
| 12 个 commit/push 前 hook（`validate-commit.sh` 48KB） | 我方 commit 全由人工/主线程做，无「LLM 子代理擅自 commit 绕过」风险面；收口时刻 = gate 全绿 + 三方对账，比 commit hook 更彻底 |
| 5 步协作协议逐 agent 内嵌 | 我方无静态 agent 配置文件（子代理临时 spawn + 开轮必读清单 + 规范回执），**逐 agent 内嵌形态做不了也不需**；但 CCGS 五步协议（Ask→Options→Decision→Draft→Approval）的**思想**值得借鉴 —— S51.0 起**形态借鉴**：固化进 `03_templates.md` §7 任务书模版「开轮协作五步」段 + `06_flywheel` §2 开轮必读第 7 条（非 49 个 agent 文件，而是子代理任务书模版段，和我方既有载体一致） |

## 5. 一个隐藏风险点（CCGS 没解决，我方 S50.2 部分解决）

CCGS `studio-tests/` 元测试思想 = 测「测代码的测试」本身。我方 `doccheck2` 41 项断言本身可能写错/漏写
（EXP-170/171 复发防护 dod-commands 部分覆盖「判据从未执行」类缺陷），**缺「门禁断言自身被正确触发」的元层验证**。
S50.2 的 3 类断言（§3②）即此面的机器化落地。

## 6. 登记与 WBS

- WBS = [`../plan-s50-studio-interface.md`](../plan-s50-studio-interface.md)（S50.0 立项 → S50.1/2/3 三路并行 → S50.4 收口）
- 本文（32 号）= 决策依据；WBS = 执行面；两者不重述结论（硬规则 6 单一事实源）。
- 登记：`docs/README.md` 索引 + `AGENTS.md` 路由表 S50 行 + `plan/ledger.yaml` S50.0~S50.4 5 条。
