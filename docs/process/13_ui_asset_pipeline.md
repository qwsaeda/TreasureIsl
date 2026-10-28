# 13 · UI 资产管线（清单驱动 · S18.8b 起）

> **状态**：S18.8b 落地（随 `image` 原语 + Bevy 发货壳）。
> **定位**：界面美术资产的**生产流水线**规格 —— 生图 → 抠透明 → 规范化 → 入库 → 门禁校验。
> 界面**数据**（`assets/data/scenes.json`）只引用资产路径；资产**清单**（`assets/ui_manifest.json`）
> 是资产的单一事实源。改界面 = 改 JSON；加资产 = 清单加一条 + 跑管线。

## 1. 为什么要有这条管线

S18.8a 之前，界面全是代码绘制的几何原语（`rect`/`icon`/`grad`…）→ 观感"半成品"
（英雄立绘是黑块、图标是几何占位、提示行是几条横线）。要贴近设计图，必须有**位图资产**，
而位图资产的生产（AI 生图 + 抠透明 + 规范化 + 命名 + 入库）此前是一次性手工操作 → 不可复用。

本管线把这套操作**固化成脚本 + 清单**，任何界面、任何后续项目都能直接复用。

## 2. 目录与约定

| 路径 | 角色 | 入库 |
|---|---|---|
| `assets/ui_manifest.json` | **资产清单**（单一事实源）：角色预设 + 资产登记 | ✅ |
| `assets/ui/<name>.png` | 运行时资产（RGBA8，已抠透明/规范化） | ✅ |
| `evidence/ui_concept/raw_<name>.png` | 原始生成图（未抠图） | ❌（gitignore） |
| `evidence/ui_concept/<name>.png` | 设计概念图（人工参考） | ❌（gitignore） |
| `evidence/bevy_native_<scene>.png` | 界面实拍（离屏出图，验收用） | ✅ |

- **资产路径口径 = 相对仓库根**（`assets/ui/board.png`）：
  - Bevy 侧：`AssetPlugin.file_path = 仓库根` → 资产服务器同口径解析；
  - CPU 侧：`ti_app::image_cache` → `<仓库根>/<路径>`。
  - 两侧同路径 → 双后端同画面（`09_render_framework.md` §5.1 `image` 原语行）。

## 3. 角色预设（role）

资产规格预设 = **生图比例 + 目标尺寸 + 抠图模式 + 提示词模板**。新增角色只加一条预设。

| role | 比例 | 目标尺寸 | 抠图 | 用途 |
|---|---|---|---|---|
| `background` | 9:16 | 1080×1920 | 无 | 界面背景（夜空/海面…） |
| `medallion` | 1:1 | 512×512 | **亮度**（纯黑底） | 金色圆形纹章（菜单图标） |
| `board` | 21:9 | 1024×439 | **品红** | 雕花木牌（标题面板） |
| `portrait` | 2:3 | 512×768 | **亮度** | 角色立绘 |

抠图算法（`scripts/ui_assets.py`）：

- **亮度（luma）**：提示词要求「纯黑底 + 金色线稿」→ `alpha = 亮度`（阈值 58/125 两点线性），
  RGB 按 alpha **预乘**（消掉暗底残留的蓝紫）。适合**金色线稿**。
- **品红（magenta）**：提示词要求「纯品红底」→ 品红度 `min(r,b)-g` 判定，边缘软过渡，
  品红像素置全透明。适合**实心物件**（木牌）。
- **品红去紫边（magenta_clean）**：`magenta` 的加固版（`key_magenta` 行为不变，既有资产零影响）。
  **何时用**：生图模型把浅水**真的渐变到品红底**时（实测渐隐带宽 8~11px @2048），
  `magenta` 会在物体外缘留一圈半透明像素，其 RGB 混着底色 ⇒ 叠在浅色海图上 = **可见紫轮廓**。
  三步消掉：① 收紧 alpha 阈值（`hi=32 / lo=8`，比 `magenta` 的 60/20 更狠，边缘收 1~2px）；
  ② **去色溢 despill**（半透明像素按品红增量等量扣 r/b，保留真实色相）；
  ③ 预乘后缩放（避免 RGBA 直缩放串色）。
  > ⚠️ **不要按 alpha 反预乘**：品红度启发式的 alpha ≠ 真实覆盖率，反预乘会把品红**放大**成亮圈。
  > 现役用法：`island_home` 角色（中央据点岛，浮在海图上）用 `magenta_clean`；
  > `board` / `plaque` 等不贴浅底的角色仍用 `magenta`。

> 提示词里放 `{what}` 占位符 = 图案描述（如 `a ship's helm wheel`），
> 由 `add --what` / 清单 `params` 填充。

## 4. 命令

```sh
# 补齐清单里缺失的资产（幂等；已存在则 SKIP）
python scripts/ui_assets.py build
python scripts/ui_assets.py build --force          # 全部重生成
python scripts/ui_assets.py build --only icon_compass

# 登记新资产（写清单 + 立刻生成）
python scripts/ui_assets.py add --name icon_lamp --role medallion --what "a ship lantern"

# 重抠图 / 重规范化（改抠图参数或尺寸后）
python scripts/ui_assets.py rekey --name board
python scripts/ui_assets.py normalize --name board

# 校验（清单 ↔ 文件 ↔ scenes.json 引用 三方一致）
python scripts/ui_assets.py verify

# 界面实拍闭环（改完 scenes.json → 出图 → 看效果）
powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1 index
powershell -ExecutionPolicy Bypass -File scripts/ui_preview.ps1        # 全部 7 场景
```

环境变量：`AGNES_API_KEY`（Agnes API 密钥；仅 `build`/`add` 需要）。

## 5. 门禁接线

| 断言 | 位置 | 拦什么 |
|---|---|---|
| `check_data_driven` | `scripts/check_data_driven.py` | ① `image` 原语的资产**文件存在**；② 资产**已登记**在清单；③ 原语闭集 = 7 种（多一种即 FAIL） |
| `check_primitives_data` | `crates/ti-tools/src/bin/doccheck2.rs` | 同上（Rust 侧双保险） |
| `ui_assets verify` | `scripts/ui_assets.py` | 清单 ↔ 文件 ↔ `scenes.json` 引用三方一致 + **孤儿文件**（`assets/ui/` 下未登记的文件） |

> **为什么要「登记」这道关**：资产文件存在 ≠ 界面会用到它。未登记的文件会堆积成
> 「没人知道它是什么、也不确定能不能删」的黑洞（与 EXP-101/108 同类的静默漂移）。

## 6. 迭代闭环（一个界面怎么做完）

```
① 出设计图/概念图（Agnes 生图 → evidence/ui_concept/ 参考）
② 定资产清单（背景/木牌/纹章/立绘…）→ python scripts/ui_assets.py add ...
③ 改界面数据（assets/data/scenes.json 的 primitives：image 原语 + 文本/矩形）
④ 出图看效果 → python scripts/ui_preview.ps1 <scene>   （evidence/ui_preview/ 缩略图 + 非黑像素统计）
⑤ 迭代 ③~④ 直到对齐设计图
⑥ 门禁全绿 + 用户过目签字
```

**注意**：`ui_preview.ps1` 报「非黑像素 = 0」= 空帧（渲染/资产没生效），
这是最常见的失败信号；出图不稳定时先看 `scripts/ui_preview.ps1` 的统计，别靠肉眼猜。

## 7. 已知边界

- **`--shot` 截图竞态**（S18.8b 修）：目标纹理尚未渲染出内容就被截下 → 读回全透明。
  现为「空帧允许重截」；仍偶发时以 `ui_preview.ps1` 的多次统计为准。
- **贴图是异步加载**：Bevy 资产服务器读盘在后台线程 → 截图/首帧可能还没贴图
  （截图帧阈值已提到 40 帧 + 每帧 4ms sleep）。
- **CPU 路径性能**：全屏背景贴图 = 每帧 2M 像素 alpha 混合（headless/诊断路径可接受，
  交付路径走 Bevy GPU）。
- **抠图不是分割**：亮度/品红键对「描边主体 + 纯底」有效；模型给了复杂背景时需重生成
  （换更严格的提示词）或改用其它抠图模式（新增 role 预设）。
