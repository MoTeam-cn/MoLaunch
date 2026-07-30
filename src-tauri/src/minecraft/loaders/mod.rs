//! Loader management module

pub mod fabric_api;
pub mod forge_html;
pub mod forge_installer;

mod fabric;
mod forge;
mod liteloader;
mod neoforge;
mod optifine;
mod shared;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::minecraft::download::config::DownloadManagerConfig;
use super::sources::DownloadSourceMode;

/// Loader type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoaderType {
    Forge,
    NeoForge,
    Fabric,
    OptiFine,
    LiteLoader,
}

/// Loader version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub version: String,
    pub is_recommended: bool,
    pub release_time: Option<String>,
}

/// List Forge versions
pub async fn list_forge_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    forge::list_versions(mc_version, mirror_url, source_mode).await
}

/// List NeoForge versions
pub async fn list_neoforge_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    neoforge::list_versions(mc_version, mirror_url, source_mode).await
}

/// List Fabric versions
pub async fn list_fabric_versions(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    fabric::list_versions(mirror_url, source_mode).await
}

/// List OptiFine versions
pub async fn list_optifine_versions(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    optifine::list_versions(mirror_url, source_mode).await
}

/// List LiteLoader versions
pub async fn list_liteloader_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    liteloader::list_versions(mc_version, mirror_url, source_mode).await
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
            forge::install(
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
            neoforge::install(
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
            fabric::install(
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
            optifine::install(mc_version, loader_version, progress_callback, config.source_mode)
                .await
        }
        LoaderType::LiteLoader => {
            liteloader::install(
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
