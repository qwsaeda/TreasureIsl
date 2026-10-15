# 界面、控件与渲染实现规格

本文档是 UI 系统的完整实现规格，实现者无需阅读其他文档即可写出正确的界面渲染与控件逻辑代码。

> 事实来源：`docs/design/ui_controls.md` + `docs/design/constants.md` §1/§2/§8 + `docs/facts/map.md` + `docs/facts/font.md`。

---

## 1. 消息/帮助/信息界面

### 1.1 五类界面

| 种类 | 资源文件 | 语义 | 总页数 |
|---|---|---|---|
| Msgs | `msgs.map` | 通用消息提示/退出对话框 | 1 |
| Msgl | `msgl.map` | 据点名/地标对话框 + 资源结算面板 | 1 |
| Msgm | `msgm.Map` | 多页面板/卡片/贸易结果 | 1 |
| Help | `help.map` + `switchpage.pcx` | 多页帮助文本（主菜单「说明」） | 3 |
| Info | `information.map` | 信息/角色状态界面 | 1 |

可读标签（渲染用）：Msgs=「消息」、Msgl=「据点」、Msgm=「面板」、Help=「帮助」、Info=「信息」。

---

## 2. 翻页状态机

### 2.1 类型定义

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgKind {
    Msgs,
    Msgl,
    Msgm,
    Help,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsgPage {
    pub kind: MsgKind,
    pub page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgEvent {
    Open(MsgKind),
    NextPage,
    PrevPage,
    Close,
}

#[derive(Debug, Clone)]
pub struct MessageSession {
    pub cur: Option<MsgPage>,
}
```

### 2.2 状态转移规则

```
oracle_message_transition(cur: Option<MsgPage>, ev: MsgEvent) → Option<MsgPage>
```

| 事件 | `cur == None` | `cur == Some(c)` |
|---|---|---|
| `Open(kind)` | `Some({kind, page: 0, total: total_pages(kind)})` | 同左（覆盖） |
| `NextPage` | `None` | `page+1 < total` ? `page+1` : `page`（末页保持） |
| `PrevPage` | `None` | `page == 0` ? `total-1`（首页回末页，循环） : `page-1` |
| `Close` | `None` | `None` |

### 2.3 边界规则（定死）

| 规则 | 说明 |
|---|---|
| NextPage 到末页 | 保持末页（`page` 不变），不回绕 |
| PrevPage 到首页 | 回末页（`page = total - 1`），循环 |
| Close | 幂等（`None` → `None`） |
| Open | `page = 0`（重置到首页） |
| `total_pages == 1` | NextPage/PrevPage 均无变化（`page` 恒 0） |

### 2.4 消息事件对应原版事件码

| 事件 | 对应原版事件码 |
|---|---|
| `Open(kind)` | 13（进入）/ 主地图事件分派 |
| `NextPage` | 38（下一页） |
| `PrevPage` | 37（上一页/前页） |
| `Close` | 27（返回） |

### 2.5 实现

```rust
impl MessageSession {
    pub fn new() -> Self { MessageSession { cur: None } }
    pub fn current(&self) -> Option<MsgPage> { self.cur }
    pub fn kind(&self) -> Option<MsgKind> { self.cur.map(|c| c.kind) }
    pub fn open(&mut self, kind: MsgKind) {
        let total = total_pages(kind);
        self.cur = Some(MsgPage { kind, page: 0, total_pages: total });
    }
    pub fn next(&mut self) {
        if let Some(c) = self.cur {
            if c.page + 1 < c.total_pages {
                self.cur = Some(MsgPage { page: c.page + 1, ..c });
            }
        }
    }
    pub fn prev(&mut self) {
        if let Some(c) = self.cur {
            let new_page = if c.page == 0 { c.total_pages - 1 } else { c.page - 1 };
            self.cur = Some(MsgPage { page: new_page, ..c });
        }
    }
    pub fn close(&mut self) { self.cur = None; }
}

pub fn total_pages(kind: MsgKind) -> u32 {
    match kind {
        MsgKind::Msgs => 1,
        MsgKind::Msgl => 1,
        Kind::Msgm => 1,
        MsgKind::Help => 3,
        MsgKind::Info => 1,
    }
}
```

---

## 3. 对话框对象布局

| 族 | 对象大小 | vtable | 装载函数 |
|---|---|---|---|
| 对话框族（msgl/msgm/msgs） | `0x610` = 1552 字节 | `off_6FDFC` | `msgl` |
| Help 族 | `0x830` = 2096 字节 | `off_6F47C` | `help` |
| Info 族 | `0x48C` = 1164 字节 | `off_6F4FC` | `00029110:场景管理器` |

对话框标志：`a1[255] = 1`。

---

## 4. 文本行写入

`对话框文本行写入器` 把宽字符串逐字拷入 `a1+260` 起始缓冲：

| 槽位 | 语义 |
|---|---|
| `a1[255]` | 行模式（1 = 单行标题，2 = 双行内容/翻页） |
| `a1[256]` | 翻页标志（0 = 无翻页按钮，1 = 启用） |
| `a1[259]` | 条目索引（对应 `cards/event.ini` 事件条目或场景内嵌文本索引） |

### 4.1 文本内容来源

| 界面 | 格式化函数 | 格式串 |
|---|---|---|
| Msgl 据点名 | `主地图帧更新` 行 1766 | `%s：%s`（标题 + 据点名） |
| Msgl 资源结算 | `场景管理器帧更新总控` 行 523-527 | `word_7E470`（推测「%d 资源 +5」） |
| Msgm 升级 | `msgm` 行 48 | `word_7D694`（静态文本） |
| Msgm 贸易结果 | `msgm` 行 99/132/166 | `word_7D6F0`/`word_7D6D8`/`word_7D714` |
| Msgs 退出/提示 | `主地图帧更新` 行 1102 | `%d`（货物/金币结算） |
| Help 页标题 | `help` 行 29 | 页表项（6 项宽字符串数组 `off_6F32C`） |

---

## 5. 翻页按钮

`switchpage.pcx` 装载：
- `help` 行 49-73：宽字符串 → `0001CCE8:精灵注入;0001E830:容器索引;00020088:资源装载;0002FC44:容器加载;00033548:按钮注入;0003A058:控件注入;0003F8FC:帮助翻页;0003FE84:角色控件;00041B5C:角色头像;00042390:选项控件;0004290C:帮助控件;00043D94:场景注入;0004F904:目标场景` 哈希查找 → `free` 位图解码
- 解码到 `a1+522`，16 色索引 + 透明标志
- 翻页按钮为独立精灵槽，不占文本行

---

## 6. help 页数来源

| 入口 | 参数 `a4` | 总页数 |
|---|---|---|
| `saveload` case 3 | 3 | 3 |
| `saveload` case 4 | 2 | 2 |
| `帮助` | 2 | 2 |
| `选项场景帧处理` | 2 | 2 |

`help` 行 23 写 `a1+257 = a4 - 1`（总页数减 1 = 末页哨兵）。

oracle 取**定死 3 页**（case 3 入口有明确证据）。

---

## 7. 控件表

### 7.1 主地图按钮（29 个）

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

### 7.2 角色头像控件（6 个）

`Target_head.pcx` + `Player_{panda,tichy,kidd,drake,katharina}.pcx`（主菜单用）

SelectRole 场景 11 个控件槽位：
- 5 × `face_*.pcx`（头像）
- 5 × `Player_*.pcx`（精灵）
- 1 × `Target_knife.pcx`（选中标记）

---

## 8. .Map 控件记录布局

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

### 8.1 .Map 两族

| 族 | 头字段 | 文件 | 数量 |
|---|---|---|---|
| UI 场景族 | `B0 00 00 00 DC 00 00 00`（176×220） | `main/card/build/help/...Map` | 20 个 |
| 地形族 | `14 00 00 00 14 00 00 00`（20×20） | `glmapm01/02/03.Map` | 3 个 |

`.Map` 内**无任何字符串**（ascii/utf-16 扫描均空）→ 资源绑定靠索引/key。

---

## 9. 事件 id 不在 .Map 中

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

---

## 10. 控件类型码（f5，44 种）

全量清单见 `ref/controls.json`（20 场景 2041 条记录 44 种 f5）。
非 0 值集中在 20..1400 分档（100/200/.../1400）。
语义随场景逻辑解释（属 S3.3+ 场景逻辑，未完全逆出）。

---

## 11. 资源容器

### 11.1 容器清单（7 个，F-207）

| 容器 | 磁盘名 | 内容 |
|---|---|---|
| Ui | `UI.pak` | 场景 `.Map` + 控件表 |
| Glpics | `glpics.pak` | 精灵 PCX（`face_*`/`troop`/`ship`/`flag`/`landmark*`） |
| GlmapM01 | `glmap_m01.pak` | 地形地图 1 瓦片（47 条，F-207） |
| GlmapM02 | `glmap_m02.pak` | 地形地图 2 瓦片（37 条，F-207） |
| Cards | `cards.lpk` | 卡牌数据（`cards.ini`/`event_00N`） |
| Goods | `goods.lpk` | 商品数据（`goods.mdu`/`goods_0N`） |
| Music | `music.lpk` | 音频（`victory.wav`/`glmusic.mod`/`press.wav`/`invalid.wav`） |

### 11.2 LucencyPackage 容器格式

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

### 11.3 装载校验规则

```rust
pub enum SceneAsset { Pcx(String), Mod(String), Sav(String), Map(String) }
pub struct RenderError { pub missing: Vec<String> }

pub fn load_scene(
    refs: &[String], available_containers: &[&str], gl_rec_magic: u32, glmapm_indices: &[u32],
) -> Result<Vec<SceneAsset>, RenderError> {
    let mut assets = Vec::new();
    let mut missing = Vec::new();
    for r in refs {
        let (container, name) = split_container_name(r);
        if !available_containers.contains(&container) { missing.push(r.clone()); continue; }
        if container == "Glpics" && gl_rec_magic != 0x01315A47 { missing.push(r.clone()); continue; }
        if container == "Ui" && name.starts_with("glmapm") && name.ends_with(".Map") {
            let idx: u32 = name[6..name.len()-4].parse().unwrap_or(0);
            if !glmapm_indices.contains(&idx) { missing.push(r.clone()); continue; }
        }
        let asset = match name.rsplit('.').next() {
            Some("pcx") => SceneAsset::Pcx(name.to_string()),
            Some("mod") => SceneAsset::Mod(name.to_string()),
            Some("sav") => SceneAsset::Sav(name.to_string()),
            _ => SceneAsset::Map(name.to_string()),
        };
        assets.push(asset);
    }
    if missing.is_empty() { Ok(assets) } else { Err(RenderError { missing }) }
}
```

**事务性**：`Ok` 时才写入内部 `scenes`；`Err` 时 `scenes` 保持调用前不变。

### 11.4 通用装载器

`场景资源通用装载(buf, ui, name)`：按容器 + 资源名从索引取 PCX/控件表 → 装入场景。所有场景装载器共用。

### 11.5 场景管理器结构

```rust
pub struct SceneManager { scenes: Vec<SceneAsset> }

impl SceneManager {
    pub fn new() -> Self { SceneManager { scenes: Vec::new() } }
    pub fn assets(&self) -> &[SceneAsset] { &self.scenes }
    pub fn load_scene(&mut self, refs: &[String], containers: &[&str], magic: u32, indices: &[u32])
        -> Result<Vec<SceneAsset>, RenderError> {
        let assets = load_scene(refs, containers, magic, indices)?;
        self.scenes = assets.clone();
        Ok(assets)
    }
    pub fn render_frame(&self) -> RenderPlan {
        RenderPlan { layers: self.scenes.clone(), blit_count: self.scenes.len() }
    }
}
```

---

## 12. 渲染管线

### 12.1 图层顺序 = 装载顺序

```
GXBeginDraw
  → 按装载顺序逐层 blit（不排序、不分组）
GXEndDisplay
```

```rust
pub struct RenderPlan {
    pub layers: Vec<SceneAsset>,
    pub blit_count: usize,
}
```

### 12.2 逻辑帧缓冲

```rust
pub const LOGICAL_W: u32 = 176;
pub const LOGICAL_H: u32 = 220;
pub const FB_PIXELS: usize = (LOGICAL_W * LOGICAL_H) as usize;

pub struct Framebuf {
    pixels: Box<[u16]>,
    dirty_rows: [bool; 220],
}

impl Framebuf {
    pub fn new() -> Self { Framebuf { pixels: vec![0u16; FB_PIXELS].into_boxed_slice(), dirty_rows: [true; 220] } }
    pub fn clear(&mut self, color: u16) { self.pixels.fill(color); self.dirty_rows.fill(true); }
    pub fn as_slice(&self) -> &[u16] { &self.pixels }
    pub fn as_mut_slice(&mut self) -> &mut [u16] { &mut self.pixels }
    pub fn mark_dirty(&mut self, x: i32, y: i32, w: u32, h: u32) {
        for row in (y.max(0) as usize)..((y + h as i32).min(LOGICAL_H as i32) as usize) {
            self.dirty_rows[row] = true;
        }
    }
    pub fn has_dirty(&self) -> bool { self.dirty_rows.iter().any(|&d| d) }
    pub fn dirty_row_range(&self) -> (usize, usize) {
        (self.dirty_rows.iter().position(|&d| d).unwrap_or(0), self.dirty_rows.iter().rposition(|&d| d).unwrap_or(0))
    }
    pub fn reset_dirty(&mut self) { self.dirty_rows.fill(false); }
}
```

### 12.3 视口（坐标变换）

```rust
pub struct Viewport { scale: f32, offset_x: f32, offset_y: f32 }

impl Viewport {
    pub fn screen_to_logical(&self, sx: f32, sy: f32) -> (f32, f32) {
        ((sx - self.offset_x) / self.scale, (sy - self.offset_y) / self.scale)
    }
    pub fn logical_to_screen(&self, lx: f32, ly: f32) -> (f32, f32) {
        (lx * self.scale + self.offset_x, ly * self.scale + self.offset_y)
    }
}
```

等比缩放（letterbox/pillarbox），保证任意窗口尺寸下坐标互逆（P4-2 命中判定不偏移）。

### 12.4 呈现器

```rust
pub enum UploadMode { FullRebuild, Partial }

pub struct FramePresenter { mode: UploadMode }

impl FramePresenter {
    pub fn present(&self, fb: &Framebuf) {
        if !fb.has_dirty() { return; }
        let (min, max) = fb.dirty_row_range();
        match self.mode {
            UploadMode::FullRebuild => { /* 全帧上传 */ }
            UploadMode::Partial => { /* 只上传 [min, max] 行区间 */ }
        }
    }
}
```

- 脏行带部分上传：只上传 `[min_dirty, max_dirty]` 行区间
- 静止帧零 GPU 工作（`has_dirty() == false` → 跳过整帧）
- 首次呈现必上传（新建时全脏）

### 12.5 字形光栅

```rust
pub struct Raster { font: Option<FontSet> }
pub const GLYPH_ADVANCE: i32 = 12;
pub const INK_COLOR: u16 = 0xFFFF;

impl Raster {
    pub fn draw_glyph(&self, fb: &mut Framebuf, x: i32, y: i32, ch: u16) {
        if let Some(font) = &self.font {
            if let Some(glyph) = font.get_glyph(ch) { glyph.blit(fb, x, y, INK_COLOR); return; }
        }
        draw_tofu(fb, x, y);
    }
}
```

- 真 `GoldLand.Fnt` 墨点渲染
- 缺字画**豆腐块**回退（绝不静默空白）
- 字库字符集 1016 字（`font.dat` UTF-16 字符集清单）

### 12.6 场景渲染函数

| 函数 | 逻辑 | 常量 |
|---|---|---|
| `IndexScene::render`（`scenes/index.rs`） | 主菜单（标题 + 菜单项 + 选中高亮 + 状态行），坐标读 `scenes.json` index 段 | `list.y=48`, `row_h=32` |

> FIND-101：旧 `MenuShell::render` + `MENU_TOP=96`/`ROW_H=20`/`STATUS_X=4`/`STATUS_Y=208` 常量已随死路径删除；
> 主菜单渲染改由 `IndexScene` 走 `scenes.json`（与触摸命中区 `menu_row_at` 同源，EXP-071 正逆成对）。

静态帧优化：`last_rendered` 同步源 = `GameApp.menu_selected()`（Index 场景选中项）；`last_rendered_scene` 场景切换时强制整帧重渲染。

---

## 13. 音频控制

| 项 | 实现 |
|---|---|
| BGM | OGG 循环播放（`glmusic.ogg`） |
| 音效 | WAV 即时播放（`press.wav`/`invalid.wav`/`victory.wav`） |
| 音量 | 音效 + BGM 双通道，0~100% |
| 静音 | 全局开关 + 浏览器后台自动静音（`visibilitychange`） |
| 通道限制 | 音效并发 ≤ 8 |
