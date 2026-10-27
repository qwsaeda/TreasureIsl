# 经验库 · tools

> 本机工具链与 MCP：IDA/Python/REA/clippy 的用法与已知缺陷。条目 ID 全局唯一（EXP-NNN），维护规则与全量索引见 docs/experience/index.md。

### EXP-111 工具：PowerShell 5.1 `Get-Content -Raw` + `Set-Content -Encoding utf8` 改含中文的 `.rs` → GBK 误解码 + **吞掉换行**（160 处「文档注释与代码同行」）+ 加 BOM ⇒ 文件结构被静默改写
- 现象：为改一处 `&b` → `b`（clippy `needless_borrow`）用 `(Get-Content f -Raw) -replace … | Set-Content -NoNewline -Encoding utf8 f` 改 `crates/ti-app/src/game_app.rs`（含大量中文注释）。命令「成功」，但随后 `git diff` 显示 446 插入 / 767 删除、行数从 2148 掉到 1658；正则扫出 **160 行**形如 `/// 说明文字    pub player: Unit,`（文档注释与下一行代码被并成一行）；中文整体变 GBK mojibake；文件头多出 UTF-8 BOM。
- 根因：Windows PowerShell 5.1 的 `Get-Content` 默认按**系统 ANSI 代码页（本机 GBK）**解码无 BOM 的 UTF-8 文件 ⇒ 字节被重新解释成 GBK 字符；GBK 解码器把 UTF-8 尾字节 + `0x0A` 当作**双字节字符的后半**，于是**换行被吃掉**并留下 U+FFFD；`Set-Content -Encoding utf8` 再把这份 mojibake 写回（PS 5.1 该编码**带 BOM**）。比 EXP-051/083/086/091 更危险：那些是「中文变乱码」可肉眼发现，本例是**代码结构被改写**（行合并）⇒ 编译期才炸，且 `git diff` 噪声极大、极易误判为「自己刚改的」。
- 解法：① **禁止用任何 PowerShell cmdlet（`Set-Content`/`Out-File`/`Add-Content`/`Get-Content`+重定向）改仓库内含 CJK 的源码/文档**——改文件一律用编辑工具（按唯一子串精确替换），或用 `python` 以 `encoding='utf-8', newline='\n'` 读写；② 真要批量替换，先 `python -c` 脚本化（显式 `encoding='utf-8'` + `newline='\n'`），不要用 cmdlet 管道；③ 已被写坏时：`git checkout -- <file>` 回滚到上一个 commit，再用编辑工具重做（**先确认该文件的在途改动是否已 commit**：本例 S19.4 的改动未 commit ⇒ 回滚会一并丢失，须照原意重做）；④ 回滚后必跑 `cargo build` + `cargo test` 复核（本次回滚丢了 `event_chance_pct` helper，编译报 `E0425` 才暴露）。
- 验证：2026-10 S19.4 会话实测：写坏后 `git diff --stat` 1213 行变更 / 文件少 490 行、正则命中 160 处行合并；`git checkout` + 编辑工具重做后 `cargo build -p ti-app` 0 error、`cargo test -p ti-app` 39+6 passed / 0 failed、`cargo test -p ti-core --lib` 116 passed / 0 failed。
- 教训：**在 Windows 上「改文件」这件事本身就是高危操作**——凡是能让 shell 参与写文件（重定向 / cmdlet / 管道到文件）的路径，都要先问「这一步会不会经过 ANSI 代码页」。判定口诀：文件里有没有非 ASCII？只要有，shell 写它就要么显式指定 UTF-8，要么别用 shell 写。

### EXP-006 工具：本机 Python 环境缺包、`pip` 不在 PATH
- 现象：`import pefile/capstone` 失败，`pip list` 命令找不到。
- 根因：miniconda 的 `pip` 未挂进 PATH；精简环境无第三方包。
- 解法：优先手写解析（PE 头/导入表已手写成功）；确需装包先跑 `python -m pip --version` 确认可用，且装包动作要在 evidence 里记一笔。
- 验证：手写解析产出 F-101~F-103。


### EXP-010 工具：PowerShell 下 Python 报错噪声大
- 现象：`python -c` 报错时 PowerShell 追加大量红色 `CategoryInfo`，掩盖真实堆栈。
- 解法：脚本落盘用 `python -X utf8 file.py` 执行，或重定向 `2>$null` 后单独取 stderr；Windows 下统一 `-X utf8` 避免编码错。
- 验证：后续脚本执行输出可读。


### EXP-014 环境：IDA 7.5 portable 的 Hex-Rays ARM 反编译器不可用（**status: resolved by EXP-031**）

> **本条的阻塞已解除**：EXP-031 确认 7.5 portable 的授权链是根因，装 IDA 9.4 正式版后 Hex-Rays ARM 正常。
> 台账 S0.2 已是 `verified`。保留原文是因为「先怀疑授权链、而不是 DLL 版本匹配」这条排查顺序值得记住。
- 现象：`idat64.exe` 对 ARM32 目标（TreasureIsl.exe）`ida_hexrays.init_hexrays_plugin()` 返回 False；`load_plugin("hexarm")` / `load_plugin("hexarm64.dll")` 均返回 False（IDA 报 "Plugin not found"，即启动期未索引）。
- 对照实验：**同一 `idat64.exe` 对 x64 目标（notepad.exe）Hex-Rays 正常**（`hexrays_init=True`, version `7.5.0.201028`, decompile 成功）→ 许可证有效、核心可用，问题仅在 ARM 目标。
- 已排除：DLL 文件存在且 `PLUGIN` 导出正常；导入表 `hexarm64.dll → IDA64.DLL` 正确（EA64 版）；`ida.key` 含 `HEXARMW ARM Decompiler (Windows)` 许可项；处理器 `procname=ARM`（hexarm 应支持）；cwd 中文路径无关（ASCII cwd 同样失败）。
- 推测：portable 版的 `hexarm64.dll` 的 `init()` 静默返回 `PLUGIN_SKIP`，原因不明（可能是该文件与 `ida64.dll` 版本/签名不匹配，或 portable 破解导致 ARM 反编译器授权校验失败）。
- 影响：S0.2（伪代码导出）及其下游 S0.3/S0.4（分类/导入名）全部阻塞。
- 临时方案：S0.2 标 `blocked_by: ida-hexrays-arm`，IDA 链挂起；先推进不依赖 IDA 的任务（S2.1/S1.1/S1.3 等）。后续可选：(a) 换完整版 IDA、(b) 用 Ghidra（自带 ARM 反编译器）、(c) 降级为只导反汇编（需改 plan/parity）。
- 教训：环境依赖类任务应尽早冒烟（在 S0.1 就验证 IDA + hex-rays 能出伪代码），而不是等 S0.2 才发现。


### EXP-016 环境：IDA 7.5 portable headless 执行要点（**status: superseded by EXP-031**）

> **本条已被 EXP-031 作废**（9.4 正式版替换 7.5 portable）。保留原文是因为「破解版踩坑路径」本身是信息。
> **前两条（PYTHONHOME / python3.dll 复制）已作废**；后四条（UTF-8 无 BOM / `-L` 日志 / 进程管理 / `SCRIPT_TIMEOUT`）在 9.4 仍成立。
> 任何工具链决策以 **EXP-031** 为准，不要照本条行动。

- **适用范围（2026-10 更新）**：本条为 **7.5 portable 专属**，9.4 正式版已装回。前两条（python38 路径 / 复制 python3.dll）**已作废**；后四条（UTF-8 无 BOM / `-L` 日志 / 进程管理 / `SCRIPT_TIMEOUT`）**在 9.4 仍成立**——实测 `D:\soft\IDA Professional 9.4\cfg\idapython.cfg` 仍是 `SCRIPT_TIMEOUT = 3`。详见 EXP-031。
- **Python 路径（仅 7.5）**：`idat64.exe` 自带的 python38 在 `IDA\python38\` 子目录，但 `idat64.exe` 的 `_base_executable` = IDA 根 → `sys.prefix` 算成根 → `encodings` 模块找不到。修复：设 `$env:PYTHONHOME = "$p\python38"; $env:PYTHONPATH = "$p\python38\python38.zip"`。
- **插件 DLL（仅 7.5）**：`idapython3_64.dll` 的依赖 `python3.dll` 在 IDA 根不默认搜索 → 复制 `python38\python3.dll` 和 `python38\python38.dll` 到 IDA 根目录。
- **脚本编码**：Python 脚本必须是 **UTF-8 无 BOM**（BOM → `SyntaxError: invalid character in identifier`，`compile()` 报错）。opencode 的 Write 工具写出的文件无 BOM ✓；PowerShell `Set-Content -Encoding UTF8` **有 BOM** ✗。
- **输出去向**：IDA 把脚本编译/运行错误写到 `-L` 日志文件，**不写 stdout** → 表现为"静默 exit 1、零输出"。排障时必须读 `-L` 日志（GBK 编码，用 `Get-Content -Encoding UTF8` 或 `[Text.Encoding]::GetEncoding('GBK')` 解码）。
- **进程管理**：PowerShell `& idat64 ... | Select-Object -First N` 可能**不终止子进程**（管道关闭 ≠ 进程退出）→ 留下僵尸占 IDB/session 锁 → 后续实例秒退 exit 1。正确做法：`Start-Process -PassThru -NoNewWindow -RedirectStandardOutput ...` + `$pr.WaitForExit()`。杀僵尸用 WMI `Invoke-CimMethod Terminate`（`Stop-Process` 有时无效）。
- **`SCRIPT_TIMEOUT`**：`cfg\idapython.cfg` 默认 `SCRIPT_TIMEOUT = 3`（秒），长脚本会被杀 → 改为 `0`（禁超时）。


### EXP-031 环境：IDA Professional 9.4 正式版 Hex-Rays ARM 正常 → EXP-014 阻塞解除（作废 EXP-016）
- 起因：7.5 **portable 破解版**被删除，改装 `D:\soft\IDA Professional 9.4\` 正式版后重新验证 EXP-014 的结论。
- 证据：`ref/ida_run.log` 首段 `Loading processor module D:\soft\IDA Professional 9.4\procs\arm.dll for ARM...OK`；`evidence/S0.2.log` 记 `functions=1827 files=1827 csv_rows=1827 hexrays_ok=1823 hexrays_failed=4`（1823 个函数产出真实 Hex-Rays ARM 伪代码，非反汇编回退）。
- 抽查复核：`ref/pseudocode/00011000_sub_11000.c` 是真伪代码（`_DWORD *__fastcall sub_11000(...)`，含 `&off_6B000` 全局初始化），与 9.4 + idalib 无头重跑结果逐字一致。
- 结论：**不需要 Ghidra**。EXP-014 是 portable 破解版的问题，正式版不存在；台账 S0.2 已是 `verified` / `blocked_by: null`。
- 连带作废：EXP-016 记的 `PYTHONHOME=...\python38` / `python38.zip` 补丁、`idat64.exe` 进程管理等**只适用于 7.5 portable**，9.4 无需；headless 文本模式统一为 `idat.exe`（9.x 已把 `idat64.exe` 合并进去，32/64 同一程序，`-A`/`-S`/`-L` 开关不变）。
- 联动修正：`AGENTS.md`、`legacy/docs/facts/index.md`、`docs/plan-s0-s2.md`、`legacy/scripts/ida_dump.py` 中的 `IDA_Pro_v7.5_Portable\idat64.exe` 全部改为 `D:\soft\IDA Professional 9.4\idat.exe`。
- 附：REA 逆向 MCP 走同一条 IDA 路径（`idapro 0.0.9` + `ida-pro-mcp 2.0.0` 的 `idalib-mcp` headless supervisor，注册见 `~/.rea/ida-mcp.json`）。注意 `idapro` 必须装 IDA 自带版本（9.4 = 0.0.9），PyPI 上的 0.0.11 对应更新版 IDA。
- 教训：**环境类阻塞先分清「portable/破解版」与「正式版」**。EXP-014 花了 4 条排除项才定位到破解版授权校验失败，而正确答案只是换正式版；下次遇到「某功能在破解版上不可用」应优先怀疑授权链，而不是继续挖 DLL 签名/版本匹配。


### EXP-032 工具：通用解析器别夹带特定布局校验，否则误伤同家族其他成员
- 现象：`MapScene::parse` 被 `terrain.rs` 与 `build_assets.rs` 共用（UI 场景与地形图同族）。给它加「记录级整除校验（`5+6*count`，F-306）」后，`terrain_records` 13 项与 `build_assets` 3 项测试全挂——地形图字流 `words[4]=0` 但后跟 4278 字，按 UI 布局必报 `TrailingBytes`。
- 根因：**记录级语义属于 UI 族，不是字流通用语义**。校验位置放错层级。
- 修法：`MapScene::parse` 回退为纯「头 + u32 字流」透传；`MapControl::parse_words` 独立承担 UI 记录校验，`terrain::TerrainMap` 走自己的 stride=9 布局。消费方各取所需：`controls` bin 对 `MapControl::parse_words` 失败的文件**记为 skipped**（地形），不再整体报错。
- 教训：**同一家族多个子布局共用通用解析器时，子布局校验放「视图方法」而非「解析入口」**；通用层只保证「消费完每个字节」，不保证「语义结构」。改解析器前先 grep 它的全部调用方。


### EXP-033 工具：REA 6.0.0 ↔ OpenCode 互操作缺陷（headless 已验证可用，但有两坑）
- 实测配置：`rea-agents 6.0.0` 全局 + `idapro 0.0.9`（IDA 9.4 自带 wheel，**不是 PyPI 的 0.0.11**）+ `ida-pro-mcp 2.0.0`（GitHub main，PyPI 只有 1.4.0 无 supervisor API）的 `idalib-mcp --stdio` headless。注册 `~/.rea/ida-mcp.json`，OpenCode `mcp.rea.environment` 设 `REA_ANALYSIS_PROVIDER=ida` + `REA_IDA_MCP_CONFIG`。`rea doctor --provider ida` healthy。
- 坑 1：REA 里**不产出 Evidence 的工具**（`read_bytes`、无会话时的 `analyze_function` 等）声明的 outputSchema 要求 `result/evidence_id/evidence` 且 `additionalProperties:false`，实际不返回这些键 → OpenCode 报 `-32602`。替代：读内存数据用本地脚本（`scripts/pe_rva.py`），出证据走 `analyze_function`/`search_strings` 等带 Evidence 的工具。
- 坑 2：modern supervisor profile 下 `callers` 恒空（REA 自注 "Direct callers are unavailable in this profile"）。找调用方靠本地 1827 份伪代码 grep，REA 负责单函数权威证据（伪代码 + 汇编 + 证据包）。
- 性能事实：`open_binary` 2.6s 返回，首次分析请求 ~19s（IDA 自动分析在首个工具调用时执行）；supervisor worker idle TTL 1h 会驻留一个 python 进程（~70MB），是上游设计。
- 教训：接新 MCP 先 `open_binary` + 一个小 `analyze_function` 冒烟，再决定哪些工具可信；**声明的 schema 不等于实际输出**，撞 `-32602` 就降级到本地脚本。


### EXP-036 工具：REA 6.0.0 + idalib-mcp supervisor profile 下「几乎所有工具」都撞 -32602（F-314 坑 1 范围被低估；**结论修订：波动性可用，见 EXP-039/F-319**）
- 现象：本会话对 TreasureIsl.exe 跑 S3.12 RNG 实操，依次调用 `search_procedures`/`analyze_function`/`trace_native_values`/`procedure_pseudo_code`/`procedure_assembly`/`search_strings`/`list_procedures`/`binary_overview`/`address_name`/`list_strings` 共 10 个工具，**全部**返回 `MCP error -32602: Structured content does not match the tool's output schema: data must have required property 'result'/'evidence_id'/'evidence'`。仅 `open_binary`（绑会话）返回正常 JSON。
- 根因：F-314 原记「不产出 Evidence 的工具（`read_bytes` 等）」撞 -32602，措辞暗示「只有少数不产 Evidence 的工具」会撞。**实测推翻**：在 `idapro 0.0.9 + ida-pro-mcp 2.0.0` 的 `idalib-mcp --stdio` modern supervisor profile 下，**除 `open_binary` 外的全部被测工具都不返回 outputSchema 要求的 `result/evidence_id/evidence` 三键** → OpenCode 校验一律 `-32602`。F-313 记「`analyze_function`/`list_procedures` 可用」是上次会话记录，可能当时 supervisor profile 不同或没做输出 schema 校验，**不可复现**。
- 解法（**2026-10 修订**）：**波动性可用，非永久不可用**。S3.12 会话 10/10 失败是实测事实，但同日 S3.2 会话同类工具可用（EXP-039 / `evidence/S3.2.log` / S3.2 已 verified），根因是 supervisor profile / 会话状态波动。**用法：每次用前冒烟 `open_binary` + 一个轻量调用（如 `search_strings`），通过才继续，失败即退主路径，不浪费回合数**。主路径不变：① `ref/pseudocode/*.c` grep（S0.2 已全量导出 1827 份）；② `scripts/pe_rva.py`；③ `idat.exe -A -S`。REA 结构化能力（CFG/调用图/类型/值追踪/重建验证）见 F-321 + `docs/style.md` §5.11.4。
- 验证：本会话 10 次工具调用日志（见本条「现象」逐条列举）；`open_binary` 返回 `{"result":{"path":"...","sha256":"2a80f209…","architecture":"arm"}}` 正常。
- 教训：① **接新 MCP 不能只测一两个工具就写「可用」结论**（F-313 教训保留）；② **也不能因一次会话失败就写「停用」而忽略其他会话的成功记录**——EXP-036 原结论「REA 工具调用先停用」被 EXP-039 + S3.2 verified 任务推翻。必须**跨会话交叉验证**后才下可用性结论；③ 文档自相矛盾时（`style.md` §5.11.1 要求用 REA，F-319/本条说停用）必须读全所有相关条目再下判断，不能只读路由表摘要。


### EXP-042 工具：S3.12 会话 REA 冒烟再次失败（2026-10-09，F-319 波动性第三例）
- 现象：S3.12 RNG 开工前按 §5.11.4 冒烟：`open_binary` 正常（返回 ARM PE，sha256 `2a80f209…` 与 F-313 一致），但下一步轻量 `search_strings`（literal 搜 "srand"）立即返回 `MCP error -32602: Structured content does not match the tool's output schema`。同会话未再试其他 REA 工具。
- 根因：与 EXP-036 完全一致——`idapro 0.0.9 + ida-pro-mcp 2.0.0` 的 `idalib-mcp --stdio` modern supervisor profile 下，除 `open_binary` 外的工具不返回 outputSchema 要求的 `result/evidence_id/evidence` 三键。这是**同一天内第三次**会话级失败（S3.12 会话 10/10、EXP-036 记录、本次），而 S3.2 会话 10 个工具可用（EXP-039 / `evidence/S3.2.log`）仍成立。根因是 supervisor 会话状态波动，非工具永久不可用。
- 解法：按 F-319 / §5.11.4 纪律——**失败即退主路径，不浪费回合数**。S3.12 推进全程走 ① `ref/pseudocode/` grep（1827 份，播种点 `0003D4C8`/`0004E314`/`0004E568` 已在 spec §b 标好行号）+ ② `scripts/pe_rva.py` + ③ `idat.exe -A -S`。RNG LCG 常数是推测值（F-320），靠 `tests/diff_rng.rs` 差分反证，不依赖 REA。
- 验证：本次冒烟 2 个工具调用日志（`open_binary` 成功 / `search_strings` -32602）；后续 S3.12 任务 evidence 中引用本条，记录「REA 冒烟失败，走主路径」。
- 教训：① REA 波动性可用是**会话级**事实，每次会话必须重新冒烟，不许把上次会话的可用/失败结论外推；② 冒烟失败时立刻切主路径，不要在同一会话反复试 REA 不同工具（EXP-036 的 10 连失败就是浪费）；③ 本条与 F-319 形成第三组交叉证据，若下次会话 REA 又全失败，应考虑在 AGENTS.md 把 REA 定位进一步降级为「仅 open_binary 绑定 + 离线时可选」。

### EXP-041 工具：REA 定位维度错误 + 文档矛盾未读全（三层定位修正，见 F-321）

- 现象：两轮会话对 REA 在项目中的定位前后矛盾——先判断为「次级交叉校验」（低估），用户反问后修正为「补结构化深度分析的洞」。根因分析发现文档内部自相矛盾：`style.md` §5.11.1 要求 S3 模块用 REA `search_strings`，但 `F-319`/`EXP-036`/`AGENTS.md` 说「按不可用对待 / 先停用」，而 `EXP-039` + S3.2 verified 任务明确反驳（search_strings 实测可用）。
- 根因：① 把 REA 与本地 grep 放在同一维度比较（批量离线 vs 结构化深度是不同维度），导致判断为「次级」；② 只读 `AGENTS.md`「按不可用对待」摘要就下判断，未读 `EXP-039` / `style.md` §5.11.1 的反证；③ 文档自相矛盾未被及时发现。
- 解法：分层定位（三层）：① **批量/离线主路径**（grep 1827 份 + pe_rva.py + idat -S，不可替代，离线可复现）；② **REA 结构化能力**（CFG/调用图/类型/值追踪/重建验证，补洞，冒烟优先，失败即退，见 F-321 + `style.md` §5.11.4）；③ **S4 重建验证探针**（`verify_reconstruction`，S4.1 花一回合验可行性）。
- 验证：F-321 REA 结构化能力 vs 已有资产对比表（逐项标「未利用」）；F-319 波动性可用修订；EXP-036 结论改写。
- 教训：**路由表摘要是入口不是全貌**——涉及工具可用性/定位判断时，必须读全 experience + facts 原文（`EXP-033/036/039` + `F-313/314/319/321`），不能只看 `AGENTS.md` 一句摘要。文档自相矛盾时，先读全再下判断，发现矛盾立即修订（不留「停用」与「要求用」并存的状态）。


### EXP-050 工具：CJK 路径下 cargo `env!("CARGO_MANIFEST_DIR")` 损坏
- 现象：`crates/ti-tools` 的 bin 在 `C:\金银岛`（CJK 字符路径）下用 `env!("CARGO_MANIFEST_DIR")` 拼 repo 根，`cargo run` 后进程里解析出的路径丢 CJK 字符 → `fs::read` 报 "系统找不到指定的文件"（os error 2），但 `Test-Path` / Python 都能正常访问同一路径。
- 根因：`env!` 宏在编译期把 `CARGO_MANIFEST_DIR` 字符串嵌进二进制，经 shell/PowerShell 传递 CJK 路径时编码损坏（Windows 控制台 GBK vs Rust `String` UTF-8 不一致），导致路径字符串失真。
- 解法：bin 的 `repo_root()` 改为**优先 `std::env::current_dir()`**（`cargo run` 与 shell 同 cwd，cwd 是运行时获取、不经过编译期宏），再回退 `env!`；用 `data/` 目录存在性判定哪个是真正的 repo 根。`build_hd_assets` 已按此修（`crates/ti-tools/src/bin/build_hd_assets.rs`）。
- 验证：`cargo run -q -p ti-tools --bin build_hd_assets -- --check` 在 `C:\金银岛` 下全通过（1340 条 0 失配）；evidence/S7.4.log 记录。
- 教训：CJK 路径项目里，**凡 bin 需要定位 repo 根，优先 `current_dir()` 运行时获取，不要依赖 `env!("CARGO_MANIFEST_DIR")` 编译期字符串**。其他 bin（`build_assets`/`unpak` 等）若遇同样问题按此法修。




### EXP-056 资源：多帧精灵 HD PNG 以分帧命名（NN_hd_fK.png），Rust 侧须按帧列表注入
- 现象：assets_hd_pack/1x/ui/ 的多帧精灵（f1=14/31/32/37）不以全图 NN_hd.png 命名，而是分帧 NN_hd_f0.png ~ NN_hd_f4.png（pack_hd_assets.py 产物）。若 blit_sprite_rgba 按单张全图 + frame_offset 切帧的方式读 PNG，PNG 宽 = PCX 宽但帧高度只占全图 1/5，行 stride 匹配但帧窗口不对（PNG 是单帧 35×40，PCX 全图 35×199）。
- 根因：pixel_upscale.py 对多帧精灵按帧切开后逐帧 PNG 化，pack_hd_assets.py 原样进 4 档。Rust 侧 hd_cache 原设计为 HashMap<u32, HdPng>（单张），多帧只能注入 1 帧。
- 解法：hd_cache 改 HashMap<u32, Vec<HdPng>>（f1 → 帧列表）；load_hd_pngs 签名改 (u32, Vec<Vec<u8>>)；blit_sprite_rgba 按 frame % frames 取对应 PNG，缺帧回退 PCX。
- 验证：render_parity_hd 判据 7（f1=14 注入 5 帧，frame 0 vs 1 blit 画面不同）全绿；1x 档 335 文件 1,519,349 B（1.45 MB）预算 1.6 MB PASS。


### EXP-091 工具：文档文件含非法 UTF-8 坏字节时 doccheck 的 `read_to_string` 静默跳过整文件 → 该文件所有检查行永久失声（**status: resolved**）
- 现象：`docs/release.md` 台账计数行长期漂移（文件里是旧值，ledger 实测已 113 任务/110 verified），但 doccheck（gate 第 9 项）一直报 0 error——计数断言（`check_ledger_counts`）形同虚设。
- 根因：release.md 含 ~292 处历史坏字节（UTF-8 三字节字符第 3 字节被替成 `0x3F '?'`，如 `EF BC 3F` 非法序列；另混入 GBK 双字节段）。`ti_tools::doccheck::push_doc` 用 `std::fs::read_to_string`（严格 UTF-8）+ `let Ok(text)=… else { return }` → 整文件被**静默跳过且无告警**，其中「N 任务 M verified」行因此从未被校验。AGENTS.md 写「release.md ⚠️ GBK 编码」亦与实测不符（实测为 UTF-8 主体 + 少量坏字节 + GBK 段混杂，2026-10 字节级扫描确认）。
- 解法：① 编辑此类文件先跑坏字节扫描（UTF-8 严格解码 + 坏序列定位，见 evidence 附脚本），确认该文件是否「对 doccheck 不可见」；② 修正只动 ASCII 区（如数字），用**字节级替换**（`ReadAllBytes`/`WriteAllBytes` 定点改，或编辑工具匹配纯 ASCII 子串），禁止整文件 Set-Content/重编码（EXP-051/083/086 三次复发教训）；③ 坏字节全量修复须另立任务（含「doccheck 增加文档可解码性检查：注册文档读不到/含非法 UTF-8 → FAIL 而非静默跳过」），登记台账后才做，不在顺手中做。
- 验证：2026-10 S16 登记会话实测：release.md 坏字节 292 处、`read_to_string` 失败可复现；计数行字节级修为「117 任务：110 verified」后 doccheck 仍 0 error（文件仍被跳过，行为不变=无回归）；坏字节全部修好后该行将首次真正进入 `check_ledger_counts` 校验（117/110 与 ledger 一致）。
- 教训：**门禁里「读不到文件」必须显式失败，不许静默 skip**——静默 skip 让 release.md 的计数漂移躲过检查多轮；同类模式（`read_to_string` + `else return`）在 doccheck2/check_ledger_reality 里也要排查一遍。
- 解决记录（2026-10）：子项 ③(a)「release.md 坏字节全量修复」已完成——未走字节逐替，而是从 git 历史取最后干净版（commit `24c5716`，UTF-8 严格解码 0 坏字节）为基底，重排瘦身成纯 UTF-8（14135→12581 字节，无 BOM），保留工作区唯一在途 WIP 行（ledger 117/110）。修复后 doccheck 重新纳管该文件（docs 计数 114→115），0 error。子项 ③(b)「doccheck 增加文档可解码性检查（读不到/含非法 UTF-8 → FAIL 而非静默跳过）」**仍未做**，如需落地另立台账任务。

### EXP-123 工具：工具声称与某判据「同口径」必须逐字复刻其计数逻辑（flywheel 顶格 Report 数 vs doccheck2 全 Report 去重漂移）
- **现象**：`scripts/flywheel_readme.ps1 -Check` FAIL「gate 项仅 12（要求 ≥21）」——但其注释自称「与 doccheck2 gate_item_actual_count 同口径」。实际 doccheck2 数**全部** `Report ` 调用（trim 开头、按项名去重 = 28），flywheel 旧实现只数**顶格** `^Report\b`（=12）⇒ 口径漂移误报。
- **根因**：gate.ps1 早期全顶格 Report，flywheel 解析器按「顶格 = 固定项」实现；S24.1 门禁分档后 Report 调用大量移入 tier 条件块（缩进），顶格数静默跌到 12；`≥21` 下限挡到最后才暴露。声称「同口径」但实现不同口径 = 工具契约漂移。
- **解法**：flywheel 的 `Read-GateItems` 改成与 doccheck2 `gate_item_actual_count` 完全一致（`trim StartsWith 'Report '` + 行内首个引号串去重）⇒ 28 ≥ 21 PASS。规范 = **任何工具声称与某判据「同口径」时，必须逐字复刻其计数/提取逻辑，并在 -Check 下限里引用同一期望值**；改 gate 结构后必须同时跑所有「同口径」消费方。
- **验证**：`powershell -File scripts/flywheel_readme.ps1 -Check` PASS（gate 28 项 = doccheck2 `check_gate_item_count` 期望 28）；`cargo run -p ti-tools --bin doccheck2` 0 error。
- **同源复发（2026-10-27，S27/S28 批次）**：`scripts/doc_numbers.py` 的 `gate_item_count()` 复刻了**旧 flywheel 口径**（只数顶格 `Report ` 行 = 12）⇒ 跑 `--fix` 会把全库声称的 **28 项回写成 12**（parity.md / git.md / README.md / gate.ps1 / playbook.ps1 / style.md / loop.md 同时被污染）⇒ doccheck2 立刻报 5 处「门禁项数声称漂移（声称 12，实测 28）」。已改：①`gate_item_count()` 改为与 doccheck2 逐字对齐（任意缩进 + 行内首个引号串**按名去重** = 28）②`_is_gate_total()` 增负例护栏：排除 `doccheck2 34→12 项` / `check_code_standards.py 12 项` 等**非门禁项数**语境（否则 `--fix` 会把这些也改成 28）。规范 = **修一个「同口径」工具后必须清点全部同口径消费方**（本次：flywheel ✅ → doc_numbers ❌ → 已修），且**回写类工具必须带负例护栏**，否则一次 `--fix` 能同时污染 7 个文件。
