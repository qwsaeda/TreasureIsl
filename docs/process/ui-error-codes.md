# UI 错误码表（TI-UI-xxx · S67.3）

> **单一事实源**：本文 = UI/工具链高频失败模式的稳定错误码。其他文档**引用码**而非复述整段错误文本，便于 Agent 机读与跨文档对账。
> 触发：LGUI `LGUI-CLI-003/144`、`LGUI-RT-001` 的稳定码形式借鉴（2026-11-22 用户拍板 S67 全部 5 条）。
> 关联（引用不复述）：`13_ui_asset_pipeline.md` §5、`18_asset_content_pipeline.md` §3/§4、`25_versioning.md`、`19_bevy_ui.md` §9.8-15、`14_tooling_catalog.md` §1.4。

## 1. 码表

| 码 | 失败模式 | 机读判据（谁报） | 处置 |
|---|---|---|---|
| `TI-UI-001` | 资产漏登记 / 悬挂引用 | `validate_data::check_asset_registry`（disk↔registry 双向同步），`ui_assets.py verify` | 新增资产先改 `assets/ui_manifest.json` 再跑 `ui_assets.py build`；禁止手改成品图 |
| `TI-UI-002` | `scenes.json` 非规范形（indent ≠ 1 / 整文件 churn） | `validate_data::check_scenes_canonical` | 读原对象 → 定点回注 → `json.dumps(indent=1)`；禁止整文件换缩进（EXP-124） |
| `TI-UI-003` | 版本三处漂移（Cargo ↔ AndroidManifest ↔ 产物名） | `validate_data::check_version_consistency` | 改 `Cargo.toml` `[workspace.package] version` 单源 + 同步 manifest + `doc_numbers.py --fix` |
| `TI-UI-004` | 出图空帧（nonblack = 0 / readback 全透明） | `treasure_bevy.rs` `write_shot_png` 统计 + `ui_preview.ps1 -Check` + `ui_verify.py` 非黑断言 | 先看探针 `max_node=` 与资产加载；`--shot` 空帧允许重截（13 号 §7 已知边界） |
| `TI-UI-005` | bevy_ui 静默不渲染（`ImageNode`/`Text` 与 `Node` 不同实体） | `ui_probe_system` node/text 计数 + 像素采样（19 号 9.8-12/15） | 内容组件与 `Node` 同实体（`commands.entity(node).insert(...)`），EXP-168 |

## 2. S67 新增码（本轮工具链）

| 码 | 失败模式 | 机读判据 | 处置 |
|---|---|---|---|
| `TI-UI-006` | 安全区配置非法（缺字段 / 负值 / 非 4 元组） | `ui_verify.py::load_safe_area`（S67.1，默认全 0 不触发） | 按上右下左四元组补 `assets/data/ui_schema/safe_area.json` |
| `TI-UI-007` | 工具未三处登记 | doccheck2 `tooling-registration`（第 35 项）+ `studio_tests.py` 断言 3 | 14 号表格行 + 04 号速查 + README 索引（14 号 §1.2） |
| `TI-UI-008` | `--json` 仅在文本里出现、无真实注册 | doccheck2 `cli-json-contract`（第 54 项） | argparse `add_argument("--json")` 或 `"--json" in sys.argv` 实际注册 |

## 3. 使用纪律

1. **报错误先给码**：工具输出 / 文档 / 飞轮 finding 引用 `TI-UI-xxx`，不复制整段文本。
2. **新增码必须有机读判据**：没有 gate/doccheck/doccheck2/工具断言支撑的失败模式不编码（防空转）。
3. **码只增不改**：语义变更 = 新码 + 旧码标 `superseded`，避免历史证据失效。
