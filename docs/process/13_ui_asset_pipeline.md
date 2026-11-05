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
| `evidence/ui_concept/raw_<name>.png` | 原始生成图（未抠图；**旧路径**，新原图落 `evidence/gen_tmp/`） | ❌（gitignore） |
| `evidence/ui_concept/<name>.png` | 设计概念图（人工参考） | ❌（gitignore） |
| `evidence/gen_tmp/raw_<name>.png` | **生图原图（未抠图）+ 临时试稿统一落点**（§2.2） | ❌（gitignore） |
| `evidence/bevy_native_<scene>.png` | 界面实拍（离屏出图，验收用） | ✅ |

- **资产路径口径 = 相对仓库根**（`assets/ui/board.png`）：
  - Bevy 侧：`AssetPlugin.file_path = 仓库根` → 资产服务器同口径解析；
  - CPU 侧：`ti_app::image_cache` → `<仓库根>/<路径>`。
  - 两侧同路径 → 双后端同画面（`09_render_framework.md` §5.1 `image` 原语行）。

### 2.1 设计稿归档规范（`evidence/ui_concept/` 命名口径，平台设计图页默认面）

`ui_concept/` 下混合了**正式设计稿**（人工参考/定案依据）与**中间产物**（未抠原图、
临时试稿）。平台设计图页默认**只展示正式稿**，中间产物隐藏（可勾选展开）。命名口径：

| 前缀 | 角色 | 设计图页默认面 |
|---|---|---|
| `raw_<name>.png` | 原始生成图（**未抠图**；§2.2 起新原图落 `evidence/gen_tmp/`，旧路径存量兼容） | 隐藏（中间产物） |
| `tmp_<name>.png` | 临时试稿 / 调试产物（无归档价值） | 隐藏（中间产物） |
| 其余 `<name>.png` | **正式设计稿**（概念图 / 变体 / 定案依据） | 展示 |

- **写图纪律（所有会话 / AI 生图 / 工具链遵守）**：
  - 未抠图原图**必须**落 `evidence/gen_tmp/raw_<name>.png`（`ui_assets.py build` 自动生成，勿手改前缀）；
  - 临时试稿 / 一次性调试图**必须**落 `evidence/gen_tmp/tmp_<name>.png`（**不带** `raw_`/`tmp_` 前缀
    的正式名混不进 `ui_concept/` 顶层 = 不会被设计图页当正式稿展示）；
  - 正式设计稿（**不带** `raw_` / `tmp_` 前缀：`main_map_v5_b.png` / `index_compass_concept.png` /
    `isle_v2_concept_a.png`…）落 `evidence/ui_concept/` 顶层；定案 / 对比 / 验收留痕用正式名。

### 2.2 生图临时图统一落点（`evidence/gen_tmp/`，2026-11-05 用户拍板）

**生图原图（未抠图）+ 临时试稿全部固定落 `evidence/gen_tmp/`**（gitignore 整目录），
不再散进 `evidence/ui_concept/`。理由与口径：

- **清理面 = 删整个目录**：`rm -rf evidence/gen_tmp/` 一条命令清空全部临时图，
  目录里**只允许**临时图（`raw_*` 原图 + `tmp_*` 试稿），**禁止**放正式设计稿 /
  合成稿 / 引擎渲染图（那些仍落 `ui_concept/` 顶层，§2.1/§2.1a 口径不变）；
- 工具链落点已改：`ui_assets.py`（build/add 写 `gen_tmp/raw_<name>.png`）+
  `ui_asset_batch.py`（generate/install 写/读同一路径）；`rekey`/`install` 读不到
  新目录时**回退读旧路径** `ui_concept/raw_<name>.png`（存量兼容，旧目录可整段删除，
  旧原图不再回写）；
- `ui_thumb.py` **不扫** `gen_tmp/`（临时图不出缩略侧车，不进平台设计图页）；
  平台 `GET /api/evidence?sub=ui_concept` 同样只枚举 `ui_concept/`，临时图天然不可见；
- 文档/evidence 里引用的旧路径（`ui_concept/raw_*.png`）= 存量留痕，不再新增引用；
  新建临时图一律 `gen_tmp/`。
- **机器兜底**：平台 `GET /api/evidence?sub=ui_concept` 已**滤除** `.thumb.png` 侧车
  （生成物非设计稿）；设计图页前端按 `raw_`/`tmp_` 前缀默认隐藏中间产物
  （`web/index.html` 设计图页 `#design-mid` 开关，口径 = 本节表；改命名规则须同步本节
  + 前端 `isMidProduct()`）。
- **压缩预览面**（2026-10-30）：正式稿 + 中间产物都可用 `scripts/ui_thumb.py` 生成
  `.thumb.png` 侧车（480px PNG-8，~53KB vs 原图 4-6MB），平台设计图卡读侧车、
  lightbox 读原图；侧车 + `.thumb_meta/` 指纹目录均为**生成物，gitignore，不入清单/不归档**。

### 2.1a 迭看图 / 引擎渲染图归档（2026-10-31 用户「图片乱放找不到」→ 定案）

迭看 / 引擎渲染 / 设计稿**必须**统一落 `evidence/ui_concept/` **顶层**（不带 `raw_`/`tmp_` 前缀 =
平台设计图页默认展示），**禁止**散落子目录（`ui_concept/<theme>/`）或新建顶层目录（如 `vs_v2_render/`）。
命名口径（主题前缀 + 阶段/角色 + `_engine` 后缀区分渲染方式）：

| 图类 | 命名 | 例 |
|---|---|---|
| 合成稿 / 概念稿 | `<theme>_<variant>.png` | `main_map_v6_composite.png` / `vs_v2_clash.png` |
| 引擎渲染迭看图 | `<theme>_<variant>_engine.png` | `main_map_v6_engine.png` / `vs_v2_clash_engine.png` |
| AI 元素源（中间产物） | `raw_<theme>_<elem>.png` | `raw_main_map_v6_island.png` |
| 临时试稿 | `tmp_<name>.png` | `tmp_vs_v2_bg.png` |

- **写图纪律**：任何迭看图/引擎渲染图**写前**先查本节命名表；写后**必须**跑
  `python3 scripts/ui_thumb.py` 生成侧车（平台卡/lightbox 才可见）；gitignore 的 `ui_concept/`
  子目录（`main_map_v6/elements/`、`vs_anim_v2/elements/`）只作 **AI 元素源**（`raw_`/中间产物），
  **不得**放正式迭看图；临时试稿 / 未抠原图落 `evidence/gen_tmp/`（§2.2，不进平台面）。
- **呈现纪律（2026-10-31 用户「磁盘物理路径，每次按这个给出」）**：迭看图给用户看时，
  **必须**给出**磁盘物理绝对路径**（`<repo>/evidence/ui_concept/<name>.png`），
  且**在对话中直接显示图**（不只给路径）——用户可能从 git 视图找（`ui_concept/` gitignore
  = git 看不到），物理路径 + 对话内显示双通道，防「找不到」。
- **工具链一致性**：`ui_thumb.py` 只扫顶层 `*.png`，子目录元素源不生成侧车（不占清单）；
  引擎渲染迭看图放子目录 = 平台看不到（2026-10-31 实测「生图在平台看不到」根因）。
- **机器兜底**：平台 `GET /api/evidence?sub=ui_concept` 只枚举顶层 → 子目录稿天然不可见
  （若把迭看图误放子目录，用户迭看必扑空）。

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

### 4.0 压缩预览侧车（设计图页大图传输优化，2026-10-30）

`ui_concept/` 设计稿单张 4-6 MB（146 张共 353 MB），平台设计图页卡/lightbox 走白名单
直读全量 PNG = 预览卡传输重。**侧车方案**：同名 `<name>.thumb.png`（480px 长边、
PNG-8 量化，~53KB/张，总 7.6 MB = 原图 2%），卡网格只读侧车、lightbox 放大读原图；
侧车 + `.thumb_meta/` 指纹目录 = **生成物，gitignore（`/evidence/ui_concept/` 已整体忽略），
不入 `ui_manifest` 清单、不归档**。

```bash
python scripts/ui_thumb.py            # 全量生成/刷新侧车（幂等；源指纹未变则 SKIP）
python scripts/ui_thumb.py --force    # 无视指纹强制重生成
python scripts/ui_thumb.py --check    # 只读对账：缺侧车列表 + 非 0 退出（工具契约面）
```

- **平台前端契约**（`web/index.html` 设计图页）：卡 `<img src>` 默认读
  `f.replace(/\.png$/, '.thumb.png')`，`onerror` 回退原图（侧车缺失不崩）；
  lightbox 放大读原图（清晰面不降）。`/api/evidence?sub=ui_concept` 清单**已滤除**
  `.thumb.png`（生成物非设计稿，不占设计图计数）。
- **缓存协商**（`routes.rs` 白名单直读面）：响应带 `ETag`（FNV-1a 内容指纹 + 长度）
  + `Cache-Control: max-age=604800`；浏览器二次请求带 `If-None-Match` 命中 = 304 空 body
  （改文件 ⇒ 指纹变 ⇒ 自动失效）。网页静态壳（`index.html`）仍 `no-store`（改即生效）。

### 4.1 补缺失资产

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
① 出设计图/概念图（Agnes 生图 → evidence/ui_concept/ 参考；原图/临时图 → evidence/gen_tmp/ §2.2）
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
