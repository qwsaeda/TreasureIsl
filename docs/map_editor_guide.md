# 地图编辑器使用指南（图文并茂版 · 极简分步 · 向导聚焦）

> **定位**：把「做一张地图包」的完整流程做成**按步引导式（wizard）可视化编辑器**，让内容运营从「改 4 个 JSON 文件 + 跑 9 条命令」变成「**跟着 10 步走，每步看说明、点按钮、看结果**」。
>
> **核心理念（用户 2026-11-09 拍板）**：**一步一步设计，每一步都有设计渲染图，可多次生成后合成效果**。本文档即按此组织：10 步各一节，每节 = 设计渲染图 + 详细说明 + 验收清单 + 怎么操作 + 前置条件。
>
> **设计定案（变体 A 向导聚焦）**：极简分步，**一次只亮当前步工作区**，其余折叠。顶部 10 圆点细步进条（已完成绿✓ / 当前金高亮 / 失败红✗），当前步 = 编辑区（add/import 亮岛色板+环画布+属性）或结果卡（其余步亮本步命令+结果回显），底部单行状态条。

## 一、编辑器在哪、怎么用

- **入口**：Dev Console（`cargo run -p ti-console`，默认 `http://127.0.0.1:8787`）→ 顶部菜单「地图」页 → 点右上角「打开编辑器」按钮。
- **界面骨架**（极简分步 · 向导聚焦）：
  - **顶部细步进条**：10 圆点横排（①新建包 ②填内容 ③同步 ④校验 ⑤打包 ⑥安装 ⑦对账 ⑧回注 ⑨出图 ⑩零 Rust），已完成绿✓ / 当前金高亮 / 失败红✗ / 待办灰。下方一行 = 当前步标题「第 N/10 步 · 步名」+ 包 id 输入。
  - **当前步工作区**（按步显隐，**一次只亮一个**）：
    - **add/import 步** = 编辑区（左岛色板 10 类 + 中环画布 31 格 + 右属性/内容配置/校验）。
    - **syncmap~zero_rust 步** = 结果卡（本步命令 + 结果回显 + 产出路径）。
  - **操作条**：▶ 执行当前步 + 结果回显 + 下一步 → / 刷新（读向导态）。
  - **底状态条**：选中格 + 当前步 + 校验通过数（极简单行）。

- **操作链路**：左边选岛 → 中间点格铺岛 → 上面步进条跟着走 → 卡在哪步，红字 + 验收清单告诉你还差什么。

## 二、10 步制作流程（每步一节，图文并茂）

> 步序与 `crates/ti-console/src/content/map_wizard.rs` 的 `MAP_STEPS` 1:1。机器判据：PASS → 自动推进下一步；FAIL → 失败即停（halted），只能重跑该步。**不可跳步**。

---

### 第 1 步 · ① 新建包（add）

![第 1 步 设计渲染图](../evidence/ui_concept/map_editor_step_01_add.png)

- **这一步干嘛**：从 `_template` 建 4 文件骨架（manifest/map/layout/strings）。**已安装态幂等跳过**（内容就绪即过，不重复建）。
- **验收清单**：`assets/maps/<id>/` 出现 manifest/map/layout/strings 四文件；校验 0 违规。
- **怎么操作**：点「执行」= 本地校验 I1~I9+S34.5 全绿 + 调后端向导 add 步（已安装且 4 文件在位 = 幂等 PASS）。
- **前置**：包 id 合法（`^[a-z0-9_]{1,16}$`）。

### 第 2 步 · ② 填内容 + 导入落盘（import）

![第 2 步 设计渲染图](../evidence/ui_concept/map_editor_step_02_fill.png)

- **这一步干嘛**：在编辑器把 **31 格**（左选岛→中铺格）+ **events** + **merchants** + **vs_theme**（右区内容面板）填完，点「导入落盘」落 4 文件。
- **验收清单**：I1~I9 + S34.5 全绿（0 违规）+ `POST /api/map/import` 返回 ok + 向导 import 步 PASS。
- **怎么操作**：左区选岛型 → 中区点格铺岛 → 右区填 events/merchants/vs_theme → 点「执行填内容」= 导入落盘 + 向导推进。
- **前置**：校验条必须 0 违规才能导入（I3 每类岛必须 2 格同 group，铺岛须成对）。

### 第 3 步 · ③ 同步 manifest[map] ← map.json（syncmap）

![第 3 步 设计渲染图](../evidence/ui_concept/map_editor_step_03_syncmap.png)

- **这一步干嘛**：manifest 里嵌的 map 段和 map.json 是冗余副本，导入后须显式同步（幂等）。
- **验收清单**：`map_pack.py syncmap` 输出「0 差」。
- **怎么操作**：点「执行同步」。
- **前置**：第 2 步已导入。

### 第 4 步 · ④ 权威校验（verify）

![第 4 步 设计渲染图](../evidence/ui_concept/map_editor_step_04_verify.png)

- **这一步干嘛**：`map_pack.py verify` 终判（含 **I7 资产 sha256 哈希**，前端镜像不含，须走权威链路）。
- **验收清单**：verify 输出「0 违规」（schema/兼容/I1~I9/S34.5 全过）。
- **怎么操作**：点「执行校验」。
- **前置**：第 2 步已导入。

### 第 5 步 · ⑤ 出可分发件（pack）

![第 5 步 设计渲染图](../evidence/ui_concept/map_editor_step_05_pack.png)

- **这一步干嘛**：已装包 → `dist/maps/<id>.map-pack/` + `.zip`（含 sha256 清单 + 池资产副本），可分发。
- **验收清单**：pack 成功 + dist 产物存在。
- **怎么操作**：点「执行打包」。
- **前置**：第 4 步 0 违规。

### 第 6 步 · ⑥ 安装 + 回注（install）

![第 6 步 设计渲染图](../evidence/ui_concept/map_editor_step_06_install.png)

- **这一步干嘛**：装到 `assets/maps/<id>/` + 接线 `board.json` maps.<id> + 更新 index.json。
- **验收清单**：install 成功。
- **怎么操作**：点「执行安装」。
- **前置**：第 5 步已出包。

### 第 7 步 · ⑦ 漂移对账（check）

![第 7 步 设计渲染图](../evidence/ui_concept/map_editor_step_07_check.png)

- **这一步干嘛**：已装态 vs board.json 全量对账（只读）。
- **验收清单**：check 输出「0 漂移」。
- **怎么操作**：点「执行对账」。
- **前置**：第 6 步已安装。

### 第 8 步 · ⑧ 回注构建期默认数据（sync）

![第 8 步 设计渲染图](../evidence/ui_concept/map_editor_step_08_sync.png)

- **这一步干嘛**：全量重算 board/scenes/manifest/strings（幂等）。
- **验收清单**：sync 成功。
- **怎么操作**：点「执行回注」。
- **前置**：第 6 步已安装。

### 第 9 步 · ⑨ 出图验收（shot）

![第 9 步 设计渲染图](../evidence/ui_concept/map_editor_step_09_shot.png)

- **这一步干嘛**：引擎真渲染 main_map（`TI_MAP=<id>`）截图 + 非黑判据（像素多样性 ≥4 种）= 视觉签收。
- **验收清单**：exit 0 + 出图非黑（`evidence/bevy_native_main_map.png`）。
- **怎么操作**：点「执行出图」（xvfb 无头真渲染）；可另点「看出图」直接打开 PNG。
- **前置**：第 8 步已回注。

### 第 10 步 · ⑩ 零 Rust 判据（zero_rust）

![第 10 步 设计渲染图](../evidence/ui_concept/map_editor_step_10_zero_rust.png)

- **这一步干嘛**：游戏 6 crate `git diff` 空 = **内容运营核心判据**（只动数据不碰代码）。
- **验收清单**：`git diff crates/ti-core…ti-formats` 空（0 diff）。
- **怎么操作**：点「执行断言」。
- **前置**：全程未改游戏 crate。

---

## 三、总览图（10 步拼图 + 流程说明）

![10 步总览](../evidence/ui_concept/map_editor_steps_overview.png)

- 5×2 缩略拼图，每格 = 该步设计渲染图（步进条高亮 + 该步工作区）。
- 步序流转（不可跳步）：① 新建包 → ② 填内容+导入 → ③ 同步 → ④ 校验 → ⑤ 打包 → ⑥ 安装 → ⑦ 对账 → ⑧ 回注 → ⑨ 出图 → ⑩ 零 Rust。

## 四、交互设计稿（3 变体选型）

**选型 = 变体 A 向导聚焦**（一次只亮当前步工作区，认知最聚焦）。B 上下分屏（画布常驻）、C 纯步骤卡（无常驻画布）为备选，效果图：

| 变体 | 设计稿 | 结论 |
|------|--------|------|
| **A 向导聚焦** | ![A](../evidence/ui_concept/map_editor_v3_minimal_a.png) | ✅ 选定：顶部 10 圆点 + 当前步专属工作区，最贴「一次只亮一个步骤」 |
| B 上下分屏 | ![B](../evidence/ui_concept/map_editor_v3_minimal_b.png) | 备选：画布常驻 + 下步骤卡，边看边调但下半屏稍重 |
| C 纯步骤卡 | ![C](../evidence/ui_concept/map_editor_v3_minimal_c.png) | 备选：10 卡竖排无常驻画布，最轻但失去所见即所得 |

## 五、节点颜色语义（与真渲染 main_map.rs 对齐）

- **节点 tile 底色 = 两态**：`tile_owned`（无主/有主，中性）/ `tile_deny`（锁定，暗红）；**不随事件/商人内容变色**（随机/每局事件无专属底色）。
- **岛色带（顶部）= 动态**：颜色来自 `theme.json` 的 `isle_*` 令牌（10 类，运行期随主题切换），非编辑器画死。
- **岛类型 = 固定 10 类闭集**（boss_fort/harbour/lighthouse/lightship/pirate_lair/shipyard/skull_cliff/trading_post/volcano/wreck），不可加新类，只能重分配 20 个领地格（I3 每类必须 2 格同 group，须成对改）。

## 六、踩坑（已固化进编辑器）

1. **I3 成对**：铺/清任一岛型格，该类格数偏离 2 立即红字标出，必须成对改。
2. **pos 越界**：拖动自由摆放 pos 超出 `world_canvas`（底 1310 / 左右 7~1059）立即红字。
3. **不可跳步**：向导状态机（map_wizard.rs）失败即停，只能重跑当前失败步；已安装态 add 步幂等跳过（修掉 halted 死锁）。
4. **events/merchants/vs_theme = I9+S34.5 硬门禁**：前端逐条 ✓/✗，不过 0 违规不能导入。
5. **只动数据不碰代码**：第 ⑩ 步 `git diff crates/` 空 = 内容运营判据（0 游戏 crate 改动）。

## 七、设计渲染图怎么再生成（可复用 / 多次后合成）

- **脚本**：`python3 scripts/map_editor_step_render.py`（11 张 = 10 步 + 总览，幂等覆盖）。
  - 无参 = 全出；`--only 04_verify` = 只重出指定步；`--check` = 只读校验产物在位。
- **交互设计稿**：`python3 scripts/map_editor_redesign_v3_minimal.py`（3 变体 A/B/C，`--check` 只读）。
- **总览合成**：脚本自动 5×2 拼图 + 流程说明（`map_editor_steps_overview.png`）。
- **出图后可**：多次生成不同包/不同步的渲染图，PIL 拼图合成对比效果（参考 `map_editor_design_gen.py` 的拼图范式）。
- 图落 `evidence/ui_concept/map_editor_step_*.png` + 总览。

## 八、视觉签收（用户过目）

| 签收图 | 内容 |
|--------|------|
| `evidence/ui_concept/map_editor_step_01_add.png` ~ `10_zero_rust.png` | 10 步各一张设计渲染图（极简分步版） |
| `map_editor_steps_overview.png` | 10 步总览（5×2 拼图 + 流程） |
| `map_editor_v3_minimal_{a,b,c}.png` | 3 变体交互设计稿（选型 A） |

**判据 = 用户视觉确认（本图 + 上述签收图过目，视觉签收类任务收口口径见 `docs/process/26_dev_stages_overview.md` 视觉签收清单）。**

## 九、文件/命令速查

| 物 | 位置 |
|----|------|
| 编辑器（前端，0 游戏 crate） | `crates/ti-console/web/index.html`（#map-editor 卡片 + ed* 函数族） |
| 向导状态机（10 步） | `crates/ti-console/src/content/map_wizard.rs` |
| 导入落盘 API | `POST /api/map/import`（`repo/import.rs`，先 verify 后 sync） |
| 权威校验 | `python3 scripts/map_pack.py verify <id>`（含 I7 哈希） |
| 设计渲染图生成 | `python3 scripts/map_editor_step_render.py`（`--check` 只读） |
| 交互设计稿（3 变体） | `python3 scripts/map_editor_redesign_v3_minimal.py`（`--check` 只读） |
| 视觉签收 | `python3 scripts/map_editor_visual_signoff.py` |
| 概念图生成 | `scripts/map_editor_design_gen.py` / `map_editor_redesign_concepts.py` |
