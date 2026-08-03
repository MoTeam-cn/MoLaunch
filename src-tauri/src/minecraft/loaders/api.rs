//! 加载器公共 API：版本列表获取与安装分发

use std::path::Path;
use std::sync::Arc;

use super::types::{LoaderType, LoaderVersion};
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::sources::DownloadSourceMode;

/// List Forge versions
pub async fn list_forge_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    super::forge::list_versions(mc_version, mirror_url, source_mode).await
}

/// List NeoForge versions
pub async fn list_neoforge_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    super::neoforge::list_versions(mc_version, mirror_url, source_mode).await
}

/// List Fabric versions
pub async fn list_fabric_versions(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    super::fabric::list_versions(mirror_url, source_mode).await
}

/// List OptiFine versions
pub async fn list_optifine_versions(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    super::optifine::list_versions(mirror_url, source_mode).await
}

/// List LiteLoader versions
pub async fn list_liteloader_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    super::liteloader::list_versions(mc_version, mirror_url, source_mode).await
}

/// Install loader (入口分发函数)
pub async fn install_loader(
    loader_type: LoaderType,
    mc_version: &str,
    loader_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    match loader_type {
        LoaderType::Forge => {
            super::forge::install(
                mc_version,
                loader_version,
                game_dir,
                mirror_url,
                progress_callback,
                config,
            )
            .await
        }
        LoaderType::NeoForge => {
            super::neoforge::install(
                mc_version,
                loader_version,
                game_dir,
                mirror_url,
                progress_callback,
                config,
            )
            .await
        }
        LoaderType::Fabric => {
            super::fabric::install(
                mc_version,
                loader_version,
                game_dir,
                mirror_url,
                progress_callback,
                config,
            )
            .await
        }
        LoaderType::OptiFine => {
            super::optifine::install(
                mc_version,
                loader_version,
                progress_callback,
                config.source_mode,
            )
            .await
        }
        LoaderType::LiteLoader => {
            super::liteloader::install(
                mc_version,
                loader_version,
                game_dir,
                mirror_url,
                progress_callback,
                config,
            )
            .await
        }
    }
}
