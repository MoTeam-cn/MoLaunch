//! 游戏目录相关命令
//!
//! Shell 命令（打开文件夹、选中文件等）走 `crate::minecraft::system::shell`；
//! 文件 / 文件夹选择对话框走前端 `@tauri-apps/plugin-dialog`。
//! 子模块函数由 `manager::dispatch` 反序列化参数后调用。

use crate::log_info;
use crate::state::AppState;
use std::path::{Component, Path, PathBuf};

/// 单次写入允许的最大字节数（文本）
const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024;
/// 单次写入允许的最大字节数（二进制，如 PNG 图片）
const MAX_BINARY_BYTES: usize = 20 * 1024 * 1024;
/// PNG 文件魔数：89 50 4E 47 0D 0A 1A 0A
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
/// 最长路径字符数（Windows MAX_PATH）
const MAX_PATH_CHARS: usize = 260;

/// 校验写入路径：非空、无空字节、长度不超限，并解析 `.` / `..` 防路径穿越，
/// 返回规范化后的路径。
fn validate_write_path(raw: &str) -> Result<PathBuf, String> {
    let path = raw.trim();
    if path.is_empty() {
        return Err("写入文件路径不能为空".to_string());
    }
    if path.contains('\0') {
        return Err("写入文件路径包含非法字符".to_string());
    }
    if path.chars().count() > MAX_PATH_CHARS {
        return Err("写入文件路径过长".to_string());
    }
    let mut normalized = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err("写入文件路径无效".to_string());
    }
    Ok(normalized)
}

/// 打开游戏目录
pub async fn open_game_dir(state: &AppState) -> Result<(), String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);

    log_info!("Opening game directory: {}", game_dir.display());

    // 如果游戏目录不存在，先尝试创建（防御性：启动时创建可能因权限失败，这里兜底）
    if !game_dir.exists() {
        std::fs::create_dir_all(&game_dir).map_err(|e| format!("创建游戏目录失败: {}", e))?;
    }

    crate::minecraft::system::shell::open_path(&game_dir.to_string_lossy())
}

/// 打开任意路径（文件夹或文件）
pub async fn open_path(path: String) -> Result<(), String> {
    crate::minecraft::system::shell::open_path(&path)
}

/// 在资源管理器中打开并选中指定文件
pub async fn reveal_in_explorer(path: String) -> Result<(), String> {
    crate::minecraft::system::shell::reveal_in_file_manager(&path)
}

/// 获取游戏目录
pub async fn get_game_dir(state: &AppState) -> Result<String, String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    Ok(game_dir.to_string_lossy().to_string())
}

/// 写入文本文件到指定路径
///
/// 用于导出示例文件（插件模板、布局示例等），路径通常由前端 `pickSavePath` 对话框返回。
/// 若文件已存在则覆盖，父目录不存在时自动创建。
/// 写入前校验路径合法性（防穿越）与内容大小上限，防止恶意文件落盘。
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
    let path = validate_write_path(&path)?;
    if content.len() > MAX_TEXT_BYTES {
        return Err(format!(
            "文本内容过大（> {} MB），已拒绝写入",
            MAX_TEXT_BYTES / 1024 / 1024
        ));
    }

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        crate::utils::fs::ensure_dir(parent)?;
    }

    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    log_info!("[System] Text file written: {}", path.display());
    Ok(())
}

/// 写入 PNG 图片到指定路径
///
/// `base64_content` 为 Base64 编码的 PNG 字节（Canvas `toDataURL` 导出）。
/// 写入前校验路径合法性（防穿越）、大小上限与 PNG 魔数，拒绝非图片内容。
pub async fn write_binary_file(path: String, base64_content: String) -> Result<(), String> {
    use base64::Engine;

    let path = validate_write_path(&path)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_content.trim())
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    if bytes.len() > MAX_BINARY_BYTES {
        return Err(format!(
            "写入内容过大（> {} MB），已拒绝写入",
            MAX_BINARY_BYTES / 1024 / 1024
        ));
    }
    if bytes.len() < PNG_SIGNATURE.len() || bytes[..PNG_SIGNATURE.len()] != PNG_SIGNATURE {
        return Err("写入内容不是合法的 PNG 图片，已拒绝".to_string());
    }

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        crate::utils::fs::ensure_dir(parent)?;
    }

    std::fs::write(&path, bytes).map_err(|e| format!("写入文件失败: {}", e))?;

    log_info!("[System] Binary file written: {}", path.display());
    Ok(())
}

/// 更新游戏目录
///
/// 通过 `system_manager` 的 `set_game_dir` action 暴露给前端。
///
/// 未合并到 `apply_config`：因为它在版本切换等内部流程中可能被直接调用，
/// 与用户设置页触发的 `apply_config({ gameDir: ... })` 走不同代码路径。
pub async fn set_game_dir(state: &AppState, game_dir: String) -> Result<(), String> {
    log_info!("Game directory changed to: {}", game_dir);
    super::update_config(state, |config| {
        config.game_dir = game_dir;
    })
    .await
}

/// 获取系统内存信息
pub async fn get_system_memory() -> Result<crate::minecraft::system::SystemMemory, String> {
    Ok(crate::minecraft::system::get_system_memory())
}
