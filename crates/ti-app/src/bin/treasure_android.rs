//! S2.6 Android 入口：macroquad native 壳（NDK ANativeWindow + AAudio）+ 资源读 APK assets。
//!
//! 与 `treasure_native` / `treasure_wasm` 壳**共用同一份交互核心**
//! （[`ti_app::game_app::GameApp`]，13 玩法模块接线，`docs/gamebook.md`），
//! 差异只在资源 IO：
//! - native（桌面）：`std::fs::read` 读仓库根 `data/`；
//! - wasm（浏览器）：`macroquad::file::load_file`（HTTP fetch）；
//! - **Android（本 bin）**：`macroquad::file::load_file`（APK assets 后端，macroquad 的
//!   android target 自动从 `assets/` 读取，与 wasm 的 HTTP fetch 同源机制）。
//!
//! 游戏主循环本体下沉到共享 [`ti_app::android_shell::run`]（与 `android-activity`
//! JNI 壳 crate 的 `game_loop` 同源，避免双开渲染后端，`docs/plan-s5.md` §S5.5）。
//! 本 bin 只做 `#[macroquad::main]` 入口转发。
//!
//! 用法（**Mac 侧，已装 Android NDK/SDK + cargo-apk**，见 `android/README.md`）：
//! ```sh
//! cargo apk build -p ti-app --features android-shell
//! # 产出 APK 装机 → 显示主菜单 + 真字形 + 音效（与桌面/浏览器同源渲染）
//! # 资源需打进 APK assets：assets/{GoldLand.Fnt,font.dat,settings.dat,music.lpk}
//! ```
//!
//! 本机 Windows 不编译此 target（gate android 行 SKIP），仅 Mac 工具链可构建。

use macroquad::prelude::*;

/// 窗口配置：强制原版逻辑分辨率 176×220（F-101）。
///
/// Android 上 miniquad 的 `window_width/height` 作为初始逻辑尺寸，实际屏幕尺寸
/// 由 `screen_width()/screen_height()` 每帧 `refit` 跟随（P4-2 letterbox/pillarbox）。
fn window_conf() -> Conf {
    Conf {
        window_title: "金银岛 TreasureIsl".to_owned(),
        window_width: 176,
        window_height: 220,
        ..Default::default()
    }
}

/// 主循环：转发到共享 `android_shell::run`（游戏主循环本体，13 玩法模块全场景
/// + 触屏输入 + 逐帧渲染 176×220 逻辑帧缓冲，`docs/gamebook.md` §4/§5）。
#[macroquad::main(window_conf)]
async fn main() {
    ti_app::android_shell::run().await;
}
