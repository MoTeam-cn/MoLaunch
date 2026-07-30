//! Forge 安装调度器 + Modern 安装（1.13+，injector 方式）

use std::path::Path;
use std::sync::Arc;

use crate::{log_info, log_warn};
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::launcher_profiles;
use crate::minecraft::sources;

use super::super::shared;
use super::legacy::install_legacy;

/// Install Forge
pub async fn install(
    mc_version: &str,
    forge_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    let file_name = format!("forge-{}-{}-installer.jar", mc_version, forge_version);
    let installer_url = sources::forge_installer_url(mc_version, forge_version);
    let temp_dir = crate::utils::cache_temp::ensure_task_temp_dir()?;
    let installer_path = temp_dir.join(&file_name);

    // 尝试获取文件 hash
    let hash_url = format!("{}.sha1", installer_url);
    let expected_hash = match crate::http::fetch_url(&hash_url).await {
        Ok(hash) => Some(hash.trim().to_string()),
        Err(_) => None,
    };

    // Download installer
    let urls = sources::build_replace_urls(
        &installer_url,
        mirror_url,
        sources::MAVEN_REPLACEMENTS,
        config.source_mode,
    );

    let manager = DownloadManager::from_config(config);
    let task = DownloadTask {
        id: "forge_installer".to_string(),
        urls,
        local_path: installer_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            let _ = std::fs::remove_file(&installer_path);
            return Err(anyhow::anyhow!("Failed to download Forge installer"));
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(10.0);
    }

    // 根据 Forge 版本选择安装方式
    if super::super::forge_installer::needs_injector(forge_version, false) {
        install_modern(
            mc_version,
            forge_version,
            &installer_path,
            game_dir,
            progress_callback,
            config,
        )
        .await
    } else {
        install_legacy(
            mc_version,
            forge_version,
            &installer_path,
            game_dir,
            progress_callback,
        )
        .await
    }
}

/// Modern Forge installation (1.13+)
async fn install_modern(
    mc_version: &str,
    forge_version: &str,
    installer_path: &Path,
    game_dir: &Path,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    log_info!("[Forge] Installing {} for MC {}", forge_version, mc_version);

    let version_id = format!("{}-forge-{}", mc_version, forge_version);

    launcher_profiles::ensure_profiles_exist(game_dir).map_err(|e: String| anyhow::anyhow!(e))?;

    if let Some(ref cb) = progress_callback {
        cb(20.0);
    }

    // 下载 Mojang 映射文件
    if let Err(e) =
        shared::download_mojang_mappings(mc_version, game_dir, installer_path, config).await
    {
        log_warn!("[Forge] Failed to download mappings: {}", e);
    }

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    log_info!("[Forge] Using injector for Forge {}", forge_version);

    let (injector_path, wrapper_path) = super::super::forge_installer::extract_embedded_resources()?;

    if let Some(ref cb) = progress_callback {
        cb(40.0);
    }

    let java_path = shared::find_java_for_install(game_dir)?;

    if let Some(ref cb) = progress_callback {
        cb(50.0);
    }

    super::super::forge_installer::run_forge_installer(
        &java_path,
        &installer_path.to_string_lossy(),
        &injector_path,
        &wrapper_path,
        &game_dir.to_string_lossy(),
        false,
        None,
    )?;

    if let Some(ref cb) = progress_callback {
        cb(80.0);
    }

    // Find and copy the generated version JSON
    shared::copy_generated_version_json(game_dir, mc_version, &version_id, "forge");

    // Copy MC JAR to Forge version folder
    shared::copy_mc_jar(game_dir, mc_version, &version_id);

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[Forge] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}
