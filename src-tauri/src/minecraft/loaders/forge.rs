//! Forge loader module

use crate::{log_info, log_warn};
use std::path::Path;
use std::sync::Arc;

use super::shared;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadTask, DownloadStatus};
use crate::minecraft::launcher_profiles;
use crate::minecraft::sources::{self, DownloadSourceMode};
use super::{LoaderVersion, utils};

/// List Forge versions
pub async fn list_versions(mc_version: &str, mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(
        mirror_url,
        &sources::forge_versions_url(mc_version),
        &format!("/forge/minecraft/{}", mc_version),
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;

    // 尝试 BMCLAPI JSON 格式
    if let Ok(json_array) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
        let mut versions: Vec<LoaderVersion> = json_array.iter().filter_map(|v| {
            let version = v["version"].as_str()?;
            let modified = v["modified"].as_str();
            let release_time = modified.and_then(|s| utils::parse_utc_to_local(s));
            Some(LoaderVersion {
                version: version.to_string(),
                is_recommended: v["category"].as_str() == Some("recommended"),
                release_time,
            })
        }).collect();

        versions.sort_by(|a, b| {
            let v_a = utils::parse_version_number(&a.version);
            let v_b = utils::parse_version_number(&b.version);
            v_b.cmp(&v_a)
        });

        return Ok(versions);
    }

    // 官方源 HTML 格式解析
    super::forge_html::parse_forge_version_html(&content)
}

/// Install Forge
pub async fn install(
    mc_version: &str,
    forge_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    let file_name = format!("forge-{}-{}-installer.jar", mc_version, forge_version);
    let installer_url = sources::forge_installer_url(mc_version, forge_version);
    let temp_dir = std::env::temp_dir().join("MoLaunch").join("TaskTemp");
    std::fs::create_dir_all(&temp_dir)?;
    let installer_path = temp_dir.join(&file_name);

    // 尝试获取文件 hash
    let hash_url = format!("{}.sha1", installer_url);
    let expected_hash = match crate::http::fetch_url(&hash_url).await {
        Ok(hash) => Some(hash.trim().to_string()),
        Err(_) => None,
    };

    // Download installer
    let urls = sources::build_replace_urls(&installer_url, mirror_url, sources::MAVEN_REPLACEMENTS, source_mode);

    let manager = DownloadManager::new(1, 0, 0, source_mode);
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
    if super::forge_installer::needs_injector(forge_version, false) {
        install_modern(mc_version, forge_version, &installer_path, game_dir, progress_callback, source_mode).await
    } else {
        install_legacy(mc_version, forge_version, &installer_path, game_dir, progress_callback).await
    }
}

/// Legacy Forge installation (1.12.2 and below)
async fn install_legacy(
    mc_version: &str,
    forge_version: &str,
    installer_path: &Path,
    game_dir: &Path,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
) -> anyhow::Result<()> {
    use std::io::Read;

    log_info!("[Forge] Legacy 安装 {} for MC {}", forge_version, mc_version);

    let version_id = format!("{}-forge-{}", mc_version, forge_version);
    let version_dir = game_dir.join("versions").join(&version_id);
    std::fs::create_dir_all(&version_dir)?;

    let installer_file = std::fs::File::open(installer_path)?;
    let mut zip = zip::ZipArchive::new(installer_file)?;

    let profile_json: serde_json::Value = {
        let mut entry = zip.by_name("install_profile.json")?;
        let mut content = String::new();
        entry.read_to_string(&mut content)?;
        serde_json::from_str(&content)?
    };

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    if profile_json.get("install").is_some() {
        // Legacy 方式 2（1.7.10 及更早）
        log_info!("[Forge] Legacy 方式 2: {}", forge_version);
        let install = &profile_json["install"];

        let file_path = install["filePath"].as_str()
            .ok_or_else(|| anyhow::anyhow!("install.filePath not found"))?;
        let lib_path = install["path"].as_str()
            .ok_or_else(|| anyhow::anyhow!("install.path not found"))?;

        let jar_dest = shared::maven_path_to_local(lib_path, game_dir);
        if let Some(parent) = jar_dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        {
            let mut entry = zip.by_name(file_path)?;
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            std::fs::write(&jar_dest, buf)?;
        }
        log_info!("[Forge] 提取 JAR: {} -> {}", file_path, jar_dest.display());

        if let Some(ref cb) = progress_callback {
            cb(60.0);
        }

        let version_info = profile_json.get("versionInfo")
            .ok_or_else(|| anyhow::anyhow!("versionInfo not found"))?;
        let mut version_json = version_info.clone();

        version_json["id"] = serde_json::Value::String(version_id.clone());
        if version_json.get("inheritsFrom").is_none() {
            version_json["inheritsFrom"] = serde_json::Value::String(mc_version.to_string());
        }

        let json_path = version_dir.join(format!("{}.json", version_id));
        std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;
        log_info!("[Forge] 写入版本 JSON: {}", json_path.display());
    } else {
        // Legacy 方式 1（1.8 ~ 1.12.2）
        log_info!("[Forge] Legacy 方式 1: {}", forge_version);

        let json_entry_name = profile_json["json"].as_str()
            .ok_or_else(|| anyhow::anyhow!("install_profile.json 中缺少 json 字段"))?
            .trim_start_matches('/');

        let mut version_json: serde_json::Value = {
            let mut entry = zip.by_name(json_entry_name)?;
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            serde_json::from_str(&content)?
        };

        version_json["id"] = serde_json::Value::String(version_id.clone());

        let json_path = version_dir.join(format!("{}.json", version_id));
        std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;
        log_info!("[Forge] 写入版本 JSON: {}", json_path.display());

        if let Some(ref cb) = progress_callback {
            cb(50.0);
        }

        // 解压 maven/ 文件夹到 libraries/
        let maven_dest = game_dir.join("libraries");
        let maven_entries: Vec<String> = zip.file_names()
            .filter(|name| name.starts_with("maven/"))
            .map(|s| s.to_string())
            .collect();

        let mut extracted_count = 0;
        for entry_name in &maven_entries {
            let relative_path = entry_name.strip_prefix("maven/").unwrap_or(entry_name);
            if relative_path.is_empty() { continue; }
            let dest_path = maven_dest.join(relative_path);

            // Zip Slip 防护：校验最终路径仍在 maven_dest 内
            let canonical_base = maven_dest.canonicalize().unwrap_or_else(|_| maven_dest.to_path_buf());
            let canonical_dest = dest_path.canonicalize().unwrap_or_else(|_| dest_path.clone());
            if !canonical_dest.starts_with(&canonical_base) {
                crate::log_warn!("[Forge] Skip path traversal entry: {}", entry_name);
                continue;
            }

            let mut entry = zip.by_name(entry_name)?;
            if entry.is_dir() {
                std::fs::create_dir_all(&dest_path)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                std::fs::write(&dest_path, buf)?;
                extracted_count += 1;
            }
        }
        log_info!("[Forge] 解压 maven/ 到 libraries/: {} 个文件", extracted_count);

        // 复制原版 JAR 到 Forge 版本目录（Legacy 方式 1 需要）
        let mc_jar = game_dir.join("versions").join(mc_version).join(format!("{}.jar", mc_version));
        let forge_jar = version_dir.join(format!("{}.jar", version_id));
        if mc_jar.exists() && !forge_jar.exists() {
            if let Err(e) = std::fs::copy(&mc_jar, &forge_jar) {
                log_warn!("[Forge] Failed to copy MC JAR: {}", e);
            } else {
                log_info!("[Forge] Copied MC JAR: {} -> {}", mc_jar.display(), forge_jar.display());
            }
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[Forge] Legacy 安装完成: {}", version_id);
    Ok(())
}

/// Modern Forge installation (1.13+)
async fn install_modern(
    mc_version: &str,
    forge_version: &str,
    installer_path: &Path,
    game_dir: &Path,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    log_info!("[Forge] Installing {} for MC {}", forge_version, mc_version);

    let version_id = format!("{}-forge-{}", mc_version, forge_version);

    launcher_profiles::ensure_profiles_exist(game_dir)
        .map_err(|e: String| anyhow::anyhow!(e))?;

    if let Some(ref cb) = progress_callback {
        cb(20.0);
    }

    // 下载 Mojang 映射文件
    if let Err(e) = shared::download_mojang_mappings(mc_version, game_dir, installer_path, source_mode).await {
        log_warn!("[Forge] Failed to download mappings: {}", e);
    }

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    log_info!("[Forge] Using injector for Forge {}", forge_version);

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
