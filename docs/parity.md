# 对齐完成标准（Parity Standard）

「和原游戏对齐」的唯一判据。分 5 级，每级由**机器可判定的命令**验收；任何一级不过 = 未完成。所有报告存 `evidence/`。

## P0 资源字节对齐

| # | 判据 | 验收命令（S0.1 后落地为真实命令） |
|---|---|---|
| P0-1 | 每个容器（`ui.pak`/`glpics.pak`/`glmap_m0*.pak`/`*.lpk`）条目提取率 100% | `cargo run -p ti-tools -- unpack --check-all` → extracted == total |
| P0-2 | 每个格式解析器 **0 未知字节**（文件 100% 字段被定义，否则报错） | `cargo run -p ti-tools -- verify --zero-unknown` → 全格式 pass |
| P0-3 | 解包后**重打包逐字节一致** | `--repack` 输出 sha256 == 原文件 sha256 |
| P0-4 | PCX 解码像素与**独立参考实现**逐像素一致 | `cargo test -p ti-formats --test pcx_parity` |

## P1 数据对齐

| # | 判据 | 验收 |
|---|---|---|
| P1-1 | 全部记录 100% 解析为 JSON，JSON→二进制 roundtrip 无损 | `define_format!` 生成的 `roundtrip_all!` 测试全绿 |
| P1-2 | 逆出的 `name→key` 哈希函数（djb2 种子 0 + 小写）对**可由资源名解释的容器条目**命中率 100%（实测 85/85，`glpics.pak`），ID 型 key 另立 S1.11 | `cargo test -p ti-tools --test pak_key` |
| P1-3 | 每个 `.Map` 控件记录 6 字段全定义 + 全量提取（f0..f5） | `cargo run -p ti-tools --bin controls -- --out ref` → `ref/controls.json` 记录数 == 控件总数（2041）；20/20 UI 场景整除吻合；字段语义见 F-311（f3 = 屏幕 Y 坐标，非事件 id） |

## P2 逻辑对齐（双通道差分）

每个玩法模块必须 4 件套齐全：

1. `ref/pseudocode/<func>.c` — Hex-Rays 伪代码归档；
2. `docs/spec/<module>.md` — 行为规格（输入/状态/输出/数值公式）；
3. `ti-core` 中的正式实现（实现 `trait Rules` 等，见 `docs/style.md` §5.6）；
4. **oracle 差分测试**：把伪代码**直译**成 `ti-core/src/oracle/<module>.rs`（测试专用、允许与正式实现重复），同一 RNG 种子、同一输入 → 输出完全相同。

```
diff_test!(oracle::battle::resolve, rules::battle::resolve, cases)
→ cargo test -p ti-core --test diff_<module>  全绿
```

## P3 行为对齐

| # | 判据 | 验收 |
|---|---|---|
| P3-1 | 44 种 f5 控件类型码逐档解释 + 事件语义完整（事件语义 S3.3 起，含菜单→控件映射） | `cargo run -p ti-tools --bin controls -- --out ref` 无 skipped（地形除外）；**2041 条 + 20/20 整除已由 P1-3 验收，本项不重复** |
| P3-2 | 每个场景可进入/返回（场景图全覆盖） | `coverage scenes` → missing = 0 |
| P3-3 | 固定种子全流程脚本跑通：开局→选人→选图→移动→贸易→建造→战斗→存档→读档 | `cargo run -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json`（seed=42，16 步断言 0 失败） |

## P4 视觉/操作对齐

| # | 判据 | 验收 |
|---|---|---|
| P4-1 | 关键场景渲染 == 资源解码参考图，逐像素一致 | `cargo test -p ti-app --test render_parity`（用 `HeadlessBackend` 渲染，对比 P0 参考图） |
| P4-2 | 176×220 逻辑坐标命中判定在任意整数/非整数缩放下不偏移 | `cargo test -p ti-platform -- test coordinate_roundtrip` |
| P4-3 | 中文点阵字形与 `GoldLand.Fnt`/`font.dat` 逐位图一致 | `cargo test -p ti-formats --test font_parity` |

## 代码质量门禁 `scripts/gate.ps1`（11 项，循环每轮强制）

```
1  cargo fmt --all -- --check
2  cargo clippy --workspace --all-targets -- -D warnings
3  cargo test  --workspace
4  构建（范围随阶段解锁，未解锁的行在脚本里跳过并打印 SKIP）：
   cargo build --workspace --target x86_64-pc-windows-msvc          # S0.1 起
   cargo build --workspace --target wasm32-unknown-unknown          # S2.5 解锁（需先 rustup target add）
   cargo build -p ti-app --target aarch64-linux-android             # S2.6 解锁（需 NDK）
5  占位扫描：Select-String -Path crates/**/*.rs -Pattern 'TODO|FIXME|unimplemented!|todo!|panic!\(' → 0 命中
6  库代码 unwrap 扫描：同上 Pattern '\.unwrap\(\)|\.expect\('，排除 `tests/` 目录 → 0 命中
   （内联 `#[cfg(test)]` 模块同样禁止 unwrap；本机无 ripgrep，扫描一律用
    PowerShell Select-String，见 EXP-011）
7  cargo run -q -p ti-tools --bin verify -- --zero-unknown   # S1 起解锁，之前 SKIP
8  台账校验：`cargo run -q -p ti-tools --bin ledger -- plan/ledger.yaml`
   （解析 `docs/loop.md` §1 schema：状态/依赖/rounds/verified 必须有 evidence 文件）
9  文档一致性：`cargo run -q -p ti-tools --bin doccheck`
    （F-/EXP- ID 唯一性与跨文档引用、任务 ID 登记、数字断言、evidence 内容特征）
9b 跨文档数值漂移 + release 台账计数同步：`cargo run -q -p ti-tools --bin doccheck2`
    （第 1-3 轮对抗审计固化：权威值 2041/UI 场景 20 + release.md 任务计数与 ledger 实测一致）
10 数据表门禁（S8.2，见 docs/process/04_tooling.md §2）：
   python scripts/validate_data.py            # assets/data/*.json 结构与数值域，须全绿
   python scripts/gen_layout.py --check       # controls.json 幂等一致，须全绿
   python scripts/check_hardcode.py --warn    # 硬编码扫描（迁移期只报告，清零后改 FAIL）
```

任一项红灯 = 本轮不可标 verified，不可推进下一任务。

## 完成定义（DoD）

`P0~P4 全 pass` + `gate 10 项全绿` + `docs/plan-s4.md` 的 S4.4 产物齐全（桌面 exe / wasm 站点 / APK）+ ledger 全 `verified`。
