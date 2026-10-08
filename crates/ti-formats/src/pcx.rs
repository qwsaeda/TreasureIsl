//! PCX（8bpp / 1 平面 / RLE）解码器（S1.3，P0-4）。
//!
//! 原版 `glpics.pak` / `ui.pak` 内的图片全部是 8bpp 单平面 RLE PCX（F-205）。
//! 独立参考实现见 `scripts/pcx_ref.py`，两者必须逐像素一致。
//!
//! **头字段非标准**（实测 ui.pak 首条）：`NPlanes@0x3D = 0`、`BytesPerLine@0x3E = 0`，
//! 真正的行跨度在 `0x42`（=34，恰等于 width）。因此行跨度不靠读字段猜，
//! 而是用强约束反解：**RLE 必须恰好消费到文件尾**。

use crate::error::{Error, ParseKind};
use crate::lucency::Package;

/// PCX 文件头长度。
pub const HEADER_LEN: usize = 128;

/// 解码后的图像：8bpp 索引调色板图。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// 宽（像素）。
    pub width: u16,
    /// 高（像素）。
    pub height: u16,
    /// 行主序索引像素，长度 = `width * height`。
    pub pixels: Vec<u8>,
    /// 256 色调色板（RGB 三元组，长度 768）；无尾部调色板时用 EGA 默认表。
    pub palette: [[u8; 3]; 256],
}

/// 标准 16 色 EGA 调色板，其余按灰阶。
#[allow(clippy::cast_possible_truncation, clippy::needless_pass_by_value)]
fn ega_palette() -> [[u8; 3]; 256] {
    const BASE: [[u8; 3]; 16] = [
        [0, 0, 0],
        [0, 0, 170],
        [0, 170, 0],
        [0, 170, 170],
        [170, 0, 0],
        [170, 0, 170],
        [170, 85, 0],
        [170, 170, 170],
        [85, 85, 85],
        [85, 85, 255],
        [85, 255, 85],
        [85, 255, 255],
        [255, 85, 85],
        [255, 85, 255],
        [255, 255, 85],
        [255, 255, 255],
    ];
    let mut p = [[0u8; 3]; 256];
    let mut i: u16 = 0;
    while i < 256 {
        p[usize::from(i)] = if i < 16 { BASE[usize::from(i)] } else { [i as u8, i as u8, i as u8] };
        i += 1;
    }
    p
}

/// RLE 解码到 `need` 字节，返回 `(数据, 已消费输入字节数)`；`None` 表示输入不足/非法。
fn rle_decode(data: &[u8], need: usize) -> Option<(Vec<u8>, usize)> {
    let mut raw = Vec::with_capacity(need);
    let mut i = HEADER_LEN;
    while raw.len() < need {
        let b = *data.get(i)?;
        i += 1;
        if b & 0xC0 == 0xC0 {
            let count = usize::from(b & 0x3F);
            if count == 0 {
                return None;
            }
            let value = *data.get(i)?;
            i += 1;
            raw.extend(std::iter::repeat_n(value, count));
        } else {
            raw.push(b);
        }
    }
    Some((raw, i))
}

impl Image {
    /// 解码 8bpp 单平面 RLE PCX。
    ///
    /// # Errors
    ///
    /// 头非法（厂商/编码/位深不符、宽高为 0）或扫描线数据不足时返回带偏移的错误。
    pub fn decode(path: &str, data: &[u8]) -> Result<Self, Error> {
        let bad = |off: usize, kind: ParseKind| Error::parse(path, off, kind);
        let head = data.get(..HEADER_LEN).ok_or_else(|| {
            bad(0, ParseKind::UnexpectedEof { field: "pcx header", needed: HEADER_LEN })
        })?;

        if head[0] != 0x0A {
            return Err(bad(0, ParseKind::BadMagic { expected: b"\x0a" }));
        }
        if head[2] != 1 {
            return Err(bad(2, ParseKind::InvalidValue { field: "pcx encoding" }));
        }
        if head[3] != 8 {
            return Err(bad(3, ParseKind::InvalidValue { field: "pcx bpp" }));
        }

        let word = |off: usize| u16::from_le_bytes([head[off], head[off + 1]]);
        let width = word(8) - word(4) + 1;
        let height = word(10) - word(6) + 1;
        if width == 0 || height == 0 {
            return Err(bad(4, ParseKind::InvalidValue { field: "pcx dimension" }));
        }

        let mut cands: Vec<u16> = vec![width, word(0x3E), u16::from(head[0x42])];
        cands.sort_unstable();
        cands.dedup();
        cands.retain(|v| *v >= width);

        // 强约束：RLE 恰好消费到文件尾
        for stride in &cands {
            let need = usize::from(*stride) * usize::from(height);
            if let Some((raw, consumed)) = rle_decode(data, need) {
                if consumed == data.len() {
                    return Self::build(width, height, *stride, &raw, data, path);
                }
            }
        }

        let stride = cands.first().copied().unwrap_or(width);
        let need = usize::from(stride) * usize::from(height);
        let raw = rle_decode(data, need).map(|(raw, _)| raw).ok_or_else(|| {
            bad(HEADER_LEN, ParseKind::UnexpectedEof { field: "pcx rle", needed: need })
        })?;
        Self::build(width, height, stride, &raw, data, path)
    }

    /// 按跨度切出像素行并挂上调色板。
    fn build(
        width: u16,
        height: u16,
        stride: u16,
        raw: &[u8],
        data: &[u8],
        path: &str,
    ) -> Result<Self, Error> {
        let w = usize::from(width);
        let stride_us = usize::from(stride);
        let mut pixels = Vec::with_capacity(w * usize::from(height));
        for y in 0..usize::from(height) {
            let start = y * stride_us;
            let row = raw.get(start..start + w).ok_or_else(|| {
                Error::parse(path, start, ParseKind::UnexpectedEof { field: "pcx row", needed: w })
            })?;
            pixels.extend_from_slice(row);
        }
        Ok(Self { width, height, pixels, palette: read_palette(data) })
    }

    /// 像素总数（`width * height`）。
    #[must_use]
    pub fn len(&self) -> usize {
        usize::from(self.width) * usize::from(self.height)
    }

    /// 是否无像素（宽高已校验，正常恒为 false）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }
}

/// 读取尾部 256 色调色板（`0x0C` 标记 + 768 字节 RGB）；缺失时用 EGA 默认表。
#[must_use]
pub fn read_palette(data: &[u8]) -> [[u8; 3]; 256] {
    let n = data.len();
    if n >= 769 && data[n - 769] == 0x0C {
        let tail = &data[n - 768..];
        let mut p = ega_palette();
        for (i, slot) in p.iter_mut().enumerate() {
            slot[0] = tail[i * 3];
            slot[1] = tail[i * 3 + 1];
            slot[2] = tail[i * 3 + 2];
        }
        p
    } else {
        ega_palette()
    }
}

/// 从容器的第 `index` 条解压并解码为 PCX 图像。
///
/// # Errors
///
/// 容器解压失败或 PCX 解析失败时返回错误。
pub fn decode_from_package(
    path: &str,
    pkg: &Package,
    buf: &[u8],
    index: usize,
) -> Result<Image, Error> {
    let data = pkg.read_entry(buf, index)?;
    Image::decode(path, &data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::cast_possible_truncation)]
    fn build_pcx(width: u16, height: u16, stride: u16, rows: &[&[u8]]) -> Vec<u8> {
        let mut out = vec![0u8; HEADER_LEN];
        out[0] = 0x0A;
        out[1] = 5;
        out[2] = 1;
        out[3] = 8;
        out[4..6].copy_from_slice(&0u16.to_le_bytes());
        out[6..8].copy_from_slice(&0u16.to_le_bytes());
        out[8..10].copy_from_slice(&(width - 1).to_le_bytes());
        out[10..12].copy_from_slice(&(height - 1).to_le_bytes());
        out[0x3D] = 0;
        out[0x3E..0x40].copy_from_slice(&0u16.to_le_bytes());
        out[0x42] = stride as u8;
        for row in rows {
            out.extend_from_slice(row);
        }
        out
    }

    #[test]
    fn decodes_literal_rows_with_padding_stride() {
        let pcx = build_pcx(2, 2, 4, &[&[1, 2, 9, 9], &[3, 4, 9, 9]]);
        let img = Image::decode("t.pcx", &pcx);
        assert!(img.is_ok());
        let i = img.ok();
        if let Some(i) = i {
            assert_eq!(i.width, 2);
            assert_eq!(i.height, 2);
            assert_eq!(i.pixels, vec![1, 2, 3, 4]);
        }
    }

    #[test]
    fn decodes_rle_runs() {
        // C3 03 = 重复 3 次的 0x03；随后 0x41 为字面量
        let mut pcx = build_pcx(4, 1, 4, &[]);
        pcx.extend_from_slice(&[0xC3, 0x03, 0x41]);
        let img = Image::decode("t.pcx", &pcx);
        let i = img.ok();
        if let Some(i) = i {
            assert_eq!(i.pixels, vec![3, 3, 3, 0x41]);
        }
    }

    #[test]
    fn rejects_bad_magic_and_depth() {
        let mut pcx = build_pcx(2, 1, 2, &[&[1, 2]]);
        pcx[0] = 0;
        assert!(Image::decode("t.pcx", &pcx).is_err());

        let mut pcx2 = build_pcx(2, 1, 2, &[&[1, 2]]);
        pcx2[3] = 4;
        assert!(Image::decode("t.pcx", &pcx2).is_err());
    }

    #[test]
    fn rejects_truncated_scanlines() {
        let pcx = build_pcx(8, 4, 8, &[&[1, 2]]);
        assert!(Image::decode("t.pcx", &pcx).is_err());
    }

    #[test]
    fn trailing_palette_overrides_default() {
        let mut pcx = build_pcx(1, 1, 1, &[&[9]]);
        pcx.push(0x0C);
        for c in 0..=255u8 {
            pcx.push(c);
            pcx.push(c);
            pcx.push(c);
        }
        let img = Image::decode("t.pcx", &pcx);
        let i = img.ok();
        if let Some(i) = i {
            assert_eq!(i.palette[5], [5, 5, 5]);
            assert_eq!(i.palette[200], [200, 200, 200]);
        }
    }
}
