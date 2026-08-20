//! 版本内容目录条目操作（枚举 / 启停 / 删除 / 安装）

use std::path::Path;

/// 目录条目（枚举结果）
pub(crate) struct DirEntry {
    pub file_name: String,
    pub enabled_name: String,
    pub is_enabled: bool,
    pub is_dir: bool,
    pub size: u64,
}

/// 去除启停后缀，得到启用时的文件名
pub(crate) fn enabled_name_of(file_name: &str) -> String {
    file_name
        .trim_end_matches(".disabled")
        .trim_end_matches(".old")
        .to_string()
}

/// 是否处于启用状态
fn is_enabled_name(file_name: &str) -> bool {
    let lower = file_name.to_lowercase();
    !(lower.ends_with(".disabled") || lower.ends_with(".old"))
}

/// 是否匹配允许的扩展名（含 .disabled / .old 变体）
fn matches_suffixes(file_name: &str, suffixes: &[&str]) -> bool {
    let lower = file_name.to_lowercase();
    suffixes.iter().any(|s| {
        let suffix = format!(".{}", s);
        lower.ends_with(&suffix)
            || lower.ends_with(&format!("{}.disabled", suffix))
            || lower.ends_with(&format!("{}.old", suffix))
    })
}

fn dir_total_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total += dir_total_size(&path);
            } else if let Ok(meta) = entry.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

/// 枚举目录条目（扩展名过滤 + 可选文件夹，按文件名排序）
pub(crate) fn list_entries(
    dir: &Path,
    suffixes: &[&str],
    include_dirs: bool,
) -> Result<Vec<DirEntry>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let read = std::fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
    for entry in read.flatten() {
        let path = entry.path();
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };
        let is_dir = path.is_dir();
        if is_dir {
            if !include_dirs {
                continue;
            }
        } else if !matches_suffixes(&file_name, suffixes) {
            continue;
        }
        let size = if is_dir {
            dir_total_size(&path)
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };
        entries.push(DirEntry {
            enabled_name: enabled_name_of(&file_name),
            is_enabled: is_enabled_name(&file_name),
            file_name,
            is_dir,
            size,
        });
    }
    entries.sort_by_key(|a| a.file_name.to_lowercase());
    Ok(entries)
}

/// 启停条目：重命名 .disabled / .old，返回新文件名
pub(crate) fn toggle_entry(dir: &Path, file_name: &str, enable: bool) -> Result<String, String> {
    let src_path = dir.join(file_name);
    if !src_path.exists() {
        return Err(format!("文件不存在: {}", file_name));
    }
    if is_enabled_name(file_name) == enable {
        return Ok(file_name.to_string());
    }
    let new_name = if enable {
        enabled_name_of(file_name)
    } else {
        let disabled_name = format!("{}.disabled", file_name);
        if !dir.join(&disabled_name).exists() {
            disabled_name
        } else {
            format!("{}.old", file_name)
        }
    };
    let dst_path = dir.join(&new_name);
    if dst_path.exists() && dst_path != src_path {
        return Err(format!("目标文件已存在: {}", new_name));
    }
    std::fs::rename(&src_path, &dst_path).map_err(|e| format!("重命名失败: {}", e))?;
    Ok(new_name)
}

/// 删除条目（文件或目录）
pub(crate) fn delete_entry(dir: &Path, file_name: &str) -> Result<(), String> {
    let path = dir.join(file_name);
    if !path.exists() {
        return Err(format!("文件不存在: {}", file_name));
    }
    if path.is_dir() {
        std::fs::remove_dir_all(&path).map_err(|e| format!("删除目录失败: {}", e))?;
    } else {
        std::fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {}", e))?;
    for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录失败: {}", e))? {
        let entry = entry.map_err(|e| format!("读取目录项失败: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| format!("复制文件失败: {}", e))?;
        }
    }
    Ok(())
}

/// 从外部路径安装条目（文件或目录，自动去除启停后缀），返回安装后的文件名
pub(crate) fn install_entry(
    dir: &Path,
    source_path: &str,
    suffixes: &[&str],
) -> Result<String, String> {
    if !crate::utils::path::is_safe_relative_path(source_path) {
        return Err("源路径不能包含 ..".to_string());
    }
    let src = Path::new(source_path);
    if !src.is_absolute() {
        return Err("源路径必须是绝对路径".to_string());
    }
    if !src.exists() {
        return Err(format!("源文件不存在: {}", source_path));
    }
    let original_name = src
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("无法获取文件名")?
        .to_string();
    let clean_name = enabled_name_of(&original_name);
    if !src.is_dir() && !matches_suffixes(&clean_name, suffixes) {
        return Err(format!("不支持的文件格式: {}", clean_name));
    }
    let dst = dir.join(&clean_name);
    if dst.exists() {
        return Err(format!("目标目录已存在同名文件: {}", clean_name));
    }
    if src.is_dir() {
        copy_dir_recursive(src, &dst)?;
    } else {
        std::fs::copy(src, &dst).map_err(|e| format!("复制文件失败: {}", e))?;
    }
    Ok(clean_name)
}
