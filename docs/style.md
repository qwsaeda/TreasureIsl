# 工程规范（Style Guide）

本文件是 Rust 代码的强制规范。与本文件冲突的写法一律返工。选型已定案：**Rust edition 2024 + macroquad**，禁止重新讨论。

## §5.1 Workspace 布局（内聚：一 crate 一职责，依赖单向）

```
treasure-island/
├── Cargo.toml              # [workspace] edition="2024"，[workspace.lints] 集中
├── rustfmt.toml            # 全局唯一格式化配置
├── clippy.toml
├── scripts/gate.sh         # 门禁（parity.md 8 项）
├── crates/
│   ├── ti-formats/         # 纯二进制格式解析/序列化：pak/lpk/pcx/map/font/save
│   │   └── src/{pak.rs, pcx.rs, map.rs, font.rs, save.rs, macros.rs, error.rs, ids.rs}
│   ├── ti-core/            # 纯玩法逻辑（无 IO、无平台依赖，可跑 golden test）
│   │   └── src/{consts.rs, scene.rs, rules/, save/, rng.rs, oracle/}
│   ├── ti-platform/        # 平台抽象 trait + macroquad 后端 + Headless 测试后端
│   │   └── src/{gfx.rs, input.rs, audio.rs, storage.rs, backend.rs}
│   ├── ti-app/             # 组装层 + 三端入口 bin
│   └── ti-tools/           # 资源逆向工具 bins：unpak/map2json/pcx2png/preview/build_assets/verify
├── ref/                    # IDA 伪代码、functions.csv、classify.json、imports.json、event_ids.json
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
| ID | `ti-formats/src/ids.rs` | **由 build.rs 从 `ref/event_ids.json` 等生成**，禁止手写重复 ID |
| 数值表 | `assets_out/*.json` 资源化，引擎只读表 | 兵种/货物/BOSS 档（50000~300000）等 |

规则：禁止无名字的裸字面量（>2 且非测试代码，gate 第 5 项辅助 + 评审）；常量 `SCREAMING_SNAKE_CASE`；同一数字出现两处即提常量。

## §5.4 宏消除重复（`ti-formats/src/macros.rs`，全部 `macro_rules!`）

| 宏 | 展开产物 | 服务的标准 |
|---|---|---|
| `define_format!(字段表)` | `read`（带 offset 追踪 + 末尾断言 `consumed==len`）/ `write`（与 read 同源顺序）/ `Debug` 差分 / `roundtrip_all!` 测试 | P0-2 0 未知字节、P0-3 roundtrip 由构造保证 |
| `define_id!` | newtype ID + `From<u32>/Display/FromStr` + 关联常量 | 消除重复 ID 定义 |
| `define_scene!` | 场景样板 + 事件分发表注册（12 个场景零样板） | P3-1 事件覆盖 |
| `diff_test!` | oracle vs 正式实现同种子对拍 harness | P2 双通道差分 |
| `parse_err!` | 带 `offset/field/path` 上下文的解析错误 | 错误处理集中 |

过程宏暂不引入；确需时先在 `docs/experience.md` 记录理由再改 §5.1。

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

## §5.7 测试组织

- 单元测试：同文件 `#[cfg(test)] mod tests`；集成测试：`tests/`；对拍类：`tests/diff_*.rs`。
- 平台相关一律用 `HeadlessBackend`，CI 不依赖窗口/声卡。
- 新增格式必须同时新增 `roundtrip_all!` 用例（否则 P0 验收缺项）。

## §5.8 命名与文档

- 每个 `pub` 项有 `///`（`missing_docs = warn`）；跨 crate 只在定义处出现一次类型，禁止复制结构体。
- crate 内 `pub use` 汇聚到 `prelude`；app 层统一 `use ti_core::prelude::*;`。
- 模块内文件按域划分：一个格式一个文件，一系统一个目录（`rules/`、`oracle/`）。

## §5.9 依赖策略

- 新增依赖必须：纯 Rust（wasm/Android 双端可编）、维护活跃、无传递膨胀；引入前在任务 evidence 里记录候选对比。
- 已定：`macroquad`（渲染/输入/音频）、`thiserror`（错误）、`serde`+`serde_json`（JSON）、`flate2`（zlib）、`png`（资源导出）。其余按需走 §5.9 流程。
- 禁止 `unsafe`；平台后端确需时单独 feature 解封 + `// SAFETY:` 注释，并先记 `docs/experience.md`。
