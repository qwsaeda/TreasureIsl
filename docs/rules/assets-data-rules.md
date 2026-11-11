---
paths:
  - "assets/**"
  - "assets/data/**"
  - "assets/ui/**"
  - "assets/maps/**"
  - "assets/ui_manifest.json"
  - "assets/registry.json"
---
# assets/data 编码红线（资产/内容数据域，CCGS `rules/*.md` `paths:` 模式借鉴，S50.3）

> 本文件 = `docs/rules/` 路径作用域红线速查面（给 LLM 子代理开轮读，**非机器拦截**；
> 机器拦截仍走 `validate_data.py`（`check_asset_registry` disk↔registry 双向同步 +
> `check_scenes_canonical` scenes.json indent=1 规范形）+ `map_pack.py verify`（I1~I9 不变量）
> + `check_data_source.py`（数据源改动 → 渲染不变式门禁）。
> 单一事实源 = `docs/process/18_asset_content_pipeline.md`（不手写像素 + 资产登记标准）+
> `docs/process/21_map_package_spec.md`（地图包规范 v1 + I1~I9），本文件只做路径域速查 + 反例，不重述结论。

## 红线（4~6 条，违反 = 返工）

1. **资产登记单源**：UI 图走 `assets/ui_manifest.json`（清单唯一写入路径 = `ui_assets.py register-*`
   子命令，indent=2 保序零 churn），字体/音频走 `assets/registry.json`；全类型 disk↔registry 双向
   同步（`validate_data` `check_asset_registry`），未登记资产 = 门禁 FAIL。反例：手生
   `assets/ui/icon_lamp.png` 不登记 `ui_manifest.json`（应 `ui_assets.py add` 登记）。
2. **不手写像素（§18 总纲）**：声明意图/数据，引擎算坐标——UI = bevy_ui 自动布局（S29.3）/
   地图 = 参数化（S27.4）/ `scenes.json` 退成元素清单；不手工画像素级坐标（18 号行业三档
   ①绝对坐标 / ②意图+自动布局 / ③参数化，本项目终态 = ②+③）。反例：`scenes.json` 手写
   `rect: [43, 250, 994, 826]` 绝对像素（应走参数化 `bbox_pct` + bevy_ui 自动布局）。
3. **地图包 I1~I9 不变量（§21）**：31 格构成（start1 + territory20 + 功能9 + boss1）/ 10 类岛各
   2 格同区 / BOSS 在环半周 `spaces[31//2]` / 8pt / 资产哈希齐 / N8 焦点 ±1px / I9 内容齐备；
   改地图包须 `map_pack.py verify` 全过（违规非 0），不手改多份 JSON。反例：手改
   `assets/maps/m01/layout.json` 环几何不跑 `map_pack.py verify`（I7 资产哈希漂移）。
4. **`scenes.json` 序列化纪律（indent=1 逐字节规范形）**：`check_scenes_canonical` 守 `scenes.json`
   须 indent=1 逐字节规范形（防整文件 churn，S29.2 曾误改 5672 行）；改 `scenes.json` 须经
   数据化通道（`ui_theme.py` / `map_pack.py sync` / 平台写端点），不手改 raw JSON。反例：手编辑
   `scenes.json` 用 indent=2（`check_scenes_canonical` FAIL 整文件重排）。
5. **`assets/data/*.json` 结构合法域（`validate_data`）**：`balance.json` / `strings.json` /
   `board.json` / `scenes.json` / `theme.json` 必填字段 + 值域 + 孤儿 key 全过 `validate_data.py`
   （门禁第 10a 项）；新 key 先落 `strings.json` / `balance.json` 再在代码读。反例：
   `balance.combat.crit_rate = 25`（越界，`validate_data` 值域 0..=20 FAIL）。

## 正确 / 反例（1~2 条，给子代理开轮速查）

- 正确：`map_pack.py add m21` 从 `_template` 引导新包 → 填 `map.json` → `syncmap` 同步
  `manifest["map"]` → `verify`（I1~I9 0 违规）→ `pack` 出 `dist/maps/m21.map-pack/` → `install`
  接线 → `sync` 回注 `board.json` → `check` 0 漂移 → `TI_MAP=m21 --shot main_map` 出图。
- 反例：手生 `assets/maps/m21/map.json` 不跑 `map_pack.py syncmap`（`manifest["map"]` 与
  `map.json` 漂移，`check` 报 I9 内容不齐，29 号 §5 坑位固化）。
