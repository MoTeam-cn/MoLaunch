//! 清理文件系统辅助（目录扫描 / 路径安全检查 / 递归删除）

use std::path::{Path, PathBuf};

use super::super::types::CleanupItem;

/// 递归扫描目录：累计所有文件大小与文件数
///
/// 不存在返回 None；存在但为空目录返回 size=0 / file_count=0 的 CleanupItem。
pub(super) fn scan_directory(dir: &Path, display_name: &str, category: &str) -> Option<CleanupItem> {
    if !dir.exists() {
        return None;
    }

    let mut size: u64 = 0;
    let mut file_count: u64 = 0;

    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];
    while let Some(p) = stack.pop() {
        let read = match std::fs::read_dir(&p) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.is_file() {
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                }
                file_count += 1;
            }
        }
    }

    Some(CleanupItem {
        path: dir.to_string_lossy().to_string(),
        display_name: display_name.to_string(),
        category: category.to_string(),
        size,
        file_count,
    })
}

/// 路径安全检查：path 必须等于 allowed_parents 中的某个，或位于其下
///
/// 防止路径遍历攻击（如传入 `..` 跳出 .minecraft）。
pub(super) fn is_path_safe(path: &Path, allowed_parents: &[PathBuf]) -> bool {
    let canonical = match path.canonicalize() {
        Ok(c) => c,
        Err(_) => return false,
    };
    for parent in allowed_parents {
        let parent_canonical = match parent.canonicalize() {
            Ok(c) => c,
            Err(_) => continue,
        };
        if canonical == parent_canonical || canonical.starts_with(&parent_canonical) {
            return true;
        }
    }
    false
}

/// 递归删除目录下的所有文件与子目录，最后删除目录本身
///
/// 返回 (累计字节数, 文件数)。
pub(super) fn remove_dir_recursive(root: &Path) -> Result<(u64, u64), String> {
    let mut size: u64 = 0;
    let mut files: u64 = 0;

    let mut dirs_to_walk: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut all_files: Vec<PathBuf> = Vec::new();
    let mut all_dirs: Vec<PathBuf> = Vec::new();

    while let Some(d) = dirs_to_walk.pop() {
        let read = std::fs::read_dir(&d).map_err(|e| format!("读取目录失败: {}", e))?;
        for entry in read.flatten() {
            let p = entry.path();
            if p.is_dir() {
                dirs_to_walk.push(p.clone());
                all_dirs.push(p);
            } else if p.is_file() {
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                }
                files += 1;
                all_files.push(p);
            }
        }
    }

    // 先删文件
    for f in &all_files {
        if let Err(e) = std::fs::remove_file(f) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("删除文件失败: {}", e));
            }
        }
    }

    // 再删目录（自底向上：按深度倒序，子目录优先）
    all_dirs.sort_by_key(|p| p.components().count());
    all_dirs.reverse();
    for d in &all_dirs {
        if let Err(e) = std::fs::remove_dir(d) {
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(format!("删除目录失败: {}", e));
            }
        }
    }

    // 最后删除 root 本身
    if root.exists() {
        std::fs::remove_dir(root).map_err(|e| format!("删除根目录失败: {}", e))?;
    }

    Ok((size, files))
}