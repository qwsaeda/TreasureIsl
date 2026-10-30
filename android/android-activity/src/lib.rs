//! android-activity — 金银岛 TreasureIsl 的 miniquad Android JNI 入口（S5.5）。
//!
//! 本 crate 是 cargo-apk 的「壳 crate」：把 miniquad 的 Android 后端（`android_glue`
//! + `miniquad`）接到 Android Activity 生命周期，然后转交给 `ti-app` 的
//! `treasure-android` bin（feature `android-shell`）的主循环（`docs/plan-s5.md` §S5.5）。
//!
//! 资源 IO 走 `macroquad::file::load_file` 的 android 后端（APK assets），与 wasm 壳的
//! HTTP fetch 同源机制。全屏 176×220 逻辑分辨率由 miniquad 跟随 ANativeWindow 实际尺寸
//! （P4-2 letterbox/pillarbox），横竖屏自适应。
//!
//! ── Mac 侧首次生成（cargo-apk 脚手架会自动创建/覆盖本文件与 ../AndroidManifest.xml）──
//!   cargo install cargo-apk
//!   cargo apk create
//!   # 若已存在本目录，cargo apk create 会提示复用/覆盖；按提示填包名。
//!
//! 入口约定：miniquad 在 android 平台由 `miniquad::run` 宏展开 JNI 符号
//! （`JNI_OnLoad` + `Java_*` Activity 生命周期回调）。`game_loop` 即游戏主循环本体，
//! 与 `treasure-android` bin 的 `#[macroquad::main]` 逻辑一致（构建共享 `GameApp`
//! 交互核心 + `TouchPad` 触屏输入，逐帧渲染 176×220 逻辑帧缓冲，与桌面/浏览器同源，
//! `docs/gamebook.md` §4）。

use miniquad::{conf, Context};

/// miniquad 游戏主循环（由 `miniquad::run(game_loop)` 在 JNI 入口转交调用）。
///
/// `ctx` 提供渲染上下文（GL/ES）；`conf` 是窗口配置。本循环驱动 `ti_app` 的
/// `GameApp` 全场景交互（菜单→选人→选图→主地图→贸易/建造→战斗→存读档），
/// 触屏输入经 `TouchPad` 命中判定（`docs/modernization.md` §5，直接触摸界面元素）。
///
/// `game_loop` 是 miniquad 的同步渲染回调（每帧调用），而 `ti_app::android_shell::run`
/// 是 async（macroquad 主循环 + `next_frame().await`），故在此建一个
/// `tokio`/`futures` 运行时把 async 循环驱动起来。miniquad 的 android 入口已在独立
/// 线程，此处用 `futures::executor::block_on` 阻塞式驱动单个 async 循环即可
/// （不跨线程、不 spawn 额外渲染线程，符合「单渲染线程」约束）。
fn game_loop(_ctx: &mut Context, _conf: conf::Conf) {
    use futures::executor::block_on;
    // 游戏主循环本体在 `treasure-android` bin（feature `android-shell`）。
    // 此处经 `ti_app::android_shell::run` 桥接——由 miniquad 的 android 入口线程直接
    // block_on 驱动，避免双开渲染后端（`docs/plan-s5.md` §S5.5「单渲染线程」约束）。
    block_on(ti_app::android_shell::run());
}

/// 入口：miniquad 的 android 平台由 `miniquad::run` 生成 JNI 符号并驱动 `game_loop`。
fn main() {
    miniquad::run(conf::Conf {
        window_title: "金银岛 TreasureIsl".to_owned(),
        window_width: 176,
        window_height: 220,
        // 全屏跟随 ANativeWindow（P4-2 letterbox）
        high_dpi: false,
        ..conf::default_conf()
    })
    .unwrap_or_else(|e| eprintln!("miniquad 启动失败：{e:?}"));
}
