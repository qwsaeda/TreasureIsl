# 逆向事实库（Reverse Facts）

**收录规则**：只写有脚本/工具证据的结论，标注 `证据`；推测必须写 `推测` 并说明待验证方法。
每条有 ID（`F-xxx`），新增/推翻时更新本文件并在 `docs/experience.md` 记一笔。

## 0. 工具环境（实测 2026-10）

| 事实 | 证据 |
|---|---|
| IDA Pro 7.5 portable 在 `D:\soft\IDA_Pro_v7.5_Portable\`，含 `plugins\hexarm.dll`（ARM Hex-Rays 反编译器）与 `python38`/IDAPython | 目录列表 |
| `TreasureIsl.exe.i64` 5.4MB，magic `IDA2`，已自动分析；函数名全是 `F1102C` 形式，**无任何人工命名** | 扫描 i64 字符串 |
| rustc/cargo 1.96.0 stable、clippy 0.1.96、rustfmt 1.9.0；已装 target 仅 `x86_64-pc-windows-msvc` | `rustc --version` / `rustup show` |
| **无 Docker**、无 Android SDK/NDK 环境变量；Java 17 已装 | 命令探测 |
| Python 3.13（miniconda），无 `pefile`/`capstone` 模块 | `python -c "import pefile"` 失败 |
| `D:\frida` 存在 | 目录探测 |

## 1. 可执行文件 `TreasureIsl.exe`

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

## 2. 资源容器 `LucencyPackage`

以 `data/ui.pak` 为样本（F-2xx 全部实测）：

| ID | 事实 |
|---|---|
| F-201 | magic `LucencyPackage\0\0`（16B）；头部字段**不是 4 字节对齐**：条目数 u32 @`0x26`（=122）、索引流长度 u32 @`0x2A`（=1470）、数据区起点 u32 @`0x32`（=1524）。按 4B 对齐解析会失败 → EXP-003 |
| F-202 | 索引流 = 文件偏移 54 起的 zlib 流，解压得 `条目数 × 16` 字节；记录布局 `{u32 key, u32 off, u32 usize, u32 csize}`，`off` 相对数据区起点（1524）。逐条校验：`off/usize/csize` 与实际 zlib 流位置/长度完全吻合 |
| F-203 | 数据区每条目是**独立 zlib 流**（`78 DA`）；`ui.pak` 共 123 流 = 1 索引 + 122 数据 |
| F-204 | `key` **不是**名字的 crc32/crc32(utf-16)/fnv1a/adler32（119 个名字 × 4 算法全 miss）。样本 key `0x2f5cfa6a, 0x2f6f12eb, 0x2f812b6c...` 高字节恒为 `0x2F` 且单调递增 → 算法待逆（S1.1） |
| F-205 | `ui.pak` 首个数据条目 = PCX：`0A 05 01 08`，8bpp、1 平面、34×12、bytesPerLine=34（RLE 编码） |
| F-206 | 用 `b'\x78\xda'` 全文扫描找流会误报（`cards.lpk` 偏移 27358 处解压报错）；**必须按索引记录遍历** → EXP-004 |
| F-207 | 容器清单与规模：`ui.pak` 122 条 / `glpics.pak` 90 流 / `glmap_m01.pak` 48 流 / `glmap_m02.pak` ~ / `cards.lpk` 5 流 / `music.lpk` 6 流 / `goods.lpk` 1 流 |
| F-208 | `cards.lpk` 首条解压为 UTF-16 文本（`; event catalog` 等 INI 注释）；`music.lpk` 首条为 `RIFF/WAVE` 8kHz 8bit；`goods.lpk` 首条为二进制表 | 逐流解压看头部 |

## 3. `data/` 其它格式

| ID | 事实 |
|---|---|
| F-301 | `.Map` 分两族：**UI 场景族**（`main/card/build/...` 头 `B0 00 00 00 DC 00 00 00` + 控件记录表，文件 276B~5124B，21 个）与**地形族**（`glmapm01/02/03.Map` 头 `14 00 00 00 14 00 00 00` = 20×20 等，17148B/18948B/8976B） |
| F-302 | `.Map` 内**无任何字符串**（ascii/utf-16 扫描均空）→ 资源绑定靠索引/key，解析必须对照 exe 与 pak 索引 |
| F-303 | `GoldLand.Fnt` 24384B：前 4×u16 疑似字宽表（`00 00 18 00 3C 00 3C 00...`）后接位图数据；`font.dat` 2034B 以 `FF FE`（UTF-16 BOM）开头后接表数据（未完全逆出） |
| F-304 | `settings.dat` = 8 字节 `5D 4A 71 4A 2E 01 01 01`；存档 `gl0~4.sav`/`gl.rec` 尚未生成（运行时写出） |
| F-305 | `.lpk` 与 `.pak` 头部结构不同（zlib 起始偏移不同：cards 118 / music 150 / goods 70），索引位置待 S1.1 确认 → 推测 两者同一格式但头部含变长区 |

## 4. 已知缺口（按计划补齐）

- `name→key` 哈希算法（S1.1）；`.Map` 记录字段语义（S1.4/S1.8）；`.Fnt`/`font.dat` 全布局（S1.5）；coredll 序号→API 名（S0.4）；玩法逻辑全量（S3）。
- **没有可运行原版的环境**：WinMo 模拟器仅 x86 镜像，本程序是 ARM → 视觉对照改用「资源解码独立参考实现逐像素比对」，可选后续 QEMU-CE（未验证可行性）。

## 5. 逆向工作流（固定，勿改）

```
1) IDA headless 导出：D:\soft\IDA_Pro_v7.5_Portable\idat64.exe -A -Sscripts\ida_dump.py TreasureIsl.exe
   产出 ref/pseudocode/*.c + ref/functions.csv
2) 分类打标：cargo run -p ti-tools -- classify   → ref/classify.json
3) 结论回写本文件（带证据），再更新 WBS/台账
```
