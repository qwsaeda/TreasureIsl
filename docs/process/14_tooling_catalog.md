# 14 · 工具台账（scripts/ 全量登记 + 流水线组合）

> 本页是 `scripts/` 目录的**工具唯一事实源**：每个工具的用途 / 用法 / 幂等与只读要求 /
> 门禁归属 / 所属流水线。速查表见 [`04_tooling.md`](04_tooling.md) §1~§2；本页是详细说明。

## 1. 管理规约

### 1.1 工具五类（分类决定登记要求）

| 类 | 判定 | 必须满足 | 目录位置 |
|---|---|---|---|
| **门禁项** | 被 `scripts/gate.ps1` 直接调用 | 幂等 + `--check` 只读 + 违规非 0 退出 + 本页登记 | `scripts/` |
| **长期工具** | 反复执行 / 结论要进文档 / 别人也会用 | 幂等 + `--check` 只读 + 非 0 退出 + 本页登记（含用法） | `scripts/` |
| **一次性工具** | 一次性改写/清理（迁移、清库） | 必须标注「一次性」+ 用法 + 不进门禁 | `scripts/`（用完即删）或 `scripts/tmp/` |
| **取证工具** | 只服务 `legacy/` 逆向取证 | 必须标注「不进产品代码」+ 用法 | `scripts/` 或 `legacy/scripts/` |
| **cargo bin / crate 工具**（S32.1 起） | Rust workspace 成员 crate 的可执行入口（`cargo run -p <crate>`；与 `scripts/` 流水线的分工 = 台账解析/HTTP 服务等库化能力 + 平台本体） | 本页 §3.5 登记 + 依赖图断言（`cargo tree -p <crate>` 不含游戏 crate，22 号 §11.2 机器兜底面）+ 根 `Cargo.toml` members 在册 | `crates/<name>/`（`[[bin]]` 或 crate 本体） |

> **落地位置是硬规则**（`doccheck2` 第 27 项机器判定）：
> 正式工具落 `scripts/`；一次性改写脚本落 `scripts/tmp/`（**用完即删**）；
> **禁止写到仓库外**（系统临时目录 / 桌面 / 任何仓库外路径）—— 仓库外的脚本
> 绕开台账登记与门禁对账，等于没进项目。
> 仓库内（`scripts/`、`legacy/` 之外）**不得散落** `.py` / `.ps1`。

### 1.2 新增工具流程（5 步，缺一不可）

1. **放 `scripts/`**（不许散落在别处；临时目录只放"一次性改写"脚本；cargo bin / crate 工具放 `crates/<name>/`，见 §3.5）；
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
| 1 | **落地位置** | 正式工具在 `scripts/`；一次性脚本在 `scripts/tmp/` 且用完即删；cargo bin / crate 工具在 `crates/<name>/`（S32.1 起，§3.5）；仓库内无散落 `.py`/`.ps1` | doccheck2 FAIL |
| 2 | **台账登记 = 表格行** | `scripts/x.py` 必须在 §3.1~§3.4 某张表里有 `\| \`x.py\` \|` 行（§3 第 26 项只查「出现过」）；cargo bin / crate 工具须在 §3.5 登记 | doccheck2 FAIL |
| 3 | **登记要素齐全** | 该行须声明「幂等/只读」与退出码语义（「非 0 退出」或等价表述） | doccheck2 FAIL |
| 4 | **只读模式 + 非 0 退出** | 新增工具（不在棘轮基线内）源码须含 `--check`/`--verify`/`--report` 之一，且违规时非 0 退出；**豁免 = cargo bin / crate 工具类**（§3.5，服务形态 bin：只读/写面语义由 22 号 §4.2 白名单 + §10.3 执行面接管，`check_tooling_contract` 实扫面 = `scripts/` 的 `.py`/`.ps1`，不含 `crates/`） | doccheck2 FAIL |

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
# 4) 全门禁（33 项；含上面三件套 + 文档/数据/资产/结构/渲染）
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

### 2.6 地图打包流水线（S31 标准：一张图 = 一个可分发的自包含包）

> 单一事实源 = `assets/maps/<id>/`（**已装包**：`manifest.json`/`map.json`/`layout.json`/`strings.json`/`assets/`）
> + `dist/maps/<id>.map-pack/`（**可分发件**）。格式与不变量 = [`21_map_package_spec.md`](21_map_package_spec.md)。
> 效果：**加图 / 换图 = 出一包 + 装上（+ 运行期切包），不改 Rust、不手改多份 JSON**。

```powershell
# 1) 出一包（已装包 → 可分发件；带 sha256 清单）
python scripts/map_pack.py pack m01
# 2) 校验包（schema/兼容/资产齐+哈希/31 格不变量 I1~I9 ⇒ 违规非 0；
#    参数面：可分发件/zip 路径，或**裸 id = 已装包目录** `assets/maps/<id>`）
python scripts/map_pack.py verify dist/maps/m01.map-pack
python scripts/map_pack.py verify m01        # 裸 id（平台制作指引面）
# 3) 装上（进 assets/maps/<id>/ + 接线默认包数据/资产登记/文案合并）
python scripts/map_pack.py install dist/maps/m01.map-pack
# 4) 列出已装包（id / version / data_version / 校验结果）
python scripts/map_pack.py list
# 5) 已装态 ↔ 内嵌默认数据漂移对账（只读）
python scripts/map_pack.py check
```

#### 平台地图编辑器导入面（S32.14 ②，平台内置端点非独立脚本）

| 端点 | 用途 | 用法 | 口径 |
|---|---|---|---|
| `POST /api/map/import`（平台「地图」页编辑器） | 环形画布编辑器出描述符 → 免鉴权导入落盘 `assets/maps/<id>/` 四文件 + 同进程 `map_pack.py sync` 回注 + `verify` 终判 I1~I9 | 平台 `http://host:8787/` 地图页「＋新建/编辑地图」→ 画环格/调几何 → 「导入到 assets/maps/」 | 唯一写点（22 号 §10.1 例外）；`id` 白名单 `^[a-z0-9_]{1,16}$` + canonicalize 前缀断言；违规 = `errors` 如实上报不静默绕过；I1~I9 权威判定 = `verify_pack`（前端校验条 = 轻量镜像，内容类 I9 由 verify 权威判）；发布链不变（`pack` 出可分发件） |


## 3. 逐工具台账

### 3.1 门禁项（被 `gate.ps1` 调用）

| 工具 | 用途 | 用法 | 幂等 / 退出码 |
|---|---|---|---|
| `gate.ps1` | 全门禁 33 项（编译/测试/文档/数据/结构/渲染/台账） | `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` | 幂等；失败项数 > 0 ⇒ 退出 1 |
| `validate_data.py` | 数据表校验（balance/strings/scenes/controls/theme 必填字段、值域、孤儿 key、grid 三键）+ **`check_scenes_canonical`**（scenes.json 须 indent=1 逐字节规范形，防整文件 churn，S29.2 §18）+ **`check_asset_registry`**（全类型资产 disk↔registry 双向同步：UI 图=`ui_manifest.json`、字体/音频=`assets/registry.json`，S29.2 §18）+ **`check_version_consistency`**（S25 版本三方对账：`Cargo.toml` `[workspace.package] version` 单源 ↔ `AndroidManifest versionName` ↔ 构建脚本产物名带 x.y.z ↔ `release.md` 版本规划行） | `python scripts/validate_data.py` | 幂等只读；违规非 0（gate 第 10a 项） |
| `check_hardcode.py` | 硬编码扫描（数值/中文面量；`EXEMPT` 带理由豁免：oracle 冻结层 / CLI 输出 / 诊断 eprintln，含 S31.2 `data/map_pack.rs`+`data/table_source.rs` 注入诊断） | `python scripts/check_hardcode.py`（严格）/ `--warn` / `--list` / `--selftest`（STR_LIT 全捕获回归自检，飞轮 R1 ⑤-05 起 CJK 通道对全字面量生效） | 幂等；严格模式命中即非 0；`--selftest` 断言失败即非 0 |
| `check_ledger_reality.py` | 台账三方对账（verified ↔ evidence 文件 ↔ 代码符号） | `python scripts/check_ledger_reality.py` | 幂等只读；虚报即非 0 |
| `check_dod_commands.py` | **DoD 判据命令载体可达性自检**（S46.1 / EXP-170+171 复发防护：扫描 AGENTS.md + docs/**/*.md〔排除 experience/impl/plan-*：叙事卷引反例、历史规格 superseded〕里的判据命令，逐一解析载体——`cargo run -p <crate> --bin <bin>` 查 bin 存在 / `cargo run -p <crate> -- <arg>` 无 `--bin` 且多 bin = 歧义 FAIL / `cargo test -p <crate> --test <name>` 查 test 存在 / `python scripts/x.py`·`pwsh scripts/x.ps1` 查文件存在；`<module>`/单字母泛指 = 占位符跳过；数据源 = `cargo metadata` 权威 bin/test 清单） | `python scripts/check_dod_commands.py`（`--check` 只读别名 / `--json` 机读） | 幂等只读；不可达判据非 0（exit 1）+ 前置缺失（cargo metadata 不可用）exit 2；gate 第 25 项 |
| `studio_tests.py` | **元测试面（S50.2 / CCGS studio-tests 模式借鉴）**：对「门禁断言 / 测试登记 / 工具台账」本身做 QA（测「测代码的测试」）——3 类断言：① doccheck2 触发面双向对账（`doccheck2.rs` `fn check_*` ↔ 注册表 `RUST_CHECKS` 引用 ↔ `gate_data.json` `doccheck2_checks` 条目，缺任一即 FAIL）② `test_manifest.json` 每条 command 首词 ∈ 程序白名单（cargo/python3/pwsh/powershell）+ 在 `console_commands.json` 有 command 映射（缺映射 = 平台面未覆盖；存量未映射进 `studio_tests_waiver.json` 棘轮基线，新增必映射）③ `scripts/*.py/.ps1` 每个文件在 14 号表格行 + 04 速查表 + README 索引 三处均登记（grep 文件名命中，缺任一即 FAIL；存量豁免进棘轮基线 + `scripts/tmp/` 前缀天然豁免） | `python3 scripts/studio_tests.py --check`（只读） | 幂等只读；3 类断言任一 FAIL ⇒ 非 0（exit 1）；前置缺失（源文件不在）⇒ exit 2；gate 第 25b 项（收口档实跑） |
| `check_oracle_boundary.py` | oracle 边界（shell 层禁 `oracle_*` 调用） | `python scripts/check_oracle_boundary.py --check` / `--report` | 幂等只读；违规非 0 |
| `check_forward_closed.py` | 业务闭环 C1~C4（零逆向引用 / 单一 oracle 源 / 根目录清空 / 无重复 crate） | `python scripts/check_forward_closed.py --check` | 幂等只读；违规非 0 |
| `check_module_contracts.py` | 模块并行契约（包白名单互斥 / 契约锚点 / 跨 crate 可见面） | `python scripts/check_module_contracts.py --check` | 幂等只读；违规非 0（WARN 级除外） |
| `check_scene_narrative.py` | UI 场景叙事约束 DSL v3（mood / 重心 / 节奏权重 / HUD 上限 / 死区 / focus_anchor 对账） | `python scripts/check_scene_narrative.py` | 幂等只读；违规非 0 |
| `check_code_standards.py` | 代码标准机器检查（`16_code_standards.md` 执行矩阵前 4 行：CS-1 资产单源 / CS-2 文件职责声明 / CS-3 参数命名 / CS-4 场景窄接缝 + 棘轮基线 `process_ratchet.json::code_standards`） | `python scripts/check_code_standards.py --check`（门禁）/ `--report`（明细）/ `--update-baseline`（写豁免） | 幂等只读；新增违例超基线 ⇒ 退出 1 |
| `check_data_source.py` | 数据源门禁（`legacy/data/*` 相对 HEAD 变动 ⇒ 强制跑渲染快照） | `python scripts/check_data_source.py`（`--check` = 只读门禁别名） | 幂等只读；数据源改动且快照漂移 ⇒ 非 0 退出 |
| `check_data_driven.py` | 数据驱动（渲染层不得认业务语义） | `python scripts/check_data_driven.py` | 幂等只读；违规非 0 |
| `check_hd_manifest.py` | HD manifest ↔ `assets_hd_pack/1x/ui` 目录一致（EXP-048/060） | `python scripts/check_hd_manifest.py --check` / `--report` | 幂等只读；漂移非 0 |
| `audit_code.py` | **代码结构**（文件 >900 行 / 函数 >140 行 / 重复模式 ≥6 次；棘轮基线 `scripts/code_budget.json`） | `python scripts/audit_code.py --check` / `--report` / `--update-baseline` | 幂等只读；新增违规非 0 |
| `flywheel_readme.ps1` | 开轮必读清单（7 条必读〔S51.0 起含开轮协作五步〕 + 本域红线 docs/rules + gate 项数 + doccheck2 断言数） | `powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Check` | 幂等只读；不符非 0 |

### 3.2 长期工具（反复执行 / 结论入文档）

| 工具 | 用途 | 用法 | 幂等 / 退出码 |
|---|---|---|---|
| `extract_board.py` | **环岛赛道数据生成**（地形陆格掩码 → Moore 轮廓 → 落格类型按比例分配） | `python scripts/extract_board.py` / `--check` | 幂等（`--check` 逐字节比对）；不一致非 0 |
| `board_measure.py` | **赛道节奏 / 跨图难度阶梯实测**（规格数字来源 + 漂移检查） | `python scripts/board_measure.py` / `--ladder` / `--check` / `--json` | 幂等只读；规格漂移非 0 |
| `code_outline.py` | **源结构清单**（文件 / 顶层项 / 方法 + 行范围 ⇒ 拆分前的模块边界与函数清单） | `python scripts/code_outline.py <file.rs>` / `--crate ti-app` / `--json` / `--check` | 幂等只读；孤立顶层 fn 非 0 |
| `ui_shot_probe.py` | **出图取样**（区域主色 / 最亮像素 / 裁剪放大 ⇒ UI 出图验收的像素证据） | `python scripts/ui_shot_probe.py <png> --region x,y,w,h` / `--crop x,y,w,h --out` / `--check` | 幂等只读；空帧非 0 |
| `ui_assets.py` | **UI 资产清单驱动**（add/build/verify/rekey/normalize；**原图落点 = `evidence/gen_tmp/raw_<name>.png`**（13 号 §2.2），rekey 旧路径 `ui_concept/raw_*.png` 只读兼容） | `python scripts/ui_assets.py add --name <n> --role <r> --what "<w>"` / `build` / `verify` / `rekey --name <n>` / `normalize --name <n>` | build 幂等（已生成不重做）；verify 不一致非 0 |
| `ui_asset_batch.py` | **UI 资产批量工具**（一组资产一次做完：规格登记 → 并行生图 → 抠图 → 审阅拼版；`ui_assets.py` 的批量补位，扣图/生图逻辑**复用** `ui_assets.py` 不重复实现；含**抠图可行性预检**：luma 需纯黑底 / magenta 需品红底，底色不对时报出来而不是事后看图才发现；**原图落点 = `evidence/gen_tmp/raw_<name>.png`**（13 号 §2.2 临时图统一目录，gitignore，删目录即清理；旧路径 `ui_concept/raw_*.png` 只读兼容） | `python scripts/ui_asset_batch.py register\|generate\|install\|build\|check\|sheet --spec assets/ui_batches/<batch>.json`（`--check` = check 别名；`generate --workers N --only a,b --force`；`sheet --out <png> --cols N`） | 幂等（register upsert / generate 已有原图跳过 / install 重抠确定）；check 只读，漂移非 0 |
| `ui_layout_sheet.py` | **UI 版面线图**（规格驱动：`assets/ui_batches/*_layout.json` 底图 + 逐元素矩形 + 可选环岛示意 ⇒ 合成「坐标表线图」供版面评审 / 出图验收对照；与 `ui_asset_batch.py sheet`（资产拼版）分工 = 本工具画**版面矩形叠层**，与 `ui_preview.ps1`（真实渲染）分工 = 设计稿线图 vs 落码后渲染） | `python scripts/ui_layout_sheet.py --spec assets/ui_batches/<name>_layout.json`（渲染）/ `--check`（只读校验规格，违规非 0）/ `--out <png> --scale 0.3` | 幂等（同规格 + 同底图 ⇒ 逐像素一致）；`--check` 只读 |
| `ui_thumb.py` | **设计稿压缩预览侧车**（`evidence/ui_concept/*.png` → 同名 `.thumb.png` 480px 长边 PNG-8 量化侧车，~53KB/张 vs 原图 4-6MB；平台设计图页卡读侧车、lightbox 读原图；侧车 + `.thumb_meta/` 指纹目录 = 生成物，gitignore 不归档；口径 = 13 号 §4.0） | `python scripts/ui_thumb.py`（全量生成/刷新，源指纹未变 SKIP，幂等）/ `--force`（无视指纹重生成）/ `--check`（只读对账：缺侧车列表 + 非 0 退出，工具契约面） | 幂等（源指纹 + 生成参数都在 `.thumb_meta/`，未变不重做）；PIL 缺失 ⇒ 明确 FAIL 非 0；`--check` 只读 |
| `review_sheet.py` | **迭看图合成一张压缩 contact sheet**（用户 2026-10-31「确认图合成一张压缩让我看」：多张迭看图（引擎渲染/合成稿）缩略拼一张 PNG-8 压缩 sheet，逐张 8MB → 一张 ~1.2MB 一次看全；口径 = 13 号 §2.1a 迭看图归档） | `python scripts/review_sheet.py --out evidence/ui_concept/review_sheet.png <img1> <img2> ...`（`--cell N` 每格长边 / `--cols N`）/ `--check`（只读：输入全在位 + PIL 可用，非 0） | 幂等（同输入同产物）；缺输入 / 缺 PIL ⇒ 非 0 退出；`--check` 只读 |
 | `compose_main_map_v6.py` | **main_map v6「古典羊皮航海图」概念稿组合合成**（元素级生成 + 组合：AI 元素 `evidence/ui_concept/main_map_v6/elements/` 按凸包弧长等分布局贴排 31 格 + 中央岛/舵轮/罗盘/绳路 ⇒ 合成稿 `main_map_v6_composite.png`；单一事实源 = `12_main_map_v6_vs_v2_spec.md` §1） | `python3 scripts/compose_main_map_v6.py`（合成 + 导出布局 JSON `assets/data/main_map_v6_layout.json`；`--seed N` 固定可复现）/ `--check`（只读：元素齐备 + 输出存在，非 0 退出） | 幂等（`--seed` 固定则输出字节一致）；元素缺失 ⇒ 非 0；`--check` 只读 |
 | `compose_vs_v2.py` | **VS 战斗动画 v2 概念稿组合合成**（AI 元素 `assets/ui/vs_v2/` 按阶段构图（clash/duel/result）⇒ 逐阶段合成稿；与引擎渲染 `TI_VS_ANIM_PROTO` 同构图对照；单一事实源 = `12_main_map_v6_vs_v2_spec.md` §2） | `python3 scripts/compose_vs_v2.py`（逐阶段出图；`--seed N` 可复现）/ `--check`（只读：元素齐备，非 0 退出） | 幂等（`--seed` 固定则输出字节一致）；元素缺失 ⇒ 非 0；`--check` 只读 |
 | `key_v6_elements.py` | **v6/vs_v2 元素去白键控**（AI 白底元素抠白 = 透明键写入批次 sidecar；引擎 blit 不再逐像素抠，键控一次定死；单一事实源 = `12_main_map_v6_vs_v2_spec.md` §2.7 资产接线） | `python3 scripts/key_v6_elements.py`（全量键控）/ `--check`（只读：批次元素 + 键控产物在位，非 0 退出） | 幂等（同输入同产物）；缺元素 ⇒ 非 0；`--check` 只读 |
 | `ui_icons.py` | **UI 图标白蒙版生成器**（S21.14 图标图片化 ④-04：读几何 spec `assets/ui_batches/icon_v1.json`（48 原生盒 ink/carve，抄自 `ti-render/icons.rs`）→ PIL 最近邻放大 512 白蒙版 → `assets/ui/icon_*.png`；运行时 `blit_image_tint` 按 tint 上色，缺图 = 代码矢量回退。生 14 张是低频脚本活故用 Python，必须 Rust 的只有运行时 tint 管线） | `python scripts/ui_icons.py`（生成/覆盖）/ `--check`（只读：重渲染字节比对幂等 + 资产在位 + 注册表 `ui_icons.json image→文件`，漂移/缺失非 0） | 幂等（同 spec ⇒ 同字节；重跑 = 覆盖同内容）；`--check` 只读，漂移非 0 |
 | `ui_islands.py` | **岛屿剪影资产生成器**（S26.2 主地图岛屿图片化：`assets/data/terrain.json` 陆格掩码 → 3 枚暖沙岛屿剪影 PNG，4× 超采抗锯齿 + 礁光晕 + 投影 + 墨线海岸 + FNV 纹理；994×826 RGBA `assets/ui/island_m01/02/03.png` + 注册 `ui_manifest.json` kind=island；运行时 `chrome::island_asset` 资产优先 blit，缺图 = 代码矢量回退） | `python scripts/ui_islands.py`（生成/覆盖 3 岛 + 登记 manifest）/ `--check`（只读：重渲染字节比对幂等 + 资产在位 + manifest 登记，漂移/缺失非 0） | 幂等（同 terrain ⇒ 同字节；重跑 = 覆盖同内容）；`--check` 只读，漂移非 0 |
| `task_close.py` | **任务收口一条命令**（跑判据 → 写 `evidence/<id>.log` → 台账**行级**置 verified → 同步 release.md 台账计数 → 跑 `check_ledger_reality.py` 复验；**判据非 0 就不写证据不改台账** ⇒ 虚报防线内置；`docs/process/15_process_ledger.md` 第 1 条流程的载体） | `python scripts/task_close.py --task S21.4 --cmd "cargo test --workspace"`（`--cmd` 可重复）/ `--no-run`（证据已在，只翻台账）/ `--check`（只读体检） | 幂等（同任务重跑 = 重写证据 + 不重复加 rounds）；判据非 0 ⇒ 退出 1 且不改台账 |
| `map_pack.py` | **地图包工具**（S31 / `21_map_package_spec.md`：**一张图 = 一个自包含可分发包**，取代 S30.1 扁平描述符 `assets/maps/<id>.json`（已归档 `legacy/data/maps_s301/`）。已装包 = `assets/maps/<id>/`（`manifest.json`/`map.json`/`layout.json`/`strings.json` + 默认包指针 `.default` + `_template/` 骨架）；`pack` = 已装包 → 可分发件 `dist/maps/<id>.map-pack/`(+zip，含池资产副本 `pool/` + `POOL_INDEX.json` 哈希索引)；`verify` = 校验包（schema/兼容/资产齐+哈希/31 格不变量 I1~I9，违规非 0）；`install` = 装包 + 接线（board.json 回注 + 池资产登记只读对账）；`add <id>` = 从 `_template` 引导新包骨架；`uninstall`/`list`；`check` = 已装态 ↔ 内嵌默认数据漂移对账（只读）；`sync` = 全部包 map→board.json + 默认包 layout→scenes/manifest/strings；`syncmap <id>` = `manifest["map"]`←`map.json`（29 号 §5 坑位固化：map.json 冗余副本无自动同步，填内容后必 syncmap 再 verify/pack/install；幂等 = 一致态跳过写；`--check` 只读 0 一致/1 漂移）；`index` = 生成 `assets/maps/index.json`（浏览器侧包清单，幂等） | `python scripts/map_pack.py pack <id>` / `verify <path>` / `install <path>` / `add <id>` / `uninstall <id>` / `syncmap <id>` / `list` / `check` / `sync` / `index`（`check`/`verify`/`syncmap --check` 只读） | 幂等（pack 同输入同产物 / install upsert / sync 重放同结果 / syncmap 一致态跳过写）；`verify`/`check`/`syncmap --check` 违规/漂移 ⇒ 非 0 退出
| `ui_variant.py` | **UI 资产视觉变体器**（S44 / 30 号 §5：固化 S42.0 的 PIL `tint_cool`（Color/Brightness 增强 + R×/G×/B× 冷移）为正式工具——选一组 `assets/ui/` 资产 + 着色参数 → 输出**同路径替换**（默认覆盖原资产，29 号 §3 同路径替换：资产名/坐标/scenes.json 全不动 ⇒ 零 Rust + 零 scenes.json diff）或 `--suffix` 变体路径（设计稿迭看）；`--register` 更新/新增 `ui_manifest.json` 条目（深拷贝防污染原条目，disk↔registry 双向同步；**r1-F5：register 后自动跑 `validate_data.py --check`，违规非 0 退出**） | `python scripts/ui_variant.py tint --assets a,b,c --color <c> --brightness <b> --rmul <r> --gmult <g> --bmult <b> [--suffix _v2] [--register] [--check]` | 幂等（同输入同产物，PIL 确定性）；`--check` 只读（0 合法 / 1 非法：参数超 [0,3] / 资产缺失）；写失败非 0；`--register` 后 validate_data 违规非 0 |
| `ui_theme.py` | **内容运营全主题创新·主题应用工具**（S48.0 / 30 号 §2.2 step3：主题包 `assets/themes/<tid>/compose.json` 的 `landscape_primitives`（完整 index.landscape.primitives）+ `assets` → 定点回注 scenes.json（read-modify-write + indent=1 规范形，0 churn 其余键）+ 部署资产到 `assets/ui/` + register + validate_data；**S49.3 平台 `/api/themes` 消费**（list/verify 只读面 + apply 写面走本脚本子进程，同 wizard 第 3 步路径）） | `python scripts/ui_theme.py apply --theme <tid> [--register] [--check]` + `list` | 幂等（同 compose 同结果）；`--check` 只读（0 合法 / 1 非法：主题缺 compose / compose 缺 landscape_primitives / 资产缺失）；写失败非 0；apply 后 validate_data 违规非 0 |
| `map_batch_gen.py` | **S37 批量地图包生成器**（基于 m01 模板程序化生成 20 张不同场景地图包 m04~m20：20 主题 tint + island_home_*.png + vs_bg_*.png + 专属海事事件；幂等（同输入同产物；已存在包跳过）；`--check` 只读校验已装包 + 资产齐备；`--only <id>` 指定单包；违规非 0 退出） | `python3 scripts/map_batch_gen.py` / `--check` / `--only m05` | 幂等（已存在包跳过）；`--check` 只读，缺包/资产未登记非 0 |
| `map_editor_design_gen.py` | **地图编辑器设计概念稿生成器**（为 `docs/map_editor_guide.md` 出一批 PNG → `evidence/ui_concept/`：主编辑器概念图 1〔环形画布 + 节点属性面板 + 实时 I1~I9 校验条；节点底图两态 `tile_owned`/`tile_deny` + 10 类岛闭集分配，非按 kind 画死〕 + 10 步向导逐步渲染图 10） | `python3 scripts/map_editor_design_gen.py`（重新生成 11 张）/ `--check`（只读：产物在位 + 非空，缺则非 0） | 幂等（PIL 确定性重画同产物）；`--check` 只读，缺图非 0 |
| `map_editor_visual_signoff.py` | **地图编辑器前端视觉签收出图**（Playwright 无头 Chromium 开平台「地图」页 → 触发编辑器 + 内容面板 + 10 步向导 → 4 张截图落 `evidence/ui_concept/map_editor_signoff_*`；`docs/map_editor_guide.md` 设计稿实现的最终视觉签收面（A 纪律收口）；前置 = 平台 `cargo run -p ti-console` 在 8787，缺则非 0 拒绝截图） | `python3 scripts/map_editor_visual_signoff.py`（重截 4 张）/ `--check`（只读：产物在位 + 非空，缺则非 0） | 幂等（同输入同产物：文件名稳定 `map_editor_signoff_*`，重截覆盖同路径）；`--check` 只读；平台未起 / 缺 playwright ⇒ 非 0 |
| `map_editor_redesign_concepts.py` | **地图编辑器重设计 3 方向概念稿生成器**（A 纪律设计稿先行：3 方向横版概念稿 1600×1000 → `evidence/ui_concept/map_editor_redesign_{A,B,C}.png` —— A 仪表盘〔左控制/中大画布/右属性+校验 + 顶步骤条〕/ B 地图为主〔全屏大海图 + 右下浮层〕/ C 卡片流〔竖向卡片堆叠〕；用户挑方向后细化） | `python3 scripts/map_editor_redesign_concepts.py`（重生成 3 图）/ `--check`（只读：产物在位非空，缺则非 0） | 幂等（PIL 确定性重画同产物）；`--check` 只读，缺图非 0 |
| `map_editor_redesign_v3_minimal.py` | **地图编辑器「极简分步」3 变体概念图生成器**（用户 2026-11-09「设计太复杂了，重新设计地图制作整个交互」：一次只亮当前步工作区、其余折叠、画布只画环 + 中央岛；3 变体 1280×800 → `evidence/ui_concept/map_editor_v3_minimal_{a,b,c}.png` —— A 向导聚焦〔10 圆点细步进条 + 当前步专属工作区〕/ B 上下分屏 / C 单步聚焦） | `python3 scripts/map_editor_redesign_v3_minimal.py`（重生成 3 图）/ `--check`（只读：产物在位非空，缺则非 0） | 幂等（PIL 确定性重画同产物）；`--check` 只读，缺图非 0 |
| `map_editor_step_render.py` | **地图编辑器 10 步逐步设计渲染图生成器**（`docs/map_editor_guide.md` 图文引用源：10 步各 1 张 1280×800 完整界面渲染图〔该步高亮 + 该步结果面板〕+ 1 张总览〔10 步 5×2 缩略拼图 + 流转标注〕→ `evidence/ui_concept/map_editor_step_{01_add..10_zero_rust}.png` + `map_editor_steps_overview.png`；步序与 `map_wizard.rs` 的 10 步 1:1） | `python3 scripts/map_editor_step_render.py`（重生成 10 步 + 总览）/ `--check`（只读：产物在位非空，缺则非 0） | 幂等（PIL 确定性重画同产物）；`--check` 只读，缺图非 0 |
| `map_editor_tile_concepts.py` | **地图格子「创意形状」概念图**（用户 2026-11-09「格子为什么用卡片？去创意生成不同格子」：真渲染 v6 已是 seed 随机 6 顶点不规则多边形但编辑器/概念图用旧矩形卡 ⇒ 对齐真渲染；4 变体 1280×800 → `evidence/ui_concept/map_editor_tile_concept_{a,b,c,d}.png` —— A 真渲染对齐〔seed 随机 6 顶点多边形 + 色带圆环 + 中央图标〕/ B 拼图块 / C 圆形徽章 / D 六边形蜂窝） | `python3 scripts/map_editor_tile_concepts.py`（生成 4 图）/ `--check`（只读：产物在位非空，缺则非 0） | 幂等（PIL 确定性重画同产物）；`--check` 只读，缺图非 0 |
| `exp_log.py` | **EXP 条目一条命令登记**（分卷正文 + `index.md` 索引行 + 分卷「条数」列 3 处一次改齐；ID 查重；条数从分卷文件 `### EXP-` 标题**实测**回填而非 +1 猜；收尾跑 `doccheck` 复验） | `python scripts/exp_log.py --volume code --id EXP-<N> --title "标题" --summary "索引摘要" --body "正文"`（`--body-file` 读文件）/ `--check`（只读查重） | 幂等（同 id 再跑 = 查重拦截，不重复写）；查重/对账失败 ⇒ 退出 1 |
| `doc_numbers.py` | **门禁项数 / 台账计数 实测回写**（一改要手抄 10 处的漂移源：parity/git/README/loop/AGENTS/style/playbook/gate 中英横幅 + doccheck2.rs 期望值；S46.2 补英文横幅与 doccheck2.rs 期望值盲区，EXP-172；门禁项数口径 = `gate.ps1` 顶层 `Report` 数（与 doccheck2 同口径），台账计数 = ledger 实测） | `python scripts/doc_numbers.py`（只读体检，漂移即非 0）/ `--fix`（实测回写）/ `--fix --dry-run` | 幂等（回写后再测 0 漂移）；漂移未修 ⇒ 退出 1 |
| `layout_spec_check.py` | **UI 版面规格三面对账**（规格表 → `scenes.json` → `scene_manifest_v3.json`：键↔坐标一致 + 8pt + N8 像素往返 ±1px + N3 焦点面积；规格表是坐标单一事实源，人工校验易漏） | `python scripts/layout_spec_check.py --spec docs/ui-redesign-v3/08_main_map_v4_spec.json [--scene main_map]` | 幂等只读；漂移非 0 |
| `check_ps_scripts.ps1` | **PowerShell 入口体检**（UTF-8 BOM 校验 + `Parser::ParseFile` 真语法自检 ⇒ 拦住「缺 BOM 入口从未真跑起来」，EXP-109） | `powershell -ExecutionPolicy Bypass -File scripts/check_ps_scripts.ps1`（`-Check` 别名） | 幂等只读；不合格非 0 |
| `ui_preview.ps1` | **7 屏出图验收**（Bevy 离屏 → `evidence/bevy_native_<scene>.png` + 缩略图 + 非空帧统计） | `powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1 [--viewport=WxH] [scene...]` / `-Check [--viewport=WxH] [scene...]` | 幂等（重出图）；未出图 / 空帧 ⇒ 非 0 退出；`-Check` 只读验收既有出图（不重渲染 / 不写缩略图，失败非 0 退出）；S38 `--viewport=WxH` 批量横屏出图（产物名 `bevy_native_<scene>_<WxH>.png`，缺省竖屏 1080×1920 保持原产物名） |
| `agnes_gen.py` | AI 生图底层（`ui_assets.py build` 调用；需 `AGNES_API_KEY`） | 由 `ui_assets.py` 调用；`python scripts/agnes_gen.py "<prompt>" --out <png>` / `--check` | 生成非幂等（同 prompt 可复现性由模型决定）；生图失败 / 缺 KEY ⇒ 非 0 退出；`--check` 只读前置体检（不生图 / 不写文件，前置缺失非 0 退出） |
| `agnes_video.py` | **AI 视频生成**（Agnes Video 2.5 Flash；异步任务：创建 → 轮询 → 下载；需 `AGNES_API_KEY`） | `python scripts/agnes_video.py --prompt "..." --out <mp4> [--seconds 5] [--ratio 16:9] [--mode text]` / `--check` / `--video-id <id>`（直接轮询已有任务） | 生成非幂等（模型输出不重复）；任务创建/轮询/下载失败 ⇒ 非 0 退出；`--check` 只读前置体检（不创建任务 / 不写文件，前置缺失非 0 退出） |
| `agnes_vision.py` | AI 读图（截图/素材分析 ⇒ 视觉问题取证） | `python scripts/agnes_vision.py <png> "<question>"` / `--check` | 只读分析；缺 KEY / 缺图 ⇒ 非 0 退出；`--check` 前置体检（不调 API / 不写文件，前置缺失非 0 退出） |
| `pcx_to_png.py` | 原版 PCX → PNG（贴图转码） | `python scripts/pcx_to_png.py <in.pcx> <out.png>` / `--check` | 幂等（解码确定）；src 缺失 ⇒ 非 0 退出；`--check` 只读校验（产物缺失/漂移 ⇒ 非 0 退出） |
| `pack_hd_assets.py` | HD 资产打包（`assets_hd_pack/`） | `python scripts/pack_hd_assets.py [--no-webp]` / `--check` | 幂等（同一输入同一产物）；pack 目录删除失败 ⇒ 非 0 退出；`--check` 只读校验（缺源/未打包/哈希漂移 ⇒ 非 0 退出） |
| `migrate_strings.py` | i18n 迁移（`.rs` 中文面量 → `strings.json`） | `python scripts/migrate_strings.py --report` / `--apply` | `--apply` 幂等（已迁过零改动）；apply 缺 `strings.json` ⇒ 非 0 退出；`--report` 只读 |
| `verify_hd_manifest.py` | HD manifest ↔ `assets_hd/` 产物 PNG 一致性**验收辅助**（与 `check_hd_manifest.py` 分工：前者对 `assets_hd_pack/1x/ui` 目录（门禁项），本工具对 `manifest_hd.json` 产物（一次性验收）） | `python scripts/verify_hd_manifest.py` | 幂等只读；不一致非 0 |
| `bevy_cpu_scene_diff.py` | 双后端**现场**像素对拍（CPU 帧缓冲 vs Bevy 离屏，7 场景） | `python scripts/bevy_cpu_scene_diff.py` / `--check` | 幂等（重出两边再比）；写 `evidence/bevy_cpu_scene_diff.md`；`--check` 只读比对既有输出（不重生 / 不写报告，缺输出 ⇒ 非 0 退出） |
| `wasm_browser_e2e.py` | **Bevy wasm 浏览器端 e2e**（Playwright 无头 Chromium 5 层：L1 实例化 = env-stub.js 覆盖全部 env 导入 + start 段跑通 / L2 渲染 = 资产 fetch 全 2xx + 画面非全黑（截图像素采样）/ L3 交互 = 键盘+鼠标不抛异常 / L4 场景切换 = select_map 鼠标点 m01 卡片 ⇒ 切 main_map（增量亮像素比判据）/ **L5 主地图资产实取 = v6 basename 集合（`ocean_bg.png`/`isle_land.png`/`isle_village.png` 等，S34 默认；v6 资产 basename **不带 `v6_` 前缀**，前缀是目录 `v6/`，EXP-145）**或** v5 环资产（`isle_*.png` 领地卡 + `island_home_*.png` 中央据点岛 + `card_panel.png` 卡板，各 ≥1 个 2xx）**） | `python3 scripts/wasm_browser_e2e.py [--url http://host:port/] [--viewport=WxH]` / `--check`（只读：打印判据不截图） | 幂等（同 URL 同产物）；写 `evidence/wasm_browser_e2e[_<WxH>].png`（S38 `--viewport=1920x1080` 横屏：视口 + L4 点击坐标按视口比例映射 + 截图名带尺寸后缀，缺省竖屏 540×960 保持原产物名）；`--check` 只读；任一层断言失败 ⇒ 非 0 退出；依赖 `serve`（`cargo run -p ti-tools --bin serve -- --dir web --port 8080`）在跑 + Playwright Chromium（缺省自动探测 `~/.cache/ms-playwright/chromium-*/`，`TI_CHROMIUM_PATH` 可覆盖）+ Pillow（L2/L4 截图像素采样）；**L5 为何必要 = L4 亮像素比会被 `chart_full` 亮背景掩盖 ⇒「环/岛/骰全灭」时 L4 仍 PASS（S32.1 回归漏网根因）**；浏览器 GPU 子进程崩溃（SwiftShader 资源耗尽）⇒ L1~L5 记名降级 SKIP 不 FAIL |
| `android_device_test.py` | **Android 真机交互验收（S49.4e，docs/parity.md §P5）**：P5-1 全屏（`wm size` vs `dumpsys window` app bounds 差 ≤4px）/ P5-2 触摸命中（`input tap` → logcat `[hit]` 或截屏 md5 变化）/ P5-3 音频生命周期（`KEYCODE_HOME` → BGM 暂停；`am start` → BGM 恢复，logcat `[audio] BGM`）/ P5-4 切后台不崩（`pidof` 在 + 无 FATAL/ANR）。**背景**（2026-11-09 真机反馈「全屏/菜单无响应/后台音乐不停」根因审计）：P0-P4 全桌面/解析器判据，真机专属行为结构性测不到——本工具把 P5 落地为 adb 驱动的机器判据 | `python3 scripts/android_device_test.py --check`（只读环境检查：adb + 设备授权 + 包已装）/ `--run`（执行 P5 全判据）/ `--run --only p5-N`（单判据）/ `--tap-x/--tap-y`（首次设备标定 P5-2 菜单坐标） | `--check` 只读 0 写盘；`--run` 写 `evidence/android_device_test.log`（判据证据）；环境缺（无 adb/设备/包）或判据 FAIL = 非 0 退出（如实上报，不静默）；判据 FAIL 不静默通过。**工具契约**：落地 `scripts/`（非 `tmp/`）；`--check` 只读 + 非 0 退出；双消费 = gate 第 4c 项（有设备跑 P5，无设备记名 SKIP 文档化）+ 平台测试页 `/api/tests?run=android_device_test`（test_manifest 登记） |
| `web_playtest.cjs` | **浏览器真运行时试玩**（S16.1：Node + Playwright CDP 按键驱动 菜单→选人→选图→主地图；L1 status 活体 + L2 截图墨点 + L3/L4 3×Enter 场景流转帧变化 + console 致命错误=0）。**S25 版本规范**：wasm 产物名带版本（`treasure_bevy_wasm-<x.y.z>.wasm`），读 glob 最新带版本文件，缺失回退旧名（兼容未重建） | `node scripts/web_playtest.cjs [--url http://host:port/] [--out evidence/web_playtest.log]` | 幂等（结果/截图重跑覆盖）；写 `evidence/web_playtest.png` + `.log`；失败非 0；依赖 `serve` 在跑 + Node playwright（`TI_CHROMIUM_PATH` 可覆盖，缺省自动探测 `~/.cache/ms-playwright/chromium-*/`） |
| `playtest_loop.py` | **试玩闭环 4 报告聚合**（S16.2：`docs/process/08_playtest_loop.md` 执行面——fuzz_playtest / full_run / render_invariants / web_playtest 一次跑全 + 归因三分类清单） | `python3 scripts/playtest_loop.py` / `--check`（只读：打印判据 + 三分类，0 新产物） | 幂等只读；4 报告全 0 失败 = 0 退出；任一 native 报告失败 ⇒ 非 0；web 报告缺 serve/node/playwright = 记名 SKIP 不 FAIL；停机 = 连续 3 轮收敛 + evidence/S16.2.log 记录（主线程复核，EXP-038/058） |
| `build_wasm.ps1` | wasm 构建（→ `web/`，Bevy 交付壳）。**S25 版本规范**：产物名带版本 `treasure_bevy_wasm-<x.y.z>.wasm`（拷 wasm-bindgen 输出 `_bg.wasm`，读 `Cargo.toml` `[workspace.package] version` 单源；删旧 raw 副本 `treasure_bevy_wasm.wasm`） | `powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1` / `-Check` | 幂等（重构建覆盖）；缺 target / 构建失败 ⇒ 非 0 退出；`-Check` 只读前置体检（不构建 / 不拷贝，前置缺失非 0 退出） |
| `build_apk.sh` | **Android APK 构建（S2.6/S18.8d）**：本机 Linux + NDK r25b 交叉编译（x86_64/aarch64-linux-android）→ cdylib .so → APK 组装（manifest aapt2 link + resources.arsc + 最小 classes.dex + .so **ZIP_DEFLATED（S35.0 24 号标准①，原 STORED 已废，实测 53→21.5MB）** + 仓内资产 **ui/fonts/audio 按引擎引用集**（`scripts/apk_audit.py --refset` 单源；未引用图 0 进包）+ maps 全量 入 APK assets 段）→ zipalign -f 4（.so 压缩后常规 4 字节对齐，`-p` unaligned 已废）→ **官方 `apksigner` CLI 签名**（S46.3：原自造 `SignV2` 助手依赖 `/tmp/opencode/apksigner/` 仓库外临时资产，/tmp 清理后即 `ClassNotFoundException: SignV2` 构建死在签名步 ⇒ 改 `apksigner sign --v1/v2-signing-enabled --out`；apksigner 自动 v2+v3，minSdk 26 无需 v1；keystore 缺失自动 `keytool` 生成〔原私钥不可恢复，新 key = 新签名，装机需卸载旧包〕；`[0/5]` 依赖前置缺一即 FAIL 早退）→ dist/TreasureIsl-<x.y.z>-<arch>.apk（S25 版本规范：读 `Cargo.toml` `[workspace.package] version` 单源，产物名带 x.y.z；arch = 脚本 CLI 参 `aarch64`/`x86_64`）；**[4/5] 体积审计**（`apk_audit.py --apk` 判据①②，FAIL 阻断）；装机 = adb install（v2 资产已入 APK `assets/` 段，**无需 legacy/data push**；`assets_reader_android` StripAssetsReader 剥 `assets/` 前缀直读，EXP-147） | `bash scripts/build_apk.sh [aarch64 或 x86_64]`（**2026-11-01 用户拍板：主要安卓玩、不要 x86_64 ⇒ 只交付 arm64**） | 幂等（重编 .so + 重打包）；NDK 路径硬编码 /tmp/opencode/android-ndk-r25b（本机实测）；.so 未 strip = 装机可跑。**2026-11-01 S34 实测已知坑**：①`android-activity` **不在根 workspace members**（S18.8d 移出，`ndk-sys` native 拒编译）⇒ `cargo build -p android-activity@0.1.0` 从根报 similar（兜 registry 0.6.1）；**正确 = 临时加回 members**（根 `Cargo.toml` §members 注释）→ `--target <arch>` 构建 → **构建后必须恢复**；②x86_64 `CC_${ARCH//_/-}` 连字符变量名（`x86_64`→`x86-64`）bash `export` 非法 ⇒ 须 `env "CC_x86_64-linux-android=..."`（aarch64 无此问题）；③android-activity 单 crate（`include!` 整个 Bevy 壳 + `[profile.release]` LTO fat + codegen-units=1）编译 >15min ⇒ `timeout` 须 ≥3600s；④`android_entry.rs` 首行 `//!` 被 `include!` 非法（E0753，非模块根）⇒ 已改 `//`（2026-11-01）；⑤**lib 目录 ABI 映射**：APK 内 `lib/<ABI>/` 必须用 Android 标准 ABI（aarch64→**arm64-v8a**、x86_64→**x86_64**），`lib/aarch64` 不被安装器认 ⇒ `INSTALL_FAILED_NO_MATCHING_ABIS` / 真机「32 位系统」提示 / 闪退（EXP-146）；⑥**`extractNativeLibs`**：`android/AndroidManifest.xml` 须 `true`——`false`（.so 直接 mmap 执行）要求 .so 未压缩+16KB 对齐（zipalign -p 4 不满足）且 **arm 转译（MuMu/redroid x86_64 跑 arm64）需解压加载** ⇒ 装不上/闪退（EXP-146）。**v2 用法**（2026-11-01 固化）：`bash scripts/build_apk.sh`（默认 aarch64/arm64，`.so` 已新则复用 + aapt2 link 每次重跑 + trap 恢复 members）；`x86_64` 传参；`--force` 强制重编 `.so`。装机验证：真机/redroid `adb install` + `am start`（redroid = x86_64 + arm 转译，同 MuMu）；⑦**launcher 图标/应用名**（EXP-148）：`android/AndroidManifest.xml` 须 `android:icon="@mipmap/ic_launcher"` + `label="@string/app_name"`（硬编码 label 部分 launcher 读不到）+ `android/res/`（mipmap 5 密度 + values/strings.xml）；aapt2 link **两阶段**（`aapt2 compile --dir android/res -o res.zip` → `link ... res.zip`，aapt2 link 不接受 res 目录 positional）+ 组装复制 base.apk 的 `res/` 条目；⑧**体积标准 S35.0**（24 号 `docs/process/24_apk_size_std.md`）：.so 必须 ZIP_DEFLATED（实测 APK 184→146MB）+ 资产只带引擎引用集（`apk_audit.py --refset`，8 项未引用图 5.3MB 移出）+ [4/5] 审计判据①② FAIL 阻断发布。配套工具 = `apk_audit.py`（引用集 `--refset` / 构成审计 `--apk` / 未引用清单 `--refset-unused`）；⑨**S46.3 API 对齐**（EXP-173）：S34 开 `bevy_audio` → cpal → ndk-sys `#[link(name="aaudio")]` 链接 `-laudio`（AAudio，**NDK 仅 API≥26 的 sysroot 有 libaaudio**），原 `android21-clang` 交叉链接失败（`unable to find library -laudio`）⇒ `build_apk.sh` clang API 21→26（变量 `API=${3:-26}`）+ `android/AndroidManifest.xml` `minSdkVersion` 21→26（AAudio 最低 API 26） |
| `apk_audit.py` | **APK 体积构成审计（S35.0，24 号单一事实源 `docs/process/24_apk_size_std.md`）**：① `--refset` 出「引擎引用集」JSON（代码字面量 + `format!` 动态前缀 icon_/isle_/island_home_/role_ 全带 + 地图包/数据岛 kind + registry audio；`build_apk.sh` / `build_wasm.ps1` 4f 打包只带引用件，未引用图 0 进产物）；② `--apk <path>` 出体积构成表（按目录聚合 压缩后/原始/文件数 + `.so` 压缩率 + 未引用 UI 漂移清单）= 平台「APK 体积」页数据源；判据 = ① `.so` 压缩率 < 90%（Stored 100% ⇒ exit 3）② 未引用 UI 0 进包（漂移 ⇒ exit 4）；③ `--refset-unused` 列磁盘有但运行时 0 引用的 `assets/ui` 文件（构建侧裁剪核对） | `python3 scripts/apk_audit.py [--refset \| --apk <path> \| --refset-unused]`（`--json` 静默输出纯 JSON） | 幂等只读（0 写盘）；`--refset`/`--refset-unused` 成功 = 0；`--apk` 判据① FAIL = 3 / 判据② FAIL = 4 / 无 lib 段 = 2 / 缺路径 = 1。**工具契约**：落地 `scripts/`（非 `tmp/`）；只读；非 0 退出（判据 FAIL）；双消费 = `build_apk.sh` [4/5] 段 + 平台 `/api/apk-size`（ti-console） |
| `build_artifacts.py` | **构建产物编排器（S35.1，24 号 §4.1 执行面；平台「构建」页数据源 + 执行器）**：把「LLM 不确定、但可程序化确定」的构建动作固化为单一入口（消除 LLM 每次手拼 bash/pwsh/cargo 命令 + 手管后台任务 + 手判 mtime 复用 + 手跑复检的重复错误面）。子命令：`pre <kind>` 构建前 mtime 锚定（apk = 回退 `treasure_bevy.rs` mtime 到 `.so` 前 10s，防手动改 .rs 误触发 15min 重编）；`run <kind>` 执行构建本体（apk=`build_apk.sh` / wasm=`build_wasm.ps1` / native=`cargo build treasure-bevy-native`）+ 构建后自动 `post` 复检 + 日志落 `evidence/console/logs/build-<kind>-<ts>.log`（gitignored）；`post <kind>` 复检（apk=`apk_audit.py --apk` 判据①② / wasm=产物在位 / native=退出码）+ 复原 .rs mtime；`list` 已构建产物清单（体积/mtime；`--json` 结构化输出 kind/artifact/size_mb/mtime/present/note 供平台 `/api/build` 渲染卡片，纯文本面保留）；`check` 前置体检（runner 在位 + 产物目录）。**S46.3**：apk 产物判定改 glob `dist/TreasureIsl-*.apk`（build_apk.sh 产物名 `TreasureIsl-<x.y.z>-<arch>.apk`，原硬编码 `TreasureIsl-arm64.apk` 永不命中 ⇒ 已构建仍报未构建；`_resolve_artifact` glob 解析）。**构建本体仍走既有脚本**（`build_apk.sh`/`build_wasm.ps1`=工具契约不变），本脚本只做编排（锚定/复检/日志/后台） | `python3 scripts/build_artifacts.py {pre\|post\|run\|list\|check} [kind]`（kind=apk\|wasm\|native；`list`/`check` 不带 kind） | 幂等（`run` 重跑覆盖产物 + 新日志）；`pre`/`post`/`list`/`check` 成功 = 0；`run` 构建失败 / 复检 FAIL = 非 0；`check` runner 缺位 = 非 0。子进程补 `~/.cargo/bin` 到 PATH（cargo 缺位回退）。**工具契约**：落地 `scripts/`（非 `tmp/`）；`run` 写 `dist/`+`web/`+`target/`+`evidence/console/logs/`（gitignored），其余只读；双消费 = `build_apk.sh` [4/5] 段 + 平台「构建」页（`/api/build` 只读 list + `?run=` 后台触发）+ `console_commands.json` 白名单 id `build_{apk,wasm,native}` |
| `publish_web.ps1` | **一键发布 publish-web（S16.3）**：gate 全绿前置 → build_wasm 重建 `web/` → 本地 `serve` 静态断言（index/wasm/js 200 + content-type）→ 发布历史（sha256 + git short sha + 日期）入 `release.md`「发布」节 → `git worktree` 推 `gh-pages`（不污染 master）→ 线上 URL 复跑 `web_playtest` 全交互 0 失败；Pages/远端不可达 = 实测定态降级登记 release.md 已知限制（不静默绕过） | `pwsh -File scripts/publish_web.ps1` / `-Check`（只读前置体检：不构建/不推/不写 release.md） | 幂等（重跑覆盖发布历史行）；前置 gate 非全绿 ⇒ 非 0 退出；线上不可达 = 记名降级非 FAIL |
| `smoke.ps1` | native + wasm 冒烟 | `powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1` | 幂等；失败非 0 |
| `playbook.ps1` | 常用命令一键入口（16 条） | `powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 -List` / `<cmd>` / `-Check` | 幂等只读转发器（判定 / 退出码由被转发工具承担）；未知命令 / 转发失败 ⇒ 非 0 退出；`-Check` = 被转发目标在位性只读体检（不执行被转发工具，缺失非 0 退出） |

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

### 3.5 cargo bin / crate 工具（S32.1 起，`crates/` 内可执行入口）

| 工具 | 用途 | 用法 |
|---|---|---|
| `ti-console`（`crates/ti-console`） | **TI Dev Console 平台本体**（S32 开发管理平台，纯管理/辅助开发；与游戏运行时解耦，定案 = `docs/process/22_dev_platform.md`）：静态壳 + `/api/health`（S32.1 骨架）+ 单实例锁 + 命令白名单（`console_commands.json`）+ 数据源（`console_sources.json`）；后续 S32.2~S32.9 逐页。**`/api/tests?run=<id>`** = 游戏测试实际运行（23 号 §6.1：test_manifest 登记 id 枚举 → command 逐参数 `Command::new` 后台 spawn → 日志落 `evidence/console/logs/test-<id>-<ts>.log` gitignored；程序白名单 cargo/python3/pwsh/powershell，禁 shell 拼接；人工验收/无命令 = 拒绝）。**`GET/POST /api/gate`** = 门禁执行（S32.4：GET 状态/历史/尾部只读；POST = X-TI-Token 鉴权 403 + tier 校验 400 + 单实例锁 409 + 后台 spawn gate.ps1，日志落 `evidence/console/logs/gate-<ts>.log`；门禁副作用只记录不判定）。**`GET /api/audit` + `POST /api/audit/taskbook|archive`** = 双审计编排（S32.5：GET 报告清单 + 收敛表〔22 号 §7.4 机械判定：轮次完整性 + pass/in_progress/escalate + 未闭环〕；POST taskbook = 生成审计任务书 `.prompt.md`（X-TI-Token 鉴权 403 + body `{"task","round","role","surface?"}` + 八要素模板机器断言 `uncovered_elements`）；POST archive = 粘贴归档报告（命名规则 + 防穿越 + 非空正文）；协议见 22 号 §7.3/§7.4）。**`GET /api/close?audit=1` + `POST /api/close`** = 任务收口（S32.6：GET = 白名单对账断言〔22 号 §4.2 人读视图 id 集合 == `console_commands.json`，S32.6 DoD，免 token〕；POST = `close_check` 只读体检（`task_close.py --check`，不改台账）/ `close_run` 写（`task_close.py --cmd`，白名单 id 精确映射 + 未登记 400，需 `--enable-write`）/ `exp_check` EXP 查重（`exp_log.py --check`）；鉴权 403 + CT 400 + body 413；日志落 `evidence/console/logs/close-*.log`；22 号 §4.2 禁止 1 禁透传自由字符串）。**`GET /api/search?q=`** = 文档检索（S32.8：全文搜索 + `EXP-`/`F-`/任务 ID 精确引用反查，只读 0 写盘；白名单根 `docs/ crates/ scripts/ plan/` + 文本扩展名 + 上限 3000/512KB/500 超限截断；`q` 空 = 空结果；**`q` 服务端先 `percent_decode`**〔前端 `encodeURIComponent`，中文/空格才命中，非法 `%XX` 原样保留〕）。**`GET /api/build?check=1`** = 构建只读体检（S32.9：`build_wasm.ps1 -Check` 只读 + 22 号 §8 冻结路径内容级 sha256 指纹差断言，`ok = 体检通过 && 差为空`，0 写盘）。**`GET /api/build?run=<kind>`** = 构建触发（S35.1：kind 白名单 `apk|wasm|native`，后台 spawn + 日志落 `evidence/console/logs/build-*.log` gitignored；非法 kind = 400） | `cargo run -p ti-console`（默认 `127.0.0.1:8787`；非回环须 `--host` + `TI_CONSOLE_TOKEN`） |
| `ti-devmeta`（`crates/ti-devmeta`） | 平台共享层（台账解析 + 静态 HTTP，纯 std 0 游戏依赖；`ledger.rs` 从 `ti-tools` 上提的单实现） | 库 crate（无 bin；`cargo test -p ti-devmeta` 验证台账解析行为对齐） |
| `verify`（`crates/ti-tools/src/bin/verify.rs`） | **全格式 0 未知字节校验**（S46.0；P0-2 判据载体）：遍历 `legacy/data/` 真实文件逐族解析 —— UI 场景 `.Map`（`MapScene::parse` + `MapControl::parse_words`）/ 地形图 `glmapm*.Map`（`TerrainMap::parse`）/ `GoldLand.Fnt`（`Font::parse`）/ `font.dat`（`Charset::parse`）/ `settings.dat`（`Settings::parse`）/ 容器 `.pak`/`.lpk`（`Package::parse` + **逐条目** `read_entry` + PCX 图像条目 `Image::decode`）；口径 = **结构后无残留字节**（`define_format!` 的 `read` 对残留报 `TrailingBytes` ⇒ 解析成功即达标）；无解析器者如实标注「跳过」，不静默通过 | `cargo run -q -p ti-tools --bin verify -- --zero-unknown`（人读报告）/ `--json`（机器可读）；**只读**（仅读 `legacy/data/`，0 写盘）；幂等（重复跑同输出）；任一文件/条目未达标 ⇒ **非 0 退出**；无参 ⇒ 打印用法并退出码 2 |

> 依赖图断言（22 号 §11.2）：`cargo tree -p ti-console` 不得含 `ti-core`/`ti-platform`/`ti-app`/
> `ti-engine`/`ti-render`/`bevy`/`macroquad`；违规 = 平台与游戏运行时耦合，返工。

## 4. 门禁项 ↔ 工具对照（排障入口）

`scripts/gate.ps1` 的 33 项按域分组，**每组对应的排障工具**如下（门禁清单本身见
[`../parity.md`](../parity.md) §代码质量门禁）：

| 域 | 门禁项 | 排障工具 |
|---|---|---|
| 编译 / 测试 | 1~4b（fmt / clippy / test / build / feature bin） | `cargo fmt` / `cargo clippy` / `cargo test` |
| 占位 / unwrap | 5~6 | `Select-String` 扫 `TODO|unwrap(`（`check_hardcode.py` 辅助） |
| 格式 0 未知字节 | 7 | `cargo run -q -p ti-tools --bin verify -- --zero-unknown`（S46.0 起默认 full 档实跑；`--json` 机器可读） |
| 台账 / 文档 | 8~9、13~14 | `cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml` / `doccheck` / `doccheck2` / `check_ledger_reality.py` |
| 判据载体 | 25 | `python scripts/check_dod_commands.py --check`（DoD 判据命令可达性；EXP-170/171） |
| 数据 | 10a~10c、20 | `python scripts/validate_data.py` / `gen_layout.py --check` / `check_hardcode.py` / `check_data_source.py` |
| 资产 | 19.5 | `python scripts/ui_assets.py verify` + `ui_preview.ps1` + `ui_shot_probe.py` |
| 结构 | 23（audit_code） | `python scripts/audit_code.py --report` + `code_outline.py --crate <c>` |
| 业务闭环 / 契约 | 15~16 | `check_forward_closed.py --check` / `check_module_contracts.py --check` |
| 飞轮 | 17 | `flywheel_readme.ps1 -Check` |
| 渲染 | 18、19、21、21.5 | `bevy_cpu_scene_diff.py` / `check_scene_narrative.py` / `ti-engine` example `bevy_vs_cpu_pixel` |
| 数值规格 | —（新增项由 `board_measure.py --check` 守） | `python scripts/board_measure.py --check` |
