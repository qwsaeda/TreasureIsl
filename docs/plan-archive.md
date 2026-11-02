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

## 仍在使用的计划（非归档，勿当历史）

| 文件 | 状态 |
|---|---|
| [`plan-s32-platform.md`](plan-s32-platform.md) | S32 开发管理平台（S32.1~S32.6 已验证，S32.7~S32.9 待做；单一事实源 = 22 号 `docs/process/22_dev_platform.md`） |
| [`plan-s34-main-map-v6-vs-v2.md`](plan-s34-main-map-v6-vs-v2.md) | S34 main_map v6「古典羊皮航海图」+ VS v2 战斗动画（进行中） |

## 归档纪律

1. 阶段全部 `verified` 后，其计划才进本索引；`in_progress`/`blocked` 的阶段留在台账与原文件。
2. 进索引时**只写一句话概括 + 链接**，不复制原计划正文（硬规则 6：同一结论只写一处）。
3. 原计划文件**不删**——它们是决策历史，且被源码注释/门禁脚本按文件名引用。
4. 历史计划里的**数字（门禁项数/计数）是该阶段时点快照**，不代表当前；当前值以 `docs/parity.md` 为准。
