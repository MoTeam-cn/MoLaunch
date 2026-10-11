//! 哈希工具
//!
//! 统一数据分段 sha256 摘要计算，消除各模块内联的等价实现。

/// 字节序列 → 小写十六进制串
///
/// 与 hex::encode 等价（本项目 hex 依赖已在多处使用）；此处保留函数名是为了让
/// 「摘要 → 十六进制」这一步在调用点可读。
pub fn hex_encode(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// 计算数据的 sha256 十六进制摘要（小写）
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}
