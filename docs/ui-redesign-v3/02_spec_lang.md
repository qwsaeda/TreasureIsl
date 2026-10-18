# 02 · UI 描述语言规格（Scene Manifest DSL v3）

**定位**：v3 的**硬约束层**。`scenes.json` 是坐标数值事实源，本 DSL 是**场景叙事约束**的事实源。
两者合一才是完整规范——坐标符合 scenes.json + 叙事符合 DSL = 一个场景才算合格。

> **设计哲学**：v2 的"8pt 整除 / 越界 / 对齐"是**几何约束**，是必要不充分条件。
> v3 新增**叙事约束**层：每个场景必须有"视觉重心"、"情绪节奏"、"信息密度梯度"。
> 这些不是软指标——它们被机器断言（`check_scene_narrative` 门禁）。

---

## 1. DSL 顶层结构

每个场景在 `assets/data/ui_schema/scene_manifest_v3.json` 描述为：

```jsonc
{
  "scene_id": "index",                       // 与 scenes.json key 对齐
  "version": "3.0",
  "narrative": {
    "mood": "epic_entry",                    // 见 §2 情绪枚举
    "focus": {                               // 视觉重心（必填，唯一）
      "anchor": "title_art",                 // 重心锚类型，见 §3 枚举
      "bbox_pct": { "x": 10, "y": 8, "w": 80, "h": 30 }  // 重心矩形占屏百分比
    },
    "rhythm": {                              // 情绪节奏（自顶向下视觉路径）
      "beats": [
        { "name": "title_art",   "weight": 0.40, "y_pct": 8  },  // 视觉权重
        { "name": "menu_list",   "weight": 0.25, "y_pct": 50 },
        { "name": "status_line", "weight": 0.10, "y_pct": 96 }
      ],
      "rule": "art_to_action_top_down"       // 见 §4 节奏规则枚举
    },
    "info_density": {                        // 信息密度梯度
      "center_max_per_row": 5,               // 视觉重心区每行可读元素上限
      "edge_min_per_row": 1,                 // 边缘区每行最少元素（避免死寂）
      "dead_zone_max_pct": 15                // 死区占屏比上限（连续无内容）
    },
    "permanent_hud_count_max": 0             // 永久 HUD 元素数量上限（P-SoT-1）
  },
  "geometry_constraints": {
    "focus_bbox_must_contain": ["title_art_bbox_in_scenes_json"],  // scenes.json 矩形必须含重心 bbox
    "no_rect_overlap_with": ["focus_bbox"],  // 其他元素不得重叠重心区
    "edge_anchors": {                        // 边角锚定（HUD 元素必须贴边）
      "bottom_hint_y_min_pct": 92            // 提示行 y ≥ 92% 屏高
    }
  }
}
```

---

## 2. 情绪枚举（mood）

14 种场景情绪，每种**对应该场景的视觉重心类型 + 节奏规则**。门禁会断言 mood 与 focus.anchor / rhythm.rule 三元一致。

| mood ID | 触发场景类型 | 推荐 focus.anchor | 推荐 rhythm.rule |
|---|---|---|---|
| `epic_entry` | 主菜单 / 启动 | `title_art` | `art_to_action_top_down` |
| `hero_choice` | 角色选择 | `portrait_carousel` | `side_anchor_center_options` |
| `map_overview` | 地图选择 | `map_collage` | `side_anchor_center_options` |
| `world_explore` | 主地图 / 探索 | `world_canvas` | `world_first_hud_corner` |
| `tense_battle` | 战斗 | `battle_arena` | `arena_center_command_edge` |
| `reading` | 消息 / 帮助 | `text_panel` | `left_hero_right_text` |
| `archive` | 存读档 | `slot_list_with_meta` | `three_column_left_hero` |

> mood 是契约不是装饰：`epic_entry` 场景不允许把 menu_list 当 focus.anchor；
> `world_explore` 不允许把任何面板当 focus.anchor（focus 必须是 world_canvas）。

---

## 3. 视觉重心锚枚举（focus.anchor）

7 种重心锚类型，每种对应一种"该场景的视觉主体"。

| anchor ID | 含义 | bbox_pct 典型范围 | scenes.json 中匹配 |
|---|---|---|---|
| `title_art` | 标题艺术（巨字 + 装饰） | x:0-20, y:5-15, w:60-100, h:25-40 | 由 chrome 程序生成，scenes.json 不存坐标 |
| `portrait_carousel` | 角色立绘轮播 | x:0-30, y:20-40, w:30-100, h:40-60 | portrait_zones 子集 |
| `map_collage` | 地图拼贴艺术 | 同上 | portrait_zones 子集 |
| `world_canvas` | 游戏世界画布（棋盘/战场） | x:5-15, y:10-20, w:70-90, h:50-70 | grid + entity 锚 |
| `battle_arena` | 战斗场地 | x:5-15, y:10-30, w:70-90, h:40-60 | portrait_zones 战斗子集 |
| `text_panel` | 正文面板 | x:5-15, y:10-20, w:70-90, h:60-75 | body |
| `slot_list_with_meta` | 带元数据的槽列表 | x:5-15, y:5-15, w:70-90, h:70-85 | list |

### 3.1 视觉重心 bbox 约束（必断言）

1. **唯一性**：每场景 focus.anchor 数 = 1（不能同时是 title_art 和 menu_list）
2. **占屏比**：`bbox_pct.w * bbox_pct.h ≥ 20`（视觉重心必须占屏 ≥ 20%）
3. **scenes.json 覆盖**：focus.bbox 必须包含（≥）至少 1 个 scenes.json 主矩形（title/list/grid/body）
4. **不重叠例外**：focus.bbox 内允许的元素 = focus.anchor 本身 + 文字，**不允许其他永久 UI 元素进入**

---

## 4. 节奏规则枚举（rhythm.rule）

7 种节奏规则，对应"玩家眼睛从重心出发应该走的路径"。

| rule ID | 路径 | 适用 mood |
|---|---|---|
| `art_to_action_top_down` | 标题艺术 → 菜单 → 提示行（自顶向下） | `epic_entry` |
| `side_anchor_center_options` | 左立绘锚 → 中央选项 → 边角确认 | `hero_choice` / `map_overview` |
| `world_first_hud_corner` | 世界画布 → 边角 HUD → 浮现提示 | `world_explore` |
| `arena_center_command_edge` | 战斗场地 → 底部命令盘 → 边角状态 | `tense_battle` |
| `left_hero_right_text` | 左立绘 → 右正文 → 底翻页 | `reading` |
| `three_column_left_hero` | 左立绘 → 中列表 → 右元数据 | `archive` |

### 4.1 节奏权重约束（必断言）

1. **权重和 = 1.0**：`sum(beat.weight) == 1.0`（精度 0.01）
2. **beats[0] = 重心 beat**：`beats[0].name == focus.anchor`，且 `beats[0].weight ≥ 0.30`。
   beat 序 = **注意力优先级序**（不是 y_pct 位置序）：最该先看的排第一。
   `y_pct` 仅是位置标注，**不参与排序约束**（world_first_hud_corner 类节奏的
   HUD 角在视觉上是后注意的，但它 y_pct 更小，按位置单调会强制把它排首位 = 错误）
3. **beat 数量 2-4**：少于 2 = 没节奏，多于 4 = 视觉过载

---

## 5. 信息密度约束（info_density）

避免"死区"和"过载"两个极端。

### 5.1 死区（dead_zone）

死区 = 屏幕内连续 ≥ 200px 高度内无任何 scenes.json 矩形或 chrome 绘制元素。
`dead_zone_max_pct` 限定死区总占屏比上限（默认 15%）。

**当前 v2 vs 战斗问题**：`y=416..960 = 544px 空白 = 28% 屏高` → 违反死区约束。

### 5.2 信息密度上限（center_max_per_row）

视觉重心区（focus.bbox_pct）每行可读元素 ≤ `center_max_per_row`。
- `epic_entry` index 主菜单：重心是 title_art，无文字行 → `center_max_per_row = 0`
- `tense_battle` vs 战斗：重心是 battle_arena，每行 ≤ 3 元素（玩家/BOSS/特效）

### 5.3 边缘最低密度（edge_min_per_row）

边缘区（屏幕最上/最下 8% 或最左/最右 8%）每行至少 1 元素，避免死寂。
v2 的"vs 战斗 y=216..416 空白"就违反此约束。

---

## 6. 永久 HUD 约束（permanent_hud_count_max）

**P-SoT-1 借鉴**：永久 HUD 元素数量 ≤ 上限。

| mood | 上限 | 理由 |
|---|---|---|
| `epic_entry` | 0 | 主菜单不该有 HUD |
| `hero_choice` | 0 | 角色选择无 HUD |
| `world_explore` | 3 | main_map HUD = 金币 + 玩家位 + 据点（≤ 3） |
| `tense_battle` | 4 | 血条 + BOSS血 + 回合 + 命令盘（≤ 4） |
| `reading` | 1 | 翻页页码（≤ 1） |
| `archive` | 1 | 提示行（≤ 1） |

**断言**：扫描 scenes.json 全部永久键（非浮现的 title/list/hud/hp/hint/status）数量 ≤ 上限。

---

## 7. 边角锚定（edge_anchors）

永久 HUD 元素**必须贴边**（距屏边 ≤ 5% 屏宽/高），禁止漂浮在屏幕中部。

| 元素类型 | 必贴边 | 允许距边 |
|---|---|---|
| 血条 | 顶 或 底 | ≤ 5% |
| HUD 数字 | 顶 或 底 | ≤ 5% |
| 提示行 | 底 | ≤ 5% |
| 命令盘 | 底 | ≤ 5% |
| 立绘锚 | 左 或 右 | ≤ 15%（立绘允许部分贴边） |

**断言**：每个永久 HUD 元素 bbox 至少有一边贴屏边（≤ 5% 容差）。

---

## 8. scenes.json 扩展契约

scenes.json 每场景新增可选键（向后兼容 v2）：

```jsonc
"index": {
  "title": { "y": 8, "h": 96 },
  "list":  { ... },
  // ... v2 既有键 ...
  "focus_anchor": {                    // 新：重心锚 bbox（与 DSL focus.bbox_pct 一致）
    "type": "title_art",
    "bbox": { "x": 108, "y": 154, "w": 864, "h": 576 }   // 1080 坐标系实际像素
  },
  "ambient_art": {                     // 新：氛围艺术区（如背景立绘/拼贴）
    "type": "code_drawn_silhouette",   // chrome 程序绘制类型
    "bbox": { "x": 0, "y": 730, "w": 1080, "h": 800 }
  }
}
```

**约束**：
- `focus_anchor.bbox` 必须与 DSL `focus.bbox_pct` 换算后**完全一致**（容差 1px）
- `ambient_art` 是可选的氛围背景，**不计入 info_density.center_max_per_row**（因为不是可读元素）
- scenes.json 与 DSL 的对账由 `check_scene_narrative` 门禁逐项断言

---

## 9. 门禁实现要求（`check_scene_narrative`）

新门禁项，加入 `scripts/gate.ps1` 顶层固定 Report。

### 9.1 检查项

| # | 检查 | 失败行为 |
|---|---|---|
| 1 | 每场景在 `scene_manifest_v3.json` 有完整 narrative 块 | FAIL：缺场景 |
| 2 | mood ∈ 14 种枚举，且 focus.anchor / rhythm.rule 与 mood 推荐一致 | FAIL：三元不一致 |
| 3 | focus.bbox 占屏 ≥ 20%，且 ≥ 1 个 scenes.json 主矩形被包含 | FAIL：重心不成立 |
| 4 | rhythm beats 权重和 = 1.0，beats[0].name == focus.anchor 且 weight ≥ 0.30 | FAIL：节奏不合法 |
| 5 | info_density 死区 ≤ dead_zone_max_pct | FAIL：死区过大 |
| 6 | permanent_hud 数量 ≤ mood 上限 | FAIL：HUD 过载 |
| 7 | 永久 HUD 元素至少一边贴屏边（≤ 5%） | FAIL：HUD 漂浮 |
| 8 | scenes.json `focus_anchor.bbox` == DSL `focus.bbox_pct` × 1080/1920 | FAIL：DSL 与 scenes 漂移 |

### 9.2 实现位置

- 脚本：`scripts/check_scene_narrative.py`（Python，与既有 doccheck 同栈）
- 注册：`scripts/gate.ps1` 第 22 项（顶层固定 Report）
- Evidence：`evidence/check_scene_narrative.log`

---

## 10. 版本与迁移

- **v3.0** = 本规格 + 7 场景 manifest 落地 + 门禁接入
- **v3.1** = scenes.json 加 focus_anchor / ambient_art 键
- **v3.2** = chrome.rs 扩展原语支持新键
- **v3.3** = index 试点落地
- **v3.4** = 推广 6 场景
- **v3.5** = 旧 8pt 整除断言降级为"几何约束子集"（不再强制全场景）

**回退策略**：v3 任意阶段 FAIL 不阻塞 v2 已 verified 状态；v3.5 收口时 v2 §4 增量表整体退役。
