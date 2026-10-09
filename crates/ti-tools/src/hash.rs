//! 资源管线指纹哈希。
//!
//! FNV-1a 64（offset basis `0xcbf29ce484222325`，prime `0x100000001b3`）：
//! `build_assets` 生成 `assets_out/manifest.json` 文件指纹、`pcx_ref` 对拍参照、
//! `build_assets` 测试幂等校验共用同一实现（S1.10 / S1.3 / S1.3 验收）。
//!
//! 幂等契约：同一输入字节序列，输出 16 位小写 hex 完全一致（无时间戳、无随机盐）。

/// FNV-1a 64 指纹：返回 16 位小写 hex（固定长度，便于 manifest 对齐）。
///
/// 标准 FNV-1a 64（RFC 3120 / 定义常数）；纯函数、无 IO、三端可编。
#[must_use]
pub fn fnv1a64(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(PRIME);
    }
    format!("{h:016x}")
}
