# 24 · APK/产物体积标准（APK Size Standard）

> 2026-11-01 用户拍板「APK 体积大，分析 → 1-3 全做 + 形成规范/标准/文档 + 纳入平台功能」。
> 单一事实源：本文（判据 + 构建口径）；机读工具 = `scripts/apk_audit.py`；平台页 = ti-console `/api/apk-size`。
> 路由表 S35 条目；台账 = `plan/ledger.yaml` S35.0。

## 1. 背景：184MB APK 实测构成（2026-11-01 取证）

`dist/TreasureIsl-arm64.apk` 183.8MB（压缩后口径），三大来源：

| 来源 | 压缩后 | 根因 |
|---|---|---|
| `lib/arm64-v8a/.so`（Stored 0% 压缩） | 53.4 MB | `build_apk.sh` 显式 `ZIP_STORED`（原为 MuMu 转译加载 hack，后口径废弃） |
| `assets/ui/vs_v2` + `assets/ui/v6`（2K×4K 背景） | 91.4 MB | 全量 `cp -r assets/ui/.` 塞包；PNG 二次 deflate 压缩率 0% |
| `assets/ui` 其余（未引用旧批次） | 31.0 MB | 同上，未引用图 8 个 ≈ 5.3MB 原样 + 历史批次冗余 |

**事实澄清（防误判）**：v6 13 张 + vs_v2 9 张**全部**有代码字面量引用（`chrome.rs` V6_*/V2_*
常量 → main_map v6 / vs v2 当前默认路径），**不可砍**；可砍的只有「磁盘存在但运行时 0 引用」
的 8 张旧批次图（`bg_night_sea_top`/`map_harbour`/`map_reef`/`board`/`map_open_sea`/
`compass_rose`/`hud_plaque`/`btn_spin`）。`.so` 54MB 本体 = Bevy 0.20 引擎能力代价（12_bevy_migration
§3 已定案 ≤8MB wasm 预算 = 历史口径，本项不翻案；.so 压缩后 20.4MB 为 Android 侧实测定态）。

## 2. 标准（判据，机器可检）

> 压缩率口径 = `compress_size / file_size`（deflate 后 / 原始）。**Stored = 100%；deflate-9 对
> `.so` 实测 ≈38%**（二进制代码压缩率天然低，与 PNG 0% 不同）。判据阈值取 **0.90 安全分界**
> （压缩后必 <0.5，未压缩必 =1.0）。

| # | 判据 | 检查点 | 工具 |
|---|---|---|---|
| ① | APK 内 `lib/<abi>/libandroid_activity.so` **必须 ZIP_DEFLATED**，压缩率 < 90%（Stored = 100% 即 FAIL） | `build_apk.sh` [4/5] 段 + 平台页 | `apk_audit.py --apk` exit 3 |
| ② | APK 内 `assets/ui/*.png` **全在引擎引用集内**（未引用图 0 进包 = 漂移即 FAIL） | 同上 | `apk_audit.py --apk` exit 4 |
| ③ | 构建侧（`build_apk.sh` / `build_wasm.ps1` 4f 段）打包 = 引用集，**禁止全量 `cp -r assets/ui`**；工具缺位回退全量时必须打 WARN（口径漂移不静默） | 代码 review + gate | grep `apk_audit.py --refset` 接线点 |
| ④ | 引用集口径变更（新引擎引用 / 新动态前缀 / registry audio 增减）**必须先改 `apk_audit.py` 再改构建**；两边独立收口 | 流程 | 本文 §3 登记表 |

## 3. 引用集（单一事实源 = `scripts/apk_audit.py --refset`）

### 3.1 组成

| 来源 | 口径 | 说明 |
|---|---|---|
| 代码字面量 | `crates/ti-{engine,app}` `.rs` 内 `"assets/...png\|jpg\|json\|wav\|fnt\|dat"` 字符串 | 含 `chrome.rs` V6_*/V2_* 批次常量（资产路径 CS-1 单源） |
| 动态 `format!` 前缀 | `icon_*` / `isle_*` / `island_home_*` / `role_*` **全带** | 运行时按名 blit、缺图走代码矢量/通用回退 ⇒ 砍任一 = 视觉回归（24 号 §1 事实澄清） |
| 地图包岛 kind | `assets/maps/**/*.json` + `assets/data/**/*.json` 的 `isle_*`/`island_*` 字面量 | board 10 类 + 专属岛 |
| registry audio | `assets/registry.json` `audio[].path`（4 项恒带） | S34 原版音频，缺 = 静默无 BGM |
| legacy | `assets/font.dat`（GoldLand.Fnt 老字体表） | 缺 = 回退系统字体，0 影响，仍带 |

### 3.2 变更登记（判据④）

| 日期 | 变更 | 原因 |
|---|---|---|
| 2026-11-01 | 初版：8 项未引用图移出包（5.3MB）；.so 压缩（53→20MB） | S35.0 |

### 3.3 已知缺口（引用集声明但磁盘无文件，构建侧 `is_file` 跳过 = 0 影响，出图时补）

- `assets/ui/island_home_m04.png`：m04 manifest `central_asset` 已声明，资产未生成（运行时缺图
  回退链 ⇒ island_home_m01 金边岛）；
- `assets/ui/icon.png`：引擎代码直引（`AAssetManager` reader 剥前缀注释提及），实际走
  `icon_{name}` 动态前缀 + 代码矢量回退；
- `assets/font.dat`：legacy GoldLand 老字体表（wasm 侧 `data/` 有副本），Android 缺 = 系统字体回退。

## 4. 平台功能（ti-console）

`/api/apk-size` = 调 `apk_audit.py --apk <最新 dist/TreasureIsl-arm64.apk>` 只读适配（**平台不执行构建**，
22 号 §1 边界：只做只读体检与预览）：

- 返回 JSON：`total_compressed` / `by_dir`（目录聚合 压缩后/原始/文件数）/ `lib`（压缩率 + method）
  + `verdict`（PASS/FAIL + 判据号）；
- 平台「APK 体积」页（左菜单第 8 页）= 构成表 + 未引用漂移清单 + 判据状态灯；
- 数据源缺失（dist/ 无 APK）= 页面显示「未构建」，不报错（只读降级口径，同 22 号 §8 快照缺省）。

## 5. 已知限制（不静默绕过）

- **web 侧 43MB wasm** 与 **APK 侧 54MB .so** 同源 = Bevy 0.20 引擎代价（12_bevy_migration §3）；
  本项只治「打包口径」（压缩 + 引用集），**不治引擎体积**；若立项砍引擎特性
  （关 `complex_script_segmentation` 省 12–15MB / wgpu 后端裁剪），另立项专项，
  不在本标准范围（S26.1 CJK 断行依赖 = 回归风险，需用户拍板）。
- x86_64 产物（MuMu）同口径复用本判据（`build_apk.sh x86_64`）。
- `zipalign` 口径 = 4 字节（.so 压缩后安装器统一解压，`-p` unaligned 模式废弃；
  `extractNativeLibs=true` 保留 = MuMu 转译加载兼容，EXP-146 坑②历史不变）。
