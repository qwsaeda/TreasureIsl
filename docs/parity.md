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
| P0-1 | 每个容器（`ui.pak`/`glpics.pak`/`glmap_m0*.pak`/`*.lpk`）条目提取率 100% | `cargo run -p ti-tools --bin unpak -- --check-all` → extracted == total |
| P0-2 | 每个格式解析器 **0 未知字节**（文件 100% 字段被定义，否则报错） | `cargo run -p ti-tools --bin verify -- --zero-unknown` → 全格式 pass（**S46.0 实测基线**：33 个真实文件 / 305 个容器条目 / 0 跳过 / 0 失败；口径 = 结构后无残留字节，`define_format!` 的 `read` 对残留报 `TrailingBytes` ⇒ 解析成功即达标；门禁第 7 项同步由永久 SKIP 转为默认 full 档实跑） |
| P0-3 | 解包后**重打包逐字节一致** | `--repack` 输出 sha256 == 原文件 sha256 |
| P0-4 | PCX 解码像素与**独立参考实现**逐像素一致 | `cargo test -p ti-tools --test pcx_ref`（Rust 解码器 vs `scripts/pcx_ref.py` 独立参考，`pcx_ref.rs` 3 用例） |
| P0-5（S10.1 新增）| 触摸命中与渲染坐标系一致（主地图等距投影逆变换） | 已并入 doccheck2 `check_touch_render_pair`（doccheck2 项；原 `touch_render_same_coords` 集成测试已退役） |
| P0-6（S10.2 新增）| HD 递归加载 100% 注入（`load_hd_pack_from_dir` 递归子目录） | `cargo test -p ti-app --test render_parity_hd`（HD RGBA8 渲染路径逐像素对齐，`hd-8bit` feature）；原 `hd_inject_full` 集成测试已退役 |

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
| P0-5（S10.1 新增） | 触摸命中与渲染坐标系一致（主地图等距投影逆变换） | `cargo test -p ti-app --test render_invariants` |

## P5 Android 真机交互（S49.4e 新增；2026-11-09 真机反馈「全屏/菜单无响应/后台音乐不停」根因审计补强）

> **背景**：P0-P4 全是桌面/解析器/格式判据，Android 仅「构建」条件项（本机 SKIP），真机专属
> 行为（全屏/触摸/音频生命周期）结构性测不到——「三端一致」是口号，Android 从未成为「能验收」
> 的一等公民（根因审计见 `docs/process/23_game_testing.md` §2 兼容类 + `docs/process/00_index.md` 变更面声明）。
> 本表把 Android 真机交互立为 P5 判据，载体 = `scripts/android_device_test.py`（adb 驱动，
> 设备接入即能跑；本机无设备 = gate 条件项记名 SKIP，不阻塞但文档化，不静默）。

| # | 判据 | 验收（`python3 scripts/android_device_test.py --run`，设备 + 包已装） |
|---|---|---|
| P5-1 | **全屏**：app surface 尺寸 ≈ 屏幕全尺寸（无系统栏压缩 → 游戏 letterbox 无黑条） | `--run --only p5-1`：`adb wm size` vs `dumpsys window` app bounds 差 ≤4px |
| P5-2 | **触摸命中**：菜单/场景 tap 触发流转（S49.4d 前 Android 触摸不产生 `MouseButton` → 1080 层命中全 miss） | `--run --only p5-2`：`input tap <tap-x> <tap-y>` → logcat `[hit]` 或截屏 md5 变化 |
| P5-3 | **音频生命周期**：切后台 BGM 暂停 / 回前台恢复（S49.4d `lifecycle_audio_system`） | `--run --only p5-3`：`KEYCODE_HOME` → logcat `BGM 暂停`；`am start` → `BGM 恢复` |
| P5-4 | **切后台不崩**：进程存活 + 无 FATAL/ANR | `--run --only p5-4`：`pidof` 在 + logcat 无 `FATAL`/`ANR in` |

> **环境检查**：`--check` 只读（adb 在位 + 设备已授权 + 包已装）；缺 = 非 0 退出（gate 条件项据此记名 SKIP）。
> **坐标标定**：首次设备接入 `--tap-x/--tap-y`（P5-2 菜单 tap 屏幕坐标；不同设备分辨率不同）。
> **多设备**：`--serial/-s <serial>` > `ANDROID_SERIAL` 环境变量 > 唯一设备；多设备未指定 = 歧义报错（红米/MuMu 与 redroid 可同时在线）。
> **门禁**：gate 第 4c 项（Android 真机）——有 adb 设备跑 P5 全判据，无设备记名 SKIP + 文档化（不静默）。
>
> **判据分级（2026-11-10 飞轮 r1 修正）**：P5-4 = 进程生命周期判据（任何 adb 设备可跑）；
> P5-1 = 窗口驻留依赖判据（app 窗口须驻留 `dumpsys window` 的 windows 段才能取 bounds；
> MuMu launcher 抢前台 + NativeActivity 出不了首帧 = 窗口不驻留 → 设备标定，需前台稳定设备）；
> P5-2/P5-3 = 渲染出帧依赖判据（触摸需截屏非黑 / BGM 需渲染管线起）。
> （触摸需截屏非黑 / BGM 需渲染管线起 ⇒ 仅 GPU 正常设备有效）。
> 无 GPU 设备（MuMu 转译层缺 `VK_KHR_swapchain_mutable_format`，EXP-181；redroid 纯 CPU pastel
> 软渲染 0fps，宿主 i915 Vulkan 1.1 不可用 / 无 virtio-gpu gfxstream，2026-11-10 三路径实测）上
> P5-1/P5-2/P5-3 会**无帧可判**：P5-2/P5-3 由脚本渲染链路健康前置检测（截屏 100% 黑 +
> swap chain Timeout ≥3）自动记 **SKIP（设备渲染限制，非 app bug，exit 0）**；P5-1 窗口
> 不驻留 dumpsys（NativeActivity 未画出首帧）→ 判据 FAIL（设备标定，需 GPU 正常设备重跑）；
> P5-4（进程生命周期）不依赖渲染出帧，照跑。
> **P5 达标口径**：P5-4 PASS + P5-1/P5-2/P5-3 在有 GPU 设备（arm64 真机）上 PASS；
> GPU 转译层设备（MuMu Adreno 640 伪装 + redroid 纯 CPU pastel）= P5-2/P5-3 SKIP 记名
> + P5-1 设备标定 FAIL（窗口不驻留）；P5-4 照跑，`evidence/android_device_test.log`
> 含设备 serial 可回溯（2026-11-10 MuMu 实测：P5-2 PASS / P5-4 PASS / P5-1 设备标定 / P5-3 SKIP）。

## 代码质量门禁 `scripts/gate.ps1`（36 项，循环每轮强制；S31.3 起含 wasm 浏览器端 e2e 第 22 项；S16.1 起含 browser-playtest 第 23 项；S16.2 起含 playtest-loop 第 24 项）

> **计数约定（本节是门禁项数的唯一事实源，其他文档只引用、不各自重数）**
>
> 「项」= `gate.ps1` 中**全部 `Report` 调用（含 `if`/`else` 条件块内缩进的 Report），按行内首个引号串 = 项名去重**（S26.3 修正口径；`Read-GateItems` 同实现）。三条口径：
> 1. **数全部 `Report`**（不只看顶格）——wasm 构建、android 构建、`zero-unknown verify`、
>    **Android 真机（第 4c 项）** 等条件块内 Report 也计项（它们都是真实门禁检查，只是环境依赖 SKIP）；
> 2. `Skip` 不单独计项（环境缺 = SKIP，判据 FAIL = Report FAIL）；
> 3. 当前全部 `Report` 去重 = **36 项**（S46.1 起 = 34 + 第 25 项 dod-commands；S49.4e 起 = 35 + 第 4c 项 Android 真机；数字对不上时以机器输出为准）。
>
> **机器校验**：该口径由 `scripts/flywheel_readme.ps1` 的 `Read-GateItems` 实现，
> 并由 gate 第 13 项 `doccheck2` 的 `check_gate_item_actual_count` 持续对账（gate 第 17 项
> `flywheel-readme -Check` 亦打印同一数字）。**数字对不上时以机器输出为准**，不要手工重数。
>
> **易错点（勿再踩）**：
> - `gate.ps1` 的**注释编号**（1..20 + 19.5 + 4b/4c，含第 7 项 zero-unknown）与**机器计数**
>   不一定相等：`10a`/`10b`/`10c` 各计一项。引用「项数」一律用机器口径 36（`gate.ps1` 实测 `Report` 去重数；数字对不上以机器输出为准，不手工重数）。
> - `10a`/`10b`/`10c` 是 **3 个独立 `Report`，各算 1 项**，不是「三合一算 1 项」。
> - 历史计划文档里的 8/10/14/16/18/20 项均为**该阶段时点快照**，不代表当前；引用时须带时点标注。
> - **`required-features` 的 bin 不被第 1~3 项编译/lint 覆盖**——第 4b 项专门补这个盲区（EXP-099）。
>
> 增删门禁项时**只改本节**，并在同一次改动里同步更新编号与计数。

> **DoD 判据命令可达性自检（S46.1，门禁第 25 项 `dod-commands`）**：P0~P4 表里的
> 判据命令由 `scripts/check_dod_commands.py --check` 机器校验——`cargo run -p <crate> --bin <bin>`
> 的 bin 须存在；多 bin crate 上的 `cargo run -p <crate> -- <arg>` **裸形式 = 歧义 FAIL**
> （EXP-170/171 实测：P0-1 判据 `-- unpack --check-all` 曾五年不可实跑）；`cargo test --test <name>`
> 与 `scripts/*.py|ps1` 载体须在位。防「验收集合 ≠ 判据集合」复发。

```
1  cargo fmt --all -- --check
2  cargo clippy --workspace --all-targets -- -D warnings
3  cargo test  --workspace
4  构建（范围随阶段解锁，未解锁的行在脚本里跳过并打印 SKIP）：
     cargo build --workspace --target x86_64-pc-windows-msvc          # S0.1 起
     cargo build --release -p ti-engine \
         --bin treasure-bevy-wasm --target wasm32-unknown-unknown     # S2.5 解锁（需先 rustup target add）
     cargo build -p ti-app --target aarch64-linux-android             # S2.6 解锁（需 NDK）
4b feature 门控 bin 编译 + lint（EXP-099：`required-features` 的 bin 不被 1~3 项覆盖）：
     cargo clippy --workspace --all-targets \
         --features ti-app/native-shell,ti-app/headless -- -D warnings
     （编译并 lint 全部 `required-features = ["native-shell"/"headless"]` 的 bin：
      treasure-headless / element-dump / sprite-step / render-png。
      这 4 个 bin 是**官方排障工具**，历史上因 1~3 项不带 feature 而长期不被编译验证，
      导致 element-dump 编译失败长期静默；wasm 交付壳 = ti-engine 的 Bevy 壳
      （`treasure-bevy-wasm`），由第 4 项 wasm 分支构建 + clippy 覆盖；
      android bin 仍不被任何行覆盖（EXP-107，随 S18.8d 退役））
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
        （脚本存在 + 输出含 7 条必读〔S51.0 起含开轮协作五步〕 + 本域红线 + gate ≥16 项 + doccheck2 ≥8 项，exit 0 = PASS）
 18 视觉回归基准图（EXP-072）：
19.5 正向渲染快照门禁（S14，与 19 并列而非从属，docs/business/08_layout_design.md §4）：
 20 数据源门禁（S13.6 契约 §5）：
        python scripts/check_data_source.py
        （legacy/data/*.Map|*.pak 相对 HEAD 有改动 → 强制跑第 19 项；无改动时 SKIP）
        （文件规模 >900 行 / 函数长度 >140 行 / 重复模式 ≥6 次；阈值与棘轮基线 = scripts/code_budget.json）
  22 代码结构门禁（S20.0c，docs/style.md §5.12）：
        python scripts/audit_code.py --check
  21 UI 场景叙事约束（v3，docs/ui-redesign-v3/02_spec_lang.md）：
        python scripts/check_scene_narrative.py
        （scene_manifest_v3.json N1-N8：Schema 校验 / mood 三元一致 / 重心占屏 ≥20% /
         节奏权重和 =1.0 + 重心 beat 首位且 ≥0.30 / beats 2-4 / HUD 上限 / 死区上限 /
         scenes.json focus_anchor 对账（未落地键 SKIP，v3.1 起全量））
   23 浏览器真运行时试玩（S16.1 web_playtest，Node + Playwright CDP 按键驱动）：
        node scripts/web_playtest.cjs --url http://127.0.0.1:8080/
        （L1 实例化活体 / L2 首帧墨点 / L3 场景切换帧有变化 / L4 console 致命错误 = 0；
         无 node playwright / serve 未跑 = 记名 SKIP，有前置且断言失败 = FAIL）
   24 试玩闭环 loop（S16.2，4 份运行时报告聚合，docs/process/08_playtest_loop.md）：
        python scripts/playtest_loop.py --check
        （fuzz_playtest / full_run / render_invariants / web_playtest 全 0 失败 = 本试戏轮无新缺陷；
         停机 = 连续 3 轮收敛入 evidence/S16.2.log + 主线程复核，EXP-038/058）
   25 DoD 判据命令载体可达性自检（S46.1，EXP-170/171 复发防护）：
        python scripts/check_dod_commands.py --check
        （文档判据命令 cargo run/test --bin/--test、scripts/*.py|ps1 载体全存在 +
         多 bin 歧义形式 0 处；「判据从未执行」类缺陷的门禁兜底）
   25b 元测试面（S50.2，CCGS studio-tests 模式借鉴，docs/process/32_ccgs_borrow.md）：
        python scripts/studio_tests.py --check
        （3 类断言：① doccheck2 触发面双向对账 ② test_manifest↔console_commands 映射
         ③ 工具台账三处登记核验；收口档实跑，存量豁免走 scripts/studio_tests_waiver.json
         棘轮基线；前置缺失 exit 2 记名 SKIP，断言 FAIL 非 0 exit 1）
    4c Android 真机交互（P5，S49.4e，docs/parity.md §P5 判据）：
        python scripts/android_device_test.py --run
        （P5-1 全屏 / P5-2 触摸命中 / P5-3 音频生命周期 / P5-4 切后台不崩，adb 驱动；
         无 adb 设备 = 记名 SKIP 非 FAIL，设备限制 SKIP 如实标注）
   ```

任一项红灯 = 本轮不可标 verified，不可推进下一任务。

## 完成定义（DoD）

`P0~P4 全 pass`（含 P0-5/P0-6/P4-4 S10 新增判据）+ `gate 36 项全绿` + `docs/plan-s4.md` 的 S4.4 产物齐全（桌面 exe / wasm 站点 / APK）+ ledger 全 `verified`。

**S10 对抗流水线停机条件**（S10.5，`docs/plan-s10-refactor.md`）：
审计 0 新发现（`rea-audit-fix` 双 agent）+ P3-3 全流程 16 步 0 失败 + gate 36 项全绿。
