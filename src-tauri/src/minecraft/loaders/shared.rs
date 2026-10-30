//! Shared utilities for loader installation

use crate::{log_error, log_info, log_warn};
use std::path::{Path, PathBuf};

use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask};
use crate::minecraft::sources;

/// 找一个能跑安装器（含 processors）的 Java
///
/// 最低版本由目标 MC 版本决定：新版 Forge/NeoForge 的 processors 是
/// 按 Java 21 编译的，用 Java 8 跑只会让产物生成失败而安装器未必报错。
pub fn find_java_for_install(_game_dir: &Path, mc_version: &str) -> anyhow::Result<String> {
    let min_major = super::super::java_selector::required_java_major(mc_version);
    let java_list = super::super::java::search_java();

    if !java_list.is_empty() {
        if let Some(java_path) =
            super::super::java_selector::get_java_for_installer_with(&java_list, min_major)
        {
            log_info!(
                "[Java] 使用自动检测的 Java 安装加载器: {}（要求 Java {}+）",
                java_path,
                min_major
            );
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

    Err(anyhow::anyhow!(
        "安装 {} 需要 Java {} 或更高版本，但没有找到合适的 Java。请先在「设置 → Java」中安装或选择一个 Java {} 后重试。",
        mc_version,
        min_major,
        min_major
    ))
}

/// Convert Maven coordinate to local file path
pub fn maven_path_to_local(maven_path: &str, game_dir: &Path) -> PathBuf {
    crate::minecraft::utils::maven::maven_to_local_path(maven_path, game_dir)
}

/// Download Mojang mappings (required for Forge/NeoForge >= 20)
pub async fn download_mojang_mappings(
    mc_version: &str,
    game_dir: &Path,
    installer_path: &Path,
    config: &DownloadManagerConfig,
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
    let extension = mojmaps_clean
        .split('@')
        .nth(1)
        .unwrap_or("txt")
        .trim_end_matches(']');

    let parts: Vec<&str> = original_name.split(':').collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Invalid MOJMAPS format: {}", mojmaps));
    }

    let group = parts[0];
    let artifact = parts[1];
    let version = parts[2];

    let group_path = group.replace('.', std::path::MAIN_SEPARATOR_STR);
    let local_dir = game_dir
        .join("libraries")
        .join(group_path)
        .join(artifact)
        .join(version);
    let filename = format!("{}-{}-mappings.{}", artifact, version, extension);
    let local_path = local_dir.join(&filename);

    if local_path.exists() {
        log_info!("[Mappings] File already exists: {}", local_path.display());
        return Ok(());
    }

    let version_list = super::super::download::fetch_version_list(None, config.source_mode).await?;
    let json_url = super::super::download::get_version_json_url(&version_list.value, mc_version)
        .ok_or_else(|| anyhow::anyhow!("Version {} not found", mc_version))?;

    let json_content = super::super::download::fetch_url(&json_url).await?;
    let version_json: serde_json::Value = serde_json::from_str(&json_content)?;

    let mappings = version_json["downloads"]["client_mappings"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("client_mappings not found"))?;

    let url = mappings["url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("client_mappings URL not found"))?;
    let sha1 = mappings["sha1"].as_str().unwrap_or_default();
    let size = mappings["size"].as_i64().unwrap_or(0);

    std::fs::create_dir_all(&local_dir)?;

    let urls =
        sources::build_replace_urls(url, None, sources::MOJANG_REPLACEMENTS, config.source_mode);

    let manager = DownloadManager::from_config(config);
    let task = DownloadTask {
        id: "mappings".to_string(),
        urls,
        local_path: local_path.to_string_lossy().to_string(),
        expected_size: size,
        expected_hash: if sha1.is_empty() {
            None
        } else {
            Some(sha1.to_string())
        },
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

/// 预下载库数量上限（防止极端 profile 拉长安装时间）
const MAX_PREFETCH_LIBRARIES: usize = 400;

/// 用启动器自己的下载器预下载安装器需要的库
///
/// 安装器跟随重定向的次数有限，镜像链路一长就会报 `Invalid number of redirects` 直接失败。
/// 我们的下载器支持多源与重试，先把 `install_profile.json` 里声明了下载地址的库备好，
/// 安装器命中「文件已存在，校验通过」就会跳过下载。
///
/// 只处理主 artifact：natives、旧版安装器（无 downloads 字段）一律跳过，不猜地址、不碰生成物。
/// 返回成功预下载的数量；单个库失败不阻断安装。
pub async fn prefetch_install_libraries(
    installer_path: &Path,
    game_dir: &Path,
    config: &DownloadManagerConfig,
) -> anyhow::Result<usize> {
    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let mut content = String::new();
    {
        let mut entry = archive.by_name("install_profile.json")?;
        std::io::Read::read_to_string(&mut entry, &mut content)?;
    }

    let profile: serde_json::Value = serde_json::from_str(&content)?;
    let libraries = profile["libraries"].as_array().cloned().unwrap_or_default();

    let mut tasks: Vec<DownloadTask> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (index, library) in libraries.iter().enumerate() {
        if tasks.len() >= MAX_PREFETCH_LIBRARIES {
            log_warn!(
                "[Installer] 预下载库数量达到上限 {}，其余交给安装器",
                MAX_PREFETCH_LIBRARIES
            );
            break;
        }
        let Some(name) = library["name"].as_str() else {
            continue;
        };
        let artifact = &library["downloads"]["artifact"];
        let Some(official_url) = artifact["url"].as_str() else {
            continue;
        };
        let relative = artifact["path"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| crate::minecraft::utils::maven::maven_to_relative_path(name));
        if !relative.ends_with(".jar") || !seen.insert(relative.clone()) {
            continue;
        }

        let local_path = game_dir.join("libraries").join(&relative);
        if local_path.exists() {
            continue;
        }

        let urls = sources::build_replace_urls(
            official_url,
            None,
            sources::MAVEN_REPLACEMENTS,
            config.source_mode,
        );
        if urls.is_empty() {
            continue;
        }

        tasks.push(DownloadTask {
            id: format!("installer-lib-{}", index),
            urls,
            local_path: local_path.to_string_lossy().to_string(),
            expected_size: artifact["size"].as_i64().unwrap_or(0),
            expected_hash: artifact["sha1"].as_str().map(str::to_string),
        });
    }

    if tasks.is_empty() {
        return Ok(0);
    }

    log_info!("[Installer] 预下载安装库 {} 个", tasks.len());
    let manager = DownloadManager::from_config(config);
    let total = tasks.len();
    let results = manager.download_batch(tasks, None).await;
    let ok = results
        .iter()
        .filter(|result| result.status != DownloadStatus::Failed)
        .count();
    if ok < total {
        log_warn!(
            "[Installer] 预下载安装库：成功 {} / {}，其余交给安装器处理",
            ok,
            total
        );
    }
    Ok(ok)
}

/// Find generated version JSON files in installer output.
fn find_generated_version_jsons(
    versions_dir: &Path,
    mc_version: &str,
    loader_keyword: &str,
) -> Vec<(PathBuf, PathBuf)> {
    let mut generated_files = Vec::new();
    let Ok(entries) = std::fs::read_dir(versions_dir) else {
        return generated_files;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
        if !dir_name.contains(loader_keyword) || !dir_name.contains(mc_version) {
            continue;
        }

        let Some(json_file) = std::fs::read_dir(&path).ok().and_then(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .find(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        }) else {
            continue;
        };

        generated_files.push((path, json_file.path()));
    }

    generated_files
}

/// Copy a generated version JSON, retrying to handle file locking.
fn copy_generated_version_json_with_retries(
    source_json: &Path,
    target_json: &Path,
    source_dir: &Path,
    loader_keyword: &str,
) -> bool {
    for retry in 0..3 {
        match std::fs::copy(source_json, target_json) {
            Ok(_) => {
                log_info!(
                    "[{}] Copied version JSON from {}",
                    loader_keyword,
                    source_dir.display()
                );
                return true;
            }
            Err(e) => {
                if retry < 2 {
                    log_warn!(
                        "[{}] Copy failed (retry {}): {}",
                        loader_keyword,
                        retry + 1,
                        e
                    );
                    std::thread::sleep(std::time::Duration::from_millis(500));
                } else {
                    log_error!("[{}] Copy failed after retries: {}", loader_keyword, e);
                }
            }
        }
    }

    false
}

/// Copy generated version JSON from installer output
pub fn copy_generated_version_json(
    game_dir: &Path,
    mc_version: &str,
    version_id: &str,
    loader_keyword: &str,
) {
    let versions_dir = game_dir.join("versions");
    let generated_files = find_generated_version_jsons(&versions_dir, mc_version, loader_keyword);
    let version_dir = game_dir.join("versions").join(version_id);
    let target_json = version_dir.join(format!("{}.json", version_id));

    for (source_dir, source_json) in generated_files {
        std::fs::create_dir_all(&version_dir).ok();
        if copy_generated_version_json_with_retries(
            &source_json,
            &target_json,
            &source_dir,
            loader_keyword,
        ) {
            return;
        }
    }
}

/// Copy MC JAR to loader version folder
pub fn copy_mc_jar(game_dir: &Path, mc_version: &str, version_id: &str) {
    let mc_jar = game_dir
        .join("versions")
        .join(mc_version)
        .join(format!("{}.jar", mc_version));
    let target_jar = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}.jar", version_id));

    if mc_jar.exists() && !target_jar.exists() {
        if let Err(e) = std::fs::copy(&mc_jar, &target_jar) {
            log_warn!("[Loader] Failed to copy MC JAR: {}", e);
        } else {
            log_info!("[Loader] Copied MC JAR to {}", target_jar.display());
        }
    }
}
