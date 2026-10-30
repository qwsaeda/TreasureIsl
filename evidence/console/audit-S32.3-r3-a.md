# S32.3 双审计 r3 · A 侧（完成度审计）报告

> task: S32.3 · round: r3 · side: A · new: 0

```
总判定: PASS
新发现: 0
误报: 0
残留: 0
```

> 审计面 = S32.3 DoD（`docs/plan-s32-platform.md` L30）逐条对照 + r2-a P2-1 修复核验（B-4-F2 残留）
> + 地图工具链健壮性面（15 号第 24 行已固化项）。**收敛验证轮**。

## 一、r2-a P2-1 修复核验（B-4-F2 残留：22 号 §11.2 无「平台壳 web/ vs 根 web/」两物对照）

**已修 ✓**（双落点，均含指代澄清）：

| 落点 | 位置 | 实文 |
|---|---|---|
| §6 引言 | `docs/process/22_dev_platform.md` L154-157 | 「**web/ 指代澄清（防混淆，S32.3 双审计 r1 B 侧 4-F2）**：平台壳静态面 = `crates/ti-console/web/`（`index.html` 单文件交付，JS/CSS 全内联，0 外部静态资源依赖）；根 `web/` = S18.8a 游戏构建产物（`treasure-bevy-wasm` + HTML 胶水，`.gitignore` `/web/`），是另一物——平台「不执行完整构建 ⇒ 不重写根 `web/`」（§3 禁止 3）≠ 平台壳 `crates/ti-console/web/`。」 |
| §11.2 覆盖边界 | L360-363 | 「**指代澄清（S32.3 双审计 B 侧 4-F2）**：本节 `web/*` = `crates/ti-console/web/`（平台壳）；**根 `web/` = S18.8a 游戏构建产物（wasm 交付件，gitignored，勿混）**。」 |

修复建议原文（r2-a P2-1）= 「22 号 §11.2 覆盖边界段补一句：`web/` 两物对照」。
实际修复 = **两处**（§6 引言 + §11.2 覆盖边界）均含明确两物指代澄清，措辞与建议一致且更完整（含 §3 禁止 3 交叉引用）。

**残留 = 0**。

## 二、判据面（WBS S32.3 DoD，`docs/plan-s32-platform.md` L30）

| 判据 | 结论 | 证据 |
|---|---|---|
| 7 页 + 左菜单分组 | **✓** | `index.html` L288 `pages = ['overview','tasks','evidence','assets','design','map','settings']`；`.rail` 双 `.grp`：看板组 6 项（概览/任务/证据/资产/设计图/地图，L132-140）+ 平台组（设置，L141-144） |
| 设计图页 = `evidence/ui_concept/` AI 设计稿（GET /api/evidence?sub=ui_concept + 组别标签 + lightbox） | **✓** | 实跑 `/api/evidence?sub=ui_concept` = 147 文件（146 png + 1 json）；前端 `designGroup` 7 组别标签〔主地图/首页/岛型/地图/对战/界面/资产表/其他〕（L489-499）+ lightbox（L544-552）+ 页面标注「非运行截图」（L231） |
| 地图页 = GET /api/map（已装包清单 + 描述符摘要 + .default + 制作指引 + 21 号链接；m04 草稿 valid=false 不崩；只读 0 写盘） | **✓** | 实跑 = `{count:4, default:m01}`；m01~m03 `valid=true` + m04 `valid=false` + `draft_note`（manifest 在位无 map 键，前端渲染「草稿」态 L568-570）；摘要面 = schema/version/data_version/central_asset/isle_assets/spaces/ring/assets_count/invariants（`map_packs_detail` L309-326）；前端制作指引 `<pre>` 命令面（L250-255）+ `docs/process/21_map_package_spec.md` 链接（L257）；端点全 `resolve_guarded` 只读（L276），单测 `map_packs_detail_lists` 断言 m04 `draft_note`（tests/data_tests.rs L51-77） |
| 越界 `sub=..` 拒 | **✓** | 实跑 `curl /api/evidence?sub=..` → **500**（`check_relative` `..` 段拒，`路径 evidence/../ 含 .. 段（穿越拒绝）`） |
| 直读 ui_concept png = 200 / 未登记 png = 404 | **✓** | `GET /evidence/ui_concept/asset_sheet_nodes.png` = **200**；`GET /assets/ui/definitely_not_registered_xyz.png` = **404**；登记 png `assets/ui/bg_night_sea.png` = **200** |
| 设置页与 /api/session 配套 | **✓** | `GET /api/session` = `{token_present, write_enabled_default:false, single_instance_lock}`；前端 `loadSettings` token 粘贴存 `sessionStorage`（L583-598） |
| 7 张截图留痕 | **✓** | `evidence/console/ui-s32.3-{overview,tasks,evidence,assets,design,map,settings}.png` 7/7 在位（ls 实测）；S32.3.log §六「用户视觉验收通过」 |
| 资产页 = ui_manifest 全量 + role 分组 + 时间倒序 + lightbox + AI prompt | **✓** | 前端 `renderAssets`（L424-451）= role 标签分组 + `generated_at` 倒序（L431-437）+ lightbox（L474-486）+ 元数据含 prompt（L469-472）；计数随批次（快照 75，实测 75） |
| 证据页含双审计报告清单 /api/audit | **✓** | 实跑 = 74 份 `audit-*.md` 清单 + §7.3 四行机器字段解析（`parse_audit_head` L246-261），`audit-S32.3-r{1,2,3}-{a,b}.md` 在列 |
| 地图工具链健壮性（15 号第 24 行已固化） | **✓** | `cmd_sync`：`board["maps"]={}` 全量重算 + `if "map" not in manifest: continue` 草稿包跳过（L474-482）；`cmd_check`：草稿包跳过对账 + 脏残留误报收口（L420-425）；`cmd_verify` 裸 id 解析（L294-295）；15 号第 24 行 = 「**2026-10-30 已落码**…已固化（S32.7 只挂门禁覆盖 P2-2）」 |

## 三、实跑面（逐命令）

| 命令 | 结果 |
|---|---|
| `cargo build -q -p ti-console` | EXIT=0 |
| `cargo test -q -p ti-console` | 7 suite：4+4+4+4 + 0×3，**全 ok**（含 `map_packs_detail_lists` m04 draft 断言） |
| `cargo clippy -q -p ti-console --all-targets` | 0 warning / 0 error |
| `cargo tree -p ti-console` | 仅 `serde_json` + `ti-devmeta`，**0 游戏 crate**（grep 全 0 命中） |
| `python3 scripts/check_code_standards.py --check` | **FAIL（1 项超基线）** = ti-engine `CS-2 vs_duel.rs` 缺 `//!` 文件头（在途红项，S32.3 范围外，不计入）；CS-1/CS-3/CS-4 全 ok |
| `python3 scripts/check_ledger_reality.py` | 全绿（247 verified 三方对账 + 0 孤儿 evidence） |
| `python3 scripts/doc_numbers.py` | 0 漂移（33 项 / 247 verified 声称点一致） |
| 起服务 `/disk/target/debug/ti-console --host=0.0.0.0 --port=8787`（setsid 后台） | `/api/map` 4 包 + default=m01 ✓；`/api/evidence?sub=ui_concept` 147 文件 ✓；`sub=..` 500 ✓；ui_concept png 直读 200 / 未登记 404 ✓；`/api/session` 正确 ✓；`/api/audit` 74 份 ✓；**测完 `pkill` + `rm .lock` 已执行**（服务 0 残留进程） |
| `python3 scripts/map_pack.py check`（只读） | **0 漂移 EXIT=0**（4 包 + 默认 m01 布局对账全过） |

## 四、Findings

**new = 0**。

逐条核验 r1~r2 全部 9 条 findings（A 侧 5 + B 侧 4）：

| r1 id | 状态 | 核验面 |
|---|---|---|
| P2-A1（web/ 死文件） | 已修 ✓ | `ls crates/ti-console/web/` = 仅 `index.html`（1 entry，29 KB，mtime 10:48） |
| P2-A2（routes.rs stale 注释） | 已修 ✓ | L9-11 = 「看板 UI v1（左菜单分组 7 页 = 概览/任务/证据/资产/设计图/地图/设置；原生 JS + CSS 全内联无 npm；视觉令牌内联于 `index.html` `<style>`）」，theme.css 表述已删 |
| P2-A3（data.rs 双 /// 块） | 已修 ✓ | `audit_list`（L198）正上方单一 `///` 块；`session`（L366-367）独立 `///` 块紧邻 `pub fn session()` |
| P2-A4（22 号 §5 assets/maps/ 语义） | 已修 ✓ | 22 号 §5 表第 7 行 = 「`assets/maps/` 已装地图包**目录项前缀**（…`index.json` 在 `assets/maps/` 前缀内放行 = 包索引只读面）」，与 `console_sources.json` L7 前缀项 1:1 |
| P2-A5（.lock 残留） | 已修 ✓ | 测前 `rm -f evidence/console/.lock` + 测后 `pkill` + `rm .lock`，0 残留 |
| B-4-F1（75 张快照注） | 已修 ✓ | 22 号 L167 = 「登记数随资产批次增长，S32.3 快照 = 75 张」；WBS L30 同措辞 |
| B-4-F2（web/ 两物指代澄清） | **已修 ✓** | §6 引言 L154-157 + §11.2 L360-363 双落点均含明确对照措辞（见「一、修复核验」） |
| B-4-F3（routes.rs L1 头注释） | 已修 ✓ | L1-2 = 「S32.3 看板 UI 静态面（设计图/地图页增量，左菜单 7 页）」 |
| B-4-F4（14 号 verify 裸 id 口径） | 已修 ✓ | 14 号 L159-161 = 示例面「`verify <path>`/**或裸 id = 已装包目录** `assets/maps/<id>`」+ `python scripts/map_pack.py verify m01`（裸 id（平台制作指引面）） |

## 五、误报排除说明（0 误报）

| 项 | 口径 |
|---|---|
| ti-engine CS-2 `vs_duel.rs` 缺 `//!` 文件头 / CS-3 `ease_out` 单字母 | 既有在途红项（untracked `crates/ti-engine/src/vs_duel.rs` + `bin/android_entry.rs`），S32.3 改动面 = 仅 `crates/ti-console`，不计入 |
| i18n 孤儿 key / android 脏面 | 既有在途红项，不计入 |
| m01 I7 哈希漂移（`verify m01` 22 项不合规） | S32.12 已立项（plan-s32 §已知问题 L58：HEAD 即存在；S27.6 岛卡刷新后池资产未 sync 回写 m01 manifest），非 S32.3 裸 id 解析缺陷，不计入 |

## 六、收敛判定

- **new = 0**（9 条 r1~r2 findings 全部已修，无新增）；
- **误报 = 0**；
- **残留 = 0**（B-4-F2 = 唯一残留项，r2-a P2-1 修复核验通过：§6 引言 + §11.2 双落点均含「平台壳 `crates/ti-console/web/` vs 根 `web/`」两物对照措辞）；
- **收敛判定 = new 0 + 误报 0 + 残留 0 ⇒ 三条判据全满足**。
- 结构性缺口（gate 不覆盖 `verify`）挂 S32.7，非 S32.3 收敛阻塞项。
