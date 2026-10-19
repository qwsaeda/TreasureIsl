# UI 重设计 v3 · 真游戏化版面（2026-10-18 启）

**定位**：v3 系列 = 打破 S17 "航海夜·精修" 模板化 7 场景同构的工程化重设计。
v2 (`10_ui_redesign.md`) 是配色/字号/图标局部精修，**v3 是版面结构与场景叙事的重构**。

## 本系列文档

| # | 文件 | 内容 |
|---|---|---|
| 1 | `01_benchmarks.md` | 对标游戏版面拆解（盗贼之海/大航海时代 IV/Hades/八方旅人） |
| 2 | `02_spec_lang.md` | **UI 描述语言规格**（硬约束 DSL，可机器断言） |
| 3 | `03_redesign_proposal.md` | 7 场景重设计提案（每场景独立视觉重心） |
| 4 | `04_index_impl.md` | index 主菜单落地实现（scenes.json + chrome.rs 改动） |
| 5 | `05_vs_impl.md` | vs 战斗落地实现（battle_arena 竞技区装饰层 + 双写键 + 像素证据 + v3.4 后续波） |
| 6 | `06_main_map_impl.md` | main_map 全量落地（board_fit G 依赖棋盘拟合 + HUD 边角化 + 底部命令盘 + 重锚 e2e + v3.5 收口） |
| — | `../../assets/data/ui_schema/scene_manifest_v3.schema.json` | 场景清单 JSON Schema（DSL 的机读形态） |

## 与既有规范的关系

- **取代**：`docs/process/10_ui_redesign.md §4 版面增量表`（8pt 整除强制对齐）的"7 场景同模板"前提
- **保留**：`docs/business/08_layout_design.md §2 配色 v2` 13 色令牌、`§3 字体四档`、`§5 7 不动`（业务规则/输入映射/场景流不变）
- **新增约束层**：在几何约束（坐标/越界/对齐）之上，加 **场景叙事约束**（视觉重心 + 情绪节奏 + 信息密度），由 `02_spec_lang.md` 定义的 DSL 描述、由新门禁 `check_scene_narrative` 断言

## 实施波次

```
v3.0 调研 + 规格（本文档系列） → v3.1 7 场景重设计提案落 scenes.json
       → v3.2 chrome.rs 扩展原语（焦点层 / 留白带 / 视觉重心锚）
       → v3.3 index 主菜单试点 → v3.4 推广其余 6 场景 → v3.5 收口 + 旧 8pt 断言退役
```

## 硬规则

1. **先规格后实现**：任何 scenes.json 改动必须先在 `02_spec_lang.md` DSL 里可表达、可断言
2. **不破坏 P0~P2 业务 parity**：游戏规则/输入/场景流/存档/经济/战斗数值不变
3. **每场景必须有且仅有一个"视觉重心"**：禁止"标题条+列表+提示行"三件套通吃
4. **门禁可测**：每条场景叙事约束必须能被 `check_scene_narrative` 脚本判定 PASS/FAIL
5. **legacy 像素层不回归**：仍 100% 代码绘制 + TTF，不重新引入 PCX 精灵做版面定位
