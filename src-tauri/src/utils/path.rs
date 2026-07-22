//! 路径与文件名安全工具
//!
//! 提供文件名和路径片段的安全校验函数，防止路径遍历攻击。
//!
//! ## 安全说明
//!
//! 用户输入的文件名必须经过校验，防止：
//! - 路径遍历（`../`）
//! - 路径分隔符注入（`/` `\`）
//! - 空字节注入（`\0`）
//! - UNC 路径（`\\`）
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use crate::utils::path;
//!
//! path::sanitize_file_name("mod.jar")?;       // OK
//! path::sanitize_file_name("../evil.jar")?;   // Err
//! path::sanitize_file_name("a/b.jar")?;       // Err
//! ```

/// 校验文件名是否安全（防路径遍历）
///
/// 拒绝以下情况：
/// - 空字符串
/// - 包含路径分隔符 `/` 或 `\`
/// - 包含路径遍历 `..`
/// - 包含空字节 `\0`
///
/// 校验通过返回 `Ok(())`，否则返回错误描述。
pub fn sanitize_file_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.contains('\0')
    {
        return Err(format!("Invalid file name: {}", name));
    }
    Ok(())
}
