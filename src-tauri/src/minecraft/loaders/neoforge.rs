//! NeoForge loader module

use crate::{log_info, log_warn};
use std::path::Path;
use std::sync::Arc;

use super::shared;
use super::LoaderVersion;
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::launcher_profiles;
use crate::minecraft::sources::{self, DownloadSourceMode};

/// List NeoForge versions
pub async fn list_versions(
    mc_version: &str,
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<Vec<LoaderVersion>> {
    crate::log_separator!("NeoForge List");
    crate::log_info!("[NeoForge] Listing versions for MC {}", mc_version);

    let urls = sources::build_urls(
        mirror_url,
        sources::NEOFORGE_API,
        sources::BMCLAPI_NEOFORGE,
        source_mode,
    );
    crate::log_debug!("[NeoForge] 尝试源: {:?}", urls);

    let content = match sources::fetch_with_fallback(&urls).await {
        Ok(c) => {
            crate::log_debug!("[NeoForge] Response length: {} bytes", c.len());
            c
        }
        Err(e) => {
            crate::log_error!("[NeoForge] 所有源失败: {}", e);
            return Ok(vec![]);
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(j) => j,
        Err(e) => {
            crate::log_error!("[NeoForge] JSON parse failed: {}", e);
            return Ok(vec![]);
        }
    };

    let files_array = json["files"]
        .as_array()
        .or_else(|| json["versions"].as_array());

    let total = files_array.map(|a| a.len()).unwrap_or(0);
    crate::log_info!("[NeoForge] Total versions in API: {}", total);

    let mut versions = Vec::new();
    if let Some(files) = files_array {
        for file in files {
            let version_str = if let Some(name) = file["name"].as_str() {
                name
            } else if let Some(s) = file.as_str() {
                s
            } else {
                continue;
            };

            if is_compatible(version_str, mc_version) {
                let is_beta = version_str.contains("beta") || version_str.contains("alpha");
                versions.push(LoaderVersion {
                    version: version_str.to_string(),
                    is_recommended: !is_beta,
                    release_time: None,
                });
            }
        }
    }

    // 检查旧版格式
    let legacy_urls = sources::build_urls(
        mirror_url,
        sources::NEOFORGE_API_LEGACY,
        sources::BMCLAPI_NEOFORGE_LEGACY,
        source_mode,
    );
    if let Ok(legacy_content) = sources::fetch_with_fallback(&legacy_urls).await {
        if let Ok(legacy_json) = serde_json::from_str::<serde_json::Value>(&legacy_content) {
            let legacy_files = legacy_json["files"]
                .as_array()
                .or_else(|| legacy_json["versions"].as_array());
            if let Some(files) = legacy_files {
                for file in files {
                    let version_str = if let Some(name) = file["name"].as_str() {
                        name
                    } else if let Some(s) = file.as_str() {
                        s
                    } else {
                        continue;
                    };
                    let prefix = format!("{}-", mc_version);
                    if version_str.starts_with(&prefix) {
                        let loader_version =
                            version_str.strip_prefix(&prefix).unwrap_or(version_str);
                        versions.push(LoaderVersion {
                            version: loader_version.to_string(),
                            is_recommended: true,
                            release_time: None,
                        });
                    }
                }
            }
        }
    }

    versions.sort_by(|a, b| {
        let v_a = crate::utils::version::parse_number(&a.version);
        let v_b = crate::utils::version::parse_number(&b.version);
        v_b.cmp(&v_a)
    });

    crate::log_info!("[NeoForge] Final result: {} versions", versions.len());
    crate::log_separator!("NeoForge End");
    Ok(versions)
}

/// Check NeoForge version compatibility
fn is_compatible(neoforge_version: &str, mc_version: &str) -> bool {
    let mc_parts: Vec<&str> = mc_version.split('.').collect();
    if mc_parts.len() < 2 {
        return false;
    }

    let mc_major: u32 = mc_parts[0].parse().unwrap_or(0);
    let mc_minor: u32 = mc_parts[1].parse().unwrap_or(0);

    if neoforge_version.starts_with("0.") {
        return false;
    }

    let neoforge_parts: Vec<&str> = neoforge_version.split('.').collect();
    if neoforge_parts.len() < 2 {
        return false;
    }

    let neoforge_major: u32 = match neoforge_parts[0].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    let neoforge_minor: u32 = match neoforge_parts[1].parse() {
        Ok(v) => v,
        Err(_) => return false,
    };

    if mc_major == 1 {
        neoforge_major == mc_minor
    } else {
        neoforge_major == mc_major && neoforge_minor == mc_minor
    }
}

/// Install NeoForge
pub async fn install(
    mc_version: &str,
    neoforge_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    config: &DownloadManagerConfig,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    log_info!(
        "[NeoForge] Installing {} for MC {}",
        neoforge_version,
        mc_version
    );

    let file_name = format!("neoforge-{}-installer.jar", neoforge_version);
    let installer_url = sources::neoforge_installer_url(neoforge_version);
    let temp_dir = crate::utils::cache_temp::ensure_task_temp_dir()?;
    let installer_path = temp_dir.join(&file_name);

    let hash_url = format!("{}.sha1", installer_url);
    let expected_hash = match crate::http::fetch_url(&hash_url).await {
        Ok(hash) => Some(hash.trim().to_string()),
        Err(_) => None,
    };

    let urls = sources::build_replace_urls(
        &installer_url,
        mirror_url,
        sources::MAVEN_REPLACEMENTS,
        config.source_mode,
    );

    let manager = DownloadManager::from_config(config);
    let task = DownloadTask {
        id: "neoforge_installer".to_string(),
        urls,
        local_path: installer_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download NeoForge installer"));
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(10.0);
    }

    let version_id = format!("{}-neoforge-{}", mc_version, neoforge_version);

    launcher_profiles::ensure_profiles_exist(game_dir).map_err(|e: String| anyhow::anyhow!(e))?;

    if let Some(ref cb) = progress_callback {
        cb(20.0);
    }

    if let Err(e) =
        shared::download_mojang_mappings(mc_version, game_dir, &installer_path, config).await
    {
        log_warn!("[NeoForge] Failed to download mappings: {}", e);
    }

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    log_info!("[NeoForge] Using injector");
    let (injector_path, wrapper_path) = super::forge_installer::extract_embedded_resources()?;

    if let Some(ref cb) = progress_callback {
        cb(40.0);
    }

    let java_path = shared::find_java_for_install(game_dir)?;

    if let Some(ref cb) = progress_callback {
        cb(50.0);
    }

    super::forge_installer::run_forge_installer(
        &java_path,
        &installer_path.to_string_lossy(),
        &injector_path,
        &wrapper_path,
        &game_dir.to_string_lossy(),
        true,
        None,
    )?;

    if let Some(ref cb) = progress_callback {
        cb(80.0);
    }

    shared::copy_generated_version_json(game_dir, mc_version, &version_id, "neoforge");

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[NeoForge] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}
