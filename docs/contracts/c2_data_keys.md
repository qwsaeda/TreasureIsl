# c2 — 数据表 key 契约（`assets/data/*.json`）

**状态：draft**（S12.0 契约冻结轮置 `frozen`）

跨包**只经 JSON key 交换数值/文案**。本文定的是「哪个 key 属于谁、谁许改」，
不是「值是多少」（值在表里改，改值不改代码 = S8 分层约定）。

## 1. 四张表

| 表 | 生成方式 | 改值路径 | 改 key 路径 |
|---|---|---|---|
| `balance.json` | 手写（策划） | 改 JSON → `validate_data.py` 校验 | 契约轮仲裁（全票） |
| `strings.json` | 手写（策划/本地化） | 改 JSON | 新增 key = 包内可加；删/改 key 名 = 契约轮 |
| `scenes.json` | 手写（布局） | 改 JSON | 契约轮 |
| `controls.json` | `legacy/scripts/gen_layout.py` 生成（gate 第 10b 项幂等） | **不许手改**（改源数据再生成） | 契约轮 |

## 2. `balance.json` 顶层段（`validate_data.py` 校验合法域）

| 段 | 内容 | 主要消费者 |
|---|---|---|
| `economy` | 初始金、银行、金上限 | P2 经济域 |
| `combat` | 伤害模型、BOSS 档、回合 | P1 战斗域 |
| `trade` | 商品、买卖价、markup、qty | P2 经济域 |
| `build` | 建筑、造价、等级 | P2 经济域 |
| `cards` | 卡池、权重、触发概率、页数 | P3 地图卡牌域 |
| `messages` | 消息分页、kind 集合 | P4 流程菜单域 |
| `save` | 存档槽位数 | P5 存档域 |
| `selection` | 角色/地图目录 | P4 流程菜单域 |
| `ui` | 界面常量 | P6 表现层（后续轮） |

## 3. `strings.json`（89 键）

- 键命名规范：`<域>.<对象>.<字段>`（如 `scene.select_role.title`、`msg.help.title`）；
- **包内可新增**自己域的 key；**不得改/删**其他域的 key（改 = 契约轮）；
- 孤儿 key（表里有、代码未引用）由 doccheck2 的 i18n 检查 WARN，不阻塞。

## 4. 加载纪律（跨包统一约定）

1. 全部经 `ti_core::data::{balance, strings, scenes, controls}` 访问器读表（`OnceLock` 缓存）；
2. 解析失败 = **硬错误**（不 fallback 默认值，不静默吞）；
3. 数值断言测试读 JSON 里「实际写着的值」，不抄公式（`02_conventions.md` §5）。

## 校验锚点

```
assets/data/balance.json
assets/data/strings.json
assets/data/scenes.json
assets/data/controls.json
ti_core::data::balance
ti_core::data::strings
ti_core::data::scenes
ti_core::data::controls
```
