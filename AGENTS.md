# AGENTS.md — 路由表

项目：Windows Mobile 游戏《金银岛 TreasureIsl》(ARM 原生) 的**逆向 + Rust 现代化移植**，目标 WASM(浏览器) + Android 双端。
背景计划不在此处，按任务路由读文档，**不要凭默认习惯开工**。

## 当前状态

- 根目录：`TreasureIsl.exe`（ARM PE32）、`TreasureIsl.exe.i64`（IDA Professional 9.4，已自动分析、**无自定义命名**）、`data/`、`Cargo.toml`（5 crate workspace）、`scripts/`、`docs/`、`plan/ledger.yaml`、`evidence/`、`ref/`。
- 工程已 bootstrap：`crates/{ti-formats,ti-core,ti-platform,ti-app,ti-tools}`；进度只由 `plan/ledger.yaml` 驱动。

## 路由表

> 全库索引见 `docs/README.md`。本表只列「开工前必读」的入口。

| 你要做的事 | 读 |
|---|---|
| **任何开工动作之前** | `docs/loop.md` + `plan/ledger.yaml` + `docs/README.md` |
| 判断「做完了没有」/ 验收 | `docs/parity.md` |
| 写 Rust（布局/常量/错误/宏/trait/风格/测试位置） | `docs/style.md` |
| 逆向 exe 或 `data/` 格式 | `docs/facts/index.md`（含已实测结论，先读再动手） |
| 找当前阶段任务与验收命令 | `docs/plan-s0-s2.md`、`docs/plan-s3.md`、`docs/plan-s4.md` |
| 避免重复踩坑 | `docs/experience/index.md` ← **每次踩坑后必须回写** |
| 容器格式（pak/lpk）字段与偏移 | `docs/formats/lucencypackage.md` |
| 某玩法模块的行为规格 | `docs/spec/<module>.md`（模块清单见 `docs/README.md`） |
| **要实现/改玩法逻辑（实现级规格）** | `docs/game_design.md` → `docs/design/<system>.md`（不含 IDA 地址） |
| 现代化改造 / 移植 / 多端适配 | `docs/modernization.md` |
| Android 壳落地 + Mac 出 APK（S5.5） | `android/README.md`（cargo-apk 脚手架 + 资源拷贝 + 编译/装机步骤） |
| 查数值（价格表/BOSS HP/RNG 常数/上限） | `docs/design/constants.md` |
| 引入新依赖 / 改 workspace 结构 | `docs/style.md` §5.1、§5.9 |

## 硬规则（违反 = 返工）

1. **不许留占位**：`TODO/FIXME/unimplemented!/todo!/panic!` 及库代码中的 `unwrap/expect` 一律为 0 命中（`tests/` 除外），门禁在 `scripts/gate.ps1`。
2. **验收才算完成**：任务的验收命令**实跑输出**写入 `evidence/<task>.log` 后，才在 `plan/ledger.yaml` 置 `verified`。⛔ **只贴 gate 总输出不算验收证据** —— `doccheck` 会检查 evidence 是否包含任务自己的判据（第 9 项）。
3. **进度只由台账驱动**：循环 = 取下一个非 `verified` 任务 → 实现 → 验收 → `scripts/gate.ps1` → 更新台账 → 终止条件见 `docs/loop.md`（推进至 S4.4）。
4. **结论必须实测**：二进制/格式相关结论写进 `docs/facts/<分卷>.md` 前必须有脚本证据；推测一律标注 `推测`。
5. **选型已定案**：Rust edition 2024 + macroquad（见 `docs/style.md`）。不要重新讨论语言/框架。
6. **文档拆分 + 单一事实源**：细节进 `docs/`，本文件只做路由与硬规则。**同一结论只写一处**，其他文档引用 ID（`F-xxx`/`EXP-xxx`/`P0-x`）而非复述。新增文档必须登记进 `docs/README.md` 与本路由表。ID 唯一性与跨文档引用有效性由 `doccheck` 门禁（第 9 项）自动校验。

## 命令

```sh
# 每轮循环的门禁（9 项，全绿才可把任务置 verified，输出 tee 到 evidence/<id>.log）
powershell -ExecutionPolicy Bypass -File scripts/gate.ps1

# 单项调试
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test  --workspace
cargo run -q -p ti-tools --bin ledger  -- plan/ledger.yaml   # 台账校验（gate 第 8 项）
cargo run -q -p ti-tools --bin doccheck                        # 文档一致性（gate 第 9 项）

# S2.5：构建浏览器产物到 web/（wasm + miniquad gl.js + 真实资源）
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
# 本地预览（wasm 需 HTTP，不能 file://）
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080

# S2.7：native + wasm 两端冒烟（本机无 bash，务必用 ps1）
powershell -ExecutionPolicy Bypass -File scripts/smoke.ps1

# S1.4b：控件记录全量导出（20 UI 场景 → ref/controls.json，44 种 f5 控件类型）
cargo run -q -p ti-tools --bin controls -- --out ref

# spec 骨架半自动生成（模板化逆向→开发：函数清单表 + 逐函数带行号证据行表，人工只补「语义」列）
cargo run -q -p ti-tools --bin specskel -- <module>              # 打印到 stdout
cargo run -q -p ti-tools --bin specskel -- <module> --out PATH   # 覆盖已有 spec 文件
cargo run -q -p ti-tools --bin specskel -- <module> --report     # 只打印统计摘要
# <module> = ref/functions_<module>.csv 的 <module>（main_loop/render/menu/selectrole/selectmap/
# mainmap/trade/build/battle/cards/saveload/messages/rng）；自动识别两种列格式
# （常规 addr|address,size,name 与 RNG 专用 kind,site,func）

# 反编译导出（S0.2）；IDA 9.x 把 7.5 的 idat64.exe 合并为 idat.exe（32/64 同一程序）
"D:\soft\IDA Professional 9.4\idat.exe" -A -Lref\ida_run.log -Sscripts\ida_dump.py TreasureIsl.exe
```

- 构建范围随阶段解锁：native（已解锁）→ wasm（**S2.5 已解锁**）→ android（S2.6 在 **Mac** 装 NDK/SDK + cargo-apk 编译 APK，本机 Windows 不产出，gate SKIP）；gate 按已装 target 自动判定并打印 SKIP。
- **桌面/浏览器可玩为当前收口目标**（S5.1–S5.4）：菜单→选人→选图→主地图→贸易/建造→战斗→存读档全链路可玩，P3-3 全流程 + P4-1 渲染对齐已机器判定；Android 为后续移动端交付，开发本身不依赖 Android 工具链。
- S5.5 Android 壳已落地（`android-shell` feature + `treasure-android` bin + `android/README.md`）；APK 实际产出在 Mac，本机 host 形态可编译校验。

## REA 逆向 MCP（2026-10 接入）

- 注册：OpenCode `mcp.rea`（`~/.config/opencode/opencode.json`），`environment` 带 `REA_ANALYSIS_PROVIDER=ida` + `REA_IDA_MCP_CONFIG=C:\Users\Administrator\.rea\ida-mcp.json`；headless supervisor = `D:\ProgramData\miniconda3\Scripts\idalib-mcp.exe`（IDA 9.4 + idapro 0.0.9 + ida-pro-mcp 2.0.0）。
- **三层定位**（F-319 波动性可用 + F-321 结构化能力 + EXP-041 分层修正）：
  - ① **批量/离线主路径**（不可替代）：`ref/pseudocode/*.c` grep（S0.2 已全量导出 1827 份）+ `scripts/pe_rva.py` + `idat.exe -A -S`；
  - ② **REA 结构化能力**（补洞，未利用）：CFG/调用图/类型恢复/值追踪/重建验证 —— 冒烟优先，失败即退主路径，见 F-321 + `docs/style.md` §5.11.4；
  - ③ **稳定性**：F-319 波动性可用（S3.2 实测可用 EXP-039 / S3.12 实测失败 EXP-036），逐工具实测表见 `docs/facts/tools.md`。