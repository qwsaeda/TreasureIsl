# 可执行文件 `TreasureIsl.exe`


**收录规则**：只写有脚本/工具证据的结论，标注 `证据`；推测必须写 `推测` 并说明待验证方法。
新增/推翻时更新本文件并在 `docs/experience/index.md` 记一笔。F-ID 全局唯一，门禁第 9 项 `doccheck` 校验。

## 0. 工具环境（实测 2026-10）

| 事实 | 证据 |
|---|---|
| IDA Professional **9.4** 在 `D:\soft\IDA Professional 9.4\`；headless 文本模式是 `idat.exe`（9.x 已合并 7.5 的 `idat64.exe`），ARM 处理器模块 `procs\arm.dll` 加载正常，Hex-Rays ARM 反编译可用 | 目录列表 + `ref/ida_run.log` 首段 `Loading processor module D:\soft\IDA Professional 9.4\procs\arm.dll for ARM...OK` |
| S0.2 实测：1827 函数 → 1827 个 `.c` + `functions.csv` 1827 行，其中 Hex-Rays 成功 **1823**、失败 4（脚本内已回退为反汇编） | `evidence/S0.2.log` |
| `TreasureIsl.exe.i64` 5.4MB，magic `IDA2`，已自动分析；函数名全是 `F1102C` 形式，**无任何人工命名** | 扫描 i64 字符串 |
| rustc/cargo 1.96.0 stable、clippy 0.1.96、rustfmt 1.9.0；已装 target 仅 `x86_64-pc-windows-msvc` | `rustc --version` / `rustup show` |
| **无 Docker**、无 Android SDK/NDK 环境变量；Java 17 已装 | 命令探测 |
| Python 3.13（miniconda），无 `pefile`/`capstone` 模块 | `python -c "import pefile"` 失败 |
| `D:\frida` 存在 | 目录探测 |

## 1. PE 头与节表

| ID | 事实 | 证据 |
|---|---|---|
| F-101 | PE32、`machine=0x01C0` (ARMNT/Thumb-2)，5 节，ImageBase `0x10000`，Entry RVA `0x59FB0` | 解析 PE 头 |
| F-102 | 节表：`.text` RVA 0x1000 sz 0x59D88 / `.rdata` 0x5B000 / `.data` 0x6D000 / `.pdata` 0x72000 sz 0x35F8 / `.rsrc` 0x76000 | 同上 |
| F-103 | 导入 3 个 DLL：`COREDLL.dll` 105 个**按序号**导入（无名字）、`gx.dll` 9 个 C++ 修饰名（GXBeginDraw/EndDraw/OpenDisplay/CloseDisplay/GetDisplayProperties/OpenInput/CloseInput/Suspend/Resume）、`AYGSHELL.dll` 2 个 | 解析导入表 |
| F-104 | 内含第三方库字符串：`Hekkus Sound System ver.0.98`、`inflate 1.1.4 Copyright ... Mark Adler`（zlib） | ascii 字符串扫描 |
| F-105 | 资源文件名在 exe 中是 **UTF-16 宽字符**（如 `button_base.pcx`、`data\gl0.sav`、`cards\cards.ini`、`music\glmusic.mod`），共 119 个含扩展名的资源名 | utf-16-le 正则扫描 |
| F-106 | 画面逻辑分辨率 **176×220**（所有 UI 类 `.Map` 头部前两个 u32 = `0xB0,0xDC`） | 逐文件读头 |
| F-107 | `.pdata` 按 x86 `RUNTIME_FUNCTION` 12 字节解析会得到荒谬 size（负数/上 GB）；**CE ARM 布局待确认**，函数总数以 IDA 的 `.i64` 为准 | `.pdata` 解析尝试（错误，见 EXP-005） |
| F-108 | 版本资源/字符串：`RIO WU / EICKE GAO / KEN ZHANG / DAVID YU / BIN / THAWYICE / YINGZI / MARU / YANZI / GREATELSOFT / YINXIN`（开发者署名，UTF-16） | 宽字符扫描 |
