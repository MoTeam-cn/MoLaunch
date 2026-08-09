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

/// 校验相对路径安全（防路径穿越与绝对路径逃逸）
///
/// 复用段级 `is_safe_relative_path` 检查 ParentDir，并额外拒绝空字符串、空字节、
/// `/`/`\` 开头与 Windows 盘符开头的绝对路径。校验通过返回原路径，否则返回错误描述。
pub fn ensure_safe_relative_path(path: &str) -> Result<String, String> {
    if path.is_empty()
        || path.contains('\0')
        || path.starts_with('/')
        || path.starts_with('\\')
        || starts_with_drive_letter(path)
        || !is_safe_relative_path(path)
    {
        return Err(format!("非法相对路径: {}", path));
    }
    Ok(path.to_string())
}

/// 判断是否以 Windows 盘符开头（如 `C:`、`c:\`）
fn starts_with_drive_letter(path: &str) -> bool {
    let b = path.as_bytes();
    b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_safe_relative_path_rejects_unsafe() {
        for p in ["../x", "..", "/abs", "\\abs", "C:\\abs", "c:/abs", "a/../b", "", "a\0b"] {
            assert!(ensure_safe_relative_path(p).is_err(), "应拒绝: {:?}", p);
        }
    }

    #[test]
    fn ensure_safe_relative_path_accepts_normal() {
        for p in ["mods/x.jar", "a/b", "mods\\x.jar"] {
            assert_eq!(ensure_safe_relative_path(p).as_deref(), Ok(p), "应通过: {:?}", p);
        }
    }
}
