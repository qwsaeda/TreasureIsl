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
