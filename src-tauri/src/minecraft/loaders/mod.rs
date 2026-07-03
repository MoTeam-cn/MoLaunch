//! Loader management module

pub mod forge_installer;

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::download::{self, manager::{DownloadManager, DownloadSourceMode, DownloadTask, DownloadStatus}};

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
pub async fn list_forge_versions(mc_version: &str, _mirror_url: Option<&str>) -> anyhow::Result<Vec<LoaderVersion>> {
    let bmclapi_url = format!("https://bmclapi2.bangbang93.com/forge/minecraft/{}", mc_version);

    let content = match download::fetch_url(&bmclapi_url).await {
        Ok(c) => c,
        Err(_) => {
            let official_url = format!("https://files.minecraftforge.net/maven/net/minecraftforge/forge/index_{}.html", mc_version);
            download::fetch_url(&official_url).await?
        }
    };

    if let Ok(json_array) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
        let mut versions: Vec<LoaderVersion> = json_array.iter().filter_map(|v| {
            let version = v["version"].as_str()?;
            let modified = v["modified"].as_str();
            
            // 格式化发布时�?
            let release_time = modified.map(|s| {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    dt.format("%Y/%m/%d %H:%M").to_string()
                } else {
                    s.to_string()
                }
            });
            
            Some(LoaderVersion {
                version: version.to_string(),
                is_recommended: v["category"].as_str() == Some("recommended"),
                release_time,
            })
        }).collect();
        
        // 按版本号降序排列（最新在前）
        versions.sort_by(|a, b| {
            let v_a = parse_version_number(&a.version);
            let v_b = parse_version_number(&b.version);
            v_b.cmp(&v_a) // 降序
        });
        
        return Ok(versions);
    }

    Ok(vec![])
}

/// 解析版本号为可比较的元组
fn parse_version_number(version: &str) -> Vec<u32> {
    version.split('.')
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// List NeoForge versions
pub async fn list_neoforge_versions(mc_version: &str, mirror_url: Option<&str>) -> anyhow::Result<Vec<LoaderVersion>> {
    crate::log_separator!("NeoForge List");
    crate::log_info!("[NeoForge] Listing versions for MC {}", mc_version);

    let url = match mirror_url {
        Some(mirror) if !mirror.is_empty() => format!(
            "{}/neoforge/meta/api/maven/details/releases/net/neoforged/neoforge",
            mirror.trim_end_matches('/')
        ),
        _ => "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge".to_string(),
    };

    crate::log_debug!("[NeoForge] Fetching from: {}", url);
    let content = match download::fetch_url(&url).await {
        Ok(c) => {
            crate::log_debug!("[NeoForge] Response length: {} bytes", c.len());
            crate::log_debug!("[NeoForge] Response preview: {}", &c[..c.len().min(200)]);
            c
        }
        Err(e) => {
            crate::log_error!("[NeoForge] Fetch failed: {}", e);
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

    // BMCLAPI 返回的是 files 数组，不是 versions
    let files_array = json["files"].as_array()
        .or_else(|| json["versions"].as_array()); // 兼容两种格式

    let total = files_array.map(|a| a.len()).unwrap_or(0);
    crate::log_info!("[NeoForge] Total versions in API: {}", total);

    let mut versions = Vec::new();
    if let Some(files) = files_array {
        for file in files {
            // BMCLAPI 格式: {"name": "26.2.0.0-beta", "type": "DIRECTORY"}
            // 官方格式: "26.2.0.0-beta"
            let version_str = if let Some(name) = file["name"].as_str() {
                name
            } else if let Some(s) = file.as_str() {
                s
            } else {
                continue;
            };

            let compatible = is_neoforge_compatible(version_str, mc_version);
            crate::log_trace!("[NeoForge] Check {} -> {}", version_str, compatible);
            if compatible {
                let is_beta = version_str.contains("beta") || version_str.contains("alpha");
                versions.push(LoaderVersion {
                    version: version_str.to_string(),
                    is_recommended: !is_beta,
                    release_time: None,
                });
            }
        }
    }

    crate::log_info!("[NeoForge] Compatible versions: {}", versions.len());
    for v in &versions {
        crate::log_debug!("[NeoForge]   - {} (recommended: {})", v.version, v.is_recommended);
    }

    // 也检查旧版格式
    let legacy_url = match mirror_url {
        Some(mirror) if !mirror.is_empty() => format!(
            "{}/neoforge/meta/api/maven/details/releases/net/neoforged/forge",
            mirror.trim_end_matches('/')
        ),
        _ => "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge".to_string(),
    };

    crate::log_debug!("[NeoForge] Fetching legacy from: {}", legacy_url);
    if let Ok(legacy_content) = download::fetch_url(&legacy_url).await {
        if let Ok(legacy_json) = serde_json::from_str::<serde_json::Value>(&legacy_content) {
            // 兼容两种格式
            let legacy_files = legacy_json["files"].as_array()
                .or_else(|| legacy_json["versions"].as_array());
                
            if let Some(files) = legacy_files {
                crate::log_debug!("[NeoForge] Legacy versions count: {}", files.len());
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
                        let loader_version = version_str.strip_prefix(&prefix).unwrap_or(version_str);
                        crate::log_debug!("[NeoForge] Found legacy: {}", loader_version);
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

    // 按版本号降序排列
    versions.sort_by(|a, b| {
        let v_a = parse_version_number(&a.version);
        let v_b = parse_version_number(&b.version);
        v_b.cmp(&v_a)
    });

    crate::log_info!("[NeoForge] Final result: {} versions", versions.len());
    crate::log_separator!("NeoForge End");
    Ok(versions)
}

fn is_neoforge_compatible(neoforge_version: &str, mc_version: &str) -> bool {
    let mc_parts: Vec<&str> = mc_version.split('.').collect();
    if mc_parts.len() < 2 { 
        return false; 
    }

    let mc_major: u32 = mc_parts[0].parse().unwrap_or(0);
    let mc_minor: u32 = mc_parts[1].parse().unwrap_or(0);

    // 跳过特殊版本（如 0.25w14craftmine.3-beta）
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

    // NeoForge 版本格式：
    // - 20.4.30 -> MC 1.20.4
    // - 24.1.0 -> MC 24.1.0  
    // - 26.2.0.0-beta -> MC 26.2
    
    if mc_major == 1 {
        // MC 1.x.y 格式：NeoForge major = MC minor
        neoforge_major == mc_minor
    } else {
        // MC x.y 格式：NeoForge major = MC major, NeoForge minor = MC minor
        neoforge_major == mc_major && neoforge_minor == mc_minor
    }
}

/// List Fabric versions
pub async fn list_fabric_versions(mirror_url: Option<&str>) -> anyhow::Result<Vec<LoaderVersion>> {
    let url = match mirror_url {
        Some(mirror) if !mirror.is_empty() => format!("{}/fabric-meta/v2/versions/loader", mirror.trim_end_matches('/')),
        _ => "https://meta.fabricmc.net/v2/versions/loader".to_string(),
    };

    let content = download::fetch_url(&url).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let mut versions = Vec::new();
    if let Some(versions_array) = json.as_array() {
        for version in versions_array {
            if let Some(version_str) = version["version"].as_str() {
                // 转换版本号：0.16.14+build.1 -> 0.16.14.1
                let formatted_version = version_str.replace("+build", "");
                versions.push(LoaderVersion {
                    version: formatted_version,
                    is_recommended: version["stable"].as_bool().unwrap_or(false),
                    release_time: None,
                });
            }
        }
    }

    Ok(versions)
}

/// List OptiFine versions
pub async fn list_optifine_versions(mirror_url: Option<&str>) -> anyhow::Result<Vec<LoaderVersion>> {
    let url = match mirror_url {
        Some(mirror) if !mirror.is_empty() => format!("{}/optifine/versionList", mirror.trim_end_matches('/')),
        _ => "https://bmclapi2.bangbang93.com/optifine/versionList".to_string(),
    };

    let content = download::fetch_url(&url).await?;

    // BMCLAPI 返回 JSON 数组
    if let Ok(json_array) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
        let mut versions: Vec<LoaderVersion> = json_array.iter().filter_map(|v| {
            let mc_ver = v["mcversion"].as_str()?;
            let type_str = v["type"].as_str().unwrap_or("");
            let patch = v["patch"].as_str().unwrap_or("");
            
            // 构建显示名称：mcversion + type(处理�? + patch
            // 例如 "1.12.2" + " HD U " + "C8" -> "1.12.2 C8"
            let type_display = type_str
                .replace("HD_U", "")
                .replace("_", " ")
                .trim()
                .to_string();
            
            let display_name = if type_display.is_empty() {
                format!("{} {}", mc_ver, patch)
            } else {
                format!("{} {} {}", mc_ver, type_display, patch)
            };
            
            let is_preview = patch.contains("pre") || patch.contains("alpha") || patch.contains("beta");
            
            Some(LoaderVersion {
                version: display_name.trim().to_string(),
                is_recommended: !is_preview,
                release_time: None,
            })
        }).collect();
        
        // 排序：正式版优先，同类型内按版本号降�?
        versions.sort_by(|a, b| {
            // 正式版优�?
            if a.is_recommended != b.is_recommended {
                return b.is_recommended.cmp(&a.is_recommended);
            }
            // 按版本号降序
            compare_optifine_version(&b.version, &a.version)
        });
        
        return Ok(versions);
    }

    Ok(vec![])
}

/// 比较OptiFine版本�?
fn compare_optifine_version(a: &str, b: &str) -> std::cmp::Ordering {
    // 提取版本号中的字母部分和数字部分
    // 例如 "1.12.2 G5" -> ["1", "12", "2", "G", "5"]
    let a_parts = extract_version_parts(a);
    let b_parts = extract_version_parts(b);
    
    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        // 尝试解析为数�?
        let a_num = a_part.parse::<u32>().ok();
        let b_num = b_part.parse::<u32>().ok();
        
        match (a_num, b_num) {
            (Some(a_n), Some(b_n)) => {
                let cmp = a_n.cmp(&b_n);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            _ => {
                // 字符串比�?
                let cmp = a_part.cmp(b_part);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
        }
    }
    
    a_parts.len().cmp(&b_parts.len())
}

/// 提取版本号中的各个部�?
fn extract_version_parts(version: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut is_digit = false;
    
    for c in version.chars() {
        if c.is_ascii_digit() {
            if !is_digit && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            current.push(c);
            is_digit = true;
        } else if c.is_ascii_alphabetic() {
            if is_digit && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
            current.push(c.to_ascii_uppercase());
            is_digit = false;
        } else if c == '.' || c == ' ' || c == '_' || c == '-' {
            if !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            }
        }
    }
    
    if !current.is_empty() {
        parts.push(current);
    }
    
    parts
}

/// List LiteLoader versions
pub async fn list_liteloader_versions(mc_version: &str, mirror_url: Option<&str>) -> anyhow::Result<Vec<LoaderVersion>> {
    let url = match mirror_url {
        Some(mirror) if !mirror.is_empty() => format!("{}/maven/com/mumfrey/liteloader/versions.json", mirror.trim_end_matches('/')),
        _ => "https://dl.liteloader.com/versions/versions.json".to_string(),
    };

    let content = download::fetch_url(&url).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let mut versions = Vec::new();
    
    // LiteLoader JSON 格式：versions -> {mc_version} -> artefacts -> com.mumfrey:liteloader -> latest
    if let Some(mc_versions) = json["versions"].as_object() {
        // 只返回指�?MC 版本的数�?
        if let Some(mc_version_data) = mc_versions.get(mc_version) {
            // 尝试�?artefacts �?snapshots 获取
            let artefacts = mc_version_data.get("artefacts")
                .or_else(|| mc_version_data.get("snapshots"));
            
            if let Some(artefacts) = artefacts {
                if let Some(liteloader) = artefacts.get("com.mumfrey:liteloader") {
                    if let Some(latest) = liteloader.get("latest") {
                        let stream = latest["stream"].as_str().unwrap_or("release");
                        
                        // 直接使用 MC 版本号，去除 -SNAPSHOT 后缀
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

/// Install loader
pub async fn install_loader(
    loader_type: LoaderType,
    mc_version: &str,
    loader_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    _max_threads: usize,
) -> anyhow::Result<()> {
    match loader_type {
        LoaderType::Forge => install_forge(mc_version, loader_version, game_dir, mirror_url).await,
        LoaderType::NeoForge => install_neoforge(mc_version, loader_version, game_dir, mirror_url).await,
        LoaderType::Fabric => install_fabric(mc_version, loader_version, game_dir, mirror_url).await,
        LoaderType::OptiFine => install_optifine(mc_version, loader_version).await,
        LoaderType::LiteLoader => install_liteloader(mc_version, loader_version, game_dir, mirror_url).await,
    }
}

async fn install_forge(mc_version: &str, forge_version: &str, game_dir: &Path, _mirror_url: Option<&str>) -> anyhow::Result<()> {
    log::info!("[Forge] Installing {} for MC {}", forge_version, mc_version);

    let file_name = format!("forge-{}-{}-installer.jar", mc_version, forge_version);
    let installer_url = format!("https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/{}", mc_version, forge_version, file_name);
    let installer_path = game_dir.join("forge-installer.jar");

    // Download installer
    let urls = vec![
        installer_url.replace("https://maven.minecraftforge.net", "https://bmclapi2.bangbang93.com/maven"),
        installer_url.clone(),
    ];

    let manager = DownloadManager::new(1, 0, DownloadSourceMode::Smart);
    let task = DownloadTask {
        id: "forge_installer".to_string(),
        urls,
        local_path: installer_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download Forge installer"));
        }
    }

    let version_id = format!("{}-forge-{}", mc_version, forge_version);

    // Check if we need the injector (Forge >= 20)
    if forge_installer::needs_injector(forge_version, false) {
        log::info!("[Forge] Using injector for Forge {}", forge_version);

        // Extract embedded resources
        let cache_dir = game_dir.join(".cache");
        let (injector_path, wrapper_path) = forge_installer::extract_embedded_resources(&cache_dir)?;

        // Find Java
        let java_path = find_java_for_install(game_dir)?;

        // Run injector
        forge_installer::run_forge_installer(
            &java_path,
            &installer_path.to_string_lossy(),
            &injector_path,
            &wrapper_path,
            &game_dir.to_string_lossy(),
            false,
            None,
        )?;

        // Find and copy the generated version JSON
        let versions_dir = game_dir.join("versions");
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                    if dir_name.contains("forge") && dir_name.contains(mc_version) {
                        let json_files: Vec<_> = std::fs::read_dir(&path)?
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                            .collect();

                        if let Some(json_file) = json_files.first() {
                            let version_dir = game_dir.join("versions").join(&version_id);
                            std::fs::create_dir_all(&version_dir)?;
                            let target_json = version_dir.join(format!("{}.json", version_id));
                            std::fs::copy(json_file.path(), &target_json)?;
                            log::info!("[Forge] Copied version JSON from {}", path.display());
                            break;
                        }
                    }
                }
            }
        }
    } else {
        // Old Forge: direct extraction
        log::info!("[Forge] Using direct extraction for old Forge {}", forge_version);

        let file = std::fs::File::open(&installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Try to get version.json
        let version_json = {
            let mut entry = archive.by_name("version.json")?;
            let mut content = String::new();
            std::io::Read::read_to_string(&mut entry, &mut content)?;
            serde_json::from_str::<serde_json::Value>(&content)?
        };

        let version_dir = game_dir.join("versions").join(&version_id);
        std::fs::create_dir_all(&version_dir)?;

        let mut merged_json = version_json.clone();
        merged_json["id"] = serde_json::Value::String(version_id.clone());
        if merged_json.get("inheritsFrom").is_none() {
            merged_json["inheritsFrom"] = serde_json::Value::String(mc_version.to_string());
        }

        let json_path = version_dir.join(format!("{}.json", version_id));
        std::fs::write(&json_path, serde_json::to_string_pretty(&merged_json)?)?;

        // Extract maven directory
        let file = std::fs::File::open(&installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let entry_name = entry.name().to_string();

            if entry_name.starts_with("maven/") {
                let relative_path = entry_name.strip_prefix("maven/").unwrap_or(&entry_name);
                let target_path = game_dir.join("libraries").join(relative_path);
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut target_file = std::fs::File::create(&target_path)?;
                std::io::copy(&mut entry, &mut target_file)?;
            }
        }
    }

    let _ = std::fs::remove_file(&installer_path);
    log::info!("[Forge] Installed: {}", version_id);
    Ok(())
}

async fn install_neoforge(mc_version: &str, neoforge_version: &str, game_dir: &Path, _mirror_url: Option<&str>) -> anyhow::Result<()> {
    log::info!("[NeoForge] Installing {} for MC {}", neoforge_version, mc_version);

    let installer_url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        neoforge_version, neoforge_version
    );
    let installer_path = game_dir.join("neoforge-installer.jar");

    let urls = vec![
        installer_url.replace("https://maven.neoforged.net/releases", "https://bmclapi2.bangbang93.com/maven"),
        installer_url.clone(),
    ];

    let manager = DownloadManager::new(1, 0, DownloadSourceMode::Smart);
    let task = DownloadTask {
        id: "neoforge_installer".to_string(),
        urls,
        local_path: installer_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    let results = manager.download_batch(vec![task], None).await;
    if let Some(result) = results.first() {
        if result.status == DownloadStatus::Failed {
            return Err(anyhow::anyhow!("Failed to download NeoForge installer"));
        }
    }

    let version_id = format!("{}-neoforge-{}", mc_version, neoforge_version);

    // NeoForge always uses injector
    log::info!("[NeoForge] Using injector");

    let cache_dir = game_dir.join(".cache");
    let (injector_path, wrapper_path) = forge_installer::extract_embedded_resources(&cache_dir)?;

    let java_path = find_java_for_install(game_dir)?;

    forge_installer::run_forge_installer(
        &java_path,
        &installer_path.to_string_lossy(),
        &injector_path,
        &wrapper_path,
        &game_dir.to_string_lossy(),
        true,
        None,
    )?;

    // Find and copy the generated version JSON
    let versions_dir = game_dir.join("versions");
    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name.contains("neoforge") {
                    let json_files: Vec<_> = std::fs::read_dir(&path)?
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().extension().map(|ext| ext == "json").unwrap_or(false))
                        .collect();

                    if let Some(json_file) = json_files.first() {
                        let version_dir = game_dir.join("versions").join(&version_id);
                        std::fs::create_dir_all(&version_dir)?;
                        let target_json = version_dir.join(format!("{}.json", version_id));
                        std::fs::copy(json_file.path(), &target_json)?;
                        log::info!("[NeoForge] Copied version JSON from {}", path.display());
                        break;
                    }
                }
            }
        }
    }

    let _ = std::fs::remove_file(&installer_path);
    log::info!("[NeoForge] Installed: {}", version_id);
    Ok(())
}

/// Find Java for installation (minimum Java 8u60)
fn find_java_for_install(_game_dir: &Path) -> anyhow::Result<String> {
    // Try to find Java from PATH
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let java_path = dir.join("java.exe");
            if java_path.exists() {
                return Ok(java_path.to_string_lossy().to_string());
            }
        }
    }

    // Try JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_path = Path::new(&java_home).join("bin").join("java.exe");
        if java_path.exists() {
            return Ok(java_path.to_string_lossy().to_string());
        }
    }

    Err(anyhow::anyhow!("Java not found. Please install Java 8+ to install Forge/NeoForge."))
}

async fn install_fabric(mc_version: &str, fabric_version: &str, game_dir: &Path, mirror_url: Option<&str>) -> anyhow::Result<()> {
    log::info!("[Fabric] Installing {} for MC {}", fabric_version, mc_version);

    let version_id = format!("fabric-{}-{}", fabric_version, mc_version);
    let version_dir = game_dir.join("versions").join(&version_id);
    std::fs::create_dir_all(&version_dir)?;

    let url = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
        mc_version, fabric_version
    );

    let urls = match mirror_url {
        Some(mirror) if !mirror.is_empty() => vec![
            format!("{}/fabric-meta/v2/versions/loader/{}/{}/profile/json", mirror.trim_end_matches('/'), mc_version, fabric_version),
            url,
        ],
        _ => vec![
            format!("https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader/{}/{}/profile/json", mc_version, fabric_version),
            url,
        ],
    };

    let manager = DownloadManager::new(1, 0, DownloadSourceMode::Smart);
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

    log::info!("[Fabric] Installed: {}", version_id);
    Ok(())
}

async fn install_optifine(mc_version: &str, optifine_version: &str) -> anyhow::Result<()> {
    log::info!("[OptiFine] {} for MC {} - manual installation required", optifine_version, mc_version);
    Ok(())
}

async fn install_liteloader(mc_version: &str, liteloader_version: &str, game_dir: &Path, mirror_url: Option<&str>) -> anyhow::Result<()> {
    log::info!("[LiteLoader] Installing {} for MC {}", liteloader_version, mc_version);

    let version_id = format!("{}-LiteLoader", mc_version);
    let version_dir = game_dir.join("versions").join(&version_id);
    std::fs::create_dir_all(&version_dir)?;

    let url = format!(
        "https://dl.liteloader.com/versions/com/mumfrey/liteloader/{}/liteloader-{}-{}.json",
        mc_version, mc_version, liteloader_version
    );

    let urls = match mirror_url {
        Some(mirror) if !mirror.is_empty() => vec![
            format!("{}/maven/com/mumfrey/liteloader/{}/liteloader-{}-{}.json", mirror.trim_end_matches('/'), mc_version, mc_version, liteloader_version),
            url,
        ],
        _ => vec![
            format!("https://bmclapi2.bangbang93.com/maven/com/mumfrey/liteloader/{}/liteloader-{}-{}.json", mc_version, mc_version, liteloader_version),
            url,
        ],
    };

    let manager = DownloadManager::new(1, 0, DownloadSourceMode::Smart);
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

    log::info!("[LiteLoader] Installed: {}", version_id);
    Ok(())
}
