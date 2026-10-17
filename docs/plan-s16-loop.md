# S16 试玩闭环 + 发布 + 设计 sheet（WBS）

**定位**：S15（业务场景穷举测试）之后的**运行时试玩质量闭环** + 发布链路 + 正向内容设计模版。
外部参照：开源 OhMyGame（Agent 驱动游戏制作工作室）——只吸收「试玩闭环 / 一键发布 / 设计-素材-代码同上下文」三个概念，
**不吸收** OS 级窗口操作试玩（headless 窗口截图挂死坑见 `evidence/B-native-menu.log`，渲染线 headless-first 立场，EXP-078）
与 AI 素材生成（本项目素材全部原版逆向真实资产 + S7 HD 管线，非 AI 生成目标物）。

## 前置与约定

- 开工前置：S15.2 verified（渲染线 WIP 收敛 + 视觉基线重生成，EXP-078 的 19 个既有测试失败清零）。
- gate 顶层项数**以当前清单为准**，新增项编号按现行清单顺序追加，本文与台账**不写死项数**。
- 所有 DoD 判据必须实跑输出落 `evidence/S16.x.log`（硬规则 2：只贴 gate 总输出不算验收证据）。
- 本机环境事实（2026-10 实测）：Chrome `C:\Users\Administrator\AppData\Local\Google\Chrome\Application\chrome.exe` 存在（`$env:REA_BROWSER_EXECUTABLE` 可覆盖）；Node v24（内置 `http` + 全局 `WebSocket`，CDP 驱动**零 npm 依赖**）；Python 3.13。
- 远端 `origin = https://github.com/qwsaeda/TreasureIsl.git`；GitHub Pages 可用性第一轮实测定态（私仓需付费计划，失败登记 release.md 已知限制，不静默绕过）。

## 任务表

| id | 依赖 | 产物 | 一句话 DoD |
|---|---|---|---|
| S16.1 | S15.2 | `scripts/web_playtest.ps1` + `scripts/web_playtest.cjs` + gate「browser-playtest」项 + `playbook.ps1` `web-playtest` 命令 | 浏览器全交互 0 失败，报告落 evidence |
| S16.2 | S16.1 | `docs/process/08_playtest_loop.md` + gate「playtest-loop」项 + `playbook.ps1` `playtest-loop` 命令 | 4 报告 0 失败 + 3 轮收敛停机，主线程复核 |
| S16.3 | S16.2 | `scripts/publish_web.ps1` + `gh-pages` 分支 + `release.md` 发布节 | 一条命令：本地全绿 → Pages 线上复跑 web_playtest 全绿 |
| S16.4 | S16.2 | `docs/process/03_templates.md` 新增 §「内容设计 sheet」+ 样例骨架（标 `模板` 前缀） | doccheck 0 error + 模版登记 README |

## S16.1 浏览器真运行时试玩（web_playtest）

把 wasm 目标端（`web/`）从「HTTP fetch 冒烟」（S2.5 口径）升级为**真浏览器运行时门禁**。

**产物**
- `scripts/web_playtest.ps1`（编排：`build_wasm.ps1` 更新 `web/` → 起本地 serve → 调 .cjs → 聚合报告 → gate 项判定）
- `scripts/web_playtest.cjs`：headless Chrome（`--remote-debugging-port=0` + 临时 user-data-dir，进程/目录全清理）
  → CDP 连 `web/index.html` → 按 `tests/scripts/full_run.json` **同一套按键序列**经 `Input.dispatchKeyEvent` 驱动
  菜单→选人→选图→主地图 → 每步断言：
  1. console 致命错误（uncaught exception / failed to load module / 资源 404）= 0；
  2. 截图墨点覆盖率 > 阈值（非空白/非纯黑）；
  3. 相邻场景切换帧的像素统计有变化（场景确实推进，防止卡死在首场景仍"全绿"）。
- 默认本地 URL（`serve web/`）；`-Url` 参数支持任意 URL（S16.3 复用到 Pages 线上地址）。

**DoD / evidence（`evidence/S16.1.log`）**
- 全交互通过：逐步 0 失败（步数与 full_run 等效口径）+ console 致命错误 0 + 逐帧非空白；
  记录 chrome 版本 + `web/` 产物 sha256。
- gate 新增「browser-playtest」项（编号按当前清单追加；条件化：wasm/ 构建缺失时 SKIP 沿用 S2.5 模式）；
  负例：注入必然 `console.error` 的坏页 → 该项 FAIL 且退出码非 0。

**降级预案（第一轮先实测定态，不假设）**
- 若 CDP 键盘事件无法驱动 miniquad canvas 交互：基线降为「加载 + console + 截图」，
  交互断言降级为 known-limitation → 登记 `docs/release.md` 已知限制 + 回写 EXP（loop §4 规则 2：计划与实测冲突以实测为准）。

## S16.2 试玩 agent 闭环 loop（process/08_playtest_loop.md）

运行时试玩版飞轮：06_flywheel 管静态审计（找漂移），08 管运行时（找玩法/交互缺陷）。

**产物**
- `docs/process/08_playtest_loop.md`（单一事实源）：
  - 4 角色：试戏（聚合 4 份运行时报告：`fuzz_playtest` S13.1 + `full_run` S5.2 + `visual_regression` S13.4 + `web_playtest` S16.1）
    → 归因 → 修复 → 复测；
  - 归因三分类（沿用 06_flywheel §3 回写表）：确定性测试缺陷 → 修码+补测试+evidence；可机械判定漂移 → doccheck2 断言；
    设计分歧/环境抖动 → EXP + 裁决理由；
  - 开轮必读复用 `scripts/flywheel_readme.ps1`（06_flywheel §6，不重造）；修复自验纪律照抄（test + clippy 全绿才回写）；
  - 与 06_flywheel 关系表（静态 vs 运行时，共用开轮必读与三分类回写）。
- gate 新增「playtest-loop」项（检查 4 报告存在 + 收敛记录）；`playbook.ps1` 加 `playtest-loop` 命令（一条命令出 4 报告 + 三分类清单）。

**停机条件（照 06_flywheel §4 口径）**
- 4 报告全 0 失败 + 连续 3 轮收敛（新运行时缺陷=0 / 归因=0 未决 / 修复=0 残留）→ 停机；
  主线程独立复核（EXP-038/058：子代理声称不算数），证据 `evidence/S16.2.log`。

## S16.3 一键发布 publish-web（本地验证 + GitHub Pages）

**产物**
- `scripts/publish_web.ps1` 全链路：
  1. 前置 gate 全绿检查（复用 playbook push 同款逻辑）；
  2. `build_wasm.ps1` 重建 `web/` + 本地 `serve` 静态断言（index/gl.js/wasm/字体 200 + content-type，沿用 S2.5 判据）；
  3. 产物 sha256 + git short sha + 日期追加 `docs/release.md` 发布历史表；
  4. Pages 段：`web/` 经 `git worktree` 推 `gh-pages` 分支（不污染 master 工作区）→ `gh api` 建/更新站点
     → **线上 URL 复跑 S16.1 `web_playtest.cjs -Url https://qwsaeda.github.io/TreasureIsl/`**（console/截图/流转全绿才算发布成功）。
- Pages 前置实测定态：仓库可见性/计划是否支持 Pages；不可用 → 发布历史表记「本地一键验证」为当前上限，已知限制登记。

**DoD / evidence（`evidence/S16.3.log`）**：一条命令全链路通过（本地 + 线上）；`release.md` 新增「发布」节；gate 全绿。
回写 `AGENTS.md` 路由表发布入口 + `docs/README.md`。

## S16.4 新内容设计 sheet 模版（Canvas 概念落地）

**产物**
- `docs/process/03_templates.md` 新增 §「内容设计 sheet」：
  ① business/ 设计文档段落（规则 + 可测断言，单一事实源）
  ② `assets/data/*.json` key 契约（挂 c2 校验，`docs/contracts/c2_data_keys.md`）
  ③ scenes.json / 资源接线步骤（挂 `docs/process/07_render_pipeline.md` 契约 + element-dump 元素级验证法 §7）
  ④ 验收锚点（doccheck2 断言 + 渲染快照 + e2e 步骤）。
- 附「新地图 m04」空样例骨架，路径/键名一律显式 `模板` 前缀（防占位扫描与 doccheck 误判真实数据）。
- 登记 `docs/README.md`（03_templates 行说明更新）；本任务只落模版，**不实现真内容**。

## 执行顺序与工作量

```
S15.2 收敛（渲染线 WIP，前置）
  → S16.1 web_playtest（全交互优先，实测失败即降级并登记）
  → S16.2 playtest loop（3 轮收敛停机）
  → S16.3 publish-web（本地 → gh-pages → 线上复跑）
  → S16.4 设计 sheet 模版（文件域与 S16.3 不相交，可并行）
```

预估：S16.1 一轮（CDP 交互实测 + 可能降级裁定）；S16.2 三轮；S16.3 一轮 + Pages 实测定态；S16.4 半轮。
