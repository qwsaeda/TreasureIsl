//! 存储：键值持久化抽象（桌面文件 / `wasm` `IndexedDB` / `Android` 内部存储）。
//!
//! 三端差异只在**后端**，读写语义完全一致，因此这里定义 trait + 可测的内存实现，
//! 并给出键名规范（ASCII、大小写不敏感、禁止路径分隔符），保证三端互不串档。

use crate::error::{Error, Result};

/// 键名规范检查：只允许 ASCII 字母数字与 `_`、`.`、`-`，长度 1..=64。
///
/// 三端统一用同一套键名（存档/设置/事件日志），避免大小写与路径差异导致丢档。
#[must_use]
pub fn validate_key(key: &str) -> bool {
    if key.is_empty() || key.len() > 64 {
        return false;
    }
    key.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-'))
}

/// 键值存储后端（对齐 `crate::backend::Storage`）。
pub trait Backend {
    /// 读取一个键；不存在返回 `None`。
    ///
    /// # Errors
    ///
    /// 底层 IO 失败时返回错误。
    fn load(&mut self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 写入一个键。
    ///
    /// # Errors
    ///
    /// 键名非法或底层 IO 失败时返回错误。
    fn store(&mut self, key: &str, value: &[u8]) -> Result<()>;

    /// 删除一个键；不存在视为成功。
    ///
    /// # Errors
    ///
    /// 键名非法或底层 IO 失败时返回错误。
    fn remove(&mut self, key: &str) -> Result<()>;

    /// 列出全部键名（升序）。
    ///
    /// # Errors
    ///
    /// 底层 IO 失败时返回错误。
    fn keys(&mut self) -> Result<Vec<String>>;
}

/// 内存实现：测试与 headless 回归用（也是 wasm/Android 后端的语义基准）。
#[derive(Debug, Default)]
pub struct MemoryStore {
    map: std::collections::BTreeMap<String, Vec<u8>>,
}

impl MemoryStore {
    /// 新建空存储。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 存储条目数。
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// 已用字节数（存档体积上限校验用）。
    #[must_use]
    pub fn bytes_used(&self) -> usize {
        self.map.values().map(Vec::len).sum()
    }
}

impl Backend for MemoryStore {
    fn load(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        if !validate_key(key) {
            return Err(Error::NotReady("invalid storage key"));
        }
        Ok(self.map.get(key).cloned())
    }

    fn store(&mut self, key: &str, value: &[u8]) -> Result<()> {
        if !validate_key(key) {
            return Err(Error::NotReady("invalid storage key"));
        }
        self.map.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        if !validate_key(key) {
            return Err(Error::NotReady("invalid storage key"));
        }
        self.map.remove(key);
        Ok(())
    }

    fn keys(&mut self) -> Result<Vec<String>> {
        Ok(self.map.keys().cloned().collect())
    }
}

/// 桌面实现：以 `dir` 为根，每个键一个文件。
#[derive(Debug)]
pub struct FileStore {
    dir: std::path::PathBuf,
}

impl FileStore {
    /// 新建文件存储；目录不存在会在首次写入时创建。
    ///
    /// # Errors
    ///
    /// 键名非法时返回错误。
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Result<Self> {
        let dir = dir.into();
        let probe = dir.join(".probe");
        let name = probe
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or(Error::NotReady("invalid storage dir"))?;
        if !validate_key(name) {
            return Err(Error::NotReady("invalid storage dir"));
        }
        Ok(Self { dir })
    }

    /// 存储根目录。
    #[must_use]
    pub fn dir(&self) -> &std::path::Path {
        &self.dir
    }

    fn path_of(&self, key: &str) -> Result<std::path::PathBuf> {
        if !validate_key(key) {
            return Err(Error::NotReady("invalid storage key"));
        }
        Ok(self.dir.join(key))
    }
}

impl Backend for FileStore {
    fn load(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        let p = self.path_of(key)?;
        match std::fs::read(&p) {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn store(&mut self, key: &str, value: &[u8]) -> Result<()> {
        let p = self.path_of(key)?;
        std::fs::create_dir_all(&self.dir).map_err(Error::Io)?;
        std::fs::write(&p, value).map_err(Error::Io)?;
        Ok(())
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        let p = self.path_of(key)?;
        match std::fs::remove_file(&p) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::Io(e)),
        }
    }

    fn keys(&mut self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let rd = match std::fs::read_dir(&self.dir) {
            Ok(rd) => rd,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(Error::Io(e)),
        };
        for entry in rd.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if validate_key(name) {
                    out.push(name.to_string());
                }
            }
        }
        out.sort();
        Ok(out)
    }
}
