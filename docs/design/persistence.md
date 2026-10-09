# 持久化与随机系统设计

本文档涵盖：存读档（settings.dat / gl*.sav / gl.rec，文件布局/魔数/迁移）+ RNG（LCG 算法/播种/调用点）。

---

## 1. 存读档系统

### 1.1 文件清单

| 文件 | 大小 | 作用 |
|---|---|---|
| `settings.dat` | 8 字节 | 全局设置（音量/画质开关） |
| `gl0.sav` ~ `gl4.sav` | 可变 | 5 个角色存档槽 |
| `gl.rec` | 10 字节 | 全局进度记录 |

### 1.2 统一魔数

所有存档文件共用魔数 `0x01315A47`（= 19,810,929）。

文件布局统一为：`[2B 前缀][4B 魔数(u32 LE)][负载]`。

读取时跳过前 2 字节（`SetFilePointer(2, FILE_BEGIN)`），然后读 4 字节 u32 校验魔数。

### 1.3 settings.dat 布局

| 偏移 | 字节数 | 字段 | 实测值 |
|---|---|---|---|
| 0 | 2 | 前缀（跳过） | `5D 4A` |
| 2 | 4 | 魔数（u32 LE） | `0x01315A47` |
| 6 | 1 | 设置 1（音量/画质） | `0x01` |
| 7 | 1 | 设置 2 | `0x01` |

最小有效大小：8 字节。

读取：`sub_3EB24`（行 19-26）—— 跳 2 字节 + 读 4 字节魔数校验 + 读 2 字节设置。
写入：`sub_30FDC`（行 19-23）—— 写魔数 + 2 字节设置。

### 1.4 gl.rec 布局

| 偏移 | 字节数 | 字段 | 语义 |
|---|---|---|---|
| 0 | 2 | 前缀（跳过） | — |
| 2 | 4 | 魔数 `0x01315A47` | — |
| 6 | 1 | `byte_81194` | 全局状态 1 |
| 7 | 1 | `byte_81195` | 全局状态 2 |
| 8 | 1 | `byte_81196` | 全局状态 3 |
| 9 | 1 | `byte_81197` | 全局状态 4 |

最小有效大小：10 字节。

4 个状态位全非零 → 5 角色初始化完成（`sub_41354` 行 64-69 → `a1[13] = 5`）。

### 1.5 gl*.sav 存档结构

#### 原版（v1）可变负载

| 偏移 | 字节数 | 字段 |
|---|---|---|
| 0 | 2 | 前缀（跳过） |
| 2 | 4 | 魔数 `0x01315A47` |
| 6 | 4 | 角色/地图标识 |
| 10 | 4 | 当前角色索引（1..5） |
| 14 | 4 | 当前地图索引 |
| 18 | 可变 | 场景状态（虚调用 `sub_44` 偏移 40） |
| ? | 4 | 游戏进度（`a1+724`） |
| ? | 可变 | 据点状态（虚调用 `sub_44` 偏移 44） |
| ? | 可变 | 资源集（虚调用 `sub_44` 偏移 40） |
| ? | 4 | 卡牌状态 1（`sub_2FA58(a1+532, 1)`） |
| ? | 4 | 卡牌状态 2（`sub_2FA58(a1+532, 1) + 4`） |
| ? | 4 | 持有物品数（`a1+500`） |
| ? | N×4 | 物品 id 列表（遍历 `a1+496` 链表） |
| ? | 可变 | 卡牌对象（虚调用 + 2×u32） |
| ? | 可变 | 据点对象（遍历 `a1+520`） |
| ? | 可变 | 兵种对象（遍历 `a1+508`） |
| ? | 可变 | 货物状态（`sub_34588(a1+608)`） |
| ? | 6 | 固定区（`a1+544`） |
| ? | N×2 | 卡牌持有量（遍历 `a1+552`） |
| ? | N×2 | 卡牌持有量 2（遍历 `a1+564`） |

v1 最小有效大小：18 字节。

#### 现代化（v2）固定布局

```
offset  0:  2B  前缀（跳过）
offset  2:  4B  魔数（u32 LE = 0x01315A47）
offset  6:  4B  version（u32 LE，1 或 2）
offset 10:  4B  character（u32 LE，角色索引 1..5）
offset 14:  4B  gold（u32 LE，当前金）
offset 18:  4B  map_slot（u32 LE，地图索引 0..4）
offset 22:  4B  position（u32 LE，游戏进度）
```

v2 最小有效大小：`2 + 4 + 4×5 = 26` 字节。

```
struct SaveSlot {
    version: u32,
    character: u32,
    gold: u32,
    map_slot: u32,
    position: u32,
}
```

### 1.6 槽位选择

```
switch (slot_index):
    case 1: → gl1.sav
    case 2: → gl2.sav
    case 3: → gl3.sav
    case 4: → gl4.sav
    default: → gl0.sav       // slot_index == 0 或 > 4
```

5 个槽位：`gl0.sav` ~ `gl4.sav`。

### 1.7 校验规则

```
oracle_validate_save(bytes):
    if bytes.len() < 26:                          // MIN_SAVE_SIZE
        return Err(TooShort)
    magic = u32_le(bytes[2..6])
    if magic != 0x01315A47:
        return Err(BadMagic)
    return Ok(())
```

| 错误 | 触发条件 |
|---|---|
| `SaveErr::TooShort` | 字节数不足最小有效大小 |
| `SaveErr::BadMagic` | 魔数不符 |
| `SaveErr::BadVersion` | `version > 2`（仅 v2 读路径） |

### 1.8 读档

```
oracle_read_slot(bytes):
    validate_save(bytes)?

    version   = u32_le(bytes[6..10])
    if version > 2:
        return Err(BadVersion)

    character = u32_le(bytes[10..14])
    gold      = u32_le(bytes[14..18])
    map_slot  = u32_le(bytes[18..22])
    position  = u32_le(bytes[22..26])

    return Ok(SaveSlot { version, character, gold, map_slot, position })
```

### 1.9 写档

```
oracle_write_slot(slot):
    buf = vec![0u8; 26]
    buf[0..2]   = [0x00, 0x00]                    // 前缀
    buf[2..6]   = REC_MAGIC.to_le_bytes()          // 0x01315A47
    buf[6..10]  = slot.version.to_le_bytes()
    buf[10..14] = slot.character.to_le_bytes()
    buf[14..18] = slot.gold.to_le_bytes()
    buf[18..22] = slot.map_slot.to_le_bytes()
    buf[22..26] = slot.position.to_le_bytes()
    return buf
```

`write_slot` 与 `read_slot` 往返逐字节一致。

### 1.10 v1 兼容读

v1 布局（无 `version` 与 `gold` 字段）：

```
offset  0:  2B  前缀
offset  2:  4B  魔数
offset  6:  4B  character
offset 10:  4B  map_slot
offset 14:  4B  position
```

```
oracle_read_slot_v1(bytes):
    if bytes.len() < 18: return Err(TooShort)
    magic = u32_le(bytes[2..6])
    if magic != 0x01315A47: return Err(BadMagic)

    character = u32_le(bytes[6..10])
    map_slot  = u32_le(bytes[10..14])
    position  = u32_le(bytes[14..18])

    return Ok(SaveSlot { version: 1, character, gold: 0, map_slot, position })
    // v1 无 version 字段 → 默认 1；v1 无 gold 字段 → 默认 0
```

### 1.11 v1 → v2 迁移

```
oracle_migrate_v1_to_v2(slot):
    new_version = (slot.version == 0 || slot.version == 1) ? 2 : slot.version
    return SaveSlot {
        version: new_version,
        character: slot.character,     // 不变
        gold: slot.gold,               // 不变
        map_slot: slot.map_slot,       // 不变
        position: slot.position,       // 不变
    }
```

| 输入 version | 输出 version |
|---|---|
| 0 | 2 |
| 1 | 2 |
| 2 | 2 |
| > 2 | 保持（读档时已 `Err(BadVersion)`） |

### 1.12 存档管理器结构

```
struct SaveManager {
    slots: Vec<SaveSlot>     // 默认 5 个槽位
}
```

| 方法 | 语义 |
|---|---|
| `new(slot_count)` | 构造（全部 `SaveSlot::default()`） |
| `slot_count()` | 槽位数 |
| `slot(index)` | 某槽位只读（越界 → `None`） |
| `validate(bytes)` | 校验字节 |
| `migrate(&slot)` | v1 → v2 迁移 |
| `read_slot(bytes)` | 解析 v2 字节 |
| `write_slot(&slot)` | 序列化 26 字节 |
| `set_slot(index, slot)` | 写入槽位（越界 → 静默忽略） |
| `magic()` | 魔数常量 |
| `min_size()` | 最小存档大小 |

`SaveSlot::default()`：`version=1, character=1, gold=0, map_slot=0, position=0`。

### 1.13 存读档流程

```
Index 场景 → [继续游戏] → saveload 子窗口

选择槽位（事件码 38/40 循环，13 确认）
    ↓
read_slot(bytes)
    ├─ Ok(slot)  → migrate(&slot) → 恢复游戏状态 → 进入 MainMap
    └─ Err(e)    → 显示错误提示（Err 变 Message 界面 Invalid kind）
```

```
主菜单/主地图 → [存档] → write_slot(&slot)
    ↓
写 gl{index}.sav（26 字节 v2 布局）
```

### 1.14 存读档门控

```
if rand() % 5 != 0:
    进入存读档流程
```

种子 = `wMinute + wSecond + wMilliseconds`（毫秒级时间播种）。
20% 概率跳过（推测为原版的设计 gimmick 或反作弊）。

### 1.15 存档时间戳

`sub_42AE8` 调 `GetLocalTime`，`sub_42DB0` 算时间戳：
- 前 8 字节存「起始/当前」时间戳
- `v4 <= Buffer + 2592000` 判 30 天过期

> ⚠️ `sub_42AE8` **不是播种点**（时间戳逻辑，非 RNG）。

---

## 2. RNG 系统

### 2.1 算法

MSVC 标准 LCG（**推测**，coredll.dll 导入 thunk 无法实测）：

```
srand(s):
    seed = s

rand():
    seed = seed × 214013 + 2531011
    return (seed >> 16) & 0x7FFF          // 值域 0..=32767
```

- 乘数 = 214,013（`0x346DC`）
- 加数 = 2,531,011（`0x269EC3`）
- 掩码 = `0x7FFF`
- 全部 u32 环绕运算（`wrapping_mul` / `wrapping_add`）

### 2.2 实现（公式单一源）

```
struct RngState { seed: u32 }

impl RngState {
    fn step(&mut self) -> i32 {
        self.seed = self.seed.wrapping_mul(214_013).wrapping_add(2_531_011);
        ((self.seed >> 16) & 0x7fff) as i32
    }
}
```

```
struct GameRng { state: RngState }

impl GameRng {
    fn new(seed: u32) -> Self
    fn set_seed(&mut self, s: u32)          // = srand
    fn next(&mut self) -> i32               // = rand（复用 step，公式单一源）
    fn fill(&mut self, buf: &mut [i32])     // = seq
    fn in_range(&mut self, n: u32) -> u32 {
        if n == 0 { return 0 }
        self.next().unsigned_abs() % n
    }
}
```

`GameRng` 只做「结构体 + 方法」封装，LCG 常数只写在 `RngState::step` 一处。

### 2.3 播种点

3 处播种，种子 = `wMinute + wSecond + wMilliseconds`：

| 函数 | 场景 | 用途 |
|---|---|---|
| `sub_3D4C8` | 卡牌场景 ctor | `srand` |
| `sub_4E314` | 存读档入口 | `srand` + `rand()%5` 门控 |
| `sub_4E568` | 存读档入口 | `srand` + `rand()%5` 门控 |

- 种子 = 分 + 秒 + 毫秒（**不是** 小时/天），毫秒级时间播种，秒级重复。
- 固定毫秒 → 固定种子 → 固定 `rand()` 序列，**确定性可复现**。

### 2.4 rand() 调用点分布

全库共 51 处 `rand()` 调用点（35 个函数）+ 3 处 `srand` + 2 个 CRT thunk。

| 模块 | 函数数 | 调用次数 | 典型用途 |
|---|---|---|---|
| battle | 11 | 12 | 伤害修正 `%9999+1`/`%6+1`/调色表随机 `byte_6C820[rand()%12]` |
| cards | 15 | 19 | 事件抽取 `%25+1`/卡牌效果 `%6+1`/26 张卡 `%26+1`/21 张卡面 `%21+1` |
| mainmap | 1 | 8 | 卡牌事件 id `%25+1`/坐标步进 `16*(rand()%6+1)`/百分比 `(rand()%40+80)*v/0x64` |
| trade | 3 | 3 | 行情 `%30+1`/买卖触发 `%100+1>40` |
| saveload | 2 | 2 | 存读档门控 `%5` |
| crt | 2 | 0 | `__imp_rand`/`__imp_srand` thunk（coredll 导入，非游戏逻辑） |

### 2.5 关键调用点详解

#### 主地图（`sub_2101C`，8 处）

| 行 | 表达式 | 用途 |
|---|---|---|
| 2841 | `rand() % 25 + 1` | 卡牌事件 id |
| 2055 | `rand() % 22 + 1` | 商品选择（22 种货物之一） |
| — | 裸 `rand()` × 2 | 通用采样 |
| — | `rand() % 25` | 卡牌库存修正 |
| — | `(rand() % 40 + 80) * v / 100` | 百分比 |
| — | `16 * (rand() % 6 + 1)` × 3 | 坐标步进 |

#### 卡牌（`sub_32960` 等，19 处）

| 行 | 表达式 | 用途 |
|---|---|---|
| 11 | `sub_16C9C(+136, 4) == 10` 时 `rand() % 10` | 卡牌库存修正 |
| 16 | `rand() % 6 + 1` | 卡牌效果等级 |
| 9 | `rand() % 26 + 1` | 26 张卡 |
| 14 | `rand() % 21 + 1` | 21 张卡面 |
| 11 | `4 * (rand() % v11)` | 偏移 |
| — | `rand() % 5 + 1` / `rand() % 9 + 1` / `rand() % 4` | 其他索引 |

#### 战斗（`sub_13CE4` 等，12 处）

| 行 | 表达式 | 用途 |
|---|---|---|
| 15 | `rand() % 9999 + 1` | 伤害修正 |
| 14 | `rand() % 6 + 1` | 乘数 |
| 16 | `rand() % v5 + 1` | 区间随机 |
| 17 | `byte_6C820[rand() % 12]` | 调色表随机 |
| 14/17 | `rand() % 5 + 1` / `rand() % 10 + 1` | 乘 / 0x64 百分比伤害 |

### 2.6 战斗 LCG 占位常数（与全局 RNG 独立）

`oracle_damage` 使用 `K = 2,654,435,761`（Knuth 乘子），**不与全局 RNG 统一**。

理由：这是战斗模块自有的伤害公式常数，语义与全局 LCG 序列不同，
强行统一会破坏已验证的 diff_battle 组合结果。

### 2.7 确定性验收

固定种子序列对拍（`tests/diff_rng.rs`）：
```
srand(123)
取 1000 次 rand()
oracle 序列 vs impl 序列 → 逐位一致
```

反证路径：若未来游戏侧能复现同种子序列 → 算法确认；否则需找 WinCE `coredll.dll` 逆向。

### 2.8 不使用 rand crate

- 不引 `rand` crate，不用 `SystemRandom`（ti-core 无平台依赖硬约束）。
- 用自实现 LCG，保证跨端（native/wasm/android）序列逐位一致。