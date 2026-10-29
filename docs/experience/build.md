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

### EXP-109 构建/工具：PowerShell 5.1 对「无 BOM 的 UTF-8 脚本」按 ANSI 解析 → 文档里的一键命令入口根本跑不起来
- **现象**：S18.8a 改 `scripts/playbook.ps1`（AGENTS.md 里的「常用命令一键入口」）与
  `scripts/smoke.ps1` 后做语法自检，两份脚本**都报 ParserError**（`字符串缺少终止符` /
  `ParserError: TerminatorExpectedAtEndOfString`）。实测
  `powershell -ExecutionPolicy Bypass -File scripts\playbook.ps1 -List` 直接失败 ——
  **AGENTS.md 写着的一键入口在中文 Windows 上从未跑起来过**（HEAD 版本即如此，非本次引入）。
- **根因**：PowerShell 5.1 读 `.ps1` 时，**无 BOM 就按系统 ANSI 代码页**（中文 Windows = GBK）
  解码；文件是 UTF-8 → 汉字被逐字节误判 → 某些字节序列恰好构成引号/反引号 → 整文件语法崩。
  仓库里 5 个 `.ps1` 只有 `playbook.ps1` / `smoke.ps1` 缺 UTF-8 BOM（其余 3 个都有）→
  恰好就是这两个含大量中文提示的脚本。门禁**不跑**这两个脚本（gate 只跑 gate.ps1 /
  flywheel_readme.ps1，两者有 BOM）→ 坏了也没人发现。
- **解法**：① 两个脚本补 **UTF-8 BOM**（BOM = PS 5.1 认 UTF-8 的唯一可靠信号），
  换行统一 CRLF，与仓库其余 `.ps1` 一致；② 写/改 `.ps1` 时**必须带 BOM**（写工具若不保留 BOM，
  改完立刻用 `[Parser]::ParseFile` 做语法自检 + 真跑一次）；③ 文档里给出的命令要**真跑一遍**
  再写进 AGENTS/发布说明（本次即靠「跑一次 playbook -List」发现入口是死的）。
- **验证**：`powershell -ExecutionPolicy Bypass -File scripts\playbook.ps1 -List` → exit 0
  （16 项命令正常列出）；`scripts\smoke.ps1` → exit 0（native 构建 + 全测试 + wasm 构建 +
  HTTP 端点 200 + unpak + 台账 全绿，「SMOKE RESULT: PASS」）。




### EXP-119 构建：bevy 文本 CJK 分词每帧刷 ICU4X 错：须开 complex_script_segmentation（parley 完整 CJK 字典）
- **现象**：bevy 0.20（parley 文本引擎）渲染 CJK 文本时，debug 态每帧打 `ICU4X data error: No segmentation model for language: ja`（`ja` = CJK 字典键，zh 亦走此键），每场景 ~24 行刷屏。
- **根因**：parley 的 icu_segmenter 仅在启用 complex-scripts feature 时构建完整字典（含 cjdict）；否则回退 `new_southeast_asian()`（只有 My/Km/Lo/Th 四语言），CJK 词元分词全部未命中 ⇒ 逐词打错。bevy 的 feature 名 = `complex_script_segmentation`（默认不含）。
- **解法**：`ti-engine/Cargo.toml` 的 bevy features 加 `"complex_script_segmentation"`（→ parley/complex-scripts → 完整 ICU 字典）。这是 bevy CJK 文本应用的**必开 feature**，写进 `docs/process/12_bevy_usage.md` 坑位清单。
- **验证**：`cargo run -q --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native -- --resizetest` 输出 grep `No segmentation model|ICU4X data error` = 0 行（改前 debug ~24 行/场景）；resizetest PASS。

### EXP-120 构建：cargo 自动探测 src/bin/*.rs 为独立 bin：子模块互引 E0433/E0601，须 autobins=false
- **现象**：`crates/ti-engine` 在 `src/bin/` 声明 2 个 bin（`treasure_bevy.rs` + `resizetest.rs`），resizetest 同时作为主模块的子模块（`mod resizetest;`）被 `treasure_bevy` 引用 ⇒ E0433（too many `super`）+ E0601（缺 main）。此前 gate 只编译声明 bin 所以潜伏。
- **根因**：cargo auto-roots 会把 `src/bin/*.rs` 每个文件都探测为独立 bin；当某 bin 文件同时是另一 bin 的子模块时，作为独立 bin 编译必然缺 main / 多 super。`--all-targets`（clippy/test）才会编译到这个第 3 个被探测目标并爆错。
- **解法**：`[package] autobins = false` 关自动探测（保留显式 `[[bin]]` 声明）。任何「bin 文件互引子模块」的布局都必须配 `autobins = false`。
- **验证**：`cargo clippy -q --manifest-path crates/ti-engine/Cargo.toml --all-targets -- -D warnings` = exit 0；`--resizetest` 相机验算 PASS（area 1080×1920，px/unit 0.5→1.0）。

### EXP-127 构建：截图判据「非空」≠「有内容」：冷启动截到相机 clear 色全幅帧静默出灰图
现象：treasure-bevy-native --shot --scene=main_map 出全灰 (43,44,47) 图；同命令重跑又正常；index 场景正常 ⇒ 全局管线好，间歇性。\n定位：显式 TI_MAP=m01 与 .default 指针两条注入路径代码等价（default_pack_id 有 trim）；layout 子集二分 8 组全部正常——键相关性为假相关；分界点是冷/热缓存，锁 finish_shot 判据。\n根因：判据「非空」无法区分 clear 色帧与真内容帧；且无帧上限兜底。\n解法：finish_shot 判据升级「非空 且 非纯色帧」（新增 is_blank_frame：单一 RGB 占比 ≥99% = 未就绪）+ 600 帧强制退出兜底（WARN 告警）。\n验证：连跑 3 次 --shot main_map，CTA 恒 (234,176,62) + 骰点区 3702~4402 亮像素 + 无 WARN。\n教训：证据管线的判据必须区分「帧非空」与「帧有内容」；间歇性失败先分冷/热缓存再怀疑数据链。

**2026-10-29 补（同题第二根因）：**「非空 且 非纯色帧」判据仍拦不住**半成品帧**——
冷启动（二进制刚重建/VM 杀毒逐文件扫描）时资产服务器只加载了**部分**贴图，截到的帧
部分资产上屏、部分缺席（CTA 色是中间态、中心像素发黑），既非纯色也非完整，内容判据
照单全收。治本 = 
un_shot 建装时把 ssets/ui/(65 png, 28MB) + ssets/fonts/
全量 load_untyped 预载，shot_preload_poll 逐帧轮询 LoadState 全部 Loaded
后才放行 
equest_shot（实测 VM 冷启动 ~290 帧就绪；热身循环上限 60→1800 帧，
原 60 帧内只加载了 31/66 就到循环上限 = readback 永远空 + write_shot_png 提前
return 留下**上一轮的陈旧 PNG**，像素检查「看起来对」其实是旧文件——验证据必须
对 mtime）。

### EXP-128 构建：把独立子 workspace 并回根 workspace 会引爆两条隐藏依赖冲突（bevy 子 crate 版本漂移 + `links=alsa` 互斥）
- **现象**（S27.7，用户要求「crates 内项目统一由根 `Cargo.toml` 管理」）：把 `crates/ti-engine` 并入根 workspace 后，`cargo metadata` 连续三次解析失败：
  ① `failed to select a version for bevy_clipboard ... required by bevy_text v0.20.1 ... previously selected bevy_clipboard v0.20.0`；
  ② 同型错误在钉 `bevy = "=0.20.0"` 后依旧（`bevy_internal 0.20.0 → bevy_dev_tools ^0.20.0 → bevy_text 0.20.1 → bevy_clipboard ^0.20.1`）；
  ③ `failed to select a version for quad-alsa-sys ... package quad-alsa-sys links to the native library alsa, but it conflicts with a previous package which links to alsa as well: alsa-sys v0.3.1 ← bevy_audio ← bevy_dev_tools ← bevy_internal`。
- **根因**：
  ① **版本漂移**：子 workspace 有自己的 `Cargo.lock`（钉住 bevy 全家 0.20.0）；并回根后根 lock 无 bevy 条目 ⇒ cargo 重新解析时给 `bevy_text`/`bevy_dev_tools` 选了 0.20.1，而 `bevy_internal 0.20.0` 要 `bevy_clipboard ^0.20.0` ⇒ 同族版本互斥。
  ② **`links=alsa` 互斥（历史冲突复现）**：bevy 0.20 的 `bevy_internal` 把 `bevy_dev_tools`（→ `bevy_audio` → `rodio` → `cpal` → `alsa` → **`alsa-sys`，links=alsa**）声明为**可选依赖**，Cargo 的解析图**包含未启用的可选依赖** ⇒ alsa 链恒在图内；而 `ti-app` 的 `android-shell = [..., "macroquad/audio"]` 让 `macroquad` 的 `quad-snd` → **`quad-alsa-sys`，links=alsa** 也进图 ⇒ 两个 `links=alsa` 的包同图 ⇒ 解析期直接报错（与 S18.1 当年拆子 workspace 的原因同源）。
- **解法**：
  ① bevy **精确钉版** `bevy = { version = "=0.20.0", ... }`（根 `[workspace.dependencies]`），并把子 workspace 的旧 `Cargo.lock`（钉住 bevy 全家 0.20.0）作为根 lock 的起点迁移 ⇒ 版本族一致；
  ② **去掉图内第二条 alsa 链**：`ti-app` 的 `android-shell` feature 去掉 `"macroquad/audio"`（`quad-snd` 不再进图），`android_shell.rs` 改用**本地音频桩**（`Sound`/`PlaySoundParams`/`load_sound_from_bytes`/`play_sound` 的空实现）⇒ 口径同 S18.8/S30.0「发货无声」，android 壳待 S18.8d 退役；
  ③ 子 workspace 的 `[workspace]` 表删除 + 改 `.workspace` 继承；`[profile.release] strip` 迁到根（子 workspace 的 profile 会被 Cargo 忽略）；脚本/文档命令统一 `-p ti-engine`；wasm 产物路径从 `crates/ti-engine/target/` 改根 `target/`。
- **验证**：`cargo metadata`（根，全依赖解析）exit 0；根 `Cargo.lock` 含 `alsa-sys`（bevy 侧）且**无** `quad-alsa-sys`；bevy 全家 = 0.20.0。
- **教训**：
  1. **子 workspace 的 `Cargo.lock` 是隐形依赖契约**——并回根前先把它的版本钉迁移过去，否则重新解析必炸（版本漂移）；
  2. **`links` 冲突按「解析图」算，不按「实际编译」算**——可选依赖里的原生库也算数 ⇒ 合并前先查两条原生库链（`cargo tree` 找 `links`），冲突方只能留一条；
  3. 合并后**产物路径、profile、脚本命令三处都会变**（`crates/x/target/` → 根 `target/`；子 profile 失效），必须同步改脚本与文档，否则门禁/发布链静默取不到产物。
  4. 写文件到仓库别用 PowerShell 的 `>` 重定向（PS 5.1 默认写 UTF-16 ⇒ cargo 报 `stream did not contain valid UTF-8`）；用 `cmd /c "git show ... > file"` 保持原始字节（EXP-125 同族）。

### EXP-129 构建/工具：换机器（Windows→Linux）续工三条坑（CRLF 归一化 + 无头 GPU + pwsh 命令差异）
- **现象**（S27.7，用户准备迁 Linux 续工，要求总结在途工作进文档）：
  ① 仓库 Windows 侧 `core.autocrlf=true` 且**无 `.gitattributes`**；Linux 侧 `git clone` 会按 Linux 默认把 CRLF 文件归一化成 LF ⇒ `doccheck2` 的 BOM/CRLF/UTF-8 门禁误报（`assets/data/strings.json` 等是 CRLF）；
  ② `--shot`（离屏出图）依赖 wgpu GPU 上下文，Linux 无显示器 = 无上下文 ⇒ 出图失败/黑图；
  ③ 全项目脚本是 `.ps1`（`gate.ps1`/`playbook.ps1`/`smoke.ps1`/`build_wasm.ps1`/`ui_preview.ps1`），Linux 无 `powershell.exe`，且部分脚本含 Windows 专属段（IDA/REA、`D:\`、`cmd /c`）。
- **解法**：
  ① 迁移二选一：**整目录拷贝**（含 `.git` + 未跟踪文件，字节原样零丢失）或 **Linux 设 `git config core.autocrlf input`** / 补 `.gitattributes`（`* text=auto`）+ `git add --renormalize .`；迁移后先跑 `pwsh -File scripts/gate.ps1` 验证；
  ② `--shot` 用 `xvfb-run -s "-screen 0 1280x800x24" ...` + mesa 软件渲染（lavapipe），或确保有 X/Wayland 会话；`s43_fps_smoke` 需真窗口（无 GPU 可豁免）；
  ③ Linux 用 **`pwsh -File scripts/X.ps1`**（无 `-ExecutionPolicy Bypass` 概念——那是 Windows 专属）；逆向相关段（IDA/REA）跳过；路径分隔 `/`。
- **教训**：
  1. **未跟踪（?）文件是迁移最容易丢的部分**——在途工作常是 tracked-M + untracked-? 混合，只 `git add` 已跟踪文件再 clone 会丢新文件（如 `vs_proto.rs`、`vs_v1.json`、新资产 PNG、渲染图）；迁移前先 `git status --porcelain` 甄别，要么整目录拷贝要么 `git add -A`；
  2. **`core.autocrlf` 是跨机器隐形契约**——无 `.gitattributes` 时换机器必然 CRLF/LF 漂移，触发的门禁误报会被误判成内容回归；
  3. **工具命令的「Windows 专属参数」要按平台剔除**（`-ExecutionPolicy`、`cmd /c`、反斜杠路径），换机器后先跑一条全门禁再开工。



### EXP-130 构建/工具：wasm 浏览器 e2e 三条坑（PointerHits 消费饿死 hover map + 快速 click 同帧丢 PointerClick + WebGL canvas 不能 2D drawImage 读像素）
- **现象**（S31.3，用户报「选图点不动/选完地图不能玩」）：
  ① bevy_ui 行鼠标点击在浏览器不生效，反复加调试系统后仍「点不动」，删了调试系统反而通了；
  ② 点「未解锁」的 m02/m03 卡片没反应，误判为命中 bug，实为 `map_unlocked` 正确拒绝（设计行为）；
  ③ Playwright 快速 `mouse.click` 偶发不触发 `PointerClick`；WebGL canvas 用 2D `drawImage`+`getImageData` 读像素恒 0（全黑误报）。
- **根因**：
  ① **任何 `MessageReader<PointerHits>` 系统（PreUpdate/Update 调试探针）会消费掉 `PointerHits` 消息 ⇒ bevy 自己的 `build_over_map`（hover map 构建）收不到命中 ⇒ `PointerClick` 永不发**。bevy_picking 的事件链 = `ui_picking` 后端产 `PointerHits` → `generate_hovermap` 消费建 `HoverMap` → `pointer_events` 发 `PointerPress/Click/Release`；调试系统插队读消息 = 饿死官方管线。诊断只能挂**观察者**（`On<PointerClick>` 等事件侧），不能读中间消息；
  ② 初始档仅 m01 解锁（`ProgressionState::initial_unlocked()`），选图卡 index 1/2 = m02/m03 锁拒 → `play_sfx(Invalid)` 无场景变化，观感 = 「点不动」；
  ③ `PointerClick` = press 帧 + release 帧**同一实体**；快速 click 两动作可能同帧内被合并/丢帧 ⇒ 无 Click。bevy_ui 0.20 的 `PointerClick` 还带 `auto_propagate`（沿祖先后代传播），一物理点击可发多个 Click（行节点 + 祖先 + window 实体）——命中解析层（`pick_flush`）按被点实体查 `PickTarget` 天然去重，**不需要在观察者层去重**；
  ④ WebGL canvas（`preserveDrawingBuffer=false`）在 `requestAnimationFrame` 外读 2D 像素 = 全黑。e2e 判据采样一律走 **Playwright `page.screenshot()` + PIL**，不用 `canvas.getContext('2d')` 桥读。
- **解法**：
  ① 删调试系统/观察者后必须**重跑浏览器实探**再下结论（`eprintln!` 在 wasm 走 `console.error`，headless 探针可捕获；`bevy LogPlugin level=WARN` 只滤 log crate，`eprintln` 不受影响）；
  ② e2e L4 判据 = 「select_map 亮像素比 < 40%（≈17.7%）→ 鼠标点 m01（右侧面板顶行，540×960 视口 x≈430 y≈100）→ 亮像素比 > 40%（main_map ≈78%）」增量断言；点击用 `move → wait 1s → down → wait 500ms → up` 分帧慢点；
  ③ Playwright 浏览器缺省自动探测 `~/.cache/ms-playwright/chromium-*/`（node-playwright 与 python-playwright 共享缓存目录），`TI_CHROMIUM_PATH` 显式覆盖；
  ④ 「选图点不动」类报告先分诊：点的是哪张卡（index）× 该卡解锁态（`unlocked_maps`），再查命中链（`UiPickingPlugin` 拾取行 `Node` → `PickTarget` → `PickEvent` → `poll_hit`）。
- **验证**：`python3 scripts/wasm_browser_e2e.py --check`（L4 before=17.7% after=78.1% PASS）+ gate 第 22 项 `wasm-browser-e2e`（无 Playwright 环境 = 记名 SKIP）+ `docs/migration_linux.md` §3 浏览器 e2e 依赖行（playwright + pillow + chromium 探测路径）。


### EXP-131 构建/工具：gate.ps1 Linux 化两条坑（cargo 不在默认 PATH + $env:PATH 拼接 ParserError）
- **现象**（S31.3，Linux 续工跑 `pwsh -File scripts/gate.ps1`）：
  ① 第 1 项 `cargo fmt --check` FAIL，报错「The term 'cargo' is not recognized」——本机 cargo 装在 `~/.cargo/bin`，
    Linux 默认 shell PATH 不含它，而 gate.ps1 全文直呼 `& cargo ...`（Windows 侧靠 rustup 初始化已入 PATH，从不暴露）；
  ② 第一次修补时写 `$env:PATH = "$cargoBin:$env:PATH"` 触发 pwsh **ParserError**：`':' was not followed by a valid
    variable name character`——PowerShell 字符串插值里 `$cargoBin:` 的 `:` 被解析成作用域分隔符（`$scope:name` 语法）。
- **根因**：
  ① gate.ps1 的脚本项已经做了跨平台映射（`$py`：Windows=python / Linux=python3，见脚本头注释 19 行），
    但 **cargo 从未做过同等映射**——WASM target 装好 ≠ cargo 在 PATH（`rustup target add` 只写 registry）；
    换机器后第一个踩的就是「文档里写 `export PATH` 才能跑 gate」的隐性前提；
  ② PowerShell 变量引用 `$x:y` 中 `:` 是**作用域前缀**（`$env:PATH`、`$script:fail` 合法），
    裸变量名后跟 `:` + 非合法作用域词 = 直接 ParserError（不是运行时错，是整个脚本解析失败）；
    字符串里要拼 `$var:` 必须 `${var}:` 花括号定界。
- **解法**（gate.ps1 已落码，Windows 行为不变）：
  ① 脚本头加 PATH 补全段：`$env:HOME` 存在且 `~/.cargo/bin` 在位时前置进 PATH（用 `"${cargoBin}:$($env:PATH)"`
    拼接避坑）；随后所有 `& cargo` 改 `& $cargoCmd`（Linux 解析 `~/.cargo/bin/cargo` 绝对路径，Windows 走 PATH 的 `cargo`）；
  ② 凡在双引号字符串里「变量名 + 冒号」组合，一律 `${var}` 花括号定界；
  ③ 改完先 `pwsh -Command "[scriptblock]::Create((Get-Content -Raw scripts/gate.ps1))"` 做纯解析自检（0 执行成本），
    再真跑 `-Fast`。
- **验证**：不带 `export PATH` 的干净 shell 里 `pwsh -File scripts/gate.ps1 -Fast` 全项 PASS（含第 1 项 cargo fmt）；
  机器口径 `flywheel_readme.ps1 -Check` gate 29 项与 doccheck2 46 断言对账 0 漂移。
- **教训**：gate 脚本的「环境前提」（PATH/依赖二进制）必须**脚本内自举**，不靠调用者 export——
  门禁是给下一位（下一个会话/下一台机器）跑的东西，隐性前提 = 下次 FAIL 归因浪费一小时。


### EXP-132 构建/工具：e2e 场景导航禁用 Back 循环（select_role 的 Back 无行为 ⇒ 死锁），Enter 链 + 点击重试
- **现象**（S31.3，gate full 档内 e2e L4 偶发 FAIL）：L4 用「Back 循环退回 select_map」定位场景，
  首次单跑 PASS（before=17.7%），gate 内复跑 FAIL（before=10.8%，Back×3 后仍卡在 main_map，点击无从谈起）。
- **根因**：`select_role` 场景的 `on_input` 无 `Back` 分支（只能 Enter/Next/Prev），而 `main_map` 的
  `Back` = 回 select_map ⇒ 若 L3 阶段（Enter+Space+click）已推进到 main_map，Back 循环第一次就退回
  select_role 系（视 menu_selected 持久态），后续 Back 全无效 ⇒ 循环 3 次耗尽仍在 main_map（≈78% 或
  中间态），`before_bright < 40` 判据永远不成立。L4 的「定位」步骤自己引入了状态歧义。
- **解法**（`scripts/wasm_browser_e2e.py` 已落码）：
  ① 定位一律走 **Enter 链**（index→select_role→select_map 每步 Confirm 幂等：select_role/select_map 的
     Confirm 在已选态 = 原地确认不推进，无歧义），替代 Back 循环；
  ② 点击 m01 卡片加**一次重试**（偶发丢 Click 兜底，两次不中 = 判据 FAIL 不静默放行）；
  ③ e2e 判据的「场景定位」步骤必须用**幂等前进**，禁止依赖「从未知态后退到已知态」。
- **验证**：修复后连跑 3 次 `python3 scripts/wasm_browser_e2e.py --check` 全 PASS（before=17.7% after=78.1%）；
  gate 第 22 项 `wasm-browser-e2e` 复跑 PASS。
- **教训**：headless 自动化里的场景导航，「前进链 + 幂等确认」比「后退链」稳——
  后退语义各场景不一致（有的无 Back、有的 Back 目标不定），前进链的 Confirm 语义全场景收敛。
