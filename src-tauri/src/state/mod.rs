//! 应用状态管理
//!
//! 按关注点拆分为 5 个子模块：
//! - `app`      AppState（聚合 SDK / 配置 / 认证 / 下载 / 启动历史等 Arc 句柄）
//! - `auth`     LocalAuthResult + AuthState
//! - `config`   AppConfig + McFolder + resolve_game_dir
//! - `download` StageStatus + DownloadStage + DownloadState
//! - `launch`   LaunchHistory
//!
//! 通过 `pub use` 统一 re-export，保持 `crate::state::X` 路径向后兼容。

mod app;
mod auth;
mod config;
mod download;
mod launch;

pub use app::AppState;
pub use auth::{AuthState, LocalAuthResult};
pub use config::{resolve_game_dir, AppConfig, McFolder};
pub use download::{DownloadStage, DownloadState, StageStatus};
pub use launch::LaunchHistory;
