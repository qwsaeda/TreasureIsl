# S32.3 双审计 r1 · B 侧（一致性审计）报告

> task: S32.3 · round: 1 · side: B · new: 4

```
总判定: PASS
新发现: 4
误报: 0
残留: 0
```

> 审计面 = 文档 ↔ 代码 ↔ 台账 ↔ 白名单 ↔ 截图 五面对账（只读；误报 0：不报既有在途红项
> ti-engine CS-3 ease_out / i18n 孤儿 key / android 脏面 = S32.3 范围外）。
> **审计员自述**：验证 15 号第 24 行时误跑 `map_pack.py sync`（写子命令）改写 5 个 tracked
> JSON（`assets/maps/{m01,m02,m03,_template}/manifest.json` + `assets/data/strings.json`），
> 已用 `git checkout --` 完整恢复（`git status` 复原至审计前状态，m04 未跟踪面不受影响）；
> 恢复后 `assets/maps` 与 `assets/data` 0 脏面。后续验证仅用 `verify`/`check` 只读子命令。

## 审计面逐项结论

| # | 审计面 | 结论 | 依据 |
|---|---|---|---|
| 1 | 22 号 §6.1 矩阵 ↔ index.html | 一致 | 矩阵 10 行（§6.1 L156-167）= 实码 7 页 + 3 行 P1/P2 占位（门禁 S32.4 / 文档 S32.8 / 构建 S32.9，DoD 明示 P0）；逐行内容描述与实码功能全对上（概览/任务/证据/资产/设计图/地图/设置），设计图行 = 组别标签 7 类（L512 `order` 7 值）+ lightbox（L547）+ 非运行截图（L230）；地图行 = 描述符摘要 + `.default` + 制作指引 + 只读（L241-259）；资产行「全量 75 张」= 台账 247/75 口径快照（当前 80 = S32.3 后新批次，非漂移，见 4-F1） |
| 2 | WBS S32.3 行（L30）↔ 实码 | DoD 可验 | 左菜单分组（L130-143）/ 资产页交互（L386-485）/ 设计图页（L488-553）/ 地图页（L555-577）/ `/api/audit`（data.rs `audit_list` + `parse_audit_head` L179-242）/ 截图 7 张（在位，见 4）全对上；浏览器 0 console error + 首屏 <2 s 留痕 = S32.3.log §六（判据 2/3 口径「用户侧浏览器终验补录」已 0 虚报标注，非本侧漂移） |
| 3 | console_sources.json ↔ 22 号 §5 | 10 条 1:1 | 逐行对照：`plan/ledger.yaml`↔L138 / `docs/`↔L139 / `evidence/`↔L140 / `gate_data`↔L141 / `ui_manifest`↔L142 / `registry`↔L143 / `assets/maps/`↔L144（目录项前缀 = §5「已装地图包目录项前缀」，前缀语义一致）/ `assets/maps/.default`↔L145 / `dist/maps/`↔L147 / `assets/ui/`↔L146；`_comment` 口径注与 §10.2「assets/ 只返回登记过的文件」一致 |
| 4 | S32.3.log §六 ↔ 实码 + 截图 | 一致（1 措辞瑕疵） | 7 张截图在位（`ui-s32.3-{overview,tasks,evidence,assets,design,map,settings}.png`，ls 实测 7/7，mtime 09:28×5 + 10:51×2 = 设计图/地图增量批次落盘时间合理）；§六地图段「`cmd_check` 草稿包跳过对账（仅脏残留误报）」= 实测 `check` 0 漂移（m04 草稿态 manifest 有但无 `map` 键，L463 区段 `if "map" not in manifest: continue` 跳过回注 + `cmd_check` 容错口径与 L57 第 24 行措辞一致） |
| 5 | 15 号第 24 行「已固化」| 属实 | `verify m01` 0 违规 EXIT=0；`check` 0 漂移 EXIT=0；`verify m04` 草稿态 11 项不合规 EXIT=1（骨架未填如实报，不崩）；sync 幂等可重放（已恢复副作用）；结构缺口 P2-2（gate 不覆盖 verify）仍挂 S32.7 第 24 行标注，未越权关闭 |
| 6 | web/ 目录面 | 一致（1 指代瑕疵） | `crates/ti-console/web/` = index.html 单文件（29 KB，10:54，app.js/theme.css/_shotpage.js 已删）；index.html 0 外部 JS/CSS 引用（`<script>` L284 / `<style>` L7 全内联，无 `<link>`/`src=` 外部引用）；**根 `web/`**（S18.8a 游戏构建产物，`.gitignore:36 /web/`）含 8 个构建件 = 另一物，任务书「untracked web/ 现应 = index.html 单文件」句指代平台壳（crates/ti-console/web/），两物不混淆（见 4-F2） |
| 7 | routes.rs 头注释 ↔ 实码 | 一致（1 stale 面） | L7「7 页」= 实码 `pages` 数组 7 值（overview/tasks/evidence/assets/design/map/settings）；L8「视觉令牌 = 航海夜·精修」= 内联 `<style>` 暗色铜蓝令牌（对齐 10 号 §3 v2）；L9「单文件交付，0 外部静态资源依赖」= index.html 实测成立；**stale = L1 头注释未标 S32.3 设计图/地图增量面**（L7「看板 UI v1」括注仍停在首轮 5 页形态，设计图/地图页为 2026-10-30 增量，头注释未同步 = 轻微漂移） |
| 8 | 21 号 spec 链接面 | 一致 | index.html L256 `docs/process/21_map_package_spec.md` 真实存在（ls 在位）+ data.rs `spec` 字段 L308 同值 + 14 号 L154 引用同值，三处路径口径一致 |
| 9 | 14 号工具台账 map_pack.py 登记行 | 在位（1 缺口） | L205 登记行 8 子命令 `pack/verify/install/add/uninstall/list/check/sync` 全列 + L159-167 流水线示例；**缺口 = L154-167 示例面未登 `verify m04` 裸 id 口径（S32.3 L463 区段「`cmd_verify` 裸 id 解析为已装包目录」使用面）**，示例面 `verify dist/maps/m01.map-pack`（L161）与 `verify m01`（已装包目录）两种形态未并列，登记面未随 S32.3 增量同步（见 4-F4） |

## findings（4 条 P2，0 P0/P1）

| id | 严重度 | 位置 | 描述 | 建议 |
|---|---|---|---|---|
| 4-F1 | P2 | `docs/process/22_dev_platform.md` L162 §6.1 资产行 | 「全量 75 张」为台账 247/75 口径快照；当前 ui_manifest = 80 张（S32.3 后新批次），文档值 75 漂移 | 随 247→253 门数漂移一并回写「全量 <N> 张」为「ui_manifest 全量（当前 N，随批次漂移）」或保持快照但标注「时点值」 |
| 4-F2 | P2 | 任务书 §6（web/ 目录面） | 任务书句「untracked `web/` 现应 = index.html 单文件（app.js/theme.css/_shotpage.js 已删）」指代 `crates/ti-console/web/`（平台壳），与根 `web/`（S18.8a 游戏构建产物，`.gitignore /web/`）两物易混淆 | 任务书/报告抬头处加「`crates/ti-console/web/`（平台壳）vs 根 `web/`（游戏构建产物，gitignored）」指代澄清，防后续误报 |
| 4-F3 | P2 | `crates/ti-console/src/routes.rs` L1 头注释 | 头注释「S32.1 骨架 + S32.2 只读数据源 API + S32.3 看板 UI 静态面」未标 S32.3 设计图/地图增量面（L7 括注停首轮 5 页形态），轻微 stale | 头注释 L1/L7 补「S32.3 增量 = 设计图页 + 地图页（2026-10-30）」一行 |
| 4-F4 | P2 | `docs/process/14_tooling_catalog.md` L154-167 §2.6 示例面 | S32.3 L463 区段「`cmd_verify` 裸 id 解析为已装包目录」使用面未在 14 号 L161 示例面并列（现示例 = `verify dist/maps/m01.map-pack`，裸 id 形态 `verify m01` 未登） | L161 示例面加 `# 或已装包目录裸 id：python scripts/map_pack.py verify m01` 一行 |

## 误报口径说明

- **0 误报**：上述 4 条均指向 S32.3 增量面（设计图/地图页 + 15 号第 24 行 + 14 号登记面）内的真实漂移，
  非既有在途红项（ti-engine CS-3 ease_out / i18n 孤儿 key / android 脏面 = S32.3 范围外，未报）。
- **审计员自述面**（非 finding）：误跑 `sync` 写子命令改写 5 个 tracked JSON 已完整恢复（见抬头注），
  恢复后 `assets/maps` + `assets/data` 0 脏面；`m04/` 未跟踪面不受影响（草稿态在途，S16.4 另挂）。
  后续验证仅用 `verify`/`check` 只读子命令，0 写盘。

## 收敛面

- 4 P2 建议 = 新发现 4（文档面漂移 + 头注释 stale + 登记面缺口，全部 P2 级「口径精化/登记补齐」类）；
- 误报 0；残留 0（无阻断性 P0/P1）；
- 建议吸收：4-F1/4-F3/4-F4 = 文档/注释 3 处小幅补齐；4-F2 = 任务书指代澄清（防后续误报）；
- 主线程独立复核（§7.5）后连续 2 轮 0/0/0 收敛 ⇒ S32.3 可置 verified。
