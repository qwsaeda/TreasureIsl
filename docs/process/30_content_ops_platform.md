# 30 · 内容运营平台化定案（TI Dev Console 固定流程向导）

> **本文件 = 内容运营平台化的唯一事实源**。触发：用户 2026-11-06
> 「既然 UI 和地图已经与程序分离，那么制定它们就应该是平台的功能——平台上有没有固定流程，
> 能固化的东西应可固定化、模式化去制定，包括最终的打包」→ 用户拍板（2026-11-06，四问全选推荐）：
> ① 范围 = 地图 + UI + 打包**全流程**；② 执行模式 = **半自动向导**（每步确认 + 机器判据，不可跳步）；
> ③ 写面放开 = **加白名单默认关 + 内容级快照判定**；④ UI 形态 = **视觉变体器**（着色/替换 + 出图 + 签收留痕）。
>
> 关联（单一事实源引用，不复述）：`29_content_ops.md`（内容运营规范：程序构成模型 + §2 改地图 10 步 + §3 换 UI 6 步）、
> `21_map_package_spec.md`（地图包格式/I1~I9/§8 工具流水线）、`22_dev_platform.md`（平台定位/边界/§4.2 白名单/§8 北极星）、
> `18_asset_content_pipeline.md`（资产管线）、`13_ui_asset_pipeline.md`（UI 资产管线）。
> 进度 = `plan/ledger.yaml`；WBS = [`plan-s44-content-ops-platform.md`](../plan-s44-content-ops-platform.md)。

## 1. 目标与边界

**是什么**：把 `29_content_ops.md` §2（改地图 10 步）/§3（换 UI 6 步）的固定顺序，做成平台的
**固定流程向导**（每步一个按钮调白名单脚本 + 机器判据回显，不可跳步，失败即停），含打包/分发面。
平台 = **编排者**（调既有工具、断言判据），**不重写**地图/UI 制作逻辑（工具契约不变，`map_pack.py`/`ui_variant.py` 仍唯一事实源）。

**不是**（防止范围蔓延，镜像 22 号 §9 + 29 号 §6）：

- 不是从零设计工具（AI 生图 + 元素创作 + 迭看 = 29 号 §6「从零设计」路径，另立项）；
- 不是游戏编辑器 / 运行时 / 关卡工具（22 号 §9 不做清单）；
- 不做任意 shell：只执行 §4.2 白名单动作 + 枚举/id 参数（22 号 §10.3）；
- 平台不重写业务规则（`ti-core` 冻结，29 号 §0 前提）。

## 2. 页面与流程

左菜单新增「**内容运营**」组（平台壳 `crates/ti-console/web/index.html`，原生 JS 单文件，镜像 22 号 §6.1）：

### 2.1 地图工作台（固定 10 步，映射 29 号 §2 1:1）

| # | 步骤 | 平台动作（§4.2 id） | 机器判据（每步回显） |
|---|---|---|---|
| 1 | 新建包 | `mappack_add`（`map_pack.py add <id>`） | `assets/maps/<id>/{manifest,map,layout,strings}.json` 4 文件存在 |
| 2 | 填内容 | 编辑器（复用 S32.15 环形画布 + 表单：map.json isle 排布 / layout 环几何 / strings 图名 / manifest 资产）→ `POST /api/map/import` | import 返回 `errors=[]`（I1~I9 终判过） |
| 3 | manifest["map"] 同步 | `mappack_syncmap`（`map_pack.py syncmap <id>`，**新子命令**：manifest["map"]←map.json，幂等 + `--check` 只读） | `--check` 0 差（29 号 §5 坑位固化） |
| 4 | 校验 | `mappack_verify`（已有） | 0 违规（I1~I9 + schema/兼容 + I7 哈希） |
| 5 | 打包 | `mappack_pack`（`map_pack.py pack <id>`，**新**） | `dist/maps/<id>.map-pack/`（+ `.zip`）存在 + 再 verify 0 违规 |
| 6 | 安装 | `mappack_install`（`map_pack.py install <path>`，**新**） | `assets/maps/<id>/` 接线 + verify 0 违规 |
| 7 | 漂移对账 | `mappack_check`（已有） | 已装态 ↔ 内嵌默认数据 0 漂移 |
| 8 | 回注 | `mappack_sync`（`map_pack.py sync`，**新**） | `board.json`/`scenes.json`/`strings.json` 回注 0 差（幂等） |
| 9 | 出图验收 | `--shot --scene=main_map`（`TI_MAP=<id>`，平台后台跑） | 出图非黑 + 与现有图**数据级差异**（isle 排布/ring 几何/events 不同，29 号 §2 注） |
| 10 | 零 Rust | 平台跑 `git diff crates/` | **空**（29 号 §1 核心不变量） |

- **不可跳步**：向导状态机按 1→10 串行，第 N 步判据未过不开放第 N+1 步（见 §4）。
- **写面纪律**：第 1/3/5/6/8/9 步写 `assets/maps/`+`dist/`+`board.json`+`ui_manifest.json`+`evidence/`（出图），须 `--enable-write`（§3 放开）；
  第 2/4/7/10 步只读（import/verify/check/zero_rust，无门无探针）。
- **第 9 步出图**：平台后台调 `TI_MAP=<id> treasure-bevy-native --shot --scene=main_map`（白名单 `map_shot`，写 `evidence/` 出图，gitignored 预览；写点 `evidence/` ⇒ 属写步须 `enable_write`）。

### 2.2 UI 工作台（固定 6 步，映射 29 号 §3 1:1）

| # | 步骤 | 平台动作 | 机器判据 |
|---|---|---|---|
| 1 | 设计稿先行 | `--shot --scene=<id> --viewport=<方向>`（当前状态出图） | 出图非黑；**用户视觉确认留痕**（平台记录：时间戳 + 用户消息，A 纪律硬规则 9） |
| 2 | 视觉变体器 | `ui_variant`（`scripts/ui_variant.py`，**新正式工具**，固化 S42.0 临时脚本：选资产组 + 着色参数 → 同路径替换/新路径 + 可选 ui_manifest 注册） | 资产生成 + `ui_manifest.json` 登记一致（disk↔registry 双向同步 validate_data 0 违规） |
| 3 | 版面（可选） | 平台表单改 `scenes.json` 原语（asset/坐标/配色）→ 写 scenes.json | `scenes.json` indent=1 规范形 + `check_scenes_canonical` 0 差 |
| 4 | 重建 | `cargo build`（嵌 scenes.json，29 号 §5 坑位：include_str! 编译期内嵌） | build 成功（scenes.json 生效） |
| 5 | 出图验收 | `--shot --scene=<id> --viewport=<方向>`（多视口） | 出图非黑 + bg 采样期望（变体色调验证） |
| 6 | 零 Rust + 签收 | 平台跑 `git diff crates/` + 视觉签收留痕 | **空** + `evidence/<id>.log` 含「用户视觉验收/过目/确认/签收」（doccheck2 第 35 项） |

- **第 2 步变体器**：固化 S42.0 的 PIL `tint_cool`（Color/Brightness 增强 + R×/G×/B× 冷移）为 `scripts/ui_variant.py`
  （登记 14 号 §3.2 + 工具契约：幂等 + `--check` 只读 + 非 0 退出 + 3 处登记）。
- **同路径替换**（29 号 §3 换 UI 核心判据）：变体器默认**覆盖原路径**（资产名/坐标/scenes.json/测试全不动 ⇒ 零 Rust + 零 scenes.json diff，S42.0 实证）；
  `--suffix` 选项出变体路径（供设计稿迭看，不落定）。
- **第 3 步版面可选**：纯视觉变体（着色）时跳过（0 scenes.json diff）；改版面/新元素才用。

### 2.3 打包/分发面

| 面 | 平台动作 | 写面 |
|---|---|---|
| 地图打包 | 地图向导第 5 步 `mappack_pack` → `dist/maps/<id>.map-pack/` | `dist/`（gitignored） |
| 产物构建 | `build_wasm` / `build_apk` / `build_native`（S35.1 **已有**，复用不重做） | `web/` / `dist/` / `target/` + gitignored 日志 |
| 发布 | `publish`（`publish_web.ps1` 编排，**新默认关**：gate 前置 + 重建 web + 静态断言 + release.md 发布历史 + gh-pages 推流） | `web/` + `docs/release.md` + gh-pages 远端 |
| 包/产物浏览 | 地图页已有 `dist/maps/` 清单 + 构建页产物清单 | 只读 |

- **发布 `publish` 单独默认关**（写 `docs/release.md` + 推远端，写面最大，须用户每次显式开启，镜像 22 号 §4.2 默认启用列口径）。

## 3. 白名单扩展与写面纪律修订（S44.1）

### 3.1 §4.2 新增动作（`console_commands.json` 机器真源 + 22 号 §4.2 人读视图同步）

| id | 命令 | 写面 | 默认启用 |
|---|---|---|---|
| `mappack_add` | `python3 map_pack.py add <id>` | `assets/maps/<id>/` | **否**（--enable-write） |
| `mappack_syncmap` | `python3 map_pack.py syncmap <id>` | `assets/maps/<id>/manifest.json` | **否** |
| `mappack_pack` | `python3 map_pack.py pack <id>` | `dist/maps/<id>.map-pack/` | **否** |
| `mappack_install` | `python3 map_pack.py install <path>` | `assets/maps/`（父目录，前缀豁免同包写）+ `assets/data/board.json` + `assets/maps/index.json` + `assets/ui_manifest.json`（机器真源全路径） | **否** |
| `mappack_sync` | `python3 map_pack.py sync` | `assets/data/{board,scenes,strings}.json` + `scene_manifest_v3.json` | **否** |
| `mappack_uninstall` | `python3 map_pack.py uninstall <id>` | `assets/maps/` + `.default` + `index.json` + `board.json` | **否** |
| `ui_variant` | `python3 ui_variant.py <asset-group> <params>` | `assets/ui/<name>.png` + 可选 `ui_manifest.json` | **否** |
| `map_shot` | `TI_MAP=<id> treasure-bevy-native --shot --scene=main_map` | `evidence/` 预览（gitignored 侧车） | 是 |
| `publish` | `pwsh publish_web.ps1` | `web/` + `docs/release.md` + gh-pages | **否** |

- **默认关列**：写 `assets/`/`dist/`/`web/`/`docs/` 的动作一律 `enabled_by_default=false`（需 `--enable-write` 显式开启，
  镜像 22 号 §4.2「会改仓库跟踪文件的写动作默认关闭」口径）；`map_shot` 只读出图默认开。
- **禁止不变**：参数只接受枚举/id（`<id>` = 平台自扫描 `assets/maps/` 已装包 ∪ 新建 id 白名单 `^[a-z0-9_]{1,16}$`，
  `<path>` = 平台按自己的已装包扫描拼出，不接受自由字符串）——22 号 §4.2 禁止 1/2/3 全保留。

### 3.2 写面纪律修订（22 号 §3.1 已知写点表 + §8 北极星）

- **§3.1 已知写点表**：新增上表 8 个写动作行（mappack_add/syncmap/pack/install/sync/uninstall/ui_variant/publish；map_shot 只读出图默认开不计；写点列见 §3.1）；表外写点发现 ⇒ 补表后再上线（原纪律保留）。
- **§8 北极星「0 游戏产物变更」口径**：从「唯一写点 = 地图导入口子（S32.15 import_probe）」**泛化为「白名单动作写点探针」**
  ——每个写动作登记其写点集（`console_commands.json` 加 `write_paths` 字段），动作前后取 `snapshot::FrozenSnapshot`，
  **豁免该动作登记写点 + gitignored 路径**，其余冻结路径（`crates/ crates/ti-core crates/ti-app crates/ti-engine crates/ti-render crates/ti-platform crates/ti-formats assets/ android/ tests/`）差异为空 ⇒ 0 游戏产物变更。
  `import_probe` 收敛为 `write_probe(动作写点集)`（S32.15 专用探针并入，行为不变）。
  > **gitignored 豁免为口径注记（当前未实现）**：`write_probe` 目前只豁免 `write_paths` 参数，未做 `git check-ignore` 遍历
  >（`snapshot.rs` 收集面已忽略 `.gitignore` 命中文件，故 gitignored 路径本就不入 `files` 快照，差异面自然豁免）——
  > 口径注记防误导 S44.2，实现无需改动。
  - **写点占位符解析约定（S44.2 向导接线必守）**：`console_commands.json` 写点可含 `<id>` 占位符（如 `assets/maps/<id>/manifest.json`），
    属**模板形态**（22 号 §4.2 前缀形态口径）。S44.2 向导以实际 id 执行动作时，**必须先解析占位符**（`<id>` → 实际包 id）再喂 `write_probe`，
    否则 diff 路径（`assets/maps/m01/...`）与模板（`assets/maps/<id>/...`）前缀不匹配 ⇒ 误判漂移（B 审计 F2 潜伏，S44.2 必触发）。
    解析规则 = 取实际 id 替换 `<id>`；`<id>.map-pack` → `<id>.map-pack`（pack 动作 dist 写点同规则）。
- **零 Rust 判据独立于快照**：`git diff crates/` = 空（动作写点探针只管「产物面」，零 Rust 是独立 `git diff crates/` 断言，两者都要）。

## 4. 模式化/固化机制（向导状态机 + 每步机器判据）

- **状态机**：`evidence/console/content-<type>-<id>.json`（gitignored；含 type/id/current_step/steps[] 每步 {done, cmd, exit_code, criterion, passed, ts}）。
  平台写点 = `evidence/console/`（22 号 §3.1 平台自身唯一写点）。
- **不可跳步**：`current_step` 严格串行；第 N 步 `passed=false` ⇒ 第 N+1 步 API 返回 409（前置未过）。
- **失败即停**：任一步非 0 / 判据未过 ⇒ 状态机 `halted`，回显失败判据，须修复后重跑该步（不自动回滚，镜像 22 号 §10.4）。
- **每步机器判据** = §2 各表「机器判据」列，平台解析脚本输出（0 违规/0 diff/0 漂移/出图非黑/`git diff crates/` 空）+ 工具退出码。
- **固化 = 步骤 + 判据都机器化**：29 号 §2/§3 的固定顺序 ↔ 平台向导步骤 1:1（S44.5 断言），不再靠人记步骤。

## 5. 工具扩展（S44.1，契约不变）

| 工具 | 改动 | 登记 |
|---|---|---|
| `scripts/map_pack.py` | 新增 `syncmap <id>`（manifest["map"]←map.json，幂等 + `--check` 只读 + 非 0 退出） | 14 号 §3.2 + 工具契约（幂等/只读/非 0/3 处登记） |
| `scripts/ui_variant.py` | **新**（固化 S42.0 临时脚本：tint_cool 着色 + 同路径替换/`--suffix` + `--register` + `--check` 只读 + 非 0 退出） | 14 号 §3.2 + 工具契约 |
| `console_commands.json` | 新增 9 动作 id（§3.1） | 22 号 §4.2 人读视图同步（id 集合对账断言 S32.6 自动覆盖） |

## 6. 关联与登记

- 单一事实源引用（不复述）：`29_content_ops.md`（流程/判据/坑位）、`21_map_package_spec.md`（I1~I9）、`22_dev_platform.md`（平台纪律）。
- 登记：`docs/README.md` + `AGENTS.md` 路由表 + `docs/process/00_index.md` 目录清单 + `14_tooling_catalog.md`（ui_variant 行）+ `15_process_ledger.md`（向导状态机 = 待固化项，挂 S44）。
- 平台页面视觉改动（硬规则 9）：S44.2/S44.3 落码前出**静态原型截图** `evidence/console/ui-s44-*.png` + 用户确认留痕（22 号 §11.1）。

## 7. 验收判据（S44 阶段门）

1. **端到端**：用平台地图向导跑一遍「改地图」（add→填→syncmap→verify→pack→install→check→sync→出图→零 Rust）+
   UI 向导跑一遍「换 UI 视觉」（设计稿→变体器→rebuild→出图→签收→零 Rust），每步判据回显，**最终 `git diff crates/` = 空**。
2. **不可跳步**：未过第 N 步判据时第 N+1 步 API 409（负例实测）。
3. **写面受控**：写动作默认关（无 `--enable-write` 401/403）+ 动作前后 `write_probe` 冻结路径 0 差（负例：越界写点 400）。
4. **判据不虚报**：每步判据 = 工具退出码 + 输出解析（0 违规/0 diff），非「命令成功即过」（29 号 §1 判据 4 + 硬规则 2）。
5. **零 Rust**：端到端后 `git diff crates/` = 空（含工具改动前的 S44.1 基线 vs 端到端后，产品源码 0 diff）。
6. **gate 全绿** + `doccheck`/`doccheck2` 0 error + 每任务双审计（22 号 §7）+ `evidence/S44.x.log`。
