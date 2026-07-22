//! Java 检测模块

use std::path::Path;

use super::JavaRuntime;

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

    // [4] 运行 java -version（走 shell 模块统一封装）
    let output = match crate::minecraft::system::shell::run_executable_output(
        &java_str,
        &["-version".to_string()],
        None,
    ) {
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

/// 提取并标准化版本号
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
