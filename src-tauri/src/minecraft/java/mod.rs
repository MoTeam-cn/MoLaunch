//! Java检测和管理模块
//! 参考PCL2的Java搜索和版本检测逻辑

pub mod download;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Java运行时信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRuntime {
    pub executable: String,
    pub path_folder: String,
    pub is_user_import: bool,
    pub version: String,
    pub major_version: u32,
    pub is_jre: bool,
    pub is_64bit: bool,
}

/// 搜索关键词（参考PCL2，共67个）
const SEARCH_KEYWORDS: &[&str] = &[
    "java",
    "jdk",
    "jre",
    "env",
    "run",
    "mc",
    "dragon",
    "well",
    "bin",
    "sdk",
    "candidate",
    "current",
    "software",
    "cache",
    "temp",
    "corretto",
    "roaming",
    "users",
    "craft",
    "program",
    "net",
    "oracle",
    "game",
    "file",
    "data",
    "jvm",
    "server",
    "client",
    "mojang",
    "eclipse",
    "microsoft",
    "hotspot",
    "runtime",
    "x86",
    "x64",
    "arm",
    "forge",
    "optifine",
    "hmcl",
    "mod",
    "fabric",
    "download",
    "launch",
    "path",
    "version",
    "pcl",
    "zulu",
    "local",
    "packages",
    "jbr",
    "bellsoft",
    "liberica",
    "graal",
    "adoptium",
    "temurin",
    "semerulu",
    "1.",
];

/// 检测单个Java
pub fn detect_java(java_path: &Path) -> Result<JavaRuntime, String> {
    if !java_path.exists() {
        return Err("Java executable not found".into());
    }

    let java_str = java_path.to_string_lossy().to_string();

    // [2] 黑名单检查
    let path_lower = java_str.to_lowercase();
    if path_lower.contains("finalshell") || path_lower.contains("paranoia file") {
        return Err("Incompatible Java variant".into());
    }

    // [3] JRE/JDK判定
    let parent_dir = java_path.parent().unwrap_or(Path::new(""));
    let javac_path = parent_dir.join("javac.exe");
    let is_jre = !javac_path.exists();

    // [4] 运行 java -version
    let output = match std::process::Command::new(&java_str)
        .arg("-version")
        .output()
    {
        Ok(output) => output,
        Err(e) => return Err(format!("Failed to execute Java: {}", e)),
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_output = format!("{}{}", stderr, stdout).to_lowercase();

    if version_output.is_empty() {
        return Err("No output from java -version".into());
    }

    // [5] 输出异常检测
    if version_output.contains("/lib/ext exists") {
        return Err("Java has /lib/ext issue".into());
    }
    if version_output.contains("a fatal error") {
        return Err("Java fatal error".into());
    }

    // [6] 版本号提取与标准化
    let version = extract_and_normalize_version(&version_output)?;
    let major_version = extract_major_version(&version)?;

    // [8] 架构检测
    let is_64bit = version_output.contains("64-bit");

    // [9] 版本合理性验证
    if major_version < 5 || major_version > 99 {
        return Err(format!("Invalid major version: {}", major_version));
    }

    let path_folder = parent_dir.to_string_lossy().to_string();

    Ok(JavaRuntime {
        executable: java_str,
        path_folder,
        is_user_import: false,
        version,
        major_version,
        is_jre,
        is_64bit,
    })
}

/// 轻量级版本检测：仅返回 Java 大版本号（用于兼容性检查）
/// 路径可以是 java.exe 的完整路径，也可以是文件夹
pub fn detect_java_version(java_path_or_dir: &str) -> Option<u32> {
    let path = std::path::Path::new(java_path_or_dir);
    // 如果是目录，尝试找 java.exe
    let java_exe = if path.is_dir() {
        let candidates = ["java.exe", "bin\\java.exe", "bin/java.exe"];
        let mut found = None;
        for c in &candidates {
            let p = path.join(c);
            if p.exists() {
                found = Some(p);
                break;
            }
        }
        found?
    } else {
        path.to_path_buf()
    };

    detect_java(&java_exe).ok().map(|j| j.major_version)
}

/// 提取并标准化版本号（参考PCL2第107-121行）
fn extract_and_normalize_version(output: &str) -> Result<String, String> {
    // 正则1: version "xxx"
    static RE1: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re1 = RE1.get_or_init(|| regex::Regex::new(r#"version "([^"]+)""#).unwrap());
    // 正则2: openjdk xxx
    static RE2: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re2 = RE2.get_or_init(|| regex::Regex::new(r"openjdk (\d+)").unwrap());

    let mut version_str = if let Some(caps) = re1.captures(output) {
        caps[1].to_string()
    } else if let Some(caps) = re2.captures(output) {
        caps[1].to_string()
    } else {
        return Err("Failed to extract version".into());
    };

    // 下划线转点
    version_str = version_str.replace('_', ".");
    // 取连字符前
    if let Some(pos) = version_str.find('-') {
        version_str = version_str[..pos].to_string();
    }

    // 防御多余段数
    let dots = version_str.matches('.').count();
    if dots > 3 {
        version_str = version_str.replace(".0.", ".");
    }

    // 补齐到4段
    while version_str.matches('.').count() < 3 {
        if version_str.starts_with("1.") {
            version_str.push_str(".0");
        } else {
            version_str = format!("1.{}", version_str);
        }
    }

    Ok(version_str)
}

/// 提取主版本号
fn extract_major_version(version: &str) -> Result<u32, String> {
    let parts: Vec<&str> = version.split('.').collect();

    // 新版本格式 (17.0.2.0)
    if let Some(first) = parts.first() {
        if let Ok(major) = first.parse::<u32>() {
            if major >= 5 {
                return Ok(major);
            }
        }
    }

    // 旧版本格式 (1.8.0.321)
    if parts.len() >= 2 {
        if let Ok(first) = parts[0].parse::<u32>() {
            if first == 1 {
                if let Ok(major) = parts[1].parse::<u32>() {
                    if major >= 5 {
                        return Ok(major);
                    }
                }
            }
        }
    }

    Err(format!("Failed to extract major version from: {}", version))
}

/// 搜索系统中的Java
pub fn search_java() -> Vec<JavaRuntime> {
    search_java_with_paths(&[])
}

/// 带额外搜索路径的 Java 搜索
///
/// `extra_paths` 用于追加搜索根目录（如游戏目录、APPDATA 等），会全遍历搜索。
/// 参考 PCL2 `JavaSearchFolder(..., IsFullSearch:=True)`。
pub fn search_java_with_paths(extra_paths: &[PathBuf]) -> Vec<JavaRuntime> {
    crate::log_separator!("Java Search");
    crate::log_info!("[Java] Starting Java search...");

    let mut java_candidates: Vec<PathBuf> = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    let mut add_candidate = |path: &Path| {
        let path_str = path.to_string_lossy().to_lowercase().replace("\\", "/");
        if !seen_paths.contains(&path_str) {
            seen_paths.insert(path_str);
            java_candidates.push(path.to_path_buf());
            crate::log_debug!("[Java] Candidate: {}", path.display());
        }
    };

    // Step 1: 环境变量扫描
    crate::log_info!("[Java] Step 1: Checking environment variables...");
    let mut env_paths = String::new();
    if let Ok(path) = std::env::var("PATH") {
        env_paths.push_str(&path);
    }
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        crate::log_debug!("[Java] JAVA_HOME: {}", java_home);
        env_paths.push(';');
        env_paths.push_str(&java_home);
        env_paths.push(';');
        env_paths.push_str(&format!("{}\\bin", java_home));
    }
    for dir in std::env::split_paths(&env_paths) {
        let dir_str = dir.to_string_lossy().to_lowercase().replace("\\", "/");
        if dir_str.is_empty() {
            continue;
        }
        // 粗略检查 javaw.exe
        let javaw_path = dir.join("javaw.exe");
        let java_path = dir.join("java.exe");
        if javaw_path.exists() {
            add_candidate(&javaw_path);
        } else if java_path.exists() {
            add_candidate(&java_path);
        }
    }

    // Step 2: 全磁盘扫描（关键词匹配）
    crate::log_info!("[Java] Step 2: Searching local drives...");
    for drive in get_local_drives() {
        search_folder_recursive(&drive, &mut add_candidate, false);
    }

    // Step 3: 用户目录深度搜索
    crate::log_info!("[Java] Step 3: Searching user directories...");
    if let Some(user_profile) = std::env::var_os("USERPROFILE") {
        let base = Path::new(&user_profile);
        search_folder_recursive(base, &mut add_candidate, false);
        // .jdks 全搜索
        search_folder_recursive(&base.join(".jdks"), &mut add_candidate, true);
        // .sdkman 全搜索
        search_folder_recursive(
            &base.join(".sdkman/candidates/java"),
            &mut add_candidate,
            true,
        );
    }

    // Step 4: 启动器目录全搜索
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            crate::log_debug!(
                "[Java] Step 4: Searching launcher directory: {}",
                exe_dir.display()
            );
            search_folder_recursive(exe_dir, &mut add_candidate, true);
        }
    }

    // Step 5: APPDATA\.minecraft\runtime\（PCL2/官启自动下载的 Java 存放处）
    // 与 PCL2 一致，runtime 下的 Java 跨游戏目录共享，必须搜索
    crate::log_info!("[Java] Step 5: Searching APPDATA .minecraft runtime...");
    if let Ok(appdata) = std::env::var("APPDATA") {
        let runtime_dir = Path::new(&appdata).join(".minecraft").join("runtime");
        if runtime_dir.exists() {
            crate::log_debug!(
                "[Java] Step 5: Searching runtime directory: {}",
                runtime_dir.display()
            );
            search_folder_recursive(&runtime_dir, &mut add_candidate, true);
        }
    }

    // Step 6: 调用方追加的额外搜索路径（如游戏目录）
    for (i, extra) in extra_paths.iter().enumerate() {
        if extra.exists() && extra.is_dir() {
            crate::log_debug!(
                "[Java] Step 6.{}: Searching extra path: {}",
                i,
                extra.display()
            );
            search_folder_recursive(extra, &mut add_candidate, true);
        }
    }

    crate::log_info!(
        "[Java] Found {} candidates, verifying...",
        java_candidates.len()
    );

    // 验证所有候选Java
    let mut java_list = Vec::new();
    for path in &java_candidates {
        match detect_java(path) {
            Ok(java) => {
                crate::log_info!("[Java] Valid: {} ({})", java.version, java.path_folder);
                java_list.push(java);
            }
            Err(e) => {
                crate::log_debug!("[Java] Invalid {}: {}", path.display(), e);
            }
        }
    }

    // 排序
    java_list.sort_by(|a, b| {
        b.major_version
            .cmp(&a.major_version)
            .then(b.is_64bit.cmp(&a.is_64bit))
    });

    crate::log_info!(
        "[Java] Search completed, found {} valid Java installations",
        java_list.len()
    );
    crate::log_separator!("Java Search End");

    java_list
}

/// 递归搜索文件夹（参考PCL2的JavaSearchFolder）
fn search_folder_recursive<F>(dir: &Path, add_candidate: &mut F, is_full_search: bool)
where
    F: FnMut(&Path),
{
    if !dir.exists() || !dir.is_dir() {
        return;
    }

    // 检查当前目录是否有 javaw.exe 或 java.exe
    let javaw_path = dir.join("javaw.exe");
    let java_path = dir.join("java.exe");
    if javaw_path.exists() {
        add_candidate(&javaw_path);
    } else if java_path.exists() {
        add_candidate(&java_path);
    }

    // 遍历子目录
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // 跳过符号链接
        if is_symlink(&path) {
            continue;
        }

        let dir_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();

        // 判断是否需要递归搜索
        let should_search = is_full_search ||
            dir_name == "users" ||
            dir_name.parse::<f64>().is_ok() ||  // 数字开头
            dir_name == "bin" ||
            SEARCH_KEYWORDS.iter().any(|kw| dir_name.contains(kw));

        if should_search {
            search_folder_recursive(&path, add_candidate, false);
        }
    }
}

fn is_symlink(path: &Path) -> bool {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.file_type().is_symlink(),
        Err(_) => false,
    }
}

fn get_local_drives() -> Vec<PathBuf> {
    let mut drives = Vec::new();
    for letter in 'A'..='Z' {
        let drive = format!("{}:\\", letter);
        let path = PathBuf::from(&drive);
        if path.exists() {
            drives.push(path);
        }
    }
    drives
}

pub fn select_best_java(
    java_list: &[JavaRuntime],
    min_version: Option<u32>,
    max_version: Option<u32>,
) -> Option<&JavaRuntime> {
    let mut candidates: Vec<&JavaRuntime> = java_list.iter().collect();

    if let Some(min) = min_version {
        candidates.retain(|java| java.major_version >= min);
    }
    if let Some(max) = max_version {
        candidates.retain(|java| java.major_version <= max);
    }

    // 排序：64位优先，JRE优先，版本权重
    candidates.sort_by(|a, b| {
        if a.is_64bit != b.is_64bit {
            return b.is_64bit.cmp(&a.is_64bit);
        }
        if a.is_jre != b.is_jre {
            return b.is_jre.cmp(&a.is_jre);
        }
        let a_weight = get_java_version_weight(a.major_version);
        let b_weight = get_java_version_weight(b.major_version);
        b_weight.cmp(&a_weight)
    });

    candidates.first().map(|&java| java)
}

/// Java版本权重（参考PCL2）
fn get_java_version_weight(major_version: u32) -> u32 {
    match major_version {
        7 => 0,
        8 => 30, // Java 8 权重最高
        9 => 4,
        10 => 5,
        11 => 14,
        12 => 6,
        13 => 7,
        14 => 8,
        15 => 9,
        16 => 12,
        17 => 31, // Java 17 权重最高
        18 => 13,
        19 => 10,
        20 => 11,
        21 => 29,
        _ => major_version,
    }
}
