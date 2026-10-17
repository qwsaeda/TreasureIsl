//! `.Map` 场景文件（S1.4）。
//!
//! 实测结构（`legacy/docs/facts/index.md` F-301/F-302）：
//! - 头部 16 字节：`width=0xB0(176)`、`height=0xDC(220)`、`flags`、`reserved`；
//! - 其后为 u32 字流（控件/场景记录），**文件内无任何字符串**（F-302）；
//! - 文件长度必为 `16 + 4k`，因此 roundtrip 可逐字节还原（0 未知字节）。
//!
//! 记录级布局已逆出（S1.11 / F-306：`words.len() == 5 + 6*count`，`count = words[4]`）；
//! 字段语义已结合 exe 侧（`sub_44004` → `sub_46294` → `sub_449C8` → `sub_460F8`，S1.4b）核实：
//! `f1` 精灵索引、`f2/f3` 屏幕 X/Y、`f4` 贴图例程模式、`f5` 控件类型
//! （见 [`MapControl::f1`]~[`MapControl::f5`] 与 `ti_tools::ids`，`ref/controls.json` 由 `ti-tools` 的
//! `controls` bin 全量导出；地形图 `glmapm*.Map` 不适用此控件布局）。

use crate::define_format;
use crate::error::{Error, ParseKind, Result};

/// 记录级布局常量（S1.11 / F-306 + S1.4b exe 侧确认）。
///
/// 16 字节头部之后的 u32 字流：前 4 个为表头，第 5 个为控件数，
/// 其后每 [`MapControl::FIELDS`] 个字为一条控件记录。
pub const TABLE_HEAD_WORDS: usize = 4;
/// 表头之外的额外字（控件数字段）。
pub const RECORD_WORD_COUNT: usize = 1;
/// 表头 + 控件数字段 = 控件数据起始偏移（以字为单位）。
pub const HEADER_WORDS: usize = TABLE_HEAD_WORDS + RECORD_WORD_COUNT;

/// 一条 `.Map` 控件记录（16 字节头部后字流内的 6 个 u32，S1.4b 实测语义）。
///
/// 顺序与 exe 侧读取一致：`sub_44004` 按 24 字节（6×u32）读入，
/// 经 `sub_46294` 落到控件对象 `ctrl[1]..=ctrl[7]`，再由 `sub_449C8`
/// 拷入容器节点；`sub_460F8` 以 `node+2` 为基址取用。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapControl {
    /// 记录第 1 字段（`ctrl[1]`，+0x04）。全部实测恒为 1，语义待定（**推测** 使能标志）。
    pub f0: u32,
    /// 精灵 / 贴图索引（`ctrl[2]`，+0x08）。实测 1..119，对应 `ui.pak`/`glpics.pak` 位图。
    pub f1: u32,
    /// 屏幕 X 坐标（`ctrl[3]`，+0x0C）。`sub_460F8` 用作 blit 原点 `dst.x`；
    /// 实测 p95 = 176 = 场景宽。
    pub f2: u32,
    /// 屏幕 Y 坐标（`ctrl[4]`，+0x10）。`sub_460F8` 用作 blit 原点 `dst.y`；
    /// 实测 p95 = 216 < 场景高 219。**F-306 曾推测为事件 id，S1.4b 实测推翻。**
    pub f3: u32,
    /// 贴图例程模式（`ctrl[7]`，+0x1C）。实测 ∈ {1, 2, 3}；`sub_460F8` 依此与
    /// 精灵位数（16/8）选择 6 个 blit 例程之一。
    pub f4: u32,
    /// 控件类型（`ctrl[6]`，+0x18）。实测 95% 为 0；非 0 取值 21..1400
    /// （100/200/300/…/900/1000/1100/1200/1300/1400 分档），语义 S3.3 起结合场景逻辑解释。
    pub f5: u32,
}

impl MapControl {
    /// 单条记录占用的 u32 字段数。
    pub const FIELDS: usize = 6;

    /// 控件数所在的下标（`words[4]`，F-306）。
    pub const COUNT_WORD: usize = HEADER_WORDS - 1;

    /// 按 [`MapControl::FIELDS`] 个一组切出全部记录；字流不整除时返回错误。
    ///
    /// # Errors
    ///
    /// 字流长度不足（`UnexpectedEof`）或记录数与字流长度不一致（`TrailingBytes`）时返回错误。
    pub fn parse_words(path: &str, words: &[u32]) -> Result<Vec<Self>> {
        if words.len() < HEADER_WORDS {
            return Err(Error::parse(
                path,
                16,
                ParseKind::UnexpectedEof { field: "map body", needed: 1 },
            ));
        }
        let count = words[HEADER_WORDS - 1] as usize;
        let need = HEADER_WORDS + count * Self::FIELDS;
        if words.len() != need {
            return Err(Error::parse(
                path,
                16 + words.len() * 4,
                ParseKind::TrailingBytes { count: words.len().saturating_sub(need) },
            ));
        }
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let w = &words[HEADER_WORDS + i * Self::FIELDS..][..Self::FIELDS];
            out.push(MapControl { f0: w[0], f1: w[1], f2: w[2], f3: w[3], f4: w[4], f5: w[5] });
        }
        Ok(out)
    }

    /// 一条记录的原始 6 个字（按文件内顺序）。
    #[must_use]
    pub fn words(&self) -> [u32; Self::FIELDS] {
        [self.f0, self.f1, self.f2, self.f3, self.f4, self.f5]
    }
}

define_format! {
    /// `.Map` 固定头部（16 字节）。
    pub struct MapHeader {
        /// 场景宽（UI 场景恒为 176，地形图为 20 —— F-301，故此处不写死）。
        pub width: u32,
        /// 场景高（UI 场景恒为 220，地形图为 20）。
        pub height: u32,
        /// 标志位（实测 0 或 1，语义待定）。
        pub flags: u32,
        /// 保留/变体字段（实测 0 或 0xFFFFFFFF，语义待定）。
        pub reserved: u32,
    }
}

/// 一个 `.Map` 场景文件：头部 + u32 字流。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapScene {
    /// 固定头部。
    pub header: MapHeader,
    /// 头部之后的 u32 字流（控件/场景记录，语义待逆）。
    pub words: Vec<u32>,
}

impl MapScene {
    /// 从字节流解析（头部 + u32 字流，逐字透传，不解释记录级布局）。
    ///
    /// 通用入口：UI 场景与地形图（`glmapm*.Map`）都走这里，二者字流布局不同，
    /// 记录级语义由各自的消费方解释：
    /// - UI 场景 → [`MapControl::parse_words`]（`5 + 6*count`，F-306）；
    /// - 地形图 → [`terrain::TerrainMap`]（stride=9 三段流，F-307）。
    ///
    /// # Errors
    ///
    /// 长度不足 16 字节或剩余长度不是 4 的倍数时返回错误。
    pub fn parse(path: &str, buf: &[u8]) -> Result<Self> {
        let header = MapHeader::read(
            path,
            buf.get(..16).ok_or_else(|| {
                Error::parse(path, 0, ParseKind::UnexpectedEof { field: "map header", needed: 16 })
            })?,
        )?;
        let body = buf.get(16..).ok_or_else(|| {
            Error::parse(path, 16, ParseKind::UnexpectedEof { field: "map body", needed: 1 })
        })?;
        if body.len() % 4 != 0 {
            return Err(Error::parse(
                path,
                16 + body.len() - body.len() % 4,
                ParseKind::TrailingBytes { count: body.len() % 4 },
            ));
        }
        let mut words = Vec::with_capacity(body.len() / 4);
        for chunk in body.chunks_exact(4) {
            words.push(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Ok(Self { header, words })
    }

    /// 逐字节还原为原始文件内容（roundtrip 保证 0 未知字节）。
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.words.len() * 4);
        out.extend_from_slice(&self.header.width.to_le_bytes());
        out.extend_from_slice(&self.header.height.to_le_bytes());
        out.extend_from_slice(&self.header.flags.to_le_bytes());
        out.extend_from_slice(&self.header.reserved.to_le_bytes());
        for w in &self.words {
            out.extend_from_slice(&w.to_le_bytes());
        }
        out
    }

    /// 导出为 JSON（纯 ASCII，无第三方依赖）。
    #[must_use]
    pub fn to_json(&self, name: &str) -> String {
        let mut s = String::with_capacity(64 + self.words.len() * 12);
        s.push_str("{\n  \"name\": \"");
        s.push_str(name);
        s.push_str("\",\n  \"width\": ");
        s.push_str(&self.header.width.to_string());
        s.push_str(",\n  \"height\": ");
        s.push_str(&self.header.height.to_string());
        s.push_str(",\n  \"flags\": ");
        s.push_str(&self.header.flags.to_string());
        s.push_str(",\n  \"reserved\": ");
        s.push_str(&self.header.reserved.to_string());
        s.push_str(",\n  \"words\": [");
        for (i, w) in self.words.iter().enumerate() {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 16B 头（176×220, flags=1, reserved=114）+ 表头 4 字 + count=1 + 一条控件记录。
    fn sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&176u32.to_le_bytes());
        v.extend_from_slice(&220u32.to_le_bytes());
        v.extend_from_slice(&1u32.to_le_bytes());
        v.extend_from_slice(&114u32.to_le_bytes());
        for w in [176u32, 219, 1, 0, 1, 1, 105, 6, 12, 1, 0] {
            v.extend_from_slice(&w.to_le_bytes());
        }
        v
    }

    #[test]
    fn parses_header_and_words() {
        let s = MapScene::parse("t.Map", &sample());
        assert!(s.is_ok());
        let m = s.ok();
        if let Some(m) = m {
            assert_eq!(m.header.width, 176);
            assert_eq!(m.header.height, 220);
            assert_eq!(m.header.flags, 1);
            assert_eq!(m.header.reserved, 114);
            assert_eq!(m.words.len(), 11);
            assert_eq!(m.words[0], 176);
        }
    }

    #[test]
    fn roundtrip_is_byte_identical() {
        let raw = sample();
        let m = MapScene::parse("t.Map", &raw);
        let m = m.ok();
        if let Some(m) = m {
            assert_eq!(m.to_bytes(), raw);
        }
    }

    #[test]
    fn records_view_yields_exactly_one_control() {
        let m = MapScene::parse("t.Map", &sample());
        let Ok(m) = m else { unreachable!("parse of valid sample must succeed") };
        let recs = MapControl::parse_words("t.Map", &m.words);
        assert!(recs.is_ok());
        let Ok(recs) = recs else { unreachable!("record parse of valid data must succeed") };
        assert_eq!(recs.len(), 1);
        let r = &recs[0];
        assert_eq!(r.f0, 1);
        assert_eq!(r.f1, 105);
        assert_eq!(r.f2, 6);
        assert_eq!(r.f3, 12);
        assert_eq!(r.f4, 1);
        assert_eq!(r.f5, 0);
        assert_eq!(r.words(), [1, 105, 6, 12, 1, 0]);
    }

    #[test]
    fn rejects_inconsistent_record_count() {
        // 通用 parse 不透传记录级布局；记录整除由 MapControl::parse_words 负责
        let mut raw = sample();
        raw.truncate(16 + (HEADER_WORDS + 4) * 4);
        let scene = MapScene::parse("t.Map", &raw);
        assert!(scene.is_ok(), "通用字流解析不应拒绝合法字节流");
        let scene = scene.ok();
        let Some(scene) = scene else { unreachable!("parse of valid sample must succeed") };
        let e = MapControl::parse_words("t.Map", &scene.words);
        assert!(e.is_err(), "短记录流必须报错");
    }

    #[test]
    fn rejects_ragged_body() {
        let mut raw = sample();
        raw.push(0);
        assert!(MapScene::parse("t.Map", &raw).is_err());
    }

    #[test]
    fn json_contains_header_fields() {
        let m = MapScene::parse("t.Map", &sample());
        let m = m.ok();
        if let Some(m) = m {
            let j = m.to_json("t");
            assert!(j.contains("\"width\": 176"), "{j}");
            assert!(j.contains("\"height\": 220"), "{j}");
            assert!(j.contains("\"words\": [ 176, 219, 1, 0, 1"), "{j}");
        }
    }

    #[test]
    fn preserves_non_ui_dimensions_in_words_only() {
        // 20×20 地形图头 + 合法 UI 控件字流：头部不拦，记录整除校验仍生效
        let mut raw = sample();
        raw[0..4].copy_from_slice(&20u32.to_le_bytes());
        raw[4..8].copy_from_slice(&20u32.to_le_bytes());
        let m = MapScene::parse("t.Map", &raw);
        let m = m.ok();
        if let Some(m) = m {
            assert_eq!((m.header.width, m.header.height), (20, 20));
            assert_eq!(m.to_bytes(), raw);
        }
    }

    #[test]
    fn terrain_glmapm_words_are_not_ui_control_layout() {
        // glmapm01 实测字流：`5 + 6*count` 不成立（words[4]=0 但后面还有 4278 字），
        // 证明本模块只适用 UI 场景，地形图必须走 terrain 模块
        let raw =
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../data/glmapm01.Map"));
        let raw = raw.as_deref().unwrap_or(&[]);
        if raw.len() < 16 {
            return; // 无数据文件时（纯单测环境）跳过
        }
        let scene = MapScene::parse("glmapm01.Map", raw);
        let scene = scene.ok();
        let Some(scene) = scene else {
            return;
        };
        let e = MapControl::parse_words("glmapm01.Map", &scene.words);
        assert!(e.is_err(), "地形图字流不得按 UI 控件布局解释");
    }
}
