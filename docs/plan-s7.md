# 执行计划 S7：HD 资产增强管线（分档落地，1 天 ~ 4 周）

目标：把原版 176×220 RGB565 索引色 PCX 美术升级为**全 HD 真彩色**资源管线，
支持 1x/2x/4x 多分辨率，三端统一消费。完整规格见 `docs/impl/14_asset_enhancement.md`。

**节奏**：1 天档（S7.1 + C/D 批量 + S7.4 打包）→ 1 周档（+ B 类 AI 超分 + A 类矢量化 + S7.5）
→ 高质量档（+ S7.3 重绘 + S7.7 路径 2）。

## 任务分解

### S7.1 资产解包 + 分类 + 备份

- `cargo run -p ti-tools --bin unpak -- --out assets_raw`（已有工具，幂等基准）。
- 新建 `scripts/classify_assets.py`：读 `assets_raw/*/manifest.csv` + filenum INI num 字段，
  按 `docs/impl/14` §2.2 文件名模式打 A/B/C/D/E/Skip 标签。
- 产出 `ref/asset_classification.json`（全 305 容器条目，`_unknown` 桶人工复核清零；
  S7.2 产出 `assets_hd/_stats.json` 335 含九宫格派生，口径不同）。
- 验收：分类覆盖 100%；`gate.ps1` 全绿；evidence/S7.1.log。

### S7.2 批量超分 / 矢量化

- 新建 `scripts/pixel_upscale.py`（C/D/E 类）：PCX 解码 → 索引转真彩 RGBA →
  C=NEAREST 2x / D=LANCZOS 2x / E=5 帧序列时序处理（无视频工具时中值去抖）→
  PNG-32 落 `assets_hd/<容器>/<name>_hd.png`。
- 新建 `scripts/vectorize_ui.py`（A 类）：Inkscape 描摹 SVG 基准（Inkscape 未装则先切
  PNG 九宫格 `<name>_hd_9s.png`，SVG 留人工，标注待办）。
- B 类（可选并入 S7.2 或 S7.3）：Real-ESRGAN anime x4（CPU 回退），alpha 分离（RGB 超分 +
  alpha Lanczos 阈值防黑边）。
- 产出 `assets_hd/` + `manifest_hd.json`（FNV-1a64 幂等，复用 `ti_tools::hash`）。
- 验收：全条目有 HD 产物（按分类标签）；PNG-32；manifest 校验通过；gate 全绿；evidence/S7.2.log。

### S7.3 关键图 AI 重绘（可选）

- B 类 5 角色：SD/ComfyUI + ControlNet（Canny/Tile/Depth）denoise 0.3~0.5 + 风格 LoRA，
  纯色背景合成后 rembg 抠图。
- 产物 `assets_hd/ui/<角色>_hd_redrawn.png`；人工评审风格一致。
- 可选任务，不阻塞 S7.4；无 GPU/工具则标 blocked 记录原因。

### S7.4 多分辨率打包 + Rust 侧校验

- 新建 `scripts/pack_hd_assets.py`：1x/2x/4x 切分 + 九宫格 + WebP 转码（wasm 侧）→
  `assets_hd_pack/`。
- 新建 `crates/ti-tools/src/bin/build_hd_assets.rs`：读 `manifest_hd.json` 逐条 FNV-1a64
  校验 + 尺寸断言，0 缺失/0 失配才成功。
- 验收：`cargo run -p ti-tools --bin build_hd_assets -- --check` 全绿；manifest 幂等；gate 全绿；
  evidence/S7.4.log。

### S7.5 渲染层切换（RGBA8 帧缓冲 + HD 图集，路径 1 并行）

- `ti-platform/src/gfx.rs`：`Framebuf` 加 RGBA8 路径（`#[cfg(feature = "hd-8bit")]`，
  默认关；RGB565 逐像素行为不变）。
- `ti-app/src/sprite_atlas.rs` + `terrain_atlas.rs`：加 PNG/RGBA8 解码路径（cfg 联动）；
  地形 2x 资源时 `ISO_X_STEP`/`ISO_Y_STEP` 翻倍。
- **PNG 解码零依赖决策**（`docs/impl/14` §4 决策点 3）：离线把 PNG 转 RGBA8 裸字节
  （PPM 或裸 buffer + 尺寸头），Rust 运行时只读裸像素，不引 `image` crate（保 §5.9）。
- 新增 `crates/ti-app/tests/render_parity_hd.rs`（HD 基准图逐像素一致，P5-2）。
- 验收：`cargo test -p ti-app` 全绿（原路径）+ `cargo test -p ti-app --test render_parity_hd`
  全绿（HD 路径）+ P4-1 原基准仍绿 + gate 全绿；evidence/S7.5.log。

### S7.6 三端接线 + 体积预算重列

- `scripts/build_wasm.ps1` 加 HD 分包（首屏 1x + 关键场景 2x，4x 桌面非首屏）。
- `docs/modernization.md` §9 体积预算 HD 单列重列（`docs/impl/14` §6 表）。
- 设置菜单加「画质：原版/HD」切换项（`docs/impl/11` §7.1 设置表扩展）。
- 验收：`evidence/S7.6-size.log` HD 首屏体积与预算比对全 PASS；三端 bin 编译通过；gate 全绿。

### S7.7 P4-1 基准切换（可选，路径 2）

- 评审决定后：`render_parity.rs` 基准图全重生成（HD 资源渲染）；删 PCX 渲染路径
  （`pcx.rs` 解码器保留供离线工具）；P4-1 判据改「HD 解码参考图逐像素一致」。
- S7.5 稳定运行 ≥ 2 周后评估；evidence/S7.7.log。

## 依赖

- S7.1 deps: S6.4（S6 阶段全 verified）
- S7.2 deps: S7.1
- S7.3 deps: S7.2（可选）
- S7.4 deps: S7.2
- S7.5 deps: S7.4
- S7.6 deps: S7.5
- S7.7 deps: S7.5（可选，稳定后）

## 终止条件

- 1 天档 = S7.1 + S7.2（C/D）+ S7.4 verified；
- 1 周档 = 上档 + S7.2（B/A）+ S7.5 verified；
- 高质量档 = 全 S7 verified（S7.3/S7.7 可选，blocked 需登记原因）。
