# 经验库 · reverse

> **定位**：逆向方法论卷（exe 内部结构、语义交叉验证、结论推翻流程）。**历史参照卷**——记录逆向期的踩坑与方法，**非正向开发依据**。逆向产物实体归档 `legacy/`，本卷仅作经验溯源。

> 条目 ID 全局唯一（EXP-NNN），维护规则与全量索引见 docs/experience/index.md。

### EXP-005 流程：`.pdata` 别按 x86 布局硬解
- 现象：按 12 字节 `RUNTIME_FUNCTION` 解析 `.pdata` 得到 size 为负数/上 GB 的荒谬结果。
- 根因：WinCE ARM 的 pdata 是另一种打包布局（疑似 8 字节 `{start, packed}`），我没验证就下结论。
- 解法：函数边界/数量一律以 IDA `.i64` 导出为准（S0.2），不自造解析器。
- 验证：S0.2 的 `functions.csv` 与 IDA 函数表一致。


### EXP-035 逆向：「事件 id 回填点」要同时搜写端与读端，+22 字节在不同对象上语义不同
- 现象：挖 `sub_31424` 的事件 id 回填点时，若只 grep `sub_33E20(` 的写端（41 处调用点），会漏掉 +22 字节的读端语义差异——`sub_31424`/`sub_314C4` 经 `*(ctrl+12)` 解引用读精灵对象 +22（事件码），而 `sub_51D88`/`sub_52080` 经 `sub_53808(网格表, col, row)` 读地图格子 +22（可通行标志 `&3==1`），`sub_31518`/`sub_48654` 经 `ctrl+20` 读资源集 +22（u16 数值 0..999）。**同一个 +22 偏移在不同对象上语义完全不同**，grep 偏移号必须带对象类型上下文。
- 根因：ARM 伪代码里 `+ 22)` / `+0x16)` 模式匹配到 100+ 处，跨对象类型混合，不区分对象就下结论会张冠李戴。
- 解法：grep `+ 22)` 后逐条看解引用路径（`*(u32*)(ctrl+12)+22` vs `*(u32*)(ctrl+20)+22` vs `sub_53808(..)+22`），按「指向哪类对象」分三组；写端 `sub_33E20/sub_33E3C/sub_2F6A8` 与读端 `sub_31424/sub_33E6C` 交叉验证值域（写 1..27，读 case 1..27 + 哨兵 27）。
- 验证：`ref/event_ids.json` 的 `event_descriptor_+22.value_domain` + `write_sites` 41 条 + `unresolved` 第 3 条（sub_51D88 路径 +22 = 地图格子标志）三处互证。


### EXP-037 逆向：`srand`/`rand` 是 coredll 导入 thunk → 游戏不自带 RNG，oracle 按 MSVC LCG 写（S3.12 实测）
- 现象：grep `ref/pseudocode/` 找 RNG，发现 `00058634_srand.c` 仅 8 字节 thunk（`void srand(unsigned int a1) { __imp_srand(a1); }`），`__imp_srand` 前缀 = coredll.dll 导入。`rand()` 同理（无独立函数体，所有调用点直接 `call __imp_rand`）。
- 根因：TreasureIsl.exe 不自带 RNG 实现，完全依赖 WinCE CRT 的 `srand`/`rand`。算法在 `coredll.dll` 内（不在 exe 里，无法从 exe 逆出）。
- 播种点（实测，grep `srand(` 全量）：
  | 函数 | 场景 | 种子表达式 | 行号 |
  |---|---|---|---|
  | `sub_3D4C8` | 卡牌场景 ctor | `wMinute + wSecond + wMilliseconds` | `0003D4C8_sub_3D4C8.c:35-36` |
  | `sub_4E314` | （待 S3.10 存读档定位） | 同上 | `0004E314_sub_4E314.c:12-13` |
  | `sub_4E568` | （待 S3.10 存读档定位） | 同上 | `0004E568_sub_4E568.c:12-13` |
  - **种子 = `wMinute + wSecond + wMilliseconds`**（不是 wHour/wDay）→ 毫秒级时间播种，秒级重复。固定毫秒 → 固定种子 → 固定 rand() 序列，**确定性可复现**。
  - ⚠️ `sub_42AE8` 也调 `GetLocalTime` 但**不是播种**：它是存档时间戳逻辑（`sub_42DB0` 算时间戳，前 8 字节存「起始/当前」时间戳，`v4 <= Buffer + 2592000` 判 30 天过期）。
- 算法（推测，标 `推测`）：MSVC WinCE ARM 的 `rand()` = 标准 LCG `seed = seed * 214013 + 2531011; return (seed >> 16) & 0x7fff`（与桌面 MSVC 一致，见 MSVC CRT 文档）。**无法从 exe 实测验证**（coredll 不在工程内），只能等 S3.12 oracle 实现后用差分测试反证：若 oracle LCG 序列与游戏行为对不上，则 coredll 用了不同算法，需找 WinCE coredll.dll 逆向。
- 解法（S3.12 oracle 实现路径；**`oracle/rng.rs` 已随 S8.7 删除**，`rules/rng.rs` 改用 `rand_pcg` PCG64，以下为历史参照）：
  1. `ti-core/src/oracle/rng.rs`：`struct RngState { seed: u32 }`；`srand(seed)`、`rand()` = `seed = seed.wrapping_mul(214013).wrapping_add(2531011); ((seed >> 16) & 0x7fff) as i32`；
  2. 差分用例：固定种子（如 `srand(123)`）→ 取 1000 次 rand() → 与 oracle 序列逐位比对；若游戏侧能复现同种子序列（需 headless 跑游戏 rand 调用点），则算法确认；
  3. **播种点不进 oracle**（毫秒时间种子不可复现），oracle 只负责「给定种子 → 确定序列」；正式实现 `rules::rng` 同理，播种由调用层（场景 ctor）注入。
- 教训：**thunk 函数（`__imp_*` 前缀）= 导入函数，算法在外部 DLL**。逆向时遇到 thunk 别浪费时间挖 exe 内实现，直接查平台 CRT 文档；oracle 按文档写 + 差分测试反证。规约补进 `docs/style.md` §5.11。

### EXP-039 逆向/工具：REA MCP 在 arm WinCE PE 上 search_strings 一次拿全资源/场景清单，比 grep 1827 文件高效（S3.2 实测）
- 现象：S3.2 资源/渲染核心需要「场景/容器/精灵/存档/事件」全清单。此前用子代理 grep ref/pseudocode 易漏且慢。
- 根因：exe 的字符串段（.rdata）集中了所有资源名/场景名/按钮 PCX/存档/事件格式串，REA 的 search_strings（基于 IDA 字符串缓存）一次 dump 全部 870 条，含 data\*.map、button_*.pcx、event%d、gl*.sav、zlib/GX API 错误串。
- 解法：open_binary(arm PE) → search_strings（literal/regex，带 cursor 分页拿全）→ 直接得到完整场景清单（index/intro/selectrole/selectmap/main/vs/Trade/Build/Castle/Option/saveload/information/help/msg*/card 等 .Map）与资源容器（UI.pak/glpics.pak/cards.lpk/goods.lpk/music.lpk）+ 渲染 API（GXBeginDraw/GXEndDraw/GXOpenDisplay 等 9 个）+ BOSS 档（50000/100000/150000/200000/300000）。
- 验证：evidence/S3.2.log 收录 REA search_strings 全清单；S3.8 battle oracle 的 boss_hp tier 0..=4 → 5 档数值即取自此清单 0x6b436 等字符串。
- 教训：**逆向 S3 模块开工前先用 REA search_strings 全量 dump 字符串段**（而非逐文件 grep），一次性拿到资源/场景/事件/数值清单作 spec 骨架；EXP-036 记的 -32602 坑在 6.0.0 部分工具仍存，但 open_binary + search_strings/search_procedures/xrefs 实测可用（本会话），撞 -32602 就降级 ref/pseudocode grep。


