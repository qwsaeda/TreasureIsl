//! 字段级读写原语：`define_format!` 生成的代码只调用本 trait，
//! 因此宏本身不必关心具体类型。
//!
//! 约定：字节序一律小端（目标机 `WinCE` ARM）；所有失败路径都返回
//! [`crate::Error::Parse`]（携带路径、偏移与字段名），不存在任何 panic。

use std::path::Path;

use crate::error::{Error, ParseKind, Result};

/// 一个格式字段的读写能力。
///
/// 为 `u8/u16/u32/u64`、对应有符号类型、`bool`、`[u8; N]` 提供实现；
/// 新格式若需要别的字段类型，先实现本 trait，**不要**绕过宏手写读写。
pub trait Field: Sized {
    /// 从 `buf[*off..]` 读取一个字段并把 `*off` 推进到字段之后。
    ///
    /// # Errors
    ///
    /// - 输入剩余字节不足：[`ParseKind::UnexpectedEof`]（字段名 + 还差字节数）；
    /// - 字段取值违反该类型自身的编码约定（如 `bool` 收到 0/1 之外的值）：
    ///   [`ParseKind::InvalidValue`]。
    ///
    /// 两种错误都带出错文件的路径与该字段起始处的绝对偏移。
    fn read(buf: &[u8], off: &mut usize, path: &Path, field: &'static str) -> Result<Self>;

    /// 按与 [`Field::read`] 严格同源的顺序把字段追加到 `out`。
    fn write(&self, out: &mut Vec<u8>);
}

/// 取出恰好 `need` 字节；不足则返回带上下文的 EOF 错误。
fn take<'a>(
    buf: &'a [u8],
    off: &mut usize,
    need: usize,
    path: &Path,
    field: &'static str,
) -> Result<&'a [u8]> {
    let start = *off;
    let available = buf.len().saturating_sub(start);
    if available < need {
        return Err(Error::parse(
            path,
            start,
            ParseKind::UnexpectedEof { field, needed: need - available },
        ));
    }
    let end = start + need;
    *off = end;
    Ok(&buf[start..end])
}

macro_rules! impl_field_int {
    ($($ty:ty => $len:expr),* $(,)?) => {
        $(
            impl Field for $ty {
                fn read(
                    buf: &[u8],
                    off: &mut usize,
                    path: &Path,
                    field: &'static str,
                ) -> Result<Self> {
                    let slice = take(buf, off, $len, path, field)?;
                    let mut bytes = [0u8; $len];
                    bytes.copy_from_slice(slice);
                    Ok(<$ty>::from_le_bytes(bytes))
                }

                fn write(&self, out: &mut Vec<u8>) {
                    out.extend_from_slice(&self.to_le_bytes());
                }
            }
        )*
    };
}

impl_field_int!(u8 => 1, u16 => 2, u32 => 4, u64 => 8, i8 => 1, i16 => 2, i32 => 4, i64 => 8);

impl<const N: usize> Field for [u8; N] {
    fn read(buf: &[u8], off: &mut usize, path: &Path, field: &'static str) -> Result<Self> {
        let slice = take(buf, off, N, path, field)?;
        let mut bytes = [0u8; N];
        bytes.copy_from_slice(slice);
        Ok(bytes)
    }

    fn write(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self);
    }
}

impl Field for bool {
    fn read(buf: &[u8], off: &mut usize, path: &Path, field: &'static str) -> Result<Self> {
        let start = *off;
        let slice = take(buf, off, 1, path, field)?;
        match slice[0] {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::parse(path, start, ParseKind::InvalidValue { field })),
        }
    }

    fn write(&self, out: &mut Vec<u8>) {
        out.push(u8::from(*self));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = &[0x34, 0x12, 0x78, 0x56, 0x34, 0x12];

    #[test]
    fn integers_read_little_endian_and_advance() {
        let mut off = 0;
        let low = u16::read(SAMPLE, &mut off, Path::new("t"), "low").ok();
        assert_eq!(low, Some(0x1234));
        assert_eq!(off, 2);
        let high = u32::read(SAMPLE, &mut off, Path::new("t"), "high").ok();
        assert_eq!(high, Some(0x1234_5678));
        assert_eq!(off, 6);
    }

    #[test]
    fn write_roundtrips_through_read() {
        let value: u32 = 0xDEAD_BEEF;
        let mut out = Vec::new();
        value.write(&mut out);
        assert_eq!(out, vec![0xEF, 0xBE, 0xAD, 0xDE]);
        let mut off = 0;
        let back = u32::read(&out, &mut off, Path::new("t"), "v").ok();
        assert_eq!(back, Some(value));
    }

    #[test]
    fn eof_error_names_field_and_remaining_bytes() {
        let mut off = 4;
        let outcome = u32::read(SAMPLE, &mut off, Path::new("data/t.bin"), "tail");
        assert!(outcome.is_err(), "只剩 2 字节必须读失败");
        if let Err(err) = outcome {
            let text = err.to_string();
            assert!(text.contains("data/t.bin"), "{text}");
            assert!(text.contains("`tail`"), "{text}");
            assert!(text.contains("2 more bytes"), "{text}");
        }
    }

    #[test]
    fn bool_rejects_non_canonical_encoding() {
        let mut off = 0;
        let outcome = bool::read(&[7], &mut off, Path::new("t"), "flag");
        assert!(outcome.is_err(), "非 0/1 必须失败");
        if let Err(err) = outcome {
            assert!(err.to_string().contains("`flag`"), "{err}");
        }
    }
}
