//! Shared utilities for loader installation

use crate::{log_info, log_warn, log_error};
use std::path::{Path, PathBuf};

use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadTask, DownloadStatus};
use crate::minecraft::sources::{self, DownloadSourceMode};

/// Find Java for installation (minimum Java 8u60)
pub fn find_java_for_install(_game_dir: &Path) -> anyhow::Result<String> {
    let java_list = super::super::java::search_java();

    if !java_list.is_empty() {
        if let Some(java_path) = super::super::java_selector::get_java_for_installer(&java_list) {
            log_info!("[Java] 使用自动检测的 Java: {}", java_path);
            return Ok(java_path);
        }
    }

    // Fallback: try PATH
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let java_path = dir.join("java.exe");
            if java_path.exists() {
                return Ok(java_path.to_string_lossy().to_string());
            }
        }
    }

    Err(anyhow::anyhow!("Java not found. Please install Java 8+ to install Forge/NeoForge."))
}

/// Convert Maven coordinate to local file path
pub fn maven_path_to_local(maven_path: &str, game_dir: &Path) -> PathBuf {
    let parts: Vec<&str> = maven_path.split(':').collect();
    let libs_dir = game_dir.join("libraries");

    if parts.len() >= 3 {
        let group = parts[0].replace('.', "/");
        let artifact = parts[1];
        let version = parts[2];
        let classifier = if parts.len() >= 4 { parts[3] } else { "" };

        let dir_path = libs_dir.join(&group).join(artifact).join(version);
        let file_name = if classifier.is_empty() {
            format!("{}-{}.jar", artifact, version)
        } else {
            format!("{}-{}-{}.jar", artifact, version, classifier)
        };

        dir_path.join(file_name)
    } else {
        libs_dir.join(maven_path.replace(':', "/"))
    }
}

/// Download Mojang mappings (required for Forge/NeoForge >= 20)
pub async fn download_mojang_mappings(
    mc_version: &str,
    game_dir: &Path,
    installer_path: &Path,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut install_profile_content = String::new();
    {
        let mut entry = archive.by_name("install_profile.json")?;
        std::io::Read::read_to_string(&mut entry, &mut install_profile_content)?;
    }

    let install_profile: serde_json::Value = serde_json::from_str(&install_profile_content)?;

    let mojmaps = match install_profile["data"]["MOJMAPS"]["client"].as_str() {
        Some(s) => s,
        None => {
            log_info!("[Mappings] No MOJMAPS data found in install_profile.json");
            return Ok(());
        }
    };

    let mojmaps_clean = mojmaps.trim_start_matches('[').trim_end_matches(']');
    let original_name = mojmaps_clean.split('@').next().unwrap_or("");
    let extension = mojmaps_clean.split('@').nth(1).unwrap_or("txt").trim_end_matches(']');

    let parts: Vec<&str> = original_name.split(':').collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Invalid MOJMAPS format: {}", mojmaps));
    }

    let group = parts[0];
    let artifact = parts[1];
    let version = parts[2];

    let group_path = group.replace('.', std::path::MAIN_SEPARATOR_STR);
    let local_dir = game_dir.join("libraries").join(group_path).join(artifact).join(version);
    let filename = format!("{}-{}-mappings.{}", artifact, version, extension);
    let local_path = local_dir.join(&filename);

    if local_path.exists() {
        log_info!("[Mappings] File already exists: {}", local_path.display());
        return Ok(());
    }

    let version_list = super::super::download::fetch_version_list(None, source_mode).await?;
    let json_url = super::super::download::get_version_json_url(&version_list.value, mc_version)
        .ok_or_else(|| anyhow::anyhow!("Version {} not found", mc_version))?;

    let json_content = super::super::download::fetch_url(&json_url).await?;
    let version_json: serde_json::Value = serde_json::from_str(&json_content)?;

    let mappings = version_json["downloads"]["client_mappings"].as_object()
        .ok_or_else(|| anyhow::anyhow!("client_mappings not found"))?;

    let url = mappings["url"].as_str()
        .ok_or_else(|| anyhow::anyhow!("client_mappings URL not found"))?;
    let sha1 = mappings["sha1"].as_str().unwrap_or_default();
    let size = mappings["size"].as_i64().unwrap_or(0);

    std::fs::create_dir_all(&local_dir)?;

    let urls = sources::build_replace_urls(url, None, sources::MOJANG_REPLACEMENTS, source_mode);

    let manager = DownloadManager::new(1, 0, 0, source_mode);
    let task = DownloadTask {
        id: "mappings".to_string(),
        urls,
        local_path: local_path.to_string_lossy().to_string(),
        expected_size: size,
        expected_hash: if sha1.is_empty() { None } else { Some(sha1.to_string()) },
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download Mojang mappings"));
        }
    }

    log_info!("[Mappings] Downloaded: {}", local_path.display());
    Ok(())
}

/// Copy generated version JSON from installer output
pub fn copy_generated_version_json(game_dir: &Path, mc_version: &str, version_id: &str, loader_keyword: &str) {
    let versions_dir = game_dir.join("versions");
    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.contains(loader_keyword) && dir_name.contains(mc_version) {
                    let json_files: Vec<_> = std::fs::read_dir(&path)
                        .ok()
                        .map(|e| e.filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                            .collect())
                        .unwrap_or_default();

                    if let Some(json_file) = json_files.first() {
                        let version_dir = game_dir.join("versions").join(version_id);
                        std::fs::create_dir_all(&version_dir).ok();
                        let target_json = version_dir.join(format!("{}.json", version_id));

                        // Retry copy to handle file locking
                        for retry in 0..3 {
                            match std::fs::copy(json_file.path(), &target_json) {
                                Ok(_) => {
                                    log_info!("[{}] Copied version JSON from {}", loader_keyword, path.display());
                                    return;
                                }
                                Err(e) => {
                                    if retry < 2 {
                                        log_warn!("[{}] Copy failed (retry {}): {}", loader_keyword, retry + 1, e);
                                        std::thread::sleep(std::time::Duration::from_millis(500));
                                    } else {
                                        log_error!("[{}] Copy failed after retries: {}", loader_keyword, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Copy MC JAR to loader version folder
pub fn copy_mc_jar(game_dir: &Path, mc_version: &str, version_id: &str) {
    let mc_jar = game_dir.join("versions").join(mc_version).join(format!("{}.jar", mc_version));
    let target_jar = game_dir.join("versions").join(version_id).join(format!("{}.jar", version_id));

    if mc_jar.exists() && !target_jar.exists() {
        if let Err(e) = std::fs::copy(&mc_jar, &target_jar) {
            log_warn!("[Loader] Failed to copy MC JAR: {}", e);
        } else {
            log_info!("[Loader] Copied MC JAR to {}", target_jar.display());
        }
    }
}
