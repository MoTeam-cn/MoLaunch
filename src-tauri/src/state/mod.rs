//! 应用状态管理
//! 按关注点拆分为 5 个子模块（app / auth / config / download / launch），
//! 通过 `pub use` 统一 re-export，保持 `crate::state::X` 路径向后兼容。

mod app;
mod auth;
mod config;
mod download;
mod launch;

pub use app::AppState;
pub use auth::{AuthState, LocalAuthResult};
pub use config::{
    resolve_game_dir, resolve_game_dir_from_state, resolve_mirror_and_source, AppConfig, McFolder,
};
pub use download::{DownloadStage, DownloadState, StageStatus};
pub use launch::LaunchHistory;
