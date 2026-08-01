use serde::{Deserialize, Serialize};

/// 内存优化请求参数
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryOptimizeParams {
    /// 优化模式："light"（轻量）或 "strong"（强力）
    pub mode: String,
}

/// 内存优化结果（所有字段单位：字节）
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryOptimizeResult {
    pub freed_bytes: u64,
    pub before_bytes: u64,
    pub after_bytes: u64,
    /// 本次优化使用的模式："light" / "strong"
    pub mode: String,
}
