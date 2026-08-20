//! 版本隔离目录解析（mods / resourcepacks / shaderpacks 所在层）

use std::path::PathBuf;

use crate::minecraft::isolation::{get_effective_game_dir, IsolationMode};
use crate::state::AppState;

/// 解析版本隔离目录（mods / resourcepacks / shaderpacks 所在层）
pub(crate) async fn resolve_effective_game_dir(
    state: &AppState,
    version_id: &str,
) -> Result<PathBuf, String> {
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let global_isolation_mode = state.config.lock().await.isolation_mode;
    let isolation_mode = crate::commands::version::list::resolve_isolation_mode(
        &game_dir,
        version_id,
        global_isolation_mode,
    );
    let version_type =
        crate::commands::version::list::detect_version_type_from_dir(&game_dir, version_id);
    let mode = IsolationMode::from_u32(isolation_mode);
    Ok(get_effective_game_dir(
        &game_dir,
        version_id,
        mode,
        version_type,
    ))
}

/// 解析版本隔离目录下的内容子目录（mods / resourcepacks / shaderpacks）
pub(crate) async fn resolve_version_subdir(
    state: &AppState,
    version_id: &str,
    subdir: &str,
) -> Result<PathBuf, String> {
    Ok(resolve_effective_game_dir(state, version_id)
        .await?
        .join(subdir))
}
