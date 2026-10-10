//! FNV 哈希（S8.7 库化：`fnv1a64` 走 `fnv` crate 的 `FnvHasher`，替手写步进）。
//!
//! S8.7 库化（`docs/process/01_architecture.md` §4 + S8.7 判据）：
//! - 旧手写 FNV-1a 降级参照物；本模块切到 `fnv` crate（v1.0.7，标准 FNV-1a 64），
//!   本文件只剩薄封装，无手写步进/常量；
//! - `fnv::FnvHasher::default()` 即 FNV-1a 64（offset basis `0xcbf2_9ce4_8422_2325`，
//!   质数 `0x1000_0000_01b3`，库内 `write` 逐字节推 `h ^= b; h = h * prime`，
//!   与旧手写同序同参），`finish()` 直接返回内部 FNV 原值（非 128-bit 混合），
//!   输出与旧实现逐位等价（`pcx_ref` 对 Python 参考实现回归可验）；
//! - 调用方（manifest 哈希 / `build_assets` 产物校验）签名不变（`fnv1a64(bytes) -> u64`）。
//!
//! 门禁红线：本实现无 `unwrap`/`panic!`（门禁扫描边界内）。

use std::hash::Hasher;

use fnv::FnvHasher;

/// FNV-1a 64-bit 哈希（`fnv` crate，与旧手写实现逐位等价）。
///
/// `bytes` 任意长度；空输入 → 返回 offset basis（FNV-1a 定义）。
#[must_use]
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hasher = FnvHasher::default();
    Hasher::write(&mut hasher, bytes);
    Hasher::finish(&hasher)
}

/// FNV 哈希的字符串形式（16 位小写 hex，供 manifest 显示用）。
#[must_use]
pub fn fnv1a64_hex(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}
