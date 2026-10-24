//! Java 候选路径优先级判定
//!
//! 两类路径**降权**（排在候选列表末尾）而不是直接排除——用户机器上可能只有这一个：
//! - 特殊路径：`system32` / `javapath_target_` / `javatmp` 是系统或安装器代理出来的
//!   转发目录，实际指向经常变化，优先使用容易选到意料之外的版本；
//! - 符号链接：路径链上有符号链接时，目标可能已卸载或指向别处。
//!
//! 注意：Windows 的目录联接（junction）不是严格意义的符号链接，`is_symlink()` 检测不到，
//! 这类情况由 `javapath_target_` 关键词兜住。

use std::path::Path;

/// 特殊路径关键词（路径已小写化、反斜杠归一为斜杠）
const SPECIAL_PATH_KEYWORDS: [&str; 4] = ["system32", "javapath_target_", "javatmp", "javartmp"];

/// 候选路径是否应降权
pub(crate) fn is_deprioritized_path(path: &Path) -> bool {
    let normalized = path.to_string_lossy().to_lowercase().replace('\\', "/");
    if SPECIAL_PATH_KEYWORDS
        .iter()
        .any(|keyword| normalized.contains(keyword))
    {
        return true;
    }
    has_symlink_component(path)
}

/// 路径自身或任一上级目录是符号链接时返回 true
///
/// 取不到元数据（权限不足 / 路径已失效）不作为降权依据，继续向上检查。
fn has_symlink_component(path: &Path) -> bool {
    let mut current = Some(path);
    while let Some(component) = current {
        if let Ok(metadata) = std::fs::symlink_metadata(component) {
            if metadata.file_type().is_symlink() {
                return true;
            }
        }
        current = component.parent();
    }
    false
}

#[cfg(test)]
#[path = "priority_test.rs"]
mod priority_test;
