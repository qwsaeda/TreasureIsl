//! 组装层错误：聚合所有子系统错误，只在入口打印一次。

/// 组装层错误（`docs/style.md` §5.5 顶层聚合）。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// 资源格式层错误。
    #[error(transparent)]
    Formats(#[from] ti_formats::Error),
    /// 平台后端错误。
    #[error(transparent)]
    Platform(#[from] ti_platform::Error),
}

/// 组装层统一返回类型。
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn formats_error_upcasts_without_loss() {
        let inner = ti_formats::Error::parse(
            "data/main.Map",
            0x20,
            ti_formats::ParseKind::UnexpectedEof { field: "records", needed: 4 },
        );
        let outer: Error = inner.into();
        assert!(outer.to_string().contains("data/main.Map"), "{outer}");
        assert!(outer.to_string().contains("0x0020"), "{outer}");
    }

    #[test]
    fn platform_error_upcasts_transparently() {
        let inner = ti_platform::Error::NotReady("audio");
        let outer: Error = inner.into();
        assert!(outer.to_string().contains("audio"), "{outer}");
    }
}
