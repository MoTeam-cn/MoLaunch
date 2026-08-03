//! 加载器批量安装（install_merged 的加载器安装阶段）
//!
//! 遍历 5 种加载器（Forge/NeoForge/Fabric/OptiFine/LiteLoader），
//! Forge/NeoForge/Fabric 分支由独立子模块实现，OptiFine/LiteLoader 内联，
//! 逐个安装并收集错误。

use crate::minecraft::loaders;
use crate::state::AppState;

use super::fabric::install_fabric;
use super::forge::install_forge;
use super::loader_helpers::install_single_loader;
use super::neoforge::install_neoforge;

/// 批量安装所有选中的加载器，返回错误列表（空 = 全部成功）
#[allow(clippy::too_many_arguments)]
pub(crate) async fn install_all_loaders(
    state: &AppState,
    mc_version: &str,
    game_dir: &std::path::Path,
    forge_version: &Option<String>,
    neoforge_version: &Option<String>,
    fabric_version: &Option<String>,
    optifine_version: &Option<String>,
    liteloader_version: &Option<String>,
    mirror_url: Option<&str>,
    max_threads: usize,
    source_mode: crate::minecraft::sources::DownloadSourceMode,
) -> Vec<String> {
    let mut loader_errors = Vec::new();

    // Forge
    if let Some(forge_ver) = forge_version {
        if let Err(e) = install_forge(
            state,
            mc_version,
            game_dir,
            forge_ver,
            mirror_url,
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    // NeoForge
    if let Some(neoforge_ver) = neoforge_version {
        if let Err(e) = install_neoforge(
            state,
            mc_version,
            game_dir,
            neoforge_ver,
            mirror_url,
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    // Fabric
    if let Some(fabric_ver) = fabric_version {
        if let Err(e) = install_fabric(
            state,
            mc_version,
            game_dir,
            fabric_ver,
            mirror_url,
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    // OptiFine
    if let Some(optifine_ver) = optifine_version {
        if let Err(e) = install_single_loader(
            state,
            loaders::LoaderType::OptiFine,
            "OptiFine",
            optifine_ver,
            mc_version,
            game_dir,
            mirror_url,
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    // LiteLoader
    if let Some(liteloader_ver) = liteloader_version {
        if let Err(e) = install_single_loader(
            state,
            loaders::LoaderType::LiteLoader,
            "LiteLoader",
            liteloader_ver,
            mc_version,
            game_dir,
            mirror_url,
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    loader_errors
}
