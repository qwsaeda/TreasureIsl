//! S18.8d：Bevy Android JNI 壳（cdylib `libandroid_activity.so`）。
//!
//! 编译形态：`include!` 引入 ti-engine 的 Bevy 壳源（`treasure_bevy.rs` 同体），
//! 使本 cdylib 与 native bin 共享同一 `run()` 编译逻辑 ⇒ 单一事实源、0 双壳漂移。
//! 壳内 `#[cfg(target_os = "android")]` 段激活 `android_main(AndroidApp)`
//! （JNI 入口，`JNI_OnLoad` 由 `android-activity` crate 导出并转交）。
//!
//! `android-activity` crate（0.6.0，features `native-activity`）负责：
//! - `JNI_OnLoad` + NativeActivity 生命周期回调导出；
//! - `android_main` 的查找（按符号名，`#[no_mangle]`）与 `AndroidApp` 注入。
//!
//! 资源链：`bevy_asset` android 后端直读 APK assets（`bevy_android::ANDROID_APP`）；
//! 旧 `legacy/data` fs 读在 android 下全缺失降级（`read_local` 返回 None，0 panic）。

// 与 ti-engine bin 同一源文件（`include!` 展开进本 crate 编译单元）：
#[path = "../../../crates/ti-engine/src/bin/treasure_bevy.rs"]
mod treasure_bevy;

// 壳入口重导出（android-activity 按 `#[no_mangle]` 符号查找 `android_main`，
// 同编译单元内 `pub use` 即可；`android_entry` 模块是 `pub(crate)` 不影响 cdylib 导出）
#[cfg(target_os = "android")]
#[doc(hidden)]
pub use treasure_bevy::android_entry::android_main;
