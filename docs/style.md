# 工程规范（Style Guide）

本文件是 Rust 代码的强制规范。与本文件冲突的写法一律返工。选型已定案：**Rust edition 2024 + macroquad**，禁止重新讨论。

## §5.1 Workspace 布局（内聚：一 crate 一职责，依赖单向）

```
treasure-island/
├── Cargo.toml              # [workspace] edition="2024"，[workspace.lints] 集中
├── rustfmt.toml            # 全局唯一格式化配置
├── clippy.toml
├── scripts/gate.ps1        # 门禁（parity.md 22 项，PowerShell 本机执行）
├── crates/
│   ├── ti-formats/         # 纯二进制格式解析/序列化：pak/lpk/pcx/map/font/save
│   │   └── src/{pak.rs, pcx.rs, map.rs, font.rs, save.rs, macros.rs, error.rs, ids.rs}
│   ├── ti-core/            # 纯玩法逻辑（无 IO、无平台依赖，可跑 golden test）
│   │   └── src/{consts.rs, scene.rs, rules/, save/, rng.rs, oracle/}
│   ├── ti-platform/        # 平台抽象 trait + macroquad 后端 + Headless 测试后端
│   │   └── src/{gfx.rs, input.rs, audio.rs, storage.rs, backend.rs}
│   ├── ti-app/             # 组装层 + 三端入口 bin
│   └── ti-tools/           # 资源逆向工具 bins：unpak/map2json/pcx2png/preview/build_assets/verify
├── ref/                    # IDA 伪代码、functions.csv、classify.json、imports.json、controls.json（S1.4b）
├── docs/                   # 规范与计划（路由见 AGENTS.md）
├── plan/ledger.yaml        # 循环台账
└── evidence/               # 每任务验收日志
```

依赖方向（禁止反向）：`ti-tools → ti-formats`；`ti-app → {ti-core, ti-platform, ti-formats}`；`ti-core → ti-formats`；`ti-platform` 不依赖 core。

## §5.2 Edition 与 lint（workspace 集中）

```toml
[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
all = { level = "warn" }
pedantic = { level = "warn" }      # 个别 allow 必须逐条写理由注释
unwrap_used = "deny"               # 库代码禁 unwrap/expect（tests 除外）
panic = "deny"                     # 禁 panic，错误走 Result 链
```

每个 crate：`[lints] workspace = true`。格式只由 `rustfmt.toml` 决定（`edition="2024"`, `max_width=100`, `group_imports="Module"`, `imports_granularity="Module"`），CI 跑 `cargo fmt --check`。

## §5.3 常量集中（分域集中，源码禁魔数）

| 层级 | 位置 | 内容 |
|---|---|---|
| 全局 | `ti-core/src/consts.rs`（带分区注释） | 逻辑分辨率 `LOGICAL_W/H=176/220`、RGB565、缩放档、帧率、存档版本 |
| 格式 | `ti-formats/src/<fmt>/consts.rs` | 头偏移、魔数、记录尺寸、标志位 |
| ID | `ti-formats/src/ids.rs` | 控件类型码 `ControlKind`（数据源 `ref/controls.json`，S1.4b）；其余 ID 域随各自任务落地，禁止手写重复 ID |
| 数值表 | `assets_out/*.json` 资源化，引擎只读表 | 兵种/货物/BOSS 档（50000~300000）等 |

规则：禁止无名字的裸字面量（>2 且非测试代码，gate 第 5 项辅助 + 评审）；常量 `SCREAMING_SNAKE_CASE`；同一数字出现两处即提常量。

## §5.4 宏消除重复（`ti-formats/src/macros.rs`，全部 `macro_rules!`）

**已落地**（`crates/ti-formats/src/macros.rs`，逐条对应源码里的 `macro_rules!`）：

| 宏 | 展开产物 | 服务的标准 |
|---|---|---|
| `define_format!(字段表)` | `read`（带 offset 追踪 + 末尾断言 `consumed==len`）/ `write`（与 read 同源顺序）/ `Debug` 差分 / `roundtrip_all!` 测试 | P0-2 0 未知字节、P0-3 roundtrip 由构造保证 |
| `parse_err!` | 带 `offset/field/path` 上下文的解析错误 | 错误处理集中 |
| `roundtrip_all!` | 由 `entries` 重建 → 字节级对比的往返测试用例 | P0-3 |

**规划中（尚未落地，禁止在本文件描述其产物形状）**：

| 宏 | 落地任务 | 用途 |
|---|---|---|
| `define_scene!` | S3.1 之后补 | 场景样板 + 事件分发表注册（20 个 UI 场景 `.Map` 零样板，见 F-301）；P3-1 事件覆盖 |
| `define_id!` | 随对应 ID 域任务 | newtype ID + `From<u32>/Display/FromStr` + 关联常量；消除重复 ID 定义 |
| `diff_test!` | 已由各模块手写 `tests/diff_*.rs` 取代 | oracle vs 正式实现同种子对拍；P2 双通道差分 |

> 门禁第 9 项 `doccheck` 会校验本表「已落地」段的每个宏名都能在 `macros.rs` 里 grep 到；把未实现的宏写进「已落地」段即判失败。

过程宏暂不引入；确需时先在 `docs/experience/index.md` 记录理由再改 §5.1。

## §5.5 错误处理集中（每 crate 一个 Error，顶层聚合）

```rust
// 每 crate 的 error.rs（统一模板）
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{path:?}@{offset:#x}: {kind}")]
    Parse { path: PathBuf, offset: usize, kind: ParseKind },
    #[error("io: {0}")] Io(#[from] std::io::Error),
}
pub type Result<T> = std::result::Result<T, Error>;

// ti-app/src/error.rs 聚合
pub enum Error { Formats(#[from] ti_formats::Error), Core(#[from] ti_core::Error), Platform(#[from] ti_platform::Error) }
```

- 全链路 `?` + `From`；库内无 unwrap/panic（lint 强制）；`main() -> Result<()>`，唯一打印处在入口。
- 解析错误**必须**带 `offset + 字段名 + 路径`（由 `define_format!` 生成，不手写）。

## §5.6 Trait 设计（依赖注入 + 单态化，测试可替换）

```rust
// ti-platform：后端注入，泛型不 dyn
pub trait Backend { type Gfx: Gfx; type Input: Input; type Audio: Audio; type Storage: Storage; type Clock: Clock; }
trait Gfx     { fn clear(&mut self, c: Rgb565); fn blit(&mut self, r: Rect, src: &[u16]); fn present(&mut self); }
trait Input   { fn poll(&mut self, out: &mut Vec<Event>); }   // 触摸/键/拖拽统一事件
trait Audio   { fn play(&mut self, id: SoundId, vol: u8) -> Channel; fn stop(&mut self, ch: Channel); }
trait Storage { fn read(&mut self, k: &str) -> Result<Vec<u8>>; fn write(&mut self, k: &str, v: &[u8]) -> Result<()>; } // 桌面文件 / wasm IndexedDB / android 内部存储
trait Clock   { fn now_ms(&mut self) -> u64; }
// 实现：MacroquadBackend（三端）、HeadlessBackend（测试/回归）

// ti-core：玩法可注入、可对拍
pub trait Scene  { fn enter(&mut self, ctx: &mut Ctx); fn on_event(&mut self, e: Event) -> Flow; fn tick(&mut self, ctx: &mut Ctx); fn draw(&self, fb: &mut Framebuf); }
pub trait Rng    { fn next_u32(&mut self) -> u32; }                    // 注入确定性 RNG
pub trait Rules  { /* 贸易/战斗结算入口 */ }                          // 正式实现与 oracle 同 trait → diff_test!
pub trait SaveCodec { fn encode(&self) -> Vec<u8>; }                   // v1/v2 各一实现
```

### 5.6.1 接口隔离原则（SceneHost 拆分）

**强制约束**：`SceneHost` 按职责拆分为窄接口，场景只依赖它需要的接口：

| 窄接口 | 职责 | 方法数 |
|---|---|---|
| `SceneControl` | 场景切换 + 输入路由 | 3 |
| `RenderHost` | 帧缓冲/字形/精灵/地形 | 7 |
| `EconomyHost` | 金/价格/持有/银行/建筑 | ~12 |
| `BattleHost` | 战斗推进/快照 | 2 |
| `SelectionHost` | 角色/地图选择状态 | ~6 |
| `MessageHost` | 消息分页 | ~6 |
| `SaveHost` | 存读档 | 3 |
| `MapHost` | 主地图状态 | ~6 |

**规则**：
1. 场景 struct 只声明它需要的窄接口（如 `IndexScene` 只依赖 `SceneControl` + `RenderHost`）；
2. 新增方法必须归入已有窄接口，不得随意加到 `SceneControl`；
3. 窄接口方法保持**一行读写**，不含业务判定。

### 5.6.2 Builder 模式（复杂对象构造）

**强制约束**：`GameApp` 等复杂对象必须用 Builder 模式构造：

```rust
let app = GameApp::builder()
    .with_scale_mode(ScaleMode::Smooth)
    .with_orientation(Orientation::Portrait)
    .with_filter_mode(FilterMode::Nearest)
    .with_hd_assets(true)
    .build()?;
```

**规则**：
1. 字段数 > 3 的 struct 必须用 Builder；
2. Builder 方法返回 `Self`，支持链式调用；
3. `build()` 返回 `Result<Self, Error>`，校验必填字段。

### 5.6.3 Type-State 模式（场景转换编译期安全）

**强制约束**：场景转换用 Type-State 保证编译期安全：

```rust
pub struct SceneTransition<From, To> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
}

// 只有合法转换才能编译
impl SceneTransition<SelectMap, MainMap> {
    pub fn new() -> Self { /* ... */ }
}
```

**规则**：
1. 场景转换必须用 Type-State 或编译期检查；
2. 非法转换（如 `Intro → Vs`）必须在编译期报错。

### 5.6.4 Command 模式（输入可序列化/重放）

**强制约束**：输入用 Command 模式封装：

```rust
pub trait Command: Serialize + DeserializeOwned {
    fn execute(&self, app: &mut GameApp) -> bool;
    fn undo(&self, app: &mut GameApp) -> bool;
}
```

**规则**：
1. 输入事件封装为 Command 对象；
2. Command 可序列化（支持重放/回放）；
3. 支持 `undo`（用于测试/回滚）。

### 5.6.5 GPU 渲染约定（S10.4，逐精灵 batch + 纹理图集 + alpha 混合 + TTF）

**强制约束**（S10 起，`docs/modernization.md` §1 修订）：

| 规则 | 内容 |
|---|---|
| 默认渲染路径 | GPU 化：macroquad `draw_texture` 逐精灵 + 纹理图集（`TextureAtlas`）+ alpha 混合（`BlendFactor::SrcAlpha/DstAlpha`），不再走 176×220 CPU 逐像素 blit |
| legacy 回退 | 176×220 RGB565 CPU blit 保留为 feature `legacy-render`（P4-1 历史基准对比 + 回退），**不再是默认** |
| 纹理来源 | PCX/HD PNG 解码 → macroquad `Texture`（一次上传，逐帧 `draw_texture` 复用），禁每帧重传 |
| alpha 混合 | HD PNG 透明通道走 alpha blend（`SrcAlpha/DstAlpha`），禁直接丢 A（`png_rgba_to_framebuf_u32` 旧路径仅 legacy 用） |
| TTF 字形 | `macroquad::load_ttf` 加载子集 TTF（Noto Sans CJK 子集），替代 `font.dat` 14 缺口字符（继续/胜负!/页 等豆腐块）；原版点阵字保留为可选「怀旧字体」模式 |
| 新增场景 | 场景 render 走「GPU 精灵 pass + 文字 pass」，禁新增逐像素 CPU blit 路径 |

**性能取向**：纹理图集 batch（减少 draw call 数）+ 静止帧跳过（脏标记）；目标 60fps（`stats` feature 实跑验证）。

## §5.7 测试组织

- **一律放集成测试目录 `crates/<crate>/tests/`**，文件名对应被测模块（`pak.rs` → `tests/pak.rs`）。⛔ **禁止**在 `src/` 内写 `#[cfg(test)] mod tests` —— 门禁第 6 项的 unwrap/expect 扫描**只豁免 `tests/` 目录**，内联模块会被命中，且 `panic = deny` 也会拦住（见 EXP-021 坑 3）。
- 对拍类：`tests/diff_<module>.rs`；资源管线类：`tests/<crate>_<主题>.rs`。
- 测试内**可以用** `.expect("常量字面量")`，但 `.expect(&format!(...))` 触发 `expect_fun_call`（见 EXP-027）。
- 平台相关一律用 `HeadlessBackend`，CI 不依赖窗口/声卡。
- 新增格式必须同时新增 roundtrip 用例（否则 P0-2 验收缺项）。

## §5.8 命名与文档

- 每个 `pub` 项有 `///`（`missing_docs = warn`）；跨 crate 只在定义处出现一次类型，禁止复制结构体。
- crate 内 `pub use` 汇聚到 `prelude`；app 层统一 `use ti_core::prelude::*;`。
- 模块内文件按域划分：一个格式一个文件，一系统一个目录（`rules/`、`oracle/`）。

## §5.9 依赖策略

- 新增依赖必须：纯 Rust（wasm/Android 双端可编）、维护活跃、无传递膨胀；引入前在任务 evidence 里记录候选对比。
- 已定：`macroquad`（渲染/输入/音频）、`thiserror`（错误）、`serde`+`serde_json`（JSON）、`flate2`（zlib）、`png`（资源导出）。其余按需走 §5.9 流程。
- 禁止 `unsafe`；平台后端确需时单独 feature 解封 + `// SAFETY:` 注释，并先记 `docs/experience/index.md`。
- **大依赖默认 feature 门控（S4.4 决策，EXP-046）**：体积敏感的可选能力（如 `opus-pure` OGG/Opus 转码）在提供它的库 crate（`ti-platform`）里做成 **optional dep + 自定义 feature**（`ogg = ["dep:opus-pure"]`，**不进 default**），调用方按需开启。
  - `ti-tools`（mod2ogg / build_assets 离线转码）经 `ti-platform = { features = ["ogg"] }` 传递引入 opus-pure，**不**直接依赖 `opus-pure`（保持单一传递路径，`cargo tree -p ti-tools` 仅 `ti-tools → ti-platform → opus-pure`）。
  - 运行时壳（`ti-app` 桌面/wasm）默认**不含** opus-pure：在线转码走独立 feature `mod-ogg = ["ti-platform/ogg"]`（默认关闭），正常启动直接播放 `build_assets` 离线预转码的 OGG 产物（`assets_out/audio/*.ogg`）。
  - 判据：`cargo build -p ti-app --features wasm-shell --release` 的依赖树里**不得**出现 opus-pure；wasm 主体 ≤ 800 KB（`evidence/S4.3-size.log`，S4.4 复核 675,047 B）。

## §5.10 Clippy pedantic 规约（`-D warnings` 强制）

workspace `[workspace.lints]` 开了 `all` + `pedantic` + `panic = deny` + `unwrap_used = deny`，以下模式反复触发，写代码时直接规避（详见 EXP-015）：

| 模式 | 触发 lint | 规避写法 |
|---|---|---|
| `assert!(false, ...)` | `assertions_on_constants` | `assert!(matches!(ev, Pat { .. }))` + `if let Pat { .. } = ev { ... }` |
| `[T; N]` 栈数组 > 16 KB | `large_stack_arrays` | `Box<[T]>` = `vec![val; n].into_boxed_slice()` |
| `u32 as f32` / `i32 as f32` | `cast_precision_loss` | f32 字面量 `const LW: f32 = 176.0;` 或 `#[allow(clippy::cast_precision_loss)]` |
| `fn new() -> Self` 无 Default | `new_without_default` | 手动 `impl Default { fn default() -> Self { Self::new() } }` |
| 宏展开后变量未用 | `unused_variables` | `#[allow(unused_variables)] let x = ...;` |
| 返回值的 pub fn | `must_use_candidate` | 加 `#[must_use]` |

## §5.11 REA MCP 使用规约 + thunk/导入识别 + RNG oracle 规约（S3.12 实测，见 EXP-036/037、F-319/F-320）

### 5.11.1 REA 工具冒烟规约（强制）

接入或升级 MCP 后，**不能只测一两个工具就写「可用」结论**（F-313 的教训）。冒烟必须：

1. **逐工具实际调用**：列出计划用的全部工具，每个跑一次真实调用（不是看文档描述）；
2. **校验返回 JSON 通过 OpenCode schema**：`-32602` = 不可用，哪怕工具「能返回数据」也算失败（OpenCode 会拒绝）；
3. **描述性结论标 `推测`**：未实测输出的工具不许写「可用」，只能写「推测可用，待冒烟」；
4. **冒烟结果回写 `legacy/docs/facts/tools.md`**：F-313/F-319 模式，带「实测日期 + 工具清单 + 通过/失败」。

> **当前状态不在本文件复述**：逐工具可用性以 `legacy/docs/facts/tools.md` §6 的实测表为唯一事实源；本节只保留「怎么测」的规约。

**S3 模块 RE 开工前固定动作**：`open_binary` 绑 arm PE → `search_strings`（cursor 分页拿全 .rdata 字符串段）一次性取资源/场景/事件/数值清单作 spec 骨架，**禁止逐文件 grep 1827 份伪代码找资源名**（慢且易漏）。

**台账与 verified 复核规约（EXP-038）**：
- 子代理改 `plan/ledger.yaml` **只许用 edit 工具 oldString/newString 含目标任务行**，禁止整文件重写（会冲掉并行子代理刚置的 verified）；
- 主线程收子代理「已 verified」报告后，**独立复核三件事才认账**：(1) 该模块代码/测试文件 `Test-Path` 真实存在；(2) 实跑 `cargo test -p ti-core --test diff_<m>` 看 `test result: ok`；(3) 重读 ledger 该行 `status: verified` + `evidence` 非空。三者齐备才算 verified，缺任一就补写/回滚并重跑 gate。

### 5.11.2 thunk / 导入函数识别规约（逆向通用）

遇到 IDA 伪代码里 `void fn(args) { __imp_fn(args); }` 且函数体仅一行 `__imp_*` 调用 + size ≤ 16 字节 → **判定为 thunk → 导入函数**，算法在外部 DLL（coredll.dll / aygshell.dll 等），**不在 exe 内**。处理：

1. **停止挖 exe 内实现**（挖不到，浪费时间）；
2. 查平台 CRT / SDK 文档（WinCE = MSVC CRT，`rand`/`srand`/`memcpy` 等都有文档）；
3. oracle 按文档写，标 `推测`，等差分测试反证（见 5.11.3）；
4. 导入名清单见 `ref/imports.json`（S0.4 已恢复 116 个导入 API 名）。

### 5.11.3 RNG oracle 实现规约（S3.12 专用）

- **算法（推测）**：MSVC WinCE ARM LCG `seed = seed.wrapping_mul(214013).wrapping_add(2531011); ((seed >> 16) & 0x7fff) as i32`（见 F-320）；
- **oracle 结构**：`ti-core/src/oracle/rng.rs` 的 `pub struct RngState { seed: u32 }` + `pub fn srand(&mut self, s: u32)` + `pub fn rand(&mut self) -> i32`；
- **播种点不进 oracle**：毫秒时间种子不可复现，oracle 只负责「给定种子 → 确定序列」；正式实现 `rules::rng` 同理，播种由调用层（场景 ctor）注入；
- **差分用例**：固定种子（如 `srand(123)`）→ 取 1000 次 `rand()` → 与 oracle 序列逐位比对；若游戏侧能复现同种子序列则算法确认，否则 coredll 用了不同算法 → 找 WinCE coredll.dll 逆向（标 `blocked_by: coredll-rand-algorithm`）；
- **rand 调用点归类**：S3.12 验收要求 `ref/classify.json` 中 `pending=0`，所有 `rand()%N` 调用点必须归到对应模块（cards/trade/battle/mainmap 等），不许留 pending。

### 5.11.4 REA 结构化能力使用指引（2026-10 制定，见 F-321 / EXP-041）

REA 在本项目的角色 = **补结构化深度分析的洞**，非次级交叉校验。主路径（grep 1827 份 + pe_rva.py + idat -S，离线可复现）不可替代；但以下场景**应试 REA 而非 grep**（REA 有结构化能力，grep 没有）：

| 场景 | REA 工具 | grep 为什么不够 |
|---|---|---|
| 精确调用链 / 可达路径 | `get_call_graph` / `trace_call_path` | grep 只找调用点，不建图、不判可达性 |
| 控制流图（basic blocks + successors） | `analyze_function` | grep 伪代码靠人脑补跳转 |
| 数据流 def-use / 常量传播 | `trace_native_values` | grep 无法追踪值依赖 |
| 原生类型恢复（struct/enum/位域） | `inspect_native_data_type` | 资源结构体靠人肉数偏移 |
| API 边界（jump table dispatch/case 映射） | `inspect_native_api` | grep 找不到 dispatch 表 |
| 重建验证 / 对齐覆盖率（S4） | `verify_reconstruction` / `build_reconstruction_obligation_ledger` | 没有机制化对齐验收 |

**必走 grep 的场景**：批量 / 离线 / 需要可复现 / 1827 份全量扫描 / 跨函数统计。

**冒烟纪律**（5.11.1 已定，此处重申）：每次用 REA 前先 `open_binary` + 一个轻量调用（如 `search_strings`），通过才继续，失败即退主路径，不浪费回合数（F-319 波动性可用）。

**S4 重建验证探针**：S4.1 开始时花一回合验 `verify_reconstruction` / `evaluate_reconstruction_coverage` 能否吃我们的 evidence bundle —— 如果能，S4.4 的对齐验收就有了机制；如果不能，继续靠 P0~P4 判据 + gate。
