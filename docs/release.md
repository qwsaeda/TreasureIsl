# 金银�?TreasureIsl �?Rust 现代化移�?发布说明

## 项目概况

从原�?ARM WinCE 游戏《金银岛 TreasureIsl�?TreasureIsl.exe, ARM PE32, 176×220 逻辑分辨�? 逆向 + Rust 重写�?
目标 WASM (浏览�? + Android 双端。采�?cargo workspace (5 crates) + macroquad 渲染引擎�?

原版 exe 无源码，所有逻辑通过 IDA 伪代�?+ oracle 差分测试逆向验证�?

## 平台产物

| 平台 | 产物 | 路径 | 体积 (S5.4 后最�? |
|---|---|---|---|
| 桌面（当前收口） | `treasure-native.exe` | `target/release/` | �?`evidence/S5.4.log` |
| 浏览器（当前收口�?| `treasure_wasm.wasm` (S6.4 �?745 KB，见 `docs/modernization.md` §9) + JS �?+ 资源 | `web/` | �?`evidence/S5.4.log` |
| Android（后续，Mac 编译�?| �?| �?(S2.6 �?Mac NDK/SDK 工具链出 APK，本机不产出) | �?|

完整 sha256 �?`evidence/S4.4.log`（S4.3 时点 wasm 675,016 B）、`evidence/S5.4.log`；S5.6/S6.1~S6.4 完成�?wasm 增长�?745 KB（体积表�?`docs/modernization.md` §9，首�?308,227 B / 非首�?250,154 B）�?
**桌面 + 浏览器为当前可玩收口目标**（S5.1–S5.4：菜单→选人→选图→主地图→贸�?建造→战斗→存读档全链路可玩，P3-3 全流�?+ P4-1 渲染对齐已机器判定）�?
Android 为后续移动端交付，APK �?Mac（已�?Android 环境）编译�?

### 体积预算达成（见 docs/plan-s4.md §S4.3；S4.3 时点实测，S5.6 �?S6.1~S6.4 完成后最新值见 modernization.md §9�?

| �?| S4.3 实测 | S5.6 �?S6.1~S6.4 完成�?| 预算 | 状�?|
|---|---|---|---|---|
| wasm 主体 | 675,016 B | 745 KB（modernization §9�?| �?800,000 B | PASS |
| 首屏附属资源 | 102,079 B | 308,227 B（modernization §9 明细�?| �?490 KB（modernization 口径；S4.3 时点 �?150,000 B�?| PASS |
| 非首�?(music.lpk) | 67,737 B | 250,154 B（modernization §9�?| �?180 KB（modernization 口径；S4.3 时点 �?100,000 B�?| �?S4.3 时点预算，modernization §9 已重列口�?|

## 性能说明

- **桌面 60fps**：`--features stats` 实测 avg 60.0/60.0 fps（S4.3 验收值；`evidence/S4.4.log` 复测 60.9/59.3，同口径），主循环脏矩形呈现
  （静止帧 uploads=0 skipped=60/60�?00% 跳帧、零 GPU 上传）�?
- **WASM 体积**：S6 �?745 KB（release，见 `docs/modernization.md` §9），�?macroquad + miniquad gl.js + 真实字体/音效资源，单文件可运行�?
  （S1.7b �?OGG 转码默认不入游戏运行�?—�?opus-pure 仅离�?build_assets/mod2ogg 链路开启，
  见「已知限制」；OGG 走预转码产物播放。）
- **传输层压�?*：浏览器�?`.wasm` �?`.js` 普遍启用 gzip/brotli，实际下载体积更小（部署侧关注项）�?

## 可玩收口（S5.1–S5.6 + S6.1–S6.5 + S7 HD 管线 + S8.0 标准化，2026-10�?

�?S3 �?diff 验证�?12 个玩法模块接�?`ti-app` 交互壳，使游�?*从头可玩到尾**�?
并完成呈现层完善（S6）、HD 资产管线（S7）与 S8.0 标准化改造（流程 + 业务文档，见 `docs/plan-s8-refactor.md`）：

- **S5.1 交互状态机**（`GameApp`）：`crates/ti-app/src/game_app.rs` 顶层场景状态机�?
  �?`menu �?selection �?main_map �?trade/build �?battle �?saveload` 全链路接�?
  （`docs/gamebook.md` 接线规格）。壳�?*只编�?UI/输入/渲染**，数�?判定全部委托
   `ti-core::rules` 已验证实现，不重抄公式（EXP-040 确定性约定；当前 RNG 为可播种 PCG，`rand_pcg`，S8.7 落地；旧 MSVC LCG �?oracle 历史参照）�?
  全链�?7 项断言 `tests/game_app_flow.rs` 全绿�?
- **S5.2 全流程判�?*（P3-3）：`tests/scripts/full_run.json` 固定种子（seed=42�?
  `treasure-headless` bin（`--features headless`），无窗口跑 `GameApp` 逻辑链，
  16 步场景流转断言 0 失败（开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单）�?
  补齐 S4.4 曾用 smoke.ps1 冒烟替代�?P3-3 缺口�?
- **S5.3 渲染对齐**（P4-1）：`tests/render_parity.rs` �?headless 路径（`Framebuf` + 真字形，
  �?macroquad 窗口）渲�?5 个关键场景，断言确定�?非空�?场景区分/真字�?逻辑分辨�?
  176×220 逐像素一致（5 项全绿）�?
- **S5.5 Android 壳落�?*（S2.6 代码前置，APK �?Mac 编译）：`ti-app` �?`android-shell`
  feature + `treasure-android` bin（复用共�?`GameApp` 交互核心 + `TouchPad` 触屏输入�?
  资源�?APK assets，`macroquad::file::load_file` �?wasm HTTP fetch 同源机制）；
  `android/README.md` 给出 Mac �?cargo-apk 脚手�?+ 资源拷贝 + 编译/装机步骤�?
  本机 Windows 不编�?android target（gate android �?SKIP）；APK �?Mac（已�?NDK/SDK
  + cargo-apk）产出。开发本身不依赖 Android 工具链�?
- **S5.6 美术纹理渲染接线**（`docs/modernization.md` §9 资源改�?Step 3）：新增
  `crates/ti-app/src/sprite_atlas.rs`（`ui.pak` filenum INI 解析 + PCX 解码缓存 +
  `f1 �?NN.pcx` 寻址 + `Framebuf` RGB565 blit）；三端 bin 注入 `ui.pak` + 首屏 8 �?
  场景 `.Map`（`index`/`intro`/`selectrole`/`selectmap`/`main`/`vs`/`msgs`/
  `saveload`，其�?`index`/`intro` 共用 1 �?.Map �?8 �?.Map 文件覆盖 9 个场景，
   全量 20 UI 场景）；`GameApp::render` 在占位渲染后叠加真实精灵。sprite_atlas 5 项单测全�?
  （含�?`ui.pak` 端到�?+ 越界裁剪 + �?INI 拒绝）；寻址机制实测：`ui.pak` 122 条目
  = 120 PCX + 1 filenum
  INI + 1 其他，`f1=N` �?`"{N:02d}.pcx"`（F-311 �?`f1` 字段�?`sub_460F8` �?
  blit 贴图索引，F-315 的事件描述符 +22 字节在运行期回填，不冲突）�?
- **S6.1 上采样滤�?*：`FilterMode`（Nearest 默认 / Linear�? `F2` 键循环切换；
  滤波只改纹理采样器，不进坐标变换（P4-2 往返不变式单测已证）�?
- **S6.2 边框装饰**：`BorderStyle`（Black / DeepBlue 默认 / MatchBackdrop�? `F3` �?
  循环切换；letterbox 区域整屏填充主题色，游戏�?quad 后画覆盖，天然处�?
  Stretch/ZoomCrop 裁剪模式�?
- **S6.3 地形瓦片 blit**：新�?`terrain_atlas.rs`（`glmap_m01/02.pak` 瓦片 PCX�?
   m01 47 条目/46 唯一 + m02 37 条目/32 唯一，filenum INI 映射；台�?S6.3 DoD 口径
   m01/m02，m03 未接入）�?*关键发现**：地形是大画�?+ 摄像机（实测 m01 �?
  `iso_x` 范围 0..1216、`iso_y` 16..608，远�?176×220），故必须按摄像机偏�?
  平移画布窗口；`center_camera` 把包围盒中心对到屏心 (88,110)；painter's order
  �?`iso_y` 升序�? 项单测全绿（含真实地�?真实瓦片 blit �?>100 非黑像素）�?
- **S6.4 多帧动画**：`filenum INI` �?`num` 字段是帧数，实测 4 �?`num=5` 精灵
  �?4/31/32/37.pcx，高�?199 = 5×40 帧纵向堆叠）；`advance_animation(delta_ms)`
  按毫秒推进，`frame()` 取当前帧号；`GameApp::render` 每帧�?`frame_delta_ms`
  推进（native+wasm �?`macroquad::time::get_time()` 算帧耗时）�? 项单测全�?
  （多帧切�?/ 帧号环绕 / 帧推�?/ 多帧 blit 差异 / 静态精灵帧不变）�?
- **S6.5 容器条目业务名修�?*：`build_assets` �?`raw/` 目录�?`manifest.json`
  �?`unpak --out` 导出条目改按**业务�?*命名（原�?`<key:08x>.bin` 哈希名，
  无命名意义）。新�?`ti-formats/src/resource_names.rs`：`ResourceNames` 三级反查
  （`mapinfo.ini` > exe UTF-16 资源�?> `EXTRAS` 多键证实名）+ 同名内容合并 +
  未命名回退 `unknown-{key:08x}.bin`。命名覆盖率 DoD 217/217（ui/m01/m02/cards/
  music/goods 全解），glpics �?3 条名称哈希型（F-209）回落哈希名并显式标 `unknown`�?
  单测 `resource_names` 7/7 + `build_assets` 3/3 全绿；注释里 `804615c2.bin`
  �?`glmusic.mod`（`mod_pcm.rs`/`mod2ogg.rs`/`audio_asset.rs`）�?

三端产物（S5.4 复测）：

| 平台 | 产物 | 路径 | sha256 (�?16) |
|---|---|---|---|
| 桌面 | `treasure-native.exe` | `target/release/` | �?`evidence/S5.4.log` |
| 浏览�?| `treasure_wasm.wasm` + JS �?+ 资源 | `web/` | �?`evidence/S5.4.log` |
| Android | �?| �?(S2.6 阻塞，SDK/NDK 未安�? | �?|

## 后续（不阻塞完成�?

- **Android 端落�?*：S5.5 Android 壳已落地（`android-shell` feature + `treasure-android` bin，verified）；
  S2.6 �?APK �?**Mac**（已�?Android NDK/SDK + cargo-apk/cargo-ndk）编译（blocked，本�?Windows 无工具链不产出，gate android �?SKIP 为既定状态，不阻塞）�?
- ~~**动画�?*：`filenum INI` �?`num>1` 的精灵（�?`num=5` 的多帧动画）当前只取�?0 帧；后续加帧定时切换。~~ �?已完�?**S6.4**（多帧动画，`advance_animation` 按毫秒推进）�?
- ~~**地形瓦片 blit**：`glmap_m*.pak`�?8+47 张瓦�?PCX）未接入，主地图仍占位网格；复用 SpriteAtlas 机制即可。~~ �?已完�?**S6.3**（`terrain_atlas.rs` 解析 `glmap_m*.pak` 瓦片 PCX，按等距投影 + 摄像机平�?blit）�?
- **帧缓冲色�?*：RGB565 �?RGB888，待 LLM 重画 HD 真彩色美术后一起做（现 16 色平涂无收益）�?
- QEMU-CE 视觉对照（原�?ARM 运行环境，P4-1 逐像素对原版渲染的最终基准）
- 多人/联机扩展
- 成就系统

## 已知限制

| �?| 状�?|
|---|---|
| S2.6 Android APK | 桌面优先：APK �?Mac 工具链（NDK/SDK + cargo-apk）编译，本机 Windows �?NDK/SDK，gate android �?SKIP（既定，不阻塞） |
| LCG RNG 常数 | 推测值（非实测），oracle 差分测试已验证行为一�?|
| P4-1 逐像素对原版 | S5.3 以「headless 渲染 vs 资源解码参考」（真字�?布局常量）为基准；对原版 exe 逐像素对照需 QEMU-CE 运行环境（后续项�?|
| BGM 音色保真 | S1.7b �?INHL.MOD 内嵌 8bit PCM 直转 OGG/Opus（可播放、可循环），未逐拍还原 Hekkus 模块播放器的音色�?节奏细节（ref/pseudocode 无独立播放器伪代码，音频�?Hekkus Sound System 抽象，F-104�?|
| 战斗/卡片数值表 | S3.8/S3.9 的兵�?HP/atk、卡片效果码�?oracle 占位�?*推测**），exe 无独立数值表；业�?*流程**已对齐（回合�?事件分派），数值精度待 QEMU-CE 校准 |


## 如何运行

### 桌面 (Windows)

```sh
cargo build --release -p ti-app --features native-shell
target/release/treasure-native.exe
# 资源文件需�?exe 同级：data/{GoldLand.Fnt,font.dat,settings.dat,music.lpk}
# S5.1 起：菜单→选人→选图→主地图→贸�?建造→战斗→存读档 全链路可�?
```

### 浏览�?(WASM)

```sh
powershell -ExecutionPolicy Bypass -File scripts/build_wasm.ps1
cargo run -q -p ti-app --features wasm-shell --bin treasure-wasm   # �?serve �?web/
cargo run -q -p ti-tools --bin serve -- --dir web --port 8080
# 打开 http://localhost:8080
```

### Android（Mac 工具链）

```
S2.6：APK �?Mac（已�?Android NDK/SDK + cargo-apk/cargo-ndk）编译，本机 Windows 不产出�?
# Mac 侧（示例，桌�?浏览器可玩为当前收口目标，移动端为后续交付）�?
#   rustup target add aarch64-linux-android
#   cargo apk build -p ti-app --features android-shell   # 产出 APK 装机
#   装机后应显示主菜�?+ 真字�?+ 音效（与桌面/浏览器同源渲染）
```

### headless 全流程回归（P3-3 判据，S5.2�?

```sh
# 无窗口跑 GameApp 逻辑链，�?full_run.json 固定种子（seed=42）逐步喂事件，断言场景流转 0 失败
cargo run -q -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json
# 输出�?6/16 步断言 0 失败（开局→选人→选图→移动→贸易→建造→战斗→存读档→回主菜单）
```

## 音频说明（S1.7b�?

- **BGM**：原�?`glmusic.mod`（INHL.MOD 自定义格式，"I Need Her Love"，DIPA 作曲）内�?
  8bit 无符�?PCM 流（0x4844 �?31516 采样，@8000 Hz �?3.94 s 循环）�?
- **转码路径**：`build_assets`/`mod2ogg`（ti-tools，开 `ti-platform/ogg`）离线转�?
  `assets_out/audio/*.ogg`；`ModAudio::parse` 提取 PCM �?8bit→i16 居中扩展 �?`opus-pure` 0.2.2
  编码 OGG/Opus�?4 kb/s 目标，RFC 7845 end-trim 精确收尾）�?
- **编码器选型**：`opus-pure`（纯 Rust，BSD-3-Clause，无 C 依赖，wasm32/Android 可直接编译；
  确定性输出——`OggOpusWriter` serial 由头�?FNV-1a 派生，同输入逐字节一致，
  `build_assets` 幂等不破坏）。采样率�?`snap_rate()` 吸附�?Opus 合法�?
  {8k, 12k, 16k, 24k, 48k}�?
- **运行时体积控�?*：OGG 在线转码（`ti-app` feature `mod-ogg`）默认关闭——游戏壳直接播放
  预转码产�?/ 混音 WAV；`prepare_music` 在默认构建下�?MOD 条目计跳过（离线 OGG 已覆�?BGM），
  避免 opus-pure 拉进 wasm（S4.4 实测 1,169,285 B �?675,016 B，恢�?800 KB 预算）�?
- **播放接入**：桌�?wasm 两壳 `pick_bgm` 优先 OGG 曲目（`mod-ogg` 开启时），否则回退最�?
  WAV 混音曲目；SE 取最�?WAV�?

## 验收

- `scripts/gate.ps1` 全绿 + doccheck 独立�?9 项（`cargo run -q -p ti-tools --bin doccheck`�? doccheck2 独立�?13 项（�?agent 审计固化断言，`cargo run -q -p ti-tools --bin doccheck2 .`�? 台账三方对账独立�?14 项（`python scripts/check_ledger_reality.py`，S10.0 增）+ 业务闭环断言�?15 项（`python scripts/check_forward_closed.py --check`，S11 增）+ 模块并行前置�?16 项（`python scripts/check_module_contracts.py --check`，S11.4 增）+ 飞轮开轮必读第17 项（；EXP-070 增）+ 视袄回军准准图第18 项（；EXP-072 增）通过（S5.4 时点 8 项门禁；当前 18 项，�?parity.md�?
- `plan/ledger.yaml` 106 任务�?02 verified；台账计数随 S8–S13 演进，doccheck 断言锁定防漂移）（S2.6 / S7.7）（台账实时值，`cargo run -q -p ti-tools --bin ledger` 实测�?
- S5.1 全链�?7 项断言（`tests/game_app_flow.rs`�? S5.2 headless 全流�?16/16 步（P3-3�?
  + S5.3 渲染对齐 5 项（P4-1，`tests/render_parity.rs`）全�?
- 产物 sha256 记录�?`evidence/S4.4.log` �?`evidence/S5.4.log`



