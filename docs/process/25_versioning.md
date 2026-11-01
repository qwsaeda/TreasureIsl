# 25 · 版本与打包规范（Versioning & Packaging Standard）

> 2026-11-01 用户拍板「游戏打包使用 x.y.z 版本方式，更新规范；每次完成过清单，出版本规划」。
> 单一事实源：本文（版本号语义 + 打包产物命名 + 更新/发布流程）；机读版本 = `Cargo.toml` `[workspace.package] version`。
> 路由表 S35 条目；台账 = `plan/ledger.yaml`（版本相关任务）。

## 1. 版本号：x.y.z（语义化版本 SemVer）

**单一事实源** = `Cargo.toml` `[workspace.package] version = "x.y.z"`。所有产物（APK / wasm / 发布历史）
从该值读取，**禁止**在脚本/文档里散写版本字面量（防漂移，`doccheck` 数值断言兜底）。

| 位 | 语义 | 何时 bump | 例 |
|---|---|---|---|
| **major (x)** | 不兼容破坏 | 架构重写 / 数据格式不兼容 / 玩家存档破坏 / 系统级重设计 | `0.1.0 → 1.0.0` |
| **minor (y)** | 向后兼容新功能 | 新场景 / 新系统（船、领地、副本）/ 新玩法 / 新内容包 | `0.1.0 → 0.2.0` |
| **patch (z)** | 向后兼容 bug 修复 | 任何 bug 修复 / 视觉修正 / 性能修正 / 文案修正 | `0.1.0 → 0.1.1` |

**当前版本**：`0.1.0`（2026-11-01 起，S0–S35 演进中；pre-1.0 阶段 major 恒 0，minor 随新系统）。

## 2. 打包产物命名（带版本）

产物名一律嵌 `x.y.z`（读 `Cargo.toml` version，见 §4 工具改动面）：

| 平台 | 产物 | 命名 | 脚本 |
|---|---|---|---|
| Android | APK | `dist/TreasureIsl-<x.y.z>-<arch>.apk`（arch = arm64/x86_64） | `scripts/build_apk.sh` |
| 浏览器 | wasm | `web/treasure_bevy_wasm-<x.y.z>.wasm` | `scripts/build_wasm.ps1` |
| Android 清单 | `versionName` | `android:versionName="<x.y.z>"` | `android/AndroidManifest.xml`（同步） |

**不变量**：`build_apk.sh` / `build_wasm.ps1` 产物名**必须**含 `Cargo.toml` version 的 `x.y.z`；
产物名与 version 不一致 = 打包漂移（`validate_data` / `doccheck` 兜底断言）。

## 3. 更新规范（何时 bump + 怎么走）

### 3.1 bump 决策

| 改动类型 | bump | 说明 |
|---|---|---|
| bug 修复 / 视觉修正 / 文案 | **patch**（+1） | 最常见；不影响玩法语义 |
| 新功能（向后兼容） | **minor**（+1） | 新场景/系统/内容；旧存档兼容 |
| 破坏性改动 | **major**（+1） | 存档/数据格式不兼容；须用户拍板 |

### 3.2 bump 流程（每次完成任务）

1. **完成清单**（见 §4）：gate 全绿 + evidence 记录 + 台账 verified + 视觉签收（用户过目）。
2. **判定 bump**：按 §3.1（本任务改动类型决定 patch/minor/major）。
3. **bump 版本**：改 `Cargo.toml` `version`（+ `AndroidManifest versionName` 同步）。
4. **出版本规划**：`docs/release.md` 发布历史追加一行（版本 + 日期 + 变更摘要 + 产物 sha256）。
5. **打包**：`build_apk.sh` / `build_wasm.ps1` 产出**带版本**产物（§2 命名）。

### 3.3 版本规划（发布记录）

`docs/release.md` §发布（publish-web 历史）每行**带版本号**：
`<x.y.z> | <日期> | <变更摘要> | 产物 sha256 | git short sha`。版本规划 = 该行 + `plan/ledger.yaml`
当前里程碑（本版本合入的任务清单）。

## 4. 完成清单（每任务完成过清单，防"没完成就交付"）

**任务完成的清单**（全部通过才算完成，硬规则 2「验收才算完成」）：

| # | 清单项 | 判据 | 载体 |
|---|---|---|---|
| 1 | 功能验收 | 任务 DoD 的验收命令**实跑输出**写 `evidence/<id>.log` | `scripts/task_close.py` |
| 2 | 回归 | `gate.ps1` 全绿（0 failure） | `evidence/gate_all_green.log` |
| 3 | 测试 | 对应 `test_manifest.json` 测试命令实测 PASS | `evidence/<test>.log` |
| 4 | 视觉签收 | UI 改动先出图 + **用户过目确认**（硬规则 9 A 纪律） | 消息留痕 + 出图 |
| 5 | 版本 | 按 §3.2 bump + release.md 版本规划 | `Cargo.toml` + `release.md` |

**机读**：清单 1/2/3 由 `check_ledger_reality`（台账↔evidence↔代码符号）+ `validate_data`
（test_manifest 命令真实性）机器兜底；清单 4/5 为流程纪律（人确认 + 台账登记）。

## 5. 工具改动面（x.y.z 读版本，禁散写）

| 工具 | 改动 |
|---|---|
| `scripts/build_apk.sh` | 读 `Cargo.toml` version → 产物 `TreasureIsl-<x.y.z>-<arch>.apk` |
| `scripts/build_wasm.ps1` | 读 `Cargo.toml` version → 产物 `treasure_bevy_wasm-<x.y.z>.wasm` |
| `android/AndroidManifest.xml` | `versionName="<x.y.z>"`（与 Cargo.toml 同步，`doccheck` 断言） |
| `docs/release.md` | 版本字段（当前版本 + 发布历史每行带版本） |

**版本漂移检测**：`scripts/validate_data.py` 增 `check_version_consistency`（Cargo.toml version ↔
AndroidManifest versionName ↔ 产物名）——构建侧工具读同一 version，产物名与版本不符即 FAIL。

## 6. 已知限制 / 说明

- **pre-1.0**：major 恒 0（`0.y.z`），minor 随新系统 bump；1.0.0 = 首版完整玩法定稿。
- **存档兼容**：破坏性改动（major bump）须出**存档迁移**（`saveload.rs` 版本字段），不静默破坏旧存档。
- **wasm 产物体积**：`treasure_bevy_wasm-<x.y.z>.wasm` 体积预算见 `11_bevy_migration.md` §3（≤8MB 目标，
  当前实测超标，见 `release.md` 已知限制，属**体积规范** S35 专项，不随版本规范处理）。
