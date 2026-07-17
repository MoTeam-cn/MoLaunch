//! Mod 命令共享辅助函数
//!
//! 包含：
//! - get_mods_dir：获取版本的 mods 目录路径（pub(crate)，供 preload 命令复用）
//! - sanitize_file_name：校验文件名防路径遍历（pub(super)，供 mod.rs / file_ops.rs 共用）

use crate::state::AppState;
use tauri::State;

/// 获取版本的 mods 目录路径（内部辅助函数，pub(crate) 供 preload 命令复用）
pub(crate) async fn get_mods_dir(
    state: &State<'_, AppState>,
    version_id: &str,
) -> Result<std::path::PathBuf, String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let global_isolation_mode = config.isolation_mode;
    drop(config);

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

/// 校验文件名，防止路径遍历
pub(super) fn sanitize_file_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.contains('\0')
    {
        return Err(format!("Invalid file name: {}", name));
    }
    Ok(())
}
