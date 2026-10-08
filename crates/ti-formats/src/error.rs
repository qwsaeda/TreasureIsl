//! 集中式错误类型（`docs/style.md` §5.5）。
//!
//! 规则：每个 crate 只有一个 `Error`，跨 crate 用 `#[from]` 上抛，
//! 解析类错误**必须**携带 `path + offset + 字段/类别`。

use std::path::PathBuf;

/// 结构性解析失败的类别。
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseKind {
    /// 输入在结构声明完成前结束。
    UnexpectedEof {
        /// 出错的字段名。
        field: &'static str,
        /// 还缺多少字节。
        needed: usize,
    },
    /// 魔数与格式签名不符。
    BadMagic {
        /// 该格式要求的魔数字节。
        expected: &'static [u8],
    },
    /// 字段取值不在允许范围内。
    InvalidValue {
        /// 出问题的字段名。
        field: &'static str,
    },
    /// 声明的结构之后仍有残留字节。
    TrailingBytes {
        /// 残留字节数。
        count: usize,
    },
}

impl std::fmt::Display for ParseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof { field, needed } => write!(
                f,
                "unexpected end of input in field `{field}`, {needed} more bytes required"
            ),
            Self::BadMagic { expected } => write!(f, "bad magic, expected bytes {expected:?}"),
            Self::InvalidValue { field } => write!(f, "invalid value in field `{field}`"),
            Self::TrailingBytes { count } => write!(f, "{count} unconsumed trailing bytes"),
        }
    }
}

/// 格式解析与 IO 的统一错误。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// 结构性解析失败，带文件路径与字节偏移。
    #[error("{path:?}@{offset:#06x}: {kind}")]
    Parse {
        /// 出错文件的路径。
        path: PathBuf,
        /// 出错时的绝对偏移。
        offset: usize,
        /// 失败类别。
        kind: ParseKind,
    },
    /// 底层 IO 失败。
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// 本 crate 的统一返回类型。
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// 构造一个带上下文的解析错误。
    ///
    /// 统一入口：格式解析代码不允许手写 `Error::Parse { .. }`，以保证
    /// `path/offset` 永远被填充。
    #[must_use]
    pub fn parse(path: impl Into<PathBuf>, offset: usize, kind: ParseKind) -> Self {
        Self::Parse { path: path.into(), offset, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_carries_offset_hex_in_display() {
        let err = Error::parse("data/ui.pak", 0x5BE, ParseKind::TrailingBytes { count: 3 });
        let text = err.to_string();
        assert!(text.contains("data/ui.pak"), "{text}");
        assert!(text.contains("0x05be"), "{text}");
        assert!(text.contains("3 unconsumed"), "{text}");
    }

    #[test]
    fn io_error_converts_via_from() {
        let io = std::io::Error::from(std::io::ErrorKind::NotFound);
        let err: Error = io.into();
        assert!(err.to_string().contains("not found"), "{err}");
    }

    #[test]
    fn eof_kind_mentions_needed_bytes() {
        let kind = ParseKind::UnexpectedEof { field: "count", needed: 16 };
        assert!(kind.to_string().contains("`count`"), "{kind}");
        assert!(kind.to_string().contains("16 more bytes"));
    }
}
