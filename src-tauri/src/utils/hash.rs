//! 哈希工具
//!
//! 统一数据分段 sha256 摘要计算，消除各模块内联的等价实现。

/// 计算数据的 sha256 十六进制摘要（小写）
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}