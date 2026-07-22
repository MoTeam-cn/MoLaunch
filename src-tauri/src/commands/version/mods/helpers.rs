//! Mod 命令共享辅助函数
//!
//! 包含：
//! - get_mods_dir：获取版本的 mods 目录路径（pub(crate)，供 preload 命令复用）
//!
//! 注：sanitize_file_name 已迁移到 `crate::utils::path::sanitize_file_name`

use crate::state::AppState;
use tauri::State;

/// 获取版本的 mods 目录路径（内部辅助函数，pub(crate) 供 preload 命令复用）
pub(crate) async fn get_mods_dir(
    state: &State<'_, AppState>,
    version_id: &str,
) -> Result<std::path::PathBuf, String> {
    let game_dir = crate::state::resolve_game_dir_from_state(&state).await;
    let global_isolation_mode = state.config.lock().await.isolation_mode;

    // 版本独立隔离设置覆盖全局
    let isolation_mode = crate::commands::version::list::resolve_isolation_mode(
        &game_dir,
        version_id,
        global_isolation_mode,
    );
    let version_type =
        crate::commands::version::list::detect_version_type_from_dir(&game_dir, version_id);
    let mode = crate::minecraft::isolation::IsolationMode::from_u32(isolation_mode);
    let effective_dir = crate::minecraft::isolation::get_effective_game_dir(
        &game_dir,
        version_id,
        mode,
        version_type,
    );

    Ok(effective_dir.join("mods"))
}
