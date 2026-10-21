# 经验库索引（Experience Index）

> 注：本文的「基线」记述属**历史**：像素基线机制已于 **S18.7b 退役**（三套 golden PNG + 三个 diff 测试均已删除）；
> 现行机制 = 结构不变式断言 + CPU↔Bevy 内存对拍。

**定位**：开发过程中踩过的坑与验证过的解法，唯一目的是让后续会话**不重复踩坑**。
本文件只放**规则 + 索引**；正文按主题分卷，见下方「分卷」。

## 维护规则（强制）

1. **触发即写**：同一问题第二次遇到还没写 = 违规；首次踩到新坑（工具/格式/平台/构建）必须当场新增条目。
2. **一条一坑**：`EXP-NNN` **全局递增、绝不重号**，写清「现象 → 根因 → 解法 → 验证」，不写空话。
3. **被推翻要改**：后续证据推翻某条结论 → 在原条目**头部加 `status:` 标记**（见下表 status 约定），正文保留「已作废/已解决」提示块，**不删条目**（保留错误路径本身就是信息）。
4. **落到正确的卷**：新增条目写入其主题分卷（格式/逆向/工具/构建/流程/代码），并在本表登记一行。
5. **关联**：任务 evidence 中引用条目 ID（如 `见 EXP-003`），台账 evidence 与本文件互相可追。
6. **与事实库的分工**：**事实库放"结论"**（`legacy/docs/facts/`，必须带证据），**本库放"过程与教训"**；结论必须先进事实库，再在本库记怎么拿到的。

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
| [`reverse.md`](reverse.md) | **历史参照卷** 逆向方法论：exe 内部结构、语义交叉验证、结论推翻流程（非正向开发依据） | 4 |
  | [`tools.md`](tools.md) | 本机工具链与 MCP：IDA / Python / REA / clippy | 13 |
  | [`build.md`](build.md) | 构建与三端产物：工具链选型、wasm 壳、浏览器自动化 | 10 |
  | [`process.md`](process.md) | 流程与协作：验收纪律、并行会话、文档治理、oracle 规约、三方对账（EXP-058）、HD/键碰撞（EXP-060）、台账先行（EXP-063）、跨会话上下文注入（EXP-064）、台账回退防护（EXP-065）、开轮必读脚本化（EXP-070）、视觉回归基准图（EXP-072）、并行半成品归因（EXP-078）、机制≠规则（EXP-088）、固定种子 vs 新种子（EXP-089）、提交乱码文件恢复（EXP-092）、feature 门控 bin 门禁盲区（EXP-099）、required-features 双命中盲区（EXP-101）、交付壳换引擎的未覆盖产物盲区（EXP-107） | 31 |
  | [`code.md`](code.md) | 代码级教训：文档与实现不一致、类型转换陷阱、UTF-8 编辑陷阱、门禁扫描边界、clippy pedantic、固化脚本契约、const 表 i18n、oracle 边界脚本、界错乱双 P0（EXP-059）、数值硬编码漏网（EXP-066）、触摸/渲染正逆成对（EXP-071）、渲染管线契约+越界求交（EXP-076）、忠实接线 Bank 注资（EXP-087）、快照基线编解码对称（EXP-093） | 33 |

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
| EXP-056 | tools | | 多帧精灵 HD PNG 分帧命名（NN_hd_fK.png），Rust 侧须按帧列表注入（S7.5 实测；条目实体在 tools.md） |
| EXP-057 | code | | 门禁脚本判 oracle 须区分「类型重导出=允许」vs「oracle_* 函数调用=禁」（check_oracle_boundary，S8.8） |
| EXP-058 | process | | 子代理报「已 verified」必须 Test-Path + 实跑复核；台账 verified ↔ evidence ↔ 代码符号三方对账（S9.5 commands.rs 虚报教训，`check_ledger_reality` 第 14 项） |
| EXP-059 | process | | 地形 camera 钳位负值 + 触摸/渲染坐标系脱节 = 主地图界错乱双 P0（`terrain_camera` 去 `max(0)` 钳位，`grid_cell_at` 走等距投影逆变换，S10.1） |
| EXP-060 | process | | HD 资产不递归子目录 + 场景 key 碰撞 = HD 全失效 + 切场景残留（`read_dir` 非递归陷阱，注入数须断言=目录全量，S10.2） |
| EXP-061 | process | | 行内全路径引用（`ti_core::data::balance()`）vs use 声明——子代理风格漂移，改动前读既有 use 块，行内全路径=0（S10.3 教训） |
| EXP-063 | process | | doccheck 任务 ID 检查是散文全库扫描：新阶段任务先登记 `plan/ledger.yaml`（pending 即可）再写文档引用；散文范围写法「某阶段 x」会被当 token 抓，写「S10 系列」或逐个编号（S11 登记 S12 WBS 踩坑） |
| EXP-064 | process | | 子代理只拿到任务书 = 跨会话上下文丢失：会话起点 prompt 必注入仓库绝对路径 + crate 前缀 `ti-`（非 `ty-`）+ 关键符号路径 + 契约锚点 + 当前任务与已 verified 清单，回执逐条对齐否则返工（模块并行 loop 纪律） |
| EXP-065 | process | | 台账被文件级回退（`git checkout -- <file>`/restore 不进 reflog）会静默丢任务条目：23 条丢失 → doccheck 第 9 项 115 error；兜底 = gate 第 14 项「孤儿 evidence 检测」（`evidence/` 里任务 ID 命名的文件其 ID 必须在台账）+ 长阶段收尾即 commit |
| EXP-066 | code | | 表现层 `start_battle` 写死 `Unit{hp:1000,atk:150}` 与 `balance.combat.player{hp:2000,atk:60}` 漂移；属 P1-P5 白名单外，S10.5 审计留 unknown 不强制修，仅登记 + 写 evidence（check_hardcode/oracle_boundary 都抓不到，需双 agent 反向审计补洞） |
| EXP-067 | code | | 跨层几何公式（rules 逻辑坐标 ↔ 渲染投影）须正逆成对测试：逆变换(正变换(p))==p，单层公式自洽不算数（S10.5 审计发现，主地图触摸命中教训 EXP-059 的泛化） |
| EXP-068 | process | | 子代理开轮不读最新规范/EXP/契约 → 重复踩坑（ty-app 前缀、跨层几何不对齐、不知 gate 第 16 项查什么）：开轮必读清单 6 条 + 规范回执固化进 06_flywheel（飞轮单一事实源）+ 05 §4.5 + 03 §7 + doccheck2 第 8 项 `check_flywheel_readback` |
| EXP-070 | process | | 开轮必读靠 LLM 手动 Read 6 份文档 → 工具调用爆炸 + 漏读（P6 子代理漏读在途白名单）：`scripts/flywheel_readme.ps1` 一键生成完整必读（环境事实 + 6 文档 + 白名单 + 本域 EXP + gate 项 + doccheck2 断言），gate 第 17 项 `flywheel-readme -Check` 验证完整性，doccheck2 第 10 项 `check_flywheel_script` 固化 |
| EXP-071 | code | | 列表场景触摸命中带必须与渲染 `item_rect` 公式正逆成对：`touch.rs` 删旧常量/旧 API，新增 `scene_list_bands`/`slot_bands` 从 `ListLayout` 推导，三场景 `on_touch` 改读 `scenes.json`，doccheck2 第 11 项 `check_touch_render_pair` 固化（S11 审计 N3） |
| EXP-072 | process | | 表现层改触摸/渲染坐标后 `cargo test` 全绿却界面错乱（漏到用户截图才发现）：P4-1 只锁 PCX 解码基准，不锁「场景渲染最终像素」→ `visual_regression` 3 场景逐像素 diff + gate 第 18 项。正文见 `process.md` |
| EXP-073 | code | | 触摸/渲染 band 数必须 == 渲染行数，禁止硬编码：`select_role.rs` 硬编码 9 条带实渲 5 行，多出的 4 条静默选第 0 角色（EXP-059/071 第 3 次变体）→ band 数走场景常量 `ROLE_COUNT` + doccheck2 第 12 项。正文见 `code.md` |
| EXP-074 | process | | 数据表迁移必须对拍旧验收视觉：凭空重填 + `-item_h/2` 居中性误解 + 缺 key 静默置 0 + 视觉基准自生成（循环论证）→ `highlight_offset`/`gold_x`/`extra_scalar` + 基准=回归后旧视觉帧（EXP-059/067/071 第 4 次变体）。正文见 `code.md` |
| EXP-075 | process | | HD 精灵洋红框 = **两条路径各一根因**（RGB565 路漏 `idx==0` 跳过 + HD 路软 alpha/漏 blit），S13.5 只修 HD 路，而 native 默认走 RGB565 故"没解决"；**native 默认质量模式才是实机路径**。正文见 `process.md` |
| EXP-076 | code | | 「贴图覆盖菜单/错位」= 越界裁剪策略错 + 渲染管线无契约：Rust `ox.min(FB_W-1)` 把越界精灵拉回右缘，原版 `sub_5EAA8` 源窗求交为空即**整条不画**；z-order 本身无错 → `clip_rect` 纯函数越界早退 + 契约文档 + 快照门禁。正文见 `code.md` |
| EXP-077 | process | | 无原版渲染 ground truth 时禁止向用户索要截图（ARM WinCE exe 本机跑不起来；替代证据链=用户指认偏移量/像素对比工具/QEMU 系统模式需 WinCE 镜像；补网格层 a1[247]） |
| EXP-078 | process | | 并行半成品致 gate FAIL 时先归因再决定能否转 verified：git stash 基线对照法归因（stash 前后 FAIL 项集合 diff），本轮引入项修干净、并行半成品项不替别人修（evidence 写明归因），台账维持 in_progress，commit 只暂存本轮改的文件（显式列文件不用 -A） |
| EXP-079 | code | | UI 色彩统一 + 场景过渡动画 + 菜单交互反馈三件套：色彩常量单一文件源（chrome.rs）、TransitionState 淡入淡出、menu_item pressed 凹陷 + 选中正弦微光；帧动画用 AtomicU32（static mut 在 Rust 2024 编译错） |
| EXP-080 | code | | 帧级动画破坏像素快照确定性（render_snapshot_forward diff 1758）：快照入口 render_to_framebuf 归零 transition + reset_animation_state，实机路径保留动画（分治）；基线由修复后代码重生成（删三套 baseline 目录首跑落盘） |
| EXP-081 | code | | 真交叉淡化必须守卫「上一帧不存在」+ 防重入：首帧前 rendered_once=false 直接跳过过渡（否则 prev 是空白缓冲 → 全帧黑，render_parity 墨点率 0）；过渡中再 enter 沿用原 prev 只重置 t，不重抓半混合帧 |
| EXP-082 | code | | 「≥4 行序列在 ≥3 处重复」= 提取信号：chrome::scene_shell/scene_hint 收敛 6 场景样板，顺带把 hint_row 的 i18n::sget 双查（为算宽度）收成单查，省每帧查表 + String 分配 |
| EXP-083 | code | | EXP-051 复发：删 3 处未用 `use` 时用 PowerShell `Get-Content\|Set-Content` 批量改含中文注释的 .rs → 3 文件 CJK 乱码 + 注释与代码行粘连（fmt 报 unexpected closing delimiter）。含非 ASCII 的文件一律用编辑工具，改完立刻 fmt --check |
| EXP-084 | code | | TTF 软件光栅化（ab_glyph）：`TtfFont::from_bytes` 加载 + `rasterize` 逐字形 1px 描边 + `advance` 字宽；`MenuShell.ttf_font` 可选字段（None 时回退 5×7 位图）；`find_ttf_font` 搜 `assets/fonts/` 再 fallback 系统字体；LXGW WenKai GB Lite（SIL OFL）放 `assets/fonts/lxgw-wenkai-reg.ttf` |
| EXP-085 | code | | 数值平滑动画：显示值与实际值分离（`displayed_player_hp` / `displayed_boss_hp`），`approach()` 25% 步进 + 最小 1 确定性插值；`title_bar(animate)` logo 用 `glow_factor()` 闪烁；trait 扩展同步 mock（`render_intent.rs` VsHost 补 `pixel_at` / `displayed_*`）；`i32 as f32` 用 `#[allow]`，`usize as i32` 用 `usize::try_from` |
| EXP-086 | code | | EXP-051/083 复发（第 3 次）：PowerShell `Get-Content\|Set-Content` 改含中文 .rs → 整文件双重编码乱码（GBK/UTF-8）；新变体：捕获 native `cargo` 中文 stdout 前未设 `[Console]::OutputEncoding=UTF8` 也乱码 log；解法=源改一律 edit/write 工具 + 设 OutputEncoding + 用 Read 核对 |
| EXP-087 | code | | 忠实接线 `BuildManager` 须用宿主当前金注资：`Bank::new(0)` 让所有建造走「金不足被拒=状态不变」静默路径 → 建造加成恒 0（hp/atk +0,+0），拒绝时不报错难察觉；须核对加成真进战斗（seed=7 实测 hp=2040=2000+50−10） |
| EXP-088 | process | | 机制≠规则：`unlocked_maps` 有存档字段/UI 置灰但无「击败 BOSS→解锁下一图」推进规则（开档 `game_app.rs:535` 全解锁、全库无写入增长端）；定稿跨图前先查规则是否存在 → `progression.md` 定稿 + `rules/progression.rs` + `cross_map_demo`，app 层接线归渲染线 |
| EXP-089 | process | | 「跑多少次都一样」是固定种子回归测试的**正确**行为；要展示「每次都不同」用新种子 demo（无参=SystemTime 种子、`--seed N` 复现）；「随机/非固定」用选择指纹 distinct（366/400）+ 受控变异证明；数值断言须实测（m01=10 步不是 20） |
| EXP-090 | code | | clippy pedantic 值语义状态 + `single_match_else`（EXP-053 复发）：小 Copy 状态结构用值语义 + 功能式（`on_boss_defeated(mut self)->Self`）；`Some/None` match → `if let/else`；demo 参数 >7 挂 `#[allow(clippy::too_many_arguments)]` |
| EXP-091 | tools | | 文档文件含非法 UTF-8 坏字节时 doccheck `read_to_string` 静默跳过整文件 → 该文件全部检查行（如台账计数漂移）永久失声：release.md 实测 292 处坏字节（三字节字符第 3 字节被替 `0x3F`），计数行长期漂移未被 gate 抓到；解法=编辑前先扫坏字节确认文件是否「对 doccheck 可见」、只动 ASCII 区用字节级替换（EXP-051/083/086 教训）、全量修复另立任务。正文见 `tools.md` |
| EXP-092 | process | | 已提交的乱码（UTF-8→GBK 误读）文件恢复 = 找最后干净 commit + 拼接干净新段，**绝不**逆向「解码回去」：`git show <sha>:<path>` 逐 commit 找最后干净 blob（标题字节判干净）+ 工作树截取乱码后追加的干净新段 + `WriteAllBytes` 写回（勿 `Set-Content`）。正文见 `process.md` |
| EXP-093 | code | | 快照基线编解码不对称致「像素漂移」假阳（EXP-080 第 2 类根因）：渲染本身跨进程确定（3 次重生成哈希一致），漂移来自 PNG encode（`Rgb` 3 通道）与 decode（`EXPAND`+手工 `px*4` buffer+`next_frame`）通道/buffer 错位 → 解码行内右移 1 字节 → 87% 假阳 diff。解法：编解码对称无损（存 `Rgba`，按 `OutputInfo` 通道数归一）+ 上线前小 fixture 往返探针先证 0 mismatch。正文见 `code.md` |
| EXP-094 | code | | 渲染框架提取时像素面必须能被测试宿主拦截：框架认具体帧缓冲型→两型同名不同型编不过；绕过宿主 `RenderHost::fill_rect` 直接画具体缓冲→测试宿主记录不到（编译绿≠行为等价，`key_rects_have_pixels` 血条宽 0）。解法：`PixelSink` trait 解帧缓冲 + `HostCanvas` 桥把像素转回宿主 trait（只读几何才走 `shell()`）+ 闭包式 `with_canvas` 避 E0515。正文见 `code.md` |
| EXP-095 | code | | 居中 `cx - w/2` 与口径 `(W-w)/2` 在**奇数宽**时差 1px：hint 条宽断言数出 0 并连带打挂 3 套快照（看着像大改版）。解法：实现统一到「先减后除」`(2*screen_cx - w)/2`；奇数宽来自 TTF 文本 advance 非常量，别改测试口径也别重生基线。正文见 `code.md` |
| EXP-099 | process | | `required-features` 门控的 bin（element-dump / sprite-step / render-png / treasure-native / treasure-headless）不被 gate 第 1~3 项编译/lint ⇒ 官方排障工具静默失效（element-dump 长期编不过而 gate 全绿）。解法 = 新增 gate 第 4b 项 `clippy --all-targets --features ti-app/native-shell,ti-app/headless`，门禁计数 21→22。正文见 `process.md` |
| EXP-100 | process | | 脚本化批量改 import 的正则陷阱：捕获组 `+` 漏单段模块、前缀带尾 `::` + `split('::')[-1]` 得空串把限定名削成裸类型名（43 处全编译失败）；正解 `(?:…::)*`、插入点锚文件顶部 import 区、嵌套 mod 补 `use super::`、逐文件 `cargo check` + 全量 clippy 双跑当裁判。正文见 `process.md` |
| EXP-101 | process | | `required-features` bin 只在「目标+feature 双命中」时被编译：裸 wasm target build 跳过 treasure-wasm（需 `wasm-shell`），gate 的 clippy 组合又只覆盖 native 侧 feature → 借用错误 + wasm 独有 clippy 3 处旧伤漏网到 `build_wasm.ps1` 才爆；gate 第 4 项 wasm 分支并入 `clippy --features wasm-shell --all-targets` 复合判定。正文见 `process.md` |
| EXP-107 | process | | 交付壳换引擎时「未被任何门禁行编译的产物」会静默烂掉：android 壳因门禁 android 行不开 `android-shell` feature 被 cargo 跳过（实际早已编译不过），Bevy 交付壳（独立 workspace）的 wasm 分支从未被 clippy 覆盖（首次纳入即暴露 5 处死代码/告警）；解法 = 门禁 wasm 项直接指向交付产物（build + `clippy -D warnings`）+ 死代码当场清理 + 台账符号期望随退役迁移。正文见 `process.md` |
| EXP-108 | process | | 门禁检查的「行识别条件」写错 → 恒不触发 = 一个不存在的门禁：doccheck2 分卷条数对账用 `.md](` 判行，而首列实为 `[`code.md`](code.md)`（中间有反引号）→ 该检查自落地起一次都没真正执行 → 分卷表条数漂移（code +5 / tools +1）与索引缺 6 行长期无人发现；解法 = 判据改 `](` + 解析失败即报错 + 补齐漂移 + **负测纪律**（新检查上线前故意破坏一次看是否报红）。正文见 `process.md` |
| EXP-109 | build | | PowerShell 5.1 对「无 BOM 的 UTF-8 .ps1」按 ANSI(GBK) 解析 → 中文提示里的字节被误判成引号 → 整文件 ParserError；仓库里唯二的 `playbook.ps1`（AGENTS 的一键命令入口）+ `smoke.ps1` 缺 BOM → **入口从未跑起来**而门禁不跑这两个脚本（无人发现）；解法 = 补 UTF-8 BOM + CRLF、改完先 ParseFile 语法自检再真跑一次。正文见 `build.md` |
| EXP-049 | formats | | 容器条目 key→业务名反查必须走三级权威源（mapinfo.ini > exe UTF-16 资源名 > EXTRAS 多键证实名），禁止纯哈希命名（S6.5 实测）。正文见 `formats.md` |
| EXP-050 | tools | | CJK 路径下 cargo `env!("CARGO_MANIFEST_DIR")` 拼出的路径丢中文 → `fs::read` 报 os error 2；改用 `PathBuf` 逐段拼 + 编译期 `include_str!` 定位仓库根。正文见 `tools.md` |
| EXP-062 | build | | macroquad `flip_y` 必须端侧固定（native 不翻 / wasm 翻），禁止因「画面反了」反复翻转（S5/S6 后续实测）。正文见 `build.md` |
| EXP-102 | code | | Bevy 手搓插件表（`MinimalPlugins` + 手动 `add_plugins`）→ 实体在但一个像素没画（黑屏也算过验收）；必须用官方 `DefaultPlugins`。正文见 `code.md` |
| EXP-103 | code | | Bevy z 层：可见性要负值、排序是升序（背景最负才在最底）；正数 z 在相机背后。正文见 `code.md` |
| EXP-104 | code | | 离屏截图的采样点标签诱人：把四角采样当成「整屏铺满」，连改 3 处本来正确的代码。正文见 `code.md` |
| EXP-105 | code | | 原语表 z 层卓埋 / 行高变了栏位没算 / JSON `text_key` 先验存在。正文见 `code.md` |
| EXP-106 | code | | 原语批次的调用位置 = 层序；背景层必须在运行时内容**之前**画。正文见 `code.md` |


## 条目格式（写新条时照抄）

```markdown
### EXP-NNN 标签（格式/逆向/工具/构建/流程/代码）
- 现象：
- 根因：
- 解法：
- 验证：
```
