# 14 — HD 资产增强管线（S7）

> 单一事实源：本文件是「分类 → 批量超分/矢量化 → 关键图 AI 重绘 → 动画时序稳定 → 多分辨率打包」
> 的完整实现规格。关联：`docs/plan-s7.md`（任务表）、`docs/modernization.md`（总纲）、
> `docs/impl/07_ui_controls.md`（渲染管线）、`docs/impl/13_appendix.md`（资源清单）。

---

## 1. 目标与硬约束

### 1.1 目标

把原版 176×220 RGB565 索引色 PCX 美术升级为**全 HD 真彩色**资源管线，
支持 1x/2x/4x 多分辨率导出，三端（native/wasm/android）统一消费。

### 1.2 硬约束

| 约束 | 来源 | 影响 |
|---|---|---|
| P4-1 逐像素对齐 | `docs/parity.md` P4 | 替换原 PCX 管线则需重写 `render_parity` 基准图；并行则新增 `render_parity_hd` |
| 首屏 ≤ 490 KB | `docs/plan-s4.md` S4.3 | HD 真彩 PNG 远超预算，需重列分包策略（§6 体积预算） |
| `.Map` f1 寻址 | F-311 / F-204 | 精灵编号不可变，HD 资源沿用 `f1 → NN.pcx` 命名映射 |
| RGB565 帧缓冲 | `crates/ti-platform/src/gfx.rs` | 全 HD 需 `Framebuf` 升 RGBA8，波及 `sprite_atlas`/`terrain_atlas`/`menu_shell` 全部 blit 路径 |
| 零依赖原则 | `docs/style.md` §5.9 | Rust 侧不引新依赖；Python 侧允许 Pillow；AI 工具走命令行 |

### 1.3 用户决策（2026-10-10）

1. 目标定位 = **完整 HD 管线（新建 S7 阶段）**；
2. 风格 = **全 HD 真彩色（放弃像素风）**；
3. P4-1 = **两路都写**（保留原 PCX 管线并行 + 预留替换路径），评审时定。

---

## 2. 资源分类（按处理策略）

原版美术资源共 **5 类**，来自 7 个 pak/lpk 容器（305 条容器条目，F-207；
`ref/asset_classification.json` 305 条 / `assets_hd/_stats.json` 335 条处理 = 305 容器条目 + 30 九宫格派生（16 条跳过））：

| 类 | 容器 | 数量 | 特征 | 处理策略 | 工具 |
|---|---|---|---|---|---|
| **A. UI 控件/图标/按钮** | `ui.pak` | ~30（button_* 29 + switchpage + 对话框背景） | 平涂索引色，几何清晰 | 矢量化：Inkscape 描摹 → SVG 基准 + PNG 九宫格 | Inkscape / Vector Magic |
| **B. 角色立绘/头像** | `ui.pak` + `glpics.pak` | ~11（face_* 5 + Player_* 5 + Target_head/knife） | 卡通角色中等细节 | AI 超分：Real-ESRGAN x4 anime；可选 SD 重绘 | Real-ESRGAN / ComfyUI |
| **C. 兵种/船只/旗帜/地标** | `glpics.pak`（实测全容器 C_pixel = 155 条，含 ui/glpics/glmap/cards） | ~77（仅 `glpics.pak` 侧：88 条去 B 类 11 个） | 小尺寸像素精灵硬边 | 最近邻 2x 整数倍（保硬边），不做 AI 平滑 | Pillow NEAREST |
| **D. 地形瓦片** | `glmap_m01/m02.pak` | ~84（47+37=84 条目，去重 46+32=78 唯一，F-207） | 大画布 63×31~127×105（S6.3 实测） | Lanczos 2x + 摄像机步进同步 2x | Pillow LANCZOS |
| **E. 多帧动画** | `ui.pak` | 4 精灵 × 5 帧（14/31/32/37.pcx，num=5，S6.4 实测） | 帧纵向堆叠高 199 | 序列当短视频 → 时序超分 → 回切帧（防闪烁） | Topaz Video AI / RIFE |

**不处理**：`GoldLand.Fnt`（点阵字体按整数缩放即可，HD 走 TTF 矢量字体见
`docs/modernization.md` §4）、`cards.lpk`/`goods.lpk`（数据文件）、`music.lpk`（音频，
S1.7b 已转 OGG）、`Thumbs.db`（OLE 文档）。

### 2.1 分类判定依据

- **A 类**：`docs/impl/07_ui_controls.md` §7.1 列 29 个 `button_*.pcx` + `switchpage.pcx`，
  几何清晰的 UI 控件。
- **B 类**：`docs/impl/07_ui_controls.md` §7.2 列 6 角色头像 + 5 `Player_*.pcx`，
  SelectRole 场景卡通立绘。
- **C 类**：`glpics.pak` 88 条（F-207）中除去 B 类 11 个，余 77 个为
  troop/ship/flag/landmark 系列（`docs/design/ui_controls.md` §3.1）；
  实测全容器（ui+glpics+glmap+cards）`C_pixel` 共 155 条（`ref/asset_classification.json` by_class）。
- **D 类**：`crates/ti-app/src/terrain_atlas.rs` 注释实测瓦片 63×31~127×105、
  iso_x 范围 0..1216（S6.3），属大画布等距地形。
- **E 类**：`docs/release.md` S6.4 实测 4 个 `num=5` 精灵（14/31/32/37.pcx），
  高度 199 = 5×40 纵向堆叠。

### 2.2 分类标签枚举

```rust
/// HD 资产分类标签（`ref/asset_classification.json` 的 category 值域）。
enum AssetClass {
    A_ui,        // UI 控件/图标/按钮/对话框背景 → 矢量化
    B_portrait,  // 角色立绘/头像 → AI 超分（+可选 SD 重绘）
    C_pixel,     // 兵种/船只/旗帜/地标 → 最近邻 2x
    D_terrain,   // 地形瓦片 → Lanczos 2x + 摄像机 2x
    E_animation, // 多帧动画 → 时序稳定
    Skip,        // 数据/音频/字体/非游戏资源
}
```

---

## 3. 流水线（5 步）

```
┌──────────────────────────────────────────────────────────────┐
│ Step 1  解包 + 分类 + 备份（S7.1）                            │
│   cargo run -p ti-tools --bin unpak -- --out assets_raw      │
│   python scripts/classify_assets.py → ref/asset_classification.json │
│   assets_raw/ = 不可变基准（unpak 幂等，P0-3 逐字节一致）     │
├──────────────────────────────────────────────────────────────┤
│ Step 2  批量超分/矢量化（S7.2）                                │
│   python scripts/pixel_upscale.py   （C/D/E 类）             │
│   python scripts/vectorize_ui.py    （A 类 Inkscape 批处理） │
│   产物：assets_hd/<容器>/<原名>_hd.png（PNG-32）+ manifest_hd.json │
├──────────────────────────────────────────────────────────────┤
│ Step 3  关键图 AI 重绘（S7.3，可选）                          │
│   B 类角色：SD/ComfyUI + ControlNet（Canny/Tile/Depth）      │
│   denoise 0.3~0.5 + 风格 LoRA，纯色背景合成后抠图            │
│   产物：assets_hd/<容器>/<原名>_hd_redrawn.png               │
├──────────────────────────────────────────────────────────────┤
│ Step 4  多分辨率导出 + 打包（S7.4）                           │
│   python scripts/pack_hd_assets.py（1x/2x/4x + 九宫格 + WebP）│
│   cargo run -p ti-tools --bin build_hd_assets -- --check（Rust 侧 manifest 校验） │
│   产物：assets_hd_pack/ + manifest_hd.json（FNV-1a64 幂等）  │
├──────────────────────────────────────────────────────────────┤
│ Step 5  渲染层切换（S7.5）                                    │
│   Framebuf: u16(RGB565) → u32(RGBA8)（cfg feature hd-8bit）  │
│   SpriteAtlas/TerrainAtlas: PCX 解码 → PNG 解码              │
│   路径 1 并行（默认 RGB565+PCX 不变）/ 路径 2 替换（S7.7 可选）│
└──────────────────────────────────────────────────────────────┘
```

### 3.1 Step 1 细节：解包 + 分类

`unpak --out assets_raw/` 产出 7 个容器目录（`ui` 122 条 / `glpics` 88 条 /
`glmap_m01` 47 条 / `glmap_m02` 37 条 / `cards` 4 条 / `music` 6 条 / `goods` 1 条，
合计 305 条，F-207）。每目录含 `manifest.csv`（key,name,usize,sha_len，
name 由 `ti-formats/src/resource_names.rs` 三级反查（S6.5，`mapinfo>exe>EXTRAS`）生成）。

**`scripts/classify_assets.py`**（S7.1 新建）：

```
读 assets_raw/*/manifest.csv（key,name,usize,sha_len）
+ 容器内 filenum INI 的 num 字段（E 类判定，复用 sprite_atlas 解析逻辑）
按文件名模式匹配（见 §2.2）→ 打标签
  - A: ui.pak 中 button_*.pcx / switchpage.pcx / 对话框背景类
  - B: face_*.pcx / Player_*.pcx / Target_head / Target_knife / Target_*.pcx
  - C: glpics.pak 余下 PCX（troop/ship/flag/landmark 系列）
  - D: glmap_m01/m02.pak 全部 PCX
  - E: ui.pak filenum INI num>1 的 4 个精灵（14/31/32/37.pcx）
  - Skip: 非 PCX 条目（INI/Mod/ole 等）
输出 ref/asset_classification.json：
  { "ui/01.pcx": {"class": "A_ui", "source": "pattern"},
    ...
    "_unknown": [ ... ] }   # 未匹配桶，人工复核清零
```

验收判据：`_unknown` 桶 = 0（或每条人工确认归 Skip），全 305 条有标签。

### 3.2 Step 2 细节：批量超分 / 矢量化

**公共前置（所有类）**：PCX 解码 → 索引色转真彩 PNG-32。

```python
# 1. PCX 解码：复用 scripts/pcx_ref.py 的 RLE 反解逻辑（强约束消费到文件尾）
# 2. 索引→真彩：rgb = palette[idx]；背景色 = 调色板 slot 0（逐图确认）
# 3. 透明处理：原版 PCX 无 alpha 通道（blit 模式由 .Map f4 字段选，F-311），
#    HD 管线把「像素值 == 背景 slot」标 alpha=0，其余 255
```

| 类 | 放大方式 | 参数 | 说明 |
|---|---|---|---|
| C | `Image.NEAREST` 2x | 整数倍 | 保像素硬边，不做 AI 平滑 |
| D | `Image.LANCZOS` 2x | — | 地形大画布平滑放大；渲染层 `ISO_X_STEP 32→64`/`ISO_Y_STEP 16→32` 同步 2x |
| E | 5 帧序列 → 合成短视频 → 视频超分 → 回切 5 帧 | 时序稳定 | 无视频超分工具时轻量方案：各帧 NEAREST 2x + 相邻帧差分中值去抖 |
| B | Real-ESRGAN x4 anime 模型 → 按需缩回 2x/4x | `realesrgan-ncnn-vulkan -n realesrgan-x4plus-anime -s 4` | alpha 分离：RGB 超分，alpha 走 Lanczos + 阈值防黑边 |
| A | Inkscape 描摹 → SVG 基准；实际打包 PNG 九宫格 | 9-slice 切图 | macroquad 不直接渲染 SVG，SVG 进 `assets_hd/ui/svgs/` 备查，打包仍用 PNG |

**`scripts/pixel_upscale.py`**（S7.2 新建）：

```
读 ref/asset_classification.json
对 class ∈ {C_pixel, D_terrain, E_animation} 的条目：
  PCX 解码 → 真彩 RGBA → 按类放大（NEAREST/LANCZOS/时序）
  写 assets_hd/<容器>/<name>_hd.png（E 类写 <name>_hd_f0~f4.png）
统计 + 校验（全部产物 PNG-32、尺寸正确、alpha 合理）
输出 manifest_hd.json（沿用 build_assets 的 FNV-1a64 哈希，幂等无时间戳）
```

**`scripts/vectorize_ui.py`**（S7.2 新建，A 类）：

```
对 class == A_ui 的条目：
  PCX 解码 → 真彩 RGBA → Inkscape 命令行描摹（--export-plain-svg）
  写 assets_hd/ui/<name>_hd.svg + 切九宫格 <name>_hd_9s.png
  九宫格切法：取边缘 1/4 像素带为角/边，中心可拉伸
```

**Real-ESRGAN 本机运行**（B 类，S7.2 或 S7.3）：

```
本机 Windows 无独立显卡（推测，需实测）→ Real-ESRGAN 走 CPU 回退（ncnn 支持），
速度约 30~60 s/张（2x 输出）。11 张 B 类可接受。
若过慢 → 降级方案：waifu2x-ncnn-vulkan（纯 CPU 更快）或最近邻 2x（不 AI）。
```

### 3.3 Step 3 细节：AI 重绘（可选）

```
ComfyUI / SD WebUI 工作流（仅 B 类 5 角色）：
1. 输入：assets_hd/ui/<角色>_hd.png（已超分）
2. ControlNet 锁构图：Canny（轮廓）+ Tile（纹理）+ Depth（深度一致性）
3. denoise 0.3~0.5（低重绘幅度保留原画）
4. 风格 LoRA：5 角色固定参考图训练，保证风格一致
5. 合成纯色背景 → 重绘 → rembg 抠图恢复 alpha
产物：assets_hd/ui/<角色>_hd_redrawn.png，与超分版并存（渲染层可选）
```

### 3.4 Step 4 细节：多分辨率打包

```
assets_hd_pack/
├── 1x/<容器>/<name>_hd.png        (原 176×220 逻辑尺寸真彩)
├── 2x/<容器>/<name>_hd_2x.png    (移动端)
├── 4x/<容器>/<name>_hd_4x.png    (桌面/PC)
├── webp/                          (wasm 侧 WebP 转码减体积)
└── manifest_hd.json               (FNV-1a64 + 尺寸 + 分类 + 帧数，幂等)
```

| 端 | 格式 | 说明 |
|---|---|---|
| Web (wasm) | WebP / AVIF | 体积最小；macroquad 侧 CPU 解码（`image` crate 或预转 PNG） |
| Android | ASTC / ETC2 | GPU 纹理压缩（后续增强，当前统一 PNG） |
| PC (native) | PNG-32 | 直接加载 |

**`crates/ti-tools/src/bin/build_hd_assets.rs`**（S7.4 新建，Rust 侧校验）：
读 `assets_hd_pack/manifest_hd.json` 逐条 FNV-1a64 校验 + 尺寸断言，
0 缺失/0 失配才成功（对齐 P0 级验收风格）。

### 3.5 Step 5 细节：渲染层切换

#### 帧缓冲色深（RGB565 → RGBA8，cfg 门控）

```rust
// crates/ti-platform/src/gfx.rs（S7.5）
pub struct Framebuf {
    pixels: Box<[u32]>,   // 0xAARRGGBB（默认仍按 RGB565 值写入低 16 位，高 8 位 0xFF）
    dirty_rows: [bool; 220],
}
// 旧 u16 API 保留为适配层（as_slice_rgb565 等），默认行为逐像素不变
```

- `sprite_atlas.rs` 的 `blit_sprite`：新增 PNG 解码路径（`image` crate 或
  自研最小 PNG 解码器——**决策点**，见 §4），`rgba` 直接写入 u32。
- `terrain_atlas.rs` 的 `blit_decoded`：同上 + `ISO_X_STEP`/`ISO_Y_STEP` 按 2x 资源
  翻倍（cfg 联动）。
- `menu_shell.rs` 的 `fill_rect`/字形 blit：u16 色值 → u32 适配。

#### P4-1 兼容（两路设计，用户已选「两路都写」）

| 路径 | 做法 | P4-1 基准 | 优点 | 缺点 |
|---|---|---|---|---|
| **1（并行，先做）** | `ColorMode`/`SourceFormat` 枚举，默认 `Rgb565`+`Pcx`；HD 走 `Rgba8`+`PngHd`；设置菜单加「画质：原版/HD」 | 原 `render_parity.rs` 不动；新增 `render_parity_hd.rs` | 零破坏 | 维护两套 blit 路径 |
| **2（替换，后评估）** | 默认改 `Rgba8`+`PngHd`；删 PCX 渲染路径（`pcx.rs` 解码器保留供离线工具用） | `render_parity.rs` 基准图全重生成（HD 资源渲染），P4-1 判据改「HD 解码参考图逐像素一致」 | 代码简洁 | P4-1 基准重写 + 重新验收 |

**建议节奏**：S7.5 落地路径 1（HD 管线稳定运行 ≥ 2 周）后，评审是否切路径 2（= S7.7）。

---

## 4. 工具链选型

| 环节 | 首选 | 备选 | 说明 |
|---|---|---|---|
| PCX 解码 | `scripts/pcx_ref.py`（已有） | `ti-formats::pcx`（Rust） | Python 侧离线批处理用 |
| 索引→真彩 / 放大 | Python Pillow（本机 12.2.0） | ImageMagick | 零新增依赖（§5.9） |
| AI 超分 | Real-ESRGAN ncnn-vulkan | waifu2x（CPU 更快） | B 类 |
| 矢量化 | Inkscape 描摹 | Vector Magic / potrace | A 类；Inkscape 未装则先落 Python 切图 + SVG 占位，标注待人工 |
| 九宫格 | 自研切图脚本 | TexturePacker | 边缘 1/4 像素带 |
| 视频超分 | Topaz Video AI | RIFE + Real-ESRGAN video | E 类；不可用时走中值去抖轻量方案 |
| AI 重绘 | ComfyUI + ControlNet | SD WebUI | S7.3 可选 |
| Rust 校验 | `ti-tools/build_hd_assets` bin | — | 复用 FNV-1a64（`ti_tools::hash`） |

**决策点（S7.2 前需确认）**：
1. Inkscape 是否本机已装（未装 → A 类先落 PNG 九宫格，矢量化留人工）；
2. Real-ESRGAN 可执行是否可获取（CPU 回退实测速度）；
3. PNG 解码进 Rust 渲染层：引 `image` crate（违反零依赖原则需走 §5.9 例外审批）
   还是自研最小 PNG/调色板解码器（推荐：离线转 RGBA8 PPM/裸字节，Rust 侧零解码器，
   运行时只读裸像素——**推荐此方案**，保持 §5.9）。

---

## 5. 风险与回退

| 风险 | 影响 | 回退 |
|---|---|---|
| 本机无独立显卡，Real-ESRGAN CPU 慢 | B 类超分耗时 | waifu2x（纯 CPU）或降最近邻 2x |
| Inkscape 未装 / 描摹质量差 | A 类不达标 | 向量 Magic / 手动重画；先 PNG 九宫格顶上 |
| RGBA8 改动波及全部 blit 路径 | 测试量大 | 路径 1 并行，默认 cfg 关 RGB565 不变 |
| HD 全量 ~5 MB（2x）超首屏预算 | 移动端加载慢 | 首屏只 1x，2x/4x 分包；WebP 压缩（§6） |
| 多帧动画时序闪烁 | UI 动画观感差 | 最近邻 + 中值去抖，不做视频超分 |
| P4-1 原基准失效（路径 2） | 需重新验收 | 路径 1 保留原基准，HD 走 `render_parity_hd` |
| 版权（资源非自有） | 商用侵权风险 | 获得授权或完全重画（用户已知悉） |

---

## 6. 体积预算（HD 单列，重列 `docs/modernization.md` §9）

| 类别 | 数量 | 单张预估（2x PNG） | 小计 |
|---|---|---|---|
| A. UI 控件 | ~30 | 5~15 KB | ~300 KB |
| B. 角色立绘 | ~11 | 30~80 KB（4x 超分后） | ~600 KB |
| C. 兵种/地标 | ~77 | 5~20 KB | ~1 MB |
| D. 地形瓦片 | ~84 | 15~50 KB | ~3 MB |
| E. 多帧动画 | 20 帧 | 5~10 KB/帧 | ~150 KB |
| **合计** | | | **~5 MB（2x）**；4x ≈ 20 MB |

- 首屏策略：只载 1x（~1.2 MB）+ 关键场景 2x；4x 仅桌面非首屏分包。
- WebP 转码再减 30~50%（wasm 侧）。
- 预算表在 S7.6 重列进 `docs/modernization.md` §9（HD 单列，与原版 380 KB 口径并存）。

---

## 7. 验收判据（P5 级，扩展 `docs/parity.md`）

| # | 判据 | 验收命令 |
|---|---|---|
| P5-1 | HD 资源全量落盘 + manifest 校验 | `cargo run -p ti-tools --bin build_hd_assets -- --check` → 0 缺失 |
| P5-2 | RGBA8 渲染逐像素一致（HD 基准图） | `cargo test -p ti-app --test render_parity_hd` 全绿 |
| P5-3 | 多分辨率（1x/2x/4x）坐标往返不变式 | `cargo test -p ti-platform --test coordinate_roundtrip`（扩展 RGBA8） |
| P5-4 | 首屏 HD 资源体积达标 | `evidence/S7.6-size.log` 与 §6 预算比对全 PASS |
| P5-5 | 三端 bin（native/wasm/android host）编译通过 | `scripts/gate.ps1` 20 项全绿（S7 时点 8 项；当前 20 项） |

> 现有 P0~P4 不变；S7.5 路径 1 下 P4-1 原基准继续绿。

---

## 8. 文件改动清单

### 8.1 新增

| 文件 | 阶段 |
|---|---|
| `scripts/classify_assets.py` | S7.1 |
| `ref/asset_classification.json` | S7.1 |
| `scripts/pixel_upscale.py` | S7.2 |
| `scripts/vectorize_ui.py` | S7.2 |
| `assets_hd/`（产物目录） | S7.2 |
| `scripts/comfyui_workflow.json` | S7.3（可选） |
| `scripts/pack_hd_assets.py` | S7.4 |
| `crates/ti-tools/src/bin/build_hd_assets.rs` | S7.4 |
| `assets_hd_pack/`（产物目录） | S7.4 |
| `crates/ti-app/tests/render_parity_hd.rs` | S7.5 |
| `docs/plan-s7.md` | S7.1 |
| 本文件 | S7.1 |

### 8.2 修改

| 文件 | 改动 | 阶段 |
|---|---|---|
| `crates/ti-platform/src/gfx.rs` | Framebuf u32 RGBA8 路径（cfg `hd-8bit`） | S7.5 |
| `crates/ti-app/src/sprite_atlas.rs` | 加 PNG/RGBA8 解码路径（cfg） | S7.5 |
| `crates/ti-app/src/terrain_atlas.rs` | RGBA8 + 2x 摄像机步进（cfg） | S7.5 |
| `crates/ti-app/src/menu_shell.rs` | u32 色值适配（cfg） | S7.5 |
| `scripts/build_wasm.ps1` | HD 分包 | S7.6 |
| `docs/modernization.md` §9 | 体积预算 HD 单列 | S7.6 |
| `docs/parity.md` | 新增 P5 判据 | S7.5 |
| `plan/ledger.yaml` | S7.1~S7.7（已登记） | S7.1 |
| `docs/README.md` + `AGENTS.md` | 路由表登记 | S7.1 |

---

## 9. 最快落地建议

| 档 | 工期 | 内容 | 适用 |
|---|---|---|---|
| **1 天** | 0.5 周 | S7.1（分类）+ C/D 最近邻/Lanczos 2x + S7.4 打包 | 原型验证 |
| **1 周** | 1 周 | 上档 + B 类 ESRGAN + A 类九宫格/矢量化 + S7.5 渲染层（路径 1） | 可玩 Demo |
| **高质量** | 2~4 周 | 全 S7.1~S7.7（含 AI 重绘 + 路径 2 评估） | 正式多端 |
