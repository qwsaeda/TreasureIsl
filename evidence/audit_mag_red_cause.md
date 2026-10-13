# 洋红框根因审计（native `QualityMode::Original` 路径）

- 审计对象：`cargo run -p ti-app --features native-shell --bin treasure-native`，人物/立绘被洋红色矩形包住。
- 结论先行：**洋红 = PCX 调色板 index 0 在 94/120 张精灵中就是 (255,0,255)，而 `SpriteAtlas::blit_sprite`（RGB565）逐像素写 `palette[idx]`、不跳过 index 0**——对应任务选项 **(a)**。原版 ARM 8-bit blit 例程跳过 index 0（透明位），Rust 侧漏了这个跳过。
- 置信度：**高**（原码级证据 + 数据实测 + 渲染路径实测，三方闭合）。

---

## 链路 5 步证据表

### Step 1 — PCX 解码与调色板

| 证据 | 位置 |
|---|---|
| 解码器：8bpp 单平面 RLE，尾部 769 字节调色板（`0x0C` 标记 + 768 RGB）；无尾调色板时回退 EGA 默认表（EGA 默认表恰好也是洋红系，`pcx.rs:34` `[0,0,170]`/`[170,0,170]`） | `crates/ti-formats/src/pcx.rs:87`（`Image::decode`）、`:170`（`read_palette`） |
| 调色板解析无「透明位」概念——`Image` 结构只有 `pixels`（索引）+ `palette`，**没有任何 alpha/透明通道字段** | `crates/ti-formats/src/pcx.rs:17-27` |
| **数据实测**（`legacy/data/ui.pak` 全量 122 条目，120 张 PCX）：调色板 index 0 的 RGB 分布——`(255,0,255)` **94 张**、`(0,0,0)` 18、`(3,4,14)` 4、`(255,255,255)` 2、`(40,40,40)` 1、`(76,67,67)` 1 | 脚本实测 `legacy/data/ui.pak`（djb2 key 反查 `assets_raw/ui/mapinfo.ini`） |
| index 0 像素占比实测：大量精灵像素值就是 0，如 `06.pcx` 13×45 中 54.0%、`16.pcx`（122×74）29.3%、`50.pcx` 44×44 75.2%（f1=50 即 44×44 档）——**index 0 是背景填充位，占比 0%~84.7%** | 同上 |

### Step 2 — 原版透明约定（PCX 精灵透明位）

| 证据 | 位置 |
|---|---|
| **原版 8-bit blit 例程 `sub_5D9C4` 逐像素跳过 index 0**：读索引 `v19 = *i++`；`if (v19 != 0) v21 = 1; else { v21 = 0; v20 = 0; }`；`if (v21 != 0) *v17 = palette[v20];` —— **index 0 时不写目标像素（保留背景）**。16-bit 例程同源结构 | `legacy/ref/pseudocode/0005D9C4_sub_5D9C4.c:64-87`（8-bit 模式 = `blit_scene` 实际走的例程族，`sub_460F8` 按精灵位深 8/16 分派，`legacy/ref/pseudocode/000460F8_sub_460F8.c:38-41`） |
| 6 个 blit 例程分派表：`f4 ∈ {1,2,3} × 位深 {8,16}`，本路径（8-bit 调色板 PCX）对应 `sub_5D9C4` / `sub_5E158` / `sub_5DD7C`，全部「palette-index-0 = 透明」语义 | `legacy/ref/pseudocode/000460F8_sub_460F8.c:28-82` |
| 事实源记载：「原版 PCX blit 是**不透明覆盖**语义（F-311，调色板 0 号位即透明）」——即非 0 索引覆盖、0 号位让底层背景透出 | `crates/ti-app/src/sprite_atlas.rs:334-335`（`ALPHA_TRANSPARENT_CUTOFF` 注释）+ `legacy/docs/facts/map.md:12`（F-311） |

### Step 3 — Rust `blit_sprite` 是否跳过透明 index

| 证据 | 位置 |
|---|---|
| **`blit_sprite`（RGB565）不判任何透明**：每个像素都执行 `let [r,g,b] = sprite.palette[idx]; ... *slot = rgb565;`，包括 `idx == 0` | `crates/ti-app/src/sprite_atlas.rs:260-267` |
| 调用链：`GameApp::render` → `host.blit_scene_sprites` → `atlas.blit_scene` → `blit_sprite`（`game_app.rs:950-956, 1011-1022`；`blit_scene` `sprite_atlas.rs:217`） | |
| 对照：16-bit 原版例程 `sub_5B0CC` 直接 `memcpy`（16-bit 位图自带格式，无透明概念）；8-bit 例程 `sub_5D9C4` 才有 index-0 跳过——Rust 漏的正是 8-bit 路径的跳过逻辑 | `legacy/ref/pseudocode/0005B0CC_sub_5B0CC.c:52-61` |

### Step 4 — 洋红色值来源

| 证据 | 位置 |
|---|---|
| **index 0 的 RGB 实测就是 (255,0,255)**：94/120 张 PCX 的尾部调色板第 0 槽 = `FF00FF`。洋红**不是**调色板损坏或 EGA 回退路径产物（走 EGA 回退的图其 `pal0=(0,0,0)` 且像素不含 0）——是 PCX 尾部调色板里**真实存的字节**，制作工具把「透明背景位」填成了洋红（典型的「故意选最刺眼颜色以便发现漏跳」的哨兵色） | `legacy/data/ui.pak` 实测（Step 1） |
| 其余洋红相近色（103 个命中）集中在 index 0；非 0 槽的洋红/紫红仅零星（如 `09.pcx` idx253、`10.pcx` idx255、`47.pcx` idx27=(213,75,255)），占比可忽略，与「纯矩形包边」现象不符（矩形 = 背景区成片 index 0） | 同上 |
| `pack_hd_assets.py` 注释亦确认资产生成侧已知此色：「原版 PCX 精灵的背景像素是**调色板色**（本项目里是洋红），alpha=0」 | `scripts/pack_hd_assets.py:66` |

### Step 5 — 修复方案验证（推演 + 既有工具）

| 验证点 | 位置 / 判据 |
|---|---|
| 最小修复：`blit_sprite`（RGB565）在取 `idx` 后加 `if idx == 0 { continue; }`（跳过 index 0 不写像素），与原版 `sub_5D9C4` 逐像素口径一致；RGB565→RGB8 后 (255,0,255) 在 `is_magenta`（`r>140 && g<90 && b>140`）内，跳过后 Index/SelectRole 洋红计数应为 0（背景位透出清屏色 `COLOR_BG=0x0010`） | 新增测试可直接断言：`blit_scene` 后 `fb` 中洋红 565 值 = 0 |
| 现有 dump 工具 `mag_dump` 已覆盖两条路径计数：`render_hd`+`fb_rgba` 洋红、`render_to_framebuf`（RGB565）洋红——**后者就是本 bug 的验收口径**（`crates/ti-app/src/bin/mag_dump.rs:115-122`） | |
| 同步需修的路径（同口径，漏则洋红仍在）：`blit_sprite_rgba` 的 **PCX 回退分支**（`sprite_atlas.rs:395-401`，HD 未注入时 `idx==0` 照样写 `palette[idx]`）、`terrain_atlas::blit_decoded`（`terrain_atlas.rs:248-255`，RGB565 瓦片路）、`blit_decoded_rgba` PCX 回退（`terrain_atlas.rs:357-362`） | |
| 测试基线注意：`render_parity.rs:246`（`blit_pcx_independent`）与基线 PNG 同口径不跳 0，修 blit 后该独立基线需同步加跳过，否则逐像素比对失配 | `crates/ti-app/tests/render_parity.rs:246-279` |
| 16-bit 例程族与 8-bit 共用 `blit_sprite` 入口：ui.pak 实测全部 120 张是 8bpp 尾部调色板（Step 1 脚本断言 `data[3]==8` 全命中），修 index-0 跳过不影响 16-bit 语义（16-bit 位图源无 index 概念；本仓库 `DecodedSprite` 只有 8-bit 像素+调色板两种形态） | `crates/ti-app/src/sprite_atlas.rs:31-42` |

---

## 根因判定

**(a) 成立**：`ui.pak` 里 94/120 张 PCX 的调色板 index 0 就是 (255,0,255)，原版 8-bit blit 例程（`sub_5D9C4`）跳过 index 0 实现透明；当前 Rust `SpriteAtlas::blit_sprite`（RGB565，`QualityMode::Original` 实际走的路）不跳过，把背景位的洋红哨兵色整片写进帧缓冲 → 人物/立绘被洋红矩形包围。

**S13.5 为什么「没解决」**：S13.5/EXP-075 修的是 **HD PNG 路径**（`pack_hd_assets.py` 软 alpha + `blit_sprite_rgba` 阈值 + `render_hd` 漏调 blit，`docs/experience/process.md:198-214`）。用户实机走的是 `QualityMode::Original`（`settings.rs:314` 默认值）→ `treasure_native.rs:363-369` 判 `quality != Hd` → `render()`（RGB565 `blit_sprite`）→ 该路径从未修过 index-0 跳过。即使 HD 路径全绿，Original 路径照样洋红——两条路径的透明判定各自独立。

## 最小修复点

1. **主修复**（`crates/ti-app/src/sprite_atlas.rs:260`，`blit_sprite` 内）：
   ```rust
   let idx = sprite.pixels.get(fbase + row * sw + col).copied().unwrap_or(0) as usize;
   if idx == 0 { continue; }   // 原版 8-bit blit 透明位（sub_5D9C4），背景位不覆盖
   let [r, g, b] = sprite.palette[idx];
   ```
2. **同口径补丁**（同一 bug 在其余 3 个 blit 变体的 PCX 分支）：
   - `blit_sprite_rgba` PCX 回退分支 `sprite_atlas.rs:397-398`：`if idx == 0 { continue; }`
   - `terrain_atlas.rs:248`（`blit_decoded`）：`if idx == 0 { continue; }`
   - `terrain_atlas.rs:359`（`blit_decoded_rgba` PCX 回退）：`if idx == 0 { continue; }`
3. **同步测试**：`tests/render_parity.rs:246` 独立基线 `blit_pcx_independent` 加同口径跳过，再重生成 `assets/hd_baseline` 相关基准（如有 PCX 场景基准）；`visual_regression.rs` 背景掩码（`EXP-074` BG565=16）继续适用。
4. **验收**：`cargo run -p ti-app --bin mag_dump -- --scene index` 输出「RGB565 路径洋红像素 = 0」（`mag_dump.rs:130-140` 报告段），然后 `render_parity` + `visual_regression` 回归。

## 遗留未知 / 待办

- 原版「16-bit 例程」是否也按某掩码透明（`sub_5BC14` 一族）——本仓库无 16-bit PCX 数据消费（ui.pak 全 8bpp），不影响本修复，标 `推测`。
- 清屏色 `COLOR_BG=0x0010`（`menu_shell.rs:42`）与原版清屏是否一致，属 `docs/parity.md` 范围，不在本次审计内。
