# 第 7 章：渲染基础

## 学习目标

- [ ] 理解帧缓冲（Framebuffer）的概念和大小
- [ ] 掌握基本图元绘制（矩形/文字/精灵）
- [ ] 理解 Z-order（绘制顺序）和裁剪
- [ ] 理解脏行带优化（只传变化区域）
- [ ] 理解 Bevy GPU 渲染路径（S18.8a 起）
- [ ] 能识别本项目渲染架构中每层的职责

**关键洞察（30 秒版）：**

```
  ① 帧缓冲 = 一张 1080×1920 的像素网格（16 位色，~4MB）
     画完整张贴到屏幕（旧 CPU 路径）或交给 GPU 画（新 Bevy 路径）

  ② 渲染顺序 = 先画背景，后画前景（后画的压前面的）
     本项目的 Z-order = 函数调用顺序（无枚举，越靠后越上面）

  ③ 裁剪 = 视野外的不画（省 40% 计算）
  ④ 脏行带 = 只有变化了的行才上传到 GPU（省 99% 传输）

  ⑤ S18.8a 起：渲染引擎 = Bevy GPU（wgpu），不再是 CPU 逐像素写
     CPU 帧缓冲降为"测试/快照后端"，生产路径走 Bevy
```

---

## 一、帧缓冲（Framebuffer）

**生活类比：画家的画布。** 画完整张再贴到屏幕上。

```
  画布 = 1080 x 1920 的像素网格（不是 176x220！那是 legacy 旧坐标）
  每个像素 = RGB565（16 位色，2 字节）
  总大小 = 1080 x 1920 x 2 = 4,147,200 字节（约 4 MB）

  画完 → 整张贴到屏幕（旧 CPU 路径）
       → 或交给 Bevy GPU 画（新 S18.8a 路径）
```

**为什么是 1080×1920 不是 176×220？**

```
  176×220 = 原版手机游戏的分辨率（macroquad 时代）
  S18 迁移到 Bevy 后，逻辑坐标系改为 1080×1920
  176×220 是 legacy 坐标，运行时禁止出现（doccheck2 grep 门禁）
  画布单一事实源 = crates/ti-render/src/framebuffer.rs:
    CANVAS_W = 1080
    CANVAS_H = 1920
```

**帧缓冲在哪**：

```
crates/ti-render/src/framebuffer.rs:7  — CANVAS_W = 1080
crates/ti-render/src/framebuffer.rs:9  — CANVAS_H = 1920
crates/ti-render/src/framebuffer.rs:21 — Framebuf（RGB565 固定大小缓冲）
```

---

## 二、基本图元

**生活类比：画家的基本笔触。** 所有画面 = 这些笔触的组合。

| 图元 | 做什么 | 用途 |
|---|---|---|
| `fill_rect(x, y, w, h, color)` | 涂一个方块 | 按钮/面板/背景/棋盘格 |
| `draw_text(x, y, "文字")` | 写一行字 | 标题/列表项/状态行 |
| `blit_image(asset, x, y, w, h)` | 贴一张图 | 背景/角色/地形（S18.8b） |

**本项目的绘制原语只有 3 个**（`Canvas` trait 方法）：

```rust
// crates/ti-render/src/canvas.rs
fn fill_rect(x, y, w, h, color)   // 矩形
fn draw_text(x, y, text, ...)      // 文字（TTF 三档 96/44/28px）
fn blit_image(asset, x, y, w, h)  // 贴图（RGBA8 位图，alpha 混合）
```

**没有"画一个三角形"或"画一个圆形"的原语**——所有形状都是矩形 + 文字 + 贴图的组合。
这是有意为之（S15 渲染线裁决：运行时零 legacy 像素，画面 100% 由代码绘制产出）。

---

## 三、Z-order（绘制顺序）

**核心：后画的压先画的。** 就像贴贴纸，后贴的盖住先贴的。

```
  画错（先人物后背景）：            画对（先背景后人物）：
    ① 画人物                       ① 画背景
    ② 画背景（盖住人物）❌          ② 画人物（盖在背景上）✅
```

**本项目的 Z-order = 函数调用顺序（无枚举）**：

```
  chrome.rs::scene_shell 的调用序（从底到顶）：

  ① parchment_bg（背景/画布）
  ② 外框/标题条
  ③ 面板/棋盘格
  ④ 角色/物体/地形
  ⑤ UI 控件（按钮/血条）
  ⑥ 文本（永远最上层）

  代码里就是按这个顺序调用 fill_rect / blit_image
  没有 z_index 字段，没有 z-order 枚举
  "谁先画" = "谁在代码里先被调用"
```

**为什么不用 z_index 枚举？**

```
  因为本项目的层是固定的（场景结构不变）
  层序在 chrome.rs 的调用序里硬编码
  加新层 = 在调用序里插一行（不用改枚举）
  Bevy 路径有 z_index（EXP-103：Bevy 0.20 z 层用负值 + 升序绘制）
```

---

## 四、裁剪（Clipping）

**视野外的不画。**

```
  世界地图 g×g 格（如 20×20 = 400 格）
  视野只装下其中一小块（约 10×8 = 80 格）

  不裁剪：400 格全算 → 白算 80%
  裁剪：  只算可见 80 格 → 省 80% 计算

  具体做法：绘制精灵前，检查目标矩形是否与视野矩形相交
    相交 → 画
    不相交 → 整个跳过（连循环都不进）
```

**本项目的裁剪**：

```
crates/ti-app/src/scenes/main_map.rs — 按 board_fit 计算可见格范围
docs/process/07_render_pipeline.md   — "越界求交裁剪"契约
```

> 注意：裁剪的是"格子"（逻辑单位），不是"像素"（物理单位）。
> 先算哪些格可见，再只画那些格。

---

## 五、脏行带优化（Dirty Rectangle）

**生活类比：只擦变化的作业页。** 100 行作业改了 2 行，不用全擦重写。

```
  帧缓冲 = 1920 行，每行 1080 个像素

  全量上传：每帧传 1920 行 × 1080 × 2 字节 = ~4 MB
  脏行带：  只有变化了的行才传

  帧 1：画面全变（场景切换）→ 全量上传 1920 行
  帧 2：只有光标移动 → 只传第 98 行附近 = 2~3 行
  帧 3：什么都没变 → 不传任何行

  回合制游戏大部分帧"什么都没变" → 省 99% 传输量
```

**本项目的脏行带**：

```
crates/ti-platform/src/gfx.rs:136  — dirty_rows: [bool; 1920]
  每行一个标记，true = 这行改了
  present 时只把 true 的行上传到 GPU 纹理
```

> S18.8a 起，生产路径走 Bevy GPU（wgpu），CPU 帧缓冲降为"测试/快照后端"。
> 脏行带优化在 CPU 后端仍有价值（离屏截图/渲染测试），GPU 路径由 wgpu 驱动。

---

## 六、Bevy 渲染路径（S18.8a 起）

**本项目渲染引擎 = Bevy 0.20（GPU，wgpu），不是 CPU 逐像素写。**

```
  旧路径（macroquad 时代）：
    CPU 逐像素写帧缓冲（1080×1920 × 2 字节 = 4MB 每帧）
    → 单纹理 blit 到屏幕
    → CPU-bound，无 GPU 加速

  新路径（S18.8a 起，现行）：
    Bevy 2D（wgpu）：
      矩形 → Mesh2D（GPU 立即绘制）
      文字 → Text2d（GPU 图集）
      贴图 → Sprite（GPU 纹理）
      相机 → Camera2d + OrthographicProjection（letterbox）
    → GPU-driven，空闲成本≈0
```

**架构形状（`ti-engine` crate）**：

```
crates/ti-engine/src/scene_render.rs  — BevyCanvas（Canvas trait 的 Bevy 实现）
crates/ti-engine/src/bin/treasure_bevy.rs — 主循环 + 相机 + 输入
crates/ti-render/src/canvas.rs        — Canvas trait（后端无关原语）
```

**CPU 帧缓冲的角色**：

```
  保留用途：
    · 离屏截图（render_invariants 测试）
    · 渲染快照门禁（bevy-cpu-parity：GPU 读回 vs CPU 期望逐像素对照）
    · 非 GPU 平台的回退路径

  不再是"生产渲染路径"（生产 = Bevy GPU）
```

**坐标换算（1080 逻辑 → Bevy 世界）**：

```
  Bevy 2D 世界：原点在画布中心（540, 960），y 向上
  1080 逻辑：原点在画布左上（0, 0），y 向下

  换算（唯一口径，scene_render.rs::to_world）：
    world_x = logical_x − 540
    world_y = 960 − logical_y
```

![渲染流水线：场景 → 帧缓冲 → GPU → 屏幕](../../evidence/tutorial/07_rendering_pipeline.png)

---

## 七、项目对照

| 概念 | 文件 | 行号 |
|---|---|---|
| 帧缓冲常量 | `ti-render/src/framebuffer.rs` | L7 CANVAS_W=1080, L9 CANVAS_H=1920 |
| 帧缓冲体 | `ti-render/src/framebuffer.rs` | L21 Framebuf（RGB565） |
| 脏行带 | `ti-platform/src/gfx.rs` | L136 dirty_rows: [bool; 1920] |
| Canvas trait | `ti-render/src/canvas.rs` | 3 原语：fill_rect / draw_text / blit_image |
| Bevy 渲染 | `ti-engine/src/scene_render.rs` | BevyCanvas 实现 |
| Z-order 调用序 | `ti-app/src/scenes/chrome.rs` | scene_shell 函数调用顺序 |
| 裁剪 | `ti-app/src/scenes/main_map.rs` | board_fit 可见格计算 |

---

## 八、实践练习

**练习 1：帧缓冲大小**

计算：1080×1920，RGB565（2 字节/像素），总字节数是多少？
（提示：1080 × 1920 × 2 = ?）

**练习 2：识别渲染顺序**

打开 `crates/ti-app/src/scenes/chrome.rs` 的 `scene_shell` 函数，标注绘制顺序（从底到顶）。
确认：文本是否永远在最上层？

**练习 3：脏行带**

在 `ti-platform/src/gfx.rs` 找到 `dirty_rows`，理解：
- 什么时候整行标记为 dirty？（`mark_dirty` 调用）
- `present` 时如何只上传 dirty 行？
- 回合制游戏为什么大部分帧 dirty_rows 全 false？

**练习 4：Bevy 坐标换算**

画布坐标 (100, 200) → Bevy 世界坐标是多少？
（提示：world_x = 100 − 540 = −440，world_y = 960 − 200 = 760）

## 常见坑

| 坑 | 原因 | 避免 |
|---|---|---|
| 用 176×220 当画布大小 | legacy 坐标，运行时禁止 | 画布 = 1080×1920（doccheck2 grep 门禁） |
| 越界写入 | 没有裁剪 | 绘制前检查边界（board_fit 可见格） |
| Z-order 错误 | 绘制顺序不对 | 按 scene_shell 调用序（先背景后前景） |
| Bevy 坐标 y 轴方向反 | Bevy y 向上，1080 y 向下 | `world_y = 960 − canvas_y`（减不是加） |
| 每帧全量重绘 | 没有脏行带 | 只上传 dirty 行（CPU 后端）/ wgpu 自动优化（GPU 路径） |
| 文字模糊 | 没用预光栅化字形 | TTF 三档 96/44/28px，回退位图 advance |

## 学习记录

- 状态：未开始
- 完成日期：
- 心得/疑问：
