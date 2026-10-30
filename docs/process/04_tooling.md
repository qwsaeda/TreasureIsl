# 工具链与脚本化（Tooling）

**原则**：凡是「重复、机械、跨文件」的活，都要有脚本；凡是人肉复制粘贴的地方，
先问「能不能 codegen」。

## 1. 现有工具一览

| 工具 | 语言 | 作用 | 幂等 |
|---|---|---|---|
| `scripts/gate.ps1` | PowerShell | 全库门禁（fmt/clippy/test/build×3/占位/unwrap/台账 + doccheck） | — |
| `scripts/build_wasm.ps1` | PowerShell | 产出 `web/`（wasm + gl.js + 资源） | 是 |
| `scripts/smoke.ps1` | PowerShell | native + wasm 两端冒烟 | — |
| `legacy/scripts/classify.py` | Python | 函数分类打标 → `ref/classify.json` | 是 |
| `legacy/scripts/classify_assets.py` | Python | 资产分类 → `ref/asset_classification.json` | 是 |
| `legacy/scripts/pixel_upscale.py` / `waifu2x_upscale.py` | Python | 批量超分 | 是 |
| `legacy/scripts/vectorize_ui.py` | Python | UI 矢量化 | 是 |
| `scripts/pack_hd_assets.py` / `verify_hd_manifest.py` | Python | 多分辨率打包 + 校验 | 是 |
| `crates/ti-tools` 各 bin | Rust | unpack / map2json / build_assets / controls / classify / specskel / doccheck | 是 |
| `crates/ti-devmeta` | Rust | 平台共享层（台账解析 + 静态 HTTP，纯 std；`ledger.rs` 上提自 ti-tools 的单实现） | 库（无 bin） |
| `crates/ti-console` | Rust | **TI Dev Console 平台本体**（S32 开发管理平台；静态壳 + `/api/health` + 单实例锁 + 命令白名单；S32.1 骨架） | 是（`cargo run -p ti-console`） |
| `scripts/ui_assets.py` | Python | UI 资产**单资产**入口（add/build/verify/rekey/normalize） | 是 |
| `scripts/ui_asset_batch.py` | Python | UI 资产**批量**入口（register/generate/install/build/check/sheet，规格驱动 + 抠图可行性预检） | 是 |
| `scripts/ui_layout_sheet.py` | Python | UI **版面线图**（规格驱动：底图 + 逐元素矩形 + 环岛示意 ⇒ 坐标表线图；设计稿评审 / 出图验收对照） | 是 |
 | `scripts/ui_icons.py` | Python | **UI 图标白蒙版生成器**（S21.14：读 `assets/ui_batches/icon_v1.json` 48 原生几何 → PIL 放大 512 白蒙版 `assets/ui/icon_*.png`；`--check` 字节幂等 + 注册表对账） | 是 |
 | `scripts/ui_islands.py` | Python | **岛屿剪影资产生成器**（S26.2：`terrain.json` 陆格掩码 → 3 枚暖沙岛屿剪影 PNG 994×826 抗锯齿+礁光晕+投影 → `assets/ui/island_m0*.png` + `ui_manifest.json` kind=island；运行时资产优先 blit 缺图回退；`--check` 只读幂等） | 否 |
| `scripts/map_pack.py` | Python | **地图包工具**（S31 / `21_map_package_spec.md`：一张图 = 一个自包含可分发包。`pack` 出一包 / `verify` 校验（sha256+31 格不变量，违规非 0）/ `install` 装上 / `uninstall` / `list` / `check` 漂移对账） | 否 |
| `scripts/task_close.py` | Python | **任务收口一条命令**（跑判据 → 写证据 → 台账行级置 verified → 同步 release 计数 → 三方对账复验；判据非 0 不改台账） | 是 |
| `scripts/exp_log.py` | Python | **EXP 登记一条命令**（分卷正文 + 索引行 + 分卷条数列 3 处改齐 + 查重 + doccheck 复验） | 是 |
| `scripts/doc_numbers.py` | Python | **门禁项数 / 台账计数实测回写**（8 处声称点一次改齐；`--fix` 回写） | 是 |
| `scripts/layout_spec_check.py` | Python | **UI 版面规格三面对账**（规格表 ↔ scenes.json ↔ manifest：坐标 + 8pt + N8 往返） | 是 |
| `scripts/check_ps_scripts.ps1` | PowerShell | **PS 入口体检**（BOM + ParseFile 语法自检，EXP-109） | 是 |

> 注：两个 HD manifest 脚本**分工不同，勿混用**——`check_hd_manifest.py` 是**门禁项**（校验 `hd_f1_manifest.json` 与 `assets_hd_pack/1x/ui` 目录一致，EXP-048/060）；`verify_hd_manifest.py` 是 **S7.2 验收辅助**（校验 `manifest_hd.json` 与 `assets_hd` 产物 PNG-32/无缺失/哈希匹配）。二者查不同 manifest 文件、不同产物阶段。

新增工具**必须**：幂等 + 提供 `--check` 只读模式 + 非 0 退出码，并登记本表 +
[`docs/README.md`](../README.md)；**详细用法与流水线组合见
[`14_tooling_catalog.md`](14_tooling_catalog.md)（工具唯一事实源）**。

新增工具**必须**：幂等 + 提供 `--check` 只读模式 + 非 0 退出码，并登记本表 +
[`docs/README.md`](../README.md)。

## 2. 工具链（已建 + 待建）

### 已建（S8.2 落地，进 gate 第 10 项）

| 工具 | 替代什么手工活 | 幂等 / --check |
|---|---|---|
| `scripts/gen_layout.py` | 把 `ref/controls.json` 2041 条控件记录转成 `assets/data/controls.json`（控件层数据源） | 连跑两次逐字节一致；`--check` 只读比对 |
| `scripts/validate_data.py` | 手查 JSON 字段与数值域 | `--check` 校验（FIND-201 统一入口，并入原 legacy 脚本 + V1/V2 场景标量兜底）：必填字段、概率域、价格/权重>0、strings 覆盖每个 name_key（无孤儿）、scenes 覆盖 7 场景、controls record_total 与 legacy/ref 一致、main_map.grid 三键正整数、全场景 extra 标量键正整数 |
| `scripts/check_hardcode.py` | 人眼找 `.rs` 里的硬编码数值/中文（`02_conventions.md` §1） | `--warn` 迁移期只报告（历史时点）；`--list` 按文件分类汇总；EXEMPT 豁免带理由（oracle/CLI/配置键）；当前严格模式 0 命中即绿 |
| `crates/ti-tools --bin doccheck` | 跨文档 ID/数值漂移 + 数据文件一致性（对抗审计固化，gate 第 9 项） | 数值断言 4 条：UI 场景数=20、控件记录=2041、LCG hex=0x343FD（非 0x3443D）、save.magic=20011591（非 20125007）；扫描 docs + AGENTS.md + plan + assets/data/*.json |

| `scripts/check_oracle_boundary.py` | S8 定位边界：shell 层（game_app/bins/scenes/widgets/i18n）禁 `oracle_*` 函数调用（oracle 仅 rules 内部参照物，类型重导出允许） | `--check` 违规即非 0（gate 第 11 项）；`--report` 列命中明细恒 0；幂等只读（EXP-054/EXP-057） |
| `scripts/migrate_strings.py` | 把 `.rs` 中命中 `KEY_HINTS` 的中文字面量批量迁到 `strings.json` + 改写调用点为 `sget(key)` | `--report` 只列不写；`--apply` 幂等（已迁过的再跑零改动）；新增文案只需在 KEY_HINTS 加一条再重跑 |
| `scripts/check_ledger_reality.py` | 台账三方对账（防虚报，S10.0）：ledger `status: verified` 行 ↔ evidence 文件（存在+非空，支持 `+` 多文件）↔ 代码符号（`SYMBOL_EXPECTATIONS` 登记的 DoD 产物在 `crates/*/src` grep）三方一致 | 全绿 0 退出码；任一虚报 FAIL 列命中（gate 第 14 项）；只读幂等；新增任务若 DoD 声明代码产物，往 `SYMBOL_EXPECTATIONS` 登记（EXP-058） |

| `scripts/audit_code.py` | 代码结构审计（`docs/style.md` §5.12 的执行手段）：文件规模（>900 行）/ 函数长度（>140 行）/ 重复模式（同一骨架 ≥N 次） | `--check` 只读检查、违规 ⇒ 非 0 退出码；`--report` 人读报告（永远 0 退出）；`--update-baseline` 写棘轮基线；阈值与基线豁免 = `scripts/code_budget.json`（每条豁免带整改任务号） |
| `scripts/check_code_standards.py` | 代码标准 4 项机器检查（`16_code_standards.md` 执行矩阵：CS-1 资产路径单源 / CS-2 文件 `//!` 职责声明 / CS-3 参数禁单字母 / CS-4 场景层窄接缝禁构造会话；规模维度归 `audit_code.py`，不重复） | `--check` 门禁判定（违例超基线 ⇒ 非 0）；`--report` 明细（恒 0）；`--update-baseline` 写豁免（棘轮只收紧）；基线 = `scripts/process_ratchet.json::code_standards`（每条带 `reason`+`fix_in`） |
| `scripts/wasm_browser_e2e.py` | Bevy wasm 浏览器端 e2e（Playwright 无头 Chromium 3 层：L1 实例化 / L2 渲染 / L3 交互，与 native `e2e_playthrough` 互补） | `--check` 只读打印判据（0 新产物）；默认断言 3 层全绿，失败非 0；需 `serve` 静态服务在跑 |
| `scripts/web_playtest.cjs` | 浏览器真运行时试玩（S16.1：Node + Playwright CDP 按键驱动 菜单→选人→选图→主地图 + console/墨点/场景流转断言） | `node scripts/web_playtest.cjs [--url ...]`；失败非 0；需 `serve` 在跑 + node playwright |
| `scripts/playtest_loop.py` | 试玩闭环 4 报告聚合（S16.2：fuzz/full_run/render_invariants/web_playtest + 归因三分类清单；`08_playtest_loop.md` 执行面） | `--check` 只读打印判据（0 新产物）；全 0 失败 = 0 退出；任一失败非 0；web 缺 serve/node = SKIP |
| `scripts/build_apk.sh` | Android APK 构建（S2.6/S18.8d：NDK 交叉编译 .so + APK 组装 + zipalign + SignV2 签名；装机 adb install + 资产 push） | 默认 x86_64；NDK 硬编码 /tmp/opencode/android-ndk-r25b |
| `scripts/publish_web.ps1` | 一键发布（S16.3：gate 全绿前置 + build_wasm + 本地静态断言 + release.md 发布历史 + gh-pages 线上复跑 web_playtest；线上不可达 = 降级登记不静默绕过） | `-Check` 只读前置体检；全链路失败非 0 |
> **gate 项数**（2026-10 扩展，随对抗审计固化）：第 9 项 `doccheck`（ID/数值漂移 + 数据文件一致性）→
> 第 10 项「数据表门禁」= `validate_data`（须绿）+ `gen_layout --check`（幂等绿）+ `check_hardcode`（严格 0 命中）。
> 历史时点 gate 为 8 项（plan-s0-s2/s3/s4 文档保留「当时 8 项」注，勿直接改写）。

> **对抗审计→门禁固化流程**（第 1-5 轮经验，EXP-048）：凡审计中发现「重复、可机械判定」的
> 数值/引用/一致性检查，**必须**固化成 `doccheck` 数值断言或 gate 脚本项，否则下一轮必然复发。
> 例：2041/0x343FD/save.magic 三条断言即来自第 4-5 轮审计反复出现的跨文档漂移。
> **gate 第 11 项** = `check_oracle_boundary.py --check`（S8.8：shell 层 0 个 `oracle_*` 函数调用；oracle 仅作 rules 内部类型/几何参照物，EXP-057）。
> **gate 第 14 项** = `check_ledger_reality.py`（S10.0：台账 verified ↔ evidence ↔ 代码符号三方对账，防子代理虚报，EXP-058；S9.5 commands.rs 虚报教训）。

### 待建

| 工具 | 替代什么手工活 | 判据 |
|---|---|---|
| `scripts/scaffold.py` | 手写新场景/控件/模块骨架（套 [`03_templates.md`](03_templates.md)） | `scaffold.py scene --name foo` 生成可编译骨架；`--check` 校验已有文件符合模板 |
| `cargo run -p ti-tools --bin datacheck` | 校验 data 表能反序列化 + 数值合法域（Rust 侧，与 Python 互补） | 单测 + 可跑 |

**新增门禁项（gate 第 10 项，S8.2 已落地）**：`validate_data.py` + `gen_layout.py --check`
+ `check_hardcode.py` 全绿才允许置 verified——这样「数值非法」「控件表与来源不一致」
「硬编码残留」会被机器拦住，不靠自觉。

## 3. codegen 原则

1. **生成物入库、生成器入库**：`.json` / `.rs` 产物提交进仓库，生成脚本也提交；
   两者一致性由 `--check` 保证；
2. **生成物顶部标注**：`// @generated by scripts/xxx.py — 勿手改`；
3. **不生成能手写清楚的**：只有「模板高度重复 + 手写必错」才 codegen（如 2041 条控件表），
   业务逻辑永远手写 + 单测；
4. **生成器自身要测试**：至少一个「跑两次产物一致」的幂等测试。

## 4. LLM 与脚本的边界（呼应 [`00_index.md`](00_index.md) §分工表）

| 场景 | 做法 |
|---|---|
| 新增 1 个场景 | LLM 手写（套模板），不 codegen |
| 新增 1 条控件类型 | LLM 手写 `from_f5` 映射 + 单测 |
| 导入 2041 条控件记录 | **脚本** `gen_layout.py` 批量转表 |
| 迁移硬编码中文 → i18n key | **脚本** `migrate_strings.py`（人只给一次 KEY_HINTS 映射表，批量自动改写 + 自动写 strings.json） |
| 平衡性调参 | 改 `data/*.json`（人或 LLM 改 JSON，**不改代码**） |
| 批量重构（如 21 个重复方法） | LLM 出方案 → 脚本或 codegen 落地 → 单测兜底 |
| 全库门禁 | **脚本** `gate.ps1` |

## 5. 幂等要求（模板化脚本的硬门槛）

- 同一输入连续跑两次，**产物逐字节一致**；
- `--check` 模式：只比对「当前产物 == 期望产物」，不一致非 0，**不写文件**；
- 不依赖环境变量/时钟/随机，除非显式传参；
- 临时文件写 `tempfile` 并清理，不污染仓库。

## 6. Agnes 生图（`scripts/agnes_gen.py`）

**用途**：UI 效果图/概念图生成（S18.7 HD 贴图管线前置；不入库密钥）。

**环境变量**：`AGNES_API_KEY`（必填；不写入脚本/不入库）。

**用法**：

```sh
# 文生图（默认 2K + 9:16 竖屏 = 1472×2624）
python scripts/agnes_gen.py --prompt "..." --out evidence/ui_concept/index_v3.png

# 图生图（参考图 URL）
python scripts/agnes_gen.py --prompt "..." --out mockup.png --image https://example.com/ref.png

# 尺寸/比例
python scripts/agnes_gen.py --prompt "..." --out mockup.png --size 2K --ratio 9:16
```

**模型**：`agnes-image-2.5-flash`（当前全部输出分辨率免费）。

**尺寸档位**：

| ratio | 1K | 2K | 3K | 4K |
|---|---|---|---|---|
| `1:1` | 1024² | 2048² | 3072² | 4096² |
| `9:16` | 736×1312 | 1472×2624 | 2208×3936 | 2944×5248 |
| `16:9` | 1312×736 | 2624×1472 | 3936×2208 | 5248×2944 |

**API**：`POST https://apihub.agnes-ai.com/v1/images/generations`（OpenAI 兼容格式）。

**注意**：
- 输出 = URL（默认）或 Base64（`return_base64: true`）；
- 图生图 = `extra_body.image` 数组（URL 或 Data URI）；
- 超时建议 60–300s（脚本默认 300s）。

## 7. Agnes 视频（`scripts/agnes_video.py`）

**用途**：教程动效 / 概念演示视频生成（异步任务：创建 → 轮询 → 下载）。

**环境变量**：`AGNES_API_KEY`（同生图）。

**用法**：

```sh
# 文生视频（默认 5s + 16:9）
python scripts/agnes_video.py --prompt "..." --out evidence/tutorial/demo.mp4

# 自定义时长 + 画幅
python scripts/agnes_video.py --prompt "..." --out demo.mp4 --seconds 8 --ratio 4:3

# 直接轮询已有任务
python scripts/agnes_video.py --prompt "..." --out demo.mp4 --video-id task_XXX
```

**模型**：`agnes-video-2.5-flash`（720P；当前限时免费）。

**画幅**：

| aspect_ratio | 输出像素 |
|---|---|
| `16:9` | 1280×704 |
| `4:3` | 960×720 |
| `1:1` | 720×720 |
| `9:16` | 720×1280 |

**异步轮询**：创建任务后立即返回 `video_id`；脚本自动轮询（2s 间隔）直至 `status=completed`（典型 3–6 分钟）。

**API**：`POST /v1/videos` + `GET /agnesapi?video_id=...&model_name=agnes-video-2.5-flash`。