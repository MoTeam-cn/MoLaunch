//! 系统命令模块

mod apply_config;
mod config;
mod download;
mod game;
mod game_dir;
mod proxy;

pub use apply_config::*;
pub use config::*;
pub use game_dir::*;

// download/game/proxy 模块保留为占位（后续如需专属命令可扩展），
// 所有 get/set 由 `get_config` / `apply_config` 统一处理。

use crate::state::AppState;

/// 更新配置并保存
pub(crate) async fn update_config<F>(state: &AppState, updater: F) -> Result<(), String>
where
    F: FnOnce(&mut crate::state::AppConfig),
{
    let mut config = state.config.lock().await;
    updater(&mut config);
    let config_clone = config.clone();
    drop(config);

    crate::config::save_config(&config_clone)?;
    Ok(())
}
