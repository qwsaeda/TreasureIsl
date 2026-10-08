# AGENTS.md — 路由表

项目：Windows Mobile 游戏《金银岛 TreasureIsl》(ARM 原生) 的**逆向 + Rust 现代化移植**，目标 WASM(浏览器) + Android 双端。
背景计划不在此处，按任务路由读文档，**不要凭默认习惯开工**。

## 当前状态

- 根目录：`TreasureIsl.exe`（ARM PE32）、`TreasureIsl.exe.i64`（IDA Professional 9.4 数据库，已自动分析、**无自定义命名**）、`data/`（原始资源）、`Cargo.toml`（5 crate workspace）、`scripts/`、`docs/`、`plan/ledger.yaml`。
- 工程已 bootstrap：`crates/{ti-formats,ti-core,ti-platform,ti-app,ti-tools}`，阶段进度看台账（`plan/ledger.yaml`）。

## 路由表

| 你要做的事 | 读 |
|---|---|
| 开工前 / 任何改动前 | `docs/loop.md` + `plan/ledger.yaml` |
| 判断「做完了没有」/ 验收 | `docs/parity.md` |
| 写 Rust（布局/常量/错误/宏/trait/风格） | `docs/style.md` |
| 逆向 exe 或 `data/` 格式 | `docs/reverse-facts.md`（含已实测结论，先读再动手） |
| 找当前阶段任务与验收命令 | `docs/plan-s0-s2.md`、`docs/plan-s3.md`、`docs/plan-s4.md` |
| 避免重复踩坑 | `docs/experience.md` ← **每次踩坑后必须回写** |
| 引入新依赖 / 改 workspace 结构 | `docs/style.md` §5.1、§5.9 |

## 硬规则（违反 = 返工）

1. **不许留占位**：`TODO/FIXME/unimplemented!/todo!/panic!` 及库代码中的 `unwrap/expect` 一律为 0 命中（测试代码除外），门禁在 `scripts/gate.ps1`。
2. **验收才算完成**：任务的验收命令跑过、日志写入 `evidence/<task>.log` 后，才在 `plan/ledger.yaml` 置 `verified`。卡住 3 轮 → 拆子任务，禁止带病标绿。
3. **进度只由台账驱动**：循环 = 取下一个非 `verified` 任务 → 实现 → 验收 → `scripts/gate.ps1` → 更新台账 → 终止条件见 `docs/loop.md`（推进至 S4.4）。
4. **结论必须实测**：二进制/格式相关结论写进 `docs/reverse-facts.md` 前必须有脚本证据；推测一律标注 `推测`。
5. **选型已定案**：Rust edition 2024 + macroquad（见 `docs/style.md`）。不要重新讨论语言/框架。
6. **文档拆分**：细节进 `docs/`，本文件只做路由与硬规则；新增文档必须登记进路由表。

## 命令

```sh
# 每轮循环的门禁（8 项，全绿才可把任务置 verified，输出 tee 到 evidence/<id>.log）
powershell -ExecutionPolicy Bypass -File scripts/gate.ps1

# 单项调试
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace
cargo test  -p ti-tools                       # 台账解析器单测
cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml   # 台账校验（= gate 第 8 项）

# 反编译导出（S0.2）；IDA 9.x 把 7.5 的 idat64.exe 合并为 idat.exe（32/64 同一程序）
"D:\soft\IDA Professional 9.4\idat.exe" -A -Lref\ida_run.log -Sscripts\ida_dump.py TreasureIsl.exe
```

- 构建范围随阶段解锁：native（已解锁）→ wasm（**S2.5 已解锁**，target 已装，产物走 `scripts/build_wasm.ps1`）→ android（S2.6 装 SDK/NDK）；gate 自动按已装 target 判定并打印 SKIP。

```sh
# S2.5：构建浏览器产物到 web/（wasm + miniquad gl.js + 真实资源）
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1

# 本地预览（wasm 需 HTTP，不能 file://）
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080

# S1.4b：控件记录全量导出（20 UI 场景 → ref/controls.json，44 种 f5 控件类型）
cargo run -q -p ti-tools --bin controls -- --out ref
```

## REA 逆向 MCP（2026-10 接入，见 docs/reverse-facts.md F-313/F-314）

- 注册：OpenCode `mcp.rea`（`~/.config/opencode/opencode.json`），`environment` 带 `REA_ANALYSIS_PROVIDER=ida` + `REA_IDA_MCP_CONFIG=C:\Users\Administrator\.rea\ida-mcp.json`；headless supervisor = `D:\ProgramData\miniconda3\Scripts\idalib-mcp.exe`（IDA 9.4 + idapro 0.0.9 + ida-pro-mcp 2.0.0）。
- 深度分析走 IDA（不需要 Hopper/Ghidra）。`rea doctor --provider ida` 应 healthy。
- 已知缺陷（F-314）：无 Evidence 的工具会撞 `-32602`，改本地脚本；`callers` 恒空，调用方靠 `ref/pseudocode/` grep。
