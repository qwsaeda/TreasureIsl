# 迁移 Linux 交接文档 + 会话总结（2026-10-29 · S27.7 在途）

> 用户 2026-10-29：准备把项目放到 Linux 电脑继续开发，需要把会话与在途工作总结进文档，
> 迁移完成可直接续工。本文 = 迁移操作 + 当前进度快照 + Linux 环境准备 + 命令差异 + 风险点。
>
> **续工第一入口仍是路由表（`AGENTS.md`）+ `plan/ledger.yaml`**；本文只回答「迁移怎么迁、
> 迁完从哪继续」。

## 0. 一句话

项目当前所有在途工作 = **S27.7 VS 对战界面改版（业务+规则已落码、5 套渲染图已出、待选型签收）**
+ **S27.5 main_map v5 出图签收（未收尾）**；**crates/ti-engine 已并入根 workspace**
（用户 2026-10-29 要求「crates 内项目统一由根 Cargo.toml 管理」，已落地）。按本文迁移即可无缝续工。

---

## 1. ⚠️ 迁移前必须做的事（Windows 侧，二选一）

在途改动是 **tracked-M + untracked-? 的混合**，只提交 tracked 会**丢工作**。必须二选一：

- **A（推荐 · 唯一合规）整目录拷贝**：把整个仓库目录原样拷到 Linux（含 `.git` + 全部未跟踪文件）。
  这是唯一把**未跟踪新文件**也带过去的方式，零丢失，且**不提交**。
  > 推荐原因：硬规则「全绿才可 commit / push」——S27.5/S27.7 目前未 gate 全绿，**此时提交 = 违反提交纪律**。
  > A 方案不产生提交，只搬工作区状态。
- **B（备选，需接受违规）先提交再 clone**：`git add -A && git commit`（在途 S27.5/S27.7 作为 WIP
  一并提交，**注意这违反「gate 全绿才可 commit」硬规则**，仅当用户明确接受 WIP 提交时用），再 `git clone`。
  **不能只 `git add` 已跟踪文件**（会丢 §1.1 全部未跟踪新文件）。

### 1.1 未跟踪（?）文件清单——A 方案必须带上，B 方案必须 `add -A` 才有

| 类别 | 文件 |
|---|---|
| VS 原型渲染器 | `crates/ti-app/src/scenes/vs_proto.rs` |
| VS 版式规格 | `docs/ui-redesign-v3/11_vs_spec.md` |
| VS 资产批次 | `assets/ui_batches/vs_v1.json` |
| 新 VS 资产（9） | `assets/ui/{boss_captain,hero_back,deck_battle,item_keg,item_medkit,item_rum,item_smoke,icon_cutlass,icon_shield}.png` |
| 5 张渲染图 | `evidence/bevy_native_vs_{a,b,c,d,e}.png` |
| 其它批次 | `assets/ui_batches/{boats_v1,card_v1}.json` |

> 另有一批**临时产物**（非在途工作，A 方案可顺手删）：`scripts/tmp/*.py`（一次性脚本，
> 硬规则 10 用完即删）、`scripts/tmp/island_home_m0*_raw*.png` / `island_home_m0*_prompt.txt`、
> `evidence/tmp_crop_*.png`。

### 1.2 已跟踪-M 文件（在途改动，A 方案随目录走，B 方案 `add -A` 带上）

`Cargo.toml` / `Cargo.lock`（workspace 合并）、`crates/ti-app/{Cargo.toml,src/android_shell.rs,
src/game_app/board.rs,src/scenes/{chrome,main_map,mod,vs}.rs,tests/s43_fps_smoke.rs}`、
`crates/ti-core/{src/data/{balance,mod}.rs,src/domain/combat.rs,src/rules/battle.rs,
tests/{data_tables,exhaust_combat,rules_battle}.rs}`、`crates/ti-engine/{Cargo.toml,
src/bin/treasure_bevy.rs,src/scene_render.rs}`、`crates/ti-render/{src/canvas.rs,src/drawlist.rs}`、
`assets/data/{balance,scenes,strings,theme}.json`、`assets/ui/{boat_1..4,island_home_m01..m03}.png`、
`assets/ui_manifest.json`、`docs/{README.md,business/combat.md,experience/{build,index}.md,
parity.md,process/{11_bevy_migration,17_layout_engine}.md,release.md,
ui-redesign-v3/{07_v3_5_pass,README}.md}`、`AGENTS.md`。

> `crates/ti-engine/Cargo.lock` 显示为 **D（删除）**——workspace 合并后 lock 迁到根 `Cargo.lock`，
> 子 lock 不再需要，属预期（B 方案 `add -A` 会记录删除）。

---

## 2. 当前进度快照（迁移后从哪继续）

### 2.1 已完成（本轮会话）

- **workspace 合并**：`crates/ti-engine` 并入根 `Cargo.toml`（members + `bevy` 依赖登记在根
  `[workspace.dependencies]`，`bevy = "=0.20.0"` 精确钉版）；子 workspace `[workspace]` 表删除 +
  改 `.workspace` 继承；`[profile.release] strip` 迁到根；脚本/文档命令统一 `-p ti-engine`；
  wasm 产物路径 `crates/ti-engine/target/` → 根 `target/`。
  - 合并触发两条隐藏冲突已根治：① bevy 子 crate 版本漂移（钉 `=0.20.0` + 迁移旧 lock）；
    ② `links=alsa` 互斥（`ti-app` 的 `android-shell` 去掉 `macroquad/audio`，
    `android_shell.rs` 改本地音频桩，口径同「发货无声」）。详见 EXP-128（`docs/experience/build.md`）。
  - `cargo metadata`（根全依赖解析）exit 0；根 `Cargo.lock` 含 `alsa-sys`、无 `quad-alsa-sys`；
    bevy 全家 = 0.20.0。

- **S27.7 VS 改版 · 业务与规则**：
  - 战斗动作 = 攻击/道具/防御；4 道具 = 火药桶(固定300)/医疗包(25%上限)/朗姆酒(3回合×1.5)/烟雾弹(挡1次反击)；
    携带上限 3/3/2/2；`defend_reduction_pct=50`。单一事实源 `docs/business/combat.md` §战斗动作，
    数值 `assets/data/balance.json → combat.items`。
  - 规则落码：`domain::combat`（`ItemEffect`/`CombatAction`）、`data::balance`（`CombatItemDef`/Raw 映射）、
    `rules::battle::step_with`（道具结算/防御减伤/增益/烟雾弹；`step()` ≡ `step_with(Attack)`，旧 API 不破）。
  - **`cargo test -p ti-core --test rules_battle` = 20/20 全绿**（含 6 条新单测：
    `combat_item_damage_is_fixed`/`heal_capped`/`atk_buff_expires`/`stagger_once`/`defend_halves_counter`/`keeps_rng_state`）。
  - 文案：`assets/data/strings.json` 加 `combat.item.{keg,medkit,rum,smoke}` + `combat.action.{attack,item,defend}`（CRLF + 字母序）。

- **S27.7 VS 改版 · 资产与 5 套渲染图**：
  - 资产批次 `assets/ui_batches/vs_v1.json`：`boss_captain`(600×900)、`hero_back`(600×900)、
    `deck_battle`(1080×1920)、`item_keg/medkit/rum/smoke`(256×256)、`icon_cutlass`/`icon_shield`(256×256 动作图标)。
    已 register/generate/install（`magenta_clean` 抠图）。
  - 原型渲染器 `crates/ti-app/src/scenes/vs_proto.rs`：5 套版式 `A/B/C/D/E`，`TI_VS_PROTO=a|b|c|d|e`
    切换（未设置 = 走正式版式）；`vs.rs` 已接分派。
  - **5 张引擎真实渲染图已出**（`--shot --scene=vs` + `TI_SHOT_NAME`）：
    `evidence/bevy_native_vs_{a,b,c,d,e}.png`（1080×1920，各 8297018 B，14:32~14:33 重渲）。
  - 版式特征（A 甲板对峙+24 段血环 / B 海岸 BOSS+扇形手牌+能量点 / C 海战对轰+竖血槽+道具转盘 /
    D 甲板透视决斗+弧形道具 / E 对称回合制+铜牌立绘框+黄铜指挥台）。

- **S27.6 已 verified**（岛卡美术风格刷新，前序会话）。

### 2.2 待办（迁移后照此继续）

1. **用户从 5 张渲染图选型签收**（消息留痕，A 纪律）→ 在 `docs/ui-redesign-v3/11_vs_spec.md`
   补「定案版式」小节（逐元素坐标表 + 资产接线 + 交互契约，镜像 `business/08_layout_design.md` §4.4）。
2. `vs.rs` 正式实现（道具带/动作键/回合徽章/伤害数字）→ `scenes.json` vs 段原语。
3. 动作接线：`GameInput::UseItem(idx)`/`Defend` + `route_hit` 命中（bevy_picking `pick_key`）。
4. 回合演出时间轴（突进→命中闪白→屏震→伤害数字→血条插值→敌方反击）。
5. 背包/经济接线（贸易港购买 + 存档持久化 + 携带上限）。
6. 文档对齐 + gate 全绿 → `evidence/S27.7.log` → `scripts/task_close.py` 置 verified。
7. **回填 S27.5**（main_map v5 出图 + 用户视觉签收 + 4 项整改：①540×960 视口不可读
   ②环格卡未按既定设计 ③玩家渲染问题 ⑤按钮图标 14 白蒙版几何呆板；中央岛效果已确认 ✓、
   index 风格统一已落码待核）。

---

## 3. Linux 环境准备

| 依赖 | 安装/说明 |
|---|---|
| Rust | `rustup`（stable ≥ 1.85，edition 2024） |
| wasm target | `rustup target add wasm32-unknown-unknown`（gate wasm 条件项需要） |
| PowerShell | `pwsh`（PowerShell Core，跨平台）——**Linux 无 `powershell.exe`** |
| Python | `python3`（脚本：`ui_assets.py`/`validate_data.py`/`map_pack.py`/`ui_asset_batch.py`/`doc_numbers.py` 等） |
| git | `git`（建议 `git config core.autocrlf input`，见 §5） |
| 图形库（Bevy/winit） | `libx11` `libxkbcommon` `libwayland` `mesa`（Vulkan 或 GL 驱动） |
| 无头出图 | `xvfb`（`xvfb-run`）+ `mesa` 软件渲染（lavapipe）——`--shot` 需要 GPU 上下文 |
| 生图脚本 | `AGNES_API_KEY` 环境变量（`agnes_gen.py`/`agnes_video.py`） |
| Android | 需 NDK/SDK（本机 Mac 产出 APK；Linux 可装但不必须） |
| 浏览器 e2e | `pip install playwright pillow`（`scripts/wasm_browser_e2e.py` L2/L4 截图像素采样 + 4 层判据）；浏览器缺失时自动探测 `~/.cache/ms-playwright/chromium-*/`（`TI_CHROMIUM_PATH` 可显式覆盖；本机实测 = `chromium-1228/chrome-linux64/chrome`） |

> **IDA/REA（legacy 逆向）是 Windows-only**（`D:\soft\IDA Professional 9.4\idat.exe` +
> `mcp.rea` + `REA_ANALYSIS_PROVIDER=ida`）；游戏开发/续工**不需要**，Linux 侧跳过。

---

## 4. 命令差异（Windows → Linux）

| 用途 | Windows（现状） | Linux（迁移后） |
|---|---|---|
| 全门禁 | `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` | `pwsh -File scripts/gate.ps1` |
| 一键入口 | `powershell -ExecutionPolicy Bypass -File scripts/playbook.ps1 run-native` | `pwsh -File scripts/playbook.ps1 run-native` |
| 桌面发货 | `cargo run -q --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native` | `cargo run -q -p ti-engine --bin treasure-bevy-native` |
| 出图 | `cargo run -q -p ti-engine --bin treasure-bevy-native -- --shot --scene=vs` | 同左（`-p ti-engine` 已统一） |
| 产物路径 | `crates/ti-engine/target/release/` | **根 `target/release/`**（并回 workspace 后） |
| 路径分隔 | 反斜杠 `\` | 正斜杠 `/` |

> **Linux 门禁兼容已落地（2026-10-29 续工实测，`evidence/linux_gate_green.log` gate 31 项 0 failure）**：
> ① `gate.ps1` 内 `python` 自动映射 `python3`（`$py` 变量，Windows 行为不变）+ 占位/unwrap 扫描
> 路径分隔 `(?i)[/\\]` 双兼容（Linux 正斜杠）；② 系统依赖 = `pkg-config` + `libudev-dev` +
> `libwayland-dev`（`sudo apt-get install` 一次）；③ `pcx_ref` 测试自动探测 `python3`；
> ④ 门禁 31 项在 Linux 全绿（含 GPU bevy-cpu-parity：本机 Intel HD 630 + Mesa Vulkan 真跑）。
> ⑤ **修复已落码（S32.11，`evidence/S32.11.log`，verified 待双审计收敛）**：gate 第 17 项 `flywheel-readme` 原在 Linux 上**假 PASS**（`scripts/gate.ps1` 硬编码 `powershell`，本机无该命令但 `$LASTEXITCODE` 仍 0；`$py` 映射只覆盖 Python，不覆盖 PowerShell）；S32.11 改为宿主解释器探测（`pwsh` 优先 → `powershell` 回退，Windows 行为不变）+ 无宿主时记名 SKIP 不假 PASS。见 22 号 §4.3。

> `-ExecutionPolicy Bypass` 是 Windows 概念，Linux pwsh 不需要。`.ps1` 脚本内 Windows 专属段
> （`D:\soft\IDA`、`cmd /c` 等）在 Linux 下要跳过（逆向相关，游戏开发不跑）。

---

## 5. 已知 Linux 风险点

1. **CRLF churn（最重要）**：Windows 侧 `core.autocrlf=true` 且**无 `.gitattributes`**。
   若 Linux 用 `git clone`，git 可能按 Linux 默认（autocrlf 未设/input）把 CRLF 文件归一化，
   触发 `doccheck2` BOM/CRLF/UTF-8 门禁误报（`assets/data/strings.json` 等是 CRLF）。
   - **A 方案（整目录拷贝）**：字节原样，无此问题。
   - **B 方案（clone）**：Linux 设 `git config core.autocrlf input`（只进不转出），或补 `.gitattributes`
     （`* text=auto`）并一次性 `git add --renormalize .`；迁移后先跑 `pwsh -File scripts/gate.ps1` 验证。
2. **`--shot` 无头 GPU**：Linux 无显示器时 wgpu 无上下文 ⇒ 出图失败/黑图。
   用 `xvfb-run -s "-screen 0 1280x800x24" pwsh -File ...` 或确保有 X/Wayland + mesa（lavapipe）。
3. **`s43_fps_smoke`**：需要真窗口 + 60fps 环境；headless/无 GPU 会 SKIP 或 FAIL（VM 环境已知项，
   可豁免，见 gate）。Linux 上跑真实窗口会话时才能验证。
4. **cwd 必须是仓库根**：native 资源用 `std::fs` 相对仓库根（`assets/`），跑 `--shot`/`run-native`
   前 `cd` 到仓库根。
5. **音频**：发货无声口径（S18.8/S30.0），Linux 无 alsa 依赖（bevy 未开 audio 特性）；
   `links=alsa` 冲突已随 workspace 合并根治（§2.1）。
6. **wasm**：`scripts/build_wasm.ps1` 在 Linux 用 `pwsh` 跑，产物落根 `target/`（§4）。

---

## 6. 迁移后第一条命令序列（续工）

```bash
cd <repo>                      # 仓库根
git status --porcelain         # 确认在途改动带过来了（应见 §1 清单）
pwsh -File scripts/gate.ps1    # 全门禁（先确认环境 OK；s43_fps_smoke 无 GPU 可豁免）
# 看 5 张渲染图选型（用户签收）
#   evidence/bevy_native_vs_{a,b,c,d,e}.png
# 选定后：11_vs_spec.md 补定案版式小节 → vs.rs 正式实现 → gate 全绿 → task_close S27.7
# 然后回填 S27.5
```

---

## 7. 会话总结（S27.7 一轮做了什么）

- **病灶**：旧 VS 界面 = 顶置双血条 + 背景 + 居中文字，无角色/无道具/无演出。
- **用户拍板**：5 套概念稿全出**引擎真实渲染图**再选型；道具集认可按此做。
- **业务定案**：动作 攻击/道具/防御；4 道具数值进 `balance.combat.items`；
  规则只消费 `ItemEffect`（加道具不改 `rules::battle`）；道具不消耗 `GameRng`（确定性）。
- **资产**：7+2 枚新资产（立绘/背景/道具图标/动作图标），`magenta_clean` 抠图干净。
- **渲染**：5 套版式原型渲染器 + 5 张 1080×1920 引擎渲染图。
- **体系**：ti-engine 并回根 workspace（用户明确要求），双依赖冲突根治 + EXP-128 记录。
- **未决**：版式选型未签收（图已出，待用户挑）；正式落码未开始；S27.5 未回填。
