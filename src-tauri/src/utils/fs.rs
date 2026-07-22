//! 文件系统操作工具
//!
//! 消除 `std::fs::create_dir_all` / `std::fs::read_to_string` 重复的 map_err 样板。

use std::path::Path;

pub fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {}", e))
}

pub fn read_to_string(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("读取文件失败: {}", e))
}
