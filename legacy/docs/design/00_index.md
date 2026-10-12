# docs/design/ — 弃用/归档索引（S8，2026-10）

> ⚠️ **本目录为逆向规格参照物，不再是开发依据或验收基准。**
> 业务规则唯一事实源 = [`docs/business/`](../business/00_index.md) + `assets/data/*.json`。
> 见 [`docs/plan-s8-refactor.md`](../plan-s8-refactor.md) §1（逆向产物降级）。

## 定位

S8 标准化改造将逆向产物（`docs/facts/`、`docs/spec/`、`ref/pseudocode/`、`oracle/`、`docs/design/`）
统一降级为**历史参照物**：记录原版游戏形态，供理解原版机制与核对实现细节使用，
但不再作为新开发或验收的依据。

## 文件清单

| 文件 | 内容（原版形态参考） |
|---|---|
| [`combat.md`](combat.md) | 战斗系统（BOSS 档/伤害公式/回合制）+ 卡牌事件系统 |
| [`constants.md`](constants.md) | 全局常量表（分辨率/金上限/BOSS HP/RNG 常数/价格表等） |
| [`economy.md`](economy.md) | 贸易系统 + 建造系统（建筑/银行/金上限） |
| [`input_mapping.md`](input_mapping.md) | 输入映射层（键盘+触摸双路径/手势/场景特定操作） |
| [`main_loop.md`](main_loop.md) | 主循环与游戏状态机（场景枚举/状态转换/跨场景状态/实现路由） |
| [`map_system.md`](map_system.md) | 地图数据、地形网格几何、移动规则、据点/地标、事件分派 |
| [`persistence.md`](persistence.md) | 存读档（文件布局/魔数/迁移）+ RNG（LCG 算法（oracle 历史参照）/PCG（当前实现）/播种/调用点） |
| [`scene_flow.md`](scene_flow.md) | 场景状态机、主菜单、角色/地图选择、事件码体系 |
| [`ui_controls.md`](ui_controls.md) | 消息界面 + 控件表 + 资源容器 + 渲染管线 + 控制方案 |

## 使用指引

- 查业务规则 → [`docs/business/`](../business/00_index.md)
- 查数值/文本 → `assets/data/*.json`
- 查原版实现形态 → 本目录（仅作参考，数值可能已过时）
