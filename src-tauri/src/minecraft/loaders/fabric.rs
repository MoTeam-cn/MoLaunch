//! Fabric loader module

use std::path::Path;
use std::sync::Arc;

use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadTask, DownloadStatus};
use crate::minecraft::sources::{self, DownloadSourceMode};
use super::LoaderVersion;

/// List Fabric versions
pub async fn list_versions(mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(mirror_url, sources::FABRIC_META, sources::BMCLAPI_FABRIC_META, source_mode);
    let content = sources::fetch_with_fallback(&urls).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let mut versions = Vec::new();
    if let Some(versions_array) = json.as_array() {
        for version in versions_array {
            if let Some(version_str) = version["version"].as_str() {
                versions.push(LoaderVersion {
                    version: version_str.to_string(),
                    is_recommended: version["stable"].as_bool().unwrap_or(false),
                    release_time: None,
                });
            }
        }
    }

    Ok(versions)
}

/// Install Fabric
pub async fn install(
    mc_version: &str,
    fabric_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback { cb(0.0); }

    crate::log_info!("[Fabric] Installing {} for MC {}", fabric_version, mc_version);

    let version_id = format!("fabric-{}-{}", fabric_version, mc_version);
    let version_dir = game_dir.join("versions").join(&version_id);
    std::fs::create_dir_all(&version_dir)?;

    let url = sources::fabric_profile_url(mc_version, fabric_version);

    let urls = match mirror_url {
        Some(mirror) if !mirror.is_empty() => vec![
            format!("{}/fabric-meta/v2/versions/loader/{}/{}/profile/json", mirror.trim_end_matches('/'), mc_version, fabric_version),
            format!("{}/fabric-meta/v2/versions/loader/{}/{}/profile/json", sources::BMCLAPI_BASE, mc_version, fabric_version),
            url,
        ],
        _ => vec![
            format!("{}/fabric-meta/v2/versions/loader/{}/{}/profile/json", sources::BMCLAPI_BASE, mc_version, fabric_version),
            url,
        ],
    };

    let manager = DownloadManager::new(1, 0, 0, source_mode);
    let task = DownloadTask {
        id: "fabric_profile".to_string(),
        urls,
        local_path: version_dir.join(format!("{}.json", version_id)).to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download Fabric profile"));
        }
    }

    crate::log_info!("[Fabric] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback { cb(100.0); }

    Ok(())
}
