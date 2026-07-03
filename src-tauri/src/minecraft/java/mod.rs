//! Java检测和管理模块

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Java运行时信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRuntime {
    /// java.exe完整路径
    pub executable: String,
    /// 所在文件夹
    pub path_folder: String,
    /// 是否手动导入
    pub is_user_import: bool,
    /// 详细版本号（如1.21.0.1）
    pub version: String,
    /// 大版本号
    pub major_version: u32,
    /// 是否为JRE
    pub is_jre: bool,
    /// 是否64位
    pub is_64bit: bool,
}

/// 检测Java
pub fn detect_java(java_path: &Path) -> Result<JavaRuntime, Box<dyn std::error::Error>> {
    if !java_path.exists() {
        return Err("Java executable not found".into());
    }
    
    let java_str = java_path.to_string_lossy().to_string();
    
    // 运行java -version获取输出
    let output = std::process::Command::new(&java_str)
        .arg("-version")
        .output()?;
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let version_output = format!("{}{}", stderr, stdout);
    
    // 提取版本号
    let version = extract_java_version(&version_output)?;
    let major_version = extract_major_version(&version)?;
    
    // 检测是否为64位
    let is_64bit = version_output.contains("64-bit");
    
    // 检测是否为JRE
    let is_jre = !version_output.contains("Java(TM) SE Runtime Environment") || 
                 version_output.contains("Server VM");
    
    // 获取所在文件夹
    let path_folder = java_path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
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

/// 提取Java版本号
fn extract_java_version(output: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 尝试匹配 "version " 后的版本号
    let re = regex::Regex::new(r#"version "([^"]+)""#)?;
    if let Some(captures) = re.captures(output) {
        return Ok(captures[1].to_string());
    }
    
    // 尝试匹配其他格式
    let re = regex::Regex::new(r"version (\d+\.\d+\.\d+[_\d]*)")?;
    if let Some(captures) = re.captures(output) {
        return Ok(captures[1].to_string());
    }
    
    Err("Failed to extract Java version".into())
}

/// 提取主版本号
fn extract_major_version(version: &str) -> Result<u32, Box<dyn std::error::Error>> {
    // 处理新版本格式（如17.0.1）
    if let Some(first_part) = version.split('.').next() {
        if let Ok(major) = first_part.parse::<u32>() {
            if major >= 1 {
                return Ok(major);
            }
        }
    }
    
    // 处理旧版本格式（如1.8.0_361）
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2 {
        if let Ok(major) = parts[1].parse::<u32>() {
            return Ok(major);
        }
    }
    
    Err("Failed to extract major version".into())
}

/// 搜索系统中的Java
pub fn search_java() -> Vec<JavaRuntime> {
    let mut java_list = Vec::new();
    
    // 检查环境变量
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let java_path = dir.join("java.exe");
            if java_path.exists() {
                if let Ok(java) = detect_java(&java_path) {
                    java_list.push(java);
                }
            }
        }
    }
    
    // 检查JAVA_HOME
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java_path = Path::new(&java_home).join("bin").join("java.exe");
        if java_path.exists() {
            if let Ok(java) = detect_java(&java_path) {
                java_list.push(java);
            }
        }
    }
    
    // 检查常见安装路径
    let common_paths = get_common_java_paths();
    for path in common_paths {
        if path.exists() {
            if let Ok(java) = detect_java(&path) {
                java_list.push(java);
            }
        }
    }
    
    // 去重
    java_list.sort_by(|a, b| a.executable.cmp(&b.executable));
    java_list.dedup_by(|a, b| a.executable == b.executable);
    
    java_list
}

/// 获取常见的Java安装路径
fn get_common_java_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Windows常见路径
    if cfg!(target_os = "windows") {
        // Program Files
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            let base = Path::new(&program_files);
            paths.extend(get_java_paths_in_dir(base));
        }
        
        // Program Files (x86)
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            let base = Path::new(&program_files_x86);
            paths.extend(get_java_paths_in_dir(base));
        }
        
        // 用户目录
        if let Some(user_profile) = std::env::var_os("USERPROFILE") {
            let base = Path::new(&user_profile);
            paths.extend(get_java_paths_in_dir(&base.join(".jdks")));
            paths.extend(get_java_paths_in_dir(&base.join(".sdkman")));
        }
    }
    
    paths
}

/// 获取目录下的Java路径
fn get_java_paths_in_dir(base: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if !base.exists() || !base.is_dir() {
        return paths;
    }
    
    // 查找java.exe
    let java_path = base.join("bin").join("java.exe");
    if java_path.exists() {
        paths.push(java_path);
    }
    
    // 查找子目录中的java.exe
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let java_path = path.join("bin").join("java.exe");
                if java_path.exists() {
                    paths.push(java_path);
                }
            }
        }
    }
    
    paths
}

/// 选择最佳Java
pub fn select_best_java(java_list: &[JavaRuntime], min_version: Option<u32>, max_version: Option<u32>) -> Option<&JavaRuntime> {
    let mut candidates: Vec<&JavaRuntime> = java_list.iter().collect();
    
    // 过滤版本范围
    if let Some(min) = min_version {
        candidates.retain(|java| java.major_version >= min);
    }
    if let Some(max) = max_version {
        candidates.retain(|java| java.major_version <= max);
    }
    
    // 排序规则：
    // 1. 64位优先
    // 2. JRE优先于JDK
    // 3. 版本号越高越好（但Java 17权重最高）
    candidates.sort_by(|a, b| {
        // 64位优先
        if a.is_64bit != b.is_64bit {
            return b.is_64bit.cmp(&a.is_64bit);
        }
        
        // JRE优先
        if a.is_jre != b.is_jre {
            return b.is_jre.cmp(&a.is_jre);
        }
        
        // 版本权重
        let a_weight = get_java_version_weight(a.major_version);
        let b_weight = get_java_version_weight(b.major_version);
        b_weight.cmp(&a_weight)
    });
    
    candidates.first().map(|&java| java)
}

/// 获取Java版本权重
fn get_java_version_weight(major_version: u32) -> u32 {
    match major_version {
        17 => 31, // Java 17权重最高
        8 => 30,  // Java 8次之
        21 => 29, // Java 21
        16 => 28,
        11 => 27,
        _ => major_version,
    }
}

/// 根据MC版本获取Java版本需求
pub fn get_java_requirements(mc_version: &str) -> (Option<u32>, Option<u32>) {
    let version_parts: Vec<&str> = mc_version.split('.').collect();
    if version_parts.len() < 2 {
        return (Some(8), None); // 默认要求Java 8+
    }
    
    let major: u32 = version_parts[0].parse().unwrap_or(1);
    let minor: u32 = version_parts[1].parse().unwrap_or(0);
    
    match (major, minor) {
        // MC 1.20.5+ 需要 Java 21+
        (1, 20) if minor >= 5 => (Some(21), None),
        (1, minor) if minor > 20 => (Some(21), None),
        // MC 1.18+ 需要 Java 17+
        (1, 18..=20) => (Some(17), None),
        // MC 1.17+ 需要 Java 16+
        (1, 17) => (Some(16), None),
        // MC 1.12+ 需要 Java 8+
        (1, 12..=16) => (Some(8), None),
        // MC 1.5.2- 最高支持 Java 8
        (1, minor) if minor <= 5 => (None, Some(8)),
        // 默认
        _ => (Some(8), None),
    }
}