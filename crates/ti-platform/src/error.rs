//! 平台层错误（`docs/style.md` §5.5 的统一模板）。

use std::path::PathBuf;

/// 平台后端操作失败。
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// 底层 IO 失败。
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// 素材加载/访问失败。
    #[error("asset {path:?}: {reason}")]
    Asset {
        /// 素材路径或逻辑名。
        path: PathBuf,
        /// 失败原因（实现内部定义，不透传平台原始错误码）。
        reason: &'static str,
    },
    /// 后端尚未初始化（例如 wasm 音频在用户手势前被调用）。
    #[error("backend not ready: {0}")]
    NotReady(&'static str),
}

/// 平台层统一返回类型。
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::{Error, PathBuf};

    #[test]
    fn asset_error_shows_path_and_reason() {
        let err = Error::Asset { path: PathBuf::from("ui.pak"), reason: "missing index stream" };
        let text = err.to_string();
        assert!(text.contains("ui.pak"), "{text}");
        assert!(text.contains("missing index stream"), "{text}");
    }

    #[test]
    fn not_ready_error_is_renderable() {
        assert_eq!(Error::NotReady("audio").to_string(), "backend not ready: audio");
    }
}
