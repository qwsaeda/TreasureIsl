# 经验库（Experience Log）

**定位**：开发过程中踩过的坑与验证过的解法，唯一目的是让后续会话**不重复踩坑**。

## 维护规则（强制）

1. **触发即写**：同一问题第二次遇到还没写 = 违规；首次踩到新坑（工具、格式、平台、构建）必须当场新增条目。
2. **一条一坑**：`EXP-NNN` 递增编号，写清「现象 → 根因 → 解法 → 验证方式」，不写空话。
3. **被推翻要改**：后续证据推翻某条结论 → 更新该条并标 `已修正`，不删（保留错误路径本身就是信息）。
4. **关联**：任务 evidence 中引用条目 ID（如 `见 EXP-003`），台账 evidence 与本文件互相可追。
5. 与 `docs/reverse-facts.md` 的分工：**事实库放"结论"，本文件放"过程与教训"**；结论必须先进事实库（带证据），再在本文件记怎么拿到的。

## 条目格式

```
### EXP-NNN 标签（工具/格式/构建/流程）
- 现象：
- 根因：
- 解法：
- 验证：
```

---

## 条目

### EXP-001 格式：exe 里的资源名是 UTF-16 宽字符
- 现象：用 ASCII 正则扫 `TreasureIsl.exe` 找 `*.pcx` 得 0 结果，误以为没有明文资源名表。
- 根因：WinCE 程序用 `W` 系 API，文件名字符串全部 UTF-16-LE 存储。
- 解法：正则 `(?:[\x20-\x7e]\x00){4,}` + `decode('utf-16-le')`，得到 119 个资源名（含 `button_*.pcx`、`cards\cards.ini`）。
- 验证：与 `data/` 实际文件名交叉吻合。

### EXP-002 格式：pak 的 key 不是常见哈希，别再穷举
- 现象：`ui.pak` 索引 key（`0x2f5cfa6a...`）拿 119 个名字试 crc32 / crc32(utf-16) / fnv1a / adler32 全 miss。
- 根因：key 是 exe 内自定义函数算出来的（高字节恒 `0x2F` 且单调，形态像带版本位的自定义散列）。
- 解法：**停止猜哈希**，去反编译 exe 中「打开 pak → 查找条目」的函数（S1.1 的正解）。
- 验证：S1.1 验收 = 119 名字命中率 100%。

### EXP-003 格式：LucencyPackage 头部字段不是 4 字节对齐
- 现象：按 4B 对齐读 `ui.pak` 头拿索引流长度得到 `0x05BE0000`，zlib 解压报 `incorrect header check`。
- 根因：头部混有 u16 字段，关键值在 `0x26`(条目数=122) / `0x2A`(索引流长度=1470) / `0x32`(数据区起点=1524)，均为偶数但非 4 的倍数。
- 解法：先 hexdump 全头，逐偏移标注；结构确认前不写解析器。
- 验证：用 `0x2A` 处长度切片解压成功得 1952B = 122×16。

### EXP-004 格式：扫 `\x78\xda` 找 zlib 流会误报
- 现象：`cards.lpk` 在偏移 27358 处的 "流" 解压报 `Error -3`。
- 根因：压缩数据里可能天然出现 `78 DA` 字节序列；只有索引指向的偏移才是真流。
- 解法：永远按索引记录遍历流，不做全文魔数扫描。
- 验证：`ui.pak` 123 流按索引遍历全部解压成功。

### EXP-005 流程：`.pdata` 别按 x86 布局硬解
- 现象：按 12 字节 `RUNTIME_FUNCTION` 解析 `.pdata` 得到 size 为负数/上 GB 的荒谬结果。
- 根因：WinCE ARM 的 pdata 是另一种打包布局（疑似 8 字节 `{start, packed}`），我没验证就下结论。
- 解法：函数边界/数量一律以 IDA `.i64` 导出为准（S0.2），不自造解析器。
- 验证：S0.2 的 `functions.csv` 与 IDA 函数表一致。

### EXP-006 工具：本机 Python 环境缺包、`pip` 不在 PATH
- 现象：`import pefile/capstone` 失败，`pip list` 命令找不到。
- 根因：miniconda 的 `pip` 未挂进 PATH；精简环境无第三方包。
- 解法：优先手写解析（PE 头/导入表已手写成功）；确需装包先跑 `python -m pip --version` 确认可用，且装包动作要在 evidence 里记一笔。
- 验证：手写解析产出 F-101~F-103。

### EXP-007 流程：低价值目标不要反复投入（`.rsrc` 解析两次失败）
- 现象：解析 `.rsrc` 目录两次出错（基址换算、根目录 numId 偏移读错），第三次才意识到它只含图标/版本信息。
- 根因：没先评估信息价值就动手。
- 解法：先问「拿到它能推进哪个验收」，不能则记为缺口、跳过。
- 验证：跳过后无任何任务受阻（无任务依赖 `.rsrc`）。

### EXP-008 构建：本机无 Docker → 放弃 macroquad docker APK 路线
- 现象：`docker` 命令不存在；macroquad 官方 Android 构建推荐 `notfl3/cargo-apk` 镜像。
- 根因：环境未装 Docker，Android SDK/NDK 也未配置（无 `ANDROID_*` 环境变量）。
- 解法：S2.6 改为本机 `cargo-apk`/`cargo-ndk`（Java 17 已具备，需补装 SDK/NDK）；`game/tools` 代码不受影响，因为构建方式隔离在 S2。
- 验证：S2.7 smoke 三端全绿后本条标 `已解决`。

### EXP-009 流程：二进制结构结论必须先 hexdump 再下笔
- 现象：EXP-003、EXP-005、EXP-007 都是「按直觉写解析 → 失败 → 回头看字节」。
- 根因：跳过了「先观察、后建模」。
- 解法：任何格式先出一张偏移标注表（docs/formats/ 里放 hexdump 片段），确认后再写 `define_format!`。
- 验证：S1 各格式一次性通过 roundtrip（作为阶段统计指标，若反复返工则本条需补充）。

### EXP-010 工具：PowerShell 下 Python 报错噪声大
- 现象：`python -c` 报错时 PowerShell 追加大量红色 `CategoryInfo`，掩盖真实堆栈。
- 解法：脚本落盘用 `python -X utf8 file.py` 执行，或重定向 `2>$null` 后单独取 stderr；Windows 下统一 `-X utf8` 避免编码错。
- 验证：后续脚本执行输出可读。

### EXP-011 构建：本机无 `rg`、`bash` 是 WSL 版，门禁必须用 PowerShell
- 现象：按计划写 `gate.sh`（含 `rg` 扫描），探测发现 `rg` 不存在、`bash.exe` 是 `system32` 的 WSL 入口而非 Git Bash。
- 根因：计划里的门禁形态默认了 Linux 工具链，没先探测本机。
- 解法：门禁改 `scripts/gate.ps1`；占位扫描用 `Select-String`；不依赖 bash。已同步改 `AGENTS.md`/`parity`/`loop`/`style`/`plan-*`/`ledger` 中全部 `gate.sh` 引用。
- 验证：`git grep gate.sh` 0 命中；gate.ps1 实际跑通。

### EXP-012 流程：文档与环境冲突时，先改文档再开工
- 现象：规范写 `gate.sh`+`rg`，环境不支持；若直接按文档硬做会产出跑不起来的脚本。
- 解法：把「文档 → 环境」的适配作为迭代 0 的独立动作，连同笔误（六 crate→5）一起修，并在计划文件写修订记录，避免规范与现实长期背离。
- 验证：修订记录与提交历史对应。

### EXP-013 流程：编译工具链后置（用户决策）
- 现象：原计划把 `rustup target add`（wasm/android，数百 MB 下载）放在 S0.1，阻塞真正的代码开发。
- 解法：**先代码、后工具链** —— S0.1 只建 workspace + native 构建；wasm/android target 分别移入 S2.5/S2.6，gate 第 4 项按阶段解锁（脚本里用 SKIP 标注未解锁项）。
- 验证：S0.1 全程不装 target 即可 gate 全绿。

### EXP-014 环境：IDA 7.5 portable 的 Hex-Rays ARM 反编译器不可用
- 现象：`idat64.exe` 对 ARM32 目标（TreasureIsl.exe）`ida_hexrays.init_hexrays_plugin()` 返回 False；`load_plugin("hexarm")` / `load_plugin("hexarm64.dll")` 均返回 False（IDA 报 "Plugin not found"，即启动期未索引）。
- 对照实验：**同一 `idat64.exe` 对 x64 目标（notepad.exe）Hex-Rays 正常**（`hexrays_init=True`, version `7.5.0.201028`, decompile 成功）→ 许可证有效、核心可用，问题仅在 ARM 目标。
- 已排除：DLL 文件存在且 `PLUGIN` 导出正常；导入表 `hexarm64.dll → IDA64.DLL` 正确（EA64 版）；`ida.key` 含 `HEXARMW ARM Decompiler (Windows)` 许可项；处理器 `procname=ARM`（hexarm 应支持）；cwd 中文路径无关（ASCII cwd 同样失败）。
- 推测：portable 版的 `hexarm64.dll` 的 `init()` 静默返回 `PLUGIN_SKIP`，原因不明（可能是该文件与 `ida64.dll` 版本/签名不匹配，或 portable 破解导致 ARM 反编译器授权校验失败）。
- 影响：S0.2（伪代码导出）及其下游 S0.3/S0.4（分类/导入名）全部阻塞。
- 临时方案：S0.2 标 `blocked_by: ida-hexrays-arm`，IDA 链挂起；先推进不依赖 IDA 的任务（S2.1/S1.1/S1.3 等）。后续可选：(a) 换完整版 IDA、(b) 用 Ghidra（自带 ARM 反编译器）、(c) 降级为只导反汇编（需改 plan/parity）。
- 教训：环境依赖类任务应尽早冒烟（在 S0.1 就验证 IDA + hex-rays 能出伪代码），而不是等 S0.2 才发现。

### EXP-015 代码：clippy pedantic 反复触发的模式（workspace `-D warnings`）
- **`assert!(false, ...)`** → 触发 `assertions_on_constants`（`all` 组）。改用 `assert!(matches!(ev, Pattern { .. }))` + `if let Pattern { .. } = ev { ... }`，不留不可达分支。
- **`[T; N]` 栈数组 > 16384 字节** → `large_stack_arrays`（`all`/perf 组）。改用 `Box<[T]>`：`vec![0u16; N].into_boxed_slice()`。
- **`u32 as f32` / `i32 as f32`** → `cast_precision_loss`（`pedantic` 组）。改法：值已知且小时用 f32 字面量（`const LW: f32 = 176.0;`）；否则在函数上加 `#[allow(clippy::cast_precision_loss)]`。
- **`fn new() -> Self` 无 `Default`** → `new_without_default`（`pedantic`）。手动 `impl Default`（数组大时不能 derive，但手动 `Self::new()` 即可）。
- **`let start = off;` 宏展开后可能未用** → `unused_variables`。在 `let` 前加 `#[allow(unused_variables)]`。
- **`use super::*`** → `wildcard_imports`（`pedantic`）在本配置**未触发**（现有代码已有 glob import 且全绿），可继续用。
- 规约已写入 `docs/style.md` §5.10。

### EXP-016 环境：IDA 7.5 portable headless 执行要点（S0.2 重试参考）
- **Python 路径**：`idat64.exe` 自带的 python38 在 `IDA\python38\` 子目录，但 `idat64.exe` 的 `_base_executable` = IDA 根 → `sys.prefix` 算成根 → `encodings` 模块找不到。修复：设 `$env:PYTHONHOME = "$p\python38"; $env:PYTHONPATH = "$p\python38\python38.zip"`。
- **插件 DLL**：`idapython3_64.dll` 的依赖 `python3.dll` 在 IDA 根不默认搜索 → 复制 `python38\python3.dll` 和 `python38\python38.dll` 到 IDA 根目录。
- **脚本编码**：Python 脚本必须是 **UTF-8 无 BOM**（BOM → `SyntaxError: invalid character in identifier`，`compile()` 报错）。opencode 的 Write 工具写出的文件无 BOM ✓；PowerShell `Set-Content -Encoding UTF8` **有 BOM** ✗。
- **输出去向**：IDA 把脚本编译/运行错误写到 `-L` 日志文件，**不写 stdout** → 表现为"静默 exit 1、零输出"。排障时必须读 `-L` 日志（GBK 编码，用 `Get-Content -Encoding UTF8` 或 `[Text.Encoding]::GetEncoding('GBK')` 解码）。
- **进程管理**：PowerShell `& idat64 ... | Select-Object -First N` 可能**不终止子进程**（管道关闭 ≠ 进程退出）→ 留下僵尸占 IDB/session 锁 → 后续实例秒退 exit 1。正确做法：`Start-Process -PassThru -NoNewWindow -RedirectStandardOutput ...` + `$pr.WaitForExit()`。杀僵尸用 WMI `Invoke-CimMethod Terminate`（`Stop-Process` 有时无效）。
- **`SCRIPT_TIMEOUT`**：`cfg\idapython.cfg` 默认 `SCRIPT_TIMEOUT = 3`（秒），长脚本会被杀 → 改为 `0`（禁超时）。

### EXP-017 格式：`LucencyPackage` 三处事实推翻 + key 是"两类"（S1.1 实测）
- **推翻 1（F-201）**：索引流长度在 `@0x2E`，不是 `@0x2A`（`@0x2A` 读出 `01 00 00 00`）。教训：非对齐头部字段**必须 hexdump 逐字节复核**，不能沿用上一版样本的偏移。
- **推翻 2（F-202）**：索引**不总是 zlib**。`.pak` 压缩、`.lpk` 原始；**与版本号无关**（`glpics.pak` 版本 2005 但索引仍压缩）。正确做法 = **按 zlib 魔数（CM=8 且 FCHECK 正确）自动判别**，别信文件名/版本号。
- **推翻 3（F-204）**：`key` 就是 **`djb2(name.to_ascii_lowercase())`，种子 0**。旧结论"不是任何常见哈希 / 高字节恒为 0x2F"来自**只看 3 个样本 + 只试 4 种算法**（crc32/fnv1a/adler32），既漏了 djb2 的 0 种子变体，又把"索引按 key 升序排列"误读成"高字节恒定"。→ **教训：定哈希前先扫 ≥10 个样本、试 ≥15 个算法变体（含种子 0/5381、大小写、路径前缀），并检查是否 key 本身有序。**
- **实测更正**：容器条目数实测为 `ui.pak` 122 / `glpics.pak` 88 / `glmap_m01.pak` 47 / `glmap_m02.pak` 37 / `cards.lpk` 4 / `music.lpk` 6 / `goods.lpk` 1（原 F-207 记的 90/48/~5 有误，原因是按 `dstart` 切片且**没校验解压长度 == 条目数×16**，把错位数据当成了合法索引 —— Python 假阳性）。→ **教训：解析必须校验 `len(index) == count × REC`，否则错误会静默传播。**
- **key 分两类（重要，影响后续 DoD）**：名称哈希型（`glpics.pak` 85/88 可由 exe 资源名解释）+ **ID 型**（`ui.pak`/`glmap_*.pak`/`*.lpk` 共 217 条无法由 exe 字符串推出；`ui.pak` 前 90 个 key 构成公差恒为 `0x121101` 的等差数列）。推测 ID 型 key 由 `.Map` 场景按索引绑定（F-302 已预告），待 S1.4 逆出 → 已立 S1.11。
- **DoD 修订**：P1-2 原写"119 个 exe 资源名命中率 100%"**不可达**（其中 `.map`/`.sav`/`settings.dat`/`GoldLand.Fnt` 是磁盘独立文件，根本没有 key）。改为「**凡是能用资源名解释的条目必须 100% 无误，且数量 ≥ 实测下界 85**」，验收命令 `cargo test -p ti-tools --test pak_key`。

### EXP-018 格式：重打包必须区分「可重导」与「原样复制」（S1.2 实测）
- 现象：`--repack` 首次实现把索引重新 zlib 压缩，`.lpk`（原始索引）**逐字节一致**，但 4 个 `.pak` 全部不一致（长度甚至差 2 字节）。
- 根因：原版用 **zlib 1.1.4** 压缩索引（exe 内含 `inflate 1.1.4 Copyright...`），而 flate2 默认参数/实现不同 → 压缩流必然不同。
- 解法（**分两层验证**，别用"整文件重压缩"当 roundtrip）：
  1. `encode_index()`：由 `entries` 重新编码索引 → 对 `.lpk` **实测逐字节一致**（这就是「16 字节记录布局 + 头部偏移理解正确」的硬证明）；
  2. `repack()`：头部三个长度字段**重算写回** + 索引/数据区**原样复制** → 7 个容器全部逐字节一致。
- 教训：**P0-3「逐字节一致」不等于「重新压缩一致」**。凡是"原版用第三方压缩器"的格式，验收要拆成"结构重导"和"载荷保真"两条，否则永远达不到，或者被迫降低标准。
- 附带：`.pak` 的 zlib 索引改为**按魔数判别**（EXP-017 已记），与版本号/扩展名无关。

### EXP-019 格式：游戏自产 PCX 的头字段是「垃圾值」，靠强约束反解（S1.3 实测）
- 现象：按标准 PCX 规范读 `NPlanes@0x3D`、`BytesPerLine@0x3E`，结果 `planes=0`、`bytesPerLine=0`，解码必然失败。
- 实测（`ui.pak` 首条，`0A 05 01 08`）：`0x3D=0`、`0x3E..0x3F=0`、**`0x42=34`**（恰等于 width），而 `Xmax@0x08=33 / Ymax@0x0A=11` → 34×12 与 F-205 吻合 → 说明宽高字段可靠、跨度字段被写坏。
- 解法：**不猜字段，用强约束反解** —— 在候选跨度 `{width, word(0x3E), byte(0x42)}` 里挑**能让 RLE 恰好消费到文件尾**的那个。这是通用的"字段不可信时"套路。
- 教训：逆向自有格式时，**头部字段要先交叉验证**（宽高 vs 数据长度 vs 文件大小），别直接套标准规范；"读到的 0"要么是"没有该字段"，要么是"写坏了"。
- 附：Python 参考实现 `scripts/pcx_ref.py` 与 Rust 解码器用**同一反解逻辑**做交叉验证（P0-4），指纹用 FNV-1a 64（无依赖、可手算）。

### EXP-020 格式：`.Map` 宽高不能写成格式约束（S1.4 实测）
- 现象：`MapHeader` 一开始把 `width = 176`、`height = 220` 写成 `define_format!` 的**恒定值约束**，结果 `glmapm01.Map`（地形图，头 20×20）直接被 `InvalidValue{field:"width"}` 拒掉。
- 解法：**格式层只声明字段，语义校验放调用层**。`.Map` 是两类共用容器（UI 176×220 / 地形 20×20），凡是"同一文件格式承载多种实例"的，宽高/类型这类值必须留给校验层。
- 附带更正：F-301 记「UI 场景族 **21** 个」有误，**实测 20 个**（+ 3 个地形图 = 23 个 `.Map`）；已更新事实库与单测断言。
- 附带拆分：P1-3（控件事件 id 全量获取）依赖 S0.4 的 exe 事件表（S0.4 因 IDA Hex-Rays ARM 被阻塞），故从 S1.4 拆出 **S1.4b** 并标 blocked，避免"带病标绿"。

### EXP-021 音频：整数归一会静默变 0；WAV 实为 8bit（S2.3 实测）
- **坑 1（严重）**：`to_i16_clipped` 最初写成 `scale = i16::MAX / peak`（**整数除法**）。当 `peak=40000` 时 `32767/40000 == 0` → 所有样本被乘 0 → **整段静音**，且不报错。
  → 解法：比例运算一律升到 `i64` 再截断：`v * i16::MAX / peak`。**凡是"先算系数再乘"的归一/缩放，必须用浮点或 i64，不能整数除法取系数。**
- **坑 2**：`music.lpk` 的 WAV 是 **8kHz / 8bit / 单声道**（F-208 的"8kHz"只对**首条**成立，第 2~5 条采样率不同，另有 1 条 `glmusic.mod` **不是 RIFF**）。8bit 是**无符号**（0..=255，静音点 128），须 `(v as i16) - 128` 居中。
- **坑 3（门禁）**：gate 第 6 项的 unwrap/expect 扫描**只豁免 `tests/` 目录**，不豁免 `src/` 里的 `#[cfg(test)] mod tests`。因此**内联单测里不能用 `.expect()`**。
  → 结论：**单元测试一律放 `crates/<crate>/tests/`**，不要写在 `src` 内联模块里（否则既触发门禁，又容易被 `-D warnings` 的 `panic = deny` 拦住）。
- 验收证据：`tests/audio.rs` 6 项（16 路满音量叠加归一无削波、5 条 WAV 同时播放 4000 帧无爆音、BGM 循环/SFX 自动停止/音量 0 静音）。

### EXP-022 存储：三端差异只放后端，语义用「同一套断言」锁（S2.4 实测）
- 设计：`storage::Backend` trait（`load/store/remove/keys`）+ 两个可测实现：`MemoryStore`（headless / 语义基准）与 `FileStore`（桌面，每键一文件）。
- **三端一致性做法**：`tests/storage.rs` 里 `all_backends_share_identical_semantics` 对**同一批键值跑两遍**并逐项比对返回值与键集合；S2.5/S2.6 接上 IndexedDB / Android 后端时，**必须复用同一组断言**（把新后端塞进同一个 for 循环），而不是另写一套测试。
- 键名规范 `validate_key`：ASCII 字母数字 + `_ . -`，长度 1..=64；**禁止路径分隔符**（否则桌面文件实现会被目录穿越），非法键在三端都必须返回错误而不是静默成功。
- 体积：`MemoryStore::bytes_used()` 用于存档上限校验；512 KB 存档往返已覆盖。
### EXP-023 格式：INI 是 UTF-16LE 且必须保留原文才能往返（S1.6 实测）
- `cards.lpk` 4 条里 **2 条是 UTF-16LE INI**（带 UTF-16 BOM `FF FE`，行尾 CRLF），另 2 条是二进制表 → 判定方法：`bytes[0..2] == FF FE`。
- 索引对应（`djb2` 小写实测）：`event.ini = 0x43104390`（索引 **0**）、`cards.ini = 0xCA00521B`（索引 **3**）。⚠️ 之前按"文件体积猜哪个是 cards.ini"猜错了——**必须用 `name_key` 反查定位，不能凭大小/顺序猜**。
- 内容：`event.ini` = `[event] Count=22` + `[event1..22]`；`cards.ini` = `[card] imageCount=2 cardCount=26` + `[image1..2]`（`file = cards\card1.pcx`）+ `[card1..26]`。
- **往返无损的关键**：解析时保留 `raw`（原始文本），序列化时直接写回 raw，而不是用结构重新拼 —— 否则注释（`; event catalog` 那 40 多行枚举说明）、空行、缩进会丢失，roundtrip 必然失败。
- 中文值需 JSON 转义为 `\uXXXX`（`json_escape`），控制台是 GBK，直接打印会乱码 —— **测试断言要针对 JSON 文本，不要针对终端输出**。
### EXP-024 字体：`GoldLand.Fnt` 位图的"12 像素"来自 bit 使用统计（S1.5 实测）
- 结构实测：头部 24B = **12 个 u16 字宽表** `[0,24,60,60,60,24,24,0,24,24,0,0]`；其后 24360B = **1015 个字形 × 24B（12 行 × 2 字节）**。
- **关键定位手法**：统计每个 bit 位的置位次数 → u16 LE 的 **bit8~bit11 恒为 0** → 每个字素行只用了「低字节 8 位 + 高 4 位」= **12 像素宽**。这比"猜 12×12/16×16"快得多，也比"看渲染结果像不像"可靠。
- ⚠️ **仍未逆出**：行列组织与 MSB/LSB 位序（按 12 像素渲染出来是噪声，不是汉字）。故把 P4-3「逐位图一致」拆为 **S1.5b**，S1.5 只交付「结构完整 + 0 未知字节 + 往返逐字节」—— 不带病标绿。
- `font.dat` 2034B 以 `FF FE`（UTF-16 BOM）开头，表数据同样待 S1.5b。
- 教训：**位图格式先做"位使用统计"再猜几何**；统计能直接给出像素宽度，比试渲染快几个数量级。
### EXP-025 格式：ID 型 key 的绑定藏在容器自身的 `mapinfo.ini`；AP 公差必须脚本实算（S1.11 实测）
- **推翻 F-209 旧推测**：ID 型 key **不是**由 `.Map` 场景按索引绑定，而是容器内自带 `mapinfo.ini`（key `0x3753DC98`，UTF-16LE+BOM，逐条 `filename=`）→ `key = djb2(filename)` 100% 闭合（ui 121 + m01 49 + m02 34 行 0 失配）。→ **教训：容器类格式找绑定关系时，先枚举容器内所有条目看有没有"清单文件"，别先猜运行时机制。**
- **推翻公差 `0x121101`**：idx0..20 实测公差集合 = {`33^4` = `0x121881`, `24×33^4`（`09→10` 进位 = `(33-9)×33^4`）}。旧值是手抄十六进制的错。→ **教训：字符串序列构成的等差数列，公差按字符位权（`33^len(尾缀)`）用脚本逐项算，禁止手算十六进制。**
- **假 preimage 警示（模逆求解器）**：`opjd79.sav`、`XZ_0_table.fnt`、`._2e52.wav` 等"命中"全是假名（内容实为 PCX/OLE）。**单键命中一律只能标推测**；可信证据 = 内容格式吻合（`Thumbs.db` 内容是 OLE 复合文档）或**多键同时命中**（`appo01~04.pcx` 四连 + `card1/2.pcx` 两连，碰撞概率可忽略）。
- 穷举 3 轮无果的键（glpics 3 条）按"卡 3 轮拆分"原则收手标 unknown，不带病标绿；`ui[121]` 靠**固定候选表里的系统约定文件名**（`Thumbs.db`，Windows 缩略图缓存 = OLE 格式）一发命中 → **候选表要包含操作系统约定文件名，别只用 exe 里出现过的词。**
- `.Map` UI 记录布局同时逆出（F-306）：`words.len() == 5 + 6×count`（`count = words[4]`），20/20 匹配；字段级语义（`f3` 值域超出 mapinfo 条目数）仍属 S1.4b。
### EXP-026 字体：「头部字宽表」误读与 MSB-first 位序的几何判定（S1.5b 实测）
- **推翻 EXP-024/F-303**：GoldLand.Fnt 没有头部 —— 24384 = **1016 × 24 精确整除**，前 24B 是第 0 个字形（ont.dat 首字符 ！ 的点阵）。教训：文件长度先做整除分解再猜结构；「头部」假设必须先证明 len - head 不能被记录尺寸整除。**唯一字符 1015 ≠ 字形 1016** 正是因为 记 重复两处（且两处字形字节全同）—— 数量对不上时先查重复项，别急着引入对齐偏移。
- **位序/行序判定法（可复用）**：不用试渲染，直接用**排版结构特征**做几何断言 ——  竖杆在左 + bowl 在右下 + 上伸部在顶 + 下缘留白（同时排除水平翻转、垂直翻转）；d =  的镜像校验水平向；p 下伸部校验垂直向；L 脚衬线只向右伸；A 尖顶在第 1 行。5 个字符共 ~15 个断言把 4 种候选解码（行序 2 × 位序 2）收敛到唯一解。
- 本字体行布局：像素 = **字节序 MSB-first 取前 12 位**（字节0 全用 + 字节1 高 4 位），字节1 低 4 位恒 0 是填充 —— 与旧记录「低 8 位 + 高 4 位」数值一致但方向（MSB 在左）是新确认的。
- ont.dat 是 **UTF-16 字符集清单**（非字宽表）：解析直接复用 UTF-16 解码 + 去 BOM，别按二进制表去猜。步进无表 → Glyph::width()=右缘墨迹+1，标**推测**。
- 测试里的字母变量命名别用单字符（/d/p/l/a）→ clippy many_single_char_names（pedantic），用 glyph_b 等。

### EXP-027 格式：地形 `.Map` 的「区域表」是整条 cell 流，头字段 `regions_off` 是尺寸不是偏移（S1.8b 实测）
- **推翻旧文档注释**：`terrain.rs` 原写「`@0x14` 为区域表起始 + 交错区域描述块，`regions_off` 指向其内偏移」。实测 `regions_off = 32 × count`（608/448），**按它当字节偏移去读 8 词记录，得到的是错位的 cell 流垃圾**（看起来像表、实则每 8 词错 2 词）。→ **教训：先验证偏移能否落在记录边界上**（`(off - start) % stride == 0`），否则大概率是尺寸/参数字段，不是偏移。
- **真正结构**：`words[1]=count = grid_w-1`；`words[2..]` 是**一整条 stride=9 的 cell 记录流**（`grid_w×grid_h-1` 条）+ 1 条 trailer + N 条连线记录。**没有独立的「区域表」**——count 只是 `grid_w-1`，其记录内容就是网格。→ **教训：文档里的「表」概念要先证明它真的是独立区段**；否则会把「前 count 条记录」误当成「一张表」。
- **cell 记录恒等式**（3 图全量验证）：`f6=32×(row+col)`、`f7=16×(row+gw-1-col)`、`f8低16位=row×gw+col`，覆盖 `0..gw×gh` **除 cell `gw-1` 外全部且无重复**。→ **教训：字段语义靠「全量不变量」锁，不要靠单图采样**。
- **推翻「`f1 = cell×0xA0001`」**：m01 恰有 `f1高16位 = 10×f1低16位`（39/39 条成立），看着像哈希；**m02 cell 258 是 `hi=430, lo=81`（430≠810），假设立刻崩**。→ **教训：单图上 100% 成立的规律，换第二、第三个样本再验一遍才算结论**（同 EXP-020「20 个 vs 21 个」的教训）。正确语义是 `f0=(目标格,类别码1..=9)` / `f1=(区域锚点格,区域码)`。
- **`f0` 高 16 位有「双语义」陷阱**：目标格 ≠ 0 时是类别码（1..=9，9 值全覆盖）；目标格 = 0 时是 **16/64/128/192 的属性位**（外加两条不透明值 4/5）。→ **教训：同一字段在不同取值区间语义不同时，单测必须按「前置条件分支」分别断言**，否则会拿错语义去校验。
- trailer 的 `f5 = 连线记录数 + 1`（75+1 / 125+1 / 23+1，3/3 精确）——**这是最强的「段尾校验子」**，可直接当解析循环的终止/一致性断言。
- clippy 补充（`tests/` 内同样适用，`--all-targets`）：`.expect(&format!(...))` 触发 `expect_fun_call`（用静态字符串）；`lo.checked_mul(10).unwrap_or(MAX)` 触发 `manual_saturating_arithmetic`（改 `saturating_mul`）；`u32 as i64` 触发 `cast_lossless`（改 `i64::from`）；文档里的 `grid_w` 等标识符要加反引号（`doc_markdown`）；单字符变量名 ≥5 个触发 `many_single_char_names`。
### EXP-028 工程：macroquad wasm 是「裸 wasm + gl.js」，不是 wasm-bindgen（S2.5 实测）
- **选型落实**：workspace 此前**根本没有 macroquad 依赖**（ti-platform 只有 trait）。S2.5 才引入 `macroquad = "0.4"`（解析到 0.4.16）+ miniquad 0.4.11。
- **关键事实（省掉大量试错）**：macroquad/miniquad 的 wasm32 产物**不需要 wasm-bindgen**。`Cargo.lock` 里**没有任何 wasm-bindgen / js-sys / web-sys**；产物是自包含模块，导出 `memory`/`main`/`frame`/`key_down`/`mouse_down` 等 24 个符号。启动方式是**经典 script 引入 miniquad 的 `js/gl.js`（67KB，随 crate 发布在 `miniquad-*/js/`）后调 `load("xxx.wasm")`** —— 不是 `import()`，也不是 `WebAssembly.instantiateStreaming`。
  -> 教训：判断 wasm 前端要不要 bindgen，先 grep Cargo.lock 有没有 wasm-bindgen；有则要 glue，无则找宿主 `gl.js`（看 crate 的 `js/` 目录与 README 的 index.html 示例）。
- **自定义 boot.js 是错的**：一开始按 wasm-bindgen 习惯写了 `import('./treasure_wasm.js')`，而产物根本没有 `.js` 胶水；改为 `instantiateStreaming` 也缺 miniquad 需要的宿主回调（`frame`/`key_down` 由 JS 侧调用 wasm，方向相反）。**教训：wasm 的胶水方向要看导出表** —— 导出里有一堆 `frame`/`mouse_*` 回调，说明是 JS 驱动 wasm（宿主式），而非 wasm 调 JS。
- **debug vs release 体积**：同一工程 debug 17.7MB / release 615KB（28x）。wasm 交付一律 `--release`。
- **API 与 0.4 文档不同**（实测踩坑）：模块是 `macroquad::texture`（**无 s**）不是 `textures`；`Texture2D::from_rgba8(u16,u16,&[u8])` **不是** `from_rgba`；`Color` 是 **f32** 通道（不是 u8）；`Mesh` 是具名字段 `{vertices, indices, texture}` **没有** `Mesh::new(...)`；`#[macroquad::main]` 收 `Conf` **不是** `&str`，也没有 `Window::new`。
- **gate 解锁**：装上 target 后 gate 第 4 项 wasm 行自动由 SKIP 转 PASS（脚本按 `rustup target list --installed` 判定，无需改脚本）。
- **PowerShell 脚本必须存 UTF-8 BOM**（EXP-016 再次命中）：`build_wasm.ps1` 用 Write 工具写出（无 BOM）时，PS 5.1 按 GBK 读中文注释，把字节打断导致 `ParserError: UnexpectedToken`（报错位置与真实语法错误无关，容易误判）。补救：`.ps1` 一律加 BOM。
- **clippy（`--all-targets` 才会扫到）**：`f32 as i64/usize` 触发 `cast_possible_truncation`/`cast_sign_loss` —— 逻辑坐标一律用整数（`i32`/`usize`），别在像素索引上用 f32；`as f32` 触发 `cast_precision_loss`（EXP-015 已记，用 `LW`/`LH` 常量）；`require-features` 的 bin 默认 lint **不扫**，要 `clippy --features wasm-shell` 单独跑。
