# S32.3 双审计 r3 · B 侧（一致性审计）报告

> task: S32.3 · round: r3 · side: B · new: 0

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 收敛验证轮：核 r1 B 侧 4 条 findings（4-F1~4-F4）当前全部在位 + 任务书 8 项一致性面
> （文档 ↔ 代码 ↔ 台账 ↔ 白名单 ↔ 截图 五面对账）。**只读审计，0 写盘 0 改码 0 跑写子命令**
> （map_pack 仅实跑只读子命令 `verify m01` / `check`；禁 sync/install 未触碰）。

## 一、r1 B 侧 4 findings 当前在位核验（r3 主判据）

| r1 id | 落点 | 当前实测 | 在位 |
|---|---|---|---|
| 4-F1（§6.1 资产行 75 快照注） | 22 号 L167 | 「`ui_manifest` 全量登记图（**登记数随资产批次增长，S32.3 快照 = 75 张**）」+ WBS L30 同措辞「计数随批次增长，S32.3 快照 = 75」；实测 `ui_manifest.json` = **75 条**（快照值吻合，无新批次漂移） | ✓ |
| 4-F2（web/ 指代澄清） | 22 号 §6 引言 + §11.2 | §6 引言 L154-157「**web/ 指代澄清（防混淆，S32.3 双审计 r1 B 侧 4-F2）**：平台壳静态面 = `crates/ti-console/web/`…根 `web/` = S18.8a 游戏构建产物（`.gitignore` `/web/`）」+ §11.2 L360-363「**指代澄清（S32.3 双审计 B 侧 4-F2）**：本节 `web/*` = `crates/ti-console/web/`（平台壳）；**根 `web/` = S18.8a 游戏构建产物（wasm 交付件，gitignored，勿混）**」。r2-a P2-1 两处落点均实测在位 | ✓ |
| 4-F3（routes.rs 头注释） | `crates/ti-console/src/routes.rs` L1-2 | 「S32.3 看板 UI 静态面（**设计图/地图页增量，左菜单 7 页**）」+ L9「看板 UI v1（左菜单分组 7 页 = 概览/任务/证据/资产/设计图/地图/设置）」；首轮 5 页 stale 面已消除 | ✓ |
| 4-F4（14 号 verify 裸 id 面） | 14 号 §2.6 L154-163 | 参数面「可分发件/zip 路径，或**裸 id = 已装包目录** `assets/maps/<id>`」+ `python scripts/map_pack.py verify m01  # 裸 id（平台制作指引面）`；21 号 L181 同面；index.html L254 同面。三处一致 | ✓ |

## 二、全量一致性面（任务书 8 面逐面）

| # | 面 | 结论 | 证据 |
|---|---|---|---|
| 1 | 22 号 §6.1 页面矩阵 ↔ `web/index.html` | 一致 | 矩阵 10 行（L163-172）= 实码 7 实页（L288 `pages = ['overview','tasks','evidence','assets','design','map','settings']`）+ 3 占位行（门禁 P1 S32.4 L166 / 文档 P2 S32.8 L170 / 构建 P2 S32.9 L171，WBS L30-36 在列）逐行对上。设计图行 L168 = `designGroup()` 7 组（L489-498 组别判定 + L513 `order` 7 值；「岛型·地图」= 前端「岛型/地图」组别名，非漂移）+ lightbox（L545-552 设计稿格点击挂 `#lightbox`）+ 非运行截图标注（L231）；实测 `/api/evidence?sub=ui_concept` = 147 文件（146 png + 1 json，r3-a 实跑同值）。地图行 L169 = `loadMap()`（L556-581）描述符摘要字段（`map_packs_detail` L309-326 面 = schema/version/data_version/central_asset/isle_assets/spaces/ring/assets_count/invariants，含「31 格不变量」= `spaces`）+ `.default`（`default_map_pack` L318-325 白名单精确项）+ 制作指引 `<pre>`（L249-260）+ 只读（端点全 `resolve_guarded`，0 写盘）；实测 4 包 m01~m04 + default=m01，m04 manifest 在位无 `map` 键 = 前端草稿态 L568-570 |
| 2 | WBS S32.3 行（L30）↔ 实码 | DoD 可验 | 左菜单分组（L130-144 看板组 6 + 平台组 1）/ 资产页交互（L387-487 role 标签 + `generated_at` 倒序 + lightbox + AI prompt 元数据 L469-472）/ 设计图页（L489-554）/ 地图页（L556-581 + 制作指引）/ 证据页 `/api/audit`（`data.rs audit_list` L198 + `parse_audit_head` L246）/ 设置页 `/api/session`（L583-598 + `routes.rs` L50）/ 截图 7 张留痕（见面 4）/ 图片直读白名单前缀（`assetUrl` L411-414 `assets/ui/` 前缀）全对上；「gate 全绿 + 双审计通过 + evidence/S32.3.log」= 台账 verified 前置（S32.3 现 `in_progress` = 双审计未收敛前的如实态，非漂移） |
| 3 | `console_sources.json` 10 条 ↔ 22 号 §5 表 | 逐行 1:1 | JSON `repo_sources` 10 条 = `plan/ledger.yaml, docs/, evidence/, assets/gate_data.json, assets/ui_manifest.json, assets/registry.json, assets/maps/, assets/maps/.default, dist/maps/, assets/ui/`；22 号 §5 表 L136-147 源列 10 行逐行同名（顺序亦一致）；文件项精确 vs 目录项前缀语义与 §10.2 混合白名单口径吻合 |
| 4 | `evidence/S32.3.log` §六 ↔ 实码 | 在位 | §六（L112-124）列 7 张截图路径逐条对应，实测 `evidence/console/ui-s32.3-{overview,tasks,evidence,assets,design,map,settings}.png` **7/7 在盘非空**；「用户视觉验收通过」（L125）+ 3 项交互修订留痕在 §六；设计图/地图页记录（L120-124 + L143-172 区段）与 `routes.rs` `/api/evidence?sub=`（L5-6/L65-69）+ `/api/map`（L57-59）端点面一致 |
| 5 | 15 号第 24 行「已固化」↔ `map_pack.py` 实码 | 在位 | `cmd_sync` L465-482 = `board["maps"] = {}` 全量重算（残留清零/幂等注 L473-474）+ `if "map" not in manifest: continue` 无 map 键草稿包跳过回注（L477-480 注释锚「15 号第 24 行」）；`cmd_verify` L294-295 裸 id 解析为已装包目录（`MAPS_DIR / path.name / manifest.json` 判在位）；`cmd_check` L420-425 草稿包跳过对账 + board 残留段误报。**实跑**：`python3 scripts/map_pack.py verify m01` = 22 项不合规全为 **I7 哈希/字节漂移**（= S32.12 在途红项，见 §四），裸 id 解析本身成功（输出路径 = `assets/maps/m01`，非「目录不存在」类错）；`python3 scripts/map_pack.py check` = **0 漂移 EXIT=0**（4 包 + 默认 m01；m04 无 `board.json maps.m04` 残留段，草稿包跳过面实测生效） |
| 6 | web/ 指代澄清（4-F2 + r2 P2-1） | 双落点在位 + 0 混淆 | 22 号 §6 引言 L154-157 + §11.2 L360-363 两处实测在位（见 §一 4-F2）；14 号（L146/L223 `web/` = build_wasm 产物语境）/ 21 号（L162/L190 wasm 部署链 `web/` 语境）/ 15 号（0 处 web/ 指代）grep 实测均处游戏构建产物语境，**0 处把根 `web/` 误指平台壳或反之**；`crates/ti-console/web/` 实测仅 `index.html` 单文件（0 外部 JS/CSS 引用） |
| 7 | `map_pack.py` 改动面 ↔ 文档 | 口径一致 | 15 号第 24 行「**2026-10-30 已落码**」记录（载体 = `scripts/map_pack.py` L463 区段，实码 L465 区段吻合）；21 号 §8 L181 verify 裸 id 示例在位；14 号 §3.2 L207 登记行 `sync` = 「全部包 map→board.json + 默认包 layout→scenes/manifest/strings」（与实码 L484-505 写点面一致，且 22 号 §3.1 已知写点表 L45 `sync` 写点清单同口径）；index.html 制作指引 L251 `sync` 注释「全量重算 board.json maps 段（幂等；无 map 键的草稿包跳过）」与实码逐字一致；`check` L255「已装包漂移对账（0 漂移才绿）」+ `verify mXX` L254 裸 id 口径一致；21 号 §8 子命令表 6 行（pack/verify/install/uninstall/list/check）+ 22 号 §4.2 白名单 `mappack_verify`/`mappack_check` 只读面与实码一致 |
| 8 | `routes.rs` 头注释 ↔ 实码 | 无 stale | L1-2「S32.1 骨架 + S32.2 只读数据源 API + S32.3 看板 UI 静态面（设计图/地图页增量，左菜单 7 页）」；端点清单 L4-8 = 实码 L49-70 全在（`/api/health` L49 + `/api/overview` L63 + `/api/tasks` L64 + `/api/evidence` 含 `?sub=` L65-69 + `/api/assets` L70 + `/api/session` L50 + `/api/audit` L56 + `/api/map` L57-59 + `/api/whitelist` L51-55 + `/api/snapshot` L60-62 注释「只读数据源适配」括注覆盖）；内联 CSS 口径 L9-11（视觉令牌内联 `index.html <style>`，实测 L8-123）；L14 POST 骨架期 403 承接 = S32.4 面（22 号 §10.1）无 stale |

## 三、0 游戏产物 / 工具面实码抽查（一致性佐证）

- `assets/ui_manifest.json` = 75 条（15 role），与 22 号 §6.1「S32.3 快照 = 75 张」及 S32.3.log §二判据 5 抽样（bg_night_sea/icon_compass/isle_harbour 均在）一致。
- `assets/maps/` = m01~m04 + `_template` + `index.json`；`.default` = `m01`；m04 `manifest.json`（version `0.1.0-draft`，无 `map` 键）= 15 号第 24 行「m04 在途草稿态」复现面在位（`check` 0 漂移已证实跳过逻辑生效）。
- 冻结游戏产物面：本侧 0 写盘；`git status` 中 `dist/maps/m01.map-pack.*` / android 脏面 = S32.12 在途（S27.6 岛卡刷新后池资产未 sync 回写 m01 manifest 之同一根因），不计入。

## 四、误报排除（0 误报）

| 项 | 口径 |
|---|---|
| ti-engine CS-3 `ease_out` 单字母 / CS-2 `vs_duel.rs` 缺 `//!` | 在途红项（r3-a §五 同口径），S32.3 改动面 = 仅 `crates/ti-console`，不计入 |
| i18n 孤儿 key / android 脏面（`android/AndroidManifest.xml` 等 M 态） | 在途红项，不计入 |
| m01 I7 哈希漂移（`verify m01` 22 项不合规） | **S32.12 在途**（WBS §已知问题 L58「HEAD 即存在」）= 池资产未 sync 回写 manifest，非 S32.3 裸 id 解析缺陷（裸 id 解析本身成功定位 `assets/maps/m01` 目录）；不计入 |
| `dist/maps/` 为 git tracked（22 号 §5 注「`dist/` 被 .gitignore 忽略」） | 历史 S31 期既有口径（`dist/maps/*.map-pack` tracked 为 S31.1 产物）；S32.3 改动面 0 触碰 `dist/`，非 S32.3 漂移，不计入 |
| S32.3.log 抬头「15 号 L55」= 15 号第 22 行（平台 UI 视觉签收）行号指代 | S32.3.log L7 抬头括注语境 = 「4 文档（22 号 §6.1/§10.2 + WBS S32.3 行 + 15 号 L55）」，正文 §六 明示「15 号第 22 行」；15 号 §2 表 L55 = 第 22 行实测吻合 = 抬头 L 指文件行、正文指台账行号，双解自洽；非漂移 |

## 五、收敛判定（22 号 §7.3 四行机器字段 + §7.4 判据）

- **总判定 = PASS**；
- **new = 0**（任务书 8 面 + r1 4 findings 全在位核验，0 新增 finding）；
- **误报 = 0**（在途红项 4 类全排除，0 误报计入）；
- **残留 = 0**（r1 B 侧 4-F1~F4 + r2-a P2-1 共 5 条修复面全部实测在位，0 残留）；
- **收敛判定 = new 0 + 误报 0 + 残留 0 ⇒ 三条判据全满足**（连续 2 轮达成 = r2 非收敛因 A 侧残留 1，r3 双侧 0/0/0 为本任务收敛轮，机械判定归 S32.5 面）；
- 结构性缺口（门禁 33 项不覆盖 `map_pack.py verify`）= 15 号第 24 行 P2-2 挂 S32.7，非 S32.3 收敛阻塞项（0 残留口径不变）。

> 只读纪律自证：本侧实跑仅 `map_pack.py verify m01` / `check`（只读子命令）+ `git status` /
> `grep` / 文件读；0 sync / 0 install / 0 写盘 / 0 改码。
