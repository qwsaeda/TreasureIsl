# S49 计划 · 内容运营续集 + 平台主题管理 + 玩法补强（四方向）

> 用户 2026-11-08 拍板：S48.0 全主题创新收口后，S49「都做」四方向——①主题创新续集 ②新地图 ③平台增强 ④玩法补强。
> 里程碑判据（全 S49）：零 Rust（游戏产品 6 crate 0 diff，纯内容运营子任务）+ gate 全绿 + doccheck/doccheck2 0 error
> + 每子任务 evidence 日志 + 用户视觉确认留痕（A 纪律硬规则 9）。

## WBS

| 任务 | 方向 | 内容 | 形态 | 判据要点 |
|---|---|---|---|---|
| **S49.0** | 立项 | WBS 定案（本文）+ 用户确认留痕 + 登记（README/AGENTS 路由表）+ 台账 | 文档 | 文档登记 3 处 + 台账 S49.0 条目 + doccheck 0 error |
| **S49.1** | 主题续集 | ~~竖屏主题化~~（用户 2026-11-08 拍板「不要竖屏，现在都是横屏」→ **作废**，跳 S49.2） | — | 作废（blocked，blocked_by 记用户决定） |
| **S49.2** | 新地图 | **新地图包 m33~m40**（8 张，`map_pack.py` 向导 add→import→syncmap→verify→pack→install→check→sync→shot→zero_rust） | 内容运营，零 Rust | 8 张 verify 0 违规 + check 0 漂移 + 出图非黑 + 数据级差异 |
| **S49.3** | 平台增强 | **主题管理页**：`/api/themes`（list/apply/verify，沿用 content 向导模式）+ web 主题面板 | 平台 | API 三端点 + 页面 + 白名单/审计对账 + close HUMAN_VIEW_IDS 对齐 |
| **S49.4** | 玩法补强 | **vs 战斗动效**：攻击/受击/道具 4 帧动画 + 演出时间轴（对齐 12 号 VS v2 规格 + 11 号 VS 规格） | 表现层 + 业务 | 动效资产 + 时间轴接线 + 出图/录像 + 视觉确认 |

## 执行顺序

S49.0 → ~~S49.1（用户作废）~~ → S49.2（新图）→ S49.3（平台页）→ S49.4（战斗动效，表现层最后）。

## 关键约束

- **零 Rust**：S49.1/S49.2 纯内容运营，游戏产品 6 crate 0 diff（平台 ti-console + 工具允许改——S49.3 平台功能）。
- **S49.4 涉及表现层**：A 纪律设计稿先行（VS v2 规格已有），先出动效规格 + 用户确认再落码。
- **复用**：`ui_theme.py`（S48.0）扩展竖屏；`map_pack.py`（S31）新图；`ui_wizard`/`map_wizard`（S44）向导模式。
- **不重复**：S49.1 竖屏主题是 S48.0 横屏的补全（index 竖屏 42 原语目前未主题化）。

## 参考

- S48.0 横屏主题：`docs/process/30_content_ops_platform.md` §2.2.1（全主题规范）
- 地图包：`docs/process/21_map_package_spec.md` + `scripts/map_pack.py`
- 平台向导：`docs/process/30_content_ops_platform.md` §2.1/§2.2 + `crates/ti-console/src/content/{map,ui}_wizard.rs`
- 战斗动画：`docs/ui-redesign-v3/12_main_map_v6_vs_v2_spec.md` §2（VS v2 4 帧/兵力 HUD/时间轴）
