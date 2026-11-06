# S44 内容运营平台化 WBS（TI Dev Console 固定流程向导）

> 定案 = [`process/30_content_ops_platform.md`](process/30_content_ops_platform.md)（定位/页面/白名单扩展/写面纪律/模式化机制/判据）。
> 执行纪律：按阶段门顺序验收 + 每任务双审计（22 号 §7）+ gate 全绿（镜像 S32 平台线）。
> 平台与游戏运行时解耦：**不改玩法/渲染/数值/`web/` 产物**（断言方式 = 22 号 §8 写点探针，30 号 §3.2）。

## 阶段总览

| 阶段 | 目标 | 任务 | 阶段门 |
|---|---|---|---|
| S44.0 定案 | 30 号定案 + WBS + 登记 | S44.0 | doccheck/doccheck2 0 error + gate 全绿 |
| S44.1 工具与写面 | `map_pack.py syncmap` + `ui_variant.py` 固化 + §4.2 白名单扩展 + §3.1/§8 写面纪律修订 | S44.1 | 工具契约全过 + 写动作默认关 + 北极星写点探针 0 差 |
| S44.2 地图工作台 | 10 步向导（add→填→syncmap→verify→pack→install→check→sync→出图→零 Rust）+ 状态机 | S44.2 | 端到端改地图 + 不可跳步 409 + 零 Rust |
| S44.3 UI 工作台 | 6 步向导（设计稿→变体器→版面→rebuild→出图→签收→零 Rust）+ 视觉签收留痕 | S44.3 | 端到端换 UI 视觉 + 零 Rust + visual-signoff |
| S44.4 打包/分发 | `dist/maps/` 浏览 + `publish` 编排（默认关） | S44.4 | publish 前置体检 + 写面受控 |
| S44.5 固化收口 | 29 号 §2/§3 步骤 ↔ 平台向导 1:1 断言 + 三连验证复盘 + 15 号固化工单 | S44.5 | 步骤映射断言 + gate 全绿 + 复盘记录 |

## 任务表

| id | 依赖 | 产物 | DoD（摘要） |
|---|---|---|---|
| S44.0 | S43.0 | 定案文档（30 号）+ 本 WBS + 登记 | 30 号要素齐全（§1~§7 页面/白名单/写面纪律/模式化/判据）+ WBS 任务表 + 登记（README/路由表/00_index/14 号/15 号）；doccheck + doccheck2 0 error；gate 全绿；evidence/S44.0.log |
| S44.1 | S44.0 | 工具扩展 + 白名单 + 写面纪律修订 | `map_pack.py syncmap`（幂等 + `--check` 只读 + 非 0 退出 + 3 处登记）；`ui_variant.py`（固化 S42.0 tint_cool，同路径替换/`--new-name`，`--check` 只读，非 0 退出，登记 14 号 §3.2）；`console_commands.json` 9 动作（§3.1）+ 22 号 §4.2 人读视图同步 + id 集合对账断言；§3.1 已知写点表 + §8 北极星写点探针（`write_probe` 泛化，`import_probe` 并入）；写动作默认关负例（无 --enable-write 401/403）；工具契约 `check_tooling_contract` PASS；gate 全绿 + evidence/S44.1.log |
| S44.2 | S44.1 | 地图工作台（向导状态机 + 10 步 + 判据回显） | `crates/ti-console/src/content/map_wizard.rs` + `routes/content_api.rs` + web `/content` 页地图向导；状态机（`evidence/console/content-map-<id>.json` gitignored）；10 步每步调白名单动作 + 判据解析回显；**不可跳步负例**（未过 N 步判据 → N+1 步 409）；**端到端**用平台跑 m22 复制改图（add→填→syncmap→verify→pack→install→check→sync→出图→零 Rust）每步判据过 + 最终 `git diff crates/` = 空；UI 原型截图用户确认留痕（22 号 §11.1）；gate 全绿 + evidence/S44.2.log |
| S44.3 | S44.1 | UI 工作台（6 步 + 视觉签收留痕） | `crates/ti-console/src/content/ui_wizard.rs` + web `/content` 页 UI 向导；6 步每步判据；**端到端**用平台跑 index 变体（设计稿出图→变体器→rebuild→出图→签收→零 Rust）每步判据过 + `git diff crates/` = 空 + `evidence/*.log` 含「用户视觉验收」；变体器同路径替换（资产名/坐标/scenes.json 不动）；UI 原型截图用户确认留痕；gate 全绿 + evidence/S44.3.log |
| S44.4 | S44.1 | 打包/分发面 | `dist/maps/` 清单页（已有地图页扩展）+ `publish` 白名单动作（默认关 + `publish_web.ps1 -Check` 只读体检 + 前置 gate 断言 + 非 0 退出上报）；`build_wasm`/`build_apk` 复用 S35.1；写面受控（publish 无 --enable-write 401/403）；gate 全绿 + evidence/S44.4.log |
| S44.5 | S44.2/S44.3/S44.4 | 固化收口 | 29 号 §2 十步 ↔ 地图向导 10 步 1:1 映射断言 + 29 号 §3 六步 ↔ UI 向导 6 步 1:1 映射断言（机器化：每步 id/判据/写面登记一致）；S40~S42 三连验证复盘入 15 号固化工单；`docs/process/29_content_ops.md` §2/§3 补「平台向导」列（流程 ↔ 平台 1:1）；gate 全绿 + evidence/S44.5.log |

## 每任务审计要求（强制，镜像 S32）

1. 主线程实现 + 自验，命令实跑输出入 `evidence/S44.x.log`（含 DoD 自身判据）。
2. 并行起两个子代理（只读、互不可见）：A 完成度（DoD 逐条）/ B 一致性（架构/规范/安全/依赖）。
3. findings 修复 → 复审计；0 新发现 + 0 误报 + 0 残留，连续 2 轮才算通过；连续 3 轮未过 ⇒ 拆子任务。
4. 报告落 `evidence/console/audit-S44.x-r<N>-{a,b}.md`，抬头含四行机器可解析字段（22 号 §7.3）。

## 不做

- 不做从零设计工具（29 号 §6 分界，另立项）；不做游戏编辑器/运行时/关卡（22 号 §9）。
- 不做任意 shell（只执行 §4.2 白名单动作 + 枚举/id 参数）；不引入数据库；不内嵌 LLM SDK。
- 平台不重写业务规则（`ti-core` 冻结）。

## 与现有资产的关系

| 现有 | 平台怎么用 |
|---|---|
| `scripts/map_pack.py` | 地图向导唯一执行源（`add/syncmap/verify/pack/install/check/sync`，写点见 30 号 §3.1） |
| `scripts/ui_variant.py`（新） | UI 向导唯一执行源（着色/同路径替换，写点见 30 号 §3.1） |
| `scripts/publish_web.ps1` | 发布面（`publish` 白名单默认关，写 `web/`+`release.md`+gh-pages） |
| `crates/ti-console/src/repo/import.rs` | 地图填内容落盘（S32.15 唯一写点，向导第 2 步复用） |
| `crates/ti-console/src/repo/snapshot.rs` | 北极星写点探针（`write_probe` 泛化，S44.1） |
| `docs/process/29_content_ops.md` | 流程单一事实源（S44.5 补「平台向导」列，1:1 映射断言） |
