// S31.2c：Bevy 浏览器壳入口（build_wasm.ps1 部署到 web/main.js；勿手改，改源 = 改本文件）。
// 职责：显式选包（?map=<id> / #map=<id> → window.ti_map_requested；壳层 apply_map_pack
//       经 js_sys::Reflect 读回）+ wasm-bindgen 初始化（--target web 胶水，default export
//       = init；实例化后 Rust `main` → Bevy 主循环自动驱动，winit web 后端绑定 #game canvas）。
import init from "./treasure-bevy-wasm.js";

const params = new URLSearchParams(location.search);
const hashMap = location.hash.match(/^#map=(\S+)$/);
const requested = params.get("map") || (hashMap ? hashMap[1] : null);
if (requested) {
  window.ti_map_requested = requested;
}

// S87.7 浏览器实时预览：编辑器 doPreview() 把 {scene, mode, schema} 写入 sessionStorage，
// 这里在 init() 前**同步**读到 JS 全局，供 wasm 壳 Reflect::get → install_injected
// （必须在 wasm `main` 实例化、首次表访问之前；iframe 每次 reload = 新 OnceLock）。
// 缺数据 = 普通游戏启动（window 全局不设 ⇒ wasm 走内嵌默认，0 行为变化）。
const previewScene = params.get("scene");
if (previewScene) {
  window.ti_preview_scene = previewScene;
}
if (params.get("preview") === "1") {
  try {
    const inject = sessionStorage.getItem("ti_preview_inject");
    if (inject) window.ti_preview_inject = inject;
  } catch (e) {
    console.warn("[preview] sessionStorage 不可用：", e);
  }
}

const status = document.getElementById("status");
const canvas = document.getElementById("game");
canvas.addEventListener("webglcontextlost", (e) => {
  e.preventDefault();
  status.textContent = "WebGL context lost（刷新页面恢复）";
});

init()
  .then(() => {
    status.textContent =
      "运行中（方向键/空格摇骰，E 据点，Enter 确认，Back 返回；鼠标/触摸点击）";
  })
  .catch((err) => {
    status.textContent = "加载失败：" + String(err);
    console.error(err);
  });
