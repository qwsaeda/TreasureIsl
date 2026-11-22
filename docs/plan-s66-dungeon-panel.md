# S66 副本情报面板 WBS（2026-11-22）

## 改动面声明（硬规则 7）
规则实现（SceneHost 2 只读查询 + J 键）+ 数据（i18n）+ 表现（main_map 面板 + 互斥）+ 测试（e2e + 登记）。
跨模块原因：需查询接口+数据+表现+输入四面同改才可独立验收，不可拆。

## 差距（坐实）
plan-s63-subscenes.md T4 降级：落格 pending_dungeon+confirm_dungeon 已接线（自动提示+Confirm 进守军战），
独立面板无待决展示态会空白。本任务扩展 SceneHost 只读查询 + J 键面板完整兑现。

## 用户价值
用户拍板 8 面板补全最后一块可独立做的。落无主格按 J 提前看岛名/守军人手/武器/奖励预览，
再决定 Confirm 进战或 Back 放弃——把盲操作变成有情报的决策。

## 视觉（A 纪律）
复用已签收候选 A 羊皮卷轴（其余面板同款），非新视觉设计；新增仅内容文案（数据驱动）。

## 判据
仅 pending_dungeon 可开 / 守军查询与 rules defender_unit 一致 / J 键 poll / Confirm 进 Vs 战 /
Back 放弃待决 / 互斥 / gate 全绿 + 出图。
