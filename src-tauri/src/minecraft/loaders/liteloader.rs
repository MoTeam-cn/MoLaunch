//! LiteLoader loader module

use std::path::Path;
use std::sync::Arc;

use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadTask, DownloadStatus};
use crate::minecraft::sources::{self, DownloadSourceMode};
use super::LoaderVersion;

/// List LiteLoader versions
pub async fn list_versions(mc_version: &str, mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(mirror_url, sources::LITELOADER_VERSIONS, sources::BMCLAPI_LITELOADER, source_mode);
    let content = sources::fetch_with_fallback(&urls).await?;
    parse_versions(&content, mc_version)
}

/// Parse LiteLoader versions from JSON
fn parse_versions(content: &str, mc_version: &str) -> anyhow::Result<Vec<LoaderVersion>> {
    let json: serde_json::Value = serde_json::from_str(content)?;
    let mut versions = Vec::new();

    if let Some(mc_versions) = json["versions"].as_object() {
        if let Some(mc_version_data) = mc_versions.get(mc_version) {
            let artefacts = mc_version_data.get("artefacts")
                .or_else(|| mc_version_data.get("snapshots"));

            if let Some(artefacts) = artefacts {
                if let Some(liteloader) = artefacts.get("com.mumfrey:liteloader") {
                    if let Some(latest) = liteloader.get("latest") {
                        let stream = latest["stream"].as_str().unwrap_or("release");
                        versions.push(LoaderVersion {
                            version: mc_version.to_string(),
                            is_recommended: stream == "release",
                            release_time: None,
                        });
                    }
                }
            }
        }
    }

    Ok(versions)
}

/// Install LiteLoader
pub async fn install(
    mc_version: &str,
    liteloader_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback { cb(0.0); }

    crate::log_info!("[LiteLoader] Installing {} for MC {}", liteloader_version, mc_version);

    let version_id = format!("{}-LiteLoader", mc_version);
    let version_dir = game_dir.join("versions").join(&version_id);
    std::fs::create_dir_all(&version_dir)?;

    let url = sources::liteloader_json_url(mc_version, liteloader_version);

    let urls = match mirror_url {
        Some(mirror) if !mirror.is_empty() => vec![
            format!("{}/maven/com/mumfrey/liteloader/{}/liteloader-{}-{}.json", mirror.trim_end_matches('/'), mc_version, mc_version, liteloader_version),
            format!("{}/maven/com/mumfrey/liteloader/{}/liteloader-{}-{}.json", sources::BMCLAPI_BASE, mc_version, mc_version, liteloader_version),
            url,
        ],
        _ => vec![
            format!("{}/maven/com/mumfrey/liteloader/{}/liteloader-{}-{}.json", sources::BMCLAPI_BASE, mc_version, mc_version, liteloader_version),
            url,
        ],
    };

    let manager = DownloadManager::new(1, 0, 0, source_mode);
    let task = DownloadTask {
        id: "liteloader_json".to_string(),
        urls,
        local_path: version_dir.join(format!("{}.json", version_id)).to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download LiteLoader JSON"));
        }
    }

    crate::log_info!("[LiteLoader] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback { cb(100.0); }

    Ok(())
}
