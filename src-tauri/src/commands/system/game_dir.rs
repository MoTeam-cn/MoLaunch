//! 游戏目录相关命令
//!
//! Shell 命令（打开文件夹、选中文件等）走 `crate::minecraft::system::shell`；
//! 文件 / 文件夹选择对话框走前端 `@tauri-apps/plugin-dialog`。
//! 子模块函数由 `manager::dispatch` 反序列化参数后调用。

use crate::log_info;
use crate::state::AppState;

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
pub async fn write_text_file(path: String, content: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(&path);

    // 确保父目录存在
    if let Some(parent) = path.parent() {
        crate::utils::fs::ensure_dir(parent)?;
    }

    std::fs::write(&path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    log_info!("[System] Text file written: {}", path.display());
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
