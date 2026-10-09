# 经验库索引（Experience Index）

**定位**：开发过程中踩过的坑与验证过的解法，唯一目的是让后续会话**不重复踩坑**。
本文件只放**规则 + 索引**；正文按主题分卷，见下方「分卷」。

## 维护规则（强制）

1. **触发即写**：同一问题第二次遇到还没写 = 违规；首次踩到新坑（工具/格式/平台/构建）必须当场新增条目。
2. **一条一坑**：`EXP-NNN` **全局递增、绝不重号**，写清「现象 → 根因 → 解法 → 验证」，不写空话。
3. **被推翻要改**：后续证据推翻某条结论 → 在原条目**头部加 `status:` 标记**（见下表 status 约定），正文保留「已作废/已解决」提示块，**不删条目**（保留错误路径本身就是信息）。
4. **落到正确的卷**：新增条目写入其主题分卷（格式/逆向/工具/构建/流程/代码），并在本表登记一行。
5. **关联**：任务 evidence 中引用条目 ID（如 `见 EXP-003`），台账 evidence 与本文件互相可追。
6. **与事实库的分工**：**事实库放"结论"**（`docs/facts/`，必须带证据），**本库放"过程与教训"**；结论必须先进事实库，再在本库记怎么拿到的。

### status 标记约定

| status | 含义 | 写法 |
|---|---|---|
| `active` | 当前有效（默认，可不写） | 省略 |
| `resolved by EXP-NNN` | 坑已解决，保留作经验 | `### EXP-NNN 标题（**status: resolved by EXP-NNN**）` |
| `superseded by EXP-NNN` | 条文已被作废，**不要照它行动** | `### EXP-NNN 标题（**status: superseded by EXP-NNN**）` |

> 门禁第 9 项 `doccheck` 会校验：`EXP-NNN` 全库唯一、status 标记格式合法、被标记的条目必须带提示块。

## 分卷

| 卷 | 内容 | 条数 |
|---|---|---|
| [`formats.md`](formats.md) | 二进制/资源格式：容器、PCX、`.Map`、字体、音频、INI | 14 |
| [`reverse.md`](reverse.md) | 逆向方法论：exe 内部结构、语义交叉验证、结论推翻流程 | 4 |
| [`tools.md`](tools.md) | 本机工具链与 MCP：IDA / Python / REA / clippy | 9 |
| [`build.md`](build.md) | 构建与三端产物：工具链选型、wasm 壳、浏览器自动化 | 6 |
| [`process.md`](process.md) | 流程与协作：验收纪律、并行会话、文档治理、oracle 规约 | 8 |
| [`code.md`](code.md) | 代码级教训：文档与实现不一致、类型转换陷阱 | 4 |

## 条目索引

| ID | 卷 | status | 标题 |
|---|---|---|---|
| EXP-001 | formats | | exe 里的资源名是 UTF-16 宽字符 |
| EXP-002 | formats | | pak 的 key 不是常见哈希，别再穷举 |
| EXP-003 | formats | | `LucencyPackage` 头部字段不是 4 字节对齐 |
| EXP-004 | formats | | 扫 `\x78\xda` 找 zlib 流会误报 |
| EXP-005 | reverse | | `.pdata` 别按 x86 布局硬解 |
| EXP-006 | tools | | 本机 Python 环境缺包、`pip` 不在 PATH |
| EXP-007 | process | | 低价值目标不要反复投入（`.rsrc` 解析两次失败） |
| EXP-008 | build | | 本机无 Docker → 放弃 macroquad docker APK 路线 |
| EXP-009 | process | | 二进制结构结论必须先 hexdump 再下笔 |
| EXP-010 | tools | | PowerShell 下 Python 报错噪声大 |
| EXP-011 | build | | 本机无 `rg`、`bash` 是 WSL 版，门禁必须用 PowerShell |
| EXP-012 | process | | 文档与环境冲突时，先改文档再开工 |
| EXP-013 | process | | 编译工具链后置（用户决策） |
| EXP-014 | tools | **resolved by EXP-031** | IDA 7.5 portable 的 Hex-Rays ARM 反编译器不可用 |
| EXP-015 | code | | clippy pedantic 反复触发的模式（workspace `-D warnings`） |
| EXP-016 | tools | **superseded by EXP-031** | IDA 7.5 portable headless 执行要点 |
| EXP-017 | formats | | `LucencyPackage` 三处事实推翻 + key 是「两类」 |
| EXP-018 | formats | | 重打包必须区分「可重导」与「原样复制」 |
| EXP-019 | formats | | 游戏自产 PCX 的头字段是「垃圾值」，靠强约束反解 |
| EXP-020 | formats | | `.Map` 宽高不能写成格式约束 |
| EXP-021 | formats | | 整数归一会静默变 0；WAV 实为 8bit |
| EXP-022 | process | | 三端差异只放后端，语义用「同一套断言」锁 |
| EXP-023 | formats | | INI 是 UTF-16LE 且必须保留原文才能往返 |
| EXP-024 | formats | | `GoldLand.Fnt` 位图的「12 像素」来自 bit 使用统计 |
| EXP-025 | formats | | ID 型 key 的绑定藏在容器自身的 `mapinfo.ini` |
| EXP-026 | formats | | 「头部字宽表」误读与 MSB-first 位序的几何判定 |
| EXP-027 | formats | | 地形「区域表」是整条 cell 流，`regions_off` 是尺寸不是偏移 |
| EXP-028 | build | | macroquad wasm 是「裸 wasm + gl.js」，不是 wasm-bindgen |
| EXP-029 | build | | headless Edge `--virtual-time-budget` 不驱动 rAF |
| EXP-030 | code | | `present_framebuffer` 文档说用 Viewport，实现却没接 |
| EXP-031 | tools | | IDA 9.4 正式版 Hex-Rays ARM 正常 → 解除 EXP-014 / 作废 EXP-016 |
| EXP-032 | tools | | 通用解析器别夹带特定布局校验 |
| EXP-033 | tools | | REA 6.0.0 ↔ OpenCode 互操作缺陷（两坑） |
| EXP-034 | process | | 并行会话工作树里先 git status 再动手 |
| EXP-035 | reverse | | 「事件 id 回填点」要同时搜写端与读端 |
| EXP-036 | tools | | REA supervisor profile 下「几乎所有工具」都撞 -32602 |
| EXP-037 | reverse | | `srand`/`rand` 是 coredll thunk → oracle 按 MSVC LCG 写 |
| EXP-038 | process | | 子代理声称 verified 后必须主线程独立复核 |
| EXP-039 | reverse | | `search_strings` 一次拿全资源/场景清单 |
| EXP-040 | process | | S3 模块 oracle 固定种子确定性公式约定 |
| EXP-041 | tools | | REA 定位维度错误 + 文档矛盾未读全 |
| EXP-042 | tools | | S3.12 会话 REA 冒烟再次失败（F-319 波动性第三例） |
| EXP-043 | code | | 双实现 + 差分对拍是冗余模式，公式/类型应单一源 |
| EXP-044 | code | | macroquad Texture2D 部分上传需 update_part，行级脏标记与之精确对应 |
| EXP-045 | build | | native macroquad 窗口下 `FramePresenter` 呈现失效 → 黑屏；且纹理 V 语义相反 → 画面翻转 |
| EXP-046 | build | | 大依赖（opus-pure）feature 门控 + 单一路径：默认 wasm/native 不含 OGG 编码器 |

## 条目格式（写新条时照抄）

```markdown
### EXP-NNN 标签（格式/逆向/工具/构建/流程/代码）
- 现象：
- 根因：
- 解法：
- 验证：
```