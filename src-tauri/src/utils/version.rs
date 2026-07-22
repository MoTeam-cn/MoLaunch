//! 版本号解析工具
//!
//! 提供统一的版本号解析函数，将版本字符串解析为可比较的数字元组。
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use crate::utils::version;
//!
//! let v = version::parse_number("1.20.1"); // [1, 20, 1]
//! let v = version::parse_number("1.20");   // [1, 20]
//! ```

/// 解析版本号为可比较的数字向量
///
/// 例如 "1.20.1" -> [1, 20, 1]，"2.0.0-beta" -> [2, 0, 0]
///
/// 非数字部分会被跳过（filter_map），因此 "1.20.1-beta.1" -> [1, 20, 1, 1]。
pub fn parse_number(version: &str) -> Vec<u32> {
    version.split('.').filter_map(|s| s.parse().ok()).collect()
}
