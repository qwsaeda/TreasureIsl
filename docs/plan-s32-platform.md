# S32 开发管理平台 WBS（TI Dev Console）

> 定案 = [`process/22_dev_platform.md`](process/22_dev_platform.md)（定位/边界/架构/数据源/双审计协议/安全/测量口径）。
> 执行纪律：**按阶段门顺序验收**（同阶段任务可并行，见台账 deps）；每个任务 = 实现 → 自验 →
> 双审计（A 完成度 / B 一致性）→ 台账 `verified`。
> 平台与游戏运行时解耦：**不改玩法 / 渲染 / 数值 / `web/` 产物**（断言方式 = 22 号 §8）。

## 阶段总览

| 阶段 | 目标 | 任务 | 阶段门 |
|---|---|---|---|
| P0 只读骨架 | 打开就能看到「阶段 / 下一任务 / 门禁 / 证据」 | S32.0a、S32.0b、S32.1~S32.3 | 首屏 < 2 s 看到关键信息；0 游戏产物变更（内容级 sha256 指纹口径，22 号 §8） |
| P1 执行与审计 | 一键门禁 / 双审计任务书 + 收敛判定 / 一键收口 | S32.4~S32.6 | 三类执行全部落盘可追溯 |
| P2 辅助开发 | 资产浏览 / 文档检索 / 构建体检与预览 | S32.7~S32.9 | 三类浏览可用；0 游戏产物变更 |
| P3 部署（**可选，不入台账**） | 部署 / 鉴权 / 多工作区 | 需要时另立项（见 §P3） | 远程可用 + 鉴权负例拒绝 |
| 旁支（双审计副产物，不阻塞 P0~P2） | 门禁 Linux 假 PASS 根治 → 地图包 I7 漂移修复（S32.12 deps=[S32.11]） | S32.11 → S32.12 | 见 §已知问题 |

> **P3 不入台账的理由**：台账终止条件（`docs/loop.md` §3 第 1 条）= 全部任务 `verified`，
> 台账里放一条「允许永不做」的任务会让终止条件不可满足 ⇒ P3 只留在本文，需要时按
> `docs/loop.md` §1 流程新增下一个 S32 序号任务并自行定 DoD。

## 任务表

| id | 依赖 | 产物 | DoD（摘要） |
|---|---|---|---|
| S32.0a | — | 定案文档（22 号）+ 本 WBS + 5 处登记 | 两份文档存在且要素齐全；`docs/README.md` 两处 / `AGENTS.md` 路由表 / `00_index.md` 目录清单 / `15_process_ledger.md` 第 21 行 / 台账 S32 段；`doccheck` + `doccheck2` 0 error |
| S32.0b | S32.0a | 双审计收敛收口（0 新发现 + 0 误报 + 0 残留 连续 2 轮） | 审计报告逐轮落 `evidence/console/audit-S32.0b-r<N>-{a,b}.md`（抬头含 22 号 §7.3 四行机器可解析字段；历史轮次 `audit-S32.0-r{1,2,3}-*` 与 `audit-S32.0a-r4 起-*` 保留原名，r1~r3 按祖父条款人工口径读，**后续轮次随 rN 增长，不写死范围**）；连续 2 轮满足三条判据；主线程独立复核记录在 `evidence/S32.0b.log` |
| S32.1 | S32.0b | `crates/ti-devmeta`（ledger 解析上提 + 静态 HTTP；以 `crates/ti-tools/src/bin/serve.rs` 为原型重写为库）+ `crates/ti-console` 骨架（`/api/health` + 静态壳 + 锁 + 白名单表） | build/test/clippy 绿；health 200；默认 `127.0.0.1`；**依赖图断言**无游戏 crate；**鉴权负例**（非回环无 token 拒绝启动 / 无 token 写操作 403）；工具台账登记三处（14 号 cargo bin 小节 + 04_tooling + README） |
| S32.2 | S32.1 | 只读 API（`/api/overview` `/api/tasks` `/api/evidence` `/api/assets`）+ repo 适配层 | 与源文件抽样一致；越界路径 400/404；0 写盘；`console_sources.json` 落地 |
| S32.3 | S32.2 | 看板 UI v1（**左菜单分组**：看板组 = 概览/任务/证据/资产/设计图/地图，平台组 = 设置；原生 JS + CSS 0 依赖；「设置」= token 粘贴 + 写操作开关，与 `GET /api/session` 配套；**资产页 = UI 图片预览**：`ui_manifest` 全量（计数随批次增长，S32.3 快照 = 75） + **role 标签分组 + `generated_at` 时间倒序 + 点击 lightbox 放大 + 元数据含 AI prompt 原文**；证据页含双审计报告清单，`GET /api/audit` 解析 §7.3 四行机器字段；**设计图页 = AI 生成设计稿**（`evidence/ui_concept/` 子目录枚举 `GET /api/evidence?sub=ui_concept` + 组别标签 + lightbox，非运行截图）；**地图页 = 地图包管理面**（`GET /api/map`：已装包清单 + manifest 描述符摘要 + `.default` 指针 + 制作指引 `scripts/map_pack.py` 命令面 + 21 号标准链接；只读浏览，制作写操作走工具链、平台 0 写盘） | 视觉原型截图先经用户确认（留痕 = `evidence/console/ui-s32.3-{page}.png` + log「用户视觉验收通过」）；浏览器 0 console error；首屏 <2 s（22 号 §8 测量方式）；设置页与 `/api/session` 实测；资产页图片直读 200（`assets/ui/` 白名单前缀）+ 未登记 404；gate 全绿 + 双审计通过 + evidence/S32.3.log |
| S32.4 | S32.3 | 门禁执行（`fast/touch/full` + 单实例锁 + 轮询日志 + 历史） | 实跑 `fast` PASS；失败负例非 0；非法 tier 400；锁占用 409；**无 token 写操作 403** |
| S32.5 | S32.4 | 双审计任务书生成 + 报告归档 + 收敛判定（A/B 报告 `-a`/`-b`） | 对 S32.0b 实跑生成任务书（模板须逐条覆盖 22 号八要素清单）；报告落盘且抬头含四行机器可解析字段（总判定/新发现/误报/残留）；收敛规则单测（三条判据 + 连续 2 轮 + 3 轮升级） |
| S32.6 | S32.5 | 收口（`task_close.py --check` 只读 + 白名单 id → `--cmd` 精确映射）+ EXP 查重 | 实跑一条只读体检；未登记 id 400；失败不改台账；**白名单对账断言**（§4.2 动作 id 集合 == `console_commands.json`） |
| S32.7 | S32.3 | 资产浏览（`ui_manifest` 缩略图 + `registry.json` + `assets/maps/` 包信息） | 只调 `map_pack.py verify`（已装包集合 = 平台自扫描，排除骨架 `_template`）；缺图占位不崩；**校验状态须区分「平台 bug」与「仓库既有漂移」**（HEAD 基线对比，见 §已知问题） |
| S32.8 | S32.3 | 文档检索（全文 + `EXP-xxx` / `F-xxx` / 任务 ID 反查） | 关键词命中；反查处引用列表；索引上限不阻塞 |
| S32.9 | S32.3 | 构建**只读体检**（`build_wasm.ps1 -Check`）+ `web/` 产物清单 + 平台自身静态服务预览 | 只跑 `-Check`（不构建/不拷贝/不写 evidence）；非法参数 400；0 游戏产物变更（22 号 §8 内容级指纹口径） |
| S32.11 | — | 门禁 Linux 假 PASS 根治（`scripts/gate.ps1` 第 17 项硬编码 `powershell`） | Linux 实跑该项不再假 PASS；其余各项行为不变（项数随结构演进）；gate 全绿 |
| S32.12 | S32.11 | 地图包 I7 资产哈希漂移修复（`assets/maps/m01~m03`） | `map_pack.py verify assets/maps/m01~m03` 全部 0 违规 + `map_pack.py check` 0 漂移；gate 全绿（含第 17 项可信，须 S32.11 根治后） |

## 每任务审计要求（强制）

1. 主线程实现 + 自验，命令实跑输出入 `evidence/S32.x.log`（含 DoD 自身判据，不只贴 gate 总输出）。
2. **并行起两个子代理**（只读、互不可见）：
   - A 完成度：DoD 逐条对照证据/代码/实跑，找虚报、漏项。
   - B 一致性：架构/规范/文档/门禁/回归/安全/依赖，找漂移与风险。
3. findings 修复 → 复审计；**0 新发现 + 0 误报 + 0 残留，连续 2 轮**才算通过；
   连续 3 轮未过 ⇒ 按 `docs/loop.md` §4 拆子任务（`Sx.y.a` / `Sx.y.b`），原行删除或 `rounds` 归零。
4. 报告落 `evidence/console/audit-<台账 id>-r<N>-a.md` / `-b.md`（**由主线程写回**，平台亦可粘贴归档），
   抬头须含四行机器可解析字段（`总判定:` / `新发现:` / `误报:` / `残留:`，见 22 号 §7.3）；
   台账 DoD 引用（收敛轮次 + 总判定）。拆分前历史轮次保留原任务 id 文件名（现存 r1~r3 = `audit-S32.0-r*`）。
5. 平台归档与机械判定口径 = 22 号 §7.3 / §7.4（平台不自己起 LLM 子代理）。

## 已知问题（双审计副产物，已立项跟踪）

| 问题 | 实测 | 任务 |
|---|---|---|
| `scripts/gate.ps1` 第 17 项硬编码 `powershell` | Linux 无该命令，但 `$LASTEXITCODE` 仍 0 ⇒ 该项**假 PASS**（第 4 轮双审计实测） | S32.11 |
| 地图包 I7 资产哈希漂移 | `map_pack.py verify assets/maps/m01~m03` **各报 22 项不合规**（I7 字节数 + 哈希），**HEAD 即存在**；门禁（项数随结构演进，当前 33 项）不覆盖 verify ⇒ 漂移不可见。（`m04` 为 S16.4 在途空骨架，未入库，另 11 项「骨架未填」，**不纳入** S32.12） | S32.12 |
| 22 号八要素 / 5 处登记无机器断言 | `doccheck`/`doccheck2` 无相关检查项 ⇒ 后续迭代删段/漏登不会被门禁发现 | 兜底落 S32.5（任务书模板逐条覆盖八要素清单） |
| 平台 UI 视觉签收无机器兜底 | `check_visual_design_signoff` 只覆盖 `scenes.json`/`theme.json` ⇒ 平台 UI 走人工留痕 | `15_process_ledger.md` 第 22 行（半自动，挂 S32.3） |

## 执行顺序说明

- **取件顺序**（机械，`docs/loop.md` §2「取第一个 deps 全 verified 且未 verified 的任务」）：
  `S32.0a` → `S32.0b` → `S32.11` → `S32.12` → `S32.1`…（按台账行序；S32.12 deps=[S32.11]，串行）；
  ⇒ 实际语义 = **定案先于旁支、旁支先于平台本体**（不是「收敛先于旁支」）；
  旁支取件序 = `S32.11`（deps=[]，先修门禁假 PASS）→ `S32.12`（deps=[S32.11]：其 DoD 含「gate 全绿」，
  须第 17 项可信后再修地图包）→ `S32.1`…；旁支 = 双审计副产物（门禁假 PASS 根治 + 地图包 I7 漂移修复），
  **不阻塞 P0~P2**（见阶段总览「旁支」行），平台本体开工前先修这两项；
- P2 三任务（S32.7/8/9）deps = `S32.3` ⇒ 与 P1（S32.4~S32.6）可并行，但**阶段门按顺序验收**。

## 阶段门

- **P0 门**：首屏 < 2 s 看到台账/门禁/证据/审计；平台自身门禁覆盖绿（22 号 §11.2）；0 游戏产物变更。
- **P1 门**：门禁 + 双审计 + 收口三类执行全部落 `evidence/console/`，可追溯、可复跑。
- **P2 门**：资产/文档/构建三类浏览可用；0 游戏产物变更。
- **P3**（可选，不入台账）：部署后远程可用 + 鉴权负例生效；
  **边界**：部署**不含**游戏构建（镜像只装 console 二进制，`web/` 用既有产物挂载）。

## 与现有资产的关系

| 现有 | 平台怎么用 |
|---|---|
| `plan/ledger.yaml` + `ti-tools::ledger` | 台账数据源；解析实现上提 `ti-devmeta`，`ti-tools` 薄 re-export（单实现） |
| `scripts/gate.ps1` | 门禁执行唯一入口（`-Fast` / `-Touch` / 全量）；注意其第 23 项会改写 `evidence/web_playtest.{log,png}`（22 号 §3.1 已列） |
| `scripts/task_close.py` / `exp_log.py` | 收口 / EXP 登记唯一入口（平台默认只用 `--check` 只读面） |
| `scripts/map_pack.py` | 地图包校验唯一入口（已装包集合 = `installed_packs()` 口径：平台自扫描，不解析人类可读输出；不重抄 I1~I8，标准见 21 号） |
| `scripts/build_wasm.ps1` | 只用 `-Check` 只读体检；**平台不执行完整构建** |
| `evidence/` | 证据浏览 + 平台写点 `evidence/console/`（运行期日志在 `logs/` 子目录，gitignored） |
| `assets/ui_manifest.json` / `assets/registry.json` / `assets/maps/` / `dist/maps/` | 资产与包浏览 |
| `docs/` | 索引与全文检索（只读） |
| `docs/process/21_map_package_spec.md` / `18_asset_content_pipeline.md` | 地图包 / 资产标准单一事实源（本文只引用，不复述） |

## 不做

- 不做游戏编辑器 / 运行时 / 关卡工具（另立项）。
- 不做账号、多租户、计费（P3 再评估）。
- 不做任意 shell；不引入数据库（v1）；不内嵌 LLM SDK；不做完整构建。
