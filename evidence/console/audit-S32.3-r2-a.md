# S32.3 双审计 r2 · A 侧（完成度审计）报告

> task: S32.3 · round: r2 · side: A · new: 1

```
总判定: PASS
新发现: 1
误报: 0
残留: 1
```

> 审计面 = S32.3 DoD（`docs/plan-s32-platform.md` L30）逐条对照 + r1 findings 修复核验（A-P2-A1~A5 + B-4-F1~F4，共 9 条）
> + 地图工具链健壮性面（15 号第 24 行已固化项）。
> **审计员自述**（非 finding）：本侧误判 1 次——`crates/ti-console/tests/data_tests.rs` 头双行
> （`#![allow...]` 行后紧跟 `//!` 头）误读为 data.rs 双 `///` 块同型残留，**已当场原样恢复**
> （`git status` 该文件 = `??` untracked 且内容恢复至审计前状态，0 改动面）。
> 误报 0 口径：既有在途红项 ti-engine CS-2/CS-3（`vs_duel.rs` 缺 `//!` 文件头 + `ease_out` 单字母）
> / i18n 孤儿 key / android 脏面 / 地图包 I7 漂移（S32.12 已立项）均不计入本侧。

## 一、r1 findings 修复核验（逐条）

### A 侧 P2（5 条）

| r1 id | 核验面 | 结论 | 证据 |
|---|---|---|---|
| P2-A1 | `crates/ti-console/web/` 死文件 | **已修 ✓** | 目录仅剩 `index.html`（`ls` 1 entry）；0 `<script src>`/外链 `<link href>`（仅 data: URI favicon）；`git status` = `?? crates/ti-console/web/`（整目录 untracked，与任务书口径一致） |
| P2-A2 | `routes.rs` L7-8 stale 注释 | **已修 ✓** | L9-11 = 「看板 UI v1（左菜单分组 7 页 = 概览/任务/证据/资产/设计图/地图/设置；原生 JS + CSS 全内联无 npm；视觉令牌内联于 `index.html` `<style>`）」，theme.css 表述已删 |
| P2-A3 | `data.rs` 双 `///` 文档块 | **已修 ✓** | `audit_list` 的 `///` 块上移为函数正上方；`session`（L366-367）独立 `///` 块紧邻 `pub fn session()`；L187-191 区段 = `/api/audit` 单一块 |
| P2-A4 | 22 号 §5 `assets/maps/<id>/` 语义 | **已修 ✓** | 22 号 §5 表第 7 行 = 「`assets/maps/`｜已装地图包**目录项前缀**（…`index.json` 在 `assets/maps/` 前缀内放行 = 包索引只读面）」，与 `console_sources.json` L9 前缀项 1:1 |
| P2-A5 | `evidence/console/.lock` 残留 | **已修 ✓（本侧复现并清理）** | 审计入场时锁文件在位（pid 3298317，`ps -p` 无进程 = stale）；按任务书实跑面 `rm -f` 后起服务，**测完 `pkill` + `rm .lock` 已执行**（当前 `ls` = No such file，0 残留） |

### B 侧 P2（4 条）

| r1 id | 核验面 | 结论 | 证据 |
|---|---|---|---|
| B-4-F1 | 22 号 §6.1 资产行「75 张」快照注 | **已修 ✓** | 22 号 L162 = 「`ui_manifest` 全量登记图（**登记数随资产批次增长，S32.3 快照 = 75 张**）」；WBS L30 同措辞「计数随批次增长，S32.3 快照 = 75」。实测 `ui_manifest.json` = **75 条**（与快照值吻合，无新批次漂移） |
| B-4-F2 | untracked web/ 指代澄清（22 号 §11.2） | **未修 ✗（残留）** | 任务书 r2 判据 = 「核 22 号 §11.2 是否澄清」；§11.2（L349-）现文字仅「`crates/ti-console/web/*`（JS/CSS 静态资源）不在上述扫描面」，**无 `crates/ti-console/web/`（平台壳）vs 根 `web/`（S18.8a 游戏构建产物，gitignored）两物对照措辞**，易混淆指代未落文档 |
| B-4-F3 | `routes.rs` L1 头注释 S32.3 增量面 | **已修 ✓** | L1-2 = 「S32.3 看板 UI 静态面（**设计图/地图页增量，左菜单 7 页**）」 |
| B-4-F4 | 14 号 `map_pack.py` 登记面 `verify m01` 裸 id 口径 | **已修 ✓** | 14 号 §2.6 L159-161 = 示例面「`verify <path>`/**或裸 id = 已装包目录** `assets/maps/<id>`」+ `python scripts/map_pack.py verify m01`（裸 id（平台制作指引面））；登记行 L205 `verify <path>` 口径；21 号 L180-181 同面（`verify m01 # 裸 id = 已装包目录（S32.3 平台制作指引面）`）三处一致 |

## 二、判据面（WBS S32.3 DoD，`docs/plan-s32-platform.md` L30）

| 判据 | 结论 | 证据 |
|---|---|---|
| 7 页 + 左菜单分组 | **✓** | index.html L288 `pages = ['overview','tasks','evidence','assets','design','map','settings']`；`.rail` 双 `.grp`：看板组 6 项（概览/任务/证据/资产/设计图/地图）+ 平台组（设置，L132-143） |
| 设计图页 = `evidence/ui_concept/` 147 张（非运行截图） | **✓** | `GET /api/evidence?sub=ui_concept` 实跑 200，files = **147**（146 png + 1 json `index_working_35.json`，png 计数 146 与 S32.3.log「147 张设计稿」口径 = 全目录 147 文件，前端 `.filter(f=>f.endsWith('.png'))` 收口）；前端 `designGroup` 7 组别标签〔主地图/首页/岛型·地图/对战/界面/资产表/其他〕+ lightbox + 页面标注「非运行截图」（L231/L244 区段） |
| 地图页 = `GET /api/map`（已装包清单 + manifest 描述符摘要 + `.default` + 制作指引 + 21 号链接；m04 草稿 valid=false 不崩；只读 0 写盘） | **✓** | 实跑 = `{count:4, default:m01}`，m01~m03 `manifest_present=true valid=true` + m04 `valid=false` + `draft_note`（manifest 在位无 map 键，前端渲染「草稿」态）；摘要面 = schema/version/data_version/central_asset/isle_assets/spaces/ring/assets_count/invariants（`map_packs_detail` L309-326）；前端制作指引 `<pre>` 命令面 + `docs/process/21_map_package_spec.md` 链接（L256 区段）；端点全 `resolve_guarded` 只读，单测 `map_packs_detail_lists` 断言 m04 `draft_note`（tests/data_tests.rs L52-75） |
| 越界 `sub=..` = 500 | **✓** | 实跑 `curl /api/evidence?sub=..` → **500**（`check_relative` `..` 段拒，Err 语义 = 500 如实上报，与任务书判据一致） |
| 直读 ui_concept png = 200 / 未登记 png = 404 | **✓** | `GET /evidence/ui_concept/asset_sheet_nodes.png` = **200**；`GET /assets/ui/definitely_not_registered_xyz.png` = **404**；登记 png（`assets/ui/bg_night_sea.png` 白名单前缀直读）= **200** |
| 设置页与 `/api/session` 配套 | **✓** | `GET /api/session` = `{token_present, write_enabled_default:false, single_instance_lock}`；前端 `loadSettings` token 粘贴存 `sessionStorage` |
| 7 张截图留痕 | **✓** | `evidence/console/ui-s32.3-{overview,tasks,evidence,assets,design,map,settings}.png` 7/7 在位；S32.3.log「用户视觉验收通过」+ 设计图/地图增量段（L120-123/L152-168） |
| 资产页 = ui_manifest 全量 + role 分组 + 时间倒序 + lightbox + AI prompt | **✓** | 前端 `renderAssets`（L386-485 区段）= role 标签 + `generated_at` 倒序 + lightbox + 元数据 prompt；计数随批次（快照 75，实测 75） |
| 证据页含双审计报告清单 `/api/audit` | **✓** | 实跑 = 72 份 `audit-*.md` 清单 + §7.3 机器字段解析（`parse_audit_head`），`audit-S32.3-r1-{a,b}.md` 在列 |
| 地图工具链健壮性（15 号第 24 行已固化） | **✓（代码面）** | `cmd_sync`：`board["maps"]={}` 全量重算 + `if "map" not in manifest: continue` 草稿包跳过（L474-482）；`cmd_check`：草稿包跳过对账 + 脏残留误报收口（L420-425）；`cmd_verify` 裸 id 解析（L294-295）；15 号第 24 行 = 「**2026-10-30 已落码**…已固化（S32.7 只挂门禁覆盖 P2-2）」；结构性缺口 P2-2（gate 不覆盖 verify）挂 S32.7 未越权关闭 ✓ |

## 三、实跑面（逐命令）

| 命令 | 结果 |
|---|---|
| `cargo build -q -p ti-console` | EXIT=0 |
| `cargo test -q -p ti-console` | 7 suite：4+4+4+4 + 0×3，**全 ok**（含 `map_packs_detail_lists` m04 draft 断言） |
| `cargo clippy -q -p ti-console --all-targets` | 0 warning / 0 error |
| `cargo tree -p ti-console` | 仅 `serde_json` + `ti-devmeta`，**0 游戏 crate**（grep `ti-core\|ti-platform\|ti-app\|ti-engine\|ti-render\|bevy\|macroquad` = 0 命中） |
| `python3 scripts/check_code_standards.py --check` | **FAIL（1 项超基线）** = ti-engine `CS-2 vs_duel.rs` 缺 `//!` 文件头（在途红项，S32.3 范围外，不计入）；CS-1/CS-3/CS-4 全 ok |
| `python3 scripts/check_ledger_reality.py` | 全绿（247 verified 三方对账 + 0 孤儿 evidence） |
| `python3 scripts/doc_numbers.py` | 0 漂移（33 项 / 247 verified 声称点一致） |
| 起服务 `/disk/target/debug/ti-console --host=0.0.0.0 --port=8787`（stale 锁已 `rm`，setsid 后台） | `/api/map` 4 包 + default=m01 ✓；`/api/evidence?sub=ui_concept` 147 文件 ✓；`sub=..` 500 ✓；ui_concept png 直读 200 / 未登记 404 ✓；**测完 `pkill` + `rm .lock` 已执行**（服务 0 残留进程） |
| `python3 scripts/map_pack.py verify m01`（裸 id 面，只读） | 裸 id **解析正常**（`-> assets/maps/m01`）但 **22 项 I7 不合规 EXIT=1**（池资产 `assets/ui/isle_*.png` 现值 ≠ m01 manifest 记录值；**非 S32.3 裸 id 解析缺陷 = I7 数据漂移，S32.12 已立项**） |
| `python3 scripts/map_pack.py check`（只读） | **0 漂移 EXIT=0**（4 包 + 默认 m01 布局对账全过） |

## 四、Findings

### P2-1：B-4-F2 残留 = 22 号 §11.2 无「平台壳 web/ vs 根 web/」两物澄清

**级别**：P2（文档口径精化，非功能/安全）。
**现象**：r1 B 侧 4-F2 判「任务书指代澄清，防后续误报」；r2 任务书判据 = 「核 22 号 §11.2 是否澄清」。
22 号 §11.2「覆盖边界（如实标注）」现仅写 `crates/ti-console/web/*`（JS/CSS 静态资源）不在门禁扫描面 +
「S32.3 判据兜底」，**未对照说明根 `web/`（S18.8a 游戏构建产物 `treasure-bevy-wasm`，`.gitignore /web/`）
是另一物**，两目录同裸名 `web/` 易在后续审计/任务书中混淆指代。
**证据**：22 号 L355-356 全文；§3 禁止 3（L115）已注「平台不执行完整构建 ⇒ 不重写根 `web/`」
但该注在 §3 侧，§11.2 覆盖边界段内无两物对照。
**修复建议**：22 号 §11.2 覆盖边界段补一句：「`web/` 两物：`crates/ti-console/web/` = 平台壳静态面
（`index.html` 单文件，JS/CSS 全内联）vs 根 `web/` = S18.8a 游戏构建产物（gitignored，平台 0 读写，§3 禁止 3）」。
**误报判定**：非误报（指向 S32.3 增量面文档，r1 findings 未修复面）。

## 五、误报排除说明（0 误报）

| 项 | 口径 |
|---|---|
| `verify m01` = 22 项 I7 不合规 | **S32.12 已立项**（plan-s32 §已知问题 L58：HEAD 即存在；S27.6 岛卡刷新后池资产未 `sync` 回写 m01 manifest）；非 S32.3 裸 id 解析缺陷（裸 id 解析本身正常 = 定位到 `assets/maps/m01` 且逐包 I7 哈希检查在跑）⇒ 不计入 new |
| ti-engine CS-2 `vs_duel.rs` 缺 `//!` 文件头 / CS-3 `ease_out` 单字母 | 既有在途红项（untracked `crates/ti-engine/src/vs_duel.rs` + `bin/android_entry.rs`），S32.3 改动面 = 仅 `crates/ti-console`，不计入 |
| i18n 孤儿 key / android 脏面 | 既有在途红项，不计入 |
| 本侧误判 `tests/data_tests.rs` 双行头 | 审计员自述误读（非 findings 面）；**已当场恢复原样**，0 改动面，不计入 |

## 六、收敛判定

- **new = 1**（P2-1，文档面澄清缺失）；
- **误报 = 0**；
- **残留 = 1**（= r1 B-4-F2 未修复面：22 号 §11.2 澄清未落）；
- **非收敛轮**（残留 1 > 0）：P2-1 落 22 号 §11.2 后，r3 三条判据（new 0 + 误报 0 + 残留 0）可判收敛；
  结构性缺口（gate 不覆盖 `verify`）挂 S32.7，非 S32.3 收敛阻塞项。
