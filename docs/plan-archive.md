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

## 仍在使用的计划（非归档，勿当历史）

| 文件 | 状态 |
|---|---|
| [`plan-s12-parallel.md`](plan-s12-parallel.md) | S12 模块化并行 WBS（D+P1–P6 七包，机读源 `plan/packages.yaml`） |
| [`plan-s13-*`](plan-s12-parallel.md) | 质量基建（种子化 playtest / 台账回退防护 / 飞轮 / 视觉回归）任务见台账，原计划已并入 S12 文档 |
| `docs/plan-s16-loop.md` | **未登记台账的 WIP 草稿**（S16 可玩性+渲染+运营 WBS），尚未 verified，勿当依据 |

## 归档纪律

1. 阶段全部 `verified` 后，其计划才进本索引；`in_progress`/`blocked` 的阶段留在台账与原文件。
2. 进索引时**只写一句话概括 + 链接**，不复制原计划正文（硬规则 6：同一结论只写一处）。
3. 原计划文件**不删**——它们是决策历史，且被源码注释/门禁脚本按文件名引用。
4. 历史计划里的**数字（门禁项数/计数）是该阶段时点快照**，不代表当前；当前值以 `docs/parity.md` 为准。
