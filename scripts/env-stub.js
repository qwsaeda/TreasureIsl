// env-stub.js：wasm "env" 模块占位（miniquad 遗留宿主 FFI；S18.8b/c 根除后删除）。
// ti-platform 非 optional 依赖 miniquad ⇒ wasm 链接出 5 个 env 导入（实测）：
//   sapp_set_cursor / sapp_set_cursor_grab / sapp_set_fullscreen /
//   fs_get_buffer_size / fs_take_buffer
// Bevy 主循环（winit/wgpu）不经过这些函数；stub 仅供 WebAssembly 实例化链接通过。
// 签名与 miniquad wasm.rs 宿主桥一致（no-op 即可，永不执行）。
export const sapp_set_cursor = () => {};
export const sapp_set_cursor_grab = () => {};
export const sapp_set_fullscreen = () => {};
export const fs_get_buffer_size = () => -1;
export const fs_take_buffer = () => {};
