//! 资源管理模块
//!
//! 统一管理所有外部资源文件的路径
//! 所有资源文件都放在 src-tauri/resources/ 目录下

use std::path::PathBuf;

/// 获取资源根目录路径
pub fn get_resources_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
}

/// 获取资源文件完整路径
pub fn get_resource_path(relative_path: &str) -> PathBuf {
    get_resources_dir().join(relative_path)
}

/// 读取资源文件内容（文本）
pub fn read_resource(relative_path: &str) -> anyhow::Result<String> {
    let path = get_resource_path(relative_path);
    Ok(std::fs::read_to_string(&path)?)
}

/// 读取资源文件内容（二进制）
pub fn read_resource_bytes(relative_path: &str) -> anyhow::Result<Vec<u8>> {
    let path = get_resource_path(relative_path);
    Ok(std::fs::read(&path)?)
}

/// 检查资源文件是否存在
pub fn exists(relative_path: &str) -> bool {
    get_resource_path(relative_path).exists()
}

/// 列出目录下的资源文件
pub fn list_dir(relative_path: &str) -> Vec<String> {
    let path = get_resource_path(relative_path);
    let mut entries = Vec::new();

    if path.exists() && path.is_dir() {
        if let Ok(dir) = std::fs::read_dir(&path) {
            for entry in dir.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
    }

    entries
}

/// 释放资源文件到目标路径
pub fn extract_resource(resource_path: &str, target_path: &PathBuf) -> anyhow::Result<()> {
    let content = read_resource_bytes(resource_path)?;
    
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // 只在文件不存在或大小不同时写入
    let need_write = if target_path.exists() {
        let existing_size = std::fs::metadata(target_path)?.len();
        existing_size != content.len() as u64
    } else {
        true
    };

    if need_write {
        std::fs::write(target_path, &content)?;
        log::info!("Extracted resource: {} -> {}", resource_path, target_path.display());
    }

    Ok(())
}

// ========== 常用资源路径常量 ==========

/// 默认配置模板路径
pub fn default_config_path() -> PathBuf {
    get_resource_path("defaults/config.ini")
}

/// 默认实例信息模板路径
pub fn default_instance_path() -> PathBuf {
    get_resource_path("defaults/instance.ini")
}

/// Forge 安装器 JAR 路径
pub fn forge_installer_path() -> PathBuf {
    get_resource_path("forge-installer.jar")
}

/// Java Wrapper JAR 路径
pub fn java_wrapper_path() -> PathBuf {
    get_resource_path("java-wrapper.jar")
}
