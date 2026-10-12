# S3.4 selectrole/selectmap — 逆向规格

数据源：`ref/pseudocode/*.c`（IDA Hex-Rays）、`ref/functions_selectrole.csv`、`ref/functions_selectmap.csv`、`docs/spec/main_loop.md` §4、`docs/spec/menu.md` §a/§c。
结论标注：`证据` = 指到具体伪代码文件 + 行号；`推测` = 需进一步验证。

---

## §a selectrole.Map 场景结构

### 装载器（证据）

`sub_3ECDC`（`0003ECDC_sub_3ECDC.c`，size 616）：

| 行 | 操作 | 说明 |
|---|---|---|
| 32 | `sub_37EDC(v15, v13, aUiPak)` | 从 `UI.pak` 索引取控件表 |
| 36 | `sub_37EDC(v16, v13, aSelectroleMap)` | 取 `selectrole.Map` 场景记录 |
| 40-42 | `operator new(0x3FC)` + `sub_43B8C(v7, a1[2])` | 场景对象 ctor（`*a1=off_6F518`，见 `main_loop.md` §4.2） |
| 45 | `a1[12] = v8` | 场景对象指针存入上下文槽位 |
| 48 | `sub_43D54(v8, *(a1[1]+16))` | 关联 UI 句柄 |
| 49-58 | `sub_132FC` × 2 + 虚调用 `(**(scene)+4)(...)` | 装载资源/控件表（7 参数：场景 + ui.pak 记录 + selectrole.Map 记录） |
| 59-60 | `sub_37EDC(v17, v13, aGlpicsPak)` + `sub_161B0(v12, v11)` | 加载 `glpics.pak` 精灵图容器 |
| 62 | `sub_3FE84(a1, v12[0], a1+8)` | **控件注入器调用**（见下），返回 1 表示成功 |
| 64-69 | `byte_81194..97` 全非零 → `a1[13]=5`、`a1[14]=58`、`a1[15]=75`、`a1[16]=5` | **推测**：场景参数写入（5 角色 + 坐标/尺寸参数），四槽位非零 = gl.rec 已有效（`sub_40D80` 行 69-87 读 `data/gl.rec` 头 4 字节） |
| 71-82 | 清理临时宽字符串 + `return 1`（成功）/ `return 0`（失败） | |

### 控件注入器（证据）

`sub_3FE84`（`0003FE84_sub_3FE84.c`，size 876）：

- 行 61-84：注入 **11 个资源**（`sub_12F04` + `sub_1318C` 逐个构造宽字符串）：
  - `Target_knife.pcx`（行 63）
  - `face_katharina.pcx` / `face_drake.pcx` / `face_kidd.pcx` / `face_tichy.pcx` / `face_panda.pcx`（行 65-74）
  - `Player_katharina.pcx` / `Player_drake.pcx` / `Player_kidd.pcx` / `Player_tichy.pcx` / `Player_panda.pcx`（行 75-84）
- 行 85：`sub_1DDA4(a1+32, 0xBu)` — 预分配 **11 个控件槽位**（0xB = 11）
- 行 88-117：逐个 `sub_1E5CC` 取资源 → `sub_63BE8` 查表 → `operator new` → `sub_645B4` 写入控件对象

> **角色映射**（`evidence/S3.2.log` 字符串清单 行 15/18/20）：
> - panda → `start_panda.pcx` + `face_panda.pcx` + `Player_panda.pcx`
> - tichy → `start_tichy.pcx` + `face_tichy.pcx` + `Player_tichy.pcx`
> - kidd → `start_kidd.pcx` + `face_kidd.pcx` + `Player_kidd.pcx`
> - drake → `start_drake.pcx` + `face_drake.pcx` + `Player_drake.pcx`
> - katharina → `start_katharina.pcx` + `face_katharina.pcx` + `Player_katharina.pcx`
>
> 控件注入器注入的是 `face_*` + `Player_*`（10 个）+ `Target_knife.pcx`（1 个）= 11 个控件槽位。
> `start_*.pcx` 是角色**立绘大图**，在 selectrole 场景的渲染阶段由场景帧更新函数（`sub_29110` 的 `a1+576` 虚调用 `+16`）使用，不在控件注入器内。

### 事件码（证据 + 推测）

selectrole.Map 场景的 UI 事件由场景对象的 `+16` 虚函数（帧更新/事件处理）分派。
`sub_29110`（`00029110_sub_29110.c`）是场景管理器总控，`switch(*(a1+716))` 分派到各场景。

| 事件码 | 含义 | 状态 | 证据/推测 |
|---|---|---|---|
| `13` | 确定/Enter（选择角色） | 真实 | `sub_41354` 行 100（index 主菜单共用事件码体系，selectrole 复用同一事件分发框架） |
| `27` | 返回（回 index.Map） | 真实 | `sub_3F4E4` 行 112（`a3==27 → sub_3DC04(v3[1], 6)` 返回场景） |
| `37` | 无效操作 | **推测** | `sub_3F4E4` 行 118 / `sub_4085C` 行 109（`a3==37` 分支 → 无效音效） |
| `38` | 上一页/上一个角色 | 真实 | `sub_4085C` 行 52（`a3==38` 翻页） |
| `40` | 下一页/下一个角色 | 真实 | `sub_4085C` 行 37（`a3==40` 翻页） |

> **推测**：selectrole.Map 场景本身的事件处理器伪代码未单独导出（`sub_3ECDC` 只装载，事件分派由场景对象 `+16` 虚函数在运行时注册，IDA 未展开为独立函数）。事件码体系与 index.Map 主菜单（`sub_41354`）共用，`13/27/37/38/40` 为框架统一事件码。

## §b selectmap.Map 场景结构

### 装载器（证据）

`sub_402D8`（`000402D8_sub_402D8.c`，size 1028）：

| 行 | 操作 | 说明 |
|---|---|---|
| 44 | `sub_37EDC(v26, v24, aUiPak)` | 从 `UI.pak` 索引取控件表 |
| 48 | `sub_37EDC(v27, v24, aSelectmapMap)` | 取 `selectmap.Map` 场景记录 |
| 52-54 | `operator new(0x3FC)` + `sub_43B8C(v7, a1[2])` | 场景对象 ctor |
| 57 | `a1[12] = v8` | 场景对象指针存入上下文 |
| 60 | `sub_43D54(v8, *(a1[1]+16))` | 关联 UI 句柄 |
| 63-70 | 虚调用 `(**(scene)+4)(...)` | 装载资源/控件表 |
| 71 | `Buffer = 0` | 探测缓冲区清零 |
| 72-74 | `sub_37EDC(v28, v24, aGlmapm01Map)` | 取 `glmapm01.Map` 记录（从 `ui.pak` 索引） |
| 76-78 | `CreateFileW(lpFileName[0], GENERIC_READ, ...)` + `ReadFile(&Buffer, 4)` | **探测 `data/glmapm01.Map` 文件是否存在**（直读文件系统，非 pak 索引） |
| 79-87 | 存在 → `sub_406DC(a1+13, &1)` / `sub_406DC(a1+16, &73)` / `sub_406DC(a1+19, &100)` | **写入 3 个槽位**：`a1[13]=1`（地图编号）、`a1[14]=73`（X 坐标）、`a1[15]=100`（Y 坐标）— 推测为地图 1 的 UI 布局参数 |
| 89-100 | 探测 `glmapm02.Map` → 存在 → `sub_406DC(a1+13, &2)` / `sub_406DC(a1+16, &67)` / `sub_406DC(a1+19, &64)` | 地图 2 的槽位参数 |
| 103-115 | 探测 `glmapm03.Map` → 存在 → `sub_406DC(a1+13, &3)` / `sub_406DC(a1+16, &117)` / `sub_406DC(a1+19, &81)` | 地图 3 的槽位参数 |
| 117-120 | 清理 + `return 1` | 成功 |
| 122-128 | `return 0` | 失败（场景对象创建失败） |

### 3 槽位写入（`sub_406DC` 语义，证据）

`sub_406DC`（`000406DC_sub_406DC.c`，size 96）= **动态数组 append**：

```
sub_406DC(int *a1, _DWORD *a2):
  v4 = a1[1];                    // end 指针
  if (v4 == a1[2])              // end == capacity → 扩容
    return sub_3854C(a1, v4, a2, &v6, 1, 1);
  *v4 = *a2;                    // 写入值
  a1[1] = v4 + 4;               // end 前移 4 字节
```

`sub_402D8` 中 `a1+13`/`a1+16`/`a1+19` 是 3 个独立的动态数组槽位（`a1[13]` = 地图编号数组、`a1[14]` = X 坐标数组、`a1[15]` = Y 坐标数组），每次探测成功时各 append 一个 u32。

> **推测**：3 个槽位对应 selectmap 场景中 3 个地图按钮的位置参数（编号 / X / Y）。`a1[13]` 的值 1/2/3 为地图编号，`a1[14]` 的 73/67/117 为 X 坐标，`a1[15]` 的 100/64/81 为 Y 坐标。具体 UI 布局需对照 `selectmap.Map` 控件表确认。

### 已解锁状态（证据 + 推测）

`sub_402D8` 行 76-115：`CreateFileW` 直读 `data/glmapm01.Map`/`glmapm02.Map`/`glmapm03.Map` 文件存在性。
- **存在** → 该地图**已解锁**，append 到 3 个槽位数组；
- **不存在** → 该地图**未解锁**，不 append（槽位数 = 已解锁地图数）。

> **推测**：地图解锁由游戏进度控制（`data/glmapm0x.Map` 文件由存档系统 `sub_30FDC` 写出或预置）。新游戏时可能只有 `glmapm01.Map` 存在，后续关卡解锁 `glmapm02`/`glmapm03`。需对照 S3.5/S3.10 确认解锁时机。

## §c 选择完成 → main.Map 触发

### 触发路径（证据）

1. **selectrole 完成**：玩家在 selectrole.Map 场景选择角色（事件码 `13` 确定）→ 场景对象 `+16` 虚函数处理选择 → **推测** 设置全局角色标识（`dword_7FB6C` 或类似全局变量）→ `sub_3DC04(ctx, N)` 切换场景码。

2. **selectmap 完成**：玩家在 selectmap.Map 场景选择地图（事件码 `13` 确定）→ 场景对象 `+16` 虚函数处理选择 → 设置 `dword_7FB6C`（地图编号 1/2/3）→ `sub_3DC04(ctx, N)` 切换场景码。

3. **main.Map 装载**（`sub_2077C`，`0002077C_sub_2077C.c`）：
   - 行 142-155：`switch (dword_7FB6C)`：
     - `case 1` → `aDataGlmapM01Pa`（`data\glmapm01.Map`）+ `aDataGlmapm01Ma`（`glmapm01.Map`）
     - `case 2` → `aDataGlmapM02Pa` + `aDataGlmapm02Ma`
     - `case 3` → `aDataGlmapM01Pa` + `aDataGlmapm03Ma`
   - 行 165：`sub_2FC44(a1, glpics.pak 路径)` + `sub_2EA98(a1, glmapm0x.Map 路径)` — 双重校验（glpics.pak 可读 + 地图文件可读）
   - 行 167-176：`operator new(0x3FC)` + `sub_43B8C` + `sub_43D54` + 虚调用 `+4` 装载 `main.Map` + `ui.pak`
   - 行 187：`sub_334BC(a1+72)` — 主地图控件表注入

> **证据**：`sub_2077C` 行 142-155 的 `switch(dword_7FB6C)` 直接读取全局地图编号变量，case 1/2/3 对应 `glmapm01/02/03.Map`。`sub_402D8` 写入的 `a1[13]` 槽位值 1/2/3 即此编号。

### 场景码切换（推测）

`sub_3DC04`（`0003DC04_sub_3DC04.c`）：

```
sub_3DC04(ctx, scene_code):
  v5 = sub_3DDF4();            // 取场景管理器单例
  if (result != 0) {
    if (虚调用+4(result, ctx, *(ctx+4)) != 0) {   // 场景管理器接受切换
      销毁旧场景（+8 虚函数 + vtable[0] 传 1）
      *ctx = scene_code;       // 写入新场景码
      *(ctx+8) = v5;
      result = 0;
    }
  }
```

| 场景码 | 目标场景 | 调用点 | 证据/推测 |
|---|---|---|---|
| `4` | 回主地图/探索 | `sub_4220C` 行 79 | 推测：selectrole/selectmap 完成后的跳转码 |
| `6` | 回 index.Map 主菜单 | `sub_3F4E4` 行 115 / `sub_4085C` 行 102 / `sub_428F0` 行 12 | 证据：子场景返回 |
| `8` | 新游戏（selectrole） | `sub_41354` 行 106 | 证据：index 主菜单 case 0 |
| `10` | help.map 返回 | `sub_3F4E4` 行 42 | 证据 |
| `12` | 场景 12 | `sub_4085C` 行 138 | 证据 |

> **推测**：selectrole 选择完成后场景码设为 `4`（主地图探索），由 `sub_29110` 的 `case 4`（行 843）触发 `sub_2077C` 装载 main.Map + `switch(dword_7FB6C)` 选择具体地图。

## §d 事件码汇总（真实 + 推测）

| 事件码 | 含义 | 状态 | 来源 |
|---|---|---|---|
| `13` | 确定/Enter（选择角色/地图） | 真实 | `sub_41354` 行 100 / `sub_3F4E4` 行 75 / `sub_4085C` 行 62 |
| `27` | 返回（回 index.Map） | 真实 | `sub_3F4E4` 行 112 / `sub_4085C` 行 100 |
| `37` | 无效操作 | **推测** | `sub_3F4E4` 行 118 / `sub_4085C` 行 109 |
| `38` | 上一页/上一个角色 | 真实 | `sub_4085C` 行 52 / `sub_3F4E4` 行 127 |
| `40` | 下一页/下一个角色 | 真实 | `sub_4085C` 行 37 / `sub_3F4E4` 行 59 |

> **推测**：selectrole 场景的 `38/40` 用于在 5 个角色间循环切换（panda→tichy→kidd→drake→katharina→panda），`13` 确认选择。selectmap 场景的 `38/40` 用于在已解锁地图间切换，`13` 确认选择。

## §e 函数归属表

完整归属见 `ref/functions_selectrole.csv` + `ref/functions_selectmap.csv`（不改动 `ref/functions.csv` 原文件）。

| 函数 | 地址 | 归属 | 理由 |
|---|---|---|---|
| `sub_3ECDC` | 0x3ECDC | selectrole | selectrole.Map 场景装载器（§a） |
| `sub_3FE84` | 0x3FE84 | selectrole | selectrole 控件注入器（11 控件槽位，§a） |
| `sub_402D8` | 0x402D8 | selectmap | selectmap.Map 装载器 + glmapm 探测（§b） |
| `sub_406DC` | 0x406DC | selectmap | 动态数组 append（3 槽位写入工具，§b） |
| `sub_2077C` | 0x2077C | selectmap→main | main.Map 装载器（§c，`switch(dword_7FB6C)` 选地图） |
| `sub_334BC` | 0x334BC | main | 主地图控件表注入总控（§c） |
| `sub_44978` | 0x44978 | main_loop | 场景对象 vtable 初始化（`sub_43B8C` 行 10 调用） |
| `sub_29110` | 0x29110 | main_loop | 场景管理器帧更新/切换总控（§c 场景码分派） |

## 推测点汇总

1. selectrole 场景的独立事件处理器伪代码未导出（场景 `+16` 虚函数在运行时注册，IDA 未展开为独立函数）— 事件码体系由 `sub_41354`/`sub_3F4E4`/`sub_4085C` 共用框架推断。
2. `sub_402D8` 写入的 3 槽位参数（73/100、67/64、117/81）的 UI 布局语义（X/Y 坐标）— 需对照 `selectmap.Map` 控件表确认。
3. 地图解锁时机（`data/glmapm0x.Map` 由谁写出）— 需对照 S3.5/S3.10 确认。
4. selectrole 完成后的场景码（推测 `4`）— 需 QEMU-CE 断点或 `sub_29110` case 4 上下文验证。
5. `sub_402D8` 行 71 `Buffer=0` 后 `ReadFile` 的 4 字节头值是否参与判断（仅 `ReadFile` 返回值 = 存在性）— 推测仅用返回值，Buffer 值未读。
