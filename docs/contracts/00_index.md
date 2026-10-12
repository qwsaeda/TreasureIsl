# docs/contracts/ — 跨包契约（模块并行开发的冻结面）

**定位**：模块并行轮（`docs/process/05_module_parallel.md`）里「包之间只能经过这里」的公共面。
契约轮（① 契约冻结）把各边界定稿并置 `frozen`；并行轮期间**冻结段只允许主线程/契约轮改**
（全票同步），包内会话不许改。

> 机读锚点：每份契约末尾有 `## 校验锚点` 段（一行一个符号，`crates/<crate>::<path>` 形式），
> 供 `scripts/check_module_contracts.py`（gate 第 16 项，随 S11.4 落地）grep 校验「契约符号存在」，
> 与 `plan/packages.yaml` 的白名单互斥校验。

## 契约清单

| ID | 文件 | 边界 | 制定者 | 状态 |
|---|---|---|---|---|
| c1 | [`c1_core_public_types.md`](c1_core_public_types.md) | ti-core 公共类型（`domain/*` + `rules::rng` + `data` 访问器） | D 数据/工具包 | draft |
| c2 | [`c2_data_keys.md`](c2_data_keys.md) | `assets/data/*.json` 的 key 契约 | D 数据/工具包 | draft |
| c3 | [`c3_app_host_api.md`](c3_app_host_api.md) | 表现层会话 API（`GameApp`/`SceneHost`） | P6 表现层包 | draft |

## 规则

1. **状态只允许 `draft` / `frozen`**；`draft` 时开并行轮会 FAIL（第 16 项断言无 draft 残留）；
2. 契约只列**跨包可见面**（公共 struct 签名 / 枚举 / 函数 / JSON key），包内私有实现不进契约；
3. 改契约 = 全票同步（主线程仲裁）；改动须重跑第 16 项校验 + 全 workspace test；
4. 数值与文案**不写死在契约**，只写 `balance.json` / `strings.json` 的 **key**（值在数据表里改）；
5. 每份契约一个 `## 校验锚点` 段，一行一个符号，供机器 grep。
