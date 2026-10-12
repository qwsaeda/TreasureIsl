# 经验库 · build

> 构建与三端产物：工具链选型、wasm 壳、浏览器自动化。条目 ID 全局唯一（EXP-NNN），维护规则与全量索引见 docs/experience/index.md。

### EXP-008 构建：本机无 Docker → 放弃 macroquad docker APK 路线
- 现象：`docker` 命令不存在；macroquad 官方 Android 构建推荐 `notfl3/cargo-apk` 镜像。
- 根因：环境未装 Docker，Android SDK/NDK 也未配置（无 `ANDROID_*` 环境变量）。
- 解法：S2.6 改为本机 `cargo-apk`/`cargo-ndk`（Java 17 已具备，需补装 SDK/NDK）；`game/tools` 代码不受影响，因为构建方式隔离在 S2。
- 验证：S2.7 smoke 三端全绿后本条标 `已解决`。


### EXP-011 构建：本机无 `rg`、`bash` 是 WSL 版，门禁必须用 PowerShell
- 现象：按计划写 `gate.sh`（含 `rg` 扫描），探测发现 `rg` 不存在、`bash.exe` 是 `system32` 的 WSL 入口而非 Git Bash。
- 根因：计划里的门禁形态默认了 Linux 工具链，没先探测本机。
- 解法：门禁改 `scripts/gate.ps1`；占位扫描用 `Select-String`；不依赖 bash。已同步改 `AGENTS.md`/`parity`/`loop`/`style`/`plan-*`/`ledger` 中全部 `gate.sh` 引用。
- 验证：`git grep gate.sh` 0 命中；gate.ps1 实际跑通。


### EXP-028 工程：macroquad wasm 是「裸 wasm + gl.js」，不是 wasm-bindgen（S2.5 实测）
- **选型落实**：workspace 此前**根本没有 macroquad 依赖**（ti-platform 只有 trait）。S2.5 才引入 `macroquad = "0.4"`（解析到 0.4.16）+ miniquad 0.4.11。
- **关键事实（省掉大量试错）**：macroquad/miniquad 的 wasm32 产物**不需要 wasm-bindgen**。`Cargo.lock` 里**没有任何 wasm-bindgen / js-sys / web-sys**；产物是自包含模块，导出 `memory`/`main`/`frame`/`key_down`/`mouse_down` 等 24 个符号。启动方式是**经典 script 引入 miniquad 的 `js/gl.js`（67KB，随 crate 发布在 `miniquad-*/js/`）后调 `load("xxx.wasm")`** —— 不是 `import()`，也不是 `WebAssembly.instantiateStreaming`。
  -> 教训：判断 wasm 前端要不要 bindgen，先 grep Cargo.lock 有没有 wasm-bindgen；有则要 glue，无则找宿主 `gl.js`（看 crate 的 `js/` 目录与 README 的 index.html 示例）。
- **自定义 boot.js 是错的**：一开始按 wasm-bindgen 习惯写了 `import('./treasure_wasm.js')`，而产物根本没有 `.js` 胶水；改为 `instantiateStreaming` 也缺 miniquad 需要的宿主回调（`frame`/`key_down` 由 JS 侧调用 wasm，方向相反）。**教训：wasm 的胶水方向要看导出表** —— 导出里有一堆 `frame`/`mouse_*` 回调，说明是 JS 驱动 wasm（宿主式），而非 wasm 调 JS。
- **debug vs release 体积**：同一工程 debug 17.7MB / release 615KB（28x）。wasm 交付一律 `--release`。
- **API 与 0.4 文档不同**（实测踩坑）：模块是 `macroquad::texture`（**无 s**）不是 `textures`；`Texture2D::from_rgba8(u16,u16,&[u8])` **不是** `from_rgba`；`Color` 是 **f32** 通道（不是 u8）；`Mesh` 是具名字段 `{vertices, indices, texture}` **没有** `Mesh::new(...)`；`#[macroquad::main]` 收 `Conf` **不是** `&str`，也没有 `Window::new`。
- **gate 解锁**：装上 target 后 gate 第 4 项 wasm 行自动由 SKIP 转 PASS（脚本按 `rustup target list --installed` 判定，无需改脚本）。
- **PowerShell 脚本必须存 UTF-8 BOM**（EXP-016 再次命中）：`build_wasm.ps1` 用 Write 工具写出（无 BOM）时，PS 5.1 按 GBK 读中文注释，把字节打断导致 `ParserError: UnexpectedToken`（报错位置与真实语法错误无关，容易误判）。补救：`.ps1` 一律加 BOM。
- **clippy（`--all-targets` 才会扫到）**：`f32 as i64/usize` 触发 `cast_possible_truncation`/`cast_sign_loss` —— 逻辑坐标一律用整数（`i32`/`usize`），别在像素索引上用 f32；`as f32` 触发 `cast_precision_loss`（EXP-015 已记，用 `LW`/`LH` 常量）；`require-features` 的 bin 默认 lint **不扫**，要 `clippy --features wasm-shell` 单独跑。


### EXP-029 流程：headless Edge `--virtual-time-budget` 不驱动 rAF，macroquad wasm 文件加载永久挂起（S2.5 补实测）
- 现象：用 `msedge --headless=new --virtual-time-budget=30000 --screenshot` 抓 wasm 壳截图，画布全黑；`--dump-dom` 探针发现 20 秒虚拟时间内 rAF 只跑了 4 帧，`assets/music.lpk` 从未被 fetch（但 GoldLand.Fnt / font.dat 已 fetch）。
- 根因：`--virtual-time-budget` 快进定时器但**不真实推进 rAF**（headless 无合成器）。macroquad 的 `macroquad::file::load_file` 在 wasm 下走 miniquad JS 的 `fs_load_file`（XMLHttpRequest），完成回调写回 `Arc<Mutex<Option>>` 后，**future 的 resolve 依赖每个 rAF 帧轮询 executor**。虚拟时间下帧极少 → 第 2 个 `load_file` 的 future 永远没被 poll 到 resolve → `Shell::new().await` 卡死 → 主循环从未进 `render()` → 画布永黑。**这不是 app 的真 bug**：真实浏览器 rAF 60fps，毫秒级完成全部加载。
- 解法：改用**真实时间**驱动浏览器。Node v24+ 自带全局 `WebSocket`，零依赖直连 CDP：`Emulation.setDeviceMetricsOverride` → `Page.navigate` → `await sleep(9000)` 真实毫秒 → `Page.captureScreenshot(fromSurface:true)`。脚本见 `C:\Users\Administrator\AppData\Local\Temp\opencode\cdp_shot.mjs`。
- 验证：真实时间下 PROBE 报 `resources=...,GoldLand.Fnt,font.dat,music.lpk wasmExports=object`（6 资源全 fetch、wasm 实例化、0 异常）；截图 1408×1760 从 13643 B→104164 B，blue 像素 23,667→1,596,391。
- 教训：**headless 抓 WebGL 截图必须用真实时间**；`--virtual-time-budget` 只适合纯 DOM/SSR 页面。CDP 比 `--screenshot` 灵活得多且 Edge/Chrome 通用。


### EXP-045 构建：native macroquad 窗口下 `FramePresenter` 呈现失效 → 黑屏；纹理 V 语义相反 → 画面翻转（S2.8 实测）
- 现象：`cargo run -p ti-app --features native-shell --bin treasure-native` 弹出 176×220 窗口但**整窗黑屏**；绕过呈现器后用每帧全帧上传，画面显示但**上下颠倒**（菜单标题「金银岛」出现在屏幕底部、状态行在顶部）。
- 根因（两层）：
  1. **黑屏**：`FramePresenter`（S4.3 脏行带 + 纹理复用）是 **wasm 路径调优的**（canvas 拉伸 + miniquad JS 宿主的 GPU 上传语义）。在 native macroquad 桌面窗口下，首帧 `new()` 里 `fb.clear(0x0000)` 置全帧脏 → `present()` 上传了**纯黑帧**后 `last_rendered` 锁死成 `Some(0)`，`selected` 不变就不再重渲 → 一直黑。stats 实测 `uploads=1 skipped=60`（首帧 1 次黑帧上传，之后全跳）。
  2. **翻转**：`draw_quad` 的纹理 V 映射（`from_rgba8` V=0 在顶）在 **native macroquad 下语义与 wasm 相反**——native 桌面端上传的 RGBA8 字节行序经 `draw_quad` 的 `V=1 在顶` 映射后，逻辑 y=12 的标题被画到屏幕底部。wasm 侧 canvas Y 向下 + miniquad 的 V 语义恰好互补，所以 wasm 正常。
- 解法：
  1. native 主循环改**每帧 `render()` + `present_framebuffer` 全帧上传**（绕过 `FramePresenter` 的脏行带 + 纹理复用），关 S4.3 静态帧优化。
  2. 新增 `ti_platform::gfx::present_framebuffer_flipped`（**仅 native bin 调用**）：上传前把 RGB565 行主序像素**按行倒序**（末行先写 = 纹理顶 V=1），使 native 纹理语义下标题回顶部。共享 wasm 路径的 `present_framebuffer`/`draw_quad` **不动**，避免破坏已正常的 wasm。
- 验证：`PrintWindow` 抓窗口客户区（client=176×220）non-black 100%（修复前 0%）；翻转后按 5 行分带采样，顶部带含青绿标题条（0x01E0）、底部带含白色字形（0xFFFF），顺序与原版一致。
- 教训：
  1. **wasm 调优的呈现层搬到 native macroquad 不能直接复用**——GPU 上传/纹理 V 语义在两端相反，搬前必须逐层实测（呈现器、UV 映射、行序）。
  2. **native 端 S4.3 脏行带优化暂不可用**（全帧上传），要 native 也吃优化需单独排任务重新适配 viewport/GPU，不阻塞基础功能。
  3. 跨端共享表现层时，**呈现（GPU 上传 + 纹理映射）必须按端分叉**，渲染逻辑（帧缓冲写入）可共享——本例 `MenuShell` 渲染共享、`present` 按端选 `present_framebuffer`（wasm）/`present_framebuffer_flipped`（native）。

### EXP-045（续）native 端 S4.3 脏行带优化已补做（status: resolved）
- 现象：EXP-045 初版 native 走「每帧全帧上传」，S4.3 静止帧 0 GPU 优化未生效（`uploads` 每帧 +1）。
- 根因：`FramePresenter` 的部分上传（`update_*`）在 native macroquad 下不重进本帧批次；直接绕开它用全帧上传虽正确但丢优化。
- 解法：`FramePresenter` 加 `UploadMode` 枚举（`Partial` 默认 wasm / `FullRebuild` native）。native bin 用 `MenuShell::with_upload_mode(..., FullRebuild)`，主循环恢复 `last_rendered` 静态帧判据 + `presenter.present()`；`FullRebuild` 模式下静止帧也 `draw_quad` 复用上次纹理（避免黑底），脏帧整帧 `from_rgba8` 重建。`draw_quad` 的 UV V 坐标翻转正（行主序 bytes 前部=逻辑顶，屏顶 V=0），画面不颠倒。
- 验证：实跑 stats `uploads=1 skipped=59` → `uploads=0 skipped=60` 全跳、60fps；`gate.ps1` 全绿。wasm 路径（`Partial` + 原 UV）未动。
- 教训补充：跨端呈现层分叉后，**S4.3 优化可随端配置**——`UploadMode` 让同一 `FramePresenter` 在两端各自取正确上传策略，而非 native 彻底绕开优化。


### EXP-046 构建：大依赖（opus-pure）feature 门控 + 单一传递路径（S4.4 复核）
- 现象：`opus-pure`（OGG/Opus 纯 Rust 编码器，~500 KB）若作为 `ti-platform` 的默认依赖被运行时壳（wasm/native）拉入，wasm 主产物体积膨胀超 800 KB 预算（`docs/plan-s4.md §S4.3`）。
- 根因：转码是**离线工具能力**（mod2ogg / build_assets），与运行时播放（quad-snd 播 OGG 字节）是两件事；把编码器写进 `ti-platform` 默认依赖会把运行时体积拖垮。
- 解法（feature-gating + 单一路径）：
  1. `ti-platform`：`opus-pure = { workspace = true, optional = true }`，feature `ogg = ["dep:opus-pure"]`（**不进 default**）；`ogg_encode` 模块用 `#[cfg(feature = "ogg")]` 门控。
  2. `ti-tools`：只经 `ti-platform = { workspace = true, features = ["ogg"] }` 传递引入 opus-pure；**移除直接依赖** `opus-pure.workspace = true`（mod2ogg/build_assets 代码只调 `ti_platform::ogg_encode::encode_ogg`，从不用 `opus_pure::` 直接项）。`cargo tree -p ti-tools` 验证单一路径 `ti-tools → ti-platform → opus-pure`。
  3. `ti-app`：独立 feature `mod-ogg = ["ti-platform/ogg"]`（默认关）；`audio_prep.rs` 的在线转码路径 `#[cfg(feature = "mod-ogg")]` 门控，默认构建不调 `encode_ogg`，MOD 条目计入 skipped（离线 OGG 由 build_assets 提供）。运行时（wasm/native）BGM 直接播离线 OGG 产物。
- 验证：`cargo build -p ti-app --features wasm-shell --release --target wasm32-unknown-unknown` 依赖树无 opus-pure；wasm 主体 675,047 B ≤ 800,000（`evidence/S4.3-size.log` S4.4 行）；gate 全绿。
- 教训：**体积敏感的可选能力（编码器/大 codec）一律「库 crate optional dep + feature，调用方按需开」**；工具与运行时各走各的 feature，默认构建不拉入；保持单一传递路径（`cargo tree` 核对），工具不直接依赖底层 codec。


### EXP-047 构建：手工 `Mesh`+`draw_mesh` 弱引用纹理在 native macroquad 下不渲染 → 黑屏；EXP-045 验证漏像素输出（S6.2 实测）
- 现象：`cargo run -p ti-app --features native-shell --bin treasure-native` 窗口整屏黑（深蓝边框 0x000084 ≈ 黑）；帧缓冲 PNG dump 证实 `render()` 正确写入菜单内容（非黑、多色），即**画面数据对，GPU 侧没画出来**。
- 根因：`draw_quad` 手工构造 `Mesh{texture: Some(texture.weak_clone())}` + `draw_mesh`。native macroquad 桌面 GL 后端的纹理批处理器对「本帧刚 `from_rgba8` 建的独立纹理 + 弱引用 mesh」不重新进批次/不绑定采样器 → 画面 quad 实际未渲染，只剩 `fill_border` 整屏深蓝 → 看起来黑。
- 被 EXP-045 掩盖：EXP-045 验证「全跳 stats `uploads=0 skipped=60`」只证明**CPU 侧脏行跳帧生效**，从未核对 native 窗口实际像素输出 → 渲染 bug 被 stats 绿灯盖住。
- 解法：`draw_quad` 改用 macroquad 宏观 API `draw_texture_ex(tex, x0, y0, WHITE, DrawTextureParams{dest_size: Some(vec2(w,h)), flip_y: !is_wasm, ..})`（自带批处理 + 正确纹理绑定）；`flip_y` 处理 native 与 wasm 纹理 V 语义相反（EXP-045），`cfg!(target_arch="wasm32")` 区分两端。
- 验证：实跑 `get_screen_data` dump native 窗口实际像素 → 非黑占比 99.6%，菜单标题区在顶部（FF00FF/FFFFFF 主导），底部为状态行（003C00）；`flip_y` 方向正确。`gate.ps1` 全绿（含 clippy `items_after_statements` 已修：`const IS_WASM` 置于函数首）。
- 教训：
  1. **跨端呈现层验收必须核对目标端实际像素输出（截图/dump 屏幕），不能只信 CPU 侧计数（uploads/skipped/fps）**。
  2. **native macroquad 画整帧纹理优先用 `draw_texture_ex`，别手工 `Mesh`+`draw_mesh` 弱引用纹理**——独立纹理在 native 批次下易漏采样。

### EXP-062 构建：macroquad `flip_y` 必须端侧固定，禁止因「画面反了」反复翻转（S5/S6 后续实测）
- 现象：native 桌面窗口画面上下颠倒；随后多轮修复中 `flip_y` 在 `IS_WASM` / `!IS_WASM` 之间来回改，导致 native/wasm 任一端被改反。
- 根因：`draw_texture_ex` 的 `flip_y` 是纹理 V 坐标翻转开关，不是「让画面看起来正确」的调试开关。行主序 framebuffer 的首行是逻辑 y=0（画面顶部）。在 macroquad 0.4 本项目实测中：native 端使用 `flip_y = IS_WASM` 时，native `IS_WASM=false` → `flip_y=false`，画面正确；wasm 端 `IS_WASM=true` → `flip_y=true`，画面正确。任何改成 `!IS_WASM` 的尝试都会把 native 改反。
- 解法：`crates/ti-platform/src/gfx.rs` 的 `draw_quad` 固定为：
  ```rust
  const IS_WASM: bool = cfg!(target_arch = "wasm32");
  DrawTextureParams { flip_y: IS_WASM, .. }
  ```
  即 native 不翻转、wasm 翻转。后续遇到画面反向时，先查 frame_delta/窗口尺寸/渲染层级/纹理上传路径，不得先改 `flip_y`。
- 验证：native 运行主菜单时，顶部标题条和「金银岛」位于屏幕上方，底部状态行位于屏幕下方；全仓 `cargo test` 与 `scripts/gate.ps1` 全绿。



