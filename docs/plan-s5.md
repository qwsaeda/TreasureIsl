# S5 可玩收口 — 计划

目标：把 S3 已 diff 验证的 12 个玩法模块接进 `ti-app` 交互壳，
使游戏**从头可玩到尾**（菜单→选人→选图→主地图→贸易/建造→战斗→存读档），
并补齐 P3-3 全流程 headless 判据、P4-1 渲染对齐、业务推测项收敛。

接线规格见 `docs/gamebook.md`（单一事实源）。数值/规则一律调用
`ti-core::rules` 已验证实现，壳层只编排 UI/输入/渲染。

## 任务分解

### S5.1 交互游戏状态机（GameApp）
- 落 `crates/ti-app/src/game_app.rs`：`GameApp` 顶层场景状态机
  （`Scene` 枚举 + 每场景 `update(输入)` / `render(fb)`，调用各 rules Session）。
- `shell_ui.rs` 的 `event_id` 占位改为 `menu.handle` 真实事件码（对齐 spec/menu.md §a）。
- 两 bin 入口改持 `GameApp`。
- 验收：`cargo test -p ti-app` 全绿 + 桌面 exe 实跑全链路点通。

### S5.2 全流程 headless + full_run.json（P3-3）
- `Cargo.toml` 加 `headless` feature；`tests/scripts/full_run.json` 固定种子脚本（seed=42）。
- `treasure-headless` bin（`--features headless`）：无窗口跑 `GameApp` 逻辑链，逐步喂事件，断言场景流转。
- 验收：`cargo run -p ti-app --features headless --bin treasure-headless -- --script tests/scripts/full_run.json` 16 步 0 失败。

### S5.3 P4-1 逐像素渲染对齐
- `crates/ti-app/tests/render_parity.rs`：HeadlessBackend 渲染关键场景，对比 P0 参考图。
- 验收：`cargo test -p ti-app --test render_parity` 逐像素一致。

### S5.4 业务对齐收敛 + 发布
- 各 spec `推测` 项实测收敛（填证进 facts）；`docs/release.md` 更新。
- `gate.ps1` 全绿 + 桌面/浏览器产物（Android 在 Mac 编译，见 S5.5）。

### S5.5 Android 壳落地（Mac 侧出 APK，本机不编译）
- `ti-app` 加 `android-shell` feature + `treasure-android` bin（`src/bin/treasure_android.rs`）：
  复用共享 `GameApp` 交互核心 + `TouchPad` 触屏输入，资源读 APK assets
  （`macroquad::file::load_file`，与 wasm 的 HTTP fetch 同源机制）。
- `android/README.md`：Mac 侧 cargo-apk 脚手架 + 资源拷贝 + 编译/装机步骤。
- 验收（Mac 侧，非本机 gate）：`cargo apk build -p ti-app --features android-shell` 出 APK，
  装机显示主菜单 + 真字形 + 音效，全链路触屏可玩。本机 Windows gate android 行保持 SKIP。
- 存档 scoped storage 持久化为后续增强项（当前存读档走内存槽位，不阻塞 APK 可玩）。

## 依赖

- S5.1 deps: S3.1, S3.4, S3.5, S3.6, S3.7, S3.8, S3.9, S3.10, S3.11（玩法模块全 verified）
- S5.2 deps: S5.1
- S5.3 deps: S5.1
- S5.4 deps: S5.2, S5.3
- S5.5 deps: S5.1（Android 壳复用 `GameApp`；APK 在 Mac 编译，非本机 gate 阻塞项）

## 终止条件

S5.4 verified = ledger S5 全 verified + P3-3/P4-1 机器判据产出 evidence + gate 全绿。
S5.5 = Android 壳代码 + 脚手架落地（本机可编译 host 形态）；APK 实际产出在 Mac（既定，
开发本身不依赖 Android 工具链，本机 gate android 行 SKIP 为既定状态）。
