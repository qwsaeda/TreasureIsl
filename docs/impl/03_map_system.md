# 地图系统

本文档定义完整的地图系统：等距网格数据结构、移动算法、方向编码、可通行规则、据点检测、摄像机系统。

---

## 1. 地图数据

### 1.1 地图清单

| 槽位 | 文件 | 网格 | cell 记录数 | 连线记录数 | 缺格 |
|---|---|---|---|---|---|
| M01 | `glmapm01.Map` | 20×20 | 399 | 75 | cell(0, 19) |
| M02 | `glmapm02.Map` | 20×20 | 399 | 125（含 4 条 M=4） | cell(0, 19) |
| M03 | `glmapm03.Map` | 15×15 | 224 | 23 | cell(0, 14) |

每个地图的 cell 记录数 = `grid_w × grid_h - 1`（恒缺 1 格）。
缺格恒为 `cell(row=0, col=grid_w-1)`（索引 `grid_w-1`），不可通行。

### 1.2 地形文件结构

```
words[0] = 0
words[1] = count = grid_w - 1          // m01/02=19, m03=14
words[2..] = stride=9 u32 记录流
```

记录流分三段：
1. **cell 记录** `grid_w × grid_h - 1` 条
2. **trailer** 1 条
3. **连线记录** N 条 + 6 词 sentinel

### 1.3 cell 记录字段

每条 cell 记录 9 个 u32：`[f0, f1, f2, f3, f4, f5, f6, f7, f8]`

| 字段 | 值 | 语义 |
|---|---|---|
| f0 | 1（使能标志） | 恒 1 |
| f1 | 瓦片索引 / 目标格 | 见连线段 |
| f2 | 0 | 恒 0 |
| f3 | 0 | 恒 0 |
| f4 | 1 | 恒 1 |
| f5 | mapinfo.ini 瓦片索引 | m01 max=44, m03 max=44 |
| f6 | `32 × (row + col)` | 等距投影 X 坐标 |
| f7 | `16 × (row + grid_w - 1 - col)` | 等距投影 Y 坐标 |
| f8 低 16 位 | `row × grid_w + col` | cell 线性索引 |
| f8 高 16 位 | 标志位 | 低字节 `≡ 1 mod 4` = 可通行（**推测**，F-307 仅验证低 16 位为 cell 索引） |

### 1.4 等距投影

```
iso_x = 32 × (row + col)
iso_y = 16 × (row + grid_w - 1 - col)
```

- row 增大 = iso 屏幕下移 = **南**
- col 增大 = iso 屏幕右移 = **东**

### 1.5 连线记录

每条 `[0, 0, 0, 0, 0, M, A, B, C]`：

| 字段 | 语义 |
|---|---|
| M | 连接类型，`M ∈ {2, 4}`（m02 独有 4 条 M=4） |
| A | 瓦片索引（< 49） |
| B | 等距像素 X 坐标 |
| C | 等距像素 Y 坐标（非降序排列 = painter's 绘制序） |

连线目标格满足：`|Δcol| ≤ 1` 且 `Δrow ≤ 0`（同列或邻列，绝不向下）。
类别码 1..=9 全覆盖（9 种路线类别）。

文件尾固定 6 词 sentinel `[0, 0, 0, 0, 0, 2]`。

---

## 2. 数据结构定义

```rust
// crates/ti-formats/src/mapfile.rs

#[derive(Debug, Clone)]
pub struct MapData {
    pub grid_w: u32,
    pub grid_h: u32,
    pub cells: Vec<CellRecord>,
    pub links: Vec<LinkRecord>,
    pub landmarks: Vec<Landmark>,
}

#[derive(Debug, Clone)]
pub struct CellRecord {
    pub f0: u32,        // 使能标志，恒 1
    pub f1: u32,        // 瓦片索引 / 目标格
    pub f2: u32,        // 恒 0
    pub f3: u32,        // 恒 0
    pub f4: u32,        // 恒 1
    pub f5: u32,        // mapinfo.ini 瓦片索引
    pub iso_x: i32,     // 32 × (row + col)
    pub iso_y: i32,     // 16 × (row + grid_w - 1 - col)
    pub cell_idx: u32,  // row × grid_w + col
    pub flags: u32,     // 高 16 位标志
}

#[derive(Debug, Clone)]
pub struct LinkRecord {
    pub link_type: u32, // M ∈ {2, 4}
    pub tile_idx: u32,  // A
    pub iso_x: i32,     // B
    pub iso_y: i32,     // C
}

#[derive(Debug, Clone)]
pub struct Landmark {
    pub row: u32,
    pub col: u32,
    pub name: String,
    pub resource_value: u32,
}
```

---

## 3. 移动规则

### 3.1 方向枚举

| 方向码 | 枚举 | Δ(row, col) | 探测优先级 | 方向位 |
|---|---|---|---|---|
| 0 | North（北） | (-1, 0) | 4（兜底） | 8 |
| 1 | West（西） | (0, -1) | 3 | 4 |
| 2 | South（南） | (+1, 0) | 1（优先） | 2 |
| 3 | East（东） | (0, +1) | 2 | 1 |

方向位掩码编码：1=西/2=南/4=东/8=北。

### 3.2 可通行性

```
可通行 = (cell.flags & 3) == 1
```

越界格子一律视为不可通行。

### 3.3 单步移动

```rust
pub fn oracle_move(
    grid_w: u32,
    grid_h: u32,
    blocked: &[bool],
    pos: GridPos,
    dir: Direction,
) -> Option<GridPos> {
    let (dr, dc) = dir.delta();
    let new_row = pos.row as i32 + dr;
    let new_col = pos.col as i32 + dc;

    if new_row < 0 || new_col < 0 {
        return None;
    }
    if new_row >= grid_h as i32 || new_col >= grid_w as i32 {
        return None;
    }

    let idx = (new_row as u32) * grid_w + (new_col as u32);
    if blocked[idx as usize] {
        return None;
    }

    Some(GridPos {
        row: new_row as u32,
        col: new_col as u32,
    })
}
```

### 3.4 走 N 步

```rust
pub fn move_n_steps(
    grid_w: u32,
    grid_h: u32,
    blocked: &[bool],
    pos: GridPos,
    dir: Direction,
    steps: u32,
) -> (GridPos, u32) {
    let mut cur = pos;
    let mut walked = 0;

    for _ in 0..steps {
        match oracle_move(grid_w, grid_h, blocked, cur, dir) {
            Some(next) => {
                cur = next;
                walked += 1;
            }
            None => break,
        }
    }

    (cur, walked)
}
```

返回 `(终点, 实际走的步数)`。

### 3.5 自动方向选择（AI 漫游）

```rust
pub fn oracle_pick_dir(
    grid_w: u32,
    grid_h: u32,
    blocked: &[bool],
    pos: GridPos,
) -> Option<Direction> {
    if blocked[pos.index(grid_w) as usize] {
        return None;
    }

    for dir in Direction::priority_order() {
        if oracle_move(grid_w, grid_h, blocked, pos, dir).is_some() {
            return Some(dir);
        }
    }

    None
}
```

### 3.6 随机漫游分支

方向位全 0 且回落仍 0 时（所有方向都不可通行）：

```
rand() % 4   // 随机选 4 方向之一，绕开已屏蔽位
```

### 3.7 单位初始定位

```
row = cell_index % grid_w
col = cell_index / grid_w
```

cell 索引 = `row × grid_w + col` 的逆运算。

---

## 4. 据点/地标

### 4.1 据点判定

```rust
pub fn oracle_at_landmark(landmarks: &[Landmark], pos: GridPos) -> Option<&Landmark> {
    landmarks.iter().find(|lm| lm.row == pos.row && lm.col == pos.col)
}
```

### 4.2 据点资源结算

据点命中时：

1. 遍历精灵匹配场景 id。
2. 命中 → 累加资源值（上限 999 = `0x3E7`）。
3. 未命中 → 扣减（贸易/建设消耗）。
4. 资源值为 0 且计数 > 1 → 补 1（保底机制）。

### 4.3 据点触发概率

```
命中 = (rand() % 100 + 1) > 40         // 40% 命中率（来源 docs/design/map_system.md §3.3；exe 实测 `trade` 行 154）
```

### 4.4 据点显示

命中据点 → 装载 `ui.pak + msgl.Map` 显示据点名：

```
swprintf(buffer, "%s：%s", 标题, 据点名)
```

---

## 5. 摄像机系统

### 5.1 摄像机结构

```rust
pub struct Camera {
    pub x: i32,      // 等距像素 X
    pub y: i32,      // 等距像素 Y
    pub target_x: i32,
    pub target_y: i32,
    pub viewport_w: u32,
    pub viewport_h: u32,
}
```

### 5.2 跟随逻辑

```rust
pub fn camera_follow(&mut self, unit_x: i32, unit_y: i32) {
    self.target_x = unit_x - (self.viewport_w / 2) as i32;
    self.target_y = unit_y - (self.viewport_h / 2) as i32;
    // 平滑插值
    self.x += (self.target_x - self.x) / 4;
    self.y += (self.target_y - self.y) / 4;
}
```

### 5.3 边界钳制

```rust
pub fn camera_clamp(&mut self, map_w: i32, map_h: i32) {
    self.x = self.x.clamp(0, map_w - self.viewport_w as i32);
    self.y = self.y.clamp(0, map_h - self.viewport_h as i32);
}
```

---

## 6. 渲染管线

### 6.1 绘制顺序

1. 清屏
2. 绘制地形瓦片（painter's 算法，按 iso_y 非降序）
3. 绘制连线（道路/路线）
4. 绘制据点标记
5. 绘制单位精灵
6. 绘制 UI 覆盖层

### 6.2 瓦片绘制

```rust
pub fn draw_tile(&self, tile_idx: u32, iso_x: i32, iso_y: i32) {
    let screen_x = iso_x - self.cam.x;
    let screen_y = iso_y - self.cam.y;
    self.atlas.draw(tile_idx, screen_x, screen_y);
}
```

---

## 7. 完整移动示例

```
初始：pos = (5, 5), grid_w = 20, grid_h = 20

1. 向北走 3 步：
   step1: (5,5) → (4,5)  ✓
   step2: (4,5) → (3,5)  ✓
   step3: (3,5) → (2,5)  ✓
   结果：pos = (2, 5), walked = 3

2. 向东走 5 步：
   step1: (2,5) → (2,6)  ✓
   step2: (2,6) → (2,7)  ✓
   step3: (2,7) → (2,8)  ✓
   step4: (2,8) → (2,9)  ✓
   step5: (2,9) → (2,10) ✓
   结果：pos = (2, 10), walked = 5

3. 向南走 10 步（撞墙）：
   step1..10: (2,10) → (3,10) → ... → (11,10) 全部可通行
   结果：pos = (11, 10), walked = 10

4. 向西走 20 步（撞墙）：
   step1..10: (11,10) → (11,9) → ... → (11,0) 全部可通行
   step11: (11,0) → (11,-1) ✗ 越界
   结果：pos = (11, 0), walked = 10
```

---

## 8. 实现检查清单

- [ ] 地图文件解析正确
- [ ] 等距投影计算正确
- [ ] 移动算法边界处理
- [ ] 可通行性判定正确
- [ ] 据点检测与结算
- [ ] 摄像机跟随与钳制
- [ ] 渲染顺序正确
