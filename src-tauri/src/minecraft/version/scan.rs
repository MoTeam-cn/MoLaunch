//! 版本管理模块

use serde::{Deserialize, Serialize};
use std::path::Path;

/// 版本状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersionState {
    /// 错误状态
    Error,
    /// 原版
    Original,
    /// 快照版
    Snapshot,
    /// 愚人节版
    Fool,
    /// OptiFine
    OptiFine,
    /// 旧版
    Old,
    /// Forge
    Forge,
    /// NeoForge
    NeoForge,
    /// LiteLoader
    LiteLoader,
    /// Fabric
    Fabric,
}

/// 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// 版本ID
    pub id: String,
    /// 版本类型（release/snapshot）
    pub version_type: String,
    /// 发布时间
    pub release_time: String,
    /// 版本JSON文件路径
    pub json_path: String,
    /// 版本文件夹路径
    pub path: String,
    /// 版本状态
    pub state: VersionState,
    /// 原版版本号（如1.20.1）
    pub original_version: Option<String>,
    /// Forge版本
    pub forge_version: Option<String>,
    /// NeoForge版本
    pub neoforge_version: Option<String>,
    /// Fabric版本
    pub fabric_version: Option<String>,
    /// OptiFine版本
    pub optifine_version: Option<String>,
    /// LiteLoader版本
    pub liteloader_version: Option<String>,
}

/// 扫描已安装版本
pub fn scan_installed_versions(game_dir: &Path) -> Vec<VersionInfo> {
    let versions_dir = game_dir.join("versions");
    let mut versions = Vec::new();
    
    if !versions_dir.exists() || !versions_dir.is_dir() {
        return versions;
    }
    
    // 遍历versions目录
    if let Ok(entries) = std::fs::read_dir(&versions_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                let json_path = path.join(format!("{}.json", dir_name));
                
                if json_path.exists() {
                    match parse_version_info(&path, &json_path) {
                        Ok(info) => versions.push(info),
                        Err(e) => {
                            log::warn!("Failed to parse version {}: {}", dir_name, e);
                            // 创建错误状态的版本信息
                            versions.push(VersionInfo {
                                id: dir_name.to_string(),
                                version_type: "unknown".to_string(),
                                release_time: String::new(),
                                json_path: json_path.to_string_lossy().to_string(),
                                path: path.to_string_lossy().to_string(),
                                state: VersionState::Error,
                                original_version: None,
                                forge_version: None,
                                neoforge_version: None,
                                fabric_version: None,
                                optifine_version: None,
                                liteloader_version: None,
                            });
                        }
                    }
                }
            }
        }
    }
    
    versions
}

/// 解析版本信息
fn parse_version_info(version_dir: &Path, json_path: &Path) -> Result<VersionInfo, Box<dyn std::error::Error>> {
    let json_content = std::fs::read_to_string(json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json_content)?;
    
    let id = json["id"].as_str().unwrap_or_default().to_string();
    let version_type = json["type"].as_str().unwrap_or("release").to_string();
    let release_time = json["releaseTime"].as_str().unwrap_or_default().to_string();
    
    // 检测加载器类型
    let (state, original_version, forge_version, neoforge_version, fabric_version, optifine_version, liteloader_version) = 
        detect_loaders(&json, &json_content);
    
    Ok(VersionInfo {
        id,
        version_type,
        release_time,
        json_path: json_path.to_string_lossy().to_string(),
        path: version_dir.to_string_lossy().to_string(),
        state,
        original_version,
        forge_version,
        neoforge_version,
        fabric_version,
        optifine_version,
        liteloader_version,
    })
}

/// 检测加载器类型
fn detect_loaders(
    json: &serde_json::Value,
    json_content: &str,
) -> (VersionState, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) {
    let mut state = VersionState::Original;
    let mut forge_version = None;
    let mut neoforge_version = None;
    let mut fabric_version = None;
    let mut optifine_version = None;
    let mut liteloader_version = None;
    
    // 检测OptiFine
    if json_content.contains("optifine") {
        state = VersionState::OptiFine;
        // 尝试从libraries中提取OptiFine版本
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("optifine") {
                        optifine_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }
    
    // 检测Fabric
    if json_content.contains("net.fabricmc:fabric-loader") || json_content.contains("org.quiltmc:quilt-loader") {
        state = VersionState::Fabric;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("fabric-loader") {
                        fabric_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }
    
    // 检测NeoForge
    if json_content.contains("net.neoforge") {
        state = VersionState::NeoForge;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("neoforge") {
                        neoforge_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }
    
    // 检测Forge（排除NeoForge）
    if json_content.contains("minecraftforge") && !json_content.contains("net.neoforge") {
        state = VersionState::Forge;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("minecraftforge") {
                        forge_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }
    
    // 检测LiteLoader
    if json_content.contains("liteloader") {
        state = VersionState::LiteLoader;
        if let Some(libraries) = json["libraries"].as_array() {
            for lib in libraries {
                if let Some(name) = lib["name"].as_str() {
                    if name.contains("liteloader") {
                        liteloader_version = Some(name.to_string());
                        break;
                    }
                }
            }
        }
    }
    
    // 获取原版版本号
    let original_version = extract_original_version(json, json_content);
    
    // 判断是否为快照版
    if let Some(id) = json["id"].as_str() {
        if id.contains("snapshot") || id.contains("pre") || id.contains("rc") {
            state = VersionState::Snapshot;
        }
    }
    
    (state, original_version, forge_version, neoforge_version, fabric_version, optifine_version, liteloader_version)
}

/// 提取原版版本号
fn extract_original_version(json: &serde_json::Value, _json_content: &str) -> Option<String> {
    // 策略1: 从inheritsFrom获取
    if let Some(inherits) = json["inheritsFrom"].as_str() {
        return Some(inherits.to_string());
    }
    
    // 策略2: 从arguments中的--fml.mcVersion获取
    if let Some(arguments) = json["arguments"].as_object() {
        if let Some(game) = arguments.get("game").and_then(|g| g.as_array()) {
            for (i, arg) in game.iter().enumerate() {
                if let Some(arg_str) = arg.as_str() {
                    if arg_str == "--fml.mcVersion" {
                        if let Some(next_arg) = game.get(i + 1) {
                            if let Some(version) = next_arg.as_str() {
                                return Some(version.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    
    // 策略3: 从downloads URL中提取
    if let Some(downloads) = json["downloads"].as_object() {
        if let Some(client) = downloads.get("client") {
            if let Some(url) = client["url"].as_str() {
                // 尝试从URL中提取版本号
                let re = regex::Regex::new(r"(\d+\.\d+(\.\d+)?(-\w+)?)").unwrap();
                if let Some(captures) = re.captures(url) {
                    return Some(captures[1].to_string());
                }
            }
        }
    }
    
    // 策略4: 从jar字段获取
    if let Some(jar) = json["jar"].as_str() {
        return Some(jar.to_string());
    }
    
    // 策略5: 从id字段正则匹配
    if let Some(id) = json["id"].as_str() {
        let re = regex::Regex::new(r"^(\d+\.\d+(\.\d+)?)").unwrap();
        if let Some(captures) = re.captures(id) {
            return Some(captures[1].to_string());
        }
    }
    
    None
}

/// 获取版本的继承链
pub fn get_version_chain(game_dir: &Path, version_id: &str) -> Vec<String> {
    let mut chain = vec![version_id.to_string()];
    let mut current = version_id.to_string();
    
    loop {
        let json_path = game_dir.join("versions").join(&current).join(format!("{}.json", current));
        if !json_path.exists() {
            break;
        }
        
        match std::fs::read_to_string(&json_path) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        if let Some(inherits) = json["inheritsFrom"].as_str() {
                            chain.push(inherits.to_string());
                            current = inherits.to_string();
                        } else {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            Err(_) => break,
        }
    }
    
    chain
}

/// 卸载版本
pub fn uninstall_version(game_dir: &Path, version_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let version_dir = game_dir.join("versions").join(version_id);
    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir)?;
        log::info!("Uninstalled version: {}", version_id);
    }
    Ok(())
}