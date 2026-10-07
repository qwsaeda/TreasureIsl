# AGENTS.md — 路由表

项目：Windows Mobile 游戏《金银岛 TreasureIsl》(ARM 原生) 的**逆向 + Rust 现代化移植**，目标 WASM(浏览器) + Android 双端。
背景计划不在此处，按任务路由读文档，**不要凭默认习惯开工**。

## 当前状态（bootstrap 前）

- 根目录只有：`TreasureIsl.exe`（ARM PE32）、`TreasureIsl.exe.i64`（IDA 7.5 数据库，已自动分析、**无自定义命名**）、`data/`（原始资源）、本文件与 `docs/`。
- **尚无 Cargo 工程、无构建脚本、无 CI**。S0.1 完成后必须回本文件把「命令」一节替换为真实命令。

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

1. **不许留占位**：`TODO/FIXME/unimplemented!/todo!/panic!` 及库代码中的 `unwrap/expect` 一律为 0 命中（测试代码除外），门禁在 `scripts/gate.sh`。
2. **验收才算完成**：任务的验收命令跑过、日志写入 `evidence/<task>.log` 后，才在 `plan/ledger.yaml` 置 `verified`。卡住 3 轮 → 拆子任务，禁止带病标绿。
3. **进度只由台账驱动**：循环 = 取下一个非 `verified` 任务 → 实现 → 验收 → `scripts/gate.sh` → 更新台账 → 终止条件见 `docs/loop.md`（推进至 S4.4）。
4. **结论必须实测**：二进制/格式相关结论写进 `docs/reverse-facts.md` 前必须有脚本证据；推测一律标注 `推测`。
5. **选型已定案**：Rust edition 2024 + macroquad（见 `docs/style.md`）。不要重新讨论语言/框架。
6. **文档拆分**：细节进 `docs/`，本文件只做路由与硬规则；新增文档必须登记进路由表。

## 命令

- bootstrap 前：无可运行命令。
- S0.1 之后在此登记真实命令，固定顺序：

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace
scripts/gate.sh        # 上述三项 + 三 target 构建 + 占位扫描 + 格式校验，全绿才算过
```
