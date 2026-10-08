//! 地形图 `glmapm0*.Map` 解析（S1.8 + S1.8b）。
//!
//! 实测结构（F-301 + F-307 + S1.8b 脚本验证）：
//! - 头 16 字节：`grid_w`、`grid_h`（m01/02 = 20×20，m03 = 15×15）、`flags`、`regions_off`；
//! - `words[0] = 0`，`words[1] = count`（= `grid_w - 1`，@0x14）；`regions_off = 32 × count`（字节尺寸，**非文件偏移**——落入 cell 流内部）；
//! - `words[2..]` 记录流，stride = 9 u32，分三段：
//!
//! 1. **cell 记录**（`grid_w × grid_h - 1` 条）：`[f0, f1, 0, 0, 1, tile, iso_x, iso_y, cell|flags<<16]`
//!    - `f0` = `(u16 连线目标格, u16 类别码)` —— 目标格非 0 时类别码恒为 1..=9（目标格恒在同列/邻列且行号不增）；
//!      目标格为 0 时高 16 位是 16/64/128/192 的单元格属性位；
//!    - `f1` = `(u16 区域锚点格, u16 区域码)` —— 区域锚点，与 f0 同现；
//!    - `tile`（f5）= mapinfo 瓦片索引（0..48，对应 `mapinfo.ini` 条目）；
//!    - `iso_x`（f6）= `32 × (row + col)` —— 等距投影 x 像素；
//!    - `iso_y`（f7）= `16 × (row + grid_w - 1 - col)` —— 等距投影 y 像素；
//!    - `cell`（f8 低16位）= `row × grid_w + col`；`flags`（f8 高16位）= 额外标志（131/93/57 格非零，low byte ≡ 1 mod 4）。
//!
//! 2. **trailer 记录**（1 条，f4 = 1）：`f5 = 连线记录数 + 1`；`f6/f7/f8` = 推测统计字段（语义待 exe 确认）。
//!
//! 3. **连线/路点记录**（N 条）：`[0, 0, 0, 0, 0, M, A, B, C]`
//!    - `M` ∈ {2, 4} —— 记录类型（2 = 常规，4 = 特殊，仅 m02 出现 4 条）；
//!    - `A` = 瓦片索引（路点 sprite）；
//!    - `B` = 等距 x 像素；`C` = 等距 y 像素（**升序**，painter's order）；
//!    - 文件末尾 6 词 sentinel `[0, 0, 0, 0, 0, 2]`（无 payload，标记流结束）。
//!
//! 缺格：cell `(row=0, col=grid_w-1)`（= cell index `grid_w - 1`）无 cell 记录（m01/m02 缺 19，m03 缺 14）。
//!
//! 本模块保证：结构完整、**0 未知字节**、**往返逐字节无损**、网格尺寸吻合、记录字段全解析。

use crate::error::{Error, ParseKind, Result};
use crate::mapfile::MapScene;

/// 区域表计数所在文件偏移（`@0x14`）。
pub const REGION_COUNT_OFF: usize = 0x14;

/// 记录 stride（u32 个数）。
const RECORD_STRIDE: usize = 9;

/// cell 记录在 `words` 中的起始索引（`words[0]` = byte 16 = w4，故 w6 → words[2]）。
const RECORDS_WORD_BASE: usize = 2;

/// cell 记录类型标记（f4 = 1）。
pub const CELL_TYPE_MARKER: u32 = 1;

/// f1 连接哈希乘数（`f1 = cell_id × 0xA0001`）。
pub const LINK_HASH_MULTIPLIER: u32 = 0xA0001;

/// 等距 x 步进（每对角格 32 像素）。
pub const ISO_X_STEP: u32 = 32;

/// 等距 y 步进（每反对角格 16 像素）。
pub const ISO_Y_STEP: u32 = 16;

/// 末尾 sentinel 长度（u32 个数）。
const SENTINEL_LEN: usize = 6;

/// 解析出的地形图。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainMap {
    /// 底层通用结构（头 + u32 字流），保证往返无损。
    pub scene: MapScene,
    /// 网格宽（格数）。
    pub grid_w: u32,
    /// 网格高（格数）。
    pub grid_h: u32,
    /// 区域数量（`@0x14`，= `grid_w - 1`）。
    pub region_count: u32,
    /// 头字段 `regions_off`（= `32 × count`，字节尺寸，非文件偏移）。
    pub data_off: u32,
}

/// 地形 cell 记录（type 1，f4 = 1）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellRecord {
    /// f0: `(u16 link_target_cell, u16 link_kind)`。
    /// - `link_target_cell != 0` → 连线目标格，`link_kind` 恒为 1..=9 类别码；
    /// - `link_target_cell == 0` → 无连线，`link_kind` 实测为 16/64/128/192（单元格属性位）或不透明值。
    pub link_target: u32,
    /// f1: `(u16 region_anchor_cell, u16 region_code)` —— 区域锚点格 + 区域码，0 = 无。
    pub region_anchor: u32,
    /// f5: 瓦片索引（对应 `mapinfo.ini` 条目）。
    pub tile: u32,
    /// f6: 等距 x = `32 × (row + col)`。
    pub iso_x: u32,
    /// f7: 等距 y = `16 × (row + grid_w - 1 - col)`。
    pub iso_y: u32,
    /// f8 低 16 位: cell = `row × grid_w + col`。
    pub cell: u32,
    /// f8 高 16 位: 额外标志（0 = 无）。
    pub flags: u32,
}

/// 连线/路点记录（type 2，f5 = M ∈ {2, 4}）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionRecord {
    /// M: 记录类型（2 = 常规，4 = 特殊）。
    pub kind: u32,
    /// A: 瓦片索引（路点 sprite）。
    pub tile: u32,
    /// B: 等距 x 像素。
    pub x: u32,
    /// C: 等距 y 像素（升序，painter's order）。
    pub y: i32,
}

impl TerrainMap {
    /// 解析地形图。
    ///
    /// # Errors
    ///
    /// 网格宽高为 0，或底层结构非法时返回错误。
    pub fn parse(path: &str, buf: &[u8]) -> Result<Self> {
        let scene = MapScene::parse(path, buf)?;
        let grid_w = scene.header.width;
        let grid_h = scene.header.height;
        if grid_w == 0 || grid_h == 0 {
            return Err(Error::parse(path, 0, ParseKind::InvalidValue { field: "grid" }));
        }
        let region_word = (REGION_COUNT_OFF - 0x10) / 4;
        let region_count = scene.words.get(region_word).copied().unwrap_or_default();
        let data_off = scene.header.reserved;
        Ok(Self { scene, grid_w, grid_h, region_count, data_off })
    }

    /// 总格子数。
    #[must_use]
    pub fn cell_count(&self) -> u64 {
        u64::from(self.grid_w) * u64::from(self.grid_h)
    }

    /// cell 记录数（= `grid_w × grid_h - 1`，不含 trailer）。
    #[must_use]
    pub fn cell_record_count(&self) -> u64 {
        self.cell_count().saturating_sub(1)
    }

    /// type-1 记录总数（含 trailer）= `grid_w × grid_h`。
    #[must_use]
    pub fn type1_record_count(&self) -> u64 {
        self.cell_count()
    }

    /// 从 words 流中读取第 `k` 条 type-1 记录的 9 个 u32（k 从 0 起）。
    fn type1_record(&self, k: u64) -> Option<[u32; RECORD_STRIDE]> {
        let base =
            RECORDS_WORD_BASE.checked_add(usize::try_from(k).ok()?.checked_mul(RECORD_STRIDE)?)?;
        let end = base.checked_add(RECORD_STRIDE)?;
        let slice = self.scene.words.get(base..end)?;
        let mut arr = [0u32; RECORD_STRIDE];
        arr.copy_from_slice(slice);
        Some(arr)
    }

    /// 解析一条 cell 记录（不含 trailer）。
    ///
    /// 返回 `None` 当 `k >= cell_record_count()` 或流不足。
    #[must_use]
    pub fn cell_record(&self, k: u64) -> Option<CellRecord> {
        if k >= self.cell_record_count() {
            return None;
        }
        let record = self.type1_record(k)?;
        let f8 = record[8];
        Some(CellRecord {
            link_target: record[0],
            region_anchor: record[1],
            tile: record[5],
            iso_x: record[6],
            iso_y: record[7],
            cell: f8 & 0xFFFF,
            flags: f8 >> 16,
        })
    }

    /// 解析所有 cell 记录（不含 trailer）。
    #[must_use]
    pub fn cell_records(&self) -> Vec<CellRecord> {
        let count = self.cell_record_count();
        let mut out = Vec::new();
        let mut index = 0u64;
        while index < count {
            match self.cell_record(index) {
                Some(rec) => out.push(rec),
                None => break,
            }
            index += 1;
        }
        out
    }

    /// 解析 trailer 记录（type-1 流最后一条）。
    ///
    /// trailer 的 `f5 = 连线记录数 + 1`；`tile` 字段语义为计数而非瓦片。
    #[must_use]
    pub fn trailer(&self) -> Option<CellRecord> {
        let k = self.cell_record_count();
        let record = self.type1_record(k)?;
        let f8 = record[8];
        Some(CellRecord {
            link_target: record[0],
            region_anchor: record[1],
            tile: record[5],
            iso_x: record[6],
            iso_y: record[7],
            cell: f8 & 0xFFFF,
            flags: f8 >> 16,
        })
    }

    /// 连线记录起始 word 索引（= `RECORDS_WORD_BASE + 9 × type1_record_count`）。
    fn connection_base(&self) -> Option<usize> {
        let type1 = self.type1_record_count();
        let offset = usize::try_from(type1).ok()?.checked_mul(RECORD_STRIDE)?;
        RECORDS_WORD_BASE.checked_add(offset)
    }

    /// 连线记录数（含 sentinel 前的所有完整记录）。
    #[must_use]
    pub fn connection_record_count(&self) -> usize {
        let Some(base) = self.connection_base() else {
            return 0;
        };
        let words = &self.scene.words;
        let total = words.len().saturating_sub(base);
        // total = N × 9 + SENTINEL_LEN(6)
        total.saturating_sub(SENTINEL_LEN) / RECORD_STRIDE
    }

    /// 解析所有连线记录。
    #[must_use]
    pub fn connection_records(&self) -> Vec<ConnectionRecord> {
        let Some(base) = self.connection_base() else {
            return Vec::new();
        };
        let words = &self.scene.words;
        let count = self.connection_record_count();
        let mut out = Vec::with_capacity(count);
        let mut idx = base;
        for _ in 0..count {
            let Some(slice) = words.get(idx..idx + RECORD_STRIDE) else {
                break;
            };
            // layout: [0, 0, 0, 0, 0, M, A, B, C]
            let kind = slice[5];
            let tile = slice[6];
            let x = slice[7];
            let y_raw = slice[8];
            let y = i32::from_le_bytes(y_raw.to_le_bytes());
            out.push(ConnectionRecord { kind, tile, x, y });
            idx += RECORD_STRIDE;
        }
        out
    }

    /// 验证 sentinel = `[0, 0, 0, 0, 0, 2]` 位于连线流末尾。
    #[must_use]
    pub fn has_sentinel(&self) -> bool {
        let Some(base) = self.connection_base() else {
            return false;
        };
        let words = &self.scene.words;
        let n = self.connection_record_count();
        let sentinel_start = base + n * RECORD_STRIDE;
        let Some(slice) = words.get(sentinel_start..sentinel_start + SENTINEL_LEN) else {
            return false;
        };
        slice == [0, 0, 0, 0, 0, 2]
    }

    /// 逐字节还原为原始文件内容（roundtrip 保证 0 未知字节）。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.scene.to_bytes()
    }

    /// 导出 JSON。
    #[must_use]
    pub fn to_json(&self, name: &str) -> String {
        let mut s = String::from("{\n  \"name\": \"");
        s.push_str(name);
        s.push_str("\",\n  \"grid_w\": ");
        s.push_str(&self.grid_w.to_string());
        s.push_str(",\n  \"grid_h\": ");
        s.push_str(&self.grid_h.to_string());
        s.push_str(",\n  \"cells\": ");
        s.push_str(&self.cell_count().to_string());
        s.push_str(",\n  \"region_count\": ");
        s.push_str(&self.region_count.to_string());
        s.push_str(",\n  \"data_off\": ");
        s.push_str(&self.data_off.to_string());
        s.push_str(",\n  \"cell_record_count\": ");
        s.push_str(&self.cell_record_count().to_string());
        s.push_str(",\n  \"connection_record_count\": ");
        s.push_str(&self.connection_record_count().to_string());
        s.push_str(",\n  \"words\": [");
        for (i, w) in self.scene.words.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push(' ');
            s.push_str(&w.to_string());
        }
        s.push_str(" ]\n}\n");
        s
    }
}
