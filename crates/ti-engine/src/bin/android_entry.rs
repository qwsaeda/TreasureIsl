// S18.8d：Android JNI 入口（`android-activity` 壳 crate 编译期 `include!` 本文件，
// 与 `treasure_bevy.rs` 的 `run()` 同一 bin 编译单元 ⇒ 0 重复链接）。
//
// 形态 = `bevy_main` 宏展开同构：`android_main(AndroidApp)` 注入 `bevy::android::ANDROID_APP`
// 后走 native 相同的 `run()`（资源读降级 + 地图包内嵌默认，见 `treasure_bevy.rs` android 注释）。
// 注意：本文件**只被 `include!`**（非独立 crate 根），首行不能用 `//!` 内层文档注释
// （include! 展开后非模块根，E0753）；改用普通 `//`。

/// JNI 入口（`android-activity` 的 `JNI_OnLoad` 把 NativeActivity 转交此函数）。
#[unsafe(no_mangle)]
#[allow(unsafe_code, reason = "android-activity JNI 入口须 no_mangle（winit 壳约定，bevy_main 同形态）")]
#[allow(unsafe_cast, reason = "AndroidApp 经 FFI 边界 = android-activity 约定入口签名")]
pub unsafe extern "C" fn android_main(app: bevy::android::android_activity::AndroidApp) {
    // S49.4d 真机「没有全屏 / 左侧黑条」根因：系统栏未隐藏 → surface 被系统栏压缩 →
    // 游戏 letterbox 有效区 → 黑条（landscape 状态栏在长边侧）。
    // 用 android-activity 原生 `set_window_flags`（0 新依赖、0 JNI，本壳可编译）：
    // - `LAYOUT_NO_LIMITS`：窗口布局延伸屏幕边缘（系统栏下仍铺满）；
    // - `FULLSCREEN`：隐藏状态栏（API 16+，NativeActivity 窗口 attribute）；
    // - `KEEP_SCREEN_ON`：游戏场景防锁屏（manifest 注释「保持唤醒」落地）。
    // 注：导航栏完全隐藏（immersive `SYSTEM_UI_FLAG_HIDE_NAVIGATION|IMMERSIVE_STICKY`）需
    // JNI `setSystemUiVisibility`（android-activity 未暴露），本机无 NDK 无法编译验证 ⇒
    // 先落 window flags（状态栏 + 布局），导航栏若仍悬浮后续补 JNI（需真机验证）。
    use bevy::android::android_activity::WindowManagerFlags;
    app.set_window_flags(
        WindowManagerFlags::LAYOUT_NO_LIMITS
            | WindowManagerFlags::FULLSCREEN
            | WindowManagerFlags::KEEP_SCREEN_ON,
        WindowManagerFlags::empty(),
    );
    let _ = bevy::android::ANDROID_APP.set(app);
    super::run();
}
