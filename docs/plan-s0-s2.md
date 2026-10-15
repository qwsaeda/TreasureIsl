# 执行计划 S0–S2：地基 / 资源逆向 / 三端骨架

> **定位**：S0–S2 是逆向期的历史计划（S0 地基 / S1 资源逆向 / S2 三端骨架），**已 archived**，非正向开发依据。逆向产物实体归档 `legacy/`。当前正向开发见 `docs/00_design.md` + `docs/business/`。

任务 ID 即台账 ID。**每个任务必须同时满足：产出物落地 + 验收命令通过 + `evidence/<id>.log` 存档 + `scripts/gate.ps1` 全绿**，缺一不可 `verified`。
全局标准见 `docs/parity.md`，循环机制见 `docs/loop.md`。

## S0 — 地基与反编译导出（0.5~1 天）

| ID | 产出物 | 验收命令 / DoD | deps |
|---|---|---|---|
| S0.1 | cargo workspace（§5.1 **5 crate** 骨架）；`rustfmt.toml`/`clippy.toml`/`[workspace.lints]`；`scripts/gate.ps1`（当时 8 项；构建项本期只含 native，wasm/android 行按阶段解锁；现 20 项） | `scripts/gate.ps1` 全绿（gate 第 4 项本期只跑 native）；占位扫描 0 命中；**完成后回写 AGENTS.md 命令一节** | — |
| S0.2 | `scripts/ida_dump.py` + `idat -A -S` 批处理；产出 `ref/pseudocode/*.c`、`ref/functions.csv`（addr/size/name/caller/callee/字符串 xref） | 伪代码文件数 == IDA 函数数；`functions.csv` 行数与 `.i64` 函数表一致（0 缺失） | S0.1 |
| S0.3 | `cargo run -p ti-tools -- classify`：按库特征（zlib 字符串/HMSS/PCX/gx 调用/UI 事件串）分类 → `ref/classify.json` + 覆盖率报告 | 覆盖率 100%（允许类目 `pending`，S3.12 前清零） | S0.2 |
| S0.4 | `ref/imports.json`：105+9+2=116 个导入序号/名字 → API 名 100% 恢复（WinCE 序号表 + 调用点特征双向校验） | 116/116 有名字；每条至少 1 处调用点证据 | S0.2 |
| S0.5 | `ti-formats/src/macros.rs` 落地 `define_format!`/`parse_err!`/`roundtrip_all!` 三个宏 + 单测 | 宏的单测全绿（用一个玩具格式验证 0 未知字节 + roundtrip 断言生效） | S0.1 |
| S0.6 | `plan/ledger.yaml` 全量任务登记（本文件与 S3/S4 的所有 ID，状态 `pending`，deps 正确） | gate 第 8 项台账校验通过（当时 8 项门禁；现 20 项） | S0.1 |

## S1 — 资源逆向（1~2 周）

**本阶段所有任务统一 DoD**：格式文档（`docs/formats/<x>.md`，含完整字段与偏移）+ `ti-formats` 解析器接入 `define_format!` + `roundtrip_all!` 通过 + `verify --zero-unknown` pass。

| ID | 产出物 | 验收 |
|---|---|---|
| S1.1 | `docs/formats/lucencypackage.md`：头部全字段（含 F-201 非对齐偏移）+ **`name→key` 哈希算法逆出** | P1-2（DoD 已修订，见 EXP-017）：凡是能用资源名解释的条目 100% 无误且数量 ≥ 实测下界 85；`cargo test -p ti-tools --test pak_key`。**旧 DoD「119 个 exe 资源名命中率 100%」不可达**（`.map`/`.sav`/`settings.dat`/`GoldLand.Fnt` 是磁盘独立文件，无 key） |
| S1.2 | `ti-tools` 的 `unpak`：通吃 `.pak`/`.lpk`（两者结构差异写入文档） | P0-1 提取率 100%；P0-3 `--repack` sha256 相等 |
| S1.3 | `pcx2png` + 独立 python 参考解码器（交叉验证用）：PCX 全变体（调色板/RLE/多平面） | P0-4 逐像素一致 |
| S1.4 | `map2json`：**20 个 UI 场景 + 3 个地形 = 23 个** `.Map` → JSON（记录级语义解析在 S1.4b） | 0 未知字节；roundtrip 逐字节一致 |
| S1.4b | `controls` bin 全量导出控件记录（`ref/controls.json`，20 场景 2041 条）+ `ti-formats/src/ids.rs` 控件类型枚举（f5） | P1-3：`ref/controls.json` 记录数 == 控件总数（2041）；20/20 UI 场景整除吻合；字段语义 exe 侧确认（F-311：f3=Y 坐标，非事件 id） |
| S1.5 | `font2bmfont`：`GoldLand.Fnt` + `font.dat` 结构解析（往返 + 0 未知字节） | 结构完整 + 0 未知字节 + 往返逐字节；**逐位图一致拆至 S1.5b** |
| S1.5b | 字形位图**行列组织与位序**逆出（12 行 × 2B、MSB-first、行序自上而下） | P4-3：`cargo test -p ti-formats --test font_parity` 全绿（charset 1:1 + 几何断言 + 渲染样例） |
| S1.6 | `ini2json`：`.lpk` 内 UTF-16 INI（cards/event/goods catalog） | 键值解析率 100%；JSON→INI 语义无损 |
| S1.7 | `audiotrans`：WAV 抽取/重采样 + `glmusic.mod` 头部规格 | 素材识别 + WAV 抽取正确 + MOD 头规格明确；**OGG 编码拆至 S1.7b** |
| S1.7b | MOD 播放器 + OGG/Vorbis 编码 | `glmusic.mod` 可播放 + OGG 转码。**已 `verified`**（opus-pure 0.2.2 纯 Rust 编码器，INHL.MOD PCM 提取 → OGG/Opus 64kb/s，`evidence/S1.7b.log`） |
| S1.8 | `map2json --terrain`：`glmapm01/02/03.Map` 地形格式结构解析 | 0 未知字节 + roundtrip + 尺寸吻合；**区域字段语义拆至 S1.8b** |
| S1.8b | 地形 cell/连线/trailer 记录字段语义逆出 | 单测 `tests/terrain.rs` 13 项 + 3/3 图全量恒等式验证（F-307/F-308） |
| S1.9 | `ti-core/src/save/`：`settings.dat` 往返 + 存档封套编解码 | `settings.dat` 往返逐字节 + 封套往返；**字段表语义拆至 S3.10** |
| S1.10 | `build_assets`（一条命令生成 `assets_out/` + `manifest.json` sha256）+ `preview`（桌面预览 UI 场景/地图/卡牌/字体） | `build_assets` 幂等；manifest 校验通过；preview 能打开全部 **20 场景 + 3 地图**。**→ P0、P1 全绿，阶段门禁** |
| S1.11 | ID 型 key 绑定机制逆出（容器内 `mapinfo.ini` 文件名清单，217 条） | `cargo test -p ti-tools --test id_keys` 全绿：ui/m01/m02 共 204 行 `filename` 的 `djb2` 在同容器 entries 中 0 失配 + UI 场景记录布局 20/20 | 

## S2 — 三端骨架（1~2 周）

| ID | 产出物 | 验收 |
|---|---|---|
| S2.1 | `ti-platform/gfx`：176×220 逻辑帧缓冲 → 整数倍/平滑缩放（macroquad 渲染） | P4-2 单测：任意窗口尺寸坐标互逆 |
| S2.2 | `ti-platform/input`：触摸/鼠标/键盘 → 统一事件（down/up/drag）+ 虚拟按键映射 | 坐标反变换单测；桌面手动清单 |
| S2.3 | `ti-platform/audio`：多通道混音、BGM 循环、音量 | 桌面播 WAV/OGG 无爆音；通道测试 |
| S2.4 | `ti-platform/storage`：桌面文件 / wasm IndexedDB / Android 内部存储 三实现 | 三端存读档往返测试 |
| S2.5 | wasm 构建链：`rustup target add wasm32-unknown-unknown` + `build_wasm` 脚本 + html 壳 + 资源加载；解锁 gate 第 4 项 wasm 行 | 浏览器打开主菜单场景，真实资源 + 中文 + 音效 |
| S2.6 | Android 构建链：**无 Docker** → `rustup target add` 三个 android triple + 装 Android SDK/NDK + `cargo-apk` 或 `cargo-ndk` 本机链路（Java 17 已备）；manifest 配置 orientation/exported；解锁 gate 第 4 项 android 行 | APK 装到设备/模拟器，显示同一主菜单 |
| S2.7 | `scripts/smoke.ps1`：native/wasm 各跑资源加载 + 场景渲染（APK 端随 S2.6 后补） | 两端全绿。**→ 阶段 S2 门禁（同时消化 macroquad Android 风险）** |

## 修订记录

- 2026-10 环境探测后修订：S2.6 由 docker `quad-apk` 改为本机 `cargo-apk`/`cargo-ndk`（无 Docker）；新增 S0.5/S0.6。
- 2026-10（迭代 0，按用户决策）：**编译工具链整体后置** —— S0.1 不再安装 wasm/android target（移入 S2.5/S2.6），gate 构建项按阶段解锁；门禁由 `gate.sh` 改为本机可执行的 `scripts/gate.ps1`（无 `rg`，占位扫描用 `Select-String`）；「六 crate」笔误修正为 5。
