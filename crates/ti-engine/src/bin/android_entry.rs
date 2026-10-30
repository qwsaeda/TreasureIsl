//! S18.8d：Android JNI 入口（`android-activity` 壳 crate 编译期 `include!` 本文件，
// 与 `treasure_bevy.rs` 的 `run()` 同一 bin 编译单元 ⇒ 0 重复链接）。
//
// 形态 = `bevy_main` 宏展开同构：`android_main(AndroidApp)` 注入 `bevy::android::ANDROID_APP`
// 后走 native 相同的 `run()`（资源读降级 + 地图包内嵌默认，见 `treasure_bevy.rs` android 注释）。

/// JNI 入口（`android-activity` 的 `JNI_OnLoad` 把 NativeActivity 转交此函数）。
#[unsafe(no_mangle)]
#[allow(unsafe_code, reason = "android-activity JNI 入口须 no_mangle（winit 壳约定，bevy_main 同形态）")]
#[allow(unsafe_cast, reason = "AndroidApp 经 FFI 边界 = android-activity 约定入口签名")]
pub unsafe extern "C" fn android_main(app: bevy::android::android_activity::AndroidApp) {
    let _ = bevy::android::ANDROID_APP.set(app);
    super::run();
}
