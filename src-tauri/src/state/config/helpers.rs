//! 从 AppState 提取配置值的异步 helper。

use std::path::PathBuf;

use super::paths::resolve_game_dir;
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;

/// 从 AppState 提取并解析 game_dir。
pub async fn resolve_game_dir_from_state(state: &AppState) -> PathBuf {
    let config = state.config.lock().await;
    resolve_game_dir(&config.game_dir)
}

/// 从 AppState 提取 mirror_url 和 meta source 模式。
pub async fn resolve_mirror_and_source(state: &AppState) -> (Option<String>, DownloadSourceMode) {
    let config = state.config.lock().await;
    let mirror_url = config.download.mirror_url.clone();
    let source_mode = DownloadSourceMode::from_str(&config.download.meta_source);
    (mirror_url, source_mode)
}
