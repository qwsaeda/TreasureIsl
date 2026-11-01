# 18 · 资产与内容数据管线定案（2026-10-27 用户拍板）

> **本文档 = 「资产/内容怎么登记、怎么加载、版面坐标写在哪」的单一事实源。**
> 触发：用户 2026-10-27「没有统一资产管理 / 资源加载标准？地图随便换？」→ 澄清「scenes.json 手写坐标不行，行业怎么做、为什么有地图编辑器」。
> 与 `17_layout_engine.md`（B1 布局）互锁：17 管「版面写在哪（bevy_ui）」，本文管「资产/内容数据怎么登记与加载」。

## 1. 行业三档（版面/关卡数据的表示方式）

| 档 | 表示 | 谁在用 | 痛点 |
|---|---|---|---|
| ① 绝对坐标手写 | 逐元素 pixel 坐标进数据文件 | 早期/原型 | 无视觉反馈、改一处动全身、易错（本项目 S29.2 曾 5672 行 churn） |
| ② 意图 + 自动布局引擎 | 声明间距/对齐/占比，引擎算像素 + resize 响应 | CSS/flexbox、Figma auto-layout、Android ConstraintLayout、Godot/Bevy taffy | 优点最多：不写像素、resize 免费、引擎强制约束 |
| ③ 参数化/程序化生成 | 几何从参数算（环心+R+N格+分区） | 大富翁/Tile/关卡生成 | 数据只写「有什么」，不写「放哪」 |

**总纲（用户选定 ②，优点最多）：不手写像素。声明意图/数据，引擎算坐标。**
② 管 UI chrome，③ 管地图——哲学同一个。

> 「为什么行业有地图编辑器」：关卡/版面是**迭代式视觉工作**，人改时必须「所见即所得」。
> 手写 JSON 坐标把「看」和「改」割裂（无反馈、易错位、改一处动全身）；编辑器 = 把视觉反馈做进编辑流程。
> 本项目不另造编辑器，而是用 ②③ 让数据「小、可读、引擎算位置」，天然规避手写坐标的坑。

## 2. 本项目终态（按区域分档）

| 区域 | 档位 | 落点（WBS） |
|---|---|---|
| **UI chrome**（7 场景版面/HUD/面板） | ② 自动布局（`bevy_ui` taffy） | **S29.3**（坐标出 scenes.json 绝对像素 → 意图约束；resize 真重排） |
| **地图/赛道**（大富翁环、岛、船落点） | ③ 参数化生成 | **S27.4**（环心+R+N格+分区 → 生成器算几何，不手写 31 格坐标） |
| **scenes.json 角色** | 退化成「**元素清单**（有什么 / 什么资产 / 什么 `pick_key` / 什么意图约束）」，不再手写 1080 绝对像素 | S29.3 起 |

**「地图随便换」= 数据驱动可行**：地图 = `map_id` → `terrain.json`（格码+图例）+ `board.json`（赛道格）。加图 = 加 JSON 数据，零代码改动（`board().map(id)`/`terrain().code_at(id,…)`）。这条**已成立**，本文只补「几何由参数算，不手写坐标」。

## 3. 资产登记标准（统一资产管理，gate 机器兜底）

**登记源（按类型分，无重复登记）：**
| 类型 | 登记源 | 加载标准 | 磁盘目录 |
|---|---|---|---|
| UI 图像 | `assets/ui_manifest.json`（`assets[].file`，97 图 + 16 role + 生成参数；以清单为准） | Bevy `AssetServer().load()`（async handle，相对 `assets/` 根） | `assets/ui/*.png` |
| 字体 | `assets/registry.json` `fonts`（1 TTF） | Bevy GPU 图集 S18.3 / native `parse_font_bytes` | `assets/fonts/*` |
| 音频 | `assets/registry.json` `audio`（**当前空 = 发货无声**） | legacy macroquad `quad-snd`（Bevy 壳双渲染期关，S18.8 切 `bevy_audio` 前 0 轨） | `assets/audio/*`（不存在） |

**gate 兜底（`scripts/validate_data.py`，gate 第 10a 项 `validate_data`）：**
- `check_asset_registry`：**全类型 disk↔registry 双向同步** —— 登记的须在盘上（悬挂引用 = FAIL）+ 盘上的须登记（漏登记 = FAIL）。
- `check_ui_icons`（既有 V3）：`ui_icons.json` 引用的白蒙版须在位。
- 数据表：`assets/data/*.json` 经 `ti_core::data` typed loader（`&'static` 缓存 + `LoadError` 快速失败，无 unwrap）。

**纪律：新增资产 = 登记到对应 registry（gate 双向同步兜底）+ 只按路径引用（`check_hardcode` 拦资产路径硬编码）。**

## 3a. AI 生图创作范围（2026-10-31 用户「划定范围 / 固化生图创作范围」→ 定案）

**总纲：AI 生图只做「元素级创作 + 参考概念」，不直接出「终态界面 / 终码资产」。**
创作范围按「能生成 / 不能生成 / 风格统一 / 禁区」四栏固化，防用户迭看反复返工
（EXP-141/142/143：写实照片背景、动漫角色、乱放图、占屏太小——全是范围没限定）。

### 能生成（AI 元素级）

| 用途 | 例子 | 工具 |
|---|---|---|
| 界面**元素**（白底/键控，引擎/PIL 组合） | 木片格子、岛、舵轮、罗盘、船、生物、人物立绘、状态框 | `agnes_gen.py` + `key_v6_elements.py` |
| 全景**背景**（满屏，不键控） | 海洋、甲板、战斗类型背景（siege/dungeon/boss） | `agnes_gen.py` |
| 概念稿（方向参考） | 变体 A/B/C、布局稿 | `agnes_gen.py` |

### 不能生成（必须程序/数据/引擎做）

- **精确布局**（环心/半径/格径/坐标）= `scripts/compose_*.py` + 引擎 `chrome::ring_geom` 算，
  不靠 AI 生图定位（AI 构图不可控 ⇒ 判据不可机检）。
- **数值/文案**（兵力、战报、嘲讽）= `balance.json` + `i18n` 数据，非 AI 自由发挥。
- **运行终态界面** = 引擎真实渲染（`--shot`），AI 概念稿只作方向参考（用户「AI 概念稿不算数」）。

### 风格统一（用户多次拍板，硬性）

1. **游戏统一风 = 羊皮纸手绘 + 黄铜/暖木**（S27.7 裁定「弃写实照片 → 游戏统一风」）。
   **禁止**写实照片背景 / 纯 3D 渲染 / 冷灰蓝无色调。
2. **动漫适中**（用户「动漫化不要太成人化」）：少年漫大眼，非成人写实、非幼稚过头。
3. **同主题背景连贯**：4 帧同主题（羊皮底 + 黄铜罗盘边框），result 可加氛围但**保留底框**。
4. **角色一致性**：一帧定角色 → 其余帧图生图锚定（EXP-141 ②）；装扮强约束（三角帽/白衬衫/破披风）。

### 禁区（禁止）

- 禁写实照片风（违反风格统一，S27.7 已裁定，VS v2 首版踩过）。
- 禁「AI 元素硬编码路径在 Rust」（CS-1 资产单源：路径走 `chrome::*` 常量 + `ui_manifest` 登记）。
- 禁迭看图落子目录 / 新开顶层目录（§2.1a 归档：顶层 + 侧车）。
- 禁 AI 生成后不键控直接 blit（引擎不抠白 ⇒ `key_v6_elements.py` 必跑）。

### 生图 → 入库流程（每条固定）

`agnes_gen.py 生成（白底）` → `key_v6_elements.py 键控透明` → `ui_asset_batch.py register`
（登记 `ui_manifest.json`）→ `ui_thumb.py` 侧车 → 迭看（用户确认留痕）→ 落终码。
**顺序不可跳**（跳键控 = 白块；跳登记 = 漏登记 FAIL；跳侧车 = 平台看不到）。

## 4. scenes.json 序列化纪律（②止血，防整文件 churn）

`scenes.json` = 手工维护的版面数据源（非派生物，S21.5/S21.6/S26 直接改），2888 行。S29.2 一次性脚本误用 `indent=2` 重序列化 → 5672 行 churn（已修回 99 行最小 diff）。
- **规范形 = `json.dumps(obj, ensure_ascii=False, indent=1)` 逐字节**。gate `check_scenes_canonical` 守此不变量。
- **改数据纪律**：读原对象 → 定点回注（只动要改的字段）→ `json.dumps(indent=1)` 写出。**禁止 LLM/脚本整文件重写换缩进**（呼应 EXP-124 一次性脚本纪律）。
- 仅锁 `scenes.json`（真痛点）；其他数据表（balance/theme/…）暂不锁规范形，防误伤，后续可扩。

## 5. 音频口径（发货版）

双渲染期 Bevy 发货壳（桌面/浏览器）**关音频**（`treasure_bevy.rs` `set_audio(0,0)`），`registry.audio` 空 = 无音频资产。
- **当前口径：发货桌面/浏览器无声；音频无文件（legacy macroquad 壳亦无音频资产在盘）。**
- 要出声音 = S18.8 切 `bevy_audio` + 生成音频资产 + 登记 `registry.audio`。未落地前，语音/SFX 需求按「移动端 macroquad 壳」口径。

## 6. 执行链（WBS 挂接）

| 项 | 挂接 | 状态 |
|---|---|---|
| ②止血 gate（scenes 规范形 + 资产双向同步） | 本文 §3/§4，`validate_data.py` `check_scenes_canonical`/`check_asset_registry`（gate 10a） | 已落 |
| ①资产 registry（`assets/registry.json` 字体/音频） | 本文 §3 | 已落 |
| ②根治：UI 坐标出 scenes.json 绝对像素 → bevy_ui 意图 | **S29.3**（`17_layout_engine.md`） | 已落（S29.3 全场景 UI + 命中 + 真重排，见 `19_bevy_ui.md`） |
| ③地图参数化（环格几何生成器） | **S27.4**（前置 `board.json` 分区数据 S27.3） | 已落（环形棋盘 `chrome::ring_geom` + 地图打包工具 `scripts/map_pack.py`，本文 §7） |
| ⑤音频（bevy_audio + 音频资产） | **S18.8** | 已落（S18.8 bevy_audio 切换；音频资产 = 发货无声口径 `registry.audio` 空，S30.0） |

## 7. 地图打包标准 → 见 21 号规范

**总纲「不手写像素：声明意图/数据，引擎算坐标」落到地图**（一张图 = 一个可分发的自包含包、
换图 = 装包 + 切包）的**格式 / 清单字段 / 版本兼容 / 完整性 / 运行期加载契约 / 失败回退**，
唯一事实源 = [`21_map_package_spec.md`](21_map_package_spec.md)；本文只保留总纲与三档定位。

| 角色 | 落点 |
|---|---|
| 地图包**格式与流程**规范 | [`21_map_package_spec.md`](21_map_package_spec.md)（§1~§12） |
| 打包工具 | `scripts/map_pack.py`（`pack`/`verify`/`install`/`uninstall`/`list`/`check`；台账见 `14_tooling_catalog.md` §3.2） |
| 已装包 / 可分发件 | `assets/maps/<id>/`（已装态，运行期直读）/ `dist/maps/<id>.map-pack/`（可分发件） |

**几何数据驱动**（不变）：环形棋盘几何 = `main_map.ring.*`（`extra_scalar` 点号键）；
呈现层 `chrome::ring_geom` 读它，缺键才回退常量 ⇒ 改环几何只改数据，不动 Rust。
卡面/中央岛资产按命名约定 `isle_<isle_id>.png` / `island_home_<map_id>.png`（id 来自数据）
⇒ 加岛型 / 加图不改 Rust。

