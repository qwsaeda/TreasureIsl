# `LucencyPackage` 容器格式规格（`*.pak` / `*.lpk`）

> 全部结论由 S1.1 实测得出，证据见 `docs/reverse-facts.md` F-201~F-209 与 `evidence/S1.1.log`。
> 解析器实现：`crates/ti-formats/src/lucency.rs`；验收：`crates/ti-tools/tests/pak_key.rs`。

## 1. 头部（固定 0x36 字节）

| 偏移 | 长度 | 含义 | 实测值 |
|---|---|---|---|
| `0x00` | 16 | magic `LucencyPackage\0\0` | 全部 7 个容器一致 |
| `0x10` | 4 | 未知常量 | 恒为 `03 02 F0 03` |
| `0x14` | 4 | 未知 | `ui.pak`=`00020000`，其余=`00010000` |
| `0x18` | 2 | 未知 | 恒 `0x0000` |
| `0x1A` | 2 | **版本号** | `2004`（ui.pak）/ `2005`（其余，含 glpics.pak） |
| `0x1C` | 2 | 未知 | 10 / 4 / 1 … |
| `0x1E` | 2 | 未知 | 12 / 27 / 21 … |
| `0x20` | 2 | 未知 | 7 / 5 / 8 … |
| `0x22` | 2 | 未知 | 10 / 57 / 42 / 43 … |
| `0x24` | 2 | 未知 | — |
| `0x26` | 4 | **条目数** `entry_count` | 122 / 88 / 47 / 37 / 4 / 6 / 1 |
| `0x2A` | 4 | （废弃，读出无意义值） | ⚠️ 旧文档误记为「索引长度」 |
| `0x2E` | 4 | **索引流长度** `index_len` | 1470 / 1112 / 597 / 498 / … |
| `0x32` | 4 | **数据区起点** `data_start` | 1524 / 1166 / 651 / 552 / 118 / 150 / 70 |
| `0x36` | — | 索引流起点（固定） | `data_start = 0x36 + index_len`（全部成立） |

> 头部字段**非 4 字节对齐**，必须按上表逐字段读取，禁止 `#[repr(C)]` 直接映射。

## 2. 索引流

- 起点 `0x36`，长度 `index_len`。
- **编码**：`index_len ≥ 2` 且首 2 字节满足 zlib 头（`CM==8` 且 `(CMF<<8|FLG) % 31 == 0`）→ zlib 压缩；否则为原始记录。
  - 实测：4 个 `.pak` 全压缩，3 个 `.lpk` 全原始。**与版本号/扩展名无必然关系**。
- 解码后长度必须 `== entry_count × 16`，否则报错（这条校验可挡住"切片错位导致的假阳性"）。

### 2.1 记录布局（16 字节，小端）

| 偏移 | 4 字节 | 含义 |
|---|---|---|
| `0x0` | u32 | `key` —— 资源键（见 §3） |
| `0x4` | u32 | `off` —— 相对 `data_start` 的偏移 |
| `0x8` | u32 | `usize` —— 解压后长度 |
| `0xC` | u32 | `csize` —— 压缩后长度 |

**不变量**：`entries[i+1].off == entries[i].off + entries[i].csize`（7 个容器全部成立，已写成单测）。

## 3. `key` = `name_key(name)`

```rust
fn name_key(name: &str) -> u32 {
    let mut h: u32 = 0;
    for b in name.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u32::from(b.to_ascii_lowercase()));
    }
    h
}
```

即 **djb2，种子 0，先 `to_ascii_lowercase()`**。大小写不敏感（`Dialogue.pcx` 与 `dialogue.pcx` 同 key）。

实测向量（单测 `name_key_matches_measured_keys`）：

| 名字 | key |
|---|---|
| `landinfo.pcx` | `0x5C5AF184` |
| `number.pcx` | `0xFCFFFB62` |
| `target_area.pcx` | `0xE56BBE98` |
| `dialogue.pcx` | `0xFFFB9A43` |
| `logo.pcx` | `0x047D426A` |
| `mapinfo.ini` | `0x3753DC98` |

### 3.1 key 的第二类：ID 型（S1.11 已解：容器内 `mapinfo.ini` 绑定）

- **绑定机制 = 容器内自带 `mapinfo.ini`**（key `0x3753DC98`，UTF-16LE+BOM，`[filenum]` +
  逐条 `filename=`），`key = djb2(filename)` 全部闭合（ui 121 行 / m01 49 行 / m02 34 行，0 失配）。
  结构与清单见 F-210；单测 `cargo test -p ti-tools --test id_keys`。
- 不在 mapinfo/exe 内的 9 条：`appo01~04.pcx`（m02 idx13~16，公差 `33^4` 四连命中）、
  `card1/2.pcx`（cards idx1/2，内容 250×120 PCX）由**多键结构同时命中**证实；
  `Thumbs.db`（ui idx121，内容 OLE 复合文档）由内容格式吻合证实。
- **覆盖率**：`DoD` 范围（ui/glmap/lpk）**217/217 全解**；全 305 条中 302 条有名，
  余 3 条在 `glpics.pak`（名称哈希型，DoD 不含，见 F-209）。
- ⚠️ 旧记录「ui 前 90 个 key 公差恒为 `0x121101`」是错的：idx0..20 实测公差集合 =
  {`33^4` = `0x121881`, `24×33^4`（进位 `09→10`）}。
- `.Map` 场景元素**不直接持 key**：`.Map` 记录布局（F-306）与其字段级语义属 S1.4b。

## 4. 数据区

- 每个条目是**独立 zlib 流**（实测首两字节 `78 DA`）。
- 条目 `i` 的压缩流位于 `[data_start + off_i, data_start + off_i + csize_i)`。
- 解压后长度应 `== usize_i`（解析器按此分配缓冲）。

## 5. 容器实测清单

| 容器 | 版本 | 条目数 | 索引编码 | `data_start` |
|---|---|---|---|---|
| `ui.pak` | 2004 | 122 | zlib | 1524 |
| `glpics.pak` | 2005 | 88 | zlib | 1166 |
| `glmap_m01.pak` | 2004 | 47 | zlib | 651 |
| `glmap_m02.pak` | 2004 | 37 | zlib | 552 |
| `cards.lpk` | 2005 | 4 | 原始 | 118 |
| `music.lpk` | 2005 | 6 | 原始 | 150 |
| `goods.lpk` | 2005 | 1 | 原始 | 70 |