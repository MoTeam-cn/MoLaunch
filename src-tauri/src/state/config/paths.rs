//! 游戏目录路径解析。

use std::path::{Path, PathBuf};

/// 获取默认游戏目录：启动器同级目录下的 .minecraft
pub(crate) fn get_default_game_dir() -> String {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(".minecraft").to_string_lossy().to_string();
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        return cwd.join(".minecraft").to_string_lossy().to_string();
    }
    ".minecraft".to_string()
}

/// 解析游戏目录：相对路径相对于可执行文件目录。
pub fn resolve_game_dir(game_dir: &str) -> PathBuf {
    let path = Path::new(game_dir);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join(game_dir);
        }
    }
    std::env::current_dir().unwrap_or_default().join(game_dir)
}
