# 界面、控件与渲染设计

本文档涵盖：消息/帮助/信息界面 + 控件表 + 资源容器 + 渲染管线 + 控制方案。

---

## 1. 消息/帮助/信息界面

### 1.1 五类界面

| 种类 | 资源文件 | 语义 | 总页数 |
|---|---|---|---|
| Msgs | `msgs.map` | 通用消息提示/退出对话框 | 1 |
| Msgl | `msgl.map` | 据点名/地标对话框 + 资源结算面板 | 1 |
| Msgm | `msgM.map` | 多页面板/卡片/贸易结果 | 1 |
| Help | `help.map` + `switchpage.pcx` | 多页帮助文本（主菜单「说明」） | 3 |
| Info | `information.map` | 信息/角色状态界面 | 1 |

可读标签（渲染用）：Msgs=「消息」、Msgl=「据点」、Msgm=「面板」、Help=「帮助」、Info=「信息」。

### 1.2 翻页状态机

```
struct MsgPage {
    kind: MsgKind,
    page: u32,          // 当前页码（0 起）
    total_pages: u32,
}
```

#### 状态转移规则

```
oracle_message_transition(cur: Option<MsgPage>, ev: MsgEvent) → Option<MsgPage>
```

| 事件 | `cur == None` | `cur == Some(c)` |
|---|---|---|
| `Open(kind)` | `Some({kind, page: 0, total: total_pages(kind)})` | 同左（覆盖） |
| `NextPage` | `None` | `page+1 < total` ? `page+1` : `page`（末页保持） |
| `PrevPage` | `None` | `page == 0` ? `total-1`（首页回末页，循环） : `page-1` |
| `Close` | `None` | `None` |

#### 边界规则（定死）

| 规则 | 说明 |
|---|---|
| NextPage 到末页 | 保持末页（`page` 不变），不回绕 |
| PrevPage 到首页 | 回末页（`page = total - 1`），循环 |
| Close | 幂等（`None` → `None`） |
| Open | `page = 0`（重置到首页） |
| `total_pages == 1` | NextPage/PrevPage 均无变化（`page` 恒 0） |

#### 消息事件

| 事件 | 对应原版事件码 |
|---|---|
| `Open(kind)` | 13（进入）/ 主地图事件分派 |
| `NextPage` | 38（下一页） |
| `PrevPage` | 37（上一页/前页） |
| `Close` | 27（返回） |

### 1.3 消息会话

```
struct MessageSession { cur: Option<MsgPage> }

方法:
    new() → cur = None
    current() → Option<MsgPage>
    kind() → Option<MsgKind>
    open(kind) → cur = Some({kind, page: 0, total: total_pages(kind)})
    next() → NextPage 语义
    prev() → PrevPage 语义
    close() → cur = None
```

### 1.4 对话框对象布局

| 族 | 对象大小 | vtable | 装载函数 |
|---|---|---|---|
| 对话框族（msgl/msgm/msgs） | `0x610` 字节 | `off_6FDFC` | `msgl` |
| Help 族 | `0x830` 字节 | `off_6F47C` | `help` |
| Info 族 | `0x48C` 字节 | `off_6F4FC` | `00029110:场景管理器` |

对话框标志：`a1[255] = 1`。

### 1.5 文本行写入

`对话框文本行写入器` 把宽字符串逐字拷入 `a1+260` 起始缓冲：

| 槽位 | 语义 |
|---|---|
| `a1[255]` | 行模式（1 = 单行标题，2 = 双行内容/翻页） |
| `a1[256]` | 翻页标志（0 = 无翻页按钮，1 = 启用） |
| `a1[259]` | 条目索引（对应 `cards/event.ini` 事件条目或场景内嵌文本索引） |

文本内容来源：

| 界面 | 格式化函数 | 格式串 |
|---|---|---|
| Msgl 据点名 | `主地图帧更新` 行 1766 | `%s：%s`（标题 + 据点名） |
| Msgl 资源结算 | `场景管理器帧更新总控` 行 523-527 | `word_7E470`（推测「%d 资源 +5」） |
| Msgm 升级 | `msgm` 行 48 | `word_7D694`（静态文本） |
| Msgm 贸易结果 | `msgm` 行 99/132/166 | `word_7D6F0`/`word_7D6D8`/`word_7D714` |
| Msgs 退出/提示 | `主地图帧更新` 行 1102 | `%d`（货物/金币结算） |
| Help 页标题 | `help` 行 29 | 页表项（6 项宽字符串数组 `off_6F32C`） |

> `word_*` 全局宽字符串常量的实际文本内容未逐字读取（BSS/初始化区布局未完整映射），
> oracle 层以确定性占位文本建模，**不声称还原原文**。

### 1.6 翻页按钮

`switchpage.pcx` 装载：
- `help` 行 49-73：宽字符串 → `0001CCE8:精灵注入;0001E830:容器索引;00020088:资源装载;0002FC44:容器加载;00033548:按钮注入;0003A058:控件注入;0003F8FC:帮助翻页;0003FE84:角色控件;00041B5C:角色头像;00042390:选项控件;0004290C:帮助控件;00043D94:场景注入;0004F904:目标场景` 哈希查找 → `free` 位图解码
- 解码到 `a1+522`，16 色索引 + 透明标志
- 翻页按钮为独立精灵槽，不占文本行

### 1.7 help 页数来源

| 入口 | 参数 `a4` | 总页数 |
|---|---|---|
| `saveload` case 3 | 3 | 3 |
| `saveload` case 4 | 2 | 2 |
| `帮助` | 2 | 2 |
| `选项场景帧处理` | 2 | 2 |

`help` 行 23 写 `a1+257 = a4 - 1`（总页数减 1 = 末页哨兵）。

> oracle 取**定死 3 页**（case 3 入口有明确证据）。

---

## 2. 控件表

### 2.1 主地图按钮（29 个）

| 控件 | 语义 | 控件 | 语义 |
|---|---|---|---|
| `button_focus.pcx` | 焦点选中 | `button_build.pcx` | 建造 |
| `button_move.pcx` | 移动 | `button_trade.pcx` | 贸易 |
| `button_view.pcx` | 查看 | `button_sell.pcx` | 卖出 |
| `button_info.pcx` | 信息 | `button_buy.pcx` | 买入 |
| `button_card.pcx` | 卡牌 | `button_pay.pcx` | 支付 |
| `button_system.pcx` | 系统 | `button_attack.pcx` | 攻击 |
| `button_load.pcx` | 读档 | `button_explore.pcx` | 探索 |
| `button_save.pcx` | 存档 | `button_finish.pcx` | 完成 |
| `button_option.pcx` | 选项 | `button_base.pcx` | 基地 |
| `button_exit.pcx` | 退出 | `button_return.pcx` | 返回 |
| `button_bank.pcx` | 银行 | `button_invest.pcx` | 投资/建造 |
| `button_troop.pcx` | 兵种 | `button_aimland.pcx` | 瞄准地块 |
| `button_aimrole.pcx` | 瞄准角色 | `button_aimcastle.pcx` | 瞄准城堡 |
| `button_capture.pcx` | 占领 | `button_notcapture.pcx` | 不占领 |
| `button_assign.pcx` | 分配 | | |

### 2.2 角色头像控件（6 个）

`Target_head.pcx` + `Player_{panda,tichy,kidd,drake,katharina}.pcx`（主菜单用）

SelectRole 场景 11 个控件槽位：
- 5 × `face_*.pcx`（头像）
- 5 × `Player_*.pcx`（精灵）
- 1 × `Target_knife.pcx`（选中标记）

### 2.3 .Map 控件记录布局

20 个 UI 场景的 `.Map` 文件，每条控件记录 6 个 u32 字段：

| 字段 | 值域 | 语义 |
|---|---|---|
| f0 | 恒为 1 | 使能标志（推测） |
| f1 | 1..119 | 精灵/贴图索引（84 种，对应 `ui.pak`/`glpics.pak` 位图） |
| f2 | — | 屏幕 **X** 坐标（blit 原点 `dst.x`） |
| f3 | — | 屏幕 **Y** 坐标（blit 原点 `dst.y`） |
| f4 | {1,2,3} | 贴图例程模式（依此与精灵位数 16/8 选择 6 个 blit 例程之一） |
| f5 | 0 / 100..1400 | 控件类型码（44 种，95% 为 0） |

记录结构：`MapScene.words.len() == 5 + 6 × count`，`count = words[4]`。

子头实测取值：`(160,192,1,0)` / `(176,219,1,0)` / `(176,220,1,0)` / `(168,216,1,0)` / `(0,0,0,0)`。

### 2.4 .Map 两族

| 族 | 头字段 | 文件 | 数量 |
|---|---|---|---|
| UI 场景族 | `B0 00 00 00 DC 00 00 00`（176×220） | `main/card/build/help/...Map` | 20 个 |
| 地形族 | `14 00 00 00 14 00 00 00`（20×20） | `glmapm01/02/03.Map` | 3 个 |

⚠️ `.Map` 内**无任何字符串**（ascii/utf-16 扫描均空）→ 资源绑定靠索引/key。

### 2.5 事件 id 不在 .Map 中

事件描述符对象（0x34 字节）：

| 偏移 | 语义 |
|---|---|
| +0 | vtable |
| +4 | 未知（=1） |
| +8 | 未知（=-1） |
| +12 | f1 精灵索引（**事件 id 读取路径起点**） |
| +16 | f2 X 坐标 |
| +20 | 未知（=1） |
| +24 | sprite width |
| +28 | sprite height |
| +36..40 | 派生自 +4/+8 |
| +44 | 位深/动画（=1） |
| +48 | 帧计数 |
| **+22** | **u8：低 2 位 = 子任务标志，高 6 位 = 事件 id** |

事件 id 计算：`(*(u8*)(*(u32*)(ctrl+12) + 22) >> 2) + 1`（值域 1..31）。

`.Map` 加载后 `*(ctrl+12)` 为 0 或由运行时场景逻辑回填事件描述符指针。

### 2.6 控件类型码（f5，44 种）

全量清单见 `ref/controls.json`（20 场景 2041 条记录 44 种 f5）。
非 0 值集中在 20..1400 分档（100/200/…/1400）。
语义随场景逻辑解释（属 S3.3+ 场景逻辑，未完全逆出）。

---

## 3. 资源容器

### 3.1 容器清单（5 个）

| 容器 | 磁盘名 | 内容 |
|---|---|---|
| Ui | `UI.pak` | 场景 `.Map` + 控件表 |
| Glpics | `glpics.pak` | 精灵 PCX（`face_*`/`troop`/`ship`/`flag`/`landmark*`） |
| Cards | `cards.lpk` | 卡牌数据（`cards.ini`/`event_00N`） |
| Goods | `goods.lpk` | 商品数据（`goods.mdu`/`goods_0N`） |
| Music | `music.lpk` | 音频（`victory.wav`/`glmusic.mod`/`press.wav`/`invalid.wav`） |

### 3.2 LucencyPackage 容器格式

```
头:
    magic / 版本 / 条目数 / 索引偏移 / data_start
索引:
    每条 = { name_key: u32, offset: u32, size: u32, ... }
name → key 哈希:
    key = djb2(lowercase(name), 种子 0)
```

ID 型 key（`ui.pak`/`glmap_*.pak`/`*.lpk` 共 217 条）绑定机制：
容器内 `mapinfo.ini` 逐 filename 闭合。

### 3.3 装载校验规则

```
oracle_load_scene(refs, available_containers, gl_rec_magic, glmapm_indices):
    for ref in refs:
        // 1. 容器存在性
        if ref.container not in available_containers:
            missing.push(ref); continue

        // 2. gl.rec 魔数（仅 Glpics 容器）
        if ref.container == Glpics and gl_rec_magic != 0x01315A47:
            missing.push(ref); continue

        // 3. glmapm 探测（仅 Ui 容器中 glmapm*.Map）
        if ref.container == Ui and ref.name matches "glmapm??.Map":
            idx = parse(ref.name[6..len-4])
            if idx not in glmapm_indices:
                missing.push(ref); continue

        // 4. 扩展名映射
        assets.push(match ext(ref.name):
            "pcx" → SceneAsset::Pcx(name)
            "mod" → SceneAsset::Mod(name)
            "sav" → SceneAsset::Sav(name)
            _     → SceneAsset::Map(name)
        )

    if missing.is_empty(): return Ok(assets)
    else: return Err(RenderError { missing })
```

**事务性**：`Ok` 时才写入内部 `scenes`；`Err` 时 `scenes` 保持调用前不变。

### 3.4 通用装载器

`场景资源通用装载(buf, ui, name)`：按容器 + 资源名从索引取 PCX/控件表 → 装入场景。
所有场景装载器共用。

### 3.5 场景管理器结构

```
struct SceneManager { scenes: Vec<SceneAsset> }

方法:
    new() → scenes 空
    assets() → &[SceneAsset]
    load_scene(refs, containers, magic, indices) → Result<Vec<SceneAsset>, RenderError>
    render_frame(&assets) → RenderPlan
```

---

## 4. 渲染管线

### 4.1 图层顺序 = 装载顺序

```
GXBeginDraw
  → 按装载顺序逐层 blit（不排序、不分组）
GXEndDisplay
```

```
struct RenderPlan {
    layers: Vec<SceneAsset>,   // 按装载顺序 = blit 顺序
    blit_count: usize,         // = layers.len()
}
```

### 4.2 逻辑帧缓冲

```
const LOGICAL_W: u32 = 176
const LOGICAL_H: u32 = 220
const FB_PIXELS: usize = 176 × 220 = 38_720

struct Framebuf {
    pixels: Box<[u16]>,        // RGB565，行主序
    dirty_rows: [bool; 220],   // 逐行脏标记
}
```

| 方法 | 语义 |
|---|---|
| `new()` | 新建全黑（全脏，保证首次上传） |
| `clear(color)` | 清空全部像素，置全帧脏 |
| `as_slice()` | 只读全部像素 |
| `as_mut_slice()` | 可变访问 |
| `mark_dirty(x, y, w, h)` | 标记矩形所在行脏（越界裁到屏内） |
| `has_dirty()` | 是否有脏行 |
| `dirty_row_range()` | 脏行范围 |
| `reset_dirty()` | 清零脏标记 |

### 4.3 视口（坐标变换）

```
struct Viewport { ... }

方法:
    screen_to_logical(sx, sy) → (lx, ly)    // 屏幕 → 逻辑（反变换）
    logical_to_screen(lx, ly) → (sx, sy)    // 逻辑 → 屏幕
```

等比缩放（letterbox/pillarbox），保证任意窗口尺寸下坐标互逆（P4-2 命中判定不偏移）。

### 4.4 呈现器

```
struct FramePresenter { ... }

enum UploadMode {
    FullRebuild,   // 桌面 native（EXP-045：GL 上下文特性）
    Partial,       // wasm（复用纹理部分上传）
}
```

- 脏行带部分上传：只上传 `[min_dirty, max_dirty]` 行区间
- 静止帧零 GPU 工作（`has_dirty() == false` → 跳过整帧）
- 首次呈现必上传（新建时全脏）

### 4.5 字形光栅

```
struct Raster {
    font: Option<FontSet>,   // GoldLand.Fnt + font.dat
}

const GLYPH_ADVANCE: i32 = 12    // 字形步进（逻辑像素）
const INK_COLOR: u16 = 0xFFFF    // 字形墨色（RGB565 白）
```

- 真 `GoldLand.Fnt` 墨点渲染
- 缺字画**豆腐块**回退（绝不静默空白）
- 字库字符集 1016 字（`font.dat` UTF-16 字符集清单）

### 4.6 场景渲染函数

| 函数 | 逻辑 | 常量 |
|---|---|---|
| `MenuShell::render` | 主菜单（标题 + 菜单项 + 选中高亮 + 状态行） | `MENU_TOP=96`, `ROW_H=20` |
| `STATUS_Y` | 底部状态行 y | `220 - 12 = 208` |
| `STATUS_X` | 底部状态行 x | `4` |

静态帧优化：
- `last_rendered: Option<usize>` — 上次渲染的 `selected`，未变化时跳过整帧
- `last_rendered_scene: Option<i32>` — 上次渲染的场景键，场景切换时强制整帧重渲染

---

## 5. 控制方案

### 5.1 全局事件码

| 事件码 | 语义 | 音效 |
|---|---|---|
| 13 | 确定/Enter | Press |
| 27 | 返回/取消 | 无 |
| 37 | 无效操作 | Invalid |
| 38 | 上一页/上一个 | 无 |
| 39 | 翻页确认（贸易专用） | 无 |
| 40 | 下一页/下一个 | 无 |

### 5.2 各场景操作

| 场景 | 38（上一个） | 40（下一个） | 13（确定） | 27（返回） |
|---|---|---|---|---|
| Index 主菜单 | 选上一菜单项 | 选下一菜单项 | 确认菜单项 | — |
| SelectRole | 上一个角色 | 下一个角色 | 确认角色 | 回 Index |
| SelectMap | 上一地图 | 下一地图 | 确认地图 | 回 Index |
| Help 帮助 | 上一页 | 下一页 | — | 回 Index |
| Trade 贸易 | 商品索引 -1 | 商品索引 +1 | 确认买卖 | 取消 |
| Saveload | 上一槽位 | 下一槽位 | 确认槽位 | 回 Index |
| MainMap | 方向移动（北） | 方向移动（南） | 触发事件 | 触发事件 |

### 5.3 虚拟按键

```
struct VirtualButton {
    name: &'static str,   // "ok" / "cancel" / "up" / "down" ...
    rect: Rect,           // 176×220 逻辑坐标矩形
}

方法:
    hit(x, y) → bool      // 逻辑坐标点是否落在区域内
```

命中判定：`x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h`。

### 5.4 事件翻译

```
translate_event(screen_event, viewport) → Event:
    PointerDown { x, y } → PointerDown { x: lx, y: ly }    // 屏幕 → 逻辑
    PointerUp   { x, y } → PointerUp   { x: lx, y: ly }
    PointerMove { x, y } → PointerMove { x: lx, y: ly }
    KeyDown / KeyUp / Quit → 原样透传
```

### 5.5 现代化输入映射

详见 [`../modernization.md`](../modernization.md) §5：

| 原操作 | 桌面 | 触屏 | 手柄 |
|---|---|---|---|
| 方向移动 | 方向键 / WASD | 左半区虚拟十字键 / 滑动手势 | 左摇杆 |
| 确定（13） | Enter / 空格 | 右半区「确认」按钮 / 双击 | A 键 |
| 返回（27） | ESC | 「返回」按钮 / 长按 0.5s | B 键 |
| 上一页（38） | ↑ / Q | 上滑 | 十字键上 |
| 下一页（40） | ↓ / E | 下滑 | 十字键下 |

### 5.6 音频控制

| 项 | 实现 |
|---|---|
| BGM | OGG 循环播放（`glmusic.ogg`） |
| 音效 | WAV 即时播放（`press.wav`/`invalid.wav`/`victory.wav`） |
| 音量 | 音效 + BGM 双通道，0~100% |
| 静音 | 全局开关 + 浏览器后台自动静音（`visibilitychange`） |
| 通道限制 | 音效并发 ≤ 8 |