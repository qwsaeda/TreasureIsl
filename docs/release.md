# 金银岛 TreasureIsl — Rust 现代化移植 发布说明

> **S18.7b：像素基线机制已退役（不再存在）**。
> 依旧的 `assets/render_baseline/` / `forward_baseline/` / `hd_baseline/` 三套 golden PNG、
> `render_snapshot` / `render_snapshot_forward` / `visual_regression` 三个 diff 测试、`visual_baseline.ps1`
> 与其门禁项、`baseline-mirror-identity` 校验均**已全部删除**。
> 现行机制：**结构不变式断言**（`tests/render_invariants.rs`，7 场景 × 非黑覆盖率/内容签名互异/暖色板在位/TTF 在位）
> 与 **CPU↔Bevy 内存对拍**（`bevy-cpu-parity` 门禁项：GPU 真实离屏读回 vs CPU 期望逐像素对照）。
> 本文中仍有的「基线」词句为**历史记述**，不再是现行机制。

> **S18.7a 注**：`evidence/render_png/*.png` = **CPU 自绘后端**（`ti-render` 内存帧缓冲 → PNG），**不是 Bevy 输出**；引擎原生首界面 = `evidence/bevy_native_index.png`（`treasure-bevy-native --shot`，GPU 实体渲染 + 读回像素）。像素基线（`assets/*_baseline/`）当前锁的是 **CPU 自绘版**，基线源待迁到 Bevy GPU 读回（S18.7 正文）。

## 项目概况

从原版 ARM WinCE 游戏《金银岛 TreasureIsl》(TreasureIsl.exe, ARM PE32, 176×220 逻辑分辨率) 逆向 + Rust 重写，
目标 WASM (浏览器) + Android 双端。采用 cargo workspace（6 个开发 crate；oracle 参照实现唯一源 = `crates/ti-core/src/oracle`（`pub mod oracle` 导出，测试直接 `use ti_core::oracle::`），逐字节重复 crate `legacy/oracle-rust` 已于 S15.8 删除（`check_forward_closed.py` C4 反向守卫防重引入）。

原版 exe 无源码，所有逻辑通过 IDA 伪代码 + oracle 差分测试逆向验证。

## 平台产物

> **S18.8a：发货壳切 Bevy**。桌面/浏览器的**唯一发货入口** = `crates/ti-engine`（独立 workspace）
> 的 Bevy 壳 `treasure-bevy-native` / `treasure-bevy-wasm`；ti-app 的 macroquad native/wasm 壳
> （`treasure-native` / `treasure-wasm`）已删除（Android 壳仍在，S18.8d 切 Bevy）。
> 浏览器页（wasm-bindgen 胶水 + HTML + Bevy 资源管线）属 **S18.8e**，当前 `web/` 只有 wasm 产物与资源。

| 平台 | 产物 | 路径 | 体积 (S18.8a 后最新) |
|---|---|---|---|
| 桌面（当前收口） | `treasure-bevy-native.exe` | `crates/ti-engine/target/release/` | 见 `evidence/S18.8a-wasm-size.log` 同目录口径（Bevy 壳体积待 S18.8e 复验） |
| 浏览器（当前收口，页面待 S18.8e） | `treasure_bevy_wasm.wasm`（Bevy 壳）+ 真实资源（无 JS 壳/HTML） | `web/` | 见 `evidence/S18.8a-wasm-size.log`（预算口径 ≤8 MB，见 `docs/process/11_bevy_migration.md` §3） |
| Android（后续，Mac 编译） | — | — (S2.6 在 Mac NDK/SDK 工具链出 APK，本机不产出) | — |

完整 sha256 见 `evidence/S4.4.log`（S4.3 时点 wasm 675,016 B）、`evidence/S5.4.log`；S5.6/S6.1~S6.4 完成后 macroquad wasm 增长至 745 KB（体积表见 `docs/modernization.md` §9，首屏 308,227 B / 非首屏 250,154 B）——**该组数字是 macroquad 壳的历史记述**。
**桌面 + 浏览器为当前可玩收口目标**（S5.1–S5.4：菜单→选人→选图→主地图→贸易/建造→战斗→存读档全链路可玩，P3-3 全流程 + P4-1 渲染对齐已机器判定）；
Android 为后续移动端交付，APK 在 Mac（已装 Android 环境）编译。

### 体积预算达成（历史口径：docs/plan-s4.md §S4.3；**当前预算见 `docs/process/11_bevy_migration.md` §3**）

| 项 | S4.3 实测（macroquad 壳，历史） | S5.6 及 S6.1~S6.4 完成后（历史） | 预算 | 状态 |
|---|---|---|---|---|
| wasm 主体 | 675,016 B | 745 KB（modernization §9） | ≤ 800,000 B（**S18.0 已作废**，新预算 ≤8 MB） | 历史 PASS（口径已换） |
| 首屏附属资源 | 102,079 B | 308,227 B（modernization §9 明细） | ≤ 490 KB（modernization 口径；S4.3 时点 ≤ 150,000 B） | 历史 PASS（口径已换） |
| 非首屏 (music.lpk) | 67,737 B | 250,154 B（modernization §9） | ≤ 180 KB（modernization 口径；S4.3 时点 ≤ 100,000 B） | 超 S4.3 时点预算，modernization §9 已重列口径 |

## 性能说明

- **桌面 60fps**：`--features stats` 实测 avg 60.0/60.0 fps（S4.3 验收值；`evidence/S4.4.log` 复测 60.9/59.3，同口径），主循环脏矩形呈现
  （静止帧 uploads=0 skipped=60/60，100% 跳帧、零 GPU 上传）——**macroquad 壳的历史口径**。
  S18.8a 起交付壳 = Bevy（Bevy 内建时钟，静止跳帧指标退役，`stats::FpsMeter::bevy_tick`），
  新口径的 native/wasm 实测属 **S18.8e**。
- **WASM 体积**：macroquad 壳 S6 后 745 KB（历史，见 `docs/modernization.md` §9）；**当前交付壳 = Bevy**，
  体积口径与预算见 `docs/process/11_bevy_migration.md` §3（≤8 MB），实测记录在 `evidence/S18.8a-wasm-size.log`。
  （S1.7b 起 OGG 转码默认不入游戏运行时 —— opus-pure 仅离线 build_assets/mod2ogg 链路开启，见「已知限制」；OGG 走预转码产物播放。）
- **传输层压缩**：浏览器对 `.wasm` 与 `.js` 普遍启用 gzip/brotli，实际下载体积更小（部署侧关注项）。

## 可玩收口（S5.1–S5.6 + S6.1–S6.5 + S7 HD 管线 + S8.0 标准化，2026-10）

把 S3 已 diff 验证的 12 个玩法模块接进 `ti-app` 交互壳，使游戏**从头可玩到尾**，
并完成呈现层完善（S6）、HD 资产管线（S7）与 S8.0 标准化改造（流程 + 业务文档，见 `docs/plan-s8-refactor.md`）：

- **S5.1 交互状态机**（`GameApp`）：`crates/ti-app/src/game_app/` 顶层场景状态机，
  把 `menu → selection → main_map → trade/build → battle → saveload` 全链路接通
  （`docs/gamebook.md` 接线规格）。壳层**只编排 UI/输入/渲染**，数值/判定全部委托
   `ti-core::rules` 已验证实现，不重抄公式（EXP-040 确定性约定；当前 RNG 为可播种 PCG，`rand_pcg`，S8.7 落地；旧 MSVC LCG 为 oracle 历史参照）。
  全链路 7 项断言 `tests/game_app_flow.rs` 全绿。
- **S5.2 全流程判据**（P3-3）：`tests/scripts/full_run.json` 固定种子（seed=42）+
  `treasure-headless` bin（`--features headless`），无窗口跑 `GameApp` 逻辑链，
  16 步场景流转断言 0 失败（开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单）。
  补齐 S4.4 曾用 smoke.ps1 冒烟替代的 P3-3 缺口。
- **S5.3 渲染对齐**（P4-1）：`tests/render_parity.rs` 用 headless 路径（`Framebuf` + 真字形，
  无 macroquad 窗口）渲染 5 个关键场景，断言确定性/非空白/场景区分/真字形/逻辑分辨率
  176×220 逐像素一致（5 项全绿）。
- **S5.5 Android 壳落地**（S2.6 代码前置，APK 在 Mac 编译）：`ti-app` 加 `android-shell`
  feature + `treasure-android` bin（复用共享 `GameApp` 交互核心 + `TouchPad` 触屏输入，
  资源读 APK assets，`macroquad::file::load_file` 与 wasm HTTP fetch 同源机制）；
  `android/README.md` 给出 Mac 侧 cargo-apk 脚手架 + 资源拷贝 + 编译/装机步骤。
  本机 Windows 不编译 android target（gate android 行 SKIP）；APK 在 Mac（已装 NDK/SDK
  + cargo-apk）产出。开发本身不依赖 Android 工具链。
- **S5.6 美术纹理渲染接线**（`docs/modernization.md` §9 资源改造 Step 3）：新增
  `crates/ti-app/src/sprite_atlas.rs`（`ui.pak` filenum INI 解析 + PCX 解码缓存 +
  `f1 → NN.pcx` 寻址 + `Framebuf` RGB565 blit）；三端 bin 注入 `ui.pak` + 首屏 8 个
  场景 `.Map`（`index`/`intro`/`selectrole`/`selectmap`/`main`/`vs`/`msgs`/
  `saveload`，其中 `index`/`intro` 共用 1 个 .Map → 8 个 .Map 文件覆盖 9 个场景，
   全量 20 UI 场景）；`GameApp::render` 在占位渲染后叠加真实精灵。sprite_atlas 5 项单测全绿
  （含真 `ui.pak` 端到端 + 越界裁剪 + 空 INI 拒绝）；寻址机制实测：`ui.pak` 122 条目
  = 120 PCX + 1 filenum
  INI + 1 其他，`f1=N` → `"{N:02d}.pcx"`（F-311 的 `f1` 字段在 `sub_460F8` 是
  blit 贴图索引，F-315 的事件描述符 +22 字节在运行期回填，不冲突）。
- **S6.1 上采样滤波**：`FilterMode`（Nearest 默认 / Linear）+ `F2` 键循环切换；
  滤波只改纹理采样器，不进坐标变换（P4-2 往返不变式单测已证）。
- **S6.2 边框装饰**：`BorderStyle`（Black / DeepBlue 默认 / MatchBackdrop）+ `F3` 键
  循环切换；letterbox 区域整屏填充主题色，游戏区 quad 后画覆盖，天然处理
  Stretch/ZoomCrop 裁剪模式。
- **S6.3 地形瓦片 blit**：新增 `terrain_atlas.rs`（`glmap_m01/02.pak` 瓦片 PCX，
   m01 47 条目/46 唯一 + m02 37 条目/32 唯一，filenum INI 映射；台账 S6.3 DoD 口径
   m01/m02，m03 未接入）；**关键发现**：地形是大画布 + 摄像机（实测 m01 的
  `iso_x` 范围 0..1216、`iso_y` 16..608，远超 176×220），故必须按摄像机偏移
  平移画布窗口；`center_camera` 把包围盒中心对到屏心 (88,110)；painter's order
  按 `iso_y` 升序。6 项单测全绿（含真实地形+真实瓦片 blit 出 >100 非黑像素）。
- **S6.4 多帧动画**：`filenum INI` 的 `num` 字段是帧数，实测 4 个 `num=5` 精灵
  （14/31/32/37.pcx，高度 199 = 5×40 帧纵向堆叠）；`advance_animation(delta_ms)`
  按毫秒推进，`frame()` 取当前帧号；`GameApp::render` 每帧按 `frame_delta_ms`
  推进（macroquad 壳用 `macroquad::time::get_time()` 算帧耗时；S18.8a 起交付壳 = Bevy，
  帧耗时取 Bevy 内建时钟 `Time::delta`）。5 项单测全绿
  （多帧切分 / 帧号环绕 / 帧推进 / 多帧 blit 差异 / 静态精灵帧不变）。
- **S6.5 容器条目业务名修正**：`build_assets` 的 `raw/` 目录与 `manifest.json`
  及 `unpak --out` 导出条目改按**业务名**命名（原为 `<key:08x>.bin` 哈希名，
  无命名意义）。新增 `ti-formats/src/resource_names.rs`：`ResourceNames` 三级反查
  （`mapinfo.ini` > exe UTF-16 资源名 > `EXTRAS` 多键证实名）+ 同名内容合并 +
  未命名回退 `unknown-{key:08x}.bin`。命名覆盖率 DoD 217/217（ui/m01/m02/cards/
  music/goods 全解），glpics 余 3 条名称哈希型（F-209）回落哈希名并显式标 `unknown`。
  单测 `resource_names` 7/7 + `build_assets` 3/3 全绿；注释里 `804615c2.bin`
  改 `glmusic.mod`（`mod_pcm.rs`/`mod2ogg.rs`/`audio_asset.rs`）。

三端产物（S5.4 复测，**macroquad 壳历史口径**；当前交付产物见上方「平台产物」）：

| 平台 | 产物 | 路径 | sha256 (前 16) |
|---|---|---|---|
| 桌面 | `treasure-native.exe` | `target/release/` | 见 `evidence/S5.4.log` |
| 浏览器 | `treasure_wasm.wasm` + JS 壳 + 资源 | `web/` | 见 `evidence/S5.4.log` |
| Android | — | — (S2.6 阻塞，SDK/NDK 未安装) | — |

## 后续（不阻塞完成）

- **帧缓冲色深**：RGB565 → RGB888，待 LLM 重画 HD 真彩色美术后一起做（现 16 色平涂无收益）。
- QEMU-CE 视觉对照（原版 ARM 运行环境，P4-1 逐像素对原版渲染的最终基准）
- 多人/联机扩展
- 成就系统

> 已收口（勿再列入「后续」）：S6.4 多帧动画、S6.3 地形瓦片 blit、S5.5 Android 壳——详见上方「可玩收口」。

## 已知限制

| 项 | 状态 |
|---|---|
| S2.6 Android APK | 桌面优先：APK 在 Mac 工具链（NDK/SDK + cargo-apk）编译，本机 Windows 无 NDK/SDK，gate android 行 SKIP（既定，不阻塞） |
| LCG RNG 常数 | 推测值（非实测），oracle 差分测试已验证行为一致 |
| P4-1 逐像素对原版 | S5.3 以「headless 渲染 vs 资源解码参考」（真字形+布局常量）为基准；对原版 exe 逐像素对照需 QEMU-CE 运行环境（后续项） |
| BGM 音色保真 | S1.7b 把 INHL.MOD 内嵌 8bit PCM 直转 OGG/Opus（可播放、可循环），未逐拍还原 Hekkus 模块播放器的音色表/节奏细节（ref/pseudocode 无独立播放器伪代码，音频经 Hekkus Sound System 抽象，F-104） |
| 战斗/卡片数值表 | S3.8/S3.9 的兵种 HP/atk、卡片效果码为 oracle 占位（**推测**），exe 无独立数值表；业务**流程**已对齐（回合制/事件分派），数值精度待 QEMU-CE 校准 |

## 如何运行

### 桌面 (Windows)

```sh
# S18.8a：交付壳 = Bevy（ti-engine 独立 workspace；macroquad 壳已退役）
cargo run -q --manifest-path crates/ti-engine/Cargo.toml --bin treasure-bevy-native
# 资源文件在仓库根的 data/（Bevy 壳按仓库根读 legacy/data 同源资源）
# S5.1 起：菜单→选人→选图→主地图→贸易/建造→战斗→存读档 全链路可玩
# 窗口可拖拽/最大化：内容恒 1080×1920 逻辑画布 letterbox 等比跟随（默认 Smooth；
# F1 = 等比/整数倍/拉伸/裁剪 循环，F4 切画质）——2026-10-26 窗口缩放跟随
# （`window_resize_system` 重建 shell `vp`，相机与输入同源不错位）
# 2026-10-26 修（S24.0 / EXP-116）：相机正交 `o.scale` 语义 = 世界单位/像素（与 shell
# 视口比 `vp.scale()` 互为倒数）⇒ `o.scale = 1/(vp.scale()×zoom)`。改前实机长期
# 4× 放大画布中央 1/4（`o.area`=270×480），拖大窗口内容反而变小；改后 `o.area`
# =1080×1920（画布完整可见）+ 窗口变大时内容等比放大。口径卡见
# `docs/process/12_bevy_usage.md` §3。
# 2026-10-26 修（S24.1 / EXP-117）：首页点击失灵 = 指针位置用 delta 累加而非窗口
# 绝对光标位置（光标不在左上角启动时命中全偏）→ 改用 `Window::cursor_position()`
#（逻辑像素）。诊断 flag：`--resizetest`（机器判定：程序化翻倍窗口 → `o.area` 恒等
# 画布 + px/unit 翻倍，退出码 0=PASS）/ `--resizeprobe`（活体探针：画面左上角 +
# 控制台实时 `win/sf/vp/mode/zoom/area`）。
# 2026-10-26 功能（S21.12）：主地图「建造面板」上线——点 HUD/deck 建造按钮开面板
#（4 建筑 + 「造船」第 5 入口；点选条目→再点确认，键盘 ↑↓/Enter/Back 同语义），
# 玩家船标按船等级切 4 张船图（小舢板→双桅→三桅→巡防，`assets/ui/boat_1..4.png`）；
# 造船走 `rules::boat` 单向升级（扣金 + 货舱/攻防立即生效，满级/缺金有提示条）。
# 另：代码标准三件套（`docs/process/16_code_standards.md` + `check_code_standards.py`
# 4 项机器检查 + 棘轮基线）入门禁。
```

### 浏览器 (WASM)

```sh
# S18.8a：wasm 交付产物 = Bevy 壳 `treasure-bevy-wasm`（构建 + 拷资源到 web/）
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080
# 注意：S18.8a 的 web/ 只有 wasm 产物 + 真实资源；**可打开的 Bevy 浏览器页**
# （wasm-bindgen JS 胶水 + HTML + 资源管线 + 体积预算复验）属 S18.8e。
```

### Android（Mac 工具链）

```
S2.6：APK 在 Mac（已装 Android NDK/SDK + cargo-apk/cargo-ndk）编译，本机 Windows 不产出。
# Mac 侧（示例，桌面/浏览器可玩为当前收口目标，移动端为后续交付）：
#   rustup target add aarch64-linux-android
#   cargo apk build -p ti-app --features android-shell   # 产出 APK 装机
#   装机后应显示主菜单 + 真字形 + 音效（与桌面/浏览器同源渲染）
```

### headless 全流程回归（P3-3 判据，S5.2）

```sh
# 无窗口跑 GameApp 逻辑链，按 full_run.json 固定种子（seed=42）逐步喂事件，断言场景流转 0 失败
cargo run -q -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json
# 输出：16/16 步断言 0 失败（开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单）
```

## 音频（S1.7b，摘要）

- BGM `glmusic.mod`（INHL.MOD，"I Need Her Love"）内嵌 8bit PCM，`build_assets`/`mod2ogg` 离线转码为 OGG/Opus（`opus-pure` 纯 Rust、确定性输出）；在线转码 feature `mod-ogg` 默认关闭（避免拉进 wasm，S4.4 实测 1,169,285 B → 675,016 B 恢复 800 KB 预算），游戏壳直接播预转码产物。
- 详见 `docs/plan-s0-s2.md` S1.7b + `legacy/docs/facts/`（INHL.MOD 格式、Hekkus Sound System 抽象，F-104）。

## 验收

- `scripts/gate.ps1` 全项绿（门禁项数与口径**单一事实源**见 `docs/parity.md` §代码质量门禁；doccheck / doccheck2 / 台账三方对账 / 业务闭环 / 模块并行 / 飞轮开轮必读 / 视觉回归等独立门禁项已陆续并入，逐项编号见 parity.md，不再在此硬编码）
- `plan/ledger.yaml` 234 任务（203 verified；台账计数随 S8–S26 演进，`doccheck` 断言锁定防漂移）（台账实时值，`cargo run -q -p ti-tools --bin ledger` 实测）
- S5.1 全链路 7 项断言（`tests/game_app_flow.rs`）+ S5.2 headless 全流程 16/16 步（P3-3）
  + S5.3 渲染对齐 5 项（P4-1，`tests/render_parity.rs`）全绿
- 产物 sha256 记录于 `evidence/S4.4.log` 与 `evidence/S5.4.log`
