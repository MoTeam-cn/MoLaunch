//! setup.ini 解析与版本号检测的辅助函数

use std::path::{Path, PathBuf};

use super::types::VersionSetup;

/// 简单的 INI 解析器（flat：忽略 section，按 key 聚合到 HashMap）
///
/// 注意：setup.ini 的字段在 [info] 和 [Memory] 两个 section 中且字段名全局唯一，
/// 因此 flat 解析与段感知解析行为一致。底层委托 `storage::ini::IniFile`，
/// 避免重复实现 BOM 剥离、注释跳过、键值解析等逻辑。
pub(crate) fn parse_ini(content: &str) -> std::collections::HashMap<String, String> {
    let ini = crate::storage::ini::IniFile::parse(content);
    let mut map = std::collections::HashMap::new();
    for section in ini.sections() {
        for (k, v) in ini.get_section(&section) {
            map.insert(k, v);
        }
    }
    map
}

/// 从 Maven 坐标提取版本号
pub(crate) fn extract_maven_version(name: &str, prefix: &str) -> Option<String> {
    name.strip_prefix(prefix).map(|s| s.to_string())
}

/// 从 setup.ini 读取 `(OriginalVersion, loader-Type)`。
/// loader 仅当 Type 非 release/snapshot 时返回（视为 modloader 类型）。
/// setup.ini 不存在或读取失败时返回 `(None, None)`。
pub fn read_setup_version_and_loader(version_dir: &Path) -> (Option<String>, Option<String>) {
    let setup_path = version_dir.join("setup.ini");
    if !setup_path.exists() {
        return (None, None);
    }
    let Ok(content) = std::fs::read_to_string(&setup_path) else {
        return (None, None);
    };
    let mut mc_version = None;
    let mut loader = None;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("OriginalVersion=") {
            let v = value.trim().to_string();
            if !v.is_empty() {
                mc_version = Some(v);
            }
        } else if let Some(value) = line.strip_prefix("Type=") {
            let t = value.trim().to_lowercase();
            if !t.is_empty() && t != "release" && t != "snapshot" {
                loader = Some(t);
            }
        }
    }
    (mc_version, loader)
}

/// 从 version.json 读取 Mojang 版本号（优先 inheritsFrom，否则 id，否则回退 version_id）。
pub fn read_mc_version_from_json(version_dir: &Path, version_id: &str) -> String {
    let json_path = version_dir.join(format!("{}.json", version_id));
    if json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&json_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(inherits_from) = json.get("inheritsFrom").and_then(|v| v.as_str()) {
                    if !inherits_from.is_empty() {
                        return inherits_from.to_string();
                    }
                }
                if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
                    return id.to_string();
                }
            }
        }
    }
    version_id.to_string()
}

/// 从 setup.ini 或 version.json 读取 MC 版本号和加载器类型（统一入口，消除重复实现）。
/// 优先 setup.ini 的 OriginalVersion/Type，缺失则从 version.json 读取版本号。
pub fn detect_version_and_loader(version_dir: &Path, version_id: &str) -> (String, Option<String>) {
    let (mc_version, loader) = read_setup_version_and_loader(version_dir);
    let mc_version =
        mc_version.unwrap_or_else(|| read_mc_version_from_json(version_dir, version_id));
    (mc_version, loader)
}

impl VersionSetup {
    /// 获取 setup.ini 文件路径
    pub fn file_path(version_dir: &Path) -> PathBuf {
        version_dir.join("setup.ini")
    }

    /// 检查 setup.ini 是否存在
    pub fn exists(version_dir: &Path) -> bool {
        Self::file_path(version_dir).exists()
    }
}
