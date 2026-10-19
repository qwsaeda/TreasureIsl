//! 声明式格式解析宏（`docs/style.md` §5.4）。
//!
//! 三个宏的分工：
//! - `define_format!`：字段表 → `read` / `write` / `to_bytes`。`read` 末尾强制
//!   `已消费 == 输入长度`，「0 未知字节」(P0-2) 由构造保证；
//! - `parse_err!`：解析代码构造错误的唯一入口（保证 path/offset 必被填充）；
//! - `roundtrip_all!`：批量往返测试 (P0-3)，读→写→逐字节相等→再读。
//!
//! 字段类型只需实现 [`crate::Field`]；`u8..u64`、有符号整数、`bool`、
//! `[u8; N]` 已内置，小端序。

/// 由字段表生成 `read` / `write` / `to_bytes`。
///
/// ```text
/// define_format! {
///     /// 场景头（示例）。
///     pub struct SceneHeader {
///         magic: [u8; 4] = [b'S', b'C', b'N', b'E'],
///         count: u16,
///         locked: bool,
///     }
/// }
/// ```
///
/// - 字段类型必须实现 [`crate::Field`]；
/// - `= 表达式` 为**恒定值约束**（魔数/常量字段），表达式类型必须与字段
///   完全一致（字节数组请写 `[b'A', b'B']` 这样的数组字面量）；
///   `==` 不能跟在 `ty` 片段之后（宏 follow-set 限制），故用 `=`；
/// - 生成的 `read` 在结构体之外还有残留字节时报 `TrailingBytes`；
/// - 生成的类型带 `Debug + Clone + PartialEq + Eq`。
///
/// # Errors
///
/// 输入不足（`UnexpectedEof`）、约束不符（`InvalidValue`）、
/// 结构之后仍有残留字节（`TrailingBytes`）时返回错误，
/// 一律携带出错文件路径与绝对偏移。
#[macro_export]
macro_rules! define_format {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$fmeta:meta])*
                $fvis:vis $fname:ident : $fty:ty $(= $expected:expr)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        $vis struct $name {
            $(
                $(#[$fmeta])*
                $fvis $fname : $fty
            ),*
        }

        impl $name {
            /// 解析整段字节流；结构之外的残留字节视为错误（0 未知字节）。
            ///
            /// # Errors
            ///
            /// 输入不足、字段约束不符或仍有未消费字节时返回带路径与偏移
            /// 的解析错误。
            $vis fn read(
                path: impl Into<std::path::PathBuf>,
                buf: &[u8],
            ) -> $crate::Result<Self> {
                let path: std::path::PathBuf = path.into();
                let mut off = 0usize;
                $(
                    let $fname = {
                        #[allow(unused_variables)]
                        let start = off;
                        let value = match <$fty as $crate::Field>::read(
                            buf,
                            &mut off,
                            &path,
                            stringify!($fname),
                        ) {
                            Ok(value) => value,
                            Err(err) => return Err(err),
                        };
                        $(
                            let expected: $fty = $expected;
                            if value != expected {
                                return Err($crate::parse_err!(
                                    path.as_path(),
                                    start,
                                    $crate::ParseKind::InvalidValue {
                                        field: stringify!($fname),
                                    }
                                ));
                            }
                        )?
                        value
                    };
                )*
                if off != buf.len() {
                    return Err($crate::parse_err!(
                        path.as_path(),
                        off,
                        $crate::ParseKind::TrailingBytes { count: buf.len() - off }
                    ));
                }
                Ok(Self { $($fname),* })
            }

            /// 按与 `read` 同源的字段顺序写入 `out`。
            $vis fn write(&self, out: &mut Vec<u8>) {
                $( <$fty as $crate::Field>::write(&self.$fname, out); )*
            }

            /// `write` 的便捷入口：序列化到新缓冲。
            #[must_use]
            $vis fn to_bytes(&self) -> Vec<u8> {
                let mut out = Vec::new();
                self.write(&mut out);
                out
            }
        }
    };
}

/// 构造带路径与偏移的解析错误（解析代码禁止手写 `Error::Parse`）。
///
/// `parse_err!(path, offset, kind)` → `crate::Error::Parse`，
/// 保证 §5.5 要求的 `path + offset` 永不缺失。
#[macro_export]
macro_rules! parse_err {
    ($path:expr, $offset:expr, $kind:expr $(,)?) => {
        $crate::Error::parse($path, $offset, $kind)
    };
}

/// 为给定类型生成批量往返测试：每个样本 `read → write → 逐字节相等 → 再 read`。
///
/// ```text
/// roundtrip_all!(SceneHeader, [ b"SCNE\x02\x00\x01", &[0u8; 4] ]);
/// roundtrip_all!(my_cases => SceneHeader, [ &[1, 2, 3] ]);
/// ```
///
/// 样本必须是 `&[u8]`。省略名字时测试模块固定为 `roundtrip_tests`，
/// 同一模块内只能用一次；多次使用请用带名字的形式。
#[macro_export]
macro_rules! roundtrip_all {
    ($modname:ident => $ty:ty, [ $( $sample:expr ),* $(,)? ]) => {
        #[cfg(test)]
        mod $modname {
            use super::*;

            #[test]
            fn roundtrip_all_cases() {
                let samples: &[&[u8]] = &[ $( $sample ),* ];
                let mut failures: Vec<String> = Vec::new();
                for (index, sample) in samples.iter().enumerate() {
                    let parsed = match <$ty>::read("roundtrip", sample) {
                        Ok(parsed) => parsed,
                        Err(err) => {
                            failures.push(format!("case {index}: read failed: {err}"));
                            continue;
                        }
                    };
                    let bytes = parsed.to_bytes();
                    if bytes.as_slice() != *sample {
                        failures.push(format!(
                            "case {index}: wrote {bytes:02x?}, want {sample:02x?}"
                        ));
                        continue;
                    }
                    if let Err(err) = <$ty>::read("roundtrip", &bytes) {
                        failures.push(format!("case {index}: re-read failed: {err}"));
                    }
                }
                assert!(failures.is_empty(), "roundtrip failures: {failures:?}");
            }
        }
    };
    ($ty:ty, [ $( $sample:expr ),* $(,)? ]) => {
        $crate::roundtrip_all!(roundtrip_tests => $ty, [ $( $sample ),* ]);
    };
}

#[cfg(test)]
mod tests {
    use crate::error::{Error, ParseKind};

    define_format! {
        struct ToyHeader {
            magic: [u8; 4] = *b"TIT1",
            count: u16,
            locked: bool,
        }
    }

    const SAMPLE: &[u8] = &[b'T', b'I', b'T', b'1', 0x02, 0x00, 0x01];

    roundtrip_all!(ToyHeader, [SAMPLE, &[b'T', b'I', b'T', b'1', 0x00, 0x00, 0x00]]);

    #[test]
    fn read_parses_fields_in_order() {
        let outcome = ToyHeader::read("toy.bin", SAMPLE);
        assert!(outcome.is_ok(), "合法样本必须解析成功");
        if let Ok(value) = outcome {
            assert_eq!(value.magic, [b'T', b'I', b'T', b'1']);
            assert_eq!(value.count, 2);
            assert!(value.locked);
        }
    }

    #[test]
    fn trailing_byte_is_rejected_with_offset() {
        let mut buf = SAMPLE.to_vec();
        buf.push(0xFF);
        let outcome = ToyHeader::read("toy.bin", &buf);
        assert!(outcome.is_err(), "残留 1 字节必须被拒绝");
        if let Err(err) = outcome {
            let text = err.to_string();
            assert!(
                matches!(
                    err,
                    Error::Parse { offset: 7, kind: ParseKind::TrailingBytes { count: 1 }, .. }
                ),
                "{text}"
            );
            assert!(text.contains("toy.bin"), "{text}");
        }
    }

    #[test]
    fn short_input_reports_field_and_missing_bytes() {
        let outcome = ToyHeader::read("toy.bin", &SAMPLE[..3]);
        assert!(outcome.is_err(), "缺 1 字节必须失败");
        if let Err(err) = outcome {
            let text = err.to_string();
            assert!(text.contains("`magic`"), "{text}");
            assert!(text.contains("1 more bytes"), "{text}");
        }
    }

    #[test]
    fn magic_mismatch_is_rejected() {
        let mut buf = SAMPLE.to_vec();
        buf[1] = b'X';
        let outcome = ToyHeader::read("toy.bin", &buf);
        assert!(outcome.is_err(), "魔数不符必须失败");
        if let Err(err) = outcome {
            let text = err.to_string();
            assert!(
                matches!(
                    err,
                    Error::Parse {
                        offset: 0,
                        kind: ParseKind::InvalidValue { field: "magic" },
                        ..
                    }
                ),
                "{text}"
            );
        }
    }

    #[test]
    fn parse_err_carries_path_and_hex_offset() {
        let err = parse_err!("x.dat", 0x10, ParseKind::BadMagic { expected: b"PK" });
        let text = err.to_string();
        assert!(text.contains("x.dat"), "{text}");
        assert!(text.contains("0x0010"), "{text}");
        assert!(text.contains("bad magic"), "{text}");
    }

    #[test]
    fn write_is_source_identical_to_read() {
        let outcome = ToyHeader::read("toy.bin", SAMPLE);
        assert!(outcome.is_ok(), "样本必须合法");
        if let Ok(value) = outcome {
            let bytes = value.to_bytes();
            assert_eq!(bytes.as_slice(), SAMPLE);
        }
    }
}
