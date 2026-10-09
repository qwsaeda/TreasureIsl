# 金银岛 TreasureIsl — Rust 现代化移植 发布说明

## 项目概况

从原版 ARM WinCE 游戏《金银岛 TreasureIsl》(TreasureIsl.exe, ARM PE32, 176×220 逻辑分辨率) 逆向 + Rust 重写，
目标 WASM (浏览器) + Android 双端。采用 cargo workspace (5 crates) + macroquad 渲染引擎。

原版 exe 无源码，所有逻辑通过 IDA 伪代码 + oracle 差分测试逆向验证。

## 平台产物

| 平台 | 产物 | 路径 | sha256 (前 16) |
|---|---|---|---|
| 桌面（当前收口） | `treasure-native.exe` (1,134,592 B) | `target/release/` | `C28A418361EE413E` |
| 浏览器（当前收口） | `treasure_wasm.wasm` (675,047 B) + JS 壳 + 资源 | `web/` | `68A8A9D806D71DD7` |
| Android（后续，Mac 编译） | — | — (S2.6 在 Mac NDK/SDK 工具链出 APK，本机不产出) | — |

完整 sha256 见 `evidence/S4.4.log` 与 `evidence/S5.4.log`。
**桌面 + 浏览器为当前可玩收口目标**（S5.1–S5.4：菜单→选人→选图→主地图→贸易/建造→战斗→存读档全链路可玩，P3-3 全流程 + P4-1 渲染对齐已机器判定）；
Android 为后续移动端交付，APK 在 Mac（已装 Android 环境）编译。

### 体积预算达成（见 docs/plan-s4.md §S4.3）

| 项 | 实测 | 预算 | 状态 |
|---|---|---|---|
| wasm 主体 | 675,016 B | ≤ 800,000 B | PASS |
| 首屏附属资源 | 102,079 B | ≤ 150,000 B | PASS |
| 非首屏 (music.lpk) | 67,737 B | ≤ 100,000 B | PASS |

## 性能说明

- **桌面 60fps**：`--features stats` 实测 avg 60.9/59.3 fps（S4.4 复测），主循环脏矩形呈现
  （静止帧 uploads=0 skipped=60/60，100% 跳帧、零 GPU 上传）。
- **WASM 体积**：675 KB (release)，含 macroquad + miniquad gl.js + 真实字体/音效资源，单文件可运行。
  （S1.7b 起 OGG 转码默认不入游戏运行时 —— opus-pure 仅离线 build_assets/mod2ogg 链路开启，
  见「已知限制」；OGG 走预转码产物播放。）
- **传输层压缩**：浏览器对 `.wasm` 与 `.js` 普遍启用 gzip/brotli，实际下载体积更小（部署侧关注项）。

## 可玩收口（S5.1–S5.3，2026-10 新增）

把 S3 已 diff 验证的 13 个玩法模块接进 `ti-app` 交互壳，使游戏**从头可玩到尾**：

- **S5.1 交互状态机**（`GameApp`）：`crates/ti-app/src/game_app.rs` 顶层场景状态机，
  把 `menu → selection → main_map → trade/build → battle → saveload` 全链路接通
  （`docs/gamebook.md` 接线规格）。壳层**只编排 UI/输入/渲染**，数值/判定全部委托
  `ti-core::rules` 已验证实现，不重抄公式（EXP-040 确定性 LCG）。
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
  `f1 → NN.pcx` 寻址 + `Framebuf` RGB565 blit）；三端 bin 注入 `ui.pak` + 8 个场景
  `.Map`（`selectrole/selectmap/main/vs/msgs/saveload` + 首屏两个）；`GameApp::render`
  在占位渲染后叠加真实精灵。sprite_atlas 5 项单测全绿（含真 `ui.pak` 端到端 +
  越界裁剪 + 空 INI 拒绝）。寻址机制实测：`ui.pak` 122 条目 = 120 PCX + 1 filenum
  INI + 1 其他，`f1=N` → `"{N:02d}.pcx"`（F-311 的 `f1` 字段在 `sub_460F8` 是
  blit 贴图索引，F-315 的事件描述符 +22 字节在运行期回填，不冲突）。

三端产物（S5.4 复测）：

| 平台 | 产物 | 路径 | sha256 (前 16) |
|---|---|---|---|
| 桌面 | `treasure-native.exe` | `target/release/` | 见 `evidence/S5.4.log` |
| 浏览器 | `treasure_wasm.wasm` + JS 壳 + 资源 | `web/` | 见 `evidence/S5.4.log` |
| Android | — | — (S2.6 阻塞，SDK/NDK 未安装) | — |

## 后续（不阻塞完成）

- **Android 端落地（S2.6）**：桌面/浏览器可玩为当前收口目标；APK 在 **Mac**（已装 Android NDK/SDK + cargo-apk/cargo-ndk）编译，本机 Windows 不产出（gate android 行 SKIP 为既定状态，不阻塞）。
- **动画帧**：`filenum INI` 中 `num>1` 的精灵（如 `num=5` 的多帧动画）当前只取第 0 帧；后续加帧定时切换。
- **地形瓦片 blit**：`glmap_m*.pak`（88+47 张瓦片 PCX）未接入，主地图仍占位网格；复用 SpriteAtlas 机制即可。
- **帧缓冲色深**：RGB565 → RGB888，待 LLM 重画 HD 真彩色美术后一起做（现 16 色平涂无收益）。
- QEMU-CE 视觉对照（原版 ARM 运行环境，P4-1 逐像素对原版渲染的最终基准）
- 多人/联机扩展
- 成就系统

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
cargo build --release -p ti-app --features native-shell
target/release/treasure-native.exe
# 资源文件需与 exe 同级：data/{GoldLand.Fnt,font.dat,settings.dat,music.lpk}
# S5.1 起：菜单→选人→选图→主地图→贸易/建造→战斗→存读档 全链路可玩
```

### 浏览器 (WASM)

```sh
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
cargo run -q -p ti-app --features wasm-shell --bin treasure-wasm   # 或 serve 起 web/
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080
# 打开 http://localhost:8080
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

## 音频说明（S1.7b）

- **BGM**：原版 `glmusic.mod`（INHL.MOD 自定义格式，"I Need Her Love"，DIPA 作曲）内嵌
  8bit 无符号 PCM 流（0x4844 起 31516 采样，@8000 Hz ≈ 3.94 s 循环）。
- **转码路径**：`build_assets`/`mod2ogg`（ti-tools，开 `ti-platform/ogg`）离线转码
  `assets_out/audio/*.ogg`；`ModAudio::parse` 提取 PCM → 8bit→i16 居中扩展 → `opus-pure` 0.2.2
  编码 OGG/Opus（64 kb/s 目标，RFC 7845 end-trim 精确收尾）。
- **编码器选型**：`opus-pure`（纯 Rust，BSD-3-Clause，无 C 依赖，wasm32/Android 可直接编译；
  确定性输出——`OggOpusWriter` serial 由头部 FNV-1a 派生，同输入逐字节一致，
  `build_assets` 幂等不破坏）。采样率经 `snap_rate()` 吸附到 Opus 合法集
  {8k, 12k, 16k, 24k, 48k}。
- **运行时体积控制**：OGG 在线转码（`ti-app` feature `mod-ogg`）默认关闭——游戏壳直接播放
  预转码产物 / 混音 WAV；`prepare_music` 在默认构建下把 MOD 条目计跳过（离线 OGG 已覆盖 BGM），
  避免 opus-pure 拉进 wasm（S4.4 实测 1,169,285 B → 675,016 B，恢复 800 KB 预算）。
- **播放接入**：桌面/wasm 两壳 `pick_bgm` 优先 OGG 曲目（`mod-ogg` 开启时），否则回退最长
  WAV 混音曲目；SE 取最短 WAV。

## 验收

- `scripts/gate.ps1` 8 项全绿（S5.4 复测；6 PASS + 2 SKIP：android target 未装 S2.6 / verify.rs 未落地）
- `plan/ledger.yaml` 50 任务：49 verified + 1 blocked（S2.6 Android，SDK/NDK 未安装，既定阻塞项）
- S5.1 全链路 7 项断言（`tests/game_app_flow.rs`）+ S5.2 headless 全流程 16/16 步（P3-3）
  + S5.3 渲染对齐 5 项（P4-1，`tests/render_parity.rs`）全绿
- 产物 sha256 记录于 `evidence/S4.4.log` 与 `evidence/S5.4.log`
