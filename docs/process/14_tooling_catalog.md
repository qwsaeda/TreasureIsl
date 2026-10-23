# 14 · 工具台账（scripts/ 全量登记 + 流水线组合）

> 本页是 `scripts/` 目录的**工具唯一事实源**：每个工具的用途 / 用法 / 幂等与只读要求 /
> 门禁归属 / 所属流水线。速查表见 [`04_tooling.md`](04_tooling.md) §1~§2；本页是详细说明。

## 1. 管理规约

### 1.1 工具四类（分类决定登记要求）

| 类 | 判定 | 必须满足 | 目录位置 |
|---|---|---|---|
| **门禁项** | 被 `scripts/gate.ps1` 直接调用 | 幂等 + `--check` 只读 + 违规非 0 退出 + 本页登记 | `scripts/` |
| **长期工具** | 反复执行 / 结论要进文档 / 别人也会用 | 幂等 + `--check` 只读 + 非 0 退出 + 本页登记（含用法） | `scripts/` |
| **一次性工具** | 一次性改写/清理（迁移、清库） | 必须标注「一次性」+ 用法 + 不进门禁 | `scripts/`（用完即删）或 `scripts/tmp/` |
| **取证工具** | 只服务 `legacy/` 逆向取证 | 必须标注「不进产品代码」+ 用法 | `scripts/` 或 `legacy/scripts/` |

> **落地位置是硬规则**（`doccheck2` 第 27 项机器判定）：
> 正式工具落 `scripts/`；一次性改写脚本落 `scripts/tmp/`（**用完即删**）；
> **禁止写到仓库外**（系统临时目录 / 桌面 / 任何仓库外路径）—— 仓库外的脚本
> 绕开台账登记与门禁对账，等于没进项目。
> 仓库内（`scripts/`、`legacy/` 之外）**不得散落** `.py` / `.ps1`。

### 1.2 新增工具流程（5 步，缺一不可）

1. **放 `scripts/`**（不许散落在别处；临时目录只放"一次性改写"脚本）；
2. **满足工具契约**：幂等（连跑两次结果一致）+ 提供 `--check` 只读模式 + 违规时**非 0 退出码**
   （`04_tooling.md` §1）；
3. **本页登记**：用途 / 用法（可直接复制的命令）/ 幂等与退出码 / 所属流水线 / 是否门禁项；
4. **速查表登记**：`04_tooling.md` §1 表 + `docs/README.md` scripts 表；
5. **自动对账**：`doccheck2` 第 26 项 `check_tooling_catalog`（新增脚本必须在本页出现 ⇒ 漏登记即 FAIL）
   + 第 27 项 `check_tooling_contract`（落地位置 / 表格行 / 登记要素 / 只读模式 + 非 0 退出）。

### 1.3 临时脚本纪律（EXP-113）

- 临时脚本**只允许**做「一次性改写」（改完仓库即删），且**必须落 `scripts/tmp/`**；
- **禁止写到仓库外**（系统临时目录 / 桌面 / 任何仓库外路径）——那等于绕开登记与对账；
- 凡是需要**反复跑 / 结论进文档 / 别人也会跑**的 ⇒ **必须**提升为 `scripts/` 工具并按 §1.2 登记；
- 缺工具的代价是实测过的：同一逻辑被重复写 2~4 遍（修的是副本不是原件）⇒
  **写规范时同步问一句「这条规范要人反复做吗？⇒ 那它得先有脚本」**。

### 1.4 工具契约（4 条，机器可判定）

> `doccheck2` 第 27 项 `check_tooling_contract` 逐条判定（存量豁免见棘轮基线
> `scripts/tooling_contract_baseline.json`，每条豁免带理由 + 整改任务号）。

| # | 契约 | 判定方式 | 违反后果 |
|---|---|---|---|
| 1 | **落地位置** | 正式工具在 `scripts/`；一次性脚本在 `scripts/tmp/` 且用完即删；仓库内无散落 `.py`/`.ps1` | doccheck2 FAIL |
| 2 | **台账登记 = 表格行** | `scripts/x.py` 必须在 §3.1~§3.4 某张表里有 `\| \`x.py\` \|` 行（§3 第 26 项只查「出现过」） | doccheck2 FAIL |
| 3 | **登记要素齐全** | 该行须声明「幂等/只读」与退出码语义（「非 0 退出」或等价表述） | doccheck2 FAIL |
| 4 | **只读模式 + 非 0 退出** | 新增工具（不在棘轮基线内）源码须含 `--check`/`--verify`/`--report` 之一，且违规时非 0 退出 | doccheck2 FAIL |

### 1.5 组合优先（不新增工具的优先序）

动手写脚本前，按此顺序找解法，**命中即停**：

1. **现有工具能直接跑** → 用它（`scripts/playbook.ps1 -List` 列全部命令）；
2. **现有工具能组合出结果** → 组合（例：`ui_assets.py verify` + `ui_shot_probe.py --crop` + `ui_preview.ps1`）；
3. **现有工具缺一个能力** → **补齐那个工具**（`scripts/` 新增 + §1.2 登记），
   **不要**写一次性脚本把它包一层；
4. 以上都不行（真的一次性改写） → `scripts/tmp/` 写、用完即删。

> **反面教材（EXP-113 + S21 实证）**：`ui_assets.py` 只有单资产入口，于是「批量登记 +
> 并行生图 + 抠图」被写成临时脚本搬到**仓库外临时目录** ⇒ 台账无登记、门禁查不到、
> 事后无法复用；正解 = 补 `scripts/ui_asset_batch.py` + `assets/ui_batches/*.json` 规格。

## 2. 流水线组合（端到端命令序列）

### 2.1 UI 资产流水线（AI 生图 → 抠透明 → 清单登记 → 出图验收 → 像素取证）

> 单一事实源 = `assets/ui_manifest.json`（清单）；产物 = `assets/ui/`；规范 =
> [`13_ui_asset_pipeline.md`](13_ui_asset_pipeline.md)。

```powershell
# 1) 加资产到清单（先清单后产物）
python scripts/ui_assets.py add --name icon_lamp --role medallion --what "a ship lantern"
# 2) 生成：AI 生图（agnes_gen.py）+ 抠透明 + 落 assets/ui/
python scripts/ui_assets.py build
# 3) 透明度重抠 / 尺寸归一（键色不对时）
python scripts/ui_assets.py rekey --name board
python scripts/ui_assets.py normalize --name board
# 4) 三方校验：清单 ↔ assets/ui/ ↔ scenes.json 引用
python scripts/ui_assets.py verify
# 5) 出图验收（7 屏；main_map 带 --spin=5 出带状态图）
powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1
# 6) 像素取证（缩略图看不清时：区域主色 / 最亮像素 / 裁剪放大）
python scripts/ui_shot_probe.py evidence/bevy_native_main_map.png --region 400,1420,280,100
python scripts/ui_shot_probe.py evidence/bevy_native_main_map.png --crop 300,1000,780,580 --out crop.png
```

底层工具：`agnes_gen.py`（AI 生图，被 `ui_assets.py` 调用）· `pcx_to_png.py`（原版 PCX → PNG）·
`pack_hd_assets.py`（HD 包打包）· `enhance_portraits.py`（立绘增强，一次性）·
`agnes_vision.py`（AI 读图：截图/素材分析）。

### 2.2 环岛赛道流水线（地形 + 平衡表 → 赛道数据 → 规则层 → 节奏实测 → 出图）

> 数据源 = `assets/data/terrain.json` + `assets/data/balance.json`；产物 = `assets/data/board.json`；
> 业务规格 = [`../business/progression.md`](../business/progression.md) §环岛赛道 / §落格节奏 / §跨图难度阶梯。

```powershell
# 1) 生成赛道数据（环岛轮廓追踪 + 落格类型按比例分配）
python scripts/extract_board.py
python scripts/extract_board.py --check          # 幂等校验（与脚本输出逐字节一致）
# 2) 节奏 / 难度阶梯实测（业务规格里的数字来源；balance 一改就重跑）
python scripts/board_measure.py                   # 落格节奏表
python scripts/board_measure.py --ladder          # 跨图难度阶梯表（含 BOSS 战模拟）
python scripts/board_measure.py --check           # 规格与 balance 一致性（漂移 ⇒ 非 0）
# 3) 规则层不变量
cargo test -p ti-core --test difficulty_ladder
# 4) 出图验收
powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1 main_map
```

### 2.3 门禁与提交流水线（改完 → 门禁 → 台账 → 推送）

```powershell
# 1) 常用命令一键入口（列出全部 16 条）
powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 -List
# 2) 编译期三件套（格式 / 静态 / 单测）
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
# 3) 结构审计（文件规模 / 函数长度 / 重复模式，棘轮基线）
python scripts/audit_code.py --check
# 4) 全门禁（23 项；含上面三件套 + 文档/数据/资产/结构/渲染）
powershell -ExecutionPolicy Bypass -File scripts/gate.ps1
# 5) 台账三方对账（verified ↔ evidence ↔ 代码符号）
python scripts/check_ledger_reality.py
# 6) 提交（证据写入 evidence/<id>.log，须含该任务自身判据）
```

### 2.4 逆向取证流水线（`legacy/` 参照物）

```powershell
python scripts/pcx_ref.py ...      # PCX 参照解码（与实现对拍）
python scripts/pe_rva.py ...       # PE RVA / 地址查询
python scripts/rgb565.py ...       # RGB565 换算
```

⇒ 结论写 `legacy/docs/facts/<分卷>.md`（**必须有脚本证据**；推测一律标注「推测」）。

### 2.5 交付与运行流水线

```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1   # wasm 构建（→ web/）
powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1        # native + wasm 冒烟
powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Check   # 开轮必读清单
```

## 3. 逐工具台账

### 3.1 门禁项（被 `gate.ps1` 调用）

| 工具 | 用途 | 用法 | 幂等 / 退出码 |
|---|---|---|---|
| `gate.ps1` | 全门禁 23 项（编译/测试/文档/数据/结构/渲染/台账） | `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` | 幂等；失败项数 > 0 ⇒ 退出 1 |
| `validate_data.py` | 数据表校验（balance/strings/scenes/controls 必填字段、值域、孤儿 key、grid 三键） | `python scripts/validate_data.py` | 幂等；违规非 0 |
| `check_hardcode.py` | 硬编码扫描（数值/中文面量） | `python scripts/check_hardcode.py`（严格）/ `--warn` / `--list` / `--selftest`（STR_LIT 全捕获回归自检，飞轮 R1 ⑤-05 起 CJK 通道对全字面量生效） | 幂等；严格模式命中即非 0；`--selftest` 断言失败即非 0 |
| `check_ledger_reality.py` | 台账三方对账（verified ↔ evidence 文件 ↔ 代码符号） | `python scripts/check_ledger_reality.py` | 幂等只读；虚报即非 0 |
| `check_oracle_boundary.py` | oracle 边界（shell 层禁 `oracle_*` 调用） | `python scripts/check_oracle_boundary.py --check` / `--report` | 幂等只读；违规非 0 |
| `check_forward_closed.py` | 业务闭环 C1~C4（零逆向引用 / 单一 oracle 源 / 根目录清空 / 无重复 crate） | `python scripts/check_forward_closed.py --check` | 幂等只读；违规非 0 |
| `check_module_contracts.py` | 模块并行契约（包白名单互斥 / 契约锚点 / 跨 crate 可见面） | `python scripts/check_module_contracts.py --check` | 幂等只读；违规非 0（WARN 级除外） |
| `check_scene_narrative.py` | UI 场景叙事约束 DSL v3（mood / 重心 / 节奏权重 / HUD 上限 / 死区 / focus_anchor 对账） | `python scripts/check_scene_narrative.py` | 幂等只读；违规非 0 |
| `check_data_source.py` | 数据源门禁（`legacy/data/*` 相对 HEAD 变动 ⇒ 强制跑渲染快照） | `python scripts/check_data_source.py` | 幂等只读 |
| `check_data_driven.py` | 数据驱动（渲染层不得认业务语义） | `python scripts/check_data_driven.py` | 幂等只读；违规非 0 |
| `check_hd_manifest.py` | HD manifest ↔ `assets_hd_pack/1x/ui` 目录一致（EXP-048/060） | `python scripts/check_hd_manifest.py --check` / `--report` | 幂等只读；漂移非 0 |
| `audit_code.py` | **代码结构**（文件 >900 行 / 函数 >140 行 / 重复模式 ≥6 次；棘轮基线 `scripts/code_budget.json`） | `python scripts/audit_code.py --check` / `--report` / `--update-baseline` | 幂等只读；新增违规非 0 |
| `flywheel_readme.ps1` | 开轮必读清单（6 条必读 + gate 项数 + doccheck2 断言数） | `powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Check` | 幂等只读；不符非 0 |

### 3.2 长期工具（反复执行 / 结论入文档）

| 工具 | 用途 | 用法 | 幂等 / 退出码 |
|---|---|---|---|
| `extract_board.py` | **环岛赛道数据生成**（地形陆格掩码 → Moore 轮廓 → 落格类型按比例分配） | `python scripts/extract_board.py` / `--check` | 幂等（`--check` 逐字节比对）；不一致非 0 |
| `board_measure.py` | **赛道节奏 / 跨图难度阶梯实测**（规格数字来源 + 漂移检查） | `python scripts/board_measure.py` / `--ladder` / `--check` / `--json` | 幂等只读；规格漂移非 0 |
| `code_outline.py` | **源结构清单**（文件 / 顶层项 / 方法 + 行范围 ⇒ 拆分前的模块边界与函数清单） | `python scripts/code_outline.py <file.rs>` / `--crate ti-app` / `--json` / `--check` | 幂等只读；孤立顶层 fn 非 0 |
| `ui_shot_probe.py` | **出图取样**（区域主色 / 最亮像素 / 裁剪放大 ⇒ UI 出图验收的像素证据） | `python scripts/ui_shot_probe.py <png> --region x,y,w,h` / `--crop x,y,w,h --out` / `--check` | 幂等只读；空帧非 0 |
| `ui_assets.py` | **UI 资产清单驱动**（add/build/verify/rekey/normalize） | `python scripts/ui_assets.py add --name <n> --role <r> --what "<w>"` / `build` / `verify` / `rekey --name <n>` / `normalize --name <n>` | build 幂等（已生成不重做）；verify 不一致非 0 |
| `ui_asset_batch.py` | **UI 资产批量工具**（一组资产一次做完：规格登记 → 并行生图 → 抠图 → 审阅拼版；`ui_assets.py` 的批量补位，扣图/生图逻辑**复用** `ui_assets.py` 不重复实现；含**抠图可行性预检**：luma 需纯黑底 / magenta 需品红底，底色不对时报出来而不是事后看图才发现） | `python scripts/ui_asset_batch.py register\|generate\|install\|build\|check\|sheet --spec assets/ui_batches/<batch>.json`（`--check` = check 别名；`generate --workers N --only a,b --force`；`sheet --out <png> --cols N`） | 幂等（register upsert / generate 已有原图跳过 / install 重抠确定）；check 只读，漂移非 0 |
| `ui_layout_sheet.py` | **UI 版面线图**（规格驱动：`assets/ui_batches/*_layout.json` 底图 + 逐元素矩形 + 可选环岛示意 ⇒ 合成「坐标表线图」供版面评审 / 出图验收对照；与 `ui_asset_batch.py sheet`（资产拼版）分工 = 本工具画**版面矩形叠层**，与 `ui_preview.ps1`（真实渲染）分工 = 设计稿线图 vs 落码后渲染） | `python scripts/ui_layout_sheet.py --spec assets/ui_batches/<name>_layout.json`（渲染）/ `--check`（只读校验规格，违规非 0）/ `--out <png> --scale 0.3` | 幂等（同规格 + 同底图 ⇒ 逐像素一致）；`--check` 只读 |
| `task_close.py` | **任务收口一条命令**（跑判据 → 写 `evidence/<id>.log` → 台账**行级**置 verified → 同步 release.md 台账计数 → 跑 `check_ledger_reality.py` 复验；**判据非 0 就不写证据不改台账** ⇒ 虚报防线内置；`docs/process/15_process_ledger.md` 第 1 条流程的载体） | `python scripts/task_close.py --task S21.4 --cmd "cargo test --workspace"`（`--cmd` 可重复）/ `--no-run`（证据已在，只翻台账）/ `--check`（只读体检） | 幂等（同任务重跑 = 重写证据 + 不重复加 rounds）；判据非 0 ⇒ 退出 1 且不改台账 |
| `exp_log.py` | **EXP 条目一条命令登记**（分卷正文 + `index.md` 索引行 + 分卷「条数」列 3 处一次改齐；ID 查重；条数从分卷文件 `### EXP-` 标题**实测**回填而非 +1 猜；收尾跑 `doccheck` 复验） | `python scripts/exp_log.py --volume code --id EXP-<N> --title "标题" --summary "索引摘要" --body "正文"`（`--body-file` 读文件）/ `--check`（只读查重） | 幂等（同 id 再跑 = 查重拦截，不重复写）；查重/对账失败 ⇒ 退出 1 |
| `doc_numbers.py` | **门禁项数 / 台账计数 实测回写**（一改要手抄 8 处的漂移源：parity/git/README/loop/AGENTS/style/playbook/gate 横幅；门禁项数口径 = `gate.ps1` 顶层 `Report` 数（与 doccheck2 同口径），台账计数 = ledger 实测） | `python scripts/doc_numbers.py`（只读体检，漂移即非 0）/ `--fix`（实测回写）/ `--fix --dry-run` | 幂等（回写后再测 0 漂移）；漂移未修 ⇒ 退出 1 |
| `layout_spec_check.py` | **UI 版面规格三面对账**（规格表 → `scenes.json` → `scene_manifest_v3.json`：键↔坐标一致 + 8pt + N8 像素往返 ±1px + N3 焦点面积；规格表是坐标单一事实源，人工校验易漏） | `python scripts/layout_spec_check.py --spec docs/ui-redesign-v3/08_main_map_v4_spec.json [--scene main_map]` | 幂等只读；漂移非 0 |
| `check_ps_scripts.ps1` | **PowerShell 入口体检**（UTF-8 BOM 校验 + `Parser::ParseFile` 真语法自检 ⇒ 拦住「缺 BOM 入口从未真跑起来」，EXP-109） | `powershell -ExecutionPolicy Bypass -File scripts/check_ps_scripts.ps1`（`-Check` 别名） | 幂等只读；不合格非 0 |
| `ui_preview.ps1` | **7 屏出图验收**（Bevy 离屏 → `evidence/bevy_native_<scene>.png` + 缩略图 + 非空帧统计） | `powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1 [scene...]` | 幂等（重出图）；空帧计入失败 |
| `agnes_gen.py` | AI 生图底层（`ui_assets.py build` 调用；需 `AGNES_API_KEY`） | 由 `ui_assets.py` 调用；`python scripts/agnes_gen.py "<prompt>" --out <png>` | 生成非幂等（同 prompt 可复现性由模型决定） |
| `agnes_vision.py` | AI 读图（截图/素材分析 ⇒ 视觉问题取证） | `python scripts/agnes_vision.py <png> "<question>"` | 只读；需 `AGNES_API_KEY` |
| `pcx_to_png.py` | 原版 PCX → PNG（贴图转码） | `python scripts/pcx_to_png.py <in.pcx> <out.png>` | 幂等（解码确定） |
| `pack_hd_assets.py` | HD 资产打包（`assets_hd_pack/`） | `python scripts/pack_hd_assets.py [--no-webp]` | 幂等（同一输入同一产物） |
| `migrate_strings.py` | i18n 迁移（`.rs` 中文面量 → `strings.json`） | `python scripts/migrate_strings.py --report` / `--apply` | `--apply` 幂等（已迁过零改动） |
| `verify_hd_manifest.py` | HD manifest ↔ `assets_hd/` 产物 PNG 一致性**验收辅助**（与 `check_hd_manifest.py` 分工：前者对 `assets_hd_pack/1x/ui` 目录（门禁项），本工具对 `manifest_hd.json` 产物（一次性验收）） | `python scripts/verify_hd_manifest.py` | 幂等只读；不一致非 0 |
| `bevy_cpu_scene_diff.py` | 双后端**现场**像素对拍（CPU 帧缓冲 vs Bevy 离屏，7 场景） | `python scripts/bevy_cpu_scene_diff.py` | 幂等（重出两边再比）；写 `evidence/bevy_cpu_scene_diff.md` |
| `build_wasm.ps1` | wasm 构建（→ `web/`，Bevy 交付壳） | `powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1` | 幂等（重构建覆盖） |
| `smoke.ps1` | native + wasm 冒烟 | `powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1` | 幂等；失败非 0 |
| `playbook.ps1` | 常用命令一键入口（16 条） | `powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 -List` / `<cmd>` | 转发到对应工具 |

### 3.3 一次性工具（标注：不做门禁 / 用完即弃）

| 工具 | 用途 | 用法 |
|---|---|---|
| `enhance_portraits.py` | 角色立绘增强（AI 超分 + 2x 锐化） | `python scripts/enhance_portraits.py`（产物须过 `ui_assets.py verify`） |
| `cleanup_opencode_db.py` | 清理本地 opencode 会话缓存库（归档而非删会话） | `python scripts/cleanup_opencode_db.py`（干跑）/ `--yes`（实执行 + VACUUM）/ `--yes --no-children` |

### 3.4 取证工具（标注：不进产品代码，只服务 `legacy/` 取证）

| 工具 | 用途 | 用法 |
|---|---|---|
| `pcx_ref.py` | PCX 参照解码（与实现对拍，验证 P0-1/P0-4） | `python scripts/pcx_ref.py ...` |
| `pe_rva.py` | PE 文件 RVA / 节区查询（逆向地址定位） | `python scripts/pe_rva.py ...` |
| `rgb565.py` | 8-bit hex ↔ RGB565（R11G5B10）换算 | `python scripts/rgb565.py ...` |

## 4. 门禁项 ↔ 工具对照（排障入口）

`scripts/gate.ps1` 的 23 项按域分组，**每组对应的排障工具**如下（门禁清单本身见
[`../parity.md`](../parity.md) §代码质量门禁）：

| 域 | 门禁项 | 排障工具 |
|---|---|---|
| 编译 / 测试 | 1~4b（fmt / clippy / test / build / feature bin） | `cargo fmt` / `cargo clippy` / `cargo test` |
| 占位 / unwrap | 5~6 | `Select-String` 扫 `TODO|unwrap(`（`check_hardcode.py` 辅助） |
| 台账 / 文档 | 8~9、13~14 | `cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` / `doccheck` / `doccheck2` / `check_ledger_reality.py` |
| 数据 | 10a~10c、20 | `python scripts/validate_data.py` / `gen_layout.py --check` / `check_hardcode.py` / `check_data_source.py` |
| 资产 | 19.5 | `python scripts/ui_assets.py verify` + `ui_preview.ps1` + `ui_shot_probe.py` |
| 结构 | 23（audit_code） | `python scripts/audit_code.py --report` + `code_outline.py --crate <c>` |
| 业务闭环 / 契约 | 15~16 | `check_forward_closed.py --check` / `check_module_contracts.py --check` |
| 飞轮 | 17 | `flywheel_readme.ps1 -Check` |
| 渲染 | 18、19、21、21.5 | `bevy_cpu_scene_diff.py` / `check_scene_narrative.py` / `ti-engine` example `bevy_vs_cpu_pixel` |
| 数值规格 | —（新增项由 `board_measure.py --check` 守） | `python scripts/board_measure.py --check` |
