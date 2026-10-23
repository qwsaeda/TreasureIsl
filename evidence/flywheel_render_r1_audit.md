# 飞轮 R1 · 审计结论 —— 主题「渲染数据/图片分离」（`evidence/flywheel_render_r1_findings.md` 97 条逐条裁决）

- 轮次：第 1 轮 · 审计 agent（只读）
- 裁决口径：`08_render_arch.md`（契约 4：组件内几何常量属实现可写死；契约 1：1080 单源 + 壳层同值副本）+ `09_render_framework.md`（§5 数据分离：色板权威 = `Theme::DEFAULT`，theme.json 是**测试锁定恒等的镜像**，非独立源）+ `02_conventions.md` §1（「进 scenes.json **或命名常量**」）+ `scripts/check_hardcode.py` EXEMPT 表 + `plan/ledger.yaml` S21.x + `08_main_map_v4_spec.md` §7/§8。
- 方法：97 条逐条打开 文件:行 亲验（grep/Read）；⑤-05/⑤-06 按主线程点名**实测**（见 §实测记录）。

## 顶部统计

| 裁决桶 | 条数 | 编号 |
|---|---|---|
| **真漂移** | **38** | ①-04,①-06,①-10,①-13,①-23,①-24,①-25,①-26,①-27,①-28,①-29,①-30,①-32,②-07,③-01,③-02,③-13,③-14,③-15,③-16,③-17,③-18,④-05,⑤-01,⑤-02,⑤-03,⑤-05,⑤-06,⑤-07,⑥-01,⑥-02,⑥-03,⑥-07,⑥-10,⑥-11,⑥-12,⑦-10,⑦-11 |
| **误报** | **28** | ①-02,①-03,①-05,①-07,①-08,①-09,①-11,①-14,①-15,①-16,①-17,①-18,①-19,①-20,①-21,①-22,②-02,②-03,②-05,②-06,③-12,④-02,④-03,⑥-04,⑥-05,⑥-06,⑥-08,⑥-09 |
| **计划内** | **27** | ①-31,①-33,②-01,②-04,②-08,③-03,③-04,③-05,③-06,③-07,③-08,③-09,③-10,③-11,④-01,④-06,④-07,⑦-01,⑦-02,⑦-03,⑦-04,⑦-05,⑦-06,⑦-07,⑦-08,⑦-09,⑦-12 |
| **需人裁决** | **4** | ①-01,①-12,④-04,⑤-04 |
| 合计 | **97** | |

### 特别核对 10 项结论（主线程点名，已亲自 Read/实测）

| 项 | 结论 | 一句话依据 |
|---|---|---|
| ①-01（chrome.rs:319 TOKENS） | **需人裁决** | 9 分区→令牌映射表现 = `chrome::group_color_token` 文档化单一源（S19.7 D2 收敛）；落哪份数据（balance/terrain/代码）是设计分歧（主线程点名的例子） |
| ②-01（draw_board 海洋格） | **计划内 S21.4** | `main_map.rs:577-608` 全格绘制（含 sea_shallow/sea_deep 图例格）实存；S21.4 dod 明文「只画 陆/礁/宝（海洋格不画 → 巨型蓝菱形消失）」 |
| ③-01（FX_BATTLE_HIT_POS） | **真漂移** | `mod.rs:281 = (540.0,880.0)` + `input.rs:127` 消费，注释自述「视觉验收可微调」= 典型待数据化参数；不在 S21 计划 |
| ③-04（toast 常量） | **计划内 S21.6/S21.7** | `main_map.rs:541-556` pad 24/min_w 240/h 72/540/1520 全裸字面量实存；v4 定案 `{344,1600,392,64}`（spec §2），S21.3 规格 §7「数据改在 S21.5/6/7」 |
| ④-01（节点图标 match） | **计划内 S21.5** | `main_map.rs:496-505` 8 类图标表实存；spec §6 已定案 kind→`node_*.png` 映射「S21.5 落 chrome.rs 映射表」+ S21.5 dod「kind↔资产映射表唯一事实源」 |
| ④-02（generic.rs icon 丢 size/theme） | **误报** | `generic.rs:102-105` 实存（`let _ = size` + `Theme::default()`），但 `09_render_framework.md` §5.1 明文：「CPU 执行器忽略 `params.size`；字形仍为原生几何」「双后端同构」——按 rect 左上放原生 48px，Bevy 侧 `bake_icon`（`scene_render.rs:177-184`）+ sprite 左上对齐 = 同构；「丢弃」是规格行为非缺陷。唯一理论分叉 = size<48 时 Bevy 烘焙裁切而 CPU 不裁——现 scenes.json icon 原语 size=64（≥48）未触发 |
| ④-03（bake_icon） | **误报** | 同 ④-02：`size×size` raster 画原生几何于 (0,0)（size>48 留白、size<48 裁切）+ sprite 中心 = `rect.x+size/2` → 图标落点 = rect 左上，与 CPU 端逐像素同位（双后端一致）；`Theme::default()` 与运行时 theme 当前恒等（§5.1 权威口径）；size<48 裁切为潜伏 edge case 非既成漂移 |
| ⑤-05（STR_LIT 正则） | **真漂移（实测坐实）** | 实测 3：导入脚本原 `STR_LIT` 后，CJK 字面量 `…不 fallback）` 的 `group(1)` = 末尾单字符 `）`（len=1，非 CJK 区）→ 漏检；以汉字结尾的 `加载失败` 才命中。`python scripts/check_hardcode.py` 实跑 = **OK (153 files, 0 hits)** → gate 12 CJK 通道对「以标点/ASCII 结尾」的 CJK 字面量实际失明。修法 `r'"((?:[^"\\]|\\.)*)"'` 后全仓扫 = **64 条 CJK 命中**（清单估 ≥20，实测更多） |
| ⑤-06（EXEMPT 失配） | **真漂移（实测坐实）** | 实跑 + 路径子串验证：EXEMPT 模式 `game_app.rs` 与 S20.0b 拆分后 `game_app/{board,host,mod,assets}.rs` 全部失配（子串不成立）；`treasure_bevy.rs`/`bevy_app.rs` 无豁免行。修 ⑤-05 后 64 命中散布 12+ 文件，EXEMPT 现表仅覆盖其中 0 个渲染域文件 → 不联修 = 60+ 新 FAIL |
| ⑥-08（29 个 COLOR_* 镜像） | **误报（判断错）** | 实数 = 34 个重导出常量（chrome.rs:26-92，清单「×29」误计）；但 `theme.rs:152` 注释明文「COLOR_* 兼容重导出与 theme.json 镜像都锚定此处（单一事实源：改色只改这里，常量/JSON 自动同步）」+ `09_render_framework §5.1`「权威 = Rust 默认值，theme.json 是逐字段恒等测试锁定的镜像」→ 镜像链编译期自锁，**无漂移路径**；清单「theme.json 改色对使用点不生效」判断错（改 JSON 不改 Rust 会被 theme_json.rs 测试直接锁红） |

## §实测记录（⑤-05 / ⑤-06 审计轨迹）

1. **捕获组行为**（导入 `scripts/check_hardcode.py` 原 `STR_LIT` 实测，非自写复刻）：
   - `STR_LIT.search('eprintln!("[stats] …（硬错误，不 fallback）")')` → `group(1) = '）'`，len=1，`CJK.search(group(1)) = False`（漏检）；
   - `STR_LIT.search('"加载失败"')` → `group(1) = '败'`，len=1，命中（仅末字符为汉字时才中）；
   - 结论：`"([^"\\]|\\.)*"` 的捕获组 1 在 `*` 量词内**只保留最后一次迭代**（= 字面量末字符），清单 ⑤-05 判断**成立**。
2. **gate 12 现状**：`python scripts/check_hardcode.py` → `check_hardcode: OK (153 file(s), 0 hits)`，exit 0 → CJK 通道对靶点域失明（清单「51 条 CJK 字面量，gate 12 报 0 命中」与实测一致）。
3. **修复后暴露面**（把 `STR_LIT` 换成 `re.compile('"((?:[^"\\]|\\.)*)"')` 复扫全 crates/，内存内操作未写文件）：**64 条 CJK 命中**，散布：`game_app/board.rs`×9（37/46/56/67/78/140/275/280/286）、`game_app/host.rs`×2（192/197）、`scenes/main_map.rs`×3（31/62/162）、`scenes/mod.rs`×19（228-246 宏表）、`bevy_app.rs`×1（49）、`treasure_bevy.rs`×10（394/440/559/568/574/631/740/756/757/760/761）、`ti-core/data/mod.rs`×2（659/677）、`ti-platform/hd_png.rs`×3、`ogg_encode.rs`×3、`ti-formats/ids.rs`×1、`sprite_step.rs`(bin)×9、`android_shell.rs`×1。→ ⑤-06「EXEMPT 失配」坐实，且暴露面比清单估计（「20+ 条」）更大。

## ① 色值/数值字面量（33 条）

- [①-01] 需人裁决 | 9 分区→令牌映射（`chrome.rs:319` `TOKENS[9]`）实存且为 S19.7 D2 文档化单一源（赛道标记 + 航程条共用，:315-316 注释），当前不算漂移；分歧点 = 该表该挪进 `balance.json board.group_tokens` / `terrain.json legend.group_tokens` / 仍留代码（分区码来自 terrain 数据、映射是视觉决策，归属无先例）；v4 节点图片化（S21.5）后 v3 该表消费方收缩，随 S21 出图后裁决。挂接：S21.9 出图验收时一并拍板。
- [①-02] 误报 | `main_map.rs:402` `Theme::DEFAULT.moon_pale` 实存，但与运行时 theme 恒等（权威 = Rust 默认值，§5.1；theme.json 只是镜像）→ 无漂移路径（同 ⑥-08）；且整张图标表随 S21.5 节点 PNG 化退役（挂 S21.5）。
- [①-03] 误报 | `main_map.rs:496-505` 图标 tint 表实存，值全部编译期锚定权威源（同上）；该表 S21.5 整体退役（挂 S21.5；清单建议的 `scenes.json track_icons` 中间态无必要）。
- [①-04] 真漂移（中） | `chrome.rs:528-536`（清单行号 :531 漂移 1 行）`page_brightness_factor` 的 0.6 下限/0.4 幅值实存，未登记 `gate_data.json ui_motion`（现仅 4 行 MOTION_*，实测）也未入 `10_ui_redesign.md §7` 表 = 未受 doccheck2 `check_ui_redesign_tokens` 锁定的第 5 个动效参数。挂接：gate_data.json ui_motion + §7 表登记 0.6/0.4 两参 + doccheck2 扩断言（最小修复，不扩 theme.json schema）。
- [①-05] 误报 | `chrome.rs:360-362/378/405-407/424-425` + `main_map.rs:461/485/548-549` 的 `unwrap_or(COLOR_GOLD_WARM)` 实存，但 = 文档化兜底惯用法（未知令牌 → 命名兜底色）；值锚定权威源无漂移路径（同 ⑥-08）；「统一 fallback」属风格项非漂移。
- [①-06] 真漂移（低） | `board.rs:21` `TOAST_TICKS: u32 = 120` 帧制视觉时长实存，未入 S17.4 动效表（ms 制）/gate_data（同 ①-04 机制外）。挂接：登记 gate_data ui_motion + §7 表（帧制→ms 制归一属子分歧，最低限度 = 命名常量 + doccheck2 断言登记）。
- [①-07] 误报 | `host.rs:81` `map_or(20, ...)` 实存，但 = 数据源（terrain_map）**缺失时**的 crash-safe 兜底（02_conventions §4「加载失败 = 硬错误」族）；清单建议 `balance.json board.fallback_grid_w` 逻辑自洽性差（兜底值放哪个文件都非「改平衡」场景）；命名可接受。
- [①-08] 误报 | `mod.rs:410` `map_or(5, |b| b.save.slot_count)` 实存，同 ①-07 族（balance.json 缺失时的兜底，不能从缺失文件读自己）；非渲染数值。
- [①-09] 误报 | `mod.rs:285` `FX_SEED = 0xC0_FF_EE` 为**命名 pub const + doc 注释**（「FX 专用 LCG，不进业务 RNG」）= 满足 02_conventions「或命名常量」口径；确定性视觉验收判据非调参项，数据化进 theme.json 反成噪音。
- [①-10] 真漂移（低） | `assets.rs:160` `for k in 0..5u32` 5 帧假设实存（`multiframe_f1` 恒按 f0..f4 加载）；帧数 = 资产结构数据，写死则第 6 帧起静默丢失。挂接：manifest 驱动帧数（`ui_manifest` multiframe 条目带帧数键或按 f0..fN 探测），`hd-8bit` feature 内修复。
- [①-11] 误报 | `chrome.rs:293` `(…).max(4)` 实存 = 退化尺寸守护（语义 guard：百分比×tw 过小不画 4px 以下残点），非调参；契约 4 组件实现。
- [①-12] 需人裁决（低） | `theme.rs:161/162` `gold_glow == gold_bright == 0x4E3D` 实存（两令牌同值）；分歧点 = 合并令牌（37→36，需同步 by_name/§7 表/doccheck2/theme.json 镜像）vs 保留别名 + 显式标注；涉 theme schema 面。挂接：theme.json 三方断言扩「同值令牌对须标注 alias」（清单建议），随 10_ui_redesign 令牌表维护拍板。
- [①-13] 真漂移（低） | `treasure_bevy.rs:188` 底色 `Color::srgb(0.1,0.07,0.05)` 实存 = Bevy 壳全屏底色精灵不走 theme 令牌（CPU 侧同位 = `theme.parch_dark`）；双壳底色口径漂移风险。挂接：改 `rgb565_to_color(theme.parch_dark)`（shell 经 GameStore 取 theme），双后端底色同源。
- [①-14] 误报 | `treasure_bevy.rs:230/233` zoom 步进 1.25/0.8 实存，但属**交互手感参数**非渲染决策；「进 theme.json camera 组」= schema 扩展无先例；同句的裸 540.0/960.0 字面量已由 ③-18 报（本条不重复计）。
- [①-15] 误报 | `shell.rs:12` `let f = 12i32` 实存，但 `08_render_arch 契约 4` 明文以「`wood_frame` 的 12px 外框宽」为例 = **可写死在代码的实现几何常量**；非场景级坐标（x/y/w/h 均参数传入）。
- [①-16] 误报 | `shell.rs:76-93` menu_item 8px 描边/2px bevel/8×8 铆钉实存 = 组件内部样式几何，契约 4 豁免族（同 ①-15 理据；清单建议 theme.json `menu_item{}` 参数组 = schema 扩展，超出 09 §5 现行数据分离范围）。
- [①-17] 误报 | `shell.rs:100-101` bar_slot 4px 内缩/2px 阴影实存 = 同上（组件内部样式，契约 4）。
- [①-18] 误报 | `shell.rs:106-111` panel 2px 木边/1px bevel 实存 = 同上。
- [①-19] 误报 | `shell.rs:117-125` hint_row +8/+4/-2/1px 实存 = 同上。
- [①-20] 误报 | `shell.rs:230-257` title_art_block 几何组实存 = 同上（装饰组件内部几何，场景矩形经参数传入）。
- [①-21] 误报 | `shell.rs:274-308` battle_arena 几何组实存 = 同上。
- [①-22] 误报 | `shell.rs:319-343` world_canvas_frame 2px/6px/12×12 实存 = 同上；清单挂 S21.3 v4 框参数——v4 画布框几何由 `scenes.json world_canvas` 键 + S21.6 定，组件内样式参数仍属契约 4 豁免。
- [①-23] 真漂移（低） | `shell.rs:376/388/396` `const DITHER` 已命名，但亮度曲线 `550 + idx*450/steps`（55%→100%）与 `clamp(8,2048)` 为**内联未命名视觉数值**（色值决策非几何），违反 02_conventions §1「命名常量」底线口径。挂接：最小修复 = 抽命名常量（`NIGHT_SKY_BASE_PERMILLE` 等）；theme.json sky 组 = 可选项（schema 扩展不属本轮）。
- [①-24] 真漂移（低） | `shell.rs:405/412` `LAYERS=16` 已命名，但 `120 + …*880/16` 光晕亮度斜坡内联未命名 = 同 ①-23 口径。挂接：抽命名常量。
- [①-25] 真漂移（低） | `shell.rs:436-437` 波浪 `step_by(4)`/`%3`/`-1` 内联未命名（装饰算法参数）= 同 ①-23 口径。挂接：抽命名常量。
- [①-26] 真漂移（低） | `shell.rs:455/457` CORNER/ARM 已命名，但 32 辐条（`for i in 0..32`）、`cy = y0 - 24`、4px 木框/8px 金线内缩内联未命名 = 同族。挂接：抽命名常量。
- [①-27] 真漂移（低） | `shell.rs:528` `48i32.min(height - 8)` 图标默认 48 内联 = ⑥-02 图标 48 单源族（4 处各抄一份）。挂接：随 ⑥-02 单源（`ui_icons.json` 注册表 size / `ICON_SIZE` 派生）。
- [①-28] 真漂移（低） | `generic.rs:143` glow 衰减 `350 + t*650`（35%→100% 千分比）内联未命名 = 视觉数值。挂接：抽命名常量（双后端同读，与 ⑥-12 同批）。
- [①-29] 真漂移（低） | `generic.rs:113` `steps.clamp(2, 256)` 内联钳制区间 = 同族（Bevy 侧 `scene_render.rs:280` 同式再抄，见 ⑥-12）。挂接：抽 `clamp_grad_steps` 共享函数或命名常量。
- [①-30] 真漂移（低） | `primitives.rs:54/56` `gradient_row(…, 0.18)` + `vignette(…, 36)` 内联未命名（渐变强度 + 暗角宽 = 视觉调参值）= 同族；v2 回退路径但非 S21 退役对象。挂接：抽命名常量。
- [①-31] 计划内（S21.6） | `shell.rs:41-48` nautical_grid 几何（13×30/1040/60/32/20×2）实存，但 v4 spec §8 定案「经纬网/罗盘烘焙进 `chart_full`」，本函数随 v2 双写支路在 S21.6 整体退役——修参数无意义，随删。挂 S21.6。
- [①-32] 真漂移（低） | `shell.rs:203` 星空屏心净空锚点 50/52 内联（`STAR_CENTER_EXCLUDE=12` 已命名）= 视觉数据 + ④-05 STARS 表同族。挂接：随 ④-05 星位表数据化（`scenes.json <scene>.ambient_art.stars` + center_x/y_pct + exclude_radius_pct）。
- [①-33] 计划内（S21.5/6） | `chrome.rs:366-389` voyage_bar 参数组（pad 24/2px 条/6px 槽/10/6 点/±14,-40,28/±12,14,24 标记）实存，但 v4 spec §8「45 点噪声条 → 1 条进度尺 984×48」由 S21.5 重写（S21.5 dod 含「航线进度尺（1 线 + 9 扇区段 + 船标 + BOSS 终标，替代 45 点噪声航程条）」）+ S21.6 落 `voyage_bar {48,1088,984,48}` 新键——旧参数组随旧画法退役。挂 S21.5/6。

## ② 画/不画决策（8 条）

- [②-01] 计划内（S21.4） | `main_map.rs:577-608` `draw_board` 全格绘制含海洋格实存（legend 取色 `theme.by_name(&lg.token)`，sea 格照画）；S21.4 dod 明文重做 `draw_board`「跳过 sea 格 + 海岸线判定 + 陆地纹理」。清单「高」严重度成立但任务已登记，本轮不修。挂 S21.4。
- [②-02] 误报 | `main_map.rs:386` `if !spaces.is_empty() { continue; }` 实存，但 = **状态条件画法规则**（有赛道 → 赛道空间标记接管 balance 据点，防双标记）；契约 3 规定视图层「读状态 → 决定画什么」是本职，02_conventions/契约 4 的数据化要求针对坐标/数值/色值而非条件逻辑；`scenes.json visible_when` 机制无先例 = 未裁决设计分歧，本轮不扩靶（同 ②-03）。
- [②-03] 误报 | `main_map.rs:411` `if spaces.is_empty() { …BOSS 格… }` 实存 = 同 ②-02 口径（语义去重 guard，非渲染参数）。
- [②-04] 计划内（S21.5） | `chrome.rs:353` `spaces.len() < 2 { return; }` 实存；voyage_bar 整体由 S21.5 重写为进度尺（新画法的显隐门槛随 S21.5 定），单独给旧条加 `min_spaces` 键 = 马上要删的代码加数据键，无收益。挂 S21.5。
- [②-05] 误报 | `shell.rs:251` `if ih >= 96` 实存 = 组件自适应守护（面板过矮不画罗盘，退化尺寸 guard）+ 契约 4 组件内部几何。
- [②-06] 误报 | `shell.rs:475` `if fw > 16 && fh > 16` 实存 = 同上（ornate_frame 金线退化守护）。
- [②-07] 真漂移（低） | `message.rs:92` `body.y + body.h - 48` 溢出 reserve 48px 裸字面量实存 = 场景级偏移未走 `scenes.json`（同段 :89 行高 52 / :82 x 衬边 4 / :103-107 页点 12×12/36/48 同族裸数）。挂接：`scenes.json message.body {clip_reserve:48, line_h:52, …}`（或 8pt 栅格值）+ 代码读键。
- [②-08] 计划内（S21.6） | `main_map.rs:316-321` v3/v2 双轨 if 实存；v4 定案后 v2 支路整体删除 = S21.6 版面重排范围（spec §8「v4 不画 v2 回退木框」；S21.6 dod「main_map 95 原语重排 + 画布矩形」）。挂 S21.6。

## ③ 尺寸/坐标/偏移常量（18 条）

- [③-01] 真漂移（中） | `mod.rs:281` `FX_BATTLE_HIT_POS = (540.0, 880.0)` 实存（`input.rs:127` 消费）；FX 触发坐标 = 版面数值，注释自述「视觉验收可微调，S18.7 贴图管线接手版面」= 待数据化未落地；不在 S21.x 计划。挂接：`scenes.json vs.fx_hit {x,y}`（或 theme.json fx 组）+ input.rs 读数据。
- [③-02] 真漂移（中） | `mod.rs:283` `FX_COIN_GOLD_POS = (900.0, 140.0)` 实存；应锚定 HUD 面板（v4 = `hud_plaque_l/r` 区，spec §2），裸坐标与版面键脱钩。挂接：同 ③-01 通道（`scenes.json main_map.fx_coin` 或直接引用 `hud_plaque_r` 矩形中心；v4 坐标值随 S21.6 版面重排后核对一次，规则层不擅改值）。
- [③-03] 计划内（S21.6/7） | `main_map.rs:526-527` 骰点 x=540 屏心 + y=1296 实存；v4 spec §2「摇骰反馈 = CTA（deck_spin {408,1160,264,264}）上沿 y≈1136」已定案，S21.7 dod「提示签条 + 摇骰后落点」含骰点锚改。挂 S21.6（数据键）/S21.7（代码）。
- [③-04] 计划内（S21.6/7） | `main_map.rs:541-556` pad 24/min_w 240/h 72/540×2/1520×2 实存；v4 定案 `{344,1600,392,64}`（spec §2，8pt 校验 ✓），spec §7「数据改在 S21.5/6/7」。挂 S21.6（`scenes.json main_map.board_toast` 键）/S21.7（代码读键）。
- [③-05] 计划内（S21.6） | `main_map.rs:123/130` v3 HUD `rc.x + 8, rc.y + rc.h/2 - 22` 实存；v4 顶角双铜牌 `hud_plaque_l/r {48,48,448,192}` 替换 `hud_gold/hud_player`（spec §2）= S21.6 版面重排范围，数字落点随新键定。挂 S21.6。
- [③-06] 计划内（S21.6） | `main_map.rs:144/147` v2 HUD 偏移 +4/+2/+56 实存 = v2 残留，S21.6「v2 键删除」同批收口。挂 S21.6。
- [③-07] 计划内（S21.5） | `chrome.rs:386/389` 玩家 compass(-14,-40,28)/BOSS sword(-12,+14,24) 实存；v4 进度尺「玩家船标 + ☠ 端标」（spec §8）随 S21.5 重写定新几何。挂 S21.5。
- [③-08] 计划内（S21.5） | `main_map.rs:476-481` 路径连线 n=5 插值 + 3×3 虚线方块实存；v4「黄铜航线实线替 45 点小方块」= S21.5 赛道重绘（dod 明文）。挂 S21.5。
- [③-09] 计划内（S21.5） | `main_map.rs:493-495` 赛道标记 20×20±10 + 1px bevel 实存；v4 节点 = `node_*.png` 多形状牌「随 tw 倍率 1:1」（spec §6/§8）随 S21.5 整体替换。挂 S21.5。
- [③-10] 计划内（S21.5） | `main_map.rs:507` 节点图标 cx-14/cy-30/28 实存；v4 节点牌图片化后本行退役（清单自述）。挂 S21.5。
- [③-11] 计划内（S21.5） | `main_map.rs:513/517` 标签 +12/-8 实存（×2：BOSS/起点）；同 S21.5 节点牌管线内定。挂 S21.5。
- [③-12] 误报 | `shell.rs:24-35` title_bar 12/16/20/8/3/48 + 罗盘 28×3/3×28/4×4 实存 = 组件内部几何常量（契约 4 明示「几何常量属实现，可写死」；场景标题条高 `title_h` 由 scenes.json 传入，本条只涉框内装饰参数）。
- [③-13] 真漂移（中） | `select_map.rs:170-171` 锁块 `(card_right - 6 - lock_x).min(40)` / `.max(4)` / 高 12 裸字面量实存（`text_x` 已数据化但几何参数未随迁）= 场景级版面数值写死。挂接：`scenes.json select_map.locked {inset:6, max_w:40, min_w:4, h:12}` + 代码读键。
- [③-14] 真漂移（中） | `select_map.rs:183-191` BOSS 难度星 14/6/12/10/上限 3/2px 全裸字面量实存 = 场景级版面参数组写死。挂接：`scenes.json select_map.stars {size:14, gap:6, inset:12, top:10, max:3, bevel_px:2}` + 代码读键（max 与 `rules::progression` 星数钳制是两回事，仅几何入 JSON）。
- [③-15] 真漂移（低） | `saveload.rs:151` `ty + 52` 摘要行偏移实存 = 场景级偏移写死（行高 224 数据槽已有，摘要 dy 未随迁）。挂接：`scenes.json saveload.list.summary_dy: 52` + 读键。
- [③-16] 真漂移（低；条内一半误报） | `index.rs:95-97`：「status」键裸字符串未入 `chrome::keys` 模块（键治理缺漏，实存）= 有效；但「+2 偏移写死」= **误报**——`chrome::hint_row`（chrome.rs:814）签名 `_x` 忽略入参、条体屏心居中（:816 `cx = LOGICAL_W_I32/2`），`status.x + 2` 是死参无效果。挂接：「status」键并入 `chrome::keys`（如 `STATUS: &str = "status"`）+ 删无效 +2 传参；键名治理纳入 ⑦-08 S21.6 换版同批可省一次改动。
- [③-17] 真漂移（中） | `canvas.rs:40/47` `cx - 540.0, 960.0 - cy` 裸字面量实存（同文件 :38 已 import 并使用 `LOGICAL_W_I32/LOGICAL_H_I32`，`scene_render.rs:42-44` 的 `to_world` 用 `LOGICAL_W/2` 派生式 = 仓内正确口径）；同 crate 两口径并存 = 1080 单源契约（契约 1）漂移风险。挂接：`canvas.rs` 改 `LOGICAL_W_I32 as f32 / 2.0` 派生 + doccheck2 扩 grep 断言（`ti-engine` 禁裸 540/960，同 `check_runtime_no_legacy_coords` 手法）。
- [③-18] 真漂移（低） | `treasure_bevy.rs:230/233/321/322` `[540.0, 960.0]` ×4 + `cam.cam[0] - 540.0` / `960.0 - cam.cam[1]` 裸字面量实存 = 同 ③-17 族。挂接：随 ③-17 归一（引擎侧派生 `LOGICAL_W/H` 半值常量一处定义）。

## ④ 该用图片处用代码画（7 条）

- [④-01] 计划内（S21.5） | `main_map.rs:496-505` 节点 8 类代码图标 + 20×20 方块实存；spec §6 定案 kind→`node_*.png` 映射「S21.5 落 chrome.rs 映射表」（资产 S21.1 已入仓），S21.5 dod「kind↔资产映射表唯一事实源 + 45 节点牌可辨出图」。挂 S21.5。
- [④-02] 误报 | `generic.rs:102-105` `let _ = size` + `Theme::default()` 实存；但 §5.1（09_render_framework）明文契约「图标按注册表原生 48px 绘制在 rect 左上角，`rect` 边长仅占位盒（CPU 执行器忽略 `params.size`；Bevy 执行器按 rect 边长烘焙纹理，**字形仍为原生几何**）」——两后端对「原生几何置于 rect 左上」同构，「丢 size」= 规格行为；theme 用 `default()` 与运行时 theme 恒等（权威口径）。清单「双后端不一致」判断不成立（唯一理论分叉 size<48 裁切未触发：scenes.json 现存 icon 原语 size=64）。
- [④-03] 误报 | `scene_render.rs:177-184` `bake_icon` 实存（size×size raster，原生几何 (0,0)，忽略 theme）；与 CPU 端（④-02）出图同位（sprite 左上 = rect 左上，custom_size = size²）→「不一致」不成立；size>48 留白 / size<48 裁切均为规格内行为（§5.1「改图标大小须改 rect 边长」）。潜伏项：size<48 裁切 vs CPU 不裁切的分叉，建议在 S21.5 节点图片化管线或后续「烘焙按 size/48 缩放」专项处理（非本轮既成漂移，不计真漂移）。
- [④-04] 需人裁决（中） | `icons.rs:57-165` 14 枚代码矢量图标（上百条像素偏移 fill_rect）实存；但 = 文档化设计决策（`10_ui_redesign.md §5`：「图标是**框架层**几何绘制，零外部资产」「风格统一 4px 笔画」）；分歧点 = 图标几何图片化（`assets/ui/icon_*.png` + `ui_asset_batch.py` 批次 + 注册表 `image` 字段）vs 维持代码矢量（现状，`ui_icons.json` 只管映射）；改动面 = 资产管线 + 框架契约 §5.1 + 原语闭集门禁 + 48px 盒数据化，超本主题靶点。挂接：S21.9 出图验收或专项拍板；过渡期最小数据化（48px 盒 + 笔画宽入 `ui_icons.json`）可作选项。
- [④-05] 真漂移（中） | `shell.rs:152-179` `pub const STARS: &[(u8,u8,u8,bool); 26]` 星位表实存 = **值表写代码**（非算法：26 行 (x_pct, y_pct, 边长, 亮暗) 纯数据，注释自述「确定性渲染快照可复现」正说明它该是数据）；v3 index/vs 装饰层非 S21 退役对象（`chart_full` 烘焙只覆盖 main_map 底）。挂接：`scenes.json <scene>.ambient_art.stars`（+ ①-32 的 50/52/12 净空参数）数据化，代码只遍历；或 `assets/ui/star_sky.png` 一张图（v4 海图「装饰烘焙进图」同思路，spec §8 判据）。
- [④-06] 计划内（S21.12） | `main_map.rs:373/375` 玩家 = 绿方块 + 代码 flag 图标实存；v4 spec §8「绿方块 + flag 图标 → 船（boat_level 切 4 图，S21.12）」+ S21.12 dod「玩家标记按 boat_level blit_image 切换，缺资产回退链 ship_player → 代码 flag」。挂 S21.12。
- [④-07] 计划内（S21.5） | `main_map.rs:377-378/404-405/420-421` 1px bevel 双行 ×3 实体块实存；v4 实体（玩家船/节点牌）S21.5/S21.12 图片化后本模式在 main_map 退役；ti-render `shell::panel`/`compact_menu_item` 同族模式属框架层（契约 4 组件内部，不报）。挂 S21.5（清单建议的 `bevel_box` 单一原语 = 框架重构，见 ⑥-10，非本轮）。

## ⑤ 中文面量（7 条）

- [⑤-01] 真漂移（低，随 ⑤-06 批） | `board.rs:37/46/56/67/78/140/275/280/286` 9 处 CJK 诊断实存（亲验）；其中 :56/:67/:78/:275/:280/:286 **不在 `#[cfg(stats)]` 门后**（恒编译 eprintln），仅 :37/:46/:140 stats 门后。这些是硬错误诊断打印（非玩家 UI），口径 = 诊断通道豁免：EXEMPT 补 `game_app/` 路径模式（现 `game_app.rs` 模式 S20.0b 拆分后失配，实测）。挂 ⑤-06 同批。
- [⑤-02] 真漂移（低，随 ⑤-06 批） | `host.rs:192`（stats 门后）/`:197`（恒编译）+ `scenes/main_map.rs:31/62/162`（stats 门后 `[stats]` 前缀）实存（亲验）；同 ⑤-01 口径。挂 ⑤-06 同批（EXEMPT 需逐文件带理由声明，避免豁免面失控——清单自述，保留）。
- [⑤-03] 真漂移（低，随 ⑤-06 批） | `treasure_bevy.rs:440/559/568/574/740/756/757/760/761` CJK 诊断实存（实测修正：另漏 :394，共 10 处非清单 8 处）；= 离屏/自检诊断 bin，应比照 `render_png.rs`/`forward_nav.rs`/`treasure_headless.rs` 的 EXEMPT 先例补 `treasure_bevy.rs` 行（理由：诊断终端输出非游戏 UI）。挂 ⑤-06 同批。
- [⑤-04] 需人裁决（低） | `treasure_bevy.rs:631` 窗口标题 `"金银岛 TreasureIsl（Bevy）"` + `bevy_app.rs:49` `"金银岛 Bevy S18.1 spike"` 实存（亲验）；分歧点 = 窗口标题算「玩家可见」走 `strings.json app.window_title`（i18n 清零口径）还是 app 外壳 EXEMPT（`treasure_headless.rs` 先例族）；spike 壳标题明显开发工具（宜豁免）。挂接：EXEMPT 修订（⑤-06 批）内逐条拍板，或 strings.json 新增 app 段。
- [⑤-05] 真漂移（高，工具链根因 A） | **实测坐实**（§实测记录 1-3）：`STR_LIT` 捕获组在 `*` 量词内只保留末次迭代 → CJK 扫描退化为「末字符是否汉字」；gate 12 实跑 0 hits（153 文件）= CJK 通道对以 `）`/`，`/ASCII 结尾的字面量失明；修法 `r'"((?:[^"\\]|\\.)*)"'` 实测全仓 64 条 CJK 命中。挂接：本批修复（check_hardcode.py 正则 + 回归 fixture：CJK 字面量以 `）` 结尾必须 FAIL；登记 `docs/process/14_tooling_catalog.md` 工具行为变更）；与 ⑤-06 同 commit。
- [⑤-06] 真漂移（高，工具链根因 B） | **实测坐实**：EXEMPT `("game_app.rs", …)` 与 `game_app/{board,host,mod,assets}.rs` 全失配（路径子串验证：`"game_app.rs" in "…/game_app/board.rs"` = False）；`treasure_bevy.rs`/`bevy_app.rs` 无豁免行；修 ⑤-05 后 64 命中散布的 12+ 文件中 EXEMPT 现表一个都没覆盖到渲染域（实测列表见 §实测记录 3）。挂接：EXEMPT 按拆分后文件清单重刷（`game_app/`（cjk，诊断口径）+ `treasure_bevy.rs`/`bevy_app.rs` + 实测新暴露的 `scenes/mod.rs`（宏表）/`ti-core/data`（LoadError 文案）/`ti-platform/{hd_png,ogg_encode}` /`ti-formats/ids`/`sprite_step.rs` bin/`android_shell.rs`，每条带理由）；与 ⑤-05 同批（只修 A 不修 B = 60+ FAIL 直接落 gate 12）。
- [⑤-07] 真漂移（低，随 ⑤-06 批） | `main_map.rs:31/62/162` `[stats] TradeBuy/TradeSell/Build …（硬报错，不 fallback）` ×3 实存（亲验，均在 stats 门后）；同 ⑤-01/02 口径。挂 ⑤-06 同批。

## ⑥ 跨文件重复常量（12 条）

- [⑥-01] 真漂移（中；条内混合） | 亲验：`canvas.rs:40/47` 裸 540.0/960.0（引擎侧，见 ③-17）+ `treasure_bevy.rs:230/233/321/322` ×4（见 ③-18）= 引擎侧 6 处裸字面量**本轮可修**（派生 `LOGICAL_W_I32/2`）；`main_map.rs:526/544/555`（540 屏心 ×3）属 S21.6/7 计划内（骰点/提示条 v4 坐标随数据键定），`mod.rs:281` 属 ③-01（真漂移已单列）。挂接：引擎侧 2 文件归一 + doccheck2 grep 断言（与 ③-17/③-18 同批一次改完）。
- [⑥-02] 真漂移（中） | 图标默认 48 四处各抄一份实存：`chrome.rs:676`（CPU 原语执行器 `unwrap_or(48)`）、`scene_render.rs:337`（Bevy 侧同式）、`icons.rs:14`（`ICON_SIZE: i32 = 48`）、`shell.rs:528`（`48i32.min(height-8)`）；`ui_icons.json` 注册表已有逐图标 `size` 字段（chrome.rs:456 `def.size` 消费先例），缺省语义无单源。挂接：单源 = 注册表缺省 size（`ui_icons.json` 顶层 `default_size: 48`）或 `theme.json icon_default_px`，四调用点改读 + 双后端同读（①-27 同批）。
- [⑥-03] 真漂移（中） | 亲验：`chrome.rs:455-461`（deck_button 图标排布 `r.w >= def.size + 8` / `r.x + 8` / `ix + def.size + 8`）与 `main_map.rs:100-106`（render_hud_button 同式逐行复制）实存 = 两场景按钮排布逻辑重复（同族 8px 衬边参数 ①-33 族）。挂接：抽 `chrome::layout_icon_label(host, rect, label, icon_def) -> (label_x, icon_opt)` 单一函数，两调用点改一行。
- [⑥-04] 误报（判断错：死代码） | `scene_render.rs:433-467` `known_tokens` 实存且确实漏新令牌（实数 25 名 vs `by_name` 37 令牌，清单「26/漏 11」两处小误计）；但**全仓 grep `known_tokens` 零消费方**（仅定义自身）= 死函数，无实际漂移风险；清单建议「theme 反射生成」对死代码过度。挂接：直接删除该函数（死代码清零口径，同 ⑦-10/⑦-11 批）；若保留则加 doccheck2 死导出守卫。
- [⑥-05] 误报 | `scene_render.rs:36-38`（f32 1080/1920）+ `treasure_bevy.rs:50-55`（u32 WIN/LOGICAL）实存，但 = **命名常量 + doc 注释指向单一源**（「逻辑画布（`ti_render::framebuffer` 单一事实源）」），满足契约 1「壳层同名常量 = 同值副本」+ 02_conventions「或命名常量」双口径；「引擎侧派生 `LOGICAL_W_I32 as f32`」是更好的风格（ti-engine 已依赖 ti-render），但非漂移——无值分叉风险（副本恒等于权威）。挂接：可选清理（随 ③-17 批把 engine 侧改派生，一次顺手，不单独立项）。
- [⑥-06] 误报 | `treasure_bevy.rs:137/721/730/748/749` 黑像素判据 `> 8` 五处实存（亲验），但 = **离屏截图诊断判据**（「非黑像素计数」验收口径）非渲染数据；同文件内 5 处重复 = 代码风格项（抽 `fn frame_has_content` + 命名阈值即可，可选），不达「渲染决策写死」门槛。
- [⑥-07] 真漂移（中低） | `select_role.rs:136` `for i in 0..5` 实存（亲验）——同文件 :12 `const ROLE_COUNT: u32 = 5` 被 on_input（:32/:40/:41/:101）正常使用但 render 循环绕过；doccheck2 `ui_icon_registry`/EXP-073 的 band 数门禁只锁 `scene_list_bands(… ROLE_COUNT)` 入参（doccheck2.rs:1067-1088 实读），`render` 的字面 5 是其盲区（亲验门禁代码）。`selected_border(…, 3)`（select_role.rs:141）与 `saveload.rs:130` 的 3px 各抄一份 = 面板语言 3px 参数重复。挂接：循环改 `0..ROLE_COUNT`（+ doccheck2 扩 render 循环字面量断言，补 EXP-073 盲区）；3px 面板边随 ⑥-03 抽公共函数时一并读 `wood_panel`/`selected_border` 单一参数源。
- [⑥-08] 误报（判断错；条内计数错） | 实数 34 个重导出常量（chrome.rs:26-92 亲数，清单「×29」误计）+ 运行时使用点（main_map.rs:373/395/418、vs.rs:70-71、select_map.rs:171/190、saveload.rs:119/130 等）实存；但判断错：`theme.rs:152` 注释明文锚定（「COLOR_* 兼容重导出与 theme.json 镜像都锚定此处，改色只改这里，常量/JSON 自动同步」）+ `09_render_framework §5.1`「色板权威 = `Theme::default()`，theme.json 是**逐字段恒等测试锁定**的镜像」（`ti-render/tests/theme_json.rs`）→ 三条腿（Rust 默认值 / JSON 镜像 / COLOR_* 常量）编译期 + 测试双锁，**无漂移路径**；清单「theme.json 改色对使用点不生效」= 误读权威方向（改 JSON 不改 Rust 会测试红，正确流程先改 Rust）。清单「使用点改 host.theme().xxx」在运行时 theme ≡ DEFAULT 现状下零行为变化，属风格迁移不属修复。挂接：不修；若未来引入「运行时从 JSON 装载主题」设计，再统一切运行时取色（需人裁决，非本轮）。
- [⑥-09] 误报 | `shell.rs:194-197/381-384/430-431/462-465` 四处 `.max(12)/…-12` 外框夹取实存 = 与 `wood_frame` 12px 互锁的**组件内部几何**（契约 4 豁免族，同 ①-15）；「抽 theme.frame_inset 单源」= schema 扩展非既成漂移。
- [⑥-10] 真漂移（低） | `main_map.rs` 3 实体块 + `shell.rs:110-111`/`244-246` 等 1px bevel 双行模式 ×5+ 实存（亲验）；main_map 实例随 S21.5/12 图片化退役（见 ④-07），残余 = ti-render 框架层抽 `bevel_box` 原语。挂接：低优先框架重构（新增组合函数须同步 `08_render_arch.md §2` 组合函数表，§8.6 口径）；本轮可只修 main_map 侧（若 S21.5 先行则本条自然消解，修复 agent 按「S21.5 未动 main_map 实体画法」为前提判断是否动手）。
- [⑥-11] 真漂移（低） | `treasure_bevy.rs:610/626` `set_rng_seed(42)` ×2 + `tests/scripts/full_run.json:2` `"seed": 42`（亲验）= 三处「S5.2 同源固定种子」口径各写一份；改 seed 忘改另一处 = 出图/自检静默不同源。挂接：doccheck2 断言（`full_run.json.seed` == `treasure_bevy.rs` 字面量，grep 对账）或 Bevy 壳从 full_run.json 读 seed；最小修复 = 壳内 2 处合一命名常量 + 断言。
- [⑥-12] 真漂移（低） | `generic.rs:113` 与 `scene_render.rs:280` `steps.clamp(2, 256)` 双后端各钳制一份实存（亲验）= 参数漂移风险（一后端改区间另一后端不跟）。挂接：抽 `ti_render::generic::clamp_grad_steps` 共享函数（双端同读）；与 ①-29 同批。

## ⑦ v2/v3 残留（12 条）

- [⑦-01] 计划内（S21.5） | `main_map.rs:473-521` v3 赛道画法（20×20 方块 + 28px 图标 + 5 段虚线）实存（亲验）；v4 spec §8 定案「黄铜航线实线 + 45 多形状实心黄铜牌（全显）」，S21.5 dod「出图（45 个节点牌可辨 + 航线连续 + 船在当前格）」。挂 S21.5。
- [⑦-02] 计划内（S21.6） | `main_map.rs:140-155` render_hud_v2 + v2 三键（`chrome.rs:184-188` keys）+ 触摸命中分支（main_map.rs:278-286）实存（亲验）；v4 spec §2「顶角双铜牌替换 v3 hud_gold/hud_player，v2 键同批退役」，S21.6「v2 键删除」范围。挂 S21.6。
- [⑦-03] 计划内（S21.6） | `main_map.rs:318-320/338-345` v2 回退支路（parchment_bg + nautical_grid + wood_frame/title_bar）实存（亲验）；spec §8「v4 不画 v2 回退木框/装饰，键保留数据」→ S21.6 以 `image chart_full` 替代后 v2 双写支路删除（`render_hud_v2` 的 `has_world_canvas` 判定一并收口）。挂 S21.6。
- [⑦-04] 计划内（S21.5/6；清单挂 S21.7 有误） | `chrome.rs:344-391` v3 45 点噪声条实存（亲验）；但「航线进度尺」重写在 **S21.5**（dod 明文「替代 45 点噪声航程条」），新几何键 `voyage_bar {48,1088,984,48}` 落 **S21.6**（spec §2/§7「数据改在 S21.5/6/7」）；清单「S21.7 交互升级重写为进度尺」任务号挂错。挂 S21.5（代码）/S21.6（数据键）。
- [⑦-05] 计划内（S21.6/7） | `main_map.rs:536-556` 提示条 1520 魔法数实存（注释自述「面板 1484..1556」，亲验）；v4 spec §2「落格提示签条 {344,1600,392,64}，替换 v3 y=1520」。挂 S21.6（`board_toast` 键）/S21.7（读键）。
- [⑦-06] 计划内（S21.12） | `main_map.rs:365-380` 玩家「绿方块 + flag」实存（亲验）；spec §8 → S21.12「船贴图 4 张 + 玩家标记按船等级切换（缺资产回退 ship_player → 代码 flag 三级链）」。挂 S21.12。
- [⑦-07] 计划内（S21.7） | `main_map.rs:526-527` 骰点 y=1296 实存（亲验）；spec §2「CTA 上沿 y≈1136」，S21.7 交互升级范围。挂 S21.7。
- [⑦-08] 计划内（S21.6） | `scenes.json` main_map 段 v2+v3 双键并存实存（亲验：`hud_trade/hud_sell/hud_build/grid` @1711-1729 与 `world_canvas/hud_gold/hud_player/deck_trade/canvas_entities/focus_anchor` @1756-1798 + `deck_spin`@3085/`voyage_bar`@3091；v4 键 `hud_plaque_l/r` 未落）；spec §7 已声明「数据改在 S21.5/6/7」，S21.6 dod「main_map 95 原语重排 + manifest focus/bbox_pct 对账（N8）」。挂 S21.6（含 ③-16 键治理顺带）。
- [⑦-09] 计划内（S21.6） | `chrome.rs:159-166` GRID_* 键 + :262-268 `board_fit` v2 回退分支 + :204-218 `iso_cell/iso_tile`（亲验零生产调用，仅定义/文档/touch 测试名提及）实存；v4 定案 `world_canvas` 唯一画布键 → v2 回退分支与遗留函数删除属 S21.6 换版收口。挂 S21.6。
- [⑦-10] 真漂移（低） | 亲验（grep 全仓）：`chrome.rs:470/483/488` `scene_shell/scene_base/scene_frame` **零生产调用**（仅定义 + 自调 + doc 提及；vs.rs:49/index.rs:78 命中均为注释）；`chrome.rs:770/786` `menu_item/bar_slot` 转发亦零生产调用（ti-render 侧仅 `tests/geometry.rs` 消费）= 死导出挂着（09 §4「勿当已完成」注所指的 chrome 转发层收敛欠账）。挂接：删除或收敛（`scene_base/scene_frame` 若保留为拆分便利需 doc 标注；`scene_shell` 无调用方应删）；随本轮清理批（与 ⑦-11 同批），不进 S21。
- [⑦-11] 真漂移（低） | 亲验（grep 全仓 `scene_cache` 仅 3 命中：mod.rs:251 定义 / :419 初始化 / assets.rs:20 insert）= **零读方死字段**（S15.4 删 legacy 精灵 blit 后 `.Map` 解析链仅为喂它而活；`render_png.rs`/`android_shell.rs` 注入循环 + `scenes/mod.rs:115` `asset_name` 映射表随之成死链）。挂接：删字段 + `add_scene` + 两 bin 注入循环 + 映射表；legacy `.Map` 解析归 `legacy/` 参照（契约 1.3）；与 ⑦-10 同批清理。
- [⑦-12] 计划内（S21.6；清单挂 S21.8 有误） | `doccheck2.rs:1670-1684` `ui_8pt_walk` 8pt 豁免键表硬编码 v3 键名清单实存（亲验：`world_canvas/hud_gold/hud_player/deck_trade/deck_sell/deck_build/canvas_entities` + 子键）；v4 换版时旧键（`world_canvas {43,250,…}` 非 8pt 值）在 S21.6 被替换后表项自然失效，**新 v4 键（48/1088/984/48/408/1160/264…）全部 8 整除 = 无需新增豁免**（spec §2 8pt 校验已自证）；真正风险 = S21.6 换版 commit 须与豁免表同改（spec §5 N8/8pt 耦合已声明）。清单「S21.8 存量豁免清零即此」= 任务号挂错（S21.8 = `scripts/tooling_contract_baseline.json` 16 个存量工具契约清零，与 8pt 键表无关）。挂 S21.6（豁免表随 scenes.json 同 commit）。

## 本轮修复 agent 应修清单（真漂移 38 条全量）

**高优先（工具链根因，一 commit 收口）**
1. ⑤-05 — `scripts/check_hardcode.py` `STR_LIT` 改 `r'"((?:[^"\\]|\\.)*)"'` + 回归 fixture（CJK 字面量以 `）` 结尾必须 FAIL；实测后全仓 64 命中全绿才可置 verified）
2. ⑤-06 — EXEMPT 按 S20.0b 拆分后文件清单重刷（`game_app/` + `treasure_bevy.rs`/`bevy_app.rs` + §实测记录 3 列表，逐条带理由）；与 ⑤-05 同 commit
3. ⑤-01 / ⑤-02 / ⑤-03 / ⑤-07 — 不单独改码，由 ⑤-06 的 EXEMPT 重刷覆盖（验收 = gate 12 修后 0 未豁免 CJK 命中）

**中优先（渲染数据化 + 单源归一）**
4. ③-01 / ③-02 — FX 触发坐标 → `scenes.json`（`vs.fx_hit` / `main_map.fx_coin`）+ `input.rs` 读数据；删 `mod.rs:281/283` 两常量
5. ⑥-01（引擎侧）/ ③-17 / ③-18 — `canvas.rs:40/47` + `treasure_bevy.rs:230/233/321/322` 裸 540.0/960.0 改 `LOGICAL_W_I32/LOGICAL_H_I32` 半值派生 + doccheck2 扩 grep 断言（ti-engine 禁裸 540/960）；⑥-05 的 engine 侧 LOGICAL 常量顺手改派生（同批，不立项）
6. ⑥-02 — 图标默认 48 单源（`ui_icons.json` 顶层 `default_size` 或 theme 键），`chrome.rs:676` / `scene_render.rs:337` / `shell.rs:528` / `icons.rs:14` 四口归一
7. ⑥-03 — 抽 `chrome::layout_icon_label` 单一函数，`deck_button` / `render_hud_button` 双调用点收口
8. ⑥-07 — `select_role.rs:136` `0..5` → `0..ROLE_COUNT` + doccheck2 扩 render 循环字面量断言（补 EXP-073 盲区）
9. ①-04 — 翻页亮度 0.6/0.4 登记 `gate_data.json ui_motion` + `10_ui_redesign.md §7` + `check_ui_redesign_tokens` 扩断言（第 5/6 参数）
10. ④-05 / ①-32 — 星空表 + 净空参数数据化（`scenes.json <scene>.ambient_art.stars`；index/vs 两场景）
11. ③-13 / ③-14 — `select_map.locked` / `select_map.stars` 几何参数组入 scenes.json + 代码读键
12. ②-07 / ③-15 — `message.body` clip_reserve/行高 与 `saveload.list.summary_dy` 入 scenes.json

**低优先（命名常量抽取 / 死代码清理，机械改动，可批量一次编译）**
13. ①-13 — Bevy 壳底色 `Color::srgb(0.1,0.07,0.05)` 改 `rgb565_to_color(theme.parch_dark)`
14. ①-23 / ①-24 / ①-25 / ①-26 / ①-28 / ①-29 / ①-30 / ①-06 — shell/generic/primitives 内联视觉数值抽命名常量（550/450、120/880、波浪 4/3/1、32/24、350/650、clamp(2,256)、0.18/36、TOAST_TICKS 登记）；⑥-12 与 ①-29 同批（`clamp_grad_steps` 共享函数）
15. ①-10 — `assets.rs:160` 多帧 5 帧假设改 manifest 驱动帧数
16. ⑥-11 — seed 42 三处归一（壳内合一 + doccheck2 对账 `full_run.json.seed`）
17. ⑥-04 — 删 `scene_render.rs known_tokens` 死函数（零消费方）
18. ⑦-10 / ⑦-11 — 死导出/死字段清理批：`chrome::scene_shell/scene_base/scene_frame`（零调用删）+ `menu_item/bar_slot` 转发收敛 + `scene_cache` 字段与 `.Map` 注入死链删除
19. ③-16 — `index.rs:95`「status」键并入 `chrome::keys` + 删无效 `status.x + 2` 传参（hint_row 忽略 x）
20. ③-18 / ⑥-01 引擎侧其余点 — 与第 5 项同批（勿拆两次改同文件）

> 修复批边界提示：第 13-20 项全部为「无像素语义变化」的机械重构（命名/单源/删死码），可一次落完一次编译（硬规则 8）；第 4/9/10/11/12 项改 scenes.json 须同 commit 跑 `doccheck2`（8pt/键表/N8 耦合）+ `ui_assets.py verify`；⑤ 批只动 `scripts/check_hardcode.py` + EXEMPT 表 + fixture，不动业务码。全部修复后 gate 23 项全绿方可回台账。

## 需人裁决 4 条（列分歧点，不阻塞本轮）

- ①-01：9 分区→令牌表归属（balance vs terrain vs 代码留置）；S21.5 节点图片化后该表消费面变化，建议 S21.9 出图验收时拍板。
- ①-12：`gold_glow`/`gold_bright` 同值（0x4E3D）：合并令牌（37→36，同步 by_name/theme.json/§7 表/doccheck2）vs 保留别名 + 显式 alias 标注（`check_ui_redesign_tokens` 扩「同值令牌对须标注」断言）。
- ④-04：14 枚代码矢量图标图片化与否（现状 = `10_ui_redesign §5` 文档化「零外部资产」决策；图片化 = 资产批次 + 注册表 + §5.1 契约 + 原语闭集门禁联动，超本主题靶点）。
- ⑤-04：窗口标题 CJK 面量走 `strings.json app.window_title`（i18n 清零）vs app 外壳 EXEMPT（spike 壳标题宜直接豁免）；随 ⑤-06 批内逐条拍板。

## 误报 28 条（不修；前 10 条见回报，全量编号见顶部统计）

核心误报理据三条（覆盖 28 条中的 20+ 条）：
1. **Theme 权威链误读**（①-02/03/05、④-02/03、⑥-08、⑥-09）：`09_render_framework §5.1` 权威 = Rust `Theme::DEFAULT`，theme.json 是测试锁定恒等镜像，COLOR_*/Theme::DEFAULT 使用点无漂移路径；④-02/03 的「双后端不一致」与 §5.1 明文契约（原生 48px 几何置 rect 左上、双端同构）冲突，现数据（size=64）未触发唯一理论分叉。
2. **契约 4 组件内几何豁免**（①-15~22、②-05/06、③-12）：`08_render_arch` 契约 4 明文「几何常量如 wood_frame 的 12px 外框宽属实现，可写死在 chrome.rs」；场景矩形经参数传入，组件内部样式参数非场景级坐标。
3. **crash-safe 兜底 / 语义 guard / 命名常量口径**（①-07/08/09/11、②-02/03）：数据源缺失兜底值、退化尺寸守护、状态条件画法（契约 3 视图层本职）均不属「渲染决策数值写死」；①-09 FX_SEED 为命名 pub const + doc = 满足 02_conventions「或命名常量」。
其余 8 条（③-12 外的逐条理由）见正文逐条裁决。
