# 历史阶段计划归档索引（Plan Archive）

> 本文件是**已完成阶段**计划的单一入口，只做「一句话概括 + 原文件跳转」，不复制原计划内容。
>
> **状态源是 `plan/ledger.yaml`，不是本文件**；本文件也不作为开发依据。
> 原计划文件**保留在 `docs/` 原位**（大量文档/源码注释/门禁脚本按文件名引用它们，移动会打断 40+ 处引用），
> 需要细节时点链接进原文件。

## 怎么用这个索引

| 你想做的事 | 走哪 |
|---|---|
| 查**当前**该做什么 | `plan/ledger.yaml` + `docs/loop.md`（不要读历史计划） |
| 查某阶段的**背景/决策原因** | 下面表格点进原计划 |
| 查**验收判据** | `docs/parity.md`（P0~P4 + 门禁项数唯一事实源） |
| 查**业务规则** | `docs/business/`（不是历史计划，也不是 legacy/） |

## 已完成阶段一览

| 阶段 | 一句话概括 | 原计划 |
|---|---|---|
| **S0 地基** | 建 cargo workspace + fmt/clippy + `gate.ps1`；IDA 批量导出伪代码与 `functions.csv` | [`plan-s0-s2.md`](plan-s0-s2.md) |
| **S1 资源逆向** | 逆出并实现 `LucencyPackage`/`.Map`/PCX/字体/INI/音频/存档 全部格式器（含 name→key 哈希） | [`plan-s0-s2.md`](plan-s0-s2.md) |
| **S2 三端骨架** | `gfx`/`input`/`audio`/`storage` 四层 + wasm 构建链 + 双端冒烟 | [`plan-s0-s2.md`](plan-s0-s2.md) |
| **S3 玩法模块** | 12 个玩法模块按 `legacy/docs/spec/` 实现，`oracle` vs `rules` diff 测试 | [`plan-s3.md`](plan-s3.md) |
| **S4 收尾** | 缩放/虚拟按键/存档 v2+迁移/性能与体积/全流程回归 | [`plan-s4.md`](plan-s4.md) |
| **S5 可玩收口** | `GameApp` 交互壳串通全链路 + headless 固定种子 + P4-1 渲染对齐 + 业务对齐 | [`plan-s5.md`](plan-s5.md) |
| **S6 呈现层** | 上采样滤波/边框/地形瓦片 blit/多帧动画/容器条目业务名 | 见 `docs/release.md` 可玩收口章节 |
| **S7 HD 资产** | 资产分类 → 批量超分/矢量化 → 关键图增强 → 多分辨率打包 → 渲染层切换 | [`plan-s7.md`](plan-s7.md) |
| **S8 标准化改造** | **定位变更**：逆向产物降级为参照物；建 `docs/process/` + `docs/business/`；数值数据化；工具链脚本化；玩法重写 | [`plan-s8-refactor.md`](plan-s8-refactor.md) |
| **S9 骨架** | 场景真拆分 → 窄接口 8 模块 → 字段私有化 → Builder+Type-State → Command 模式 | 前言见 [`plan-s10-refactor.md`](plan-s10-refactor.md) |
| **S10 现代化收口** | P0 错乱修复 + 占位清零 + GPU 化渲染 + 对抗流水线 loop 至停机 | [`plan-s10-refactor.md`](plan-s10-refactor.md) |
| **S11 正向开发收口** | 逆向产物物理归档 `legacy/` + oracle 桥接层独立 crate + 业务闭环 + 模块并行 loop 固化 | [`plan-s11-forward.md`](plan-s11-forward.md) |
| **S12 模块化并行** | 契约→并行→集成 loop 固化（D+P1–P6 七包，机读源 `plan/packages.yaml`） | [`plan-s12-parallel.md`](plan-s12-parallel.md) |
| **S13 质量基建** | 种子化 playtest / 台账回退防护 / 飞轮流水线 / 视觉回归门禁（任务见台账，原计划已并入 S12 文档） | [`plan-s12-parallel.md`](plan-s12-parallel.md) |
| **S14 业务对齐收口** | 业务规则/数值/版面单一事实源落地（S14.0 已验证，任务见台账） | 见台账 |
| **S15 业务场景穷举** | 9 场景业务闭环穷举 + 跨图推进规则定稿（S15.1~S15.2 已验证，任务见台账） | 见台账 |
| **S16 试玩闭环+发布** | 浏览器真运行时试玩 + 试玩 agent 闭环 + 一键发布 + 内容设计 sheet（S16.1~S16.4 已验证） | [`plan-s16-loop.md`](plan-s16-loop.md) |
| **S17~S31 UI 重设计/迁移** | v2 设计令牌→v3 场景叙事→v3.5 数据化→v4 羊皮海图→v5 环形棋盘→布局引擎 B1→资产/内容管线→地图包标准；计划 = 各设计/流程 spec（`docs/ui-redesign-v3/` + `docs/process/`），**非 plan-\* 文档** | 见 `docs/README.md` + 26 号 overview 阶段表 |
| **S33 领地/大富翁 v2** | 兵力模型/海事事件/商人/驻军/攻占/破产/AI（S33.1~S33.6 已验证） | [`plan-s33-territory-v2.md`](plan-s33-territory-v2.md) |
| **S35~S55 平台/测试/内容运营/流程强化** | APK 体积标准 + 构建编排（S35.0/S35.1，含版本打包规范面）/ Dev Console 平台面续集（S36~S39 平台 WBS）/ 内容运营平台化（S44）/ 测试接入标准（S45）/ 内容运营续集 + 平台主题 + 玩法补强（S49）/ CCGS 工作室制度借鉴（S50~S51）/ rules 测试盲区补测 + i18n 假孤儿清零 + 死代码 3 函数挂 deprecated（S52）/ 飞轮 R 断言固化 ×4（S53/S54.1/S55.1，doccheck2 44→47 项）；计划 = [`plan-s49.md`](plan-s49.md) / [`plan-s44-content-ops-platform.md`](plan-s44-content-ops-platform.md) / [`plan-s45-test-ingress.md`](plan-s45-test-ingress.md) / [`plan-s50-studio-interface.md`](plan-s50-studio-interface.md)（S32~S48 平台/内容 WBS 各见对应 plan-sNN 文档，全 verified） | 全 verified（台账 327→341 阶段面） |
| **S56~S57 流程强化（轨道 3 拍板面）** | S56.0 = `doc_numbers.py` 扩「doccheck2 项号文字 + 数值声称点」对账面（15 号行 36 载体，EXP-189 高频复现模式固化）；S57.0 = 红线挂包域补全（packages.yaml P7 引擎域 + P8 平台域，6 红线全挂）；S57.1 = state.md 压缩纪律载体面（doccheck2 第 48 项 `state-md-discipline`）；S57.2a/2b = 符号覆盖率第一批 28% 基线棘轮 + 第二批 `doc_numbers.py` 扩「符号覆盖率」回写面（M-04 阶梯判据面固化，后续 60%→100% = 覆盖率提升轮）；计划 = [`plan-s56-process-hardening.md`](plan-s56-process-hardening.md) | 全 verified（台账 341→345 阶段面，原竖屏 blocked 作废除外） |

## 仍在使用的计划（非归档，勿当历史）

| 文件 | 状态 |
|---|---|
| [`plan-s32-platform.md`](plan-s32-platform.md) | S32 开发管理平台（S32.1~S32.15 全 verified；单一事实源 = 22 号 `docs/process/22_dev_platform.md`） |
| [`plan-s34-main-map-v6-vs-v2.md`](plan-s34-main-map-v6-vs-v2.md) | S34 main_map v6「古典羊皮航海图」+ VS v2 战斗动画（S34.0~S34.7 全 verified；视觉签收被 S34.6/S34.7 取代收口） |

> 注：「仍在使用的计划」表 = **历史在途面快照**（建表时 S32/S34 在途）；2026-11-09 台账全收口后两阶段
> 已全 verified，本表仅作决策历史留档。现行进度唯一事实源 = `plan/ledger.yaml`。

## 归档纪律

1. 阶段全部 `verified` 后，其计划才进本索引；`in_progress`/`blocked` 的阶段留在台账与原文件。
2. 进索引时**只写一句话概括 + 链接**，不复制原计划正文（硬规则 6：同一结论只写一处）。
3. 原计划文件**不删**——它们是决策历史，且被源码注释/门禁脚本按文件名引用。
4. 历史计划里的**数字（门禁项数/计数）是该阶段时点快照**，不代表当前；当前值以 `docs/parity.md` 为准。
