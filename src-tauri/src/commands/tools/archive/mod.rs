//! 存档管理（备份/恢复/列表/种子提取）
//!
//! 子模块：list（列表扫描）、backup（打包 zip）、restore（解压恢复）、
//! seed（level.dat 种子提取）、helpers（zip I/O 辅助）。

mod backup;
mod helpers;
mod list;
mod restore;
mod seed;

pub use backup::backup;
pub use list::list;
pub use restore::restore;
pub use seed::extract_save_seed;

use std::path::PathBuf;

use crate::minecraft::isolation::{get_effective_game_dir, IsolationMode};
use crate::state::resolve_game_dir;
use crate::state::AppState;

/// 解析 saves 目录（同 screenshot::resolve_shots_dir 的语义）
pub(super) async fn resolve_saves_dir(state: &AppState, version_id: Option<&str>) -> PathBuf {
    let game_dir = {
        let config = state.config.lock().await;
        resolve_game_dir(&config.game_dir)
    };
    match version_id {
        None => game_dir.join("saves"),
        Some(vid) => {
            let global_mode = state.config.lock().await.isolation_mode;
            let isolation_mode =
                crate::commands::version::list::resolve_isolation_mode(&game_dir, vid, global_mode);
            let version_type =
                crate::commands::version::list::detect_version_type_from_dir(&game_dir, vid);
            let mode = IsolationMode::from_u32(isolation_mode);
            let effective_dir = get_effective_game_dir(&game_dir, vid, mode, version_type);
            effective_dir.join("saves")
        }
    }
}
