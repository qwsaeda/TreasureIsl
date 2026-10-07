# 执行计划 S0–S2：地基 / 资源逆向 / 三端骨架

任务 ID 即台账 ID。**每个任务必须同时满足：产出物落地 + 验收命令通过 + `evidence/<id>.log` 存档 + `scripts/gate.sh` 全绿**，缺一不可 `verified`。
全局标准见 `docs/parity.md`，循环机制见 `docs/loop.md`。

## S0 — 地基与反编译导出（0.5~1 天）

| ID | 产出物 | 验收命令 / DoD | deps |
|---|---|---|---|
| S0.1 | cargo workspace（§5.1 六 crate 骨架）；`rustfmt.toml`/`clippy.toml`/`[workspace.lints]`；`rustup target add wasm32-unknown-unknown aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`；`scripts/gate.sh`（8 项，Android 构建行先注释） | `scripts/gate.sh` 全绿（gate 第 4 项只跑 native+wasm）；占位扫描 0 命中；**完成后回写 AGENTS.md 命令一节** | — |
| S0.2 | `scripts/ida_dump.py` + `idat64 -A -S` 批处理；产出 `ref/pseudocode/*.c`、`ref/functions.csv`（addr/size/name/caller/callee/字符串 xref） | 伪代码文件数 == IDA 函数数；`functions.csv` 行数与 `.i64` 函数表一致（0 缺失） | S0.1 |
| S0.3 | `cargo run -p ti-tools -- classify`：按库特征（zlib 字符串/HMSS/PCX/gx 调用/UI 事件串）分类 → `ref/classify.json` + 覆盖率报告 | 覆盖率 100%（允许类目 `pending`，S3.12 前清零） | S0.2 |
| S0.4 | `ref/imports.json`：105+9+2=116 个导入序号/名字 → API 名 100% 恢复（WinCE 序号表 + 调用点特征双向校验） | 116/116 有名字；每条至少 1 处调用点证据 | S0.2 |
| S0.5 | `ti-formats/src/macros.rs` 落地 `define_format!`/`parse_err!`/`roundtrip_all!` 三个宏 + 单测 | 宏的单测全绿（用一个玩具格式验证 0 未知字节 + roundtrip 断言生效） | S0.1 |
| S0.6 | `plan/ledger.yaml` 全量任务登记（本文件与 S3/S4 的所有 ID，状态 `pending`，deps 正确） | gate 第 8 项台账校验通过 | S0.1 |

## S1 — 资源逆向（1~2 周）

**本阶段所有任务统一 DoD**：格式文档（`docs/formats/<x>.md`，含完整字段与偏移）+ `ti-formats` 解析器接入 `define_format!` + `roundtrip_all!` 通过 + `verify --zero-unknown` pass。

| ID | 产出物 | 验收 |
|---|---|---|
| S1.1 | `docs/formats/lucencypackage.md`：头部全字段（含 F-201 非对齐偏移）+ **`name→key` 哈希算法逆出** | P1-2：119 个 exe 资源名命中率 100%（`cargo test -p ti-tools --test pak_key`） |
| S1.2 | `ti-tools` 的 `unpak`：通吃 `.pak`/`.lpk`（两者结构差异写入文档） | P0-1 提取率 100%；P0-3 `--repack` sha256 相等 |
| S1.3 | `pcx2png` + 独立 python 参考解码器（交叉验证用）：PCX 全变体（调色板/RLE/多平面） | P0-4 逐像素一致 |
| S1.4 | `map2json`：21 个 UI 场景 `.Map` → JSON；同时产出 `ref/event_ids.json`；build.rs 据此生成 `ti-formats/src/ids.rs` | 0 未知字节；roundtrip 逐字节一致；P1-3 事件 id 计数吻合 |
| S1.5 | `font2bmfont`：`GoldLand.Fnt` + `font.dat` → BMFont/引擎字模 | P4-3 抽样字形逐位图一致 |
| S1.6 | `ini2json`：`.lpk` 内 UTF-16 INI（cards/event/goods catalog） | 键值解析率 100%；JSON→INI 语义无损 |
| S1.7 | `audiotrans`：WAV 直取 + `glmusic.mod` 离线转 OGG 入 `assets_out` | 每条音轨可播放，时长/采样率与源一致 |
| S1.8 | `map2json --terrain`：`glmapm01/02/03.Map` 地形格式逆出 | 0 未知字节 + roundtrip + 尺寸与头字段吻合 |
| S1.9 | `ti-core/src/save/`：`settings.dat` 等存档格式规格 + 读写实现 | roundtrip 逐字节一致 + 字段语义文档 |
| S1.10 | `build_assets`（一条命令生成 `assets_out/` + `manifest.json` sha256）+ `preview`（桌面预览 UI 场景/地图/卡牌/字体） | `build_assets` 幂等；manifest 校验通过；preview 能打开全部 21 场景 + 3 地图。**→ P0、P1 全绿，阶段门禁** | 

## S2 — 三端骨架（1~2 周）

| ID | 产出物 | 验收 |
|---|---|---|
| S2.1 | `ti-platform/gfx`：176×220 逻辑帧缓冲 → 整数倍/平滑缩放（macroquad 渲染） | P4-2 单测：任意窗口尺寸坐标互逆 |
| S2.2 | `ti-platform/input`：触摸/鼠标/键盘 → 统一事件（down/up/drag）+ 虚拟按键映射 | 坐标反变换单测；桌面手动清单 |
| S2.3 | `ti-platform/audio`：多通道混音、BGM 循环、音量 | 桌面播 WAV/OGG 无爆音；通道测试 |
| S2.4 | `ti-platform/storage`：桌面文件 / wasm IndexedDB / Android 内部存储 三实现 | 三端存读档往返测试 |
| S2.5 | wasm 构建链：`build_wasm` 脚本 + html 壳 + 资源加载 | 浏览器打开主菜单场景，真实资源 + 中文 + 音效 |
| S2.6 | Android 构建链：**无 Docker** → 装 Android SDK/NDK + `cargo-apk` 或 `cargo-ndk` 本机链路（Java 17 已备）；manifest 配置 orientation/exported | APK 装到设备/模拟器，显示同一主菜单；**gate 第 4 项 Android 行解锁** |
| S2.7 | `scripts/smoke.sh`：native/wasm/apk 各跑资源加载 + 场景渲染 | 三端全绿。**→ 阶段 S2 门禁（同时消化 macroquad Android 风险）** |

## 修订记录

- 2026-10 环境探测后修订：S2.6 由 docker `quad-apk` 改为本机 `cargo-apk`/`cargo-ndk`（无 Docker）；S0.1 增加 target 安装；新增 S0.5/S0.6。
