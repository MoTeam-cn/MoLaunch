//! 字节数与速度格式化工具
//!
//! 提供统一的字节数和速度格式化函数，避免在各业务模块中重复实现。
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use crate::utils::format;
//!
//! // 默认 1 位小数（下载日志场景）
//! assert_eq!(format::bytes(1_500_000), "1.4 MB");
//!
//! // 2 位小数（缓存清理统计场景）
//! assert_eq!(format::bytes_with(1_500_000, 2), "1.43 MB");
//!
//! // 速度格式化
//! assert_eq!(format::speed(1_500_000), "1.4 MB/s");
//! ```

/// 字节数单位（1024 进制）
const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

/// 默认小数位数（1 位，用于下载日志场景）
const DEFAULT_DECIMALS: usize = 1;

/// 格式化字节数为人类可读大小（默认 1 位小数）
///
/// ```rust,ignore
/// assert_eq!(format::bytes(0), "0 B");
/// assert_eq!(format::bytes(512), "512 B");
/// assert_eq!(format::bytes(1500), "1.5 KB");
/// assert_eq!(format::bytes(1_500_000), "1.4 MB");
/// assert_eq!(format::bytes(1_500_000_000), "1.4 GB");
/// ```
pub fn bytes(bytes: u64) -> String {
    bytes_with(bytes, DEFAULT_DECIMALS)
}

/// 格式化字节数为人类可读大小（指定小数位数）
pub fn bytes_with(bytes: u64, decimals: usize) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }

    let k = 1024u64;
    // 通过循环确定单位档位，避免使用 log 计算（f64::log 需手动实现）
    let mut i = 0usize;
    let mut value = bytes as f64;
    while value >= k as f64 && i < UNITS.len() - 1 {
        value /= k as f64;
        i += 1;
    }

    format!("{:.*} {}", decimals, value, UNITS[i])
}

/// 格式化速度为人类可读字符串（默认 1 位小数）
///
/// ```rust,ignore
/// assert_eq!(format::speed(500), "500 B/s");
/// assert_eq!(format::speed(1500), "1.5 KB/s");
/// assert_eq!(format::speed(1_500_000), "1.4 MB/s");
/// ```
pub fn speed(bytes_per_sec: u64) -> String {
    format!("{}/s", bytes(bytes_per_sec))
}

/// 格式化速度为人类可读字符串（指定小数位数）
pub fn speed_with(bytes_per_sec: u64, decimals: usize) -> String {
    format!("{}/s", bytes_with(bytes_per_sec, decimals))
}
