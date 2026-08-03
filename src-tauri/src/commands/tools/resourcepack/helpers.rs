//! 共享辅助：资源包目录解析、路径转字符串（list / convert 复用）

use std::path::PathBuf;

use crate::minecraft::isolation::{get_effective_game_dir, IsolationMode};
use crate::state::{resolve_game_dir, AppState};

/// 解析资源包目录（同 screenshot::resolve_shots_dir 的语义）
pub(super) async fn resolve_packs_dir(state: &AppState, version_id: Option<&str>) -> PathBuf {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    match version_id {
        None => game_dir.join("resourcepacks"),
        Some(vid) => {
            let global_mode = state.config.lock().await.isolation_mode;
            let isolation_mode =
                crate::commands::version::list::resolve_isolation_mode(&game_dir, vid, global_mode);
            let version_type =
                crate::commands::version::list::detect_version_type_from_dir(&game_dir, vid);
            let mode = IsolationMode::from_u32(isolation_mode);
            let effective_dir = get_effective_game_dir(&game_dir, vid, mode, version_type);
            effective_dir.join("resourcepacks")
        }
    }
}

/// 将路径转为字符串（UTF-8，丢失非 UTF-8 字符）
pub(super) fn path_to_string(path: &std::path::Path) -> String {
    path.to_str().unwrap_or("").to_string()
}