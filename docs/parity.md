# 对齐完成标准（Parity Standard）

> **S18.7b：像素基线机制已退役（不再存在）**。
> 依旧的 `assets/render_baseline/` / `forward_baseline/` / `hd_baseline/` 三套 golden PNG、
> `render_snapshot` / `render_snapshot_forward` / `visual_regression` 三个 diff 测试、`visual_baseline.ps1`
> 与其门禁项、`baseline-mirror-identity` 校验均**已全部删除**。
> 现行机制：**结构不变式断言**（`tests/render_invariants.rs`，7 场景 × 非黑覆盖率/内容签名互异/暖色板在位/TTF 在位）
> 与 **CPU↔Bevy 内存对拍**（`bevy-cpu-parity` 门禁项：GPU 真实离屏读回 vs CPU 期望逐像素对照）。
> 本文中仍有的「基线」词句为**历史记述**，不再是现行机制。

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
2. `legacy/docs/spec/<module>.md` — 行为规格（输入/状态/输出/数值公式）；
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
|  | P0-5（S10.1 新增） | 触摸命中与渲染坐标系一致（主地图等距投影逆变换） | `cargo test -p ti-app --test render_invariants` |  |

## 代码质量门禁 `scripts/gate.ps1`（22 项，循环每轮强制）

> **计数约定（本节是门禁项数的唯一事实源，其他文档只引用、不各自重数）**
>
> 「项」= `gate.ps1` 中**顶格（第 0 列）**的 `Report` 调用数。三条口径：
> 1. 只数顶格 `Report`；**缩进在 `if`/`else` 块内的条件 `Report`/`Skip` 不计入**——
>    即 wasm 构建、android 构建、`zero-unknown verify`（`if (Test-Path ...)` 守卫）三项；
> 2. `Skip` 不单独计项；
> 3. 当前顶格 `Report` = **22 项**。
>
> **机器校验**：该口径由 `scripts/flywheel_readme.ps1` 的 `Read-GateItems` 实现，
> 并由 gate 第 13 项 `doccheck2` 的 `check_gate_item_actual_count` 持续对账（gate 第 17 项
> `flywheel-readme -Check` 亦打印同一数字）。**数字对不上时以机器输出为准**，不要手工重数。
>
> **易错点（勿再踩）**：
> - `gate.ps1` 的**注释编号**（1..20 + 19.5 + 4b，含第 7 项 zero-unknown）与**机器计数**
>   不一定相等：zero-unknown 是条件项不计；`10a`/`10b`/`10c` 各算 1 项。引用「项数」
>   一律用机器口径 23。
> - `10a`/`10b`/`10c` 是 **3 个独立顶格 `Report`，各算 1 项**，不是「三合一算 1 项」。
> - 历史计划文档里的 8/10/14/16/18/20 项均为**该阶段时点快照**，不代表当前；引用时须带时点标注。
> - **`required-features` 的 bin 不被第 1~3 项编译/lint 覆盖**——第 4b 项专门补这个盲区（EXP-099）。
>
> 增删门禁项时**只改本节**，并在同一次改动里同步更新编号与计数。

```
1  cargo fmt --all -- --check
2  cargo clippy --workspace --all-targets -- -D warnings
3  cargo test  --workspace
4  构建（范围随阶段解锁，未解锁的行在脚本里跳过并打印 SKIP）：
     cargo build --workspace --target x86_64-pc-windows-msvc          # S0.1 起
     cargo build --workspace --target wasm32-unknown-unknown          # S2.5 解锁（需先 rustup target add）
     cargo build -p ti-app --target aarch64-linux-android             # S2.6 解锁（需 NDK）
4b feature 门控 bin 编译 + lint（EXP-099：`required-features` 的 bin 不被 1~3 项覆盖）：
     cargo clippy --workspace --all-targets \
         --features ti-app/native-shell,ti-app/headless -- -D warnings
     （编译并 lint 全部 `required-features = ["native-shell"/"headless"]` 的 bin：
      treasure-native / treasure-headless / element-dump / sprite-step / render-png。
      这 5 个 bin 是**官方排障工具**，历史上因 1~3 项不带 feature 而长期不被编译验证，
      导致 element-dump 编译失败长期静默；wasm / android bin 由 S2.5/S2.6 的 target
      构建行覆盖，不在本项范围）
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
      python scripts/validate_data.py            # 10a 统一入口（FIND-201 合并）：balance/strings/scenes/controls
                                                #     基础校验 + V1 main_map.grid 三键 + V2 全场景 extra 标量正整数，须全绿
      python legacy/scripts/gen_layout.py --check # 10b controls.json 幂等一致，须全绿
      python scripts/check_hardcode.py           # 10c 严格模式，发现硬编码即 FAIL；2026-10 已清零 37 处债务：
                                                #     玩家可见中文全进 strings.json、布局色全走 COLOR_* 常量；
                                                #     诊断/CLI 经 EXEMPT 豁免
      （历史：10c 曾单列为「第 12 项」，已并入本组；第 12 项编号作废不再复用）
 11 oracle 边界（S8.8）：python scripts/check_oracle_boundary.py --check
     （shell 层 0 个 oracle_* 函数调用；oracle 仅 rules 内部参照物，类型重导出允许）
 13 doccheck2（双 agent 审计固化断言，第 1~6 轮审计高频项）：
       cargo run -q -p ti-tools --bin doccheck2 .
       （文档↔代码/数据一致性 19 项：audio ChannelState API 一致性 / shell 层 oracle_* 残留 /
        LCG hex 误算（0x3443D）/ 据点阈值反向断言（<= 40）/ i18n 孤儿 key / gate 项数漂移 /
        headless BattleView API / 飞轮回读 / 战斗玩家数值数据驱动 / 飞轮脚本 /
        触摸渲染正逆成对 / 场景 band 数 / 场景布局保真 / HD 精灵透明 /
        map_slot 写档 0 起 zero_index 基准（FIND-203）/ exhaust 双写对账 /
        经验库分卷条数 / 运行时 legacy 坐标禁 / 基线镜像恒等；
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
 17 飞轮开轮必读脚本（EXP-070）：
        powershell -ExecutionPolicy Bypass -File scripts/flywheel_readme.ps1 -Check -Quiet
        （脚本存在 + 输出含 6 条必读 + gate ≥16 项 + doccheck2 ≥8 项，exit 0 = PASS）
 18 视觉回归基准图（EXP-072）：
19.5 正向渲染快照门禁（S14，与 19 并列而非从属，docs/business/08_layout_design.md §4）：
 20 数据源门禁（S13.6 契约 §5）：
        python scripts/check_data_source.py
        （legacy/data/*.Map|*.pak 相对 HEAD 有改动 → 强制跑第 19 项；无改动时 SKIP）
  21 UI 场景叙事约束（v3，docs/ui-redesign-v3/02_spec_lang.md）：
        python scripts/check_scene_narrative.py
        （scene_manifest_v3.json N1-N8：Schema 校验 / mood 三元一致 / 重心占屏 ≥20% /
         节奏权重和 =1.0 + 重心 beat 首位且 ≥0.30 / beats 2-4 / HUD 上限 / 死区上限 /
         scenes.json focus_anchor 对账（未落地键 SKIP，v3.1 起全量））
   ```

任一项红灯 = 本轮不可标 verified，不可推进下一任务。

## 完成定义（DoD）

`P0~P4 全 pass`（含 P0-5/P0-6/P4-4 S10 新增判据）+ `gate 22 项全绿` + `docs/plan-s4.md` 的 S4.4 产物齐全（桌面 exe / wasm 站点 / APK）+ ledger 全 `verified`。

**S10 对抗流水线停机条件**（S10.5，`docs/plan-s10-refactor.md`）：
审计 0 新发现（`rea-audit-fix` 双 agent）+ P3-3 全流程 16 步 0 失败 + gate 22 项全绿。
