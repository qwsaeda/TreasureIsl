# 开发阶段总览（Dev Stages Overview）

> **定位**：项目开发阶段的**横向总览**——阶段演进 / 开发流程 / 节点 / 工具矩阵 / 当前节点。
> 单一事实源声明：本文是**索引性总览**，不复制各阶段计划正文（硬规则 6，同一结论只写一处）；
> 阶段细节回原文件（`docs/plan-archive.md` 归档索引 + `plan/ledger.yaml` 进度 + `docs/loop.md` 循环协议）。
> 进度只由 `plan/ledger.yaml` 驱动；本文状态栏随台账演进（保持同步，doccheck 检查计数）。

---

## 一、项目概览

- **产品**：《金银岛 TreasureIsl》Windows Mobile 游戏（ARM 原生，176×220 逻辑分辨率）的**逆向 + Rust 现代化移植**，目标 **WASM（浏览器）+ Android** 双端。
- **技术底座**：Rust edition 2024 + **Bevy 0.20**（S18.0 定案唯一渲染引擎，推翻旧 macroquad 定案）+ cargo workspace **9 个成员 crate**。
- **进度**（`plan/ledger.yaml` 实测）：**334 任务 / 333 verified / 0 pending / 1 blocked（S7.7 可选）**。
- **9 场景**：`Intro → Index → SelectRole → SelectMap → MainMap → Vs → Message → Saveload → Exit`（20 条合法转换，非法转换编译期拒，Type-State 保证）。
- **核心循环**：探索→贸易→建造→卡牌→战斗；主地图主玩法 = **大富翁式摇骰前进**（S19 定案：摇骰定步数 → 沿环岛赛道前进 N 格 → 落格结算；方向键自由移动已删）。
- **主线玩法**（S27/S28/S33 领地系统）：31 格赛道（每格 = 领地/功能点）+ 中央可点击据点岛 + 10 类海盗岛专属副本 + 5 席位（≥1 玩家 + 4 AI）+ 落格三支分叉（无主→副本占领 / 他人→交钱或攻占 / 自己→升级加固）。

---

## 二、开发阶段总览（S0 → S35）

| 大阶段 | 阶段 | 一句话内容 | 状态 |
|---|---|---|---|
| **逆向地基** | S0 | 建 cargo workspace + fmt/clippy + `gate.ps1`；IDA 批量导出伪代码与 functions.csv | ✅ 归档 |
| | S1 | 逆出并实现 `LucencyPackage`/`.Map`/PCX/字体/INI/音频/存档 全部格式器（含 name→key 哈希） | ✅ 归档 |
| | S2 | `gfx`/`input`/`audio`/`storage` 四层 + wasm 构建链 + 双端冒烟 | ✅ 归档 |
| **玩法实现** | S3 | 12 个玩法模块按 `legacy/docs/spec/` 实现，`oracle` vs `rules` diff 测试 | ✅ 归档 |
| | S4 | 缩放/虚拟按键/存档 v2+迁移/性能与体积/全流程回归 | ✅ 归档 |
| | S5 | `GameApp` 交互壳串通全链路 + headless 固定种子 + P4-1 渲染对齐 + 业务对齐 | ✅ 归档 |
| | S6 | 上采样滤波/边框/地形瓦片 blit/多帧动画/容器条目业务名 | ✅ 归档 |
| **HD 资产** | S7 | 资产分类 → 批量超分/矢量化 → 关键图增强 → 多分辨率打包 → 渲染层切换 | ✅ 归档 |
| **标准化改造** | S8 | **定位变更**：逆向产物降级为参照物；建 `docs/process/` + `docs/business/`；数值数据化；工具链脚本化 | ✅ 归档 |
| | S9 | 场景真拆分 → 窄接口 8 模块 → 字段私有化 → Builder + Type-State → Command 模式 | ✅ 归档 |
| | S10 | P0 错乱修复 + 占位清零 + GPU 化渲染 + 对抗流水线 loop 至停机 | ✅ 归档 |
| | S11 | 逆向产物物理归档 `legacy/` + oracle 桥接层独立 crate + 业务闭环 + 模块并行 loop 固化 | ✅ 归档 |
| **并行开发** | S12 | 模块化并行 WBS（D+P1–P6 七包，机读源 `plan/packages.yaml`） | ✅ 归档 |
| | S13 | 质量基建（种子化 playtest / 台账回退防护 / 飞轮 / 视觉回归） | ✅ |
| | S14 | 存档 v4 + 存读档闭环 | ✅ |
| | S15 | **业务场景穷举测试**（S15.1 单场景穷举；S15.2 跨图推进规则定稿 = 通关门禁 `is_cleared` 逐图 + 存档 `unlocked_mask` 逐图写入） | ✅ |
| | S16 | 试玩闭环 + 一键发布（publish-web）+ 内容设计 sheet 模版 | ✅ |
| **UI 重设计** | S17 | 界面全量重设计 v2「航海夜·精修」（设计令牌 v2 / 8pt 版面 / 图标 14 项 / 动效 4 参数） | 主码 ✅；**S17.5 视觉签收待用户** |
| | S18 | 渲染引擎迁移 **Bevy 0.20**（ti-engine crate + 双渲染期 + wasm 预算 ≤8MB） | ✅ |
| | S19 | 环岛赛道（大富翁式摇骰前进） | ✅ |
| | S20 | 存档/消息/saveload 场景 v4 面板语言 | ✅ |
| | S21 | **main_map 重设计 v4「羊皮海图桌面」+ 船系统**（B 独立船轴 4 级，造价 0/800/2500/6000，货舱 99/150/240/400） | 主码 ✅；**S21.9 视觉签收待用户** |
| | S22 | 首页 index 重设计「罗盘环标题页」（6 菜单沿罗盘环 + 中心舵轮 logo） | ✅ |
| | S23 | 测试/基准基础设施 | ✅ |
| | S24 | 相机 `o.scale` 语义 + 点击绝对光标 | ✅ |
| | S25 | 代码标准三件套（CS-1~4 + 棕轮基线） | ✅ |
| | S26 | 全场景视觉收尾（ICU4X 报错根治 + 岛屿图片化 + 七场景视觉） | 主码 ✅；**S26 视觉签收待用户** |
| **领地系统** | S27 | main_map v5「中央据点岛 + 环形格子环」+ VS 重设计规格 | 主码 ✅；**S27.5 视觉签收待用户** |
| | S28 | 领地业务规格（5 席位 + 31 格 + 10 类岛专属副本 + 落格三支分叉 + 存档 v4） | ✅ |
| **布局/资产管线** | S29 | **布局引擎定案 B1 全 bevy_ui 响应式**（UI + 命中 + 重排全进 Bevy；游戏世界保持 1080 正交 + 数据驱动） | ✅ |
| | S30 | 资产/内容数据管线（不手写像素：声明意图/数据，引擎算坐标） | ✅ |
| | S31 | **地图包标准 v1**（一张图 = 一个自包含可分发包 + 运行期换图全链路 + 失败回退） | ✅ |
| **开发平台** | S32 | **TI Dev Console 开发管理平台**（纯管理/辅助开发，与游戏运行时解耦；看板 v1 + 双审计协议 + 北极星判据） | **✅ 全部**（S32.0a/S32.0b + S32.1~S32.9 + S32.11~S32.15，2026-11-02 收口） |
| **领地 v2 业务** | S33 | 领地/大富翁 v2 业务逻辑（兵力模型/海事事件/商人/驻军/攻占/破产/AI） | ✅ 全 verified |
| **v6+VS v2** | S34 | **main_map v6「古典羊皮航海图」+ VS 战斗动画 v2**（不规则岛 + 格子紧围岛 + 4 帧/兵力 HUD/格类分层/时间轴） | **✅ 全 verified**（S34.0~S34.6，2026-11-09 收口） |
| **产物体积** | S35 | **APK/产物体积标准**（.so 压缩 + 未引用 UI 0 进包 + 引用集打包 + 审计工具） | **✅ 全部**（S35.0 + S35.1，2026-11-02 收口） |

---

## 三、开发流程（三层协议）

### 1. 循环协议 `docs/loop.md`（进度推进唯一机制）

```
1 读 ledger → 取第一个 deps 全 verified 且 status != verified 的任务
2 实现（按 WBS 产出物落地，禁占位：TODO/unwrap/panic! 一律 0，tests 除外）
3 跑该任务 dod 验收命令 → 输出 tee evidence/<id>.log（须含任务自身判据）
4 跑 scripts/gate.ps1（36 项全局门禁）
5 全绿 → ledger 置 verified + evidence；否则 rounds+=1（≥3 必须拆子任务）
6 输出进度报告（verified/total、阻塞、下一轮目标）
7 回到 1
```

**终止条件**（全部满足才算项目完成）：
1. `plan/ledger.yaml` 全部任务 `verified`；
2. `docs/parity.md` 的 P0~P4 五级标准全部 pass（各自校验命令产出报告存 `evidence/`）；
3. 三端产物生成：桌面 exe、wasm 站点、APK；
4. `scripts/gate.ps1` 36 项全绿 + 占位扫描 0 命中。

**卡死与回退**：
- 验收连续失败 3 轮 → 拆子任务（`Sx.y.a`/`Sx.y.b`），子任务 `rounds` 从 1 起算；
- 计划与实测冲突 → 以实测为准，更新 `legacy/docs/facts/` + WBS/台账，**不许静默绕过**；
- 踩新坑 → 立即写 `docs/experience/index.md`，任务 evidence 中引用；
- 外部依赖缺失 → 标 `blocked`，推进无依赖任务不空转。

### 2. 标准工作流 `docs/process/00_index.md`

```
认领(读台账) → 读规格(business + process/02_conventions) → 分类(脚本/模板/库/LLM)
→ 实施(①先定模块边界+函数清单 ②批量编辑→一次编译 ③报错一次性收口)
→ 验收(dod 实跑→evidence/<id>.log) → 门禁(gate.ps1) → 记账(全绿才 verified) → 复核(主线程)
```

- **任务粒度**：一条命令可验 + 触及文件 ≤8 + 不同时改「接口+实现+数据」三层（拆 `.a`/`.b`）。
- **LLM 禁区**：不凭记忆断言数值（只来自 `assets/data/*.json` 或 `docs/business/`）/ 不顺手重构 / 推测须标注 / 不删门禁项或写占位。
- **变更面声明**（硬规则 7）：业务规则/数据/规则实现/表现/资产；跨模块必写原因；含业务规则先 `docs/business/` 定案再动代码。

### 3. 并行与对抗（两种增强 loop）

- **模块并行轮** `process/05_module_parallel.md`：任务 ≥2 且文件域不相交 → 契约冻结 → 多包并行 → 集成联调 → 记账（包白名单互斥 `plan/packages.yaml` + 任务书模板 + 纪律 5 条）。
- **对抗流水线飞轮** `process/06_flywheel.md`：找问题→审计→修复→回写 3 角色；开轮必读 7 条〔S51.0〕+ 停机条件（gate 第 17 项锚点）。

---

## 四、节点清单（当前待办 0 + 阻塞 1，S7.7 可选）

> 完整台账 = `plan/ledger.yaml`；下表 = **当前主待办**（AGENTS 路由表同步口径）。

| 节点 | 内容 | 判据摘要 |
|---|---|---|
| **S32.4** | 门禁执行 `POST /api/gate`（tier=fast/touch/full → -Fast/-Touch/全量）+ 单实例锁（evidence/console/.lock）+ 轮询日志 + 门禁既有副作用路径只记录不判定 | fast 档 PASS + 失败负例非 0 + 非法 tier 400 + 锁占用 409 + 无 token 写操作 403 |
| **S32.5** | 双审计编排（任务书模板 + 归档报告 + 机械判定：0 新发现 + 0 误报 + 0 残留 / 连续 2 轮 / 3 轮升级拆子任务） | 实跑生成任务书对账 + 收敛规则单测 + 载体口径逐条覆盖 22 号八要素。**✅ verified（2026-11-02）**：audit 四模块 + POST taskbook/archive + GET 收敛表 + 28 单测 + 双审计 r1~r3 收敛（r2/r3 连续 2 轮 0/0/0 pass）；evidence/S32.5.log |
| **S32.6** | 任务收口 `POST /api/close`（默认 `task_close.py --check` 只读体检）+ EXP 查重（`exp_log.py --check`） | 实跑 + 未登记 id 400 + 失败不改台账（执行前后 sha256 一致）+ 白名单对账。**✅ verified（2026-11-02）**：close.rs + `POST/GET /api/close` + `routes/close_api.rs` + 白名单对账断言（22 号 §4.2 机器化）+ 12 单测 + 双审计 r1~r4 收敛（r3+r4 连续 2 轮 0/0/0 = `state=pass`）；evidence/S32.6.log |
| **S32.7** | 资产浏览（ui_manifest 缩略图 + registry.json + maps 包信息，只读；校验状态只调 `map_pack.py verify`） | 可浏览 + 缺图不崩 + 校验状态与 verify 退出码一致 + 区分平台 bug 与仓库既有漂移。**✅ verified（2026-11-02）**：`map_pack_info` verify_status/problems + web 资产页校验卡 + 实测 4 包 verify=ok；evidence/S32.7.log |
| **S32.8** | 文档检索（全文搜索 + EXP/F-ID/任务 ID 反查，只读，索引上限防阻塞） | 关键词命中实测 + 反查引用列表（文件:行）+ 大文件/总索引上限不阻塞。**✅ verified（2026-11-02）**：`repo/search.rs` + `GET /api/search` + web 检索页 + 实测 EXP-116/F-319/S32.6 命中 + 空 q 空 + 上限截断标志 + **`q` 服务端 `percent_decode`**（前端 `encodeURIComponent`，中文/空格命中，非法 `%XX` 原样保留）+ token 边界反查（EXP-116 不误配 EXP-1160）；evidence/S32.8.log |
| **S32.9** | 构建只读体检与预览（`build_wasm.ps1 -Check` 不构建/不拷贝/不写 evidence + web/ 产物清单 + 0 游戏产物变更断言） | 只读体检实跑 0 写盘 + 非法参数 400 + 冻结路径内容级 sha256 指纹差为空。**✅ verified（2026-11-02）**：`runner::build::build_check` + `GET /api/build?check=1` + web 体检卡 + 实测 check_exit=0 / frozen_diff_empty=true；evidence/S32.9.log |
| **S34.0** | v6/VS v2 设计稿定案 + 引擎出图（用户视觉签收 A 纪律） | spec 用户确认留痕 + main_map v6 / VS v2 4 阶段出图 0 硬错误 + 用户视觉验收通过（不可代签）。**✅ verified（2026-11-09）**：S34.0~S34.6 全收口 |
| **S34.3** | VS v2 元素批次（9 PNG + `ui_batches/vs_v2.json` + ui_manifest 登记） | ui_asset_batch check 0 违规 + 9 元素在位 + validate_data 全绿。**✅ verified（S34 全收口 2026-11-09）** |
| **S34.4** | VS v2 动画原型落码（4 阶段 clash/duel/result_win/result_lose + 兵力 HUD + theme 色值） | 4 阶段各 `--shot vs` 出图 0 硬错误 + check_hardcode 0 命中。**✅ verified（S34 全收口 2026-11-09）** |
| **S34.5** | VS v2 按格类×地图主题分层（5 类各一张出图 + 格类分派逻辑 + TI_MAP=m02 换图主题 ≠ m01） | 5 类出图可区分 + m02 背景/BOSS ≠ m01。**✅ verified（S34 全收口 2026-11-09）** |
| **S34.6** | 视觉签收 + 出图验收（main_map v6 + VS v2 全场景出图 + 逐屏核对 + 用户视觉验收） | 出图非黑 ≥90% + 缺陷清单全清 + 用户视觉验收通过 + gate 全绿。**✅ verified（S34 全收口 2026-11-09，S17.5/S21.9/S27.5 视觉签收由其取代收口）** |
| **S35.1** | 构建编排平台化（`build_artifacts.py` 编排器 + 平台「构建」页 + 白名单 5 条） | check PASS + /api/build 只读 list + ?run 后台 spawn + apk_audit 复检 PASS + 白名单负例 400。**✅ verified（2026-11-02）**：`build_artifacts.py` 编排器 + `GET /api/build?run` 白名单 400 修 + list/check 只读；evidence/S35.1.log |

**阻塞 1**：S7.7（P4-1 基准切换可选任务，等待 S7.5 稳定运行 ≥2 周——非阻塞主线）。

**待用户视觉签收**：S17.5 / S21.9 / S27.5 已由 S34.6/S34.7 取代收口（S34 全 verified，2026-11-09），无需单独签收。

---

## 五、工具矩阵（`scripts/` 全量，唯一事实源 = `14_tooling_catalog.md`）

| 用途 | 工具 |
|---|---|
| **门禁** | `gate.ps1`（36 项固定 Report：fmt/clippy/test/build/占位/unwrap/台账/doccheck/validate_data/gen_layout/check_hardcode/check_data_driven/check_code_standards/oracle边界/no-macroquad/doccheck2/台账对账/业务闭环/模块契约/飞轮/渲染不变式/数据源/audit_code/场景叙事/bevy-cpu-parity/wasm-browser-e2e/browser-playtest/playtest-loop） |
| **数据校验** | `validate_data.py`（B 基础 + V1 grid 三键 + V2 标量 + V3 图标 + scenes_canonical + asset_registry + test_manifest + **version_consistency** 三方对账） |
| **代码校验** | `check_hardcode.py` / `check_data_driven.py` / `check_code_standards.py`（CS-1 资产单源/CS-2 文件职责/CS-3 参数命名/CS-4 场景窄接缝）/ `audit_code.py`（规模棘轮）/ `check_ledger_reality.py`（台账三方对账）/ `check_oracle_boundary.py` / `check_forward_closed.py` / `check_module_contracts.py` / `check_scene_narrative.py` / `check_data_source.py` |
| **文档一致性** | `doccheck`（155 docs/38 facts/274 task refs 交叉引用）/ `doccheck2`（42 项 audit-codified 断言；S50.3 起 = 41 + 第 42 项 `rules-files`）/ `doc_numbers.py` / `layout_spec_check.py` / `check_ps_scripts.ps1` |
| **台账/经验** | `task_close.py`（收口一条命令：跑判据→写证据→置 verified→同步计数→三方对账）/ `exp_log.py`（EXP 3 处改齐）/ `ledger`（`ti-tools --bin ledger` 台账校验） |
| **UI 资产** | `ui_assets.py`（add/build/verify）/ `ui_asset_batch.py`（规格驱动批量 + 抠图预检）/ `ui_layout_sheet.py` / `ui_thumb.py` / `ui_preview.ps1`（出图验收）/ `ui_shot_probe.py` |
| **AI 生图/视频** | `agnes_gen.py`（Agnes Image 2.5 Flash）/ `agnes_video.py`（Agnes Video 2.5 Flash）/ `agnes_vision.py` |
| **地图包** | `map_pack.py`（pack/verify/install/uninstall/list/add/check/sync/index）+ `extract_board.py` / `board_measure.py` |
| **渲染/像素** | `bevy_cpu_scene_diff.py`（GPU↔CPU 像素对拍）/ `pcx_to_png.py` / `rgb565.py` / `pcx_ref.py` |
| **浏览器 e2e** | `wasm_browser_e2e.py`（L1 实例化/L2 渲染/L3 交互/L4 场景切换/L5 环资产）/ `web_playtest.cjs`（CDP 按键驱动试玩）/ `playtest_loop.py`（4 报告聚合） |
| **构建/发布** | `build_apk.sh`（NDK 交叉编译 APK）/ `build_wasm.ps1`（wasm 交付产物）/ `build_artifacts.py`（构建编排器）/ `apk_audit.py`（体积审计判据①②）/ `publish_web.ps1`（一键发布 gh-pages）/ `smoke.ps1`（双端冒烟）/ `playbook.ps1`（22 命令一键入口） |
| **平台（ti-console）** | `cargo run -p ti-console`：`/api/{health,overview,tasks,evidence,assets,tests,map,audit,apk-size,build,whitelist}` + `/api/tests?run=<id>` 白名单后台跑测试 + `/api/map/import` 唯一写点 |

---

## 六、当前所处节点（台账全收口 时点）

**进度**：334 任务 / **333 verified / 0 pending / 1 blocked（S7.7 可选）**；gate 全绿（0 failure）。

**刚完成一轮**（「先把能做的全做了」= ① 类 4 项并行收口）：
1. ✅ S32.7 资产浏览：`map_pack_info` 每包 `verify_status=ok|drift|platform_error` + `verify_problems`（`[FAIL]` 行），web 资产页校验卡；实测 4 包 verify=ok problems=0。
2. ✅ S32.8 文档检索：`repo/search.rs` 全文搜索 + EXP/F-ID/任务 ID 精确反查 + 上限 3000/512KB/500 超限截断；`GET /api/search` + web 检索页；实测 EXP-116→15/F-319→11/S32.6→35 命中，空 q→0，耗时 197~284ms。
3. ✅ S32.9 构建只读体检：`runner::build::build_check`（`pwsh -Check` + `snapshot::FrozenSnapshot` 前后 diff）+ `GET /api/build?check=1`；实测 check_exit=0 / frozen_diff_empty=true / frozen_diff=0。
4. ✅ S35.1 构建编排平台化：`build_artifacts.py` 编排器 + `GET /api/build?run` 白名单负例 **400**（改前 500）+ list/check 只读；evidence 四份 + 台账 4 任务 verified + WBS/26 号/14 号/22 号同步。

**下一节点** = 台账已全收口（333 verified / 1 blocked S7.7 可选），后续 = 飞轮找问题 / 用户拍板新方向（S50/S51 起）；S34.0~S34.6 全 verified（2026-11-09），视觉签收类 S17.5/S21.9/S27.5 由 S34.6/S34.7 取代收口。

---

## 七、验收判据与终止条件

### P0~P4 五级判据（`docs/parity.md`，验收唯一事实源）

| 级 | 判据 |
|---|---|
| **P0** | 无占位（TODO/unimplemented!/panic!）+ 库代码 0 unwrap/expect（tests 除外） |
| **P1** | 单测绿（`cargo test --workspace` 0 failed） |
| **P2** | 门禁 36 项全绿（`gate.ps1` 全 PASS/SKIP，0 FAIL） |
| **P3** | 业务闭环（P3-3 全流程 16 步 0 失败 + P3-2 穷举覆盖） |
| **P4** | 渲染对齐（P4-1 CPU↔Bevy 像素对拍通道 ≤8 LSB 超差 <0.5%） |

### 门禁 36 项（`gate.ps1`，当前 30 PASS / 0 FAIL / 2 SKIP）

fmt / clippy / test / build(native) / wasm build+clippy / android(SKIP 无 NDK) / 占位 / unwrap / 台账 / doccheck / validate_data / gen_layout / check_hardcode / check_ledger_reality / check_forward_closed / check_oracle_boundary / no-macroquad / doccheck2 / 台账对账 / check_data_driven / check_data_source / 模块契约 / 飞轮 / 渲染不变式 / audit_code / 场景叙事 / bevy-cpu-parity / wasm-browser-e2e / browser-playtest / playtest-loop / check_code_standards / …（wasm/android 构建为条件子项）。

---

> **维护约定**：进度状态随 `plan/ledger.yaml` 演进更新（每轮收口同步 §二状态栏 + §四节点清单 + §六当前节点）；文档登记 = `docs/README.md` + `AGENTS.md` 路由表（硬规则 6）。
