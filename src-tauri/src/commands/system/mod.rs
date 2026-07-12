//! 系统命令模块

mod config;
mod download;
mod game;
mod game_dir;
mod proxy;

pub use config::*;
pub use download::*;
pub use game::*;
pub use game_dir::*;
pub use proxy::*;

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
