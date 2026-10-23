# 飞轮 R1 · 修复 agent 回报 —— 主题「渲染数据/图片分离」

输入：`evidence/flywheel_render_r1_audit.md`（38 真漂移；排除 4 条需人裁决 ①-01/①-12/④-04/⑤-04；排除 27 条计划内 S21.x）。
执行顺序：批 A（工具链根因）→ 批 B（数据化/单源）→ 批 C（机械重构）；批 B/C 按硬规则 8 批量编辑、一次编译、报错一次性收口。

## 批 A（工具链根因：⑤-05 + ⑤-06 + ⑤-01/02/03/07）

- [⑤-05] 已修 | `scripts/check_hardcode.py:31` STR_LIT 改 `r'"((?:[^"\\]|\\.)*)"'`（捕获组 = 完整字面量；旧式只留末字符，CJK 通道对「以 `）`/ASCII 结尾」的中文字面量失明）；`scripts/check_hardcode.py:118` 新增 `--selftest` 子命令（169 行 argparse 接线：5 条 fixture 断言 = CJK 以全角标点结尾必须命中 + 全捕获 + 转义引号 + 纯 ASCII 不误报，回归即非 0） | 自验：`python scripts/check_hardcode.py --selftest` = `selftest OK`；修复后复扫全仓 = 64 条 CJK 命中（与审计 §实测记录 3 逐文件逐行一致）
- [⑤-06] 已修 | `scripts/check_hardcode.py` EXEMPT 重刷：`game_app.rs` → `game_app/`（S20.0b 拆分后子串失配）；新增 `scenes/main_map.rs`（[stats] 门后诊断）、`scenes/mod.rs`（scene_transition! 宏第 3 参 = doc-only 非玩家可见，亲验 `#[doc = $doc]` 展开）、`bevy_app.rs`（spike 壳标题，⑤-04 留人裁决后再收紧）、`treasure_bevy.rs`（离屏/selftest/shot 诊断 + 窗口标题）、`android_shell.rs`（[hd] 资产注册诊断）、`sprite_step.rs`（debug bin 终端输出）、`data/mod.rs`（balance LoadError 措辞）、`hd_png.rs`/`ogg_encode.rs`/`ids.rs`（格式解析错误诊断），逐条带理由 | 自验：严格模式 `check_hardcode: OK (153 file(s), 0 hits)` = 64 条新暴露全部覆盖，逐条对照实测清单均为诊断/doc 非玩家 UI，无一条需迁 strings.json
- [⑤-01] 已豁免 | `game_app/board.rs` ×9 由 EXEMPT `game_app/` 行覆盖 | 自验：同上 0 hits
- [⑤-02] 已豁免 | `game_app/host.rs` ×2 + `scenes/main_map.rs` ×3 由新增 EXEMPT 行覆盖 | 自验：同上
- [⑤-03] 已豁免 | `treasure_bevy.rs` ×10（含 :394）由新增 EXEMPT 行覆盖 | 自验：同上
- [⑤-07] 已豁免 | `scenes/main_map.rs` ×3 `[stats]` 诊断 | 自验：同上
- 台账登记 | `docs/process/14_tooling_catalog.md` check_hardcode 行补 `--selftest`（STR_LIT 全捕获 + 非 0 退出语义）；台账行契约（幂等/非0/只读模式）不变，doccheck2 工具契约项维持 0 error | 自验：gate 27 项 `check_tooling_contract` PASS

## 批 B（数据化/单源；像素语义零变化：值照抄原字面量，只搬家）

- [③-01] 已迁 | `scenes.json` `vs.fx_hit {x:540, y:880}`（值 = 原 `FX_BATTLE_HIT_POS` 常量）；`game_app/input.rs:16` `fx_pos()` 读键（键缺失 = (0,0) crash-safe，①-07 族口径）；`game_app/mod.rs` 常量删除；键并入 `chrome::keys::FX_HIT` | 自验：test/clippy 全绿
- [③-02] 已迁 | `scenes.json` `main_map.fx_coin {x:900, y:140}`；`input.rs:138` 读键；`chrome::keys::FX_COIN` | 自验：同上
- [③-17] 已迁 | `crates/ti-engine/src/canvas.rs:43/50` 裸 540.0/960.0 → `LOGICAL_W_I32 as f32 / 2.0` / `LOGICAL_H_I32 as f32 / 2.0` 派生（1080 单源契约 1） | 自验：见 ⑥-01 行 doccheck2 断言
- [③-18] 已迁 | `treasure_bevy.rs:238/245/336/337` `[540.0, 960.0]`×4 与 `cam.cam[0]-540.0` → 本壳 `LOGICAL_W/H`（改派生自 `ti_render::framebuffer`）半值 | 自验：同上
- [⑥-01] 已迁 | 引擎侧 2 文件 6 处归一并案；doccheck2 新断言 `check_engine_no_bare_half`（`crates/ti-tools/src/bin/doccheck2.rs`：ti-engine/src 禁裸 540.0/960.0 + 派生式在位双守卫） | 自验：doccheck2 30 项 0 error
- [⑥-05] 已迁 | `treasure_bevy.rs:54-58` `LOGICAL_W/H = 1080/1920` 字面量 → `ti_render::framebuffer::LOGICAL_W_I32 as u32` 派生；`WIN_W/H` 改 `LOGICAL_W/H / 2` 派生（同批顺手，未单独立项） | 自验：wasm 构建 + clippy 绿
- [⑥-02] 已迁 | `ui_icons.json` 顶层 `default_size: 48`（单源）；`ti-core/ui_icons.rs` `UiIcons.default_size` 字段（serde 兜底 48）；`chrome.rs:707` 与 `scene_render.rs:337` 双执行器改读注册表（缺失 = 48 crash-safe）；`shell.rs` `compact_menu_item` 改 `icons::ICON_SIZE`；`icons.rs:14` doc 标注框架侧单源；doccheck2 `check_render_single_source` 断言 `ui_icons.json default_size == icons::ICON_SIZE` | 自验：doccheck2 0 error
- [⑥-03] 已迁 | 抽 `chrome::layout_icon_label(host, rect, label, icon_opt)` 单一公式函数（`chrome.rs`），`deck_button` 与 `main_map.rs:100` 双调用点收口；面板语言 3px 选中边 = `chrome::PANEL_SELECTED_BORDER_PX` 单参源（select_role/saveload 双点改引用） | 自验：test/clippy 全绿
- [⑥-07] 已修 | `select_role.rs:136` `0..5` → `0..ROLE_COUNT as usize`；doccheck2 新断言 `check_select_role_render_loop`（`0..5` 0 命中 + `0..ROLE_COUNT` 在位；补 EXP-073 band 门禁盲区） | 自验：doccheck2 0 error
- [①-04] 已迁 | `chrome.rs` `MOTION_PAGE_BRIGHTNESS_MIN/SWING: f32 = 0.6/0.4` 命名常量（`page_brightness_factor` 改引用）；`gate_data.json ui_motion.f32_params/f32_doc_row` 登记；`10_ui_redesign.md §7` 表加「翻页亮度（f32 参组）」行（时长 4 参 + f32 组 = 5 参）；doccheck2 `ui_motion_f32_check` 扩断言（f32 值对账代码常量 + §7 行登记，主 checks 计数串 27→30 同步 + §8 说明更新） | 自验：doccheck2 0 error
- [④-05] 已迁 | 星位表数据化：`scenes.json index.ambient_art.stars`（26 项 `x/y/size/bright`）+ 数据层 `ti-core/data` `AmbientStar`/`SceneLayout.ambient_stars`（缺 = 空 Vec = 不画）；`ti-render/shell.rs` 删 `STARS` 常量改 `Star` 入参 + `star_sky` 只遍历；`chrome::star_sky` 读数据 | 自验：shell 测试 fixture 改写 + 全绿
- [①-32] 已迁 | 屏心净空参数入 `scenes.json index.ambient_art {center_x:50, center_y:52, exclude:12}`（`chrome::keys::AMB_*`），`star_sky` 入参化（0 = 无净空 crash-safe） | 自验：同上
- [③-13] 已迁 | `scenes.json select_map.locked {inset:6, max_w:40, min_w:4, h:12}`；`select_map.rs` 锁块 4 个几何参数改读键（键缺失/非正 = 不画，沿用 QA033 语义） | 自验：test 全绿
- [③-14] 已迁 | `scenes.json select_map.stars {size:14, gap:6, inset:12, top:10, max:3, bevel_px:2}`；`select_map.rs` 难度星 6 参数改读键（`max` 仅几何上限，星数钳制规则层不动） | 自验：同上
- [②-07] 已迁 | `scenes.json message.body {clip_reserve:48, line_h:52, x_pad:4, edge_pad:8}` + `message.page_dots {size:12, gap:36, dy:48, inner:8}`；`message.rs` 正文折行/页点几何全读键（`inner` 独立键保 12/8 原像素，不派生） | 自验：同上
- [③-15] 已迁 | `scenes.json saveload.list.summary_dy: 52`；数据层 `ListLayout.summary_dy`（ListJson/解析 + touch.rs 两处测试字面量补 0）；`saveload.rs:151` `ty + 52` 改读 `list.summary_dy` | 自验：同上
- 8pt/豁免耦合 | doccheck2 `ui_8pt_walk` 豁免表扩 `fx_hit/fx_coin/body/page_dots/locked/stars/summary_dy`（飞轮 R1 数据化新键，同 v3 叙事键口径）+ 错误文案同步 | 自验：doccheck2 0 error
- 人读镜像 | `docs/business/08_layout_design.md §4` 补新键行（index 星位表 26 项 / select_map locked+stars / main_map fx_coin / vs fx_hit / message body+page_dots / saveload summary_dy）——`check_layout_doc_table_cross_check` 只查 md 不改 JSON | 自验：doccheck2 0 error

## 批 C（机械重构：无像素语义变化；全批改完一次编译收口）

- [①-13] 已迁 | `treasure_bevy.rs:195` 全屏底色 `Color::srgb(0.1,0.07,0.05)` → `rgb565_to_color(Theme::DEFAULT.parch_dark)`（§5.1 权威口径：运行时 theme ≡ DEFAULT；CPU 侧清屏同色，双壳底色同源——此条按审计裁决属「修漂移」，Bevy 底色棕→navy 的像素差即修复目标） | 自验：bevy-cpu-parity PASS
- [①-23] 已修 | `shell.rs` `night_sky_gradient` 亮度曲线 `550/450` → `NIGHT_SKY_BASE_PERMILLE/SWING_PERMILLE` 命名常量 | 自验：像素公式逐位等价（同一千分比表达式），test 全绿
- [①-24] 已修 | `shell.rs` `moon_glow` 光晕斜坡 `120/880` → `MOON_GLOW_BASE_PERMILLE/SWING_PERMILLE` | 自验：同上
- [①-25] 已修 | `shell.rs` `ocean_horizon` 波浪 `step_by(4)/%3/-1` → `WAVE_STEP/WAVE_PERIOD/WAVE_AMPLITUDE`（step_by 经 `usize::try_from` 保类型） | 自验：同上
- [①-26] 已修 | `shell.rs` `ornate_frame` 32 辐条/`cy=y0-24`/4px 木框/8px 金线内缩 → `COMPASS_SPOKES/COMPASS_RISE/ORNATE_FRAME_PX/ORNATE_GOLD_INSET` | 自验：同上
- [①-27] 已修 | `shell.rs` `compact_menu_item` `48i32.min(height-8)` → `crate::icons::ICON_SIZE.min(height-8)`（随 ⑥-02 单源批） | 自验：同上
- [①-28] 已修 | `generic.rs` glow 衰减 `350 + t*650` → `GLOW_BASE_PERMILLE/GLOW_SWING_PERMILLE` | 自验：同上
- [①-29] 已修 | `generic.rs` `steps.clamp(2,256)` 抽共享 `ti_render::generic::clamp_grad_steps`（双端同读；const 分支写法兼容当前工具链） | 自验：同上
- [①-30] 已修 | `primitives.rs` `parchment_bg` 渐变强度 0.18 + 暗角 36 → `PARCHMENT_GRAD_STRENGTH/PARCHMENT_VIGNETTE_PX` | 自验：同上
- [①-06] 已登记 | `board.rs:21` `TOAST_TICKS`（已是命名 pub const + doc）登记 `gate_data.json ui_motion.tick_params/tick_doc_row/tick_const_file` + `10_ui_redesign.md §7` 表行；doccheck2 `ui_motion_tick_check` 断言（帧制→ms 制归一留子分歧不动） | 自验：doccheck2 0 error
- [①-10] 已修 | `assets.rs:160` + `hd_manifest.rs` `0..5` 五帧写死假设 → `f0..fN` 探测（首个缺帧即止，manifest 驱动帧数） | 自验：test 全绿
- [⑥-11] 已修 | `treasure_bevy.rs:56` `SELFTEST_SEED: u32 = 42` 命名常量（壳内 2 处 `set_rng_seed(42)` 合一）；doccheck2 `check_render_single_source` 断言 `tests/scripts/full_run.json seed == SELFTEST_SEED` | 自验：doccheck2 0 error
- [⑥-04] 已修 | 删 `scene_render.rs known_tokens` 死函数（全仓 grep 零消费方，含 25 名硬编码表 + `token_color` + 未用 `BTreeMap` import） | 自验：grep 0 残留 + 编译绿
- [⑦-10] 已修 | 删 `chrome::scene_shell`（零生产调用，仅注释提及）；`scene_base/scene_frame` 保留 + doc 标注「拆分便利壳」；`chrome::menu_item`/`bar_slot` 转发（ti-render 侧 shell 函数仅 geometry.rs 测试消费，保留）删除 | 自验：grep 0 残留 + 编译绿
- [⑦-11] 已修 | 删 `GameApp.scene_cache` 字段 + 初始化 + `add_scene` + `treasure_bevy.rs`/`render_png.rs`/`android_shell.rs` 三处 `.Map` 注入循环 + `scenes/mod.rs asset_name` 映射表（legacy `.Map` 解析归 `legacy/` 参照，契约 1.3；`mod.rs` 未用 `HashMap` import 清理） | 自验：grep 0 残留 + 编译绿
- [③-16] 已修 | `scenes/mod.rs` `SceneKind::asset_name` 删除随 ⑦-11；`index.rs:95` 裸 `"status"` 键并入 `chrome::keys::STATUS`；无效 `status.x + 2` 传参删除（`hint_row` 忽略 x，改 0 传参 = 无死偏移） | 自验：grep 0 残留

## 顺带修复（非靶点 38 条范围，归因确认既有）

- `ti-app` `android-shell + hd-8bit` 组合既有编译断链（`hd_manifest.rs` 缺 async `load_hd_pngs_from_manifest`，`git stash` 后 pristine 树复现 = 非本轮引入；gate 组合不含该 feature 对）：本轮补回 async 加载器（含 ①-10 同口径探测循环）+ `android_shell.rs` fetch 闭包生命周期修复（`name.to_owned()` 进 async block），`cargo check --features android-shell,hd-8bit` 恢复通过。遗留（不扩靶）：该组合 clippy 有 4 条既有 pedantic 告警（android_shell.rs:29/179/229 未改动旧行，`manual_ok`/`f64→u64` cast/`&Option`，非门禁 feature 对，gate 项 5 不覆盖）。

## 自验输出（全绿后回写，§3 表口径）

- `cargo test --workspace` = 全过（ti-render shell fixture 改写：star_sky 3 星 fixture，暗星 1 + 亮星 5 + 屏心净空跳过 = 6 填充）
- `cargo clippy --workspace --all-targets -- -D warnings` = 0（wasm 侧 `cargo clippy --manifest-path crates/ti-engine/Cargo.toml --target wasm32-unknown-unknown -- -D warnings` = 0）
- `cargo run -q -p ti-tools --bin doccheck2 .` = `0 error(s) across 30 audit-codified checks`（27→30：新增 `check_engine_no_bare_half`/`check_select_role_render_loop`/`check_render_single_source`；`ui_motion_params_check` 扩 f32/tick 子检查）
- `python scripts/validate_data.py` = OK；`python scripts/check_hardcode.py` = OK (153 files, 0 hits) + `--selftest` OK
- `powershell -ExecutionPolicy Bypass -File scripts/gate.ps1` = **gate: 0 failure(s)**（23 项顶层：20 PASS + android/zero-unknown 条件 SKIP + bevy-cpu-parity PASS）
- 附：`python scripts/audit_code.py --check` = PASS（chrome.rs 901→892 行，压缩本轮新增 doc 后回落 900 上限内）

## 留人裁决（本轮未动，4 条）

- ①-01（9 分区→令牌表归属）、①-12（gold_glow/gold_bright 同值合并 vs 别名标注）、④-04（14 枚矢量图标图片化 vs 现状）、⑤-04（窗口标题 CJK：strings.json `app.window_title` vs 外壳 EXEMPT——本轮先按 audit §⑤-06 口径挂 `bevy_app.rs`/`treasure_bevy.rs` EXEMPT 行，裁决后改走 strings.json 再删豁免行）。
