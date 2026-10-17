# 执行计划 S3：玩法逆向与重写（主体，4~8 周）

> **定位**：S3 是逆向期玩法重写计划，**已 archived**（S3.1–S3.12 全 verified），非正向开发依据。当前正向开发见 `docs/00_design.md` + `docs/business/`。

台账驱动的模块循环。**每个模块必须走完 8 个固定子步骤**才可 `verified`，禁止跳步。

## 1. 每模块固定子步骤（缺一不可）

```
1 analyzed    读 ref/pseudocode/ 对应函数 → 在 ref/functions.csv 标注模块归属
2 specified   写 legacy/docs/spec/<module>.md：输入/状态机/输出/数值公式/控件字段（f5 类型码，见 ref/controls.json）/事件语义
3 oracle      ti-core/src/oracle/<module>.rs 直译伪代码（测试专用，允许与正式实现重复）
4 implemented ti-core/src/rules|<module>.rs 正式实现（实现 §5.6 的 trait，非 dyn）
5 diff        diff_test!(oracle, impl, cases) → cargo test -p ti-core --test diff_<module> 全绿
6 coverage    cargo run -p ti-tools -- coverage events|scenes → 本模块 missing = 0
7 gate        scripts/gate.ps1 全绿（S3 时点 8 项，现 21 项；占位扫描 0 命中）
8 evidence    日志写入 evidence/S3.x.log → ledger 置 verified
```

## 2. 模块清单与依赖

| ID | 模块 | 关键内容 | deps | 验收重点 |
|---|---|---|---|---|
| S3.1 | 主循环/场景管理 | `WinMain`→消息泵→GX 绘制→场景栈；帧率/定时 | S2.7 | 场景切换图与 ref 一致；`define_scene!` 接入 |
| S3.2 | 资源/渲染核心 | pak 装载、句柄缓存、图层 blt、字体、调色 | S1.10, S3.1 | P4-1 渲染逐像素一致 |
| S3.3 | index/intro 主菜单 | 菜单控件（ref/controls.json index.Map 31 条，f5 类型码）→ 各子场景进入逻辑 | S3.2 | 场景可达 + 控件类型逐档解释（F-311） |
| S3.4 | selectrole/selectmap | 5 角色（panda/tichy/kidd/drake/katharina）、3 地图选择 | S3.3 | 流程脚本 + 差分 |
| S3.5 | 主地图 | `glmapm0x` 加载、移动、据点/地标、探索 | S3.4, S1.8 | 地图数据对齐 + 移动规则差分 |
| S3.6 | 贸易 | goods 数值、行情、买卖结算、利润公式 | S3.5, S1.6 | 固定价格表逐笔结算差分 |
| S3.7 | 建造/城堡 | 建筑、投资、银行 | S3.5 | 差分 + 事件覆盖 |
| S3.8 | 战斗 | vs 场景、兵种 troop/ship、BOSS 档（exe 内 50000~300000）、胜负判定 | S3.5 | 固定种子整场战斗输出一致 |
| S3.9 | 卡牌/事件 | cards/event/catalog 规则、`event%d` 触发 | S3.6, S1.6 | 全部 event 有处理路径（P3-1）；event 与 f5 控件类型的关系在此定论 |
| S3.10 | 存读档/设置 | saveload/option/settings、字段编解码 | S1.9 | v1 兼容读写 + 迁移测试 |
| S3.11 | 消息/帮助/信息 | msgs/msgl/msgm/help/information 界面 | S3.2 | 文本渲染对齐 + 场景可达 |
| S3.12 | RNG 与全局规则 | 随机数发生器、全局常量表、RNG 序列对齐 | S3.1 | RNG 序列与 oracle 一致；**`ref/classify.json` 中 `pending = 0`** |

依赖未全 `verified` 的任务不可取用（loop 第 1 步校验）。模块间如发现依赖冲突，调整 deps 并在台账 `blocked_by` 说明，不许跳过前置。

## 3. 阶段门禁（S3 完成 → 可进 S4）

- 12 个模块全部 `verified`，每份 evidence 齐全；
- `docs/parity.md` **P2、P3 全绿**：差分测试全通过、事件覆盖 100%、场景覆盖 100%、固定种子全流程脚本跑通（P3-3）；
- `ref/classify.json` 覆盖率 `pending=0`（exe 中每个函数都有归属，无漏网逻辑）；
- `scripts/gate.ps1` 全绿（S3 时点 8 项，现 21 项）。

## 4. 模块推进中的记录要求

- 发现伪代码与 `legacy/docs/facts/index.md` 冲突 → 先实测修正事实库，再改 spec（顺序不可倒置）；
- 同一坑第二次遇到 → 必须写 `docs/experience/index.md` 并在该模块 evidence 中引用条目 ID；
- 数值表一律资源化进 `assets_out`，代码里只留常量名（§5.3）。
