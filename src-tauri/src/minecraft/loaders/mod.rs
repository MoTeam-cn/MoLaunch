//! Loader management module

pub mod forge_html;
pub mod forge_installer;
pub mod utils;

use crate::{log_info, log_warn, log_error};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use super::download::manager::{DownloadManager, DownloadTask, DownloadStatus};
use super::sources::{self, DownloadSourceMode};

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
pub async fn list_forge_versions(mc_version: &str, mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
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
    forge_html::parse_forge_version_html(&content)
}

/// List NeoForge versions
pub async fn list_neoforge_versions(mc_version: &str, mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
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
    let legacy_urls = sources::build_urls(
        mirror_url,
        sources::NEOFORGE_API_LEGACY,
        sources::BMCLAPI_NEOFORGE_LEGACY,
        source_mode,
    );

    crate::log_debug!("[NeoForge] 尝试旧版源: {:?}", legacy_urls);
    if let Ok(legacy_content) = sources::fetch_with_fallback(&legacy_urls).await {
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
        let v_a = utils::parse_version_number(&a.version);
        let v_b = utils::parse_version_number(&b.version);
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
pub async fn list_fabric_versions(mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(
        mirror_url,
        sources::FABRIC_META,
        sources::BMCLAPI_FABRIC_META,
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let mut versions = Vec::new();
    if let Some(versions_array) = json.as_array() {
        for version in versions_array {
            if let Some(version_str) = version["version"].as_str() {
                versions.push(LoaderVersion {
                    version: version_str.to_string(),  // 保留完整版本号如 0.16.14+build.1
                    is_recommended: version["stable"].as_bool().unwrap_or(false),
                    release_time: None,
                });
            }
        }
    }

    Ok(versions)
}

/// List OptiFine versions
pub async fn list_optifine_versions(mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
    // OptiFine 只有 BMCLAPI 源，没有官方 API
    let urls = sources::build_urls(
        mirror_url,
        &format!("{}{}", sources::BMCLAPI_BASE, sources::BMCLAPI_OPTIFINE),
        sources::BMCLAPI_OPTIFINE,
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;

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
pub async fn list_liteloader_versions(mc_version: &str, mirror_url: Option<&str>, source_mode: DownloadSourceMode) -> anyhow::Result<Vec<LoaderVersion>> {
    let urls = sources::build_urls(
        mirror_url,
        sources::LITELOADER_VERSIONS,
        sources::BMCLAPI_LITELOADER,
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;
    parse_liteloader_versions(&content, mc_version)
}

fn parse_liteloader_versions(content: &str, mc_version: &str) -> anyhow::Result<Vec<LoaderVersion>> {
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

/// Install loader
pub async fn install_loader(
    loader_type: LoaderType,
    mc_version: &str,
    loader_version: &str,
    game_dir: &Path,
    mirror_url: Option<&str>,
    _max_threads: usize,
    progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<()> {
    match loader_type {
        LoaderType::Forge => install_forge(mc_version, loader_version, game_dir, mirror_url, progress_callback, source_mode).await,
        LoaderType::NeoForge => install_neoforge(mc_version, loader_version, game_dir, mirror_url, progress_callback, source_mode).await,
        LoaderType::Fabric => install_fabric(mc_version, loader_version, game_dir, mirror_url, progress_callback, source_mode).await,
        LoaderType::OptiFine => install_optifine(mc_version, loader_version, progress_callback, source_mode).await,
        LoaderType::LiteLoader => install_liteloader(mc_version, loader_version, game_dir, mirror_url, progress_callback, source_mode).await,
    }
}

async fn install_forge(mc_version: &str, forge_version: &str, game_dir: &Path, mirror_url: Option<&str>, progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>, source_mode: DownloadSourceMode) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    let file_name = format!("forge-{}-{}-installer.jar", mc_version, forge_version);
    let installer_url = sources::forge_installer_url(mc_version, forge_version);
    let temp_dir = std::env::temp_dir().join("MoLaunch").join("TaskTemp");
    std::fs::create_dir_all(&temp_dir)?;
    let installer_path = temp_dir.join(&file_name);

    // 尝试获取文件 hash（从 Maven 的 .sha1 文件）
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
        expected_size: 0,  // Maven 不提供 size，下载后用实际大小
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
    let result = if forge_installer::needs_injector(forge_version, false) {
        // 新版 Forge (1.13+): 使用注入器
        do_install_forge(mc_version, forge_version, &installer_path, game_dir, progress_callback, source_mode).await
    } else {
        // 旧版 Forge (1.12.2 及以下): 解压复制
        install_forge_legacy(mc_version, forge_version, &installer_path, game_dir, progress_callback).await
    };
    // 不删除安装器，临时目录由系统或下次启动时清理
    result
}

/// 旧版 Forge 安装（1.12.2 及以下）：解压 installer.jar 并复制文件
/// 参考 PCL2 的 Legacy 方式 1 和方式 2
#[allow(clippy::too_many_arguments)]
async fn install_forge_legacy(
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

    // 打开 installer.jar 作为 ZIP
    let installer_file = std::fs::File::open(installer_path)?;
    let mut zip = zip::ZipArchive::new(installer_file)?;

    // 读取 install_profile.json
    let profile_json: serde_json::Value = {
        let mut entry = zip.by_name("install_profile.json")?;
        let mut content = String::new();
        entry.read_to_string(&mut content)?;
        serde_json::from_str(&content)?
    };

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    // 判断安装方式：是否有 "install" 节点
    if profile_json.get("install").is_some() {
        // Legacy 方式 2（1.7.10 及更早）：有 install 节点
        log_info!("[Forge] Legacy 方式 2: {}", forge_version);

        let install = &profile_json["install"];

        // 提取 Forge 主 JAR
        let file_path = install["filePath"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("install.filePath not found"))?;
        let lib_path = install["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("install.path not found"))?;

        // 计算目标路径：将 Maven 坐标转换为文件路径
        let jar_dest = maven_path_to_local(lib_path, game_dir);
        if let Some(parent) = jar_dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // 从 installer.jar 中提取 JAR
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

        // 提取版本 JSON
        let version_info = profile_json.get("versionInfo")
            .ok_or_else(|| anyhow::anyhow!("versionInfo not found"))?;
        let mut version_json = version_info.clone();

        // 设置 id 和 inheritsFrom
        version_json["id"] = serde_json::Value::String(version_id.clone());
        if version_json.get("inheritsFrom").is_none() {
            version_json["inheritsFrom"] = serde_json::Value::String(mc_version.to_string());
        }

        let json_path = version_dir.join(format!("{}.json", version_id));
        std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;
        log_info!("[Forge] 写入版本 JSON: {}", json_path.display());

    } else {
        // Legacy 方式 1（1.8 ~ 1.12.2）：无 install 节点
        log_info!("[Forge] Legacy 方式 1: {}", forge_version);

        // 从 install_profile.json 的 json 字段获取版本 JSON 的路径
        let json_entry_name = profile_json["json"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("install_profile.json 中缺少 json 字段"))?
            .trim_start_matches('/');

        // 从 installer.jar 中读取版本 JSON
        let mut version_json: serde_json::Value = {
            let mut entry = zip.by_name(json_entry_name)?;
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            serde_json::from_str(&content)?
        };

        // 修改 id
        version_json["id"] = serde_json::Value::String(version_id.clone());

        let json_path = version_dir.join(format!("{}.json", version_id));
        std::fs::write(&json_path, serde_json::to_string_pretty(&version_json)?)?;
        log_info!("[Forge] 写入版本 JSON: {}", json_path.display());

        if let Some(ref cb) = progress_callback {
            cb(50.0);
        }

        // 解压 maven/ 文件夹到 libraries/
        let maven_dest = game_dir.join("libraries");
        let mut extracted_count = 0;

        // 先收集需要解压的文件列表
        let maven_entries: Vec<String> = zip
            .file_names()
            .filter(|name| name.starts_with("maven/"))
            .map(|s| s.to_string())
            .collect();

        for entry_name in &maven_entries {
            let relative_path = entry_name.strip_prefix("maven/").unwrap_or(entry_name);
            if relative_path.is_empty() {
                continue;
            }
            let dest_path = maven_dest.join(relative_path);

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
    }

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[Forge] Legacy 安装完成: {}", version_id);
    Ok(())
}

/// 将 Maven 坐标路径转换为本地文件路径
/// 例如: "net.minecraftforge:forge:1.7.10-10.13.4.1614:universal"
///   -> "{game_dir}/libraries/net/minecraftforge/forge/1.7.10-10.13.4.1614/forge-1.7.10-10.13.4.1614-universal.jar"
fn maven_path_to_local(maven_path: &str, game_dir: &Path) -> std::path::PathBuf {
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
        // fallback: 直接拼接
        libs_dir.join(maven_path.replace(':', "/"))
    }
}

async fn do_install_forge(mc_version: &str, forge_version: &str, installer_path: &Path, game_dir: &Path, progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>, source_mode: DownloadSourceMode) -> anyhow::Result<()> {
    log_info!("[Forge] Installing {} for MC {}", forge_version, mc_version);

    let version_id = format!("{}-forge-{}", mc_version, forge_version);

    // 确保 launcher_profiles.json 存在（Forge 安装器需要）
    super::launcher_profiles::ensure_profiles_exist(game_dir)
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(ref cb) = progress_callback {
        cb(20.0);
    }

    // 下载 Mojang 映射文件（Forge >= 20 需要）
    if forge_installer::needs_injector(forge_version, false) {
        log_info!("[Forge] Downloading Mojang mappings for MC {}", mc_version);
        if let Err(e) = download_mojang_mappings(mc_version, game_dir, installer_path, source_mode).await {
            log_warn!("[Forge] Failed to download mappings: {}", e);
            // 不阻断安装，让安装器自己处理
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    // Check if we need the injector (Forge >= 20)
    if forge_installer::needs_injector(forge_version, false) {
        log_info!("[Forge] Using injector for Forge {}", forge_version);

        // Extract embedded resources
        let (injector_path, wrapper_path) = forge_installer::extract_embedded_resources()?;

        if let Some(ref cb) = progress_callback {
            cb(40.0);
        }

        // Find Java
        let java_path = find_java_for_install(game_dir)?;

        if let Some(ref cb) = progress_callback {
            cb(50.0);
        }

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

        if let Some(ref cb) = progress_callback {
            cb(80.0);
        }

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
                            // 重试复制，避免文件锁定问题
                            let mut copied = false;
                            for retry in 0..3 {
                                match std::fs::copy(json_file.path(), &target_json) {
                                    Ok(_) => {
                                        log_info!("[Forge] Copied version JSON from {}", path.display());
                                        copied = true;
                                        break;
                                    }
                                    Err(e) => {
                                        if retry < 2 {
                                            log_warn!("[Forge] Copy failed (retry {}): {}", retry + 1, e);
                                            std::thread::sleep(std::time::Duration::from_millis(500));
                                        } else {
                                            log_error!("[Forge] Copy failed after retries: {}", e);
                                        }
                                    }
                                }
                            }
                            if copied {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // 参考 PCL2：复制原版 JAR 到 Forge 版本文件夹
        let mc_jar = game_dir.join("versions").join(mc_version).join(format!("{}.jar", mc_version));
        let forge_jar = game_dir.join("versions").join(&version_id).join(format!("{}.jar", version_id));
        if mc_jar.exists() && !forge_jar.exists() {
            if let Err(e) = std::fs::copy(&mc_jar, &forge_jar) {
                log_warn!("[Forge] Failed to copy MC JAR: {}", e);
            } else {
                log_info!("[Forge] Copied MC JAR to {}", forge_jar.display());
            }
        }

        if let Some(ref cb) = progress_callback {
            cb(90.0);
        }
    } else {
        // Old Forge: direct extraction
        log_info!("[Forge] Using direct extraction for old Forge {}", forge_version);

        let file = std::fs::File::open(installer_path)?;
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
        let file = std::fs::File::open(installer_path)?;
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

        if let Some(ref cb) = progress_callback {
            cb(90.0);
        }
    }

    log_info!("[Forge] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}

async fn install_neoforge(mc_version: &str, neoforge_version: &str, game_dir: &Path, mirror_url: Option<&str>, progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>, source_mode: DownloadSourceMode) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    log_info!("[NeoForge] Installing {} for MC {}", neoforge_version, mc_version);

    let file_name = format!("neoforge-{}-installer.jar", neoforge_version);
    let installer_url = sources::neoforge_installer_url(neoforge_version);
    let temp_dir = std::env::temp_dir().join("MoLaunch").join("TaskTemp");
    std::fs::create_dir_all(&temp_dir)?;
    let installer_path = temp_dir.join(&file_name);

    // 尝试获取文件 hash
    let hash_url = format!("{}.sha1", installer_url);
    let expected_hash = match crate::http::fetch_url(&hash_url).await {
        Ok(hash) => Some(hash.trim().to_string()),
        Err(_) => None,
    };

    let urls = sources::build_replace_urls(&installer_url, mirror_url, sources::MAVEN_REPLACEMENTS, source_mode);

    let manager = DownloadManager::new(1, 0, 0, source_mode);
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

    // 确保 launcher_profiles.json 存在（NeoForge 安装器需要）
    super::launcher_profiles::ensure_profiles_exist(game_dir)
        .map_err(|e| anyhow::anyhow!(e))?;

    if let Some(ref cb) = progress_callback {
        cb(20.0);
    }

    // 下载 Mojang 映射文件（NeoForge 需要）
    log_info!("[NeoForge] Downloading Mojang mappings for MC {}", mc_version);
    if let Err(e) = download_mojang_mappings(mc_version, game_dir, &installer_path, source_mode).await {
        log_warn!("[NeoForge] Failed to download mappings: {}", e);
        // 不阻断安装，让安装器自己处理
    }

    if let Some(ref cb) = progress_callback {
        cb(30.0);
    }

    // NeoForge always uses injector
    log_info!("[NeoForge] Using injector");

    let (injector_path, wrapper_path) = forge_installer::extract_embedded_resources()?;

    if let Some(ref cb) = progress_callback {
        cb(40.0);
    }

    let java_path = find_java_for_install(game_dir)?;

    if let Some(ref cb) = progress_callback {
        cb(50.0);
    }

    forge_installer::run_forge_installer(
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
                        log_info!("[NeoForge] Copied version JSON from {}", path.display());
                        break;
                    }
                }
            }
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(90.0);
    }

    log_info!("[NeoForge] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}

/// Find Java for installation (minimum Java 8u60)
/// 优先使用自动检测的 Java，而不是系统环境变量
fn find_java_for_install(_game_dir: &Path) -> anyhow::Result<String> {
    // 使用 Java 检测模块搜索 Java
    let java_list = super::java::search_java();

    if !java_list.is_empty() {
        // 使用新的 Java 选择算法
        if let Some(java_path) = super::java_selector::get_java_for_installer(&java_list) {
            log_info!("[Java] 使用自动检测的 Java: {}", java_path);
            return Ok(java_path);
        }
    }

    // 兜底：尝试从 PATH 查找
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

async fn install_fabric(mc_version: &str, fabric_version: &str, game_dir: &Path, mirror_url: Option<&str>, progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>, source_mode: DownloadSourceMode) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    log_info!("[Fabric] Installing {} for MC {}", fabric_version, mc_version);

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

    log_info!("[Fabric] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}

async fn install_optifine(mc_version: &str, optifine_version: &str, progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>, _source_mode: DownloadSourceMode) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    log_info!("[OptiFine] {} for MC {} - manual installation required", optifine_version, mc_version);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}

async fn install_liteloader(mc_version: &str, liteloader_version: &str, game_dir: &Path, mirror_url: Option<&str>, progress_callback: Option<Arc<dyn Fn(f64) + Send + Sync>>, source_mode: DownloadSourceMode) -> anyhow::Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0);
    }

    log_info!("[LiteLoader] Installing {} for MC {}", liteloader_version, mc_version);

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

    log_info!("[LiteLoader] Installed: {}", version_id);

    if let Some(ref cb) = progress_callback {
        cb(100.0);
    }

    Ok(())
}

/// 下载 Mojang 映射文件（Forge/NeoForge >= 20 需要）
/// 参考 PCL2 的实现：从 install_profile.json 的 data.MOJMAPS.client 字段获取路径
async fn download_mojang_mappings(mc_version: &str, game_dir: &Path, installer_path: &Path, source_mode: DownloadSourceMode) -> anyhow::Result<()> {
    // 从 Forge 安装器中提取 install_profile.json
    let file = std::fs::File::open(installer_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    let mut install_profile_content = String::new();
    {
        let mut entry = archive.by_name("install_profile.json")?;
        std::io::Read::read_to_string(&mut entry, &mut install_profile_content)?;
    }
    
    let install_profile: serde_json::Value = serde_json::from_str(&install_profile_content)?;
    
    // 检查是否有 MOJMAPS 数据
    let mojmaps = match install_profile["data"]["MOJMAPS"]["client"].as_str() {
        Some(s) => s,
        None => {
            log_info!("[Mappings] No MOJMAPS data found in install_profile.json");
            return Ok(());
        }
    };
    
    // 解析格式：[net.minecraft:client:1.17.1-20210706.113038:mappings@txt]
    // 去掉 [] 和 @ 后面的部分
    let mojmaps_clean = mojmaps
        .trim_start_matches('[')
        .trim_end_matches(']');
    
    let original_name = mojmaps_clean
        .split('@')
        .next()
        .unwrap_or("");
    
    // 提取扩展名（@txt -> txt, @tsrg -> tsrg）
    let extension = mojmaps_clean
        .split('@')
        .nth(1)
        .unwrap_or("txt")
        .trim_end_matches(']');
    
    // 解析 Maven 坐标：net.minecraft:client:1.17.1-20210706.113038
    let parts: Vec<&str> = original_name.split(':').collect();
    if parts.len() < 3 {
        return Err(anyhow::anyhow!("Invalid MOJMAPS format: {}", mojmaps));
    }
    
    let group = parts[0];  // net.minecraft
    let artifact = parts[1];  // client
    let version = parts[2];  // 1.17.1-20210706.113038
    
    // 构建本地路径：libraries/net/minecraft/client/1.17.1-20210706.113038/client-1.17.1-20210706.113038-mappings.txt
    let group_path = group.replace('.', std::path::MAIN_SEPARATOR_STR);
    let local_dir = game_dir.join("libraries").join(group_path).join(artifact).join(version);
    let filename = format!("{}-{}-mappings.{}", artifact, version, extension);
    let local_path = local_dir.join(&filename);
    
    // 检查是否已存在
    if local_path.exists() {
        log_info!("[Mappings] File already exists: {}", local_path.display());
        return Ok(());
    }
    
    // 从版本 JSON 获取下载信息
    let version_list = super::download::fetch_version_list(None, source_mode).await?;
    let json_url = super::download::get_version_json_url(&version_list.value, mc_version)
        .ok_or_else(|| anyhow::anyhow!("Version {} not found", mc_version))?;
    
    let json_content = super::download::fetch_url(&json_url).await?;
    let version_json: serde_json::Value = serde_json::from_str(&json_content)?;
    
    let mappings = version_json["downloads"]["client_mappings"].as_object()
        .ok_or_else(|| anyhow::anyhow!("client_mappings not found"))?;
    
    let url = mappings["url"].as_str()
        .ok_or_else(|| anyhow::anyhow!("client_mappings URL not found"))?;
    let sha1 = mappings["sha1"].as_str().unwrap_or_default();
    let size = mappings["size"].as_i64().unwrap_or(0);
    
    // 下载
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
