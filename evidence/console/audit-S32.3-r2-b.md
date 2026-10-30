# S32.3 双审计 r2 · B 侧（一致性审计）报告

> task: S32.3 · round: 2 · side: B · new: 0

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 审计面 = 文档 ↔ 代码 ↔ 台账 ↔ 白名单 ↔ 截图 五面对账（只读，0 写盘 0 改码 0 跑写子命令；
> 仅 `verify`/`check` 只读实跑 + `git status`/`git log` 只读查询）。
> 误报 0 口径：既有在途红项 ti-engine CS-3 `ease_out` / i18n 孤儿 key / android 脏面 /
> S32.12 在途 m01 I7 哈希漂移（verify m01~m03 22 项，HEAD 即存在，`dist/maps` 未 commit，
> 台账 S32.12 行已 verified 留痕）均不计入本侧。

## 一、r1 findings 修复核验（逐条，B 侧 4 P2 + A 侧 5 P2）

### B 侧 r1（`audit-S32.3-r1-b.md` 4-F1~F4）

| r1 id | 核验面 | 结论 | 证据 |
|---|---|---|---|
| 4-F1 | 22 号 §6.1 资产行「75 张」快照注 | **已修 ✓** | 22 号 L167 = 「`ui_manifest` 全量登记图（**登记数随资产批次增长，S32.3 快照 = 75 张**）」；WBS L30 同措辞「计数随批次增长，S32.3 快照 = 75」。实测 `ui_manifest.json` = **75 条**（与快照值吻合，无新批次漂移） |
| 4-F2 | web/ 指代澄清（22 号 §6 引言 + §11.2 两处） | **已修 ✓（两处均落）** | 22 号 L154-157（§6 引言）「**web/ 指代澄清（防混淆，S32.3 双审计 r1 B 侧 4-F2）**：平台壳静态面 = `crates/ti-console/web/` …… 根 `web/` = S18.8a 游戏构建产物（`.gitignore` `/web/`），是另一物」+ §11.2 L362 区段「**指代澄清（S32.3 双审计 B 侧 4-F2）**：本节 `web/*` = `crates/ti-console/web/`（平台壳）；**根 `web/` = S18.8a 游戏构建产物（wasm 交付件，gitignored，勿混）**」。r2 任务书 = 「两处」，两处均实测在位 |
| 4-F3 | `routes.rs` 头注释 S32.3 设计图/地图增量面 | **已修 ✓** | L1-2 = 「S32.3 看板 UI 静态面（**设计图/地图页增量，左菜单 7 页**）」+ L9「看板 UI v1（左菜单分组 7 页 = 概览/任务/证据/资产/设计图/地图/设置）」，首轮 5 页 stale 面已消除 |
| 4-F4 | 14 号 `map_pack.py` verify 裸 id 登记面 | **已修 ✓** | 14 号 §2.6 L160-163 = 参数面「可分发件/zip 路径，或**裸 id = 已装包目录** `assets/maps/<id>`」+ `python scripts/map_pack.py verify m01  # 裸 id（平台制作指引面）`；21 号 L181 同面（`verify m01  # 裸 id = 已装包目录（S32.3 平台制作指引面）`）；index.html 制作指引 `verify mXX # 裸 id = 已装包目录`。三处一致 |

### A 侧 r1（`audit-S32.3-r1-a.md` P2-A1~A5）

| r1 id | 核验面 | 结论 | 证据 |
|---|---|---|---|
| P2-A1 | `web/app.js` + `web/_shotpage.js` 死文件 | **已修 ✓** | `ls crates/ti-console/web/` = **仅 `index.html`**（1 项）；0 `<script src>` / 外链 `<link href>`（仅 data: URI favicon L7）；`grep -r "app.js\|_shotpage\|theme.css" index.html` = 0 命中 |
| P2-A2 | `routes.rs` L7-8 stale 注释 | **已修 ✓** | L7 改「看板 UI v1（左菜单分组 7 页 = …）」；L10 theme.css 表述 = 「视觉令牌内联于 `index.html` `<style>`」（theme.css 0 引用） |
| P2-A3 | `data.rs` 双 `///` 文档块 | **已修 ✓** | `audit_list` 文档块（L187-197）上移至函数正上方；`session`（L366-367）独立 `///` 块紧邻 `pub fn session()`（L368），双块合并面已消除 |
| P2-A4 | 22 号 §5 `assets/maps/` 语义 | **已修 ✓** | 22 号 §5 表 L144 = 「`assets/maps/` ｜ 已装地图包**目录项前缀**（…`index.json` 在 `assets/maps/` 前缀内放行 = 包索引只读面）」，与 `console_sources.json` L9 前缀项 1:1 |
| P2-A5 | `evidence/console/.lock` 残留 | **已修 ✓（当前在位核 = 无锁）** | `ls evidence/console/.lock` = **No such file**（A 侧 r2 审计入场时 stale 锁 pid 3298317 已被复现清理，0 残留） |

**残留 = 0**：r1 双侧 9 条 findings（B 4 + A 5）全部修复在位。

## 二、一致性审计面（8 面逐项结论）

| # | 审计面 | 结论 | 依据 |
|---|---|---|---|
| 1 | 22 号 §6.1 页面矩阵 ↔ index.html | 一致 | 矩阵 10 行 = 实码 7 实页（L288 `pages = ['overview','tasks','evidence','assets','design','map','settings']`）+ 3 占位行（门禁 P1 S32.4 / 文档 P2 S32.8 / 构建 P2 S32.9，WBS L30-36 在列）逐行对上；设计图行 = `designGroup()` 7 组（L513 `order = ['主地图','首页','岛型/地图','对战','界面','资产表','其他']`，22 号「岛型·地图」= 前端「岛型/地图」组别名，非漂移）+ lightbox（L109-119 `#lightbox` 全屏 92vw×88vh + L547 区段 `lightbox-cap`）+ 非运行截图（L231/L504 页面标注）；地图行 = `loadMap()`（L556-579）描述符摘要（`map_packs_detail` schema/version/data_version/central_asset/isle_assets/spaces/ring/assets_count）+ `.default`（`default_map_pack` L318-325 白名单精确项）+ 制作指引 `<pre>`（L241-262 命令面 + 21 号链接）+ 只读（端点全 `resolve_guarded`，0 写盘） |
| 2 | WBS S32.3 行（L30）↔ 实码 | DoD 可验 | 左菜单分组（L130-143 看板组 6 + 平台组 1）/ 资产页交互（L386-485 role 标签 + `generated_at` 倒序 + lightbox + AI prompt 元数据）/ 设计图页（L488-553）/ 地图页（L555-577 + 制作指引）/ `/api/audit`（`data.rs audit_list` + `parse_audit_head`）/ 截图 7 张（在位，见 4）全对上；浏览器 0 console error + 首屏 <2 s 用户侧补录口径 = S32.3.log §六（0 虚报标注，非本侧漂移） |
| 3 | `console_sources.json` 10 条 ↔ 22 号 §5 表 | 10 条 1:1 | `plan/ledger.yaml`↔L138 / `docs/`↔L139 / `evidence/`↔L140 / `gate_data`↔L141 / `ui_manifest`↔L142 / `registry`↔L143 / `assets/maps/`（目录项前缀）↔L144 / `assets/maps/.default`（精确项）↔L145 / `assets/ui/`↔L146 / `dist/maps/`↔L147；`_comment` 口径注（混合白名单 + plan/ 单文件精确项 + dist/ 缺失不报错）与 §5/§10.2 一致；**设计图/地图面新增端点（`/api/evidence?sub=ui_concept` + `/api/map`）无新增数据源项**（全走既有 `evidence/` 目录项前缀 + `assets/maps/` + `.default` 白名单，0 越权扩白名单） |
| 4 | `evidence/S32.3.log` §六 ↔ 实码 + 截图 | 一致 | 7 张截图在位（`ui-s32.3-{overview,tasks,evidence,assets,design,map,settings}.png`，ls 实测 7/7：09:28×5 + 11:19/11:22×2 = 设计图/地图增量批次落盘时间合理）；§六设计图段（L152-158：`sub=ui_concept` 147 张 + 组别标签 + lightbox + 非运行截图）与前端 `loadDesign`/`designGroup`/`#lightbox` 逐面对上；§六地图段（L159-168：`map_packs_detail` + m04 空骨架不崩 + `.default` 精确项 + 制作走 `map_pack.py` 0 写盘）与前端 `loadMap` + `data.rs` 逐面对上；「用户视觉验收通过」留痕在位（L115-124 + L126-130 三项交互修订 = 用户过目后拍板，A 纪律 22 号 §11.1） |
| 5 | 15 号第 24 行「已固化」实跑面 | 属实 | `python3 scripts/map_pack.py verify m01`（裸 id）= **解析正常**（`-> assets/maps/m01`）但 22 项 I7 不合规 EXIT=1（池资产 `assets/ui/isle_*.png` S27.6 刷新后现值 ≠ m01 manifest 记录值 = **S32.12 在途 I7 漂移面**，台账 S32.12 行已 verified，本侧不计）；`check`（只读）= **0 漂移 EXIT=0**（4 已装包 vs board.json + 默认包 m01 布局对账全过，草稿包 m04 无 `map` 键跳过对账逻辑实跑验证 = 15 号第 24 行「草稿包跳过对账（仅脏残留误报）」口径与 L420 区段注释一致） |
| 6 | web/ 目录面 | 一致 | `crates/ti-console/web/` = **仅 `index.html`**（609 行，`app.js`/`theme.css`/`_shotpage.js` 已删）；index.html 0 外部 JS/CSS 引用（`<script>` L285 内联 + `<style>` L7 区段全内联，无外链 `src=`/`href=`，仅 data: URI favicon）；根 `web/`（S18.8a 游戏构建产物 8 构建件 + `.gitignore:36 /web/`）= 另一物，22 号 §6 引言（L154-157）+ §11.2（L362）两处指代澄清在位 |
| 7 | `map_pack.py` 改动面 ↔ 文档 | 一致 | `cmd_sync`（L465-482 区段：无 `map` 键包 `continue` 跳过 + `board["maps"] = {}` 全量重算幂等，与 15 号第 24 行「已落码」记录逐字对上）/ `cmd_verify`（L289-295 裸 id 解析 `MAPS_DIR / path.name / "manifest.json"` 回退，21 号 L181 + 14 号 L163 三处示例面同口径）/ `cmd_check`（L406-425 区段草稿包跳过对账 + 脏残留误报收口）；**唯一缺口 = S32.7 第 24 行标注「gate 33 项不覆盖 verify」**（15 号第 24 行如实挂账未越权关闭，与 21 号 §8 工具流水线 + 14 号 §3.2 `map_pack.py` 登记行 8 子命令口径一致） |
| 8 | `routes.rs` 头注释 ↔ 实码 | 一致 | L1-14 头注释全口径 = 实码：7 页（`pages` 数组 7 值）+ 内联 CSS（L10「视觉令牌内联于 `index.html` `<style>`」+ 0 外部静态资源依赖）+ `/api/evidence?sub=`（L5-6「含 `?sub=<dir>` 子目录枚举，设计图页 `sub=ui_concept`」+ L65-69 `sub=` 解析实码）+ `/api/map`（L7「`/api/map` = 只读数据源适配」+ L57-59 `map_packs_detail` 实码）；**r1 stale 面（L1 未标 S32.3 增量）已消除**（L1-2 = 「S32.3 看板 UI 静态面（设计图/地图页增量，左菜单 7 页）」） |

## 三、误报排除说明（0 误报）

| 项 | 口径 |
|---|---|
| `verify m01~m03` = 22 项 I7 不合规 EXIT=1 | **S32.12 在途 I7 漂移面**（台账 S32.12 行 verified + `dist/maps` 未 commit = 在途脏面，S27.6 岛卡刷新后池资产未 `sync` 回写 manifest）；本侧裸 id 解析缺陷核验面 = 解析正常（定位 `assets/maps/m01`），非 S32.3 改动面 ⇒ 不计入 new |
| ti-engine CS-3 `ease_out` / i18n 孤儿 key / android 脏面 | 既有在途红项（S32.3 改动面 = 仅 `crates/ti-console`），不计入 |
| `evidence/ui_concept/` = 147 文件（146 png + 1 json `index_working_35.json`，gitignored `/evidence/ui_concept/`） | 非 S32.3 改动面（S32.3 只读消费面），前端 `.filter(f=>f.endsWith('.png'))` 收口逻辑与 S32.3.log「147 张设计稿」口径 = 全目录 147 文件一致，非漂移 |
| `assets/maps/m04/` 未跟踪 + 11 项不合规 | S16.4 在途空骨架（S32.12 DoD「不纳入」口径），`check` 草稿包跳过逻辑正确，非漂移 |

## 四、收敛判定

- **new = 0**（8 面全一致；`map_pack.py` gate 覆盖缺口 = S32.7 第 24 行既有挂账非 S32.3 新增）；
- **误报 = 0**（4 项排除均指向 S32.3 范围外在途红项，非本侧漂移）；
- **残留 = 0**（r1 双侧 9 条 findings = B 4-F1~F4 + A P2-A1~A5 全部修复在位；4-F2 两处澄清实测均在位）；
- **收敛轮判定 = PASS（new 0 + 误报 0 + 残留 0）**。

> 本侧 0 新发现 ⇒ r2 与 r1-a（A 侧 new 1 = B-4-F2 残留）**不构成收敛轮**（A 侧残留 1 须 r3 复核清零）；
> 结构性缺口（gate 不覆盖 `map_pack.py verify`）挂 S32.7，非 S32.3 收敛阻塞项。
