# 第 10 章：项目架构对照

## 学习目标

- [ ] 理解本项目的完整分层架构
- [ ] 掌握 trait 约定和依赖方向
- [ ] 理解场景模型和组合根模式
- [ ] 能根据架构规范参与项目开发

## 核心概念

### 分层架构

**生活类比：餐厅分工**

```
┌─────────────────────────────────────────────┐
│ 前厅（ti-app）      服务员：接待/点菜/上菜/收银 │
├─────────────────────────────────────────────┤
│ 后勤（ti-platform） 采购/仓库/设备/水电        │
├─────────────────────────────────────────────┤
│ 厨房（ti-core）    厨师：做菜（核心业务）      │
├─────────────────────────────────────────────┤
│ 供应商（ti-formats） 食材/调料/包装供应         │
└─────────────────────────────────────────────┘
         ↕ 单向依赖，只能向下，禁止反向依赖
```

**依赖方向规则**（层序以 `Cargo.toml` 实测为准）：

```
实测依赖（2026-10 核对）：
  ti-app      → ti-core, ti-platform, ti-formats
  ti-platform → ti-core, ti-formats
  ti-core     → ti-formats
  ti-formats  → 无内部依赖

所以：后勤(ti-platform) 在 厨房(ti-core) 之上，不是之下。
      后勤仅为分辨率常量（LOGICAL_WIDTH/HEIGHT）依赖厨房。
```

| 规则 | 说明 |
|---|---|
| 只能向下 | 前厅可以叫后勤/厨房，厨房不能指挥前厅或后勤 |
| 禁止反向 | `ti-core` 不得依赖 `ti-platform`（实测成立：只依赖 `ti-formats`） |
| 跨层直接依赖是既成事实 | `ti-app` 直接依赖 `ti-formats`（读资源需要），分层是设计意图不是 Cargo 禁令 |
| 供应商只做供货 | `ti-formats` 只做编解码，不含玩法判断、不含渲染 |

> ⚠️ 本节早期版本曾把 `ti-core` 画在 `ti-platform` 之上，与实测依赖相反。
> 已同步更正 `docs/process/01_architecture.md` §1（唯一事实源）。

### Trait 约定

**生活类比：岗位说明书**

```
每个岗位有说明书：
  服务员说明书：会点菜、会上菜、会收银
  厨师说明书：会切菜、会炒菜、会摆盘

  不管谁来当服务员，按说明书做事
  不管谁来当厨师，按说明书做事

  换人容易（只要按说明书），换餐厅容易（只要说明书一样）
```

**本项目**（`scenes/mod.rs:475` 真实签名）：
```rust
pub trait Scene {
    /// 当前场景标识
    fn kind(&self) -> SceneKind;
    /// 处理一个输入事件，返回是否消费
    fn on_input(&mut self, host: &mut dyn SceneHost, input: GameInput) -> bool;
    /// 处理触摸（默认实现：不消费）
    fn on_touch(&mut self, host: &mut dyn SceneHost, sample: &TouchSample) -> bool {
        let _ = (host, sample);
        false
    }
    /// 渲染（场景不消费，副作用在帧缓冲上）
    fn render(&mut self, host: &mut dyn SceneHost);
}
```

### 组合根模式

**生活类比：餐厅经理**

```
经理 = 组合根（Composition Root）

职责：
  1. 管理所有资源（食材、设备、人员）→ 持有全部跨场景状态
  2. 把客人分配到对应服务员 → 把输入路由到场景
  3. 协调各部门 → 实现 SceneHost trait

不做：
  - 亲自做菜（具体业务逻辑）
  - 亲自摆盘（具体绘制细节）
```

## 项目对照

### 完整 crate 结构

```
crates/
  ti-formats/     — 供应商（pak/lpk、PCX、字体、.Map、存档）
  ti-core/        — 厨房（domain + rules + data + consts）
  ti-platform/    — 后勤（gfx + input + audio + storage + settings）
  ti-app/         — 前厅（scenes + menu_shell + sprite_atlas + bin/）
  ti-tools/       — 外部工具（unpak/map2json/build_assets/controls/specskel/doccheck）
  oracle 参照实现的唯一源是 `crates/ti-core/src/oracle/`（经 lib.rs `pub mod oracle` 导出，集成测试直接 `use ti_core::oracle::`）；旧的逐字节重复 crate `legacy/oracle-rust/` 已于 S15.8 删除
```
（共 6 个 workspace member，以根 `Cargo.toml` 的 `members` 为准）

### ti-core 内部结构

```
ti-core/src/
  lib.rs       — 模块声明 + 公共导出
  consts.rs    — 全局常量（LOGICAL_WIDTH/HEIGHT 等）
  data/        — 数据访问器（balance / controls / layout / strings）
  domain/      — 领域模型（map/economy/combat/selection/messages/persistence/flow）
  rules/       — 业务规则（battle/build/cards/cards_draw/main_loop/main_map/
                menu/messages/render/rng/saveload/saveload_json/selection/trade）
  oracle/      — 逆向参照物（默认构建 0 行参照代码参与）
```

> 注意两个易错点：
> - `data/` 下**没有** `scenes.rs`（是 balance/controls/layout/strings）；
> - `rules/` 下**没有** `economy.rs`（经济规则在 `trade.rs`，经济数据在 `domain/economy.rs`）。

### ti-app 内部结构

```
ti-app/src/
  game_app/       — 餐厅经理（GameApp + Scene 枚举 + SceneHost 实现）
  scenes/           — 服务员（index/select_role/select_map/main_map/vs/message/saveload）
  menu_shell.rs     — 摆盘台（帧缓冲/字形/呈现机制）
  sprite_atlas.rs   — 贴纸册（精灵图集）
  touch.rs          — 门铃（触摸采样）
  glyph_raster.rs   — 写字机（字形光栅化）
  shell_ui.rs       — 菜单模板（壳 UI 常量）
  i18n.rs           — 翻译（国际化）
  bin/              — android / headless 窗口（native/wasm 发货壳已迁 ti-engine）
```

### SceneHost 8 个子 trait

```
NavHost        — 领位员：带客人去对应桌（场景切换 + 输入路由）
RenderHost     — 摆盘师：摆盘（帧缓冲/字形）
SpriteHost     — 贴纸师：贴贴纸（精灵/地形渲染）
MenuHost       — 菜单管理员：菜单状态
SelectionHost  — 订位员：角色/地图选择状态
MapHost        — 地图向导：主地图状态
EconomyHost    — 收银员：金/价格/持有/银行/建筑
SessionHost    — 服务员领班：会话/战斗/消息/存读档
```

### 四端入口

> **S18.8a：发货入口 = Bevy 壳**（`crates/ti-engine`，独立 workspace）。
> ti-app 的 macroquad native/wasm 壳已删除；下表同步为现行形态。

```
ti-engine/src/bin/treasure_bevy.rs — 实体店（桌面 treasure-bevy-native）
                                   + 外卖店（浏览器 treasure-bevy-wasm，同源分支）
bin/treasure_android.rs            — 移动车（Android，S18.8d 切 Bevy）
bin/treasure_headless.rs           — 中央厨房（测试/回归）
```

## 实践练习

### 练习 1：绘制依赖图

绘制本项目的 crate 依赖图，标注每个 crate 的职责和依赖方向。

### 练习 2：理解 SceneHost

打开 `crates/ti-app/src/scenes/mod.rs`，找到 8 个子 trait 的定义，思考：
- 为什么拆分成 8 个而不是 1 个宽接口？
- 每个场景需要实现哪些子 trait？
- 如何新增一个场景？

### 练习 3：新增功能

假设要新增一个"成就"系统，需要：
- 在 `ti-core` 加 `domain/achievement.rs` 和 `rules/achievement.rs`
- 在 `ti-app` 加 `scenes/achievement.rs`
- 在 `Scene` 枚举加 `Achievement` 变体
- 在路由函数加分发分支

思考：这个改动涉及哪些文件？如何保证不违反分层规则？

## 常见坑

| 坑 | 原因 | 避免 |
|---|---|---|
| 反向依赖 | `ti-core` 依赖 `ti-platform` | 层序以 Cargo.toml 为准，`ti-platform` 在上 |
| 以为跨层是禁令 | `ti-app` 确实直接依赖 `ti-formats` | 分层是设计意图，不是 Cargo 硬约束 |
| 组合根膨胀 | 业务逻辑写在 GameApp | 业务逻辑放场景/rules |
| 场景持有 GameApp | 违反依赖方向 | 只持有 `&mut dyn SceneHost` |

## 学习记录

- 状态：未开始
- 完成日期：
- 心得/疑问：
