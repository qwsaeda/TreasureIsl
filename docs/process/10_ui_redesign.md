# 界面全量重设计 · 航海夜·精修（S17，设计规格单一事实源）

**定位**：S17 全量重设计（用户 2026-10-18 裁决，方向 = A 航海夜·精修）的**设计规格**。
数值层（色板/栅格/字号/动效/图标）以本文为准；`08_layout_design.md` 在 S17.2 起同步为本文的
场景坐标实例（表值 = scenes.json 同值纪律不变）。运行时像素 100% 来自 `scenes/chrome.rs`
代码绘制 + TTF 字体（S16 渲染线裁决），故本重设计**不碰 legacy 像素 parity**（P0~P2 规则
parity 与 UI 无关）；三套视觉基线随 S17.2/S17.4 重生成。

> S15.6（视觉对抗 loop）由本线取代：重设计收口（S17.5）时一并解决并回写台账。

## 1. 设计原则（4 条，验收口径）

| 原则 | 判据 |
|---|---|
| 可读性 | 正文/次级文字对比度 ≥ WCAG AA（4.5:1），HUD 数字等宽对齐（§6） |
| 一致性 | 所有视觉量走设计令牌（§3），chrome 禁止新增硬编码色/坐标（`check_hardcode` 门禁） |
| 现代 | 代码矢量图标替代码色块（§5）；场景切换有 240ms 交叉淡化（§7） |
| 移动友好 | 可点目标高 ≥ 96px（行高 168/256/280 均满足，§4） |

## 2. 方向

保留「航海夜看台」氛围（深蓝夜色 + 金色提灯），修掉三块屎：① 对比度与层次不足
（底/卡/描边只差一点，全屏幕一坨）；② 代码色块当图标（廉价）；③ 场景硬切无过渡（生硬）。

## 3. 设计令牌 v2（色板 13 色，8-bit → RGB565 已按 `scripts/rgb565.py` 口径换算）

| token | 8-bit hex | RGB565 | 落地常量（`theme.rs`/`menu_shell`） | 用途 |
|---|---|---|---|---|
| `bg0` | `#0C0E16` | `0x0865` | `COLOR_PARCH_DARK` | 页面底（比现 `#12141C` 更深，拉层次） |
| `bg1` | `#151827` | `0x10C9` | `COLOR_PARCH_LIGHT` | 背景顶部渐变层 |
| `surface` | `#1B2033` | `0x190C` | `COLOR_MENU_BG` | 卡片底 |
| `bevel_hi` | `#262C45` | `0x2171` | `COLOR_BEVEL_HI` | 卡片顶/左（凸起） |
| `stroke` | `#39415E` | `0x3A17` | `COLOR_WOOD` / `COLOR_MENU_BORDER` | 外框/描边（比现 `#3A4058` 提亮一档） |
| `gold` | `#FFC857` | `0xFE55` | `COLOR_GOLD` / `COLOR_LANDMARK` | 主强调（比现 `#F5B841` 提饱和） |
| `blue` | `#4FC3F7` | `0x4E3D` | `COLOR_GOLD_GLOW` / `COLOR_GOLD_BRIGHT` | 交互高亮/选中底（比现 `#42A5F5` 提亮） |
| `text_hi` | `#F4F1E8` | `0xF7BA` | 映射 `COLOR_TEXT_ON_PARCH` | 正文/标题墨 |
| `text_mid` | `#9AA3C0` | `0x9D30` | `COLOR_SECONDARY` | 次级文字（现 `0x8C93` 提亮达 AA） |
| `text_lo` | `#5A6178` | `0x5B1E` | 无独立 token：置灰/锁定态复用 COLOR_SECONDARY（深底上 AA 7.1:1，比旧 `#5A6178` 3.1:1 更安全） | 置灰/禁用 |
| `danger` | `#FF6B5E` | `0xFB57` | 复用 `COLOR_DANGER` 槽位 | 玩家 HP/失败 |
| `success` | `#69D58C` | `0x6EA3` | `COLOR_PLAYER` 槽位（theme 字段 `player`） | 胜利/可购、玩家棋子/血条 |
| `info` | `#45B7D8` | `0x45B6` | 复用 `COLOR_HILITE` 槽位 | 血条槽底/水系 |

### 3.1 对比度表（WCAG 2.1 相对亮度，验收 ≥ AA）

| 前景/底 | 比值 | 判定 |
|---|---|---|
| `text_hi` / `bg0` | ≈15.5:1 | AAA |
| `text_hi` / `surface` | ≈13.8:1 | AAA |
| `text_mid` / `bg0` | ≈7.1:1 | AA |
| `text_mid` / `surface` | ≈5.9:1 | AA |
| `gold` / `surface` | ≈9.8:1 | AA（大字号元素） |
| `danger` / `bg0` | ≈5.5:1 | AA |
| `text_lo` / `bg0` | ≈3.1:1 | 仅禁用态（WCAG 不强制） |

> 色值换算一律 `scripts/rgb565.py`，禁手算（08_layout_design §2 既有纪律，S15 手算翻车先例）。

## 4. 版面增量（8pt 栅格整备 + 触点放大；几何业务量 `grid.*`/`iso_cell` 不动）

| 场景/元素 | 现值 | v2 | 理由 |
|---|---|---|---|
| 标题条 `h`（全场景） | 90 | **96** | 8pt；标题 96px 档墨迹更舒展 |
| 标题 `y` | 6 | **8** | 8pt |
| `list.x`（index/select/saveload） | 90 | **96** | 8pt |
| `list.w`（index/select/saveload） | 900 | **896** | 8pt（112×8） |
| index `list.y` | 400 | **392** | 8pt（49×8） |
| index `row_h` / `item_h` | 220 / 170 | **224 / 168** | 8pt；行高 ≥96 触点 ✓ |
| index `text_x` | 240 | **248** | 8pt |
| `icon.x` | 150 | **144** | 8pt |
| select_role `row_h` / `item_h` | 300 / 260 | **304 / 256** | 8pt |
| select_map `row_h` | 340 | **336** | 8pt |
| saveload `row_h` / `item_h` | 300 / 250 | **304 / 248** | 8pt |
| saveload `gold_x` | 820 | **816** | 8pt |
| main_map `hud.y` | 150 | **152** | 8pt |
| main_map `hud.h` | 120 | **128** | 8pt |
| main_map 三按钮 `x`（trade/sell/build） | 536/696/856 | **536/704/856**（仅 696→704） | 8pt |
| main_map 三按钮 `w` | 140 | **136** | 8pt |
| vs `player_hp/boss_hp .y` | 170 | **168** | 8pt |
| vs `player_hp/boss_hp .w` | 440 | **448** | 8pt |
| vs `player_hp.x` / `boss_hp.x` | 60 / 580 | **64 / 576** | 8pt |
| vs `rounds.y` | 420 | **416** | 8pt |
| message `body.x/y/w` | 90/220/900 | **96/224/896** | 8pt |
| 底部提示行 `y=1840`（全场景） | 1840 | 不变 | 已是 230×8 |
| `grid.*`（tile_w=25/tile_h=13 等距几何） | 现值 | 不变 | 业务几何非版面令牌 |

> 全部增量落 `scenes.json`（S17.2），`08_layout_design.md` §4 各表同值同步；
> `doccheck2 check_scene_layout_fidelity` 锚点随改。

## 5. 图标 14 项（代码矢量绘制，`chrome::icon_*`；注册表 `assets/data/ui_icons.json`）

| # | 名称 | 绘制要点（chrome 函数 id） | 出现场景 |
|---|---|---|---|
| 1 | `compass` | 八芒罗盘 + 北针 | index 新游戏 |
| 2 | `chest` | 宝箱盖开缝 + 金锁 | index 读取存档 |
| 3 | `anchor` | 船锚 | index 载入游戏 |
| 4 | `gear` | 八齿齿轮 | index 选项设置 |
| 5 | `book` | 翻开书 | index 帮助说明 |
| 6 | `door` | 门 + 门框 | index 离开游戏 |
| 7 | `coin` | 双圈金币 + 面值纹 | main_map HUD 金 |
| 8 | `sword` | 交叉双剑 | main_map HUD 战备 / vs |
| 9 | `castle` | 三垛城 | main_map 建造 |
| 10 | `beacon` | 灯塔三角光束 | main_map 据点 |
| 11 | `heart` | 心形（HP） | vs 血条端帽 |
| 12 | `flag` | 三角旗 | vs 回合标记 |
| 13 | `map` | 折叠羊皮地图 | select_map 行首 |
| 14 | `monogram` | 角色首字母 + 圆环（6 角色复用，字母走数据） | select_role 像位 |

规格：逻辑尺寸 48×48（可缩放调用），单色 = `gold`，选中态 = `blue` 底 + `text_hi` 图；
注册表字段 `{ name, chrome_fn, size, tint_default, tint_selected }`。
替代目标：index 菜单图标列（现 `menu_icon` 色块）、select_role 像位（现色块）、
main_map HUD 按钮内文字符号、vs 血条端帽。

## 6. 字号阶（TTF 三档 → 四档）

| 档 | 现 | v2 | 用途 |
|---|---|---|---|
| display | 96 | 96 | 场景标题 |
| body | 44 | 44 | 菜单行/正文 |
| label | —（无） | **32** | HUD 数值（金/HP/回合，等宽对齐） |
| caption | 28 | 28 | 提示行/次级 |

数字一律用 label 档 + 等宽位（`tabular` 对齐）：HUD 金值、vs 回合数、血条百分比。

## 7. 动效参数表（4 类，统一走 frame timer；参数冻结在此表）

| 动效 | 时长 | 曲线 | 实现挂点 |
|---|---|---|---|
| 场景切换交叉淡化 | 240ms | ease-in-out | `Framebuf::crossfade_from`（ti-render，S15.10 已就位） |
| 按压闪光 | 120ms | 线性衰减 | `chrome::flash_press`（现 `press_flash` 计时参数化） |
| 血条 lerp | 160ms | 线性 | vs `bar_slot` 宽度插值 |
| 翻页亮度 | 120ms | 线性 | `page_brightness_factor`（现有） |

## 8. 验收锚点（S17.2/S17.4 固化，S17.5 收口）

1. **doccheck2 第 24 项 `check_ui_redesign_tokens`**：
   ① `scenes.json` 全部版面坐标 8pt 整除（`grid.*` 除外，白名单）；
   ② `theme.rs`/`menu_shell` 色板常量值 == §3 表（13 色）；
   ③ 动效 4 参数 == §7 表（常量锚点）；④ S17.3 后 `ui_icons.json` 14 条注册齐全。
2. **基线规程**：每阶段代码全绿后重生成三套基线（render_snapshot / forward /
   visual_regression），`doccheck2 baseline-mirror-identity` 保证三套镜像恒等（字节级）。
3. **负例纪律**：断言先注入坏值验证 FAIL 再还原（S15.9/S15.10 既有规程）。
4. **用户验收**：S17.5 用户过目（消息留痕）= 视觉闭环，S15.6 随之关闭。

## 9. 实施波次（对应台账 S17.1~S17.5）

```
S17.1 本文档（用户批准）→ S17.2 令牌+版面（色板/theme/scenes.json/08 文档/基线）
→ S17.3 图标集（ui_icons.json + chrome icon_* + 换图）
→ S17.4 动效（4 类接线 + 帧快照断言）→ S17.5 收口（基线终态 + 用户验收 + S15.6 关闭 + 回写）
```

S16 试玩/发布线（web_playtest 起）与本线文件域不相交，可并行；S17.2 依赖 S15.10（帧缓冲单源，已 verified）。
