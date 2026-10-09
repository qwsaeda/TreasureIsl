# S3.2 资源/渲染核心 spec

来源：`ref/pseudocode/*.c`（IDA Hex-Rays）、`ref/functions_render.csv`（8 个函数）、`evidence/S3.2.log`。
实现：`crates/ti-core/src/oracle/render.rs`（直译）+ `rules/render.rs`（正式，结构体方法）。
差分：`crates/ti-core/tests/diff_render.rs`（8 项）。

---

## a) 资源容器与装载（`sub_37EDC` 通用装载）

### 容器清单（5 个，`evidence/S3.2.log` §3）

| 枚举 | 磁盘名 | 内容 |
|---|---|---|
| `Ui` | `UI.pak` | 场景 `.Map` + 控件表 |
| `Glpics` | `glpics.pak` | 精灵 PCX（`face_*` / `troop` / `ship` / `flag` / `landmark*`） |
| `Cards` | `cards.lpk` | 卡牌数据（`cards.ini` / `event_00N`） |
| `Goods` | `goods.lpk` | 商品数据（`goods.mdu` / `goods_0N`） |
| `Music` | `music.lpk` | 音频（`victory.wav` / `glmusic.mod` 等） |

### 场景装载器族（`ref/functions_render.csv`）

| 函数 | 地址 | 场景 |
|---|---|---|
| `sub_42574` | 0x42574 | `intro.Map`：`new 0x3FC` + `sub_43B8C` + `sub_37EDC(buf, ui, "UI.pak")` + `sub_37EDC(buf2, ui, "intro.Map")` |
| `sub_3ECDC` | 0x3ECDC | `selectrole.Map`：`new 0x3FC` + `sub_43B8C` + `sub_37EDC(..., "selectrole.Map")` |
| `sub_402D8` | 0x402D8 | `selectmap.Map` + **`glmapm01/02/03.Map` 存在性探测**（`CreateFileW`，判定位在 `sub_406DC` 写回处） |
| `sub_40D80` | 0x40D80 | `index.Map` + `glpics.pak` + **`data/gl.rec` 魔数检查（`0x01315A47`）** |
| `sub_2077C` | 0x2077C | `main.Map` + 地形探测（格坐标 `aDataMainMap` 寻址） |
| `sub_37EDC` | 0x37EDC | **通用装载器**：按容器 + 资源名从索引取 PCX/控件表 → 装入场景 |
| `sub_43B8C` | 0x43B8C | 场景 ctor：`*a1 = off_6F518`；构建 vtable，置 `[253] = 1` 置位标志 |
| `sub_43D54` | 0x43D54 | 场景析构：UI 场景每控件释放（调 `sub_43B8C` 链） |

### 装载语义（`oracle_load_scene` ↔ `SceneManager::load_scene`）

对 `refs` 逐条判定，**任一条失败即整体 `Err`，列出全部 `missing`**：

1. **容器存在性**：`ref.container ∈ available_containers`，否则 missing；
2. **gl.rec 魔数**：仅 `Glpics` 容器要求 `gl_rec_magic == 0x01315A47`，否则该容器全部 missing；
3. **glmapm 探测**：仅 `Ui` 容器中形如 `glmapm<NN>.Map` 的资源，要求 `NN ∈ glmapm_indices`（已解锁地图编号），否则 missing；
4. **扩展名映射**：`pcx → Pcx` / `mod → Mod` / `sav → Sav` / 其他（含 `.Map`）→ `Map`。

**事务性**：`Ok` 时才写入内部 `scenes`；`Err` 时 `scenes` 保持调用前不变（见 `impl_scenes_unchanged_on_error`）。

---

## b) 渲染：图层顺序 = 装载顺序

### 关键证据 + oracle 实现

- WinCE GX API（`GXBeginDraw`/`GXEndDraw`/`GXOpenDisplay`，见 F-103 导入表）。
- 场景对象 vtable `+16` = 绘制入口；**不排序、不分组**。
- `oracle_render_frame(assets)` / `SceneManager::render_frame(assets)`：
  `RenderPlan { layers: assets.clone(), blit_count: assets.len() }`。

> ⚠️ 本模块当前**只建模「装载 → 图层列表 → blit 次数」**，像素级合成（P4-1）由 `ti-platform/gfx` 承担，
> 见 [`../../parity.md`](../../parity.md) P4-1。

---

## c) 存档/记录容器（S3.2 触及部分）

- `gl0.sav` .. `gl4.sav`：5 个角色存档。
- `gl.rec`：记录文件，魔数 `0x01315A47`；字节位 `0x81194..0x81197` = 4 个全局状态位（字段语义属 S3.10，见 [`save_load.md`](save_load.md)）。
- `settings.dat`：8 字节，F-304。

---

## d) 事件映射（不涉及）

本模块无事件分派；场景切换事件属 S3.1（[`main_loop.md`](main_loop.md) §4），卡牌事件属 S3.9。

---

## e) 缺口与推测

| 项 | 说明 |
|---|---|
| glmapm 探测的**具体文件系统 API 调用序列** | 仅确认用 `CreateFileW` 判定存在（`sub_406DC`），未逆出失败分支的完整清理路径。**推测**：失败即跳过该地图槽位 |
| vtable `[253] = 1` 置位标志的语义 | `sub_43B8C` 写入，猜测为「已初始化」标志位；**推测**，无读取点交叉验证 |
| `sub_2077C` 的 `aDataMainMap` 寻址 | 地形格坐标计算细节属 S3.5（[`main_map.md`](main_map.md)），本模块只取「存在性」 |
| P4-1 逐像素一致 | **未验证**：`render_parity` 判据在 [`../../parity.md`](../../parity.md) P4-1，当前无对应测试 |

---

## 已知问题清单

1. `define_scene!` 尚未落地（`docs/style.md` §5.4 规划中）：8 个场景装载器仍为手写样板，S3.1 之后应改为宏注册。
2. 像素级渲染（P4-1）缺测试，DoD 只覆盖到「图层顺序 + blit 次数」。
3. `sub_402D8` 的 `glmapm01/02/03` 探测只建模为「编号 ∈ 已解锁集合」，未建模文件系统错误码。