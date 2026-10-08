# 逆向事实库（Reverse Facts）

**收录规则**：只写有脚本/工具证据的结论，标注 `证据`；推测必须写 `推测` 并说明待验证方法。
每条有 ID（`F-xxx`），新增/推翻时更新本文件并在 `docs/experience.md` 记一笔。

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
| F-201 | magic `LucencyPackage\0\0`（16B）；头部字段**不是 4 字节对齐**：版本号 u16 `@0x1A`、条目数 u32 `@0x26`、**索引流长度 u32 `@0x2E`**、数据区起点 u32 `@0x32`、索引流起点固定 `@0x36`。⚠️ 旧记录写 `@0x2A` 是**错的**（该处读出 `01 00 00 00`），实测以 `@0x2E` 为准（S1.1 hexdump 复核）→ EXP-017 |
| F-202 | 索引 = `条目数 × 16` 字节，记录 `{u32 key, u32 off, u32 usize, u32 csize}`，`off` 相对数据区起点。**是否 zlib 压缩由索引首 2 字节的 zlib 魔数判定**：`.pak` 全部压缩，`.lpk` 全部为原始记录；**与版本号无关**（`glpics.pak` 版本亦为 2005 但索引仍压缩）→ EXP-017 |
| F-203 | 数据区每条目是**独立 zlib 流**（`78 DA`）；`ui.pak` 共 123 流 = 1 索引 + 122 数据 |
| F-204 | **`key = djb2(name.to_ascii_lowercase())`，种子 0，`h = h*33 + c`**（大写与小写名同 key）。实测向量：`landinfo.pcx`=0x5C5AF184、`number.pcx`=0xFCFFFB62、`dialogue.pcx`=0xFFFB9A43、`logo.pcx`=0x047D426A、`mapinfo.ini`=0x3753DC98。⚠️ 旧记录「不是任何常见哈希 / 高字节恒为 0x2F」**均已被推翻**（高字节实为 23 种不同值） |
| F-205 | `ui.pak` 首个数据条目 = PCX：`0A 05 01 08`，8bpp、1 平面、34×12（`Xmax@0x08=33`、`Ymax@0x0A=11`）。⚠️ **行跨度字段不可信**：`NPlanes@0x3D=0`、`BytesPerLine@0x3E=0`，真正的跨度在 `0x42`（=34=width）；实现用「RLE 恰好消费到文件尾」反解 → EXP-019 |
| F-206 | 用 `b'\x78\xda'` 全文扫描找流会误报（`cards.lpk` 偏移 27358 处解压报错）；**必须按索引记录遍历** → EXP-004 |
| F-207 | 容器清单与规模：`ui.pak` 122 条 / `glpics.pak` 88 条 / `glmap_m01.pak` 47 条 / `glmap_m02.pak` 37 条 / `cards.lpk` 4 条 / `music.lpk` 6 条 / `goods.lpk` 1 条（合计 305 条） |
| F-208 | `cards.lpk` 首条解压为 UTF-16 文本（`; event catalog` 等 INI 注释）；`music.lpk` 首条为 `RIFF/WAVE` 8kHz 8bit；`goods.lpk` 首条为二进制表 | 逐流解压看头部 |
| F-209 | **key 分两类，ID 型绑定机制已逆出（S1.11 实测）**：① 名称哈希型（`glpics.pak` 88 条中 **85 条**由 exe UTF-16 资源名唯一解释；余 3 条 `0x0D4EF1BE`/`0x20769D35`/`0x6171A968` 未解，内容均为 PCX，DoD 不含 glpics）；② **ID 型 217 条**（ui/glmap/lpk）：**绑定 = 容器内 `mapinfo.ini` 文件名清单**（见 F-210），`key = djb2(filename)` 闭合；其余 9 条由 exe 资源名（cards/music/goods）+ 多键结构证实名（`appo01~04.pcx` = m02 idx13~16、`card1/2.pcx` = cards idx1/2）+ `Thumbs.db`（ui idx121，内容 OLE 复合文档吻合）解释 → **`DoD` 范围 217/217 全解**（`evidence/S1.11-audit.json` + 单测 `id_keys`）。⚠️ 旧记录「ui 前 90 个 key 公差恒为 `0x121101`」**是错的**：idx0..20 实测公差集合 = {`33^4` = `0x121881`, `24×33^4`（进位 `09→10`}；「ID 型 key 由 `.Map` 场景按索引绑定」的推测**不成立**（`.Map` 元素字段语义属 S1.4b，依赖 exe）→ EXP-025 |
| F-210 | **容器内 `mapinfo.ini` = ID 型 key 的权威资源名清单**：key `0x3753DC98`（=`djb2("mapinfo.ini")`，各容器同 key），UTF-16LE + BOM `FF FE`，CRLF 行尾，结构 `[filenum]` + `filenum=N` + 逐条 `[i]` / `filename=xxx` / `num=1`。行数：`ui.pak` 121（`01.pcx`..`120.pcx` + 重复 `50.pcx`，唯一 120）/ `glmap_m01.pak` 49（唯一 46）/ `glmap_m02.pak` 34（唯一 32）；容器内索引位 ui idx21、m01 idx5、m02 idx4。全部 `filename` 的 `djb2` 均在同容器 entries 中闭合（121+49+34 行 0 失配，单测 `mapinfo_filenames_close_over_container_keys`） |

## 3. `data/` 其它格式

| ID | 事实 |
|---|---|
| F-301 | `.Map` 分两族：**UI 场景族**（`main/card/build/...` 头 `B0 00 00 00 DC 00 00 00` = 176×220 + u32 控件记录流，文件 276B~5124B，**实测 20 个**）与**地形族**（`glmapm01/02/03.Map` 头 `14 00 00 00 14 00 00 00` = 20×20，17148B/18948B/8976B）。⚠️ 原记「21 个」有误，实测 20 → EXP-020 |
| F-302 | `.Map` 内**无任何字符串**（ascii/utf-16 扫描均空）→ 资源绑定靠索引/key，解析必须对照 exe 与 pak 索引 |
| F-303 | **`GoldLand.Fnt` = 1016 字形 × 24B，无头部**（24384/24 精确整除，S1.5b 实测）。⚠️ 旧记录「前 24B 是字宽表 `[0,24,60,60,60,24,24,0,24,24,0,0]`」**是错的**：那 24B 是第 0 个字形（对应 `font.dat` 首字符 `！`，行向量 `[0,0x180,0x3C0,0x3C0,0x3C0,0x180,0x180,0,0x180,0x180,0,0]`）。字形位图 = 12 行 × 2B，**MSB-first 在左**（像素 x<8 取字节0 的 bit(7-x)，x≥8 取字节1 的 bit(15-x)），**字节1 低 4 位恒为 0**（全 12192 行 0 违例 = 填充），行序**自上而下** —— 由 `tests/font_parity.rs` 用 `b/d/p/L/A` 几何断言 + `！` 行向量验证。`font.dat` 2034B = UTF-16LE + BOM 的**字符集清单**（非字宽表），1016 字符与字形下标 1:1（唯一 1015，`记` 重复 @981/@996 且两处字形字节全同）；字符→字形取首个出现位。步进（advance）无字宽表 → `Glyph::width()` = 右缘墨迹+1，**推测**运行时按此或定宽前进 |
| F-304 | `settings.dat` = 8 字节 `5D 4A 71 4A 2E 01 01 01`；存档 `gl0~4.sav`/`gl.rec` 尚未生成（运行时写出） |
| F-305 | `.lpk` 与 `.pak` 头部结构不同（zlib 起始偏移不同：cards 118 / music 150 / goods 70），索引位置待 S1.1 确认 → 推测 两者同一格式但头部含变长区 |
| F-306 | **UI 场景 `.Map` 记录布局（S1.11 实测，20/20 匹配；S1.4b 实测复核 20/20 整除吻合）** = `[16B 头][子头 4×u32][count u32][count × 6 × u32 记录]`，即 `MapScene.words.len() == 5 + 6×count`（`count = words[4]`）。子头实测取值 `(160,192,1,0)` / `(176,219,1,0)` / `(176,220,1,0)` / `(168,216,1,0)` / `(0,0,0,0)`；记录 `f0` 恒为 1、末两字恒为 `1,0`。⚠️ **字段级语义已由 S1.4b 改写：f3 并非事件 id，而是屏幕 Y 坐标，见 F-311**（原"f3 是控件事件 id、0xFFFFFFFF/597 超 mapinfo 条目数"的推测作废）。单测 `ui_scene_map_record_layout_matches_count` + `crates/ti-formats` 的 `MapControl` 记录级视图 |
| F-307 | **地形族 `glmapm0*.Map` 完整记录布局（S1.8b 实测，3/3 匹配）**：`words[0]=0`、`words[1]=count` 且 **`count = grid_w - 1`**（m01/02=19、m03=14）、头字段 `regions_off = 32 × count`（608/448）—— ⚠️ **它是字节尺寸不是文件偏移**，落在 cell 流内部（按该偏移读 8 词记录会得到错位垃圾，勿再当表首）。`words[2..]` 为 stride=9 u32 记录流，分三段：<br>① **cell 记录 `grid_w × grid_h - 1` 条**（m01/02=399、m03=224）`[f0,f1,f2=0,f3=0,f4=1,f5,f6,f7,f8]`，恒等式 3/3 全量验证：**`f6 = 32×(row+col)`**、**`f7 = 16×(row+grid_w-1-col)`**（等距投影坐标）、**`f8 低16位 = row×grid_w+col`**（cell 索引，无重复、覆盖 `0..grid_w×grid_h` 除 **cell `grid_w-1`** 一格外）；`f2/f3` 恒 0；`f5` = `mapinfo.ini` 瓦片索引（m01 实测 max 44、m03 max 44 < 49 条 filenum）；<br>② **trailer 记录 1 条**（`f4=1` 但字段不遵 cell 公式）：**`f5` = 连线记录数 + 1**（75+1=76 / 125+1=126 / 23+1=24，3/3 精确吻合），`f6/f7/f8` 不遵任何已验证公式 → **推测**为统计/边界字段；<br>③ **连线记录 N 条**（m01=75、m02=125、m03=23）`[0,0,0,0,0,M,A,B,C]`：**`M ∈ {2,4}`**（m02 独有 4 条 `M=4`，其余全 `M=2`）、`A` = 瓦片索引（< 49）、`B/C` = 等距像素坐标（**`B` 100% 落在 `0..32×2×(gw-1)`、`C` 100% 落在 `0..16×2×(gw-1)`**，`C` 按**非降序**排列 = painter's 绘制序），文件尾固定 6 词 sentinel `[0,0,0,0,0,2]`。单测 `terrain_records.rs`（13 项） |
| F-315 | **事件 id 非 `.Map` 记录字段，而在运行时控件对象上（S1.4b 实测，`ref/event_ids.json`）**：`sub_31424(ctrl) = *(u8*)(*(u32*)(ctrl+12)+22) >> 2 + 1`，读取 `ctrl+12` 槽所指向事件描述符对象的 +22 字节。`ctrl+12` 槽 = `sub_449C8` 的 `v7[3]`（即 `.Map` f1 落点），但 `sub_45EC0` 仅初始化对象前 52 字节（`result[3]` = +12 槽 = 0），故 `*(ctrl+12)` 在 `.Map` 加载后为 0 或由 S3.3+ 场景逻辑回填事件描述符指针。**推论：`.Map` 6 字段不含事件 id**，事件分派 case 1..8（`sub_2101C` 行 1011/1709/2561 三处调用）语义 = 场景切换/对话框/卡牌/贸易/翻转/战斗/消息/退出。`ctrl+12` 槽的回填时机与值域属 S3.3+ 场景逻辑，届时逐 id 确认。 |
| F-311 | **UI `.Map` 控件记录 6 字段语义（S1.4b 实测，exe 链 `sub_44004`→`sub_46294`→`sub_449C8`→`sub_460F8` 双向确认）**：`f0` 恒为 1（推测 使能标志）；`f1` 精灵/贴图索引（1..119，84 种，对应 `ui.pak`/`glpics.pak` 位图）；`f2` 屏幕 **X** 坐标（`sub_460F8` 用作 blit 原点 `dst.x`，实测 p95 = 176 = 场景宽）；`f3` 屏幕 **Y** 坐标（`dst.y`，p95 = 216 < 场景高 219）——**F-306 的 "f3 是控件事件 id" 已推翻**；`f4` ∈ {1,2,3} = 贴图例程模式（`sub_460F8` 依此与精灵位数 16/8 选择 6 个 blit 例程之一）；`f5` 控件类型码（95% 为 0，非 0 值集中在 20..1400 分档，100/200/…/1400，语义随 S3.3+ 解释）。全量清单见 `ref/controls.json`（`cargo run -p ti-tools --bin controls -- --out ref`，20 场景 2041 记录 44 种 f5）。 |
| F-308 | **地形 cell 的连线字段语义（S1.8b 实测）**：`f0 = (u16 目标格, u16 类别码)` —— 目标格 ≠ 0 时（m01/m02/m03 分别 75/36/31 条）**类别码恒为 1..=9 且 9 个值全覆盖**、目标格恒满足 **`|Δcol| ≤ 1` 且 `Δrow ≤ 0`**（同列或邻列、绝不向下）→ 即**路线邻接**；目标格 = 0 时（m01/02/m03 分别 4/6/4 条）高 16 位是 **16/64/128/192** 的单元格属性位（外加 m02 两条 4/5 不透明值）→ **不是**类别码。`f1 = (u16 区域锚点格, u16 区域码)`，锚点非 0 时（39/40/31 条）**100% 是合法 cell 索引**，与 `f0` 同现。⚠️ 早期「`f1 = cell×0xA0001`」的结论是 m01 单图巧合（m01 有 `hi=10×lo`，m02 cell 258 为 `hi=430, lo=81`），已推翻 → 见 EXP-027 |

## 4. 已知缺口（按计划补齐）

- `name→key` 哈希算法（S1.1）；`.Map` 记录字段语义（S1.4/S1.8）；`.Fnt`/`font.dat` 全布局（S1.5）；coredll 序号→API 名（S0.4）；玩法逻辑全量（S3）。
- UI 控件记录 6 字段语义已定（F-311）；`f5` 控件类型非 0 档（100/200/300…1400）逐档含义与交互行为（S3.3+）。
- **没有可运行原版的环境**：WinMo 模拟器仅 x86 镜像，本程序是 ARM → 视觉对照改用「资源解码独立参考实现逐像素比对」，可选后续 QEMU-CE（未验证可行性）。

## 5. 逆向工作流（固定，勿改）

```
1) IDA headless 导出："D:\soft\IDA Professional 9.4\idat.exe" -A -Lref\ida_run.log -Sscripts\ida_dump.py TreasureIsl.exe
   产出 ref/pseudocode/*.c + ref/functions.csv
2) 分类打标：cargo run -p ti-tools -- classify   → ref/classify.json
3) 结论回写本文件（带证据），再更新 WBS/台账

4) 控件记录导出：cargo run -p ti-tools --bin controls -- --out ref   → ref/controls.json（S1.4b）
```

## 6. 工具链事实

| ID | 事实 | 证据 |
|---|---|---|
| F-313 | **REA（rea-agents 6.0.0）headless 经 `ida-pro-mcp` supervisor 分析 TreasureIsl.exe 正常**（`open_binary`/`analyze_function` 出 Hex-Rays ARM 伪代码 + 汇编 + 证据包，`list_procedures` 19s 全量函数表）。注册在 `~/.rea/ida-mcp.json` + OpenCode `mcp.rea.environment`（`REA_ANALYSIS_PROVIDER=ida`），见 AGENTS.md 末尾 | `rea doctor --provider ida` healthy:true；证据包 sha256 `2a80f209…` |
| F-314 | **REA↔OpenCode 互操作缺陷（6.0.0 实测）**：① 不产出 Evidence 的工具（`read_bytes` 等）声明的 outputSchema 强制 `result/evidence_id/evidence` 且 `additionalProperties:false`，实际不返回 → MCP `-32602`，需用本地脚本替代；② modern supervisor profile 下 `callers` 恒空（REA 自注 "Direct callers unavailable"），调用方需靠本地伪代码语料 grep。分工：REA 出权威证据，本地 grep 做候选发现 | 2026-10 会话实测 |
