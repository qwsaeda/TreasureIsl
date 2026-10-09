# Android APK 构建脚手架（S2.6 / S5.5，Mac 侧使用；本机 Windows 不编译 android target）
#
# 决策：APK 在 Mac（已装 Android NDK/SDK + cargo-apk/cargo-ndk）编译，本机 Windows
# 无 NDK/SDK，gate android 行 SKIP。桌面/浏览器可玩为当前收口目标（S5.1–S5.4 已机器判定），
# Android 为后续移动端交付。
#
# 目录约定（cargo-apk 标准布局，`android/` 与 `Cargo.toml` 同级）：
#
#   treasure-island/
#   ├── Cargo.toml                  # workspace 根
#   ├── android/
#   │   ├── android-activity/       # JNI 入口（miniquad 的 android 后端 → ti-app 主循环）
#   │   │   ├── Cargo.toml          # cdylib 壳 crate（本仓已预写）
#   │   │   └── src/lib.rs          # miniquad::run(game_loop) → ti_app::android_shell::run（已预写）
#   │   ├── AndroidManifest.xml     # 包名/权限/横竖屏（本仓已预写，包名 com.example.treasureisland 占位）
#   │   └── assets/                 # 打进 APK 的资源（macroquad::file::load_file 读取）
#   │       ├── GoldLand.Fnt        # ← 从 data/GoldLand.Fnt 复制（构建前手动/脚本）
#   │       ├── font.dat            # ← 从 data/font.dat 复制
#   │       ├── settings.dat        # ← 从 data/settings.dat 复制
#   │       └── music.lpk           # ← 从 data/music.lpk 复制
#   └── crates/ti-app/
#       ├── src/bin/treasure_android.rs   # 本仓已落地（feature `android-shell`，转发到 android_shell::run）
#       └── src/android_shell.rs          # 游戏主循环本体（与 android-activity 的 game_loop 同源，单渲染线程）
#
# 本仓 ti-app 的 `treasure-android` bin（`feature = android-shell`）资源走
# `macroquad::file::load_file`（APK assets 后端），读 `assets/GoldLand.Fnt` 等，
# 与 wasm 壳的 HTTP fetch 同源机制。`android-activity` 的 `game_loop` 经
# `futures::executor::block_on` 驱动 `ti_app::android_shell::run`（async macroquad 主循环），
# 不 spawn 额外渲染线程（单渲染线程约束）。
#
# ── 方式 A：本仓已预写脚手架（Mac 侧直接编译，免 `cargo apk create`）──
#   # 1. 装工具链（一次）
#   cargo install cargo-apk
#   # 2. 把资源打进 APK assets
#   mkdir -p android/assets
#   cp data/GoldLand.Fnt data/font.dat data/settings.dat data/music.lpk android/assets/
#   # 3. 按你的包名改 android/AndroidManifest.xml 的 package（占位 com.example.treasureisland）
#   # 4. 编译 + 装机
#   cargo apk build -p ti-app --features android-shell
#   adb install target/universal-debug.apk
#
# ── 方式 B：cargo-apk 自动脚手架（Mac 侧包名自定义，会覆盖/补全本仓预写文件）──
#   cargo apk create
#   # 产出 android/android-activity/ + android/AndroidManifest.xml + .cargo/config.toml 的
#   # aarch64/armv7 链接器配置（指向本机 NDK）；按提示填包名
#   # （本仓 android/android-activity 已预写，cargo apk create 会提示复用/覆盖）
#
# ── 把资源打进 APK assets（宏定义，Mac 侧手动或脚本）──
#   mkdir -p android/assets
#   cp data/GoldLand.Fnt data/font.dat data/settings.dat data/music.lpk android/assets/
#   # （若 ti-app 已开 mod-ogg，则 OGG 预转码产物 assets_out/audio/*.ogg 也需复制进
#   #  android/assets/audio/，让 macroquad 播放 OGG BGM；默认关，WAV 混音即可）
#
# ── 编译 debug APK ──
#   cargo apk build -p ti-app --features android-shell
#   # 产出 target/aarch64-linux-android/debug/ 下的 .so + 壳 apk
#   # 装机：
#   #   adb install target/universal-debug.apk   （或按 cargo-apk 实际产物名）
#   # 装机后应显示主菜单 + 真字形中文 + 真实音效（与桌面/浏览器同源渲染）
#
# ── 编译 release APK ──
#   cargo apk build -p ti-app --features android-shell --release
#
# ── 验证（DoD，与 ledger S2.6 一致）──
#   1. APK 装机显示主菜单 + 真字形 + 音效（手动清单）；
#   2. 全链路可玩：菜单→选人→选图→主地图→贸易/建造→战斗→存读档（触屏输入）；
#   3. `gate.ps1` 的 android 行（本机若装了 aarch64 target 则编译 PASS，否则 SKIP）。
#
# 存档写 scoped storage（`getExternalFilesDir`）：当前 `GameApp` 存读档走内存槽位
# （`SaveSlot`），尚未接 Android 文件持久化；如需持久化，后续在 `ti-platform`
# 的 `Storage` trait 加 Android 后端（`FileStore` 指向 external files dir），
# 属移动端增强项，不阻塞 APK 主流程可玩。
#
# 触屏为主输入（无键盘）：`treasure_android.rs` 的 `KeyState` 恒 `default()`（全 false），
# 输入全走 `poll_touch` 触屏路径（与 wasm 壳同源，`docs/modernization.md` §5）。
# 横竖屏：`android/AndroidManifest.xml` 设 `screenOrientation`（建议 landscape，见
# modernization.md §2.3）；miniquad 自动跟随系统方向。
