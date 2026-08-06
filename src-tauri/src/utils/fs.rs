//! 文件系统操作工具
//!
//! 消除 `std::fs::create_dir_all` / `std::fs::read_to_string` 重复的 map_err 样板，
//! 并提供日志/报告场景的通用读取（末尾 N 行、指定行段、最新文件）。

use std::path::Path;

/// 创建目录（含父目录）
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {}", e))
}

/// 读取文件全部内容
pub fn read_to_string(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))
}

/// 取文本末尾 N 行（倒序取再反转，保证原顺序）
pub fn tail_lines(content: &str, lines: usize) -> String {
    content
        .lines()
        .rev()
        .take(lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

/// 读取文件末尾 N 行，超长时按字符数截断（保留行首，即最新内容）
pub fn read_tail(path: &Path, lines: usize, max_chars: usize) -> Result<String, String> {
    let content = read_to_string(path)?;
    let tail = tail_lines(&content, lines);
    Ok(crate::utils::format::truncate_chars(&tail, max_chars))
}

/// 目录中修改时间最新的文件（可按扩展名过滤，可选）
pub fn newest_file(dir: &Path, ext: Option<&str>) -> Option<std::path::PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| {
            ext.map(|e| p.extension().map(|x| x == e).unwrap_or(false))
                .unwrap_or(true)
        })
        .max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok())
}
