//! 应用配置模块
//!
//! 配置结构体、默认值、路径解析和 AppState helper 分别位于子模块，
//! 通过 `pub use` 保持原有 `state::AppConfig` 等公共路径不变。

mod defaults;
mod helpers;
mod models;
mod paths;

pub use helpers::{resolve_game_dir_from_state, resolve_mirror_and_source};
pub use models::{AppConfig, McFolder};
pub use paths::resolve_game_dir;
