# 模版库（Templates）

可直接复制套用的模版。每个模版后附「套用后必须做什么」的收尾清单。
脚本化套用见 [`04_tooling.md`](04_tooling.md) §2。

## 1. 场景模版（Scene）

**目标**：消灭「一个场景 = handle_X + touch_X + render_X 三个重复方法」的反模式
（现状 `game_app.rs` 有 23 个这样的场景方法（7×handle_ + 7×touch_ + 7×render_，另有
render_hd/render_hd_text 两 HD 辅助方法），S8.4 重构）（S8.4 落地）。

```rust
// crates/ti-app/src/scenes/<name>.rs
//! <场景中文名>场景。规格见 docs/business/<module>.md §<节>。
//! （S8.4 落地：`ti_app::scene::{Flow, Scene}` 与 `ti_app::ui::{Framebuf, Rect, StringTable}` 待建）

use ti_app::scene::{Flow, Scene};
use ti_app::ui::{Framebuf, Rect, StringTable};
use ti_core::rules::<module>::<Session>;

/// <场景中文名>会话。状态全部在此，不放全局。
pub struct <Name>Scene {
    /// 玩法状态（唯一数值来源，禁止在表现层另存一份）。
    session: <Session>,
    /// 文本表（key → 中文）。
    s: &'static StringTable,
}

impl Scene for <Name>Scene {
    /// 输入 → 流转决策。`true` = 消费；`false` = 交回全局（切场景/退格）。
    fn on_event(&mut self, ev: &Event) -> Flow {
        match ev {
            Event::Up | Event::Prev => { self.session.move_cursor(-1); Flow::Redraw }
            Event::Down | Event::Next => { self.session.move_cursor(1); Flow::Redraw }
            Event::Confirm => { /* 调 rules 的动作；返回切场景时用 Flow::Goto(..) */ Flow::Redraw }
            Event::Back => Flow::Back,
            _ => Flow::Pass,
        }
    }

    /// 渲染。布局/配色来自 data/scenes.json，文本经 `s.get(key)`，不写中文字面量。
    fn render(&self, fb: &mut Framebuf, layout: &SceneLayout) {
        layout.clear(fb);
        layout.title(fb, self.s, "scene.<name>.title");
        // 列表项：位置来自 layout 迭代器，样式来自 layout 常量。
        for (i, item) in self.session.items().iter().enumerate() {
            let r = layout.row_rect(i);
            if i == self.session.cursor() { layout.row_selected(fb, r); }
            layout.row_text(fb, r, self.s, item.label_key());
        }
        layout.hint(fb, self.s, "scene.<name>.hint");
    }
}

// 套用收尾：
// [ ] 1) crate 里 `pub mod <name>;` 登记（ti-app/src/lib.rs）
// [ ] 2) data/scenes.json 增 `<name>` 条目 + data/strings.json 增 key
// [ ] 3) tests/scenes_<name>.rs 至少覆盖：初始 cursor / 移动上下界 / Confirm 流转 / render 非空白
// [ ] 4) 在 GameApp 的场景注册表登记（不再写 match 分支）
```

**关键约定**：键盘 `Event` 与触摸 `Event` **先归一**再进 `Scene::on_event`，
所以一个场景只需实现一次输入（现状 `handle_X` 与 `touch_X` 是重复的）。

## 2. 控件模版（Widget）

**目标**：让 `ref/controls.json` 的 2041 条控件记录真正驱动 UI，替代 `fill_rect` 手绘。

```rust
// crates/ti-app/src/ui/widgets/<kind>.rs
//! <控件类型>渲染。字段语义见 docs/facts/map.md（F-3xx），布局见 data/controls.json。
//! （S8.5 落地：`ti_app::ui::widgets` 与 `Control` trait 待建）

use super::{Control, Framebuf, StringTable};

/// <控件类型>：<一句话描述>。
#[derive(Debug, Clone)]
pub struct <Kind> {
    /// 控件矩形（逻辑坐标，来自 controls.json）。
    pub rect: Rect,
    /// 显示文本 key（中文在 strings.json，不在代码里）。
    pub text_key: &'static str,
    /// 是否可见。
    pub visible: bool,
}

impl Control for <Kind> {
    /// 按类型分派绘制。
    fn draw(&self, fb: &mut Framebuf, s: &StringTable) {
        if !self.visible { return; }
        match self.kind_id() { /* 44 种 f5 类型在此归组 */ _ => {} }
        fb.text(self.text_key, s, self.rect);
    }

    /// 命中判定（触摸）。
    fn hit(&self, p: Point) -> bool { self.visible && self.rect.contains(p) }
}

// 套用收尾：
// [ ] 1) 在 WidgetKind 枚举增变体 + from_f5(u16) 映射（f5 非 0 档）
// [ ] 2) data/controls.json 补该类型的渲染参数（不要写进 .rs）
// [ ] 3) 单测：draw 后该区域墨点 > 0；hit 边界（矩形内外各一点）
```

## 3. 数据表模版（balance / scenes / controls / strings）

**目标**：数值与文本一律 `assets/data/*.json`，调平衡不改代码。

```jsonc
// assets/data/balance.json —— 片段
{
  "gold_max": 999999,              // 金币上限
  "boss_tiers": [                  // BOSS 档（替代硬编码表）
    { "tier": 0, "hp": 50000 },
    { "tier": 1, "hp": 100000 }
  ],
  "damage": {                      // 伤害模型参数（规则见 docs/business/combat.md）
    "min": 1,
    "atk_scale": 1.0,
    "player_first": true
  }
}
```

```jsonc
// assets/data/scenes.json —— 片段（替代散落的 fill_rect 常量）
{
  "select_role": {
    "bg": 16,
    "title": { "x": 40, "y": 4, "style": "title" },
    "list": { "x": 20, "y": 50, "w": 136, "row_h": 30, "item_h": 26 },
    "selected": { "style": "highlight" },
    "hint": { "x": 30, "y": 210 }
  }
}
```

```jsonc
// assets/data/strings.json —— 片段（替代内联中文）
{
  "scene.select_role.title": "选择角色",
  "scene.select_role.hint": "上下切换  确认",
  "battle.win": "胜利!"
}
```

**加载约定**：`include_str!` 进包 → serde 反序列化 → `OnceLock` 缓存；
解析失败 = 硬错误，不 fallback 默认值（见 `02_conventions.md` §1）。

## 4. 玩法模块模版（rules）

```rust
// crates/ti-core/src/rules/<sys>.rs
//! <系统名>规则。业务规格见 docs/business/<module>.md §<节>（唯一事实源）。
//! 本模块纯逻辑、零 IO、确定性（同输入同输出，禁止 thread_rng）。

/// <系统>会话状态。
#[derive(Debug, Clone, Default)]
pub struct <Sys>Session { /* 字段… */ }

impl <Sys>Session {
    /// 构造。数值来自 balance.json（调用方注入，测试可注假值）。
    #[must_use] pub fn new(cfg: &<Cfg>) -> Self { /* … */ }

    /// 光标移动。返回是否发生变化（供表现层决定重绘）。
    pub fn move_cursor(&mut self, delta: i32) -> bool { /* … */ }

    /// 主动作（Confirm）。返回动作结果供表现层播提示。
    pub fn confirm(&mut self) -> <Result> { /* … */ }

    /// 只读视图（渲染用）。避免表现层拿到 &mut。
    #[must_use] pub fn items(&self) -> &[<Item>] { /* … */ }
}

// 套用收尾：
// [ ] 1) tests/rules_<sys>.rs：边界（首/末移动）、状态机全覆盖、确定性（同输入两次结果相同）
// [ ] 2) 数值断言从 balance.json 读，测试里断言「读到的值 == JSON 里的值」
// [ ] 3) 不写 .unwrap()/.expect()（src 下为 0 命中）
```

## 5. 测试模版

```rust
// crates/<crate>/tests/<name>.rs
//! <被测点>。判据见 docs/parity.md 对应条目 或 ledger 任务 dod。

use <crate path>::*;

/// <一句话说明这个测试在证明什么>。
#[test]
fn <行为>_<期望>() {
    // 构造：优先用测试夹具/fake，不用真实文件/窗口
    let mut s = make_fixture();

    // 动作
    s.move_cursor(-1);

    // 断言：具体期望值，不断言「不是 None」这类空断言
    assert_eq!(s.cursor(), 0);
}
```

测试约定见 [`02_conventions.md`](02_conventions.md) §5。

## 6. 脚本模版（Python，供 ti-tools / scripts 复用）

```python
#!/usr/bin/env python3
"""<一句话作用>。

用法：python scripts/<name>.py <子命令> [--out PATH]
幂等：同一输入两次运行，产物逐字节一致（进 gate 第 10 项）。
退出码：0 成功；非 0 失败（gate 直接判 FAIL）。
"""

import argparse
import sys
from pathlib import Path


def do_check(root: Path) -> int:
    """校验：只读，报告问题清单，有问题时返回非 0。"""
    problems: list[str] = []
    # ... 逐项检查，problems.append(f"{path}: {原因}")
    for p in problems:
        print(f"ERROR {p}")
    return 1 if problems else 0


def do_gen(root: Path, out: Path) -> int:
    """生成：幂等写入，--check 模式只比对不写。"""
    # ...
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("cmd", choices=["check", "gen"])
    ap.add_argument("--root", type=Path, default=Path("."))
    ap.add_argument("--out", type=Path)
    ap.add_argument("--check", action="store_true", help="只校验不改产物")
    a = ap.parse_args()
    if a.cmd == "check":
        return do_check(a.root)
    return do_gen(a.root, a.out)


if __name__ == "__main__":
    sys.exit(main())
```

**硬要求**：脚本必须幂等 + `--check` 只读模式 + 非 0 退出码让 gate 判 FAIL。

## 7. 模块包任务书模版（并行轮下发用）

> 用法：契约冻结轮按 [`05_module_parallel.md`](05_module_parallel.md) §2 逐包填充本模版，
> 下发给领取该包的会话；**契约未 `frozen` 不许下发**。模版字段一个都不能少。

```markdown
# 任务书 <包 ID> <包名>（S<轮>.<n>）

## 环境事实（下发方填，子代理禁止靠摸目录试错 —— EXP-064）
- 工作目录：`<绝对路径>`（本项目 = `C:\金银岛`）
- crate 前缀：`ti-`（**不是** `ty-`；目录名 `crates/ti-app`、`crates/ti-core`、`crates/ti-platform`、`crates/ti-tools`、`crates/ti-formats`）
- 关键命令：`cargo build --workspace` / `cargo test --workspace` / `cargo clippy --workspace --all-targets -- -D warnings` / `grep -rn "oracle" crates/`
- 逆向产物已归档在 `legacy/`（`ref/`、`TreasureIsl.exe`、`oracle-rust/` 均在 `legacy/` 下），`rules/` 与 `domain/` 层正向代码**不得** import `legacy::oracle`（EXP-061）
- 当前在途并行包（本包之外的）：<包 ID 清单 + 各自白名单一句话>，避免误改他包文件

## 契约（只读，不许改）
- 公共类型：docs/contracts/c1_core_public_types.md（状态须为 frozen）
- 数值/文本 key：docs/contracts/c2_data_keys.md
- 表现层会话 API：docs/contracts/c3_app_host_api.md（P6 表现层包才需要）

## 文件白名单（只能改这些，越界即违规）
- <crate 路径>/src/<module>.rs
- <crate 路径>/tests/<test>.rs

## 业务依据（唯一事实源）
- docs/business/<module>.md §<节>  ← 规则
- assets/data/balance.json key <a.b.c>  ← 数值（不在代码里写死）
- 可测规则表：本包负责第 <n> 条（见下）

## 工程规范
- docs/process/02_conventions.md（数据驱动/命名/错误/测试）
- docs/process/03_templates.md §4（玩法模块模版）/ §5（测试模版）
- docs/style.md §5（Rust 规范）

## 开轮必读（下发方列清单，子代理逐条回执 —— 06_flywheel §开轮必读）
- [ ] docs/process/00_index.md + docs/business/00_index.md
- [ ] docs/process/01_architecture.md + 02_conventions.md
- [ ] docs/contracts/00_index.md + <涉及契约 c1/c2/c3>
- [ ] docs/experience/index.md（重点读 <本包域> 相关 EXP）
- [ ] scripts/gate.ps1 项清单 + doccheck2 断言清单
- [ ] 在途并行包白名单：<本轮 P 包清单 + 文件域>

## 规范回执（子代理回报必填）
- 已读：<逐条>
- 命中现行规范：<条数 + 哪几条>
- 读到本包域相关 EXP：<EXP-xxx>
- 未读/读不了：<原因>

## 禁区（违反即打回）
1. 跨包 import（只许契约里的公共类型）
2. 写死数值/中文面量（进 balance.json / strings.json）
3. TODO / unimplemented! / panic! / unwrap()（tests 除外）
4. 白名单外文件；契约 `frozen` 段；`docs/business/`；台账 verified 行

## 分支与基线
- 分支：task/S<轮>.<n>
- 基线：标签 baseline-s<轮>（从该标签切出，勿从主干 HEAD 切）

## 包内验收（三条全绿才算「包内绿」）
1. cargo test -p <crate> --test <本包测试>            # test result: ok
2. 越界 grep = 0                                      # 只碰白名单文件
3. 业务可测规则表本包条目全部有对应用例

## 产物与回报
- 代码 + evidence/S<轮>.<n>.log（包内自检实跑输出）
- 回报格式：「包内绿 / 卡在哪 / 是否触碰契约边界」（**不写台账 verified**）
```

**并行纪律提示**：台账 `verified` 与 `evidence` 由集成轮统一给（见 [`05_module_parallel.md`](05_module_parallel.md) §5），
包会话回报「包内绿」即可；越界或触碰契约 → 立即停手并回报，不许自行扩大白名单。