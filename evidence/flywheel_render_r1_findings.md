# 飞轮 R1 · 找问题清单 —— 主题「渲染数据/图片分离」（定向飞轮 §8 主题卡）

- 轮次：第 1 轮 · 找问题 agent（只读）
- 代码面：`crates/ti-app/src/scenes/**`、`crates/ti-app/src/game_app/**`、`crates/ti-render/src/**`、`crates/ti-engine/src/**`
- 判定口径：`check_hardcode.py` EXEMPT 表内不算；几何算法本身不报（算法**参数**报）；`docs/ui-redesign-v3/08_main_map_v4_spec.md`（S21.3 定案）§8 v3→v4 差异表作为 ⑦ 判据。
- 数据面建议落点：`scenes.json`（坐标/尺寸/显隐键）/ `theme.json`（色板/字号/样式参数）/ `terrain.json`（地形显隐）/ `balance.json`（数值）/ `assets/ui/*.png`（该图片化处）。

## 顶部统计

| 类别 | 条数 | 高 | 中 | 低 |
|---|---|---|---|---|
| ① 色值/数值字面量 | 33 | 1 | 14 | 18 |
| ② 画/不画决策 | 8 | 1 | 3 | 4 |
| ③ 尺寸/坐标/偏移常量 | 18 | 2 | 11 | 5 |
| ④ 该用图片处用代码画 | 7 | 3 | 3 | 1 |
| ⑤ 中文面量（含 2 工具链根因） | 7 | 2 | 2 | 3 |
| ⑥ 跨文件重复常量 | 12 | 1 | 5 | 6 |
| ⑦ v2/v3 残留（含 1 门禁耦合） | 12 | 1 | 6 | 5 |
| **合计** | **97** | **11** | **44** | **42** |

> 不足 100 条 = 本主题第一轮未凑满（接近收敛方向；工具链 2 条修好后 ⑤ 类预计新增命中 ≥20 条，下轮可补）。

---

## ① 色值/数值字面量

- [①-01] `crates/ti-app/src/scenes/chrome.rs:319` | ① | `const TOKENS: [&str; 9] = ["land_sand","reef","sea_shallow","gold_warm","moon_pale","brass","secondary","hilite","danger"];`（`group_color_token`，9 分区→主题令牌映射表写死在 Rust） | 数据化建议：`balance.json` 增 `board.group_tokens: [9 令牌名]`（或 `terrain.json legend.group_tokens`），代码只做 `token = tokens[group-1]` dispatch；赛道标记（main_map.rs）与航程条（chrome::voyage_bar）共用同一数据源。严重度：高
- [①-02] `crates/ti-app/src/scenes/main_map.rs:402` | ① | `chrome::entity_icon(host, "beacon", lx, ly, s, ti_render::theme::Theme::DEFAULT.moon_pale)` | 据点图标 tint 用**编译期** `Theme::DEFAULT` 而非运行时 `host.theme().by_name("moon_pale")` → theme.json 改色对本处无效。严重度：中
- [①-03] `crates/ti-app/src/scenes/main_map.rs:500` | ① | `SpaceKind::Event => Some(("book", ti_render::theme::Theme::DEFAULT.moon_pale)), SpaceKind::Card => Some(("map", ti_render::theme::Theme::DEFAULT.moon_pale))` | 同上：icon tint 表（`render_track` 内 8 种 SpaceKind→(图标名, tint) 表，:496-505）整体写死；建议 `scenes.json main_map.track_icons: {kind: {icon, tint_token}}`，tint 走 theme 令牌。严重度：中
- [①-04] `crates/ti-app/src/scenes/chrome.rs:531` | ① | `if left >= MOTION_PAGE_MS { return 0.6; } ... 0.6 + done * 0.4`（翻页亮度下限 0.6/回升幅 0.4 写死） | 动效参数应入 `theme.json` motion 组（如 `motion.page_brightness_floor: 0.6`）并同步 `10_ui_redesign.md` §7 表 + doccheck2 断言（MOTION_* 已有此机制，本处 0.6/0.4 是未登记的第 5 个参数）。严重度：中
- [①-05] `crates/ti-app/src/scenes/chrome.rs:360` | ① | `let panel = theme.by_name("brass_dark").unwrap_or(COLOR_GOLD_WARM);`（voyage_bar/wood_panel/toast 等 ≥5 处 `unwrap_or(COLOR_GOLD_WARM)` 编译期兜底色） | 令牌缺失兜底色应单源：`theme.json` 增 `fallback_token` 或统一 `Theme::fallback(name)`；当前 `unwrap_or` 散落 main_map.rs:461/485/548 等处。严重度：低
- [①-06] `crates/ti-app/src/game_app/board.rs:21` | ① | `const TOAST_TICKS: u32 = 120;`（落格提示条时长 120 帧写死在代码） | 视觉时长参数 → `theme.json` `motion.toast_ticks`（或 ms 制对齐 S17.4 动效表）；`hold_board_toast` 出图口径随之数据化。严重度：中
- [①-07] `crates/ti-app/src/game_app/host.rs:81` | ① | `fn terrain_grid_w(&self) -> u32 { self.terrain_map.as_ref().map_or(20, |t| t.grid_w) }`（缺省网格宽 20 魔法数） | → `balance.json` 增 `board.fallback_grid_w`（或 terrain.json 缺图兜底键）；saveload.rs:40 注释也引用该 20。严重度：中
- [①-08] `crates/ti-app/src/game_app/mod.rs:410` | ① | `let count = data::balance().map_or(5, |b| b.save.slot_count) as usize;`（存档槽数兜底 5） | 兜底值入 `balance.json save.slot_count_fallback`；与 balance 加载失败口径一致（0/明确回退）。严重度：低
- [①-09] `crates/ti-app/src/game_app/mod.rs:285` | ① | `pub const FX_SEED: u64 = 0xC0_FF_EE;`（FX 粒子 LCG 魔法 seed） | 确定性 seed 入 `theme.json fx.seed`（或 balance.json），文档化 1 处；当前注释即唯一事实源。严重度：低
- [①-10] `crates/ti-app/src/game_app/assets.rs:160` | ① | `for k in 0..5u32 { let name = format!("assets_hd_pack/1x/ui/{f1}_hd_f{k}.png"); ...`（多帧精灵恒按 5 帧假设） | 帧数应读 manifest（`multiframe_f1` 的帧数表）而非字面 5；与 doc「f0..f4（5 帧）」双写即漂移源。严重度：低
- [①-11] `crates/ti-app/src/scenes/chrome.rs:293` | ① | `return (bg.tw * ratio / 100).max(4);`（实体 size 下限 4px 魔法数） | → `theme.json layout.entity_min_px` 或 scenes.json `canvas_entities.min`。严重度：低
- [①-12] `crates/ti-render/src/theme.rs:161` | ① | `gold_glow: 0x4E3D,` / `gold_bright: 0x4E3D`（两令牌同值，语义模糊） | 数据面问题（theme.json 镜像同值）：要么拆值，要么合并令牌并删 1 个（`by_name` 37 令牌表随之收敛）；doccheck2 `check_ui_redesign_tokens` 应断言「同值令牌对须显式标注 alias」。严重度：低
- [①-13] `crates/ti-engine/src/bin/treasure_bevy.rs:188` | ① | `Sprite::from_color(Color::srgb(0.1, 0.07, 0.05), ...)`（底图层底色硬编码 sRGB，不走 theme.bg 令牌） | → `theme.bg`（RGB565）经 `rgb565_to_color`；底色属渲染决策数据。严重度：中
- [①-14] `crates/ti-engine/src/bin/treasure_bevy.rs:230` | ① | `g.camera_zoom_at([540.0, 960.0], 1.25, ...)` / `:233 ...0.8...`（zoom 步进 1.25/0.8 写死） | 相机步进入 `theme.json camera.zoom_step_up/down`（或 balance.json）。严重度：低
- [①-15] `crates/ti-render/src/shell.rs:12` | ① | `let f = 12i32;`（`wood_frame` 12px 边框宽写死） | → `theme.json frame.border_w`；`star_sky/night_sky/ocean_horizon/ornate_frame` 的 12px 夹取（:194-197/:381-384/:430-431/:462-465）同改同源。严重度：中
- [①-16] `crates/ti-render/src/shell.rs:76` | ① | `host.fill_rect(left, top, width, 8, theme.menu_border);`（`menu_item` 8px 卡片描边 + 2px bevel + 8×8 铆钉 @ +24/-32，:76-93） | 样式参数组 → `theme.json menu_item {border_w:8, bevel_px:2, rivet_px:8, rivet_inset:24}`。严重度：中
- [①-17] `crates/ti-render/src/shell.rs:100` | ① | `host.fill_rect(x + 4, y + 4, w - 8, 2, theme.bevel_shadow);`（`bar_slot` 4px 内缩 + 2px 阴影） | → `theme.json bar_slot {inset:4, shadow_px:2}`。严重度：低
- [①-18] `crates/ti-render/src/shell.rs:106` | ① | `panel` 2px 木边 + 1px bevel（:106-111） | → `theme.json panel {border_px:2, bevel_px:1}`（与 `chrome::wood_panel` 的 2px 黄铜边同族参数）。严重度：低
- [①-19] `crates/ti-render/src/shell.rs:118` | ① | `let w = host.hint_text_width(text) + 8; let h = host.hint_glyph_height() + 4; ... fill_rect(cx, y - 2, ...)`（`hint_row` +8 衬边/+4 高/-2 偏移/1px bevel） | → `theme.json hint_row {pad:8, extra_h:4, bevel_px:1}`。严重度：低
- [①-20] `crates/ti-render/src/shell.rs:230` | ① | `title_art_block`：12px 内缩 + 2px 双描边 + 8px 饰线内缩 + 罗盘 56×3/3×56/8×8 + `ih >= 96` 门槛（:230-257） | 组件样式组 → `theme.json title_art {inset:12, border_px:2, line_inset:8, compass:{w:56,arm:3,center:8}, compass_min_h:96}`。严重度：中
- [①-21] `crates/ti-render/src/shell.rs:274` | ① | `battle_arena`：2px 发丝框 + 16px 内缩 + `h*2/3` 饰线 + 阶梯 `h/14` + 宽比 `(9,7,5)/10` + 底座 `h-16` + 罗盘臂 28（:274-308） | 组件样式组 → `theme.json battle_arena {frame_px:2, inset:16, line_frac:"2/3", tier_step_div:14, tier_widths:[9,7,5], base_inset:16, compass_arm:28}`。严重度：中
- [①-22] `crates/ti-render/src/shell.rs:319` | ① | `world_canvas_frame`：2px 外框 + 6px 内金线内缩 + 12×12 四角罗盘方位块（:319-343） | → `theme.json world_canvas_frame {frame_px:2, inner_inset:6, corner:12}`（S21.3 v4 画布框 2px 黄铜同族参数）。严重度：中
- [①-23] `crates/ti-render/src/shell.rs:376` | ① | `night_sky_gradient`：`const DITHER: [u32; 4] = [0, 1, 0, 1];` + `let k = 550 + idx * 450 / steps_u + DITHER[...]`（55%→100% 亮度曲线 + 抖动相位写死；:388 `steps.clamp(8, 2048)`） | 渐变视觉参数 → `theme.json sky {base_permille:550, span_permille:450, dither:[0,1,0,1], steps_min:8, steps_max:2048}`。严重度：中
- [①-24] `crates/ti-render/src/shell.rs:405` | ① | `moon_glow`：`const LAYERS: i32 = 16;` + `let k = 120 + (i+1)*880/16`（光晕层数/亮度斜坡写死） | → `theme.json moon {layers:16, base_permille:120, span_permille:880}`。严重度：低
- [①-25] `crates/ti-render/src/shell.rs:436` | ① | `ocean_horizon`：`for x in (12..LOGICAL_W_I32 - 12).step_by(4) { let wave_y = y0 + ((x / 4) % 3) - 1;`（波浪步长 4 / 相位模 3 / ±1 写死） | → `theme.json ocean {wave_step:4, wave_phase_mod:3, wave_amp:1}`。严重度：低
- [①-26] `crates/ti-render/src/shell.rs:455` | ① | `ornate_frame`：`const CORNER: i32 = 16; const ARM: i32 = 20;` + 4px 木框 + 8px 金线内缩 + 32 辐条玫瑰 + `cy = y0 - 24`（:455-503） | → `theme.json ornate {corner:16, arm:20, frame_px:4, line_inset:8, rose_steps:32, rose_offset:24}`。严重度：低
- [①-27] `crates/ti-render/src/shell.rs:528` | ① | `compact_menu_item`：`let icon_size = 48i32.min(height - 8);` + 2px 描边 + `icon_x = left + 8` | → `theme.json compact_menu {border_px:2, icon_box:48, icon_inset:8}`。严重度：低
- [①-28] `crates/ti-render/src/generic.rs:143` | ① | `let level = scale_rgb565(color, 350 + u32::try_from((t * 650.0) as i64)...)`（glow 衰减 35%→100% 区间写死） | → 原语默认参数入 `theme.json glow {att_min:350, att_span:650}`（千分比）。严重度：低
- [①-29] `crates/ti-render/src/generic.rs:113` | ① | `let steps = steps.clamp(2, 256);`（grad 级数钳制区间写死；`scene_render.rs:280` 同式再抄一份） | 钳制区间 → 数据面常量单源（`theme.json grad {steps_min:2, steps_max:256}`，双后端同读）。严重度：低
- [①-30] `crates/ti-render/src/primitives.rs:54` | ① | `gradient_row(host, theme, py, 0.18);` + `:56 vignette(host, theme, 36)`（背景渐变强度 0.18 + 暗角 36px 写死） | → `theme.json bg {gradient_strength:0.18, vignette_px:36}`（parchment_bg 是 v2 回退路径，参数仍应数据化）。严重度：中
- [①-31] `crates/ti-render/src/shell.rs:42` | ① | `nautical_grid`：`for row in 0..13 { let y = 1040 + row * 60; for col in 0..30 { host.fill_rect(60 + col * 32, y, 20, 2, ...`（网格 13 行 ×30 列 + 1040 起点 + 60/32 步长 + 20×2 虚线段全写死） | 装饰层几何 → `scenes.json` 各场景 `nautical_grid` 参数键（或图片化，见 ⑦-03：v4 海图已把经纬网烘焙进 `chart_full`，本函数是 v2 残留装饰）。严重度：中
- [①-32] `crates/ti-render/src/shell.rs:203` | ① | `if fx.abs_diff(50) <= STAR_CENTER_EXCLUDE && fy.abs_diff(52) <= STAR_CENTER_EXCLUDE`（星空屏心净空锚点 50,52 + 半径 12 写死） | → `scenes.json ambient {center_x_pct:50, center_y_pct:52, exclude_radius_pct:12}`。严重度：低
- [①-33] `crates/ti-app/src/scenes/chrome.rs:370` | ① | `let pad = 24;`（voyage_bar 内边距；:367-368 2px 黄铜条 / :372 6px 槽 / :380 点 10/6 / :386 罗盘标 -14,-40,28 / :389 剑标 -12,+14,24 同族魔法数） | → `scenes.json main_map.voyage_bar {pad:24, strip_px:2, slot_h:6, dot_plain:6, dot_special:10, player_marker:{dx:-14,dy:-40,size:28}, boss_marker:{dx:-12,dy:14,size:24}}`。严重度：低

## ② 画/不画决策

- [②-01] `crates/ti-app/src/scenes/main_map.rs:577` | ② | `for r in 0..gw_i { for c in 0..gw_i { ... host.fill_rect(sx, sy, tw*2, th*2, base); }`（`draw_board` 全格绘制，**含海洋格**；v4 规格 §8 明确「只画 陆/礁/宝，海洋格不画 → 巨型蓝菱形消失」） | `terrain.json legend` 增 `hidden: bool`（S/R = true、L/T = false，v4 定案值），代码 `if lg.hidden { continue; }`；显隐 = 数据标志而非代码常量。严重度：高
- [②-02] `crates/ti-app/src/scenes/main_map.rs:386` | ② | `if !spaces.is_empty() { continue; }`（有赛道时不画 balance 据点 —— 视觉去重规则写死在 if 里） | 画/不画 = 数据决策：`scenes.json main_map.landmark_visible_when: "no_track"`（或 terrain/board 数据 `dedup_rule`），代码只做标志判断。严重度：中
- [②-03] `crates/ti-app/src/scenes/main_map.rs:411` | ② | `if spaces.is_empty() { ...BOSS 格... }`（有赛道时不画 BOSS 格标记，同族去重规则） | 同 ②-02：`scenes.json main_map.boss_visible_when: "no_track"`。严重度：中
- [②-04] `crates/ti-app/src/scenes/chrome.rs:353` | ② | `if spaces.len() < 2 { return; }`（voyage_bar 少于 2 格不画，显隐门槛写死） | → `scenes.json main_map.voyage_bar_min_spaces: 2`（键缺失/非正 = 不画的既有判据同族）。严重度：低
- [②-05] `crates/ti-render/src/shell.rs:251` | ② | `if ih >= 96 { ...罗盘... }`（`title_art_block` 内高 ≥96 才画罗盘，视觉规则写死） | → `theme.json title_art.compass_min_h`（同 ①-20 参数组）。严重度：低
- [②-06] `crates/ti-render/src/shell.rs:475` | ② | `if fw > 16 && fh > 16 { ...金线... }`（`ornate_frame` 尺寸门槛写死） | → `theme.json ornate.line_min_size`。严重度：低
- [②-07] `crates/ti-app/src/scenes/message.rs:92` | ② | `if y_cursor > body.y + body.h - 48 { break; }`（正文溢出「不画越界行」的 48px 门槛写死） | → `scenes.json message.body.clip_reserve: 48`（或 8pt 栅格对齐值）。严重度：低
- [②-08] `crates/ti-app/src/scenes/main_map.rs:316` | ② | `if v3 { chrome::render_primitives(host, "main_map", -1); } else { chrome::parchment_bg(host); chrome::nautical_grid(host); }`（v3/v2 双轨画法选择 = `world_canvas` 键在位与否，v2 支路为退役残留） | v4 定案后 v2 支路应整体删除（S21.6），画/不画统一走 `scenes.json main_map.primitives`；过渡期双写已使「画哪套 chrome」成为代码 if 而非数据。严重度：中

## ③ 尺寸/坐标/偏移常量

- [③-01] `crates/ti-app/src/game_app/mod.rs:281` | ③ | `pub const FX_BATTLE_HIT_POS: (f32, f32) = (540.0, 880.0);`（战斗命中火花触发坐标写死；注释「视觉验收可微调」= 典型待数据化参数） | `scenes.json vs.fx_hit: {x:540,y:880}`（或 theme.json fx 组）；input.rs:127 消费处改读数据。严重度：高
- [③-02] `crates/ti-app/src/game_app/mod.rs:283` | ③ | `pub const FX_COIN_GOLD_POS: (f32, f32) = (900.0, 140.0);`（金币光触发坐标写死；应锚定 `hud_gold` 面板） | `scenes.json main_map.fx_coin` 或直接引用 `hud_gold`/`hud_plaque_l` 矩形（v4 = `{48,48,448,192}` 中心）。严重度：中
- [③-03] `crates/ti-app/src/scenes/main_map.rs:526` | ③ | `let rx = 540 - host.shell().hint_text_width_px(&roll_str) / 2; host.draw_hint_text(&roll_str, rx, 1296, ...)`（骰点数 x 用裸 540 屏心 + y=1296 写死；v4 定案 = CTA 上沿 y≈1136） | `scenes.json main_map.roll_text: {anchor:"deck_spin.top", y:1296→1136}`（v4 随 S21.7 落码改数据）。严重度：中
- [③-04] `crates/ti-app/src/scenes/main_map.rs:541` | ③ | `let pad = 24; let panel_w = (tw + pad * 2).max(240); let panel_h = 72; let px = 540 - panel_w / 2; let py = 1520 - panel_h / 2;`（落格提示条 540 屏心/y1520/最小宽 240/高 72/边距 24 全写死；:555-556 `540`、`1520` 再抄 2 次；v4 定案 `{344,1600,392,64}`） | `scenes.json main_map.board_toast: {y:1600, min_w:392, h:64, pad:24}`，代码读键计算居中（v4 值 S21.7 落码）。严重度：高
- [③-05] `crates/ti-app/src/scenes/main_map.rs:123` | ③ | `chrome::draw_body_text(host, &gold_str, rc.x + 8, rc.y + rc.h / 2 - 22);`（v3 HUD 金币/持有量文本偏移 `+8` / `h/2-22` 写死；:130 同式再抄 1 份） | `scenes.json main_map.hud_gold.text_offset: {dx:8, dy_frac:"mid-22"}`（v4 铜牌 = `hud_plaque_l` 同键换几何）。严重度：中
- [③-06] `crates/ti-app/src/scenes/main_map.rs:144` | ③ | `chrome::draw_body_text(host, &gold_str, layout.hud.x + 4, layout.hud.y + 2);` / `:147 ...layout.hud.y + 56`（v2 HUD 双行偏移 +4/+2/+56 写死） | v2 残留（S21.6 删）；过渡期 → `scenes.json main_map.hud.text_offsets [4,2,56]`。严重度：低
- [③-07] `crates/ti-app/src/scenes/chrome.rs:386` | ③ | `entity_icon(host, "compass", player_x - 14, track_y - 40, 28, COLOR_GOLD_WARM);` / `:389 entity_icon(host, "sword", boss_x - 12, track_y + 14, 24, danger)`（voyage_bar 玩家/BOSS 标记偏移+尺寸写死） | 并入 ①-33 的 `scenes.json main_map.voyage_bar` 参数组。严重度：中
- [③-08] `crates/ti-app/src/scenes/main_map.rs:476` | ③ | `let n = 5;` + `:480-481 x - 1, y - 1, 3, 3`（路径连线 5 段插值 + 3×3 虚线方块写死） | 线密度/虚线尺寸 → `scenes.json main_map.track_line {dash:3, density:5}`。严重度：低
- [③-09] `crates/ti-app/src/scenes/main_map.rs:493` | ③ | `host.fill_rect(cx - 10, cy - 10, 20, 20, color);` + `:494-495` 1px bevel（赛道标记 20×20±10 写死；注释「14×14→20×20」= 改数值必须改码的典型） | `scenes.json main_map.track_marker {size:20, bevel_px:1}`。严重度：中
- [③-10] `crates/ti-app/src/scenes/main_map.rs:507` | ③ | `chrome::entity_icon(host, name, cx - 14, cy - 30, 28, tint);`（节点图标偏移 cx-14/cy-30 + 28px 写死；v4 节点牌 = 图片随 tw 倍率 1:1） | v4 随 S21.5 节点图片化后本行整体退役；过渡期 → `scenes.json main_map.track_icon {dx:-14, dy:-30, size:28}`。严重度：中
- [③-11] `crates/ti-app/src/scenes/main_map.rs:513` | ③ | `entity_texts.push((cx + 12, cy - 8, lbl));`（×2：BOSS/起点标签偏移 +12/-8 写死） | 并入 track_marker 参数组（label_dx/label_dy）。严重度：低
- [③-12] `crates/ti-render/src/shell.rs:24` | ③ | `host.fill_rect(12, 12, LOGICAL_W_I32 - 24, h, theme.wood); ... 16, 16, ...- 32, h - 8 ... 20, 20, 8, h - 16 ... h - 8, ..., 3 ... cx = LOGICAL_W_I32 - 48`（`title_bar` 12/16/20/8/3/48 内缩组 + 罗盘 28×3/3×28/4×4 写死） | → `theme.json title_bar {inset:12, inner_inset:16, accent_x:20, accent_w:8, line_px:3, compass:{inset:48, arm:28, arm_w:3, center:4}}`。严重度：中
- [③-13] `crates/ti-app/src/scenes/select_map.rs:170` | ③ | `let lock_w = (card_right - 6 - lock_x).min(40); host.fill_rect(lock_x, ty, lock_w.max(4), 12, ...)`（锁块内缩 6/最大宽 40/最小宽 4/高 12 写死；`text_x` 已数据化但几何参数仍代码侧） | `scenes.json select_map.locked {inset:6, max_w:40, min_w:4, h:12}`（text_x 同段键）。严重度：中
- [③-14] `crates/ti-app/src/scenes/select_map.rs:183` | ③ | `let size = 14; let gap = 6; let right = r.x + r.w - 12; let top = r.y + 10;` + `:181 .min(3)` + `:191 fill_rect(x, top, size, 2, ...)`（BOSS 难度星 14px/间距 6/内缩 12/顶 10/上限 3 星/2px 高光全写死） | `scenes.json select_map.stars {size:14, gap:6, inset:12, top:10, max:3, bevel_px:2}`。严重度：中
- [③-15] `crates/ti-app/src/scenes/saveload.rs:151` | ③ | `host.draw_hint_text(&summary, layout.list.text_x, ty + 52, chrome::COLOR_SECONDARY);`（摘要行下移 52px 写死） | `scenes.json saveload.list.summary_dy: 52`。严重度：低
- [③-16] `crates/ti-app/src/scenes/index.rs:95` | ③ | `let status = layout.extra.get("status").copied().unwrap_or_default(); ... chrome::hint_row(host, footer, status.x + 2, status.y);`（尾行 +2 偏移写死 + `status` 键未登记 `chrome::keys` 模块（裸字符串）） | 偏移入 `scenes.json index.status_text_dx`；键名并入 `chrome::keys`（键治理）。严重度：低
- [③-17] `crates/ti-engine/src/canvas.rs:40` | ③ | `Vec3::new(cx - 540.0, 960.0 - cy, 0.0)` / `:47 Vec2::new(x as f32 - 540.0, 960.0 - y as f32)`（1080/1920 中心 540/960 裸字面量；`ti_render::framebuffer` 已有 `LOGICAL_W_I32/LOGICAL_H_I32` 却不用） | 改 `LOGICAL_W_I32 as f32 / 2.0`（与 `scene_render::to_world` 同式，双处口径归一）。严重度：中
- [③-18] `crates/ti-engine/src/bin/treasure_bevy.rs:321` | ③ | `st.translation.x = cam.cam[0] - 540.0; st.translation.y = 960.0 - cam.cam[1];` + `:230/:233 [540.0, 960.0]`（相机锚点 540/960 ×4 处裸字面量） | 归一到 `LOGICAL_W_I32/LOGICAL_H_I32` 派生（同 ③-17），锚点 = 画布中心。严重度：低

## ④ 该用图片处用代码画（或该代码动态处写死图片）

- [④-01] `crates/ti-app/src/scenes/main_map.rs:496` | ④ | `let icon = match sp.kind { SpaceKind::Start => Some(("flag", ...)), SpaceKind::Port => Some(("beacon", ...)), ... SpaceKind::Boss => Some(("sword", ...)) }`（赛道节点 = 8 类代码图标 + 代码 20×20 方块；v4 规格 §6 定案：节点 = `assets/ui/node_{start,city,port,event,card,battle,treasure,boss,plain}.png` 多形状实心黄铜牌 `blit_image`） | 节点 kind→资产映射表入 `scenes.json main_map.node_assets`（规格 §6 已定案，S21.5 落码）；代码只 dispatch 图片 + 按 `canvas_entities` 倍率缩放。严重度：高
- [④-02] `crates/ti-render/src/generic.rs:103` | ④ | `pub fn icon(c, kind, x, y, size, color) { let _ = size; crate::icons::icon(c, &Theme::default(), kind, x, y, color); }`（`icon` 原语**丢弃数据侧 `params.size`** + 丢运行时主题改用 `Theme::default()`；而 Bevy 端 `scene_render.rs:337` 却按 size 烘焙 → 双后端不一致） | `size` 是 scenes.json 数据声明值，CPU 端必须消费（缩放/裁剪 48px 盒）；主题参数改传调用方 theme。严重度：高
- [④-03] `crates/ti-engine/src/scene_render.rs:177` | ④ | `fn bake_icon(kind: u32, size: i32, color: u16) -> Image { let raster = Raster::new(icon_px, icon_px); ti_render::icons::icon(&mut canvas, &Theme::default(), kind, 0, 0, color); }`（48px 设计盒图标原样画进 size×size 位图 (0,0)：size<48 裁切、size>48 留白偏移，且忽略主题） | 烘焙时按 `size/48` 缩放几何（或直接用 assets/ui 图片资产，与 ④-01 同管线）；theme 用入参而非 `default()`。严重度：高
- [④-04] `crates/ti-render/src/icons.rs:57` | ④ | `pub fn icon(...) { match kind { ICON_COMPASS => { ring(host, x, y, 32, 4, tint); host.fill_rect(x + 22, y + 12, 4, 24, tint); ...`（14 枚图标 = 上百条写死像素偏移的 fill_rect 矢量；项目已有 `ui_icons.json` 注册表 + S21 图片资产管线，图标几何仍锁在 48px 代码盒） | 图标几何 → `assets/ui/icon_*.png`（`ui_asset_batch.py` 批次化，注册表 `image` 字段）；过渡期至少把 48px 盒 + 笔画宽提为数据（`ui_icons.json` 已登记 size，代码 `def.size` 部分消费）。严重度：中
- [④-05] `crates/ti-render/src/shell.rs:152` | ④ | `pub const STARS: &[(u8, u8, u8, bool)] = &[(8, 9, 1, false), (22, 5, 2, true), ...];`（26 颗星位表（百分数坐标/边长/亮暗）写死在 Rust 常量） | 星位表 = 数据：`scenes.json <scene>.ambient_art.stars`（或 `assets/ui/star_sky.png` 一张星空图，v4 海图同思路「装饰烘焙进图」）。严重度：中
- [④-06] `crates/ti-app/src/scenes/main_map.rs:373` | ④ | `host.fill_rect(px, py, s, s, COLOR_PLAYER); chrome::entity_icon(host, "flag", px, py, s, chrome::COLOR_GOLD_WARM);`（玩家 = 绿方块 + 代码 flag 图标；v4 规格 §8 定案：玩家 = 船，`boat_level` 切 `boat_{1..4}.png` 4 图（S21.12）） | 玩家标记 → `blit_image("ui/boat_N.png")`（缺资产回退 `ship_player.png`，再回退代码 flag = 规格 §6 三级回退链）；方块底色是否保留随 v4 出图定。严重度：中
- [④-07] `crates/ti-app/src/scenes/main_map.rs:377` | ④ | `host.fill_rect(px, py, s, 1, chrome::COLOR_BEVEL_HI); host.fill_rect(px, py + s - 1, s, 1, chrome::COLOR_BEVEL_SHADOW);`（×3 实体块重复：玩家 :377-378 / 据点 :404-405 / BOSS :420-421，同 2 行 1px bevel 模式复制） | 抽 `bevel_box(host, x, y, s)` 单一原语（或面板图片）；同模式见 `shell::panel`/`compact_menu_item`。严重度：低

## ⑤ 中文面量（应走 strings.json i18n / 或诊断通道豁免）

- [⑤-01] `crates/ti-app/src/game_app/board.rs:37` | ⑤ | `eprintln!("[stats] enter_main_map: balance.json 加载失败（硬错误，不 fallback）");`（同类 CJK 诊断共 9 处：:37/:46/:56/:67/:78/:140/:275/:280/:286） | 诊断文本不玩家可见，口径 = 诊断通道：`check_hardcode.py` EXEMPT 补 `game_app/` 路径模式（现 EXEMPT 的 `game_app.rs` 模式 S20.0b 拆分后已失配）；或统一转英文诊断 + 保留 i18n 面量清零。严重度：中
- [⑤-02] `crates/ti-app/src/game_app/host.rs:197` | ⑤ | `eprintln!("start_battle: balance.combat.player 缺失（硬错误，不 fallback）");`（+ :192 `[stats]` 同类；`scenes/main_map.rs:31/:62/:162` 另有 3 处 `[stats] ...（硬报错，不 fallback）`） | 同 ⑤-01：拆分后 `game_app/*.rs`、`scenes/*.rs` 均不在 EXEMPT 覆盖内（若检查器 CJK 通道生效即 FAIL）；需 EXEMPT 模式修订或文本英文化。严重度：中
- [⑤-03] `crates/ti-engine/src/bin/treasure_bevy.rs:440` | ⑤ | `eprintln!("[bevy-wasm] 双渲染期降级形态：资源链待 S18.8 bevy 资产管线（当前无字体/场景，可玩度降级不 panic）");`（同类 CJK 诊断 :559/:568/:574/:740/:756-761 共 8 处） | 诊断 bin 应比照 `render_png.rs`/`element_dump.rs` 的 EXEMPT 先例补 `treasure_bevy.rs` 豁免行（理由：离屏/诊断输出，非游戏内 UI）。严重度：低
- [⑤-04] `crates/ti-engine/src/bin/treasure_bevy.rs:631` | ⑤ | `title: "金银岛 TreasureIsl（Bevy）".to_owned(),`（窗口标题 CJK 面量；`bevy_app.rs:49` 另有 `"金银岛 Bevy S18.1 spike"`） | 应用外壳标题（非游戏内 UI）：EXEMPT 补窗口标题条目，或 `assets/data/strings.json` 增 `app.window_title`（i18n 面量归零口径）。严重度：低
- [⑤-05] `scripts/check_hardcode.py:28`（工具链根因 A） | ⑤ | `STR_LIT = re.compile(r'"([^"\\]|\\.)*"')` + `scan_file` 用 `m.group(1)` 判 CJK —— 捕获组在 `*` 量词内**只保留最后一次迭代**（实测 host.rs:197 命中 `lit len=1` = 字面量**最后一个字符**）⇒ CJK 扫描退化为「字面量末字符是否汉字」；以 `）`/`，`/ASCII 结尾的 CJK 字面量全部漏检（靶点域实测 51 条 CJK 字面量，gate 12 报 0 命中） | 修 `r'"((?:[^"\\]|\\.)*)"'`（非捕获组包一层）；补回归 fixture（CJK 字面量以 `）` 结尾必须 FAIL）。gate 12 CJK 通道当前实际失明。严重度：高
- [⑤-06] `scripts/check_hardcode.py:48`（工具链根因 B） | ⑤ | `("game_app.rs", "cjk", "runtime error eprintln diagnostics ...")` —— EXEMPT 模式 `game_app.rs` 与 S20.0b 拆分后的 `game_app/host.rs`、`game_app/board.rs`（**不含** `game_app.rs` 子串）**失配**；ti-engine bin（`treasure_bevy.rs`）亦无豁免行 | EXEMPT 表按拆分后文件清单重刷：`game_app/`（cjk，理由同上）+ `treasure_bevy.rs`/`bevy_app.rs`（cjk，诊断/窗口标题）。与 ⑤-05 联修，否则修完 A 即 20+ 条新 FAIL 无处豁免。严重度：高
- [⑤-07] `crates/ti-app/src/scenes/main_map.rs:31` | ⑤ | `eprintln!("[stats] TradeBuy: balance.json 加载失败（硬报错，不 fallback）");`（×3：:31/:62/:162；`#[cfg(feature="stats")]` 不影响文本检查器） | 同 ⑤-01/⑤-02 处置（EXEMPT `scenes/` 诊断口径需逐文件声明理由，避免豁免面无限扩大）。严重度：低

## ⑥ 跨文件重复常量

- [⑥-01] 多文件 | ⑥ | `main_map.rs:526/544/555`（540×3）+ `game_app/mod.rs:281`（540.0）+ `canvas.rs:40/47`（540.0/960.0）+ `treasure_bevy.rs:230/233/321/322`（540.0/960.0×4）——屏心 540/960 裸字面量 4 文件 9 处，而 `ti_render::framebuffer::LOGICAL_W_I32/LOGICAL_H_I32` 单源已存在 | 全部改 `LOGICAL_W_I32 / 2` 派生（引擎侧 `LOGICAL_W_I32 as f32 / 2.0`）；`chrome::hint_row`（:516 已 `LOGICAL_W_I32/2`）为正确口径。严重度：中
- [⑥-02] 多文件 | ⑥ | `chrome.rs:676` `param_i64(p, "size").unwrap_or(48)` = `scene_render.rs:337` 同式 = `icons.rs:14` `ICON_SIZE: i32 = 48` = `shell.rs:528` `48i32.min(height - 8)` —— 图标默认边长 48 四处各抄一份 | 单源 = `ui_icons.json` 注册表 `size` 字段（或 theme.json `icon_default_px`）；缺省语义「注册表 size，缺 48」一处实现。严重度：中
- [⑥-03] 多文件 | ⑥ | `chrome.rs:456-461`（deck_button 图标排布：`r.w >= def.size + 8` / `r.x + 8` / `ix + def.size + 8`）与 `main_map.rs:101-105`（render_hud_button 同式逐行复制） | 抽 `layout_icon_label(host, rect, label, icon_def) -> (label_x, icon_opt)` 单一函数（两场景按钮排布同源）；8px 衬边随 ①-33 入数据。严重度：中
- [⑥-04] `crates/ti-engine/src/scene_render.rs:435` | ⑥ | `known_tokens` 硬编码 26 令牌名（bg..black），而 `theme.rs by_name` 有 37 令牌 —— 漏 `brass*/sea_*/land_*/panel_*/gold_warm/wood_warm/moon_pale/sky_*` 等 11 个新令牌（S18.6-S20.2 新增） | 令牌表单源 = `Theme`（派生迭代器/宏生成）；`known_tokens` 改由 theme 反射生成，杜绝「新增令牌忘补表」。严重度：中
- [⑥-05] 多 crate | ⑥ | `framebuffer.rs:7-19`（CANVAS_W/H + LOGICAL_W_I32/USIZE + LOGICAL_H_I32）vs `scene_render.rs:36-38`（f32 1080/1920 另定义）vs `treasure_bevy.rs:50-55`（u32 1080/1920 又一份 + WIN 540/960） | 逻辑坐标单源 = `ti_render::framebuffer`；引擎侧派生 `const LOGICAL_W_F32: f32 = LOGICAL_W_I32 as f32` 而非重抄 1080.0。严重度：低
- [⑥-06] `crates/ti-engine/src/bin/treasure_bevy.rs:137` | ⑥ | `raw.chunks(4).any(|c| c[0] > 8 || c[1] > 8 || c[2] > 8)` —— 黑像素判据 `> 8` 在 :137/:721/:730/:748/:749 五处各抄一份 | 抽 `fn frame_has_content(raw: &[u8]) -> bool`（阈值 8 命名常量 `BLACK_THRESHOLD`，同文件 1 处定义）。严重度：低
- [⑥-07] `crates/ti-app/src/scenes/select_role.rs:136` | ⑥ | `for i in 0..5 { let r = layout.list.item_rect(i); chrome::wood_panel(...); }` —— 字面 5 绕过同文件 `const ROLE_COUNT: u32 = 5`（:12）；`selected_border(..., 3)`（:141）与 `saveload.rs:130` 的 3px 亦各抄一份 | 循环改 `0..ROLE_COUNT`（EXP-073 的 band 数门禁只锁 ROLE_COUNT，字面 5 是其盲区）；面板语言参数（3px 黄铜边）入 `scenes.json` 选中项键（见 ① 组）。严重度：中
- [⑥-08] `crates/ti-app/src/scenes/chrome.rs:26` | ⑥ | `pub const COLOR_XXX: u16 = Theme::DEFAULT.xxx;`（×29 令牌重导出）+ 渲染路径使用点（`main_map.rs:373/395/434`、`vs.rs:70-71`、`select_map.rs:171/190`、`saveload.rs:119/130` 等）——**运行时渲染用编译期镜像常量**，theme.json 改色对这些使用点不生效（唯一事实源 = theme.json + 运行时 Theme） | 使用点改 `host.theme().xxx` / `theme.by_name("xxx")`（voyage_bar 已示范）；`COLOR_*` 重导出块标注为测试/兜底专用（或随 theme.json 三方断言降级删除）。严重度：高
- [⑥-09] `crates/ti-render/src/shell.rs:194` | ⑥ | `.max(12)` 外框夹取在 `star_sky:194-197` / `night_sky_gradient:381-384` / `ocean_horizon:430-431` / `ornate_frame:462-465` 四处各写一份（且与 `wood_frame` 的 12px 宽度 ①-15 互锁） | 抽 `theme.frame_inset`（12）单源常量 + `clamp_to_frame(rect)` 单一函数（与 ①-15/①-31 同源）。严重度：低
- [⑥-10] `crates/ti-app/src/scenes/main_map.rs:377` | ⑥ | 1px bevel 双行（`bevel_hi` 顶 + `bevel_shadow` 底）×3 实体块 + `shell::panel:110-111` + `title_art_block:244-246` + `compact_menu_item` 同模式 | 单一 `bevel_box` 原语（ti-render primitives 层），各调用点 1 行调用。严重度：低
- [⑥-11] `crates/ti-engine/src/bin/treasure_bevy.rs:610` | ⑥ | `game.set_rng_seed(42);`（×2：:610/:626）= `tests/scripts/full_run.json` seed 42 同值各写一份（「固定种子 S5.2 同源」口径） | 种子入 `balance.json`/full_run.json 单源读取（selftest 与出图同读一处），避免「改 seed 忘改另一处」。严重度：低
- [⑥-12] `crates/ti-render/src/generic.rs:113` | ⑥ | `let steps = steps.clamp(2, 256);` = `scene_render.rs:280` 同式（双后端各钳制一次，参数漂移风险） | 钳制区间单源（①-29 数据化后双端同读）；或抽 `ti_render::generic::clamp_grad_steps` 共享函数。严重度：低

## ⑦ v2/v3 残留（`08_main_map_v4_spec.md` 定案待退役 + 旧键/旧画法活着）

- [⑦-01] `crates/ti-app/src/scenes/main_map.rs:489` | ⑦ | v3 赛道画法（20×20 小方块 + 28px 代码图标 + 5 段虚线连接，:473-521）——v4 规格 §8 定案退役：「45 × 20px 小方块 + 28px 图标打架 → 黄铜航线实线 + 45 多形状实心黄铜牌（全显）」 | S21.5 赛道重绘：节点牌 = `blit_image(node_<kind>.png)`（④-01），航线实线替虚线；旧画法随 S21.6/S21.7 删除。严重度：高
- [⑦-02] `crates/ti-app/src/scenes/main_map.rs:140` | ⑦ | `render_hud_v2`（羊皮纸 HUD 双行 + 三按钮，:140-155）+ `scenes.json` `hud_trade/hud_sell/hud_build` + `chrome::keys::HUD_TRADE/SELL/BUILD`（:184-188）——v2 经济按钮键/画法，v4 = `hud_plaque` 顶角双铜牌（规格 §2 替换 v3 的 `hud_gold/hud_player`，v2 键同批退役） | S21.6 版面重排时 v2/v3 HUD 键与函数整体删除；触摸命中（:278-286 的 v2 三键 `hud_hit` 分支）随键删除同步收口。严重度：中
- [⑦-03] `crates/ti-app/src/scenes/main_map.rs:318` | ⑦ | v2 回退支路 `chrome::parchment_bg(host); chrome::nautical_grid(host);`（+ `:338-345` wood_frame/title_bar 支路）——v4 规格 §8：「bg_night_sea_top 照片 + chart_wide 半幅羊皮 + 经纬网/罗盘 rect → `chart_full` 满屏海图（经纬网/罗盘/洋流烘焙进图）」，`nautical_grid`（①-31）代码装饰随之退役 | S21.6 以 `image chart_full` 原语替代后，v2 双写支路（`!v3` 全段）+ `shell::nautical_grid` 删除（`render_hud_v2` 的 `has_world_canvas` 判定一并收口）。严重度：中
- [⑦-04] `crates/ti-app/src/scenes/chrome.rs:344` | ⑦ | `voyage_bar` v3「45 点噪声条 840×120」画法——v4 规格 §8 定案：「→ 1 条航线进度尺 984×48」 | S21.7 交互升级重写为进度尺（玩家船标 + ☠ 端标）；旧 45 点循环 + ①-33 参数组随新键 `voyage_bar {48,1088,984,48}` 落数据。严重度：中
- [⑦-05] `crates/ti-app/src/scenes/main_map.rs:536` | ⑦ | `render_board_toast` 纵向位置 1520（注释自述「面板 1484..1556」）——v4 规格 §2 定案 `落格提示签条 {344,1600,392,64}`（y=1520→1600，「落在三键下沿 1576 之下」） | S21.6/7 改读 `scenes.json main_map.board_toast` 键（③-04），1520 魔法数退役。严重度：中
- [⑦-06] `crates/ti-app/src/scenes/main_map.rs:365` | ⑦ | 玩家「绿方块 + flag 代码图标」画法（:373-380）——v4 规格 §8：「绿方块 + flag 图标 → 船（boat_level 切 4 图，S21.12）」 | S21.12 船贴图落码（`assets/ui/boat_{1..4}.png` + 缺资产回退链 `ship_player.png` → 代码 flag，规格 §6）；`COLOR_PLAYER` 方块是否保留随出图定。严重度：中
- [⑦-07] `crates/ti-app/src/scenes/main_map.rs:527` | ⑦ | 骰点数 `y=1296` 画法——v4 规格 §2：`摇骰反馈 = CTA（deck_spin {408,1160,264,264}）上沿 y≈1136` | S21.7 锚 `deck_spin` 矩形（数据键）替代裸 1296（③-03）。严重度：低
- [⑦-08] `assets/data/scenes.json`（main_map 段） | ⑦ | v2+v3 双键并存：`hud/hud_trade/hud_sell/hud_build/grid/player/landmark/boss/hint/portrait_zones`（v2）与 `world_canvas/hud_gold/hud_player/deck_trade/deck_sell/deck_build/deck_spin/voyage_bar/canvas_entities/focus_anchor`（v3）——v4 键（`hud_plaque_l/r`、`btn_spin` 原语、新 `voyage_bar` 几何）未落（S21.4~S21.7 待开工，规格 §7 已声明「数据改在 S21.5/6/7」） | S21.6 一次性换版：v2 键删除（`grid.*` 回退链路随 board_fit v2 分支 ⑦-09 删）、v3 键按 v4 坐标表替换；`scene_manifest_v3.json` main_map 段 focus/bbox_pct 同批改（N8 硬耦合，规格 §5）。严重度：中
- [⑦-09] `crates/ti-app/src/scenes/chrome.rs:160` | ⑦ | v2 静态棋盘键 `GRID_X0/Y0/TILE_W/TILE_H`（:159-166）+ `board_fit` v2 回退分支（:262-268）+ `iso_cell/iso_tile` v2 遗留函数（:204-218，**生产零调用**，仅 touch.rs 测试/注释提及） | v4 定案 `world_canvas {43,250,994,826}` 为唯一画布键后：v2 回退分支与 `iso_cell/iso_tile` 删除（`board_fit` 直接读 v4 键，缺键 = 不画棋盘而非回退旧几何）；`keys` 模块删 v2 段。严重度：低
- [⑦-10] `crates/ti-app/src/scenes/chrome.rs:470` | ⑦ | `scene_shell` / `scene_base` / `scene_frame`（:470-491）生产零调用（仅注释/doc 提及；各场景已切 `render_primitives` + v3 装饰层）；同块 `menu_item`（:770）/ `bar_slot`（:786）亦零生产调用（index/vs 已原语化） | 死壳层导出：删除或标 `#[doc(hidden)]` 测试专用；`shell::menu_item/bar_slot`（ti-render 侧）同步收敛，避免「双写兼容」壳函数长期挂着。严重度：低
- [⑦-11] `crates/ti-app/src/game_app/mod.rs:251` | ⑦ | `pub scene_cache: HashMap<&'static str, ti_formats::mapfile::MapScene>`（+ `assets.rs:18 add_scene` 写入口、`render_png.rs:62`/`android_shell.rs:129` 喂数据）——S15.4 已删 legacy 精灵 blit，`scene_cache` **零读方**（死字段；`.Map` 解析链仅为喂它而活） | 删字段 + `add_scene` + 两 bin 的 `.Map` 注入循环（`asset_name` 映射表 `scenes/mod.rs:115` 随之删）；legacy `.Map` 解析归 `legacy/` 参照。严重度：低
- [⑦-12] `crates/ti-tools/src/bin/doccheck2.rs:1672`（门禁耦合残留） | ⑦ | 8pt 豁免键表硬编码 v3 键名清单（`world_canvas/hud_gold/hud_player/deck_trade/.../canvas_entities`，:1660-1688）——v4 新键（`hud_plaque_l/r`、`btn_spin`、新几何 `voyage_bar`）落 `scenes.json` 时**须同步豁免表**，否则 8pt 门禁误 FAIL / 或旧键继续豁免（S21.8「存量豁免清零」即此） | 豁免表数据化：从 `scene_manifest_v3.json`（或 08 规格 §5 表）读豁免键集，doccheck2 不再手抄键名清单；S21.6 换版与豁免表同 commit（规格 §5 N8/8pt 耦合点已声明，8pt 键表未声明 = 本条）。严重度：中

---

## 附：本轮判读备注（供审计 agent）

1. **工具链根因优先**（⑤-05/⑤-06）：`check_hardcode.py` 的 `STR_LIT` 捕获组 bug 使 gate 12 的 CJK 通道对「以标点/ASCII 结尾」的 CJK 字面量失明（靶点域实测 51 条全部漏检）；EXEMPT 表 `game_app.rs` 模式在 S20.0b 模块拆分后失配。两条与「渲染数据分离」正交，但决定 ⑤ 类的真实存量——建议审计先裁这两条，再裁 ⑤-01~04。
2. **不算违规的已核项**（避免误报）：`chrome.rs:101-107` MOTION_*（doccheck2 `check_ui_redesign_tokens` 与 `10_ui_redesign.md` §7 表断言锁定）；`theme.rs Theme::DEFAULT`（theme.json 三方恒等断言锁定，`check_ui_redesign_tokens` ②）；`framebuffer.rs LOGICAL_*`（1080 单一坐标系契约）；`iso.rs` 全文件（纯几何算法，参数全外部传入）；`board.rs:21` 除外已单列（①-06）。
3. ⑦ 类全部条目判据 = `docs/ui-redesign-v3/08_main_map_v4_spec.md` §8 v3→v4 差异表（S21.3 verified 定案）；S21.4~S21.12 未开工 = 旧画法「尚活着」，非漂移本身，属**待退役残留**（审计裁「需人裁决：随 S21 WBS 删除」或「真漂移：v2 死支路应立即删」）。
4. ④-02/④-03 是**双渲染期一致性缺陷**（CPU 端丢 size/theme，Bevy 端烘焙不缩放）：同一条 `scenes.json icon` 原语在两个后端出图不同——与 `12_bevy_usage.md` 双后端同构口径冲突，建议审计标高价值。
