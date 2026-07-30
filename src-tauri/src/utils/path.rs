//! 路径与文件名安全工具
//!
//! 提供文件名和路径片段的安全校验函数，防止路径遍历攻击。

/// 校验文件名是否安全（防路径遍历）
///
/// 拒绝空字符串、路径分隔符 `/` `\`、路径遍历 `..`、空字节 `\0`。
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
