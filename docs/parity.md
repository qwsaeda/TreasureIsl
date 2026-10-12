# 对齐完成标准（Parity Standard）

「和原游戏对齐」的唯一判据。分 5 级，每级由**机器可判定的命令**验收；任何一级不过 = 未完成。所有报告存 `evidence/`。

## P0 资源字节对齐

| # | 判据 | 验收命令（S0.1 后落地为真实命令） |
|---|---|---|
| P0-1 | 每个容器（`ui.pak`/`glpics.pak`/`glmap_m0*.pak`/`*.lpk`）条目提取率 100% | `cargo run -p ti-tools -- unpack --check-all` → extracted == total |
| P0-2 | 每个格式解析器 **0 未知字节**（文件 100% 字段被定义，否则报错） | `cargo run -p ti-tools -- verify --zero-unknown` → 全格式 pass |
| P0-3 | 解包后**重打包逐字节一致** | `--repack` 输出 sha256 == 原文件 sha256 |
| P0-4 | PCX 解码像素与**独立参考实现**逐像素一致 | `cargo test -p ti-formats --test pcx_parity` |
| P0-5（S10.1 新增）| 触摸命中与渲染坐标系一致（主地图等距投影逆变换） | `cargo test -p ti-app --test touch_render_same_coords`：点按棋子 → 实际移动方向一致 |
| P0-6（S10.2 新增）| HD 递归加载 100% 注入（`load_hd_pack_from_dir` 递归子目录） | `cargo test -p ti-app --test hd_inject_full`：注入计数 == `assets_hd_pack` 全量 PNG |

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
| P4-1 | 关键场景渲染 == 资源解码参考图，逐像素一致 | `cargo test -p ti-app --test render_parity`（用 `HeadlessBackend` 渲染，对比 P0 参考图）；S10.4 起 GPU 路径基准全重生成（联动 S7.7 解锁），legacy CPU blit 基准保留 |
| P4-2 | 176×220 逻辑坐标命中判定在任意整数/非整数缩放下不偏移 | `cargo test -p ti-platform -- test coordinate_roundtrip` |
| P4-3 | 中文点阵字形与 `GoldLand.Fnt`/`font.dat` 逐位图一致 | `cargo test -p ti-formats --test font_parity` |
| P4-4（S10.4 新增）| GPU 渲染观感对齐（sprite batch + alpha 混合 + TTF 字形无豆腐块） | `cargo test -p ti-app --test render_parity_gpu`：关键场景截图对比 + TTF 14 缺口字渲染非空心方框 |

## 代码质量门禁 `scripts/gate.ps1`（16 项，循环每轮强制）

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
      （F-/EXP- ID 唯一性与跨文档引用、任务 ID 登记、数字断言、evidence 内容特征；
       第 9 项内含跨文档数值断言（2041/UI 场景 20/0x343FD/20011591）+
       release 台账计数漂移检查（check_ledger_counts））
10 数据表门禁（S8.2，见 docs/process/04_tooling.md §2）：
     python scripts/validate_data.py            # assets/data/*.json 结构与数值域，须全绿
     python scripts/gen_layout.py --check       # controls.json 幂等一致，须全绿
11 oracle 边界（S8.8）：python scripts/check_oracle_boundary.py --check
     （shell 层 0 个 oracle_* 函数调用；oracle 仅 rules 内部参照物，类型重导出允许）
12 check_hardcode 严格（第 10 项组内 10c，单独计第 12 项）：python scripts/check_hardcode.py
     （发现硬编码即 FAIL；2026-10 已清零 37 处债务：玩家可见中文全进 strings.json、
      布局色全走 COLOR_* 常量；诊断/CLI 经 EXEMPT 豁免）
13 doccheck2（双 agent 审计固化断言，第 1~5 轮审计高频项）：
      cargo run -q -p ti-tools --bin doccheck2 .
      （文档↔代码/数据一致性 6 项：audio ChannelState API 一致性 / shell 层 oracle_* 残留 /
       LCG hex 误算（0x3443D）/ 据点阈值反向断言（<= 40）/ i18n 孤儿 key / release 门禁项数漂移；
       替代每轮双 agent 审计里靠子代理现场 grep 的重复检查，见 docs/experience/build.md）
14 台账三方对账（S10.0，防子代理虚报，EXP-058）：
       python scripts/check_ledger_reality.py
       （遍历 ledger status: verified 任务行，校验 (a) evidence 文件存在且非空；
        (b) DoD 声明的关键代码符号在 crates/*/src grep 命中；缺任一 FAIL「台账虚报」。
        机器化拦截「子代理报 verified 但代码产物不存在」（S9.5 commands.rs 教训））
15 业务闭环断言（S11，docs/plan-s11-forward.md §1 闭环判据 1~4）：
       python scripts/check_forward_closed.py --check
       （C1 业务文档零逆向引用（legacy/ 出处标注豁免）/ C2 表现层 0 oracle /
        C3 根目录清空原版文件 + legacy 归档在位 / C4 oracle 桥接层与归档同源）
16 模块并行前置（S11.4，docs/process/05_module_parallel.md §4 并行纪律 1/2）：
       python scripts/check_module_contracts.py --check
       （M1 包白名单两两互斥（plan/packages.yaml）/ M2 契约校验锚点在 crates/ grep 到 /
        M3 ti-app 跨 crate 可见面只走契约模块（domain/rules/data）/ M4 draft 残留 WARN）
```

任一项红灯 = 本轮不可标 verified，不可推进下一任务。

## 完成定义（DoD）

`P0~P4 全 pass`（含 P0-5/P0-6/P4-4 S10 新增判据）+ `gate 14 项全绿` + `docs/plan-s4.md` 的 S4.4 产物齐全（桌面 exe / wasm 站点 / APK）+ ledger 全 `verified`。

**S10 对抗流水线停机条件**（S10.5，`docs/plan-s10-refactor.md`）：
审计 0 新发现（`rea-audit-fix` 双 agent）+ P3-3 全流程 16 步 0 失败 + gate 14 项全绿。
