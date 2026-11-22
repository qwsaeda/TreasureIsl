# S67 · LGUI 验收机制吸收（2026-11-22 用户拍板「全部 5 条」）

> **单一事实源**：本文 = S67 五条改进的 WBS 与验收判据；不复制 LGUI 文章内容（来源 = 公众号《游戏 UI 是 AI 做游戏最卡的一层，有人把它拆出了引擎》2026-11-20）。
> 触发：用户发 LGUI 拆解文 → 主线程出 5 条对比改进 → 用户拍板「全部 5 条」。
> 关联（引用不复制）：`13_ui_asset_pipeline.md` §6 迭代闭环、`29_content_ops.md` §3 换 UI 流程、
> `19_bevy_ui.md` §9.8 坑位、`14_tooling_catalog.md` §1.2 工具契约、`31_test_ingress_standard.md`。

## 0. 改动面声明（硬规则 7）

- **工具**：`scripts/ui_verify.py`（新，设备矩阵出图 + 机检断言 + `--json` 汇总）、`crates/ti-tools/src/bin/doccheck2.rs` + `assets/gate_data.json`（第 3 条新增断言）、若干 `scripts/*.py` 补 `--json`。
- **数据**：`assets/data/ui_schema/safe_area.json`（新，四边 inset 默认全 0）、`assets/data/test_manifest.json`（第 5 条登记）。
- **测试**：`crates/ti-app/tests/e2e_ui_flow_deterministic.rs`（新）。
- **文档**：`docs/process/{04,13,14,19,23,29}_*.md`、`docs/README.md`、`docs/experience/`（EXP 回写）。
- **跨模块原因**：① `ui_verify` 同时消费出图链路（`--shot`）、安全区数据、错误码表，三者必须同改才可一条命令闭环；② 确定性测试同时改测试层与登记面（`test_manifest`），拆开则平台不可见；③ 新增工具/断言/任务会改变台账与项数声称点，数字载体必须随台账同批改（`doc_numbers.py --fix`），否则 `doccheck2` 跨文档计数 FAIL ⇒ 无法拆成互不依赖的可验收块。
- **A 纪律（硬规则 9）**：安全区 inset **默认全 0 = 现有画面零变化**，不新增/不改任何视觉意图，故本轮不触发设计稿先行；若日后启用非 0 inset 改变元素落点 = 新视觉意图，须重新出图过目。

## 1. WBS

| 任务 | 改进 | 产出物 | 判据（每条独立可验收） |
|---|---|---|---|
| **S67.0** | ① 设备矩阵出图验收 | `scripts/ui_verify.py` + 14/04/README 三处登记 + 13 号 §6 / 29 号 §3 同步 | `--check` 只读实跑：默认矩阵出图、每图非黑 + 探针断言、`--json` 汇总、空帧非 0 退出；负例（空帧/坏视口）精准 FAIL + 还原 0；工具契约 3 处登记 0 漂移 |
| **S67.1** | ② 安全区模拟 | `assets/data/ui_schema/safe_area.json` + `ui_verify.py` 增量 + 19 号新增段 | `--check` 实跑：默认全 0 ⇒ 生成图与基线逐字节一致（零视觉变化）；注入非 0 inset ⇒ 产出 `*_safe.png` 且遮罩四边宽度 = 配置值；横竖旋转四边跟随；负例（缺字段/负值）非 0 |
| **S67.2** | ③ CLI `--json` 全覆盖 | `doccheck2.rs` 第 54 项 `cli-json-contract` + `gate_data.json` + 高频工具补 flag | 双负例实跑（去掉某验类工具 `--json` 注册 ⇒ 精准 FAIL + 还原 0 error）+ `doc_numbers.py --fix` 项号文字 0 漂移 + doccheck2 54 项 |
| **S67.3** | ④ 稳定错误码表 | `docs/rules/ui-error-codes.md` + README 登记 + 14 号错误码列 | 高频失败（资产漏登记/非规范形/版本漂移/空帧/bevy_ui 静默不渲染）分配 `TI-UI-xxx`；`doccheck2` 引用码 0 漂移 + 索引登记在位 |
| **S67.4** | ⑤ 确定性 UI 流程测试 | `crates/ti-app/tests/e2e_ui_flow_deterministic.rs` + `test_manifest.json` + 23 号段 | 固定 seed + 固定 `frame_delta_ms`（假时钟等价物），同输入两次运行逐位一致；`cargo test -p ti-app --test e2e_ui_flow_deterministic` 全绿 + manifest 对账 0 违规 |

## 2. 验收顺序与门禁

1. 逐任务：`dod` 命令实跑 → `evidence/S67.0.log` … `evidence/S67.4.log`（须含任务自身判据，硬规则 2）。
2. 全量：`pwsh scripts/gate.ps1` 36 项 0 failure。
3. 收口：`python3 scripts/task_close.py --task S67.0 --no-run`（示例；对 S67.0~S67.4 各跑一次，证据已在；自动 `sync_release_counts` + 三方对账）。
4. 数字回写：`python3 scripts/doc_numbers.py --fix`（台账 372 任务 / 372 verified）。
5. EXP 回写：`python3 scripts/exp_log.py` 记录本轮踩坑（至少：工具契约静态文本≠执行级判据、`doc_numbers --fix` 漏 26 号现状区需手补）。

## 3. 边界与不借鉴

- **不照搬** LGUI 的「UI 拆成独立浏览器工程 + TS 重写 + 后端适配层」：本项目 UI 已是数据 + bevy_ui，再拆是净增构建与三处版本同步负担，收益为负。
- **不采纳**「30 分钟做完 UI」官方未复现口径。
- 安全区当前无移动端刘海设备需求（Android 侧 0 命中 cutout/notch）；本轮只做**截图取证层**模拟，不做渲染层避让。
- `--json` 断言只覆盖「验类/写类」工具（`scripts/*.py` 的 50 个 `argparse` 工具中为高频子集），不追求 63 个全量改造。
