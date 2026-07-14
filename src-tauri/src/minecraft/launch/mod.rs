//! Game launch module
//!
//! This module provides Minecraft launch functionality:
//! - Build launch arguments (JVM args, game args, classpath)
//! - Launch game process
//! - Version isolation support
//! - Complete launch pipeline (inspired by PCL2)
//! - Game process monitoring and crash detection
//!
//! Architecture:
//! - pipeline.rs: 完整的启动流水线，支持并行执行和进度追踪
//! - watcher.rs: 游戏进程监控和崩溃检测
//! - mod.rs: 基础启动参数构建和进程启动

use crate::log_info;
use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::{setup::VersionSetup, state::VersionType};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod pipeline;
pub mod watcher;

// Re-export pipeline types
pub use pipeline::{
    LaunchConfig, LaunchPipeline, LaunchProgress, LaunchResult as PipelineLaunchResult, LaunchStage,
};

// Re-export watcher types
pub use watcher::{CrashCategory, CrashInfo, ExitInfo, GameState, GameWatcher, LoadProgress};

/// Launch arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchArguments {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub main_class: String,
    pub classpath: String,
    pub version_id: String,
    pub game_dir: String,
    pub assets_dir: String,
    pub asset_index: String,
    pub auth_info: AuthInfo,
}

/// Auth info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub client_token: String,
    pub login_type: String,
}

/// Build launch arguments with isolation support
pub fn build_launch_arguments(
    game_dir: &Path,
    version_id: &str,
    java_path: &Path,
    auth_info: &AuthInfo,
    min_memory: u32,
    max_memory: u32,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<&str>,
    server_port: Option<u32>,
    isolation_mode: u32,
    extra_jvm_args: &[String],
    extra_game_args: &[String],
) -> anyhow::Result<LaunchArguments> {
    let version_dir = game_dir.join("versions").join(version_id);
    let json_path = version_dir.join(format!("{}.json", version_id));

    if !json_path.exists() {
        return Err(anyhow::anyhow!("Version {} not found", version_id));
    }

    let json_content = std::fs::read_to_string(&json_path)?;
    let json: serde_json::Value = serde_json::from_str(&json_content)?;

    let main_class = json["mainClass"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("mainClass not found"))?
        .to_string();

    let classpath = build_classpath(game_dir, &json)?;
    let assets_dir = game_dir.join("assets").to_string_lossy().to_string();
    let asset_index = json["assetIndex"]["id"]
        .as_str()
        .or_else(|| json["assets"].as_str())
        .unwrap_or("legacy")
        .to_string();

    // 获取版本类型：优先从 setup.ini 读取，否则从 JSON 检测
    let version_type = match VersionSetup::load(&version_dir) {
        Ok(Some(setup)) => {
            log_info!(
                "Loaded version type from setup.ini: {:?}",
                setup.version_type
            );
            setup.version_type
        }
        _ => {
            let detected = VersionType::detect_from_json(version_id, &json);
            log_info!("Detected version type from JSON: {:?}", detected);
            detected
        }
    };

    // 计算隔离后的有效游戏目录
    let mode = IsolationMode::from_u32(isolation_mode);
    let effective_game_dir =
        isolation::get_effective_game_dir(game_dir, version_id, mode, version_type);

    // 确保隔离目录存在
    if effective_game_dir != game_dir {
        // 根据版本类型创建不同的目录结构
        let result = if version_type.is_modded() {
            isolation::ensure_modded_dirs(&effective_game_dir)
        } else {
            isolation::ensure_isolated_dirs(&effective_game_dir)
        };
        if let Err(e) = result {
            log_info!("Warning: Failed to create isolated dirs: {}", e);
        }
    }

    log_info!(
        "Game dir: {} -> effective: {} (isolation mode: {}, version type: {:?})",
        game_dir.display(),
        effective_game_dir.display(),
        isolation_mode,
        version_type
    );

    let jvm_args = build_jvm_args(
        game_dir, version_id, &classpath, min_memory, max_memory, java_path,
        extra_jvm_args, &json,
    )?;
    let game_args = build_game_args(
        &json,
        &effective_game_dir,
        version_id,
        &assets_dir,
        &asset_index,
        auth_info,
        window_width,
        window_height,
        server_address,
        server_port,
        extra_game_args,
    )?;

    // 在 launch 前设置游戏语言为中文（写入有效目录，适配隔离模式）
    if let Err(e) = crate::minecraft::language::set_game_language(
        &effective_game_dir,
        version_id,
        version_id, // 用 version_id 作为 MC 版本号（后续可从 setup.ini 读 OriginalVersion）
    ) {
        log_info!("[Language] Failed to set game language: {}", e);
    }

    Ok(LaunchArguments {
        jvm_args,
        game_args,
        main_class,
        classpath,
        version_id: version_id.to_string(),
        game_dir: effective_game_dir.to_string_lossy().to_string(),
        assets_dir,
        asset_index,
        auth_info: auth_info.clone(),
    })
}

/// Build classpath
fn build_classpath(game_dir: &Path, json: &serde_json::Value) -> anyhow::Result<String> {
    let mut entries = Vec::new();

    // 参考 PCL2 的 McLibListGet 函数
    // 递归查找最深层的继承版本来获取原版jar
    let jar_version = find_original_version(game_dir, json);
    let version_jar = game_dir
        .join("versions")
        .join(&jar_version)
        .join(format!("{}.jar", jar_version));

    if version_jar.exists() {
        entries.push(version_jar.to_string_lossy().to_string());
    } else {
        log_info!(
            "[Classpath] Warning: Main jar not found: {}",
            version_jar.display()
        );
    }

    if let Some(libraries) = json["libraries"].as_array() {
        for lib in libraries {
            // 应用 rules 过滤（平台适配）
            let rules: Option<Vec<serde_json::Value>> = lib
                .get("rules")
                .and_then(|v| v.as_array())
                .map(|a| a.clone());
            if !crate::minecraft::version::libraries::check_rules(&rules) {
                continue;
            }

            // 解析 maven name 判断是否有 classifier（如 "natives-windows"）
            // 有 classifier 的是 native 包，应通过 extract_natives 处理，不放入 classpath
            // 但对于"无 natives 字段但有 classifier"的新格式，需要特殊处理（见 extract_natives）
            let has_classifier = lib["name"]
                .as_str()
                .map(|n| n.split(':').count() > 3)
                .unwrap_or(false);

            // 优先用 downloads.artifact.path（更准确）
            if let Some(artifact) = lib.get("downloads").and_then(|d| d.get("artifact")) {
                if let Some(path) = artifact["path"].as_str() {
                    // 跳过 native 包（有 classifier 且 classifier 含 "natives"）
                    let is_native = has_classifier
                        && path.contains("natives-");
                    if is_native {
                        continue;
                    }
                    let lib_path = game_dir.join("libraries").join(path);
                    if lib_path.exists() {
                        entries.push(lib_path.to_string_lossy().to_string());
                    }
                }
            } else if let Some(name) = lib["name"].as_str() {
                // 没有 downloads.artifact，用 maven name 解析路径
                let path = maven_name_to_path(name);
                let lib_path = game_dir.join("libraries").join(&path);
                if lib_path.exists() {
                    entries.push(lib_path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(entries.join(if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }))
}

/// 递归查找最深层的继承版本（参考 PCL2 的 McLibListGet）
fn find_original_version(game_dir: &Path, json: &serde_json::Value) -> String {
    // 检查是否有 jar 字段指定
    if let Some(jar) = json.get("jar").and_then(|v| v.as_str()) {
        return jar.to_string();
    }

    // 检查 inheritsFrom
    if let Some(inherits_from) = json.get("inheritsFrom").and_then(|v| v.as_str()) {
        if !inherits_from.is_empty() {
            // 加载父版本JSON
            let parent_json_path = game_dir
                .join("versions")
                .join(inherits_from)
                .join(format!("{}.json", inherits_from));
            if parent_json_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&parent_json_path) {
                    if let Ok(parent_json) = serde_json::from_str::<serde_json::Value>(&content) {
                        // 递归查找
                        return find_original_version(game_dir, &parent_json);
                    }
                }
            }
            // 如果父版本不存在，返回inheritsFrom作为版本名
            return inherits_from.to_string();
        }
    }

    // 没有继承，使用当前版本
    json.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Convert Maven name to path
fn maven_name_to_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.to_string();
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = if parts.len() > 3 { parts[3] } else { "" };

    if classifier.is_empty() {
        format!(
            "{}/{}/{}/{}-{}.jar",
            group, artifact, version, artifact, version
        )
    } else {
        format!(
            "{}/{}/{}/{}-{}-{}.jar",
            group, artifact, version, artifact, version, classifier
        )
    }
}

/// Build JVM arguments
fn build_jvm_args(
    game_dir: &Path,
    version_id: &str,
    classpath: &str,
    min_memory: u32,
    max_memory: u32,
    java_path: &Path,
    extra_jvm_args: &[String],
    json: &serde_json::Value,
) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();

    args.push(format!("-Xms{}M", min_memory));
    args.push(format!("-Xmx{}M", max_memory));

    let java_version = get_java_version(java_path);
    if let Some(version) = java_version {
        if version >= 21 {
            args.push("-XX:+UseZGC".to_string());
            args.push("-XX:+ZGenerational".to_string());
        } else if version >= 15 {
            args.push("-XX:+UseZGC".to_string());
        } else {
            args.push("-XX:+UseG1GC".to_string());
        }
    }

    // 版本 JSON 的 arguments.jvm（必需 JVM 参数，如 -Djava.net.preferIPv6Addresses=system）
    // 参考 PCL2：解析 arguments.jvm，应用 rules 过滤，跳过已处理的 -cp 和 -Djava.library.path
    if let Some(jvm_args_json) = json["arguments"]["jvm"].as_array() {
        for arg in jvm_args_json {
            let (value, rules) = if let Some(s) = arg.as_str() {
                (s.to_string(), None)
            } else if let Some(obj) = arg.as_object() {
                let value = obj.get("value").and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = v.as_array() {
                        Some(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>().join(" "))
                    } else {
                        None
                    }
                });
                let rules = obj.get("rules").and_then(|r| r.as_array()).map(|a| a.clone());
                match value {
                    Some(v) => (v, rules),
                    None => continue,
                }
            } else {
                continue;
            };

            // 应用 rules 过滤
            if !crate::minecraft::version::libraries::check_rules(&rules) {
                continue;
            }

            // 跳过已处理的参数
            if value.contains("${classpath}") || value.contains("${natives_directory}") {
                continue;
            }

            args.push(value);
        }
    }

    // 用户额外 JVM 参数（版本独立 > 全局，参考 PCL2 的 AdvanceJvm）
    args.extend(extra_jvm_args.iter().cloned());

    args.push("-cp".to_string());
    args.push(classpath.to_string());

    let natives_dir = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}-natives", version_id));
    args.push(format!(
        "-Djava.library.path={}",
        natives_dir.to_string_lossy()
    ));

    Ok(args)
}

/// Get Java version
fn get_java_version(java_path: &Path) -> Option<u32> {
    let output = std::process::Command::new(java_path)
        .arg("-version")
        .output()
        .ok()?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
    let re = RE
        .get_or_init(|| regex::Regex::new(r#"version "(\d+)\."#).ok())
        .as_ref()?;
    let captures = re.captures(&stderr)?;
    captures[1].parse().ok()
}

/// Build game arguments
fn build_game_args(
    json: &serde_json::Value,
    game_dir: &Path,
    version_id: &str,
    assets_dir: &str,
    asset_index: &str,
    auth_info: &AuthInfo,
    window_width: Option<u32>,
    window_height: Option<u32>,
    server_address: Option<&str>,
    server_port: Option<u32>,
    extra_game_args: &[String],
) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();

    if let Some(game_args) = json["arguments"]["game"].as_array() {
        for arg in game_args {
            let (value, rules) = if let Some(s) = arg.as_str() {
                (s.to_string(), None)
            } else if let Some(obj) = arg.as_object() {
                let value = obj.get("value").and_then(|v| {
                    if let Some(s) = v.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = v.as_array() {
                        Some(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>().join(" "))
                    } else {
                        None
                    }
                });
                let rules = obj.get("rules").and_then(|r| r.as_array()).map(|a| a.clone());
                match value {
                    Some(v) => (v, rules),
                    None => continue,
                }
            } else {
                continue;
            };

            if !crate::minecraft::version::libraries::check_rules(&rules) {
                continue;
            }

            args.push(value);
        }
    } else if let Some(mc_args) = json["minecraftArguments"].as_str() {
        for arg in mc_args.split(' ') {
            args.push(arg.to_string());
        }
    }

    // 如果 arguments.game 未提供标准 Minecraft 客户端参数（如 Forge 26.2 自包含 JSON），
    // 自动补充必需参数（参考 Mojang 原版 JSON 的 arguments.game 模板）
    if !args.iter().any(|a| a == "--accessToken") {
        let mut std_args = vec![
            "--username".to_string(),
            "${auth_player_name}".to_string(),
            "--version".to_string(),
            "${version_name}".to_string(),
            "--gameDir".to_string(),
            "${game_directory}".to_string(),
            "--assetsDir".to_string(),
            "${assets_root}".to_string(),
            "--assetIndex".to_string(),
            "${assets_index_name}".to_string(),
            "--uuid".to_string(),
            "${auth_uuid}".to_string(),
            "--accessToken".to_string(),
            "${auth_access_token}".to_string(),
            "--userType".to_string(),
            "${user_type}".to_string(),
            "--versionType".to_string(),
            "${version_type}".to_string(),
        ];
        std_args.extend(args);
        args = std_args;
    }

    let mut final_args = Vec::new();
    for arg in args {
        let replaced = arg
            .replace("${auth_player_name}", &auth_info.username)
            .replace("${auth_session}", &auth_info.access_token)
            .replace("${auth_uuid}", &auth_info.uuid)
            .replace("${auth_access_token}", &auth_info.access_token)
            .replace("${auth_client_token}", &auth_info.client_token)
            .replace("${user_type}", &auth_info.login_type)
            .replace("${version_name}", version_id)
            .replace("${game_directory}", &game_dir.to_string_lossy())
            .replace("${game_assets}", assets_dir)
            .replace("${assets_root}", assets_dir)
            .replace("${assets_index_name}", asset_index)
            .replace("${user_properties}", "{}")
            .replace("${version_type}", "MoLaunch");
        final_args.push(replaced);
    }

    if let (Some(width), Some(height)) = (window_width, window_height) {
        final_args.push("--width".to_string());
        final_args.push(width.to_string());
        final_args.push("--height".to_string());
        final_args.push(height.to_string());
    }

    if let Some(server) = server_address {
        final_args.push("--server".to_string());
        final_args.push(server.to_string());
        if let Some(port) = server_port {
            final_args.push("--port".to_string());
            final_args.push(port.to_string());
        }
    }

    // 用户额外游戏参数（参考 PCL2 的 AdvanceGame）
    final_args.extend(extra_game_args.iter().cloned());

    Ok(final_args)
}

/// Launch game process
pub fn launch_game(
    java_path: &Path,
    arguments: &LaunchArguments,
    game_dir: &Path,
) -> anyhow::Result<u32> {
    let mut cmd = std::process::Command::new(java_path);

    for arg in &arguments.jvm_args {
        cmd.arg(arg);
    }

    cmd.arg(&arguments.main_class);

    for arg in &arguments.game_args {
        cmd.arg(arg);
    }

    cmd.current_dir(game_dir);

    let child = cmd.spawn()?;
    let pid = child.id();

    log_info!("Game launched with PID: {}", pid);

    Ok(pid)
}
