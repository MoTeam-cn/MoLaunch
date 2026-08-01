//! 应用状态管理
//! 按关注点拆分为 5 个子模块（app / auth / config / download / launch），
//! 通过 `pub use` 统一 re-export，保持 `crate::state::X` 路径向后兼容。
//! 另提供 `resolve_game_dir_from_state` / `resolve_mirror_and_source` 两个 helper
//! 消除后端重复的 lock/clone/drop 套件。

mod app;
mod auth;
mod config;
mod download;
mod launch;

use std::path::PathBuf;

pub use app::AppState;
pub use auth::{AuthState, LocalAuthResult};
pub use config::{resolve_game_dir, AppConfig, McFolder};
pub use download::{DownloadStage, DownloadState, StageStatus};
pub use launch::LaunchHistory;

/// 从 AppState 提取 game_dir（消除 lock/resolve/drop 三行套件）
pub async fn resolve_game_dir_from_state(state: &AppState) -> PathBuf {
    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);
    drop(config);
    game_dir
}

/// 从 AppState 提取 mirror_url 和 source_mode（消除 lock/clone/from_str/drop 四行套件）
pub async fn resolve_mirror_and_source(
    state: &AppState,
) -> (
    Option<String>,
    crate::minecraft::sources::DownloadSourceMode,
) {
    let config = state.config.lock().await;
    let mirror_url = config.download.mirror_url.clone();
    let source_mode =
        crate::minecraft::sources::DownloadSourceMode::from_str(&config.download.meta_source);
    drop(config);
    (mirror_url, source_mode)
}
