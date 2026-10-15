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

> 约定（当前门禁未自动校验提示块）：`doccheck`（gate 第 9 项，独立命令）校验 `EXP-NNN` 全库唯一 + 跨文档引用有效；status 标记格式与「被标记条目必须带提示块」为文档约定，当前门禁不自动校验。

## 分卷

| 卷 | 内容 | 条数 |
|---|---|---|
| [`formats.md`](formats.md) | 二进制/资源格式：容器、PCX、`.Map`、字体、音频、INI | 15 |
| [`reverse.md`](reverse.md) | 逆向方法论：exe 内部结构、语义交叉验证、结论推翻流程 | 4 |
| [`tools.md`](tools.md) | 本机工具链与 MCP：IDA / Python / REA / clippy | 9 |
| [`build.md`](build.md) | 构建与三端产物：工具链选型、wasm 壳、浏览器自动化 | 10 |
  | [`process.md`](process.md) | 流程与协作：验收纪律、并行会话、文档治理、oracle 规约、三方对账（EXP-058）、HD/键碰撞（EXP-060）、台账先行（EXP-063）、跨会话上下文注入（EXP-064）、台账回退防护（EXP-065）、开轮必读脚本化（EXP-070）、视觉回归基准图（EXP-072）、并行半成品归因（EXP-078） | 16 |
| [`code.md`](code.md) | 代码级教训：文档与实现不一致、类型转换陷阱、UTF-8 编辑陷阱、门禁扫描边界、clippy pedantic、固化脚本契约、const 表 i18n、oracle 边界脚本、界错乱双 P0（EXP-059）、数值硬编码漏网（EXP-066）、触摸/渲染正逆成对（EXP-071）、渲染管线契约+越界求交（EXP-076） | 17 |

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
| EXP-045（续） | build | **status: resolved** | native 端 S4.3 脏行带优化补做（`UploadMode::FullRebuild`，见 build.md） |
| EXP-046 | build | | 大依赖（opus-pure）feature 门控 + 单一路径：默认 wasm/native 不含 OGG 编码器 |
| EXP-047 | build | | 手工 `Mesh`+`draw_mesh` 弱引用纹理在 native macroquad 不渲染 → 黑屏；EXP-045 验证只查 stats 漏了像素输出 |
| EXP-048 | process | | 对抗审计发现的「可机械判定」检查项必须固化进 doccheck/gate，否则下一轮必然复发 |
 | EXP-062 | build | | macroquad `flip_y` 必须端侧固定（native 不翻 / wasm 翻），禁止因「画面反了」反复改（S5/S6 实测，Q053 重号修正：原误标 EXP-058，与 process 卷「子代理虚报」条撞号，另立 EXP-062） |
 | EXP-049 | formats | | 容器条目 key 的业务名由三级反查权威源提供，禁止裸哈希命名（S6.5 实测） |
 | EXP-050 | tools | | CJK 路径下 cargo `env!("CARGO_MANIFEST_DIR")` 损坏，bin 定位 repo 根优先 `current_dir()` |
| EXP-051 | code | | PowerShell `Set-Content` 重写会破坏 UTF-8 源码（CJK 乱码+换行合并），改源码禁用 PS 文本替换 |
| EXP-052 | code | | 门禁第 5/6 项扫全量 `.rs`，`src/` 内联 `#[cfg(test)]` 的 `expect`/`panic!` 同样被拦 → 单测放 `tests/` |
| EXP-053 | code | | clippy pedantic 拦 `f64 as u32` 与 `Copy` 按引用传；`#[allow]` 须挂 item 非 let |
| EXP-054 | code | | 固化脚本三契约（幂等/--report/非0退出码），且必须被门禁/循环消费才不算摆设 |
| EXP-055 | code | | `pub const` 表禁调非 const fn（E0015）；i18n 用「常量存 key + 运行时 sget 解析」 |
| EXP-056 | resources | | 多帧精灵 HD PNG 分帧命名（NN_hd_fK.png），Rust 侧须按帧列表注入（S7.5 实测） |
| EXP-057 | code | | 门禁脚本判 oracle 须区分「类型重导出=允许」vs「oracle_* 函数调用=禁」（check_oracle_boundary，S8.8） |
| EXP-058 | process | | 子代理报「已 verified」必须 Test-Path + 实跑复核；台账 verified ↔ evidence ↔ 代码符号三方对账（S9.5 commands.rs 虚报教训，`check_ledger_reality` 第 14 项） |
| EXP-059 | code | | 地形 camera 钳位负值 + 触摸/渲染坐标系脱节 = 主地图界错乱双 P0（`terrain_camera` 去 `max(0)` 钳位，`grid_cell_at` 走等距投影逆变换，S10.1） |
| EXP-060 | process | | HD 资产不递归子目录 + 场景 key 碰撞 = HD 全失效 + 切场景残留（`read_dir` 非递归陷阱，注入数须断言=目录全量，S10.2） |
| EXP-061 | code | | 行内全路径引用（`ti_core::data::balance()`）vs use 声明——子代理风格漂移，改动前读既有 use 块，行内全路径=0（S10.3 教训） |
| EXP-063 | process | | doccheck 任务 ID 检查是散文全库扫描：新阶段任务先登记 `plan/ledger.yaml`（pending 即可）再写文档引用；散文范围写法「某阶段 x」会被当 token 抓，写「S10 系列」或逐个编号（S11 登记 S12 WBS 踩坑） |
| EXP-064 | process | | 子代理只拿到任务书 = 跨会话上下文丢失：会话起点 prompt 必注入仓库绝对路径 + crate 前缀 `ti-`（非 `ty-`）+ 关键符号路径 + 契约锚点 + 当前任务与已 verified 清单，回执逐条对齐否则返工（模块并行 loop 纪律） |
| EXP-065 | process | | 台账被文件级回退（`git checkout -- <file>`/restore 不进 reflog）会静默丢任务条目：23 条丢失 → doccheck 第 9 项 115 error；兜底 = gate 第 14 项「孤儿 evidence 检测」（`evidence/` 里任务 ID 命名的文件其 ID 必须在台账）+ 长阶段收尾即 commit |
| EXP-066 | code | | 表现层 `start_battle` 写死 `Unit{hp:1000,atk:150}` 与 `balance.combat.player{hp:2000,atk:60}` 漂移；属 P1-P5 白名单外，S10.5 审计留 unknown 不强制修，仅登记 + 写 evidence（check_hardcode/oracle_boundary 都抓不到，需双 agent 反向审计补洞） |
| EXP-067 | code | | 跨层几何公式（rules 逻辑坐标 ↔ 渲染投影）须正逆成对测试：逆变换(正变换(p))==p，单层公式自洽不算数（S10.5 审计发现，主地图触摸命中教训 EXP-059 的泛化） |
| EXP-068 | process | | 子代理开轮不读最新规范/EXP/契约 → 重复踩坑（ty-app 前缀、跨层几何不对齐、不知 gate 第 16 项查什么）：开轮必读清单 6 条 + 规范回执固化进 06_flywheel（飞轮单一事实源）+ 05 §4.5 + 03 §7 + doccheck2 第 8 项 `check_flywheel_readback` |
| EXP-070 | process | | 开轮必读靠 LLM 手动 Read 6 份文档 → 工具调用爆炸 + 漏读（P6 子代理漏读在途白名单）：`scripts/flywheel_readme.ps1` 一键生成完整必读（环境事实 + 6 文档 + 白名单 + 本域 EXP + gate 项 + doccheck2 断言），gate 第 17 项 `flywheel-readme -Check` 验证完整性，doccheck2 第 10 项 `check_flywheel_script` 固化 |
| EXP-071 | code | | 列表场景触摸命中带必须与渲染 `item_rect` 公式正逆成对：`touch.rs` 删旧常量/旧 API，新增 `scene_list_bands`/`slot_bands` 从 `ListLayout` 推导，三场景 `on_touch` 改读 `scenes.json`，doccheck2 第 11 项 `check_touch_render_pair` 固化（S11 审计 N3） |
 | EXP-072 | process | | 表现层改触摸/渲染坐标后界面错乱但 `cargo test` 全绿漏到用户才发现：P4-1 只锁 PCX 解码基准不锁「场景渲染最终像素」，gate 无视觉几何回归。`visual_regression.rs` 3 场景 headless 逐像素 diff + `visual_baseline.ps1` 基准图 + gate 第 18 项 `visual-regression`（EXP-072） |
 | EXP-073 | code | | 触摸/渲染 band 数必须 == 渲染行数（`Character::all().len()`），禁止硬编码：`select_role.rs` `scene_list_bands(..., 9)` 硬编码 9 条带但渲染只 5 行，多出的 4 条带 `from_index(5..=8) → None → unwrap_or(0)` 静默选第 0 角色，违反 EXP-067（EXP-059/EXP-071 第 3 次变体）。band 数走场景常量 `ROLE_COUNT`，doccheck2 第 12 项 `check_scene_band_count` 固化 |
 | EXP-074 | code | | 数据表迁移（scenes.json 坐标回归旧视觉值）：P6 重填表值未对拍 ab0e5b5 旧视觉、`item_rect` 误加 `-item_h/2` 居中性（旧代码高亮是「文字基线 y 上移固定 offset」非居中）、extra 解析器把缺 key 静默置 0 丢失 `locked.text_x=130`、视觉基准自生成（循环论证）。修复：`ListLayout` 新增 `highlight_offset`/`gold_x`、`extra_scalar` 抽 `*_x`/`*_y`、`menu_shell` 标题条 40px 回归、视觉回归只比「UI 像素分布」掩背景色（HD 背景≠PCX 背景非排版漂移），基准=回归后旧视觉帧。HD 化只换贴图不动逻辑布局坐标；数据表值必须==旧验收视觉，子代理禁止凭空重填+自证基准图（EXP-059/067/071 第 4 次变体） |
  | EXP-075 | code | | HD 精灵「洋红框」根因实为**两条独立路径、两个独立根因，前次只修了一半**：① PCX 调色板 index 0 = 洋红哨兵色（ui.pak 94/120 张实测 (255,0,255)，index 0 占背景区 0%~84.7%），原版 8-bit blit 例程 sub_5D9C4 逐像素 `if (idx != 0) write` 跳过 index 0，但 Rust `blit_sprite`（RGB565，native 默认 `QualityMode::Original` 实际走的路）漏了这个跳过 → 洋红背景整片写满帧缓冲；② HD PNG 路（`QualityMode::Hd`）是另一根因：`pack_hd_assets.py` Lanczos 缩采样产生 3.8 万软 alpha + `blit_sprite_rgba` 只判 `a==0` + `render_hd` 漏调 blit。前次 S13.5 只修了②（HD 路），native 默认 ①（RGB565 路）未修 → 用户"没解决"。修复：① `blit_sprite`/`blit_sprite_rgba` PCX 回退/`terrain_atlas` 2 处加 `if idx == 0 { continue; }`（同 sub_5D9C4 口径）+ render_parity 基线同步；② 保留 pack_hd_assets.py sanitize_alpha + ALPHA_TRANSPARENT_CUTOFF + render_hd 补 blit。教训：**同一种视觉现象（洋红框）在 RGB565 路与 HD 路各有一个独立根因，修一条路必须查另一条路的同口径点；native 默认质量模式（Original）才是实机路径，headless 测试若只跑默认路径会漏 HD 路，反之亦然**。doccheck2 第 12 项 check_hd_sprite_transparency 扩为锚定两路 4 处不变量 |
  | EXP-076 | code | | 「贴图覆盖菜单/特定区域错位」= 越界裁剪策略错 + 渲染管线无契约无分层。**现象**：native 默认 RGB565 路，index 场景 f1=111（105×11 长条）有 3 条 x=185/191/207 越界（>176），Rust `blit_sprite` 的 `ox = px_x.min(FB_W-1)` 把越界精灵**拉回屏幕右缘 175 列**画 1 列宽竖条 → 落在菜单区右缘 = 用户报的"贴图覆盖菜单/特定区域错位"。**根因**：原版 `sub_5EAA8` 源窗求交为空 → `sub_5D9C4` 早退**整条不画**（高置信，`evidence/audit_zorder.md` A3），Rust 漏了这个早退。**z-order 本身无错**（子代理坐实：原版按文件顺序 blit，非 f5 分档、非分帧，`evidence/audit_f5_zorder.md` 高置信）；`words[4]=176` 是误读（实际 `MapScene.words[4]`=w[8]=31=记录数，正确）。**解法**（职责分离 + 契约固化）：① 新增 `render_pipeline.rs` `clip_rect(src_w,src_h,ox,oy)->Option<ClipRect>` 纯函数（越界早退，6 单测），`blit_sprite`/`blit_sprite_rgba` 接入（修掉越界拉回）；② `docs/process/07_render_pipeline.md` 渲染契约（路径表/z-order 文件顺序/越界求交/PCX index 0 透明位，单一事实源）；③ `tests/render_snapshot.rs` 7 场景逐像素基准 diff（gate 第 19 项，基准由修复后代码生成杜绝自证循环）；④ `scripts/check_data_source.py`（gate 第 20 项，数据源 .Map/.pak 改动→快照门禁）。**教训**：渲染链路关键行为（越界裁剪/透明位/z-order）散在多文件无契约 → 每轮瞎猜。**必须**：(a) 渲染行为提成纯函数可单测；(b) 契约文档登记进路由表，改渲染代码前必读；(c) 快照门禁锁"最终像素"，数据源改动当场拦；(d) 修 bug 前先按 native 真实链路逐层定位（哪条 quality 路 + 越界/透明语义），不靠猜。gate 20 项全绿 |
| EXP-077 | process | | 无原版渲染 ground truth 时禁止向用户索要截图（ARM WinCE exe 本机跑不起来；替代证据链=用户指认偏移量/像素对比工具/QEMU 系统模式需 WinCE 镜像；补网格层 a1[247]） |
| EXP-078 | process | | 并行半成品致 gate FAIL 时先归因再决定能否转 verified：git stash 基线对照法归因（stash 前后 FAIL 项集合 diff），本轮引入项修干净、并行半成品项不替别人修（evidence 写明归因），台账维持 in_progress，commit 只暂存本轮改的文件（显式列文件不用 -A） |

## 条目格式（写新条时照抄）

```markdown
### EXP-NNN 标签（格式/逆向/工具/构建/流程/代码）
- 现象：
- 根因：
- 解法：
- 验证：
```