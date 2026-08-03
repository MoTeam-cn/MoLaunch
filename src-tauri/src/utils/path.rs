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
        // 文件名场景的字面净化：文件名字符串中不允许出现连续 `..` 子串
        || name.contains("..")
        || name.contains('\0')
    {
        return Err(format!("Invalid file name: {}", name));
    }
    Ok(())
}

/// 判断相对路径是否不包含路径遍历段 `..`
///
/// 逐路径段检查（`Path::components` 的 `ParentDir`），非字面 `contains("..")`：
/// `foo..bar` 不构成穿越，属安全路径；真正的目录穿越必须以 `..` 为独立路径段，
/// 段级检查可全部覆盖。调用方应自行负责空值、分隔符与绝对路径等其余校验。
pub fn is_safe_relative_path(path: &str) -> bool {
    !std::path::Path::new(path)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
}
