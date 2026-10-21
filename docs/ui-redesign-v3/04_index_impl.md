# 04 · index 主菜单 v3 试点落地实现（v3.1 双写）

> **S18.7b：像素基线机制已退役（不再存在）**。
> 依旧的 `assets/render_baseline/` / `forward_baseline/` / `hd_baseline/` 三套 golden PNG、
> `render_snapshot` / `render_snapshot_forward` / `visual_regression` 三个 diff 测试、`visual_baseline.ps1`
> 与其门禁项、`baseline-mirror-identity` 校验均**已全部删除**。
> 现行机制：**结构不变式断言**（`tests/render_invariants.rs`，7 场景 × 非黑覆盖率/内容签名互异/暖色板在位/TTF 在位）
> 与 **CPU↔Bevy 内存对拍**（`bevy-cpu-parity` 门禁项：GPU 真实离屏读回 vs CPU 期望逐像素对照）。
> 本文中仍有的「基线」词句为**历史记述**，不再是现行机制。

**定位**：`03_redesign_proposal.md §3.1` 的 index 试点（v3.3 波次 = 本任务 S17.6 提前落地
试点层）。**双写纪律**（§3.10）：v2 键（`title`/`list`/`icon`/`status`/`portrait_zones`）
原值不动，v3 叙事键（`title_art`/`ambient_art`/`focus_anchor`）叠加；v3.5 收口时 v2 键才退役。

> 本文件 = index 试点的**坐标方案 + 实现记录 + 验收判据**。数值层 = `scenes.json`；
> 叙事约束层 = `ui_schema/scene_manifest_v3.json`（gate 第 22 项 `check_scene_narrative`）。

---

## 1. index v3.1 坐标方案（1080×1920）

```
┌─────────────────────────────────────┐ y=0
│ ★ 星空氛围层 ambient_art（全屏，26 星） │
│ [标题条 v2 y=12..108「金银岛」]        │  （v2 保留）
│                                      │
│ ╔══════ 仪式面板 title_art ═════════╗ │ y=192..352（空带，菜单首块顶 352 之上）
│ ║  木纹双描边 + bevel + 上下金饰线    ║ │
│ ║        ┼ 罗盘饰（56×3 针 + 8×8 点）  ║ │  罗盘中心 (540, 272)
│ ╚═══════════════════════════════════╝ │
│ [菜单 6 项 v2 几何 352..1640 不动]     │
│ [尾行 y=1840 v2 不动]                 │
└─────────────────────────────────────┘
```

| 键 | 值（`scenes.json` index 段） | 说明 |
|---|---|---|
| `title_art {x:108, y:192, w:864, h:160}` | 仪式面板绘制区 | 落在标题条底 108 与菜单首块顶 352 之间的**空带**，`chrome::title_art_block` 消费；键缺失/非正 = 不画（契约 4） |
| `ambient_art {x:0, y:0, w:1080, h:1920}` | 星空绘制区 | `chrome::star_sky` 消费；绘制自动夹 12px 外框内（星不盖 `wood_frame`） |
| `focus_anchor {bbox:{x:108, y:192, w:864, h:672}}` | 叙事重心区 | 与 manifest index `focus.bbox_pct {10,10,80,35}` × (1080,1920) 恒等；**gate 第 22 项 N8 对账锚点**；运行时零消费（Python 直读 JSON） |

**v3.1 → v3.2 演进**（不在本轮）：菜单瘦身后（`list` 项 v3 值 + doccheck2 fidelity 锚同步 +
08 §4.1 表同步），`title_art` 扩为完整仪式感区（提案 §3.1 的 672 高面板 + 巨字标题），
focus_anchor 语义区与绘制区合并。

## 2. 实现物（本轮 diff 面）

| 物 | 文件 | 内容 |
|---|---|---|
| 星空原语 | `ti-render/src/shell.rs` `star_sky` | 26 星常量表（无 RNG，快照可复现）；暗星 1px `parch_light` / 亮星 2px `gold_glow` + 四向 1px 光晕；屏心净空 12%（P-SoT-2）；夹 12px 外框内 |
| 仪式面板原语 | `ti-render/src/shell.rs` `title_art_block` | 卡底 + 木纹双描边 + bevel + 中央罗盘饰（`title_bar` 罗盘 28×3 语言 ×2）+ 上下金饰线；内区 `ih < 96` 时只画饰线不画罗盘（防挤压） |
| 数据驱动包装 | `ti-app/src/scenes/chrome.rs` `star_sky` / `title_art_block` | 读 `scenes.json` 该场景 `extra` 键；缺失/非正 = 不画（与 icon 列/实体 size 同判据） |
| 接线 | `ti-app/src/scenes/index.rs` `render` | z-order（调用序）：`scene_shell`（v2 外壳）→ `star_sky` → `title_art_block` → 菜单（v2）→ 尾行（v2） |
| 8pt 豁免 | `ti-tools/src/bin/doccheck2.rs` `ui_8pt_walk` | v3 叙事键 `title_art`/`ambient_art`/`focus_anchor` 出 8pt 强约束（v3 规格 `02 §10`：v3 坐标由 DSL 接管）；**v2 键仍走 8pt 纪律** |
| 文档镜像 | `docs/business/08_layout_design.md §4.1` | 新键 3 行（doccheck2 `check_layout_doc_table_cross_check` 对账面） |

## 3. 验收判据（S17.6 DoD 对应）

1. `python scripts/check_scene_narrative.py` → N1-N8 **0 失败**，N8 index `focus_anchor` 对账
   **激活**（其余 6 场景 SKIP 至 v3.2+）
2. index 帧含新层（像素抽检，`evidence/render_png/index.png` + `git show HEAD:` 旧基线对照）：
   - 罗盘金点 `(540,272)` = `0xFE55` 金墨（旧 = 深底）
   - 亮星 `(244,106)` = `0x4E3D` / 暗星 `(96,182)` = `0x10C9`（旧 = 背景）
   - 菜单项 `(540,420)`、尾行 `(540,1840)` 新旧**恒等**（v2 几何未动）
3. 三套基线重生成：`render_baseline/index.png` + `forward_baseline/index.png`（S17.6 新帧）
   + `hd_baseline/Index.png`；`vs.png` 两套 = **S17.4 动效 WIP 吸收**（归因标注，非本任务漂移）；
   其余 5 场景 diff = 0
4. `cargo test --workspace` 0 failed + `clippy -D warnings` + `fmt --check` 0 diff
5. `doccheck` 0 error + `doccheck2` 0 error（含 08 §4.1 表镜像 + gate 项数 23 口径 +
   基线镜像恒等）
6. gate 22 项全绿 → `evidence/S17.6.log` → 台账 S17.6 verified

## 4. 已知限制（诚实记录）

- 试点面板 160px 空带版：视觉升级 = 星空 + 仪式面板，**巨字标题/菜单瘦身 = v3.2**
  （依赖 `list` 值改动 + doccheck2 `check_scene_layout_fidelity` 锚点同步 + 08 §4.1 表同步，
  属独立收口轮）
- 星空 26 星为常量表（非数据驱动）：与 `nautical_grid` 硬编码几何同先例（契约 4 豁免面 =
  原语内部几何，键消费面数据驱动）
- `focus_anchor` 嵌套 `bbox` 经 Rust 解析器时 `extra["focus_anchor"]` = 空 Rect（`type`/`bbox`
  非 x/y/w/h 顶层字段被静默丢弃）——**运行时零消费属设计意图**（N8 走 Python 直读），
  勿在 chrome 里读 `focus_anchor`
