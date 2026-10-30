# 09 — 持久化系统实现规格

> 单一事实源：本文件是 `ti-platform::fs` + `ti-formats::save` + `ti-core::rules::saveload` 的实现规格。
> 数值溯源（历史实测参照，归档 `legacy/`）：存档布局（前缀 `5D 4A` / magic 字节序列）与 RNG 播种点均来自 S1.9 实测，完整证据见 `legacy/evidence/`。LCG 为 oracle 历史参照，S8.7 起 rules 侧改 `rand_pcg` PCG。

---

## 1. 文件清单与路径

| 文件 | 平台路径 | 大小 | 用途 |
|---|---|---|---|
| `settings.dat` | `<data_dir>/settings.dat` | 8 B | 全局设置（音量/开关） |
| `gl0.sav` ~ `gl4.sav` | `<data_dir>/gl{N}.sav` | 18 B (v1) / 26 B (v2) | 5 个角色存档槽 |
| `gl.rec` | `<data_dir>/gl.rec` | 10 B | 全局进度记录 |

**`data_dir` 按端区分**：

| 端 | `data_dir` 解析 |
|---|---|
| 桌面 native | `./data`（exe 同级） |
| 浏览器 wasm | `web/`（fetch base） |
| Android | `getExternalFilesDir(null)` scoped storage |

---

## 2. 统一魔数与前缀

所有存档文件共享同一头部布局：

（下表为 `gl.rec` / `gl*.sav` 布局；`settings.dat` 例外，见 §3：前缀 `5D 4A`、无魔数。）

```
┌──────────┬─────────────────┬──────────────────────────┐
│ 偏移 0   │ 偏移 2          │ 偏移 6                    │
│ 2B 前缀  │ 4B 魔数 u32 LE  │ 负载（变长）               │
└──────────┴─────────────────┴──────────────────────────┘
```

- 前缀：2 字节。`gl*.sav` / `gl.rec` 写入固定 `[0x00, 0x00]`，读取时跳过（`seek(2, SeekFrom::Start)`）；`settings.dat` 前缀实测 `5D 4A`（见 §3.1）。
- 魔数：`REC_MAGIC = 0x01315A47`（= 20,011,591），**仅 `gl.rec` / `gl*.sav`；`settings.dat` 无魔数（实测 8 字节 `5D 4A 71 4A 2E 01 01 01`）**。
- 校验：`u32_le(bytes[2..6]) == REC_MAGIC`（gl.rec / gl*.sav）。

---

> 本节头布局适用于 `gl.rec` / `gl*.sav`；`settings.dat` 例外（无魔数，见 §3.1）。

## 3. settings.dat

### 3.1 字节布局

| 偏移 | 字节数 | 字段 | 类型 | 实测值 | 说明 |
|---|---|---|---|---|---|
| 0 | 2 | 前缀 | u8[2] | `5D 4A` | 固定（前缀 magic 实测值） |
| 2 | 4 | 负载 | u8[4] | `71 4A 2E 01` | 无魔数（settings.dat 例外，实测 8 字节全量） |
| 6 | 1 | settings[0] | u8 | `0x01` | 音量/BGM 开 |
| 7 | 1 | settings[1] | u8 | `0x01` | 音效开 |

> 注意：`settings.dat` **无** `REC_MAGIC` 魔数（魔数仅 `gl.rec` / `gl*.sav`，§2）。

### 3.2 Rust 类型

> §3.2 的读写逻辑对 `settings.dat` 不适用魔数校验（无魔数，仅校验 8 字节长度 + 负载值域 0..=100）。

```rust
pub struct Settings {
    pub bgm_volume: u8,   // 0..=100
    pub sfx_enabled: u8,  // 0 or 1
}

impl Default for Settings {
    fn default() -> Self {
        Settings { bgm_volume: 60, sfx_enabled: 1 }
    }
}
```

### 3.3 读写流程

```
read_settings(path):
    bytes = read_file(path)?
    if bytes.len() < 8: return Err(TooShort)
    # settings.dat 无魔数校验（§3.1，（§3.1）），仅校验长度 + 负载值域
    return Ok(Settings {
        bgm_volume: bytes[6],
        sfx_enabled: bytes[7],
    })

write_settings(path, settings):
    # 实测前缀 5D 4A（前缀 magic 实测值）；settings.dat 负载无魔数校验（§3.1 → §3），此处 4 字节负载按读取侧回填值写回
    buf = [0x5D, 0x4A, 0x71, 0x4A, 0x2E, 0x01, settings.bgm_volume, settings.sfx_enabled]
    write_file(path, &buf)
```

### 3.4 校验规则

| 条件 | 错误 |
|---|---|
| `bytes.len() < 8` | `SaveErr::TooShort` |
| 文件不存在 | 使用 `Settings::default()`，不报错 |

> `settings.dat` 无 `BadMagic` 路径（无魔数，§3.1）；`BadMagic` 仅适用于 `gl.rec` / `gl*.sav`。

---

> `settings.dat` 无魔数（§3.1 → §3），本节魔数校验仅适用于 `gl.rec` / `gl*.sav`。

## 4. gl.rec

### 4.1 字节布局

| 偏移 | 字节数 | 字段 | 语义 |
|---|---|---|---|
| 0 | 2 | 前缀 | 跳过 |
| 2 | 4 | 魔数 | `0x01315A47` |
| 6 | 1 | `byte_81194` | 全局状态 1 |
| 7 | 1 | `byte_81195` | 全局状态 2 |
| 8 | 1 | `byte_81196` | 全局状态 3 |
| 9 | 1 | `byte_81197` | 全局状态 4 |

### 4.2 语义规则

```
if byte_81194 != 0 && byte_81195 != 0 && byte_81196 != 0 && byte_81197 != 0:
    5 角色初始化完成（a1[13] = 5）
```

### 4.3 校验规则

| 条件 | 错误 |
|---|---|
| `bytes.len() < 10` | `SaveErr::TooShort` |
| `magic != REC_MAGIC` | `SaveErr::BadMagic` |

---

## 5. gl*.sav 存档槽

### 5.1 槽位映射

| 槽位索引 | 文件名 | 说明 |
|---|---|---|
| 0 | `gl0.sav` | 默认/兜底 |
| 1 | `gl1.sav` | 槽 1 |
| 2 | `gl2.sav` | 槽 2 |
| 3 | `gl3.sav` | 槽 3 |
| 4 | `gl4.sav` | 槽 4 |

```
fn slot_path(data_dir: &Path, slot_index: u32) -> PathBuf {
    let name = match slot_index {
        1 => "gl1.sav",
        2 => "gl2.sav",
        3 => "gl3.sav",
        4 => "gl4.sav",
        _ => "gl0.sav",
    };
    data_dir.join(name)
}
```

### 5.2 v2 布局（当前版本）

```
┌──────────┬────────────┬───────────┬────────────┬──────────┬───────────┬───────────┐
│ 偏移 0   │ 偏移 2     │ 偏移 6    │ 偏移 10    │ 偏移 14  │ 偏移 18   │ 偏移 22   │
│ 2B 前缀  │ 4B 魔数    │ 4B ver    │ 4B char    │ 4B gold  │ 4B map    │ 4B pos    │
└──────────┴────────────┴───────────┴────────────┴──────────┴───────────┴───────────┘
```

| 偏移 | 字节数 | 字段 | 类型 | 值域 | 说明 |
|---|---|---|---|---|---|
| 0 | 2 | 前缀 | u8[2] | `00 00` | 跳过 |
| 2 | 4 | 魔数 | u32 LE | `0x01315A47` | 校验 |
| 6 | 4 | version | u32 LE | `2` | 格式版本 |
| 10 | 4 | character | u32 LE | `1..=5` | 角色索引 |
| 14 | 4 | gold | u32 LE | `0..=999999` | 当前金量 |
| 18 | 4 | map_slot | u32 LE | `0..=4` | 地图索引 |
| 22 | 4 | position | u32 LE | `0..` | 游戏进度 |

v2 最小有效大小：**26 字节**。

### 5.3 v1 布局（兼容）

```
┌──────────┬────────────┬────────────┬───────────┬───────────┐
│ 偏移 0   │ 偏移 2     │ 偏移 6     │ 偏移 10   │ 偏移 14   │
│ 2B 前缀  │ 4B 魔数    │ 4B char    │ 4B map    │ 4B pos    │
└──────────┴────────────┴────────────┴───────────┴───────────┘
```

v1 与 v2 差异：无 `version` 和 `gold` 字段，其余字段相同位置。

v1 最小有效大小：**18 字节**。

### 5.4 Rust 类型

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct SaveSlot {
    pub version: u32,
    pub character: u32,
    pub gold: u32,
    pub map_slot: u32,
    pub position: u32,
}

impl Default for SaveSlot {
    fn default() -> Self {
        SaveSlot {
            version: 1,
            character: 1,
            gold: 0,
            map_slot: 0,
            position: 0,
        }
    }
}

#[derive(Debug)]
pub enum SaveErr {
    TooShort,
    BadMagic,
    BadVersion,
    Io(String),
}
```

### 5.5 v2 读写

```
read_slot_v2(bytes) -> Result<SaveSlot, SaveErr>:
    if bytes.len() < 26: return Err(TooShort)
    magic = u32_le(bytes[2..6])
    if magic != REC_MAGIC: return Err(BadMagic)
    version   = u32_le(bytes[6..10])
    if version > 2: return Err(BadVersion)
    character = u32_le(bytes[10..14])
    gold      = u32_le(bytes[14..18])
    map_slot  = u32_le(bytes[18..22])
    position  = u32_le(bytes[22..26])
    return Ok(SaveSlot { version, character, gold, map_slot, position })

write_slot_v2(slot) -> Vec<u8>:
    buf = vec![0u8; 26]
    buf[2..6]   = REC_MAGIC.to_le_bytes()
    buf[6..10]  = slot.version.to_le_bytes()
    buf[10..14] = slot.character.to_le_bytes()
    buf[14..18] = slot.gold.to_le_bytes()
    buf[18..22] = slot.map_slot.to_le_bytes()
    buf[22..26] = slot.position.to_le_bytes()
    return buf
```

### 5.6 v1 兼容读

```
read_slot_v1(bytes) -> Result<SaveSlot, SaveErr>:
    if bytes.len() < 18: return Err(TooShort)
    magic = u32_le(bytes[2..6])
    if magic != REC_MAGIC: return Err(BadMagic)
    character = u32_le(bytes[6..10])
    map_slot  = u32_le(bytes[10..14])
    position  = u32_le(bytes[14..18])
    return Ok(SaveSlot {
        version: 1,
        character,
        gold: 0,        // v1 无 gold 字段
        map_slot,
        position,
    })
```

### 5.7 自动版本检测

```
read_slot_auto(bytes) -> Result<SaveSlot, SaveErr>:
    if bytes.len() < 18: return Err(TooShort)
    magic = u32_le(bytes[2..6])
    if magic != REC_MAGIC: return Err(BadMagic)

    if bytes.len() >= 26:
        version = u32_le(bytes[6..10])
        if version > 2:
            return Err(BadVersion)      // 与 §5.2 一致
        if version == 1:
            // 短布局 v1（18 字节）或 version 字段为 1
            return read_slot_v1(bytes)
        // version == 0 视为无 version 字段的 v1
        if version == 0:
            return read_slot_v1(bytes)
        return read_slot_v2(bytes)

    // 默认按 v1 处理
    return read_slot_v1(bytes)
```

### 5.8 v1 → v2 迁移

```
migrate_v1_to_v2(slot) -> SaveSlot:
    // 与 oracle/saveload.rs oracle_migrate_v1_to_v2 一致：
    // version ∈ {0,1} → 2，其余保持；gold 及其余字段原样保留（v1 读档时 gold 默认 0，
    // 迁移不重写 gold 字段）。
    new_version = if slot.version == 0 || slot.version == 1 { 2 } else { slot.version }
    return SaveSlot {
        version: new_version,
        character: slot.character,
        gold: slot.gold,
        map_slot: slot.map_slot,
        position: slot.position,
    }
```

| 输入 version | 输出 version | gold 处理 |
|---|---|---|
| 0 | 2 | 原样保留（v1 读档时已默认 0） |
| 1 | 2 | 原样保留（v1 读档时已默认 0） |
| 2 | 2 | 保持 |
| > 2 | 读档时已 `Err(BadVersion)` | — |

### 5.9 SaveManager

```rust
pub struct SaveManager {
    slots: Vec<SaveSlot>,
}

impl SaveManager {
    pub fn new(slot_count: usize) -> Self;
    pub fn slot_count(&self) -> usize;
    pub fn slot(&self, index: usize) -> Option<&SaveSlot>;
    pub fn set_slot(&mut self, index: usize, slot: SaveSlot);
    pub fn validate(&self, bytes: &[u8]) -> Result<(), SaveErr>;
    pub fn read_slot(&self, bytes: &[u8]) -> Result<SaveSlot, SaveErr>;
    pub fn write_slot(&self, slot: &SaveSlot) -> Vec<u8>;
    pub fn migrate(&self, slot: &SaveSlot) -> SaveSlot;
    pub fn magic() -> u32 { REC_MAGIC }
    pub fn min_size() -> usize { 26 }
}
```

### 5.10 存读档流程

**读档**：

```
Index → [继续游戏] → Saveload 子窗口
  → 选择槽位（事件码 38/40 循环，13 确认）
    → read_slot_auto(bytes)
      ├─ Ok(slot) → migrate_v1_to_v2(slot) → 恢复游戏状态 → MainMap
      └─ Err(e) → 显示错误提示（Message Invalid kind）
```

**写档**：

```
主菜单/主地图 → [存档] → Saveload 子窗口
  → 选择空槽位（事件码 38/40 循环，13 确认）
    → write_slot_v2(slot) → 写入 gl{N}.sav
```

### 5.11 存读档门控

```
if rand() % 5 != 0:
    进入存读档流程
```

- 种子 = `wMinute + wSecond + wMilliseconds`（毫秒级时间播种）。
- 20% 概率跳过（`rand() % 5 != 0` 门控，RNG 播种点（历史实测））。

---

## 6. 存档时间戳

> 以下布局为 `gl.rec` 扩展区时间戳字段（`WriteFile`/`00042AE8:存档写`），**非 v2 26 字节存档槽布局**；
> 存档槽（gl*.sav）的 26 字节布局见 §5.2，时间戳不落在 26 字节槽内（推测，S3 逆向产物）。

| 字段 | 偏移 | 说明 |
|---|---|---|
| 起始时间戳 | 0..8 | 创建时间 |
| 当前时间戳 | 8..16 | 最后修改时间 |

```
过期判定 = (current_timestamp - start_timestamp) <= 2_592_000  // 30 天（秒）
```

> `WriteFile` 调 `GetLocalTime` 是时间戳逻辑，**不是** RNG 播种点（历史实测）。

---

## 7. 错误处理与校验总结

| 错误类型 | 触发条件 | 用户可见行为 |
|---|---|---|
| `TooShort` | 字节数不足最小有效大小 | 提示「存档损坏」 |
| `BadMagic` | 魔数不符 | 提示「存档损坏」 |
| `BadVersion` | version > 2 | 提示「版本不兼容」 |
| `Io` | 文件系统错误 | 提示「读写失败」 |

**校验伪代码**：

```
fn validate_save(bytes: &[u8]) -> Result<(), SaveErr> {
    if bytes.len() < 26 { return Err(TooShort); }
    let magic = bytes[2..6].try_into().ok()?.try_into(); // 文档伪代码示意；实际用 u32::from_le_bytes(bytes[2..6].try_into().ok()?)
    let magic: u32 = u32::from_le_bytes(magic);
    if magic != REC_MAGIC { return Err(BadMagic); }
    Ok(())
}
```

---

## 8. 平台 IO 抽象

> **规格（尚未实现）**：`ti-platform` 现有实现为同步 `trait Storage`（`crates/ti-platform/src/backend.rs:104`，
> `fn read/write(&mut self, key, value)`）；下方 `SaveIo` 为独立规格，`#[async_trait]` 与 ti-platform 无关。

```rust
#[async_trait]
pub trait SaveIo {
    async fn read(&self, path: &str) -> Result<Vec<u8>, SaveErr>;
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), SaveErr>;
    async fn exists(&self, path: &str) -> bool;
    async fn delete(&self, path: &str) -> Result<(), SaveErr>;
}
```

| 端 | 实现 |
|---|---|
| 桌面 native | `LocalFileIo`（`std::fs`） |
| 浏览器 wasm | `WasmFileIo`（`localStorage` / IndexedDB） |
| Android | `AndroidFileIo`（scoped storage） |

---

## 9. 数据完整性保障

1. **写前读**：写档前先读旧文件，保留备份 `gl{N}.sav.bak`。
2. **CRC 校验**（可选 v3 预留）：v2 末尾可追加 4B CRC32。
3. **原子写入**：先写 `.tmp` 文件，成功后 `rename` 覆盖。
4. **首次启动**：无 `settings.dat` 时创建默认设置；无 `gl.rec` 时创建全零记录。
