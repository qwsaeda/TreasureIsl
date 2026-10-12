# 台账三方对账（S10.0 增 gate 第 14 项，2026-10 起进 `scripts/gate.ps1`）

> **目的**：验证台账 `plan/ledger.yaml` 里每条 `status: verified` 的任务，
> 其 `evidence` 文件**真实存在且非空**，且 dod 声明的关键产物在代码里 grep 得到。
> 防止子代理虚报（EXP-034 / S9.5 命令模式虚报 + S10.3 台账 83→81→86 漂移 的教训）。
>
> **用法**：
> ```bash
> python scripts/check_ledger_reality.py            # 校验全部 verified 任务
> python scripts/check_ledger_reality.py S10.1      # 校验指定任务
> python scripts/check_ledger_reality.py --report   # 打印明细（恒 0，供报告用）
> ```
> 门禁模式：任一 (a)/(b) 不满足 → 退出码 1，FAIL。

## 校验对象

| 层 | 来源 | 判定 |
|---|---|---|
| 台账 | `plan/ledger.yaml` 所有 `status: verified` 任务 | 取 `id` / `dod` / `evidence` 字段 |
| evidence | `evidence/<文件名>.log` | (a) 文件存在 (b) 非空 (c) 非纯占位 |
| 代码 | `crates/*/src/**/*.rs` + `scripts/*.py` + `scripts/*.ps1` | dod 声明的产物 grep 得到 |
| evidence 反向 | `evidence/*.{log,txt,md,json,png}` 中以任务 ID 命名的文件 | (c) 其任务 ID 必须在台账存在（S13.2 增） |

## 判定细节

### 1. 非纯占位判据（`is_placeholder`）

evidence 文件须**至少一条**：

- 含 `test result: ok` / `All tests passed` / `cargo test` 等**测试通过标记**；
- 或含 `PASS` 门禁项标记（gate 输出 `PASS ...`）；
- 或含 `=== gate` / `gate:` / `failures` 等 **gate 完整运行**标记；
- 或含 `16/16` / `0 failure` / `0 error` 等**全流程通过**标记；
- 或含 `realpath` / `sha256` / `hexdump` 等**产物验证**标记。

只有"任务描述复述 + 引用其他 evidence"而无以上任何标记 → 判为纯占位 → FAIL。

### 2. dod 关键产物 grep 规则

dod 里出现的**代码标识符**（snake_case / PascalCase / 文件路径片段）逐条在
`crates/*/src/**/*.rs` 与 `scripts/*.{py,ps1}` 下 grep。
**任一** dod 标识符 grep 不到 → (b) FAIL。

**豁免词**（不算标识符，跳过 grep）：
- `gate` / `PASS` / `FAIL` / `SKIP` / `verified` / `pending` / `blocked`（门禁词）
- `cargo` / `test` / `build` / `fmt` / `clippy`（命令词）
- `全绿` / `全通` / `通过` / `全量`（中文标记词）
- `S<阶段>.<子号>` 任务 ID（`S5.4` 等）

### 3. 已知豁免清单（`EXEMPT`）

早期任务（S0 系列 / S1 系列 / S2 系列）的 dod 不含可 grep 的代码标识符，
或 evidence 格式为纯日志（无测试标记），**直接豁免**（不校验 (a)/(b)）。
新增任务默认**不**豁免，dod 写清关键产物名。

当前豁免清单（脚本里 `EXEMPT_IDS`，随任务演进维护，完整列表见脚本源码）：
S0 系列 6 条 + S1 系列 6 条 + S2 系列 6 条，共 18 条（早期任务，dod 无代码标识符或 evidence 为纯日志）。

S1.7 起全部**走真校验**（S3/S4/S5/S7/S8/S9/S10 各阶段同），
dod 须含可 grep 的关键产物。若 dod 只写中文描述无标识符，(b) 自动 PASS（vacuously true），
由 doccheck 第 9 项（evidence 内容特征）兜底。

### 4. 孤儿 evidence 检测（S13.2 增）

`evidence/` 下形如 `<任务ID>[_后缀]` / `<任务ID>[-后缀]` 的证据文件，其任务 ID
必须在 `plan/ledger.yaml` 里存在，否则判 FAIL（退出码 1）。

判它是「台账被文件级回退丢掉条目」的信号：证据日志还在、任务行没了 →
`doccheck` 第 9 项会连带爆一串「references task which is not in plan/ledger.yaml」
（S11/S12 共 23 条丢失、doccheck 115 error 的事故，见 EXP-065）。

不参与本检查的文件：`gate*.log`、`font_render.txt` 等不以任务 ID 开头的名字
（用 `EVIDENCE_ID_RE` 只匹配行首 `S<数字>(.<小写>)*`），以及扩展名不在
`EVIDENCE_EXTS`（`.log/.txt/.md/.json/.png`）内的文件。

正例/负例证据见 `evidence/S13.2.log`：正常态 0 孤儿；临时构造一个台账里不存在的
任务 ID 证据文件后，脚本报 FAIL 且退出码为 1。

## 退出码

- `0`：全部 verified 任务 (a)(b) 通过，或全部命中豁免
- `1`：任一 (a) evidence 缺失/纯占位，或 (b) dod 关键产物 grep 失败
- `2`：台账解析失败（YAML 语法错误 / 文件缺失）

## 与 doccheck 的分工

- **本脚本**：台账 verified ↔ evidence 实存 ↔ 代码产物 grep（三方对账，EXP-058）
- **doccheck 第 9 项**：文档 ID 唯一性 + release 台账计数漂移（数值对账）
- **gate.ps1 第 8 项**（`ledger` bin）：台账语法 + 依赖 + 状态机合法性

三者互补，不重复。

## 维护

- 新增 S<阶段>.<子号> 任务，dod 写清 grep 标识符（如 `cargo test -p ti-app --test rules_trade`
  里的 `rules_trade`），本脚本自动校验
- dod 只写中文描述时 (b) 自动 PASS，由 doccheck 兜底
- 发现某任务 dod 太模糊（grep 不到标识符但 evidence 又纯日志）→ 补全 dod 标识符，
  别加豁免
