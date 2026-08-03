//! Forge 加载器安装（install_merged 加载器安装阶段）
//!
//! 从 `install_all_loaders` 中拆出的 Forge 分支，统一走
//! `install_single_loader` 通用安装流程。

use crate::minecraft::loaders;
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;

use super::loader_helpers::install_single_loader;

/// 安装 Forge 加载器
#[allow(clippy::too_many_arguments)]
pub(crate) async fn install_forge(
    state: &AppState,
    mc_version: &str,
    game_dir: &std::path::Path,
    forge_version: &str,
    mirror_url: Option<&str>,
    max_threads: usize,
    source_mode: DownloadSourceMode,
) -> Result<(), String> {
    install_single_loader(
        state,
        loaders::LoaderType::Forge,
        "Forge",
        forge_version,
        mc_version,
        game_dir,
        mirror_url,
        max_threads,
        source_mode,
    )
    .await
}
