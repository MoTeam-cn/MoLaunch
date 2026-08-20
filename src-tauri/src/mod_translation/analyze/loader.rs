//! 加载器探测：fabric.mod.json / mods.toml / neoforge.mods.toml 元数据提取

use std::path::Path;

use super::super::types::Loader;

/// 探测加载器：fabric.mod.json / META-INF/mods.toml / META-INF/neoforge.mods.toml / mcmod.info
pub fn detect_loader(workspace: &Path) -> (Loader, Vec<String>, Vec<String>) {
    if workspace.join("fabric.mod.json").is_file() {
        let mod_id = read_fabric_mod_id(&workspace.join("fabric.mod.json"));
        return (Loader::Fabric, mod_id.into_iter().collect(), Vec::new());
    }
    if workspace.join("META-INF/neoforge.mods.toml").is_file() {
        let ids = read_mods_toml_ids(&workspace.join("META-INF/neoforge.mods.toml"));
        return (Loader::NeoForge, ids, Vec::new());
    }
    if workspace.join("META-INF/mods.toml").is_file() {
        let ids = read_mods_toml_ids(&workspace.join("META-INF/mods.toml"));
        return (Loader::Forge, ids, Vec::new());
    }
    (Loader::Unknown, Vec::new(), Vec::new())
}

fn read_fabric_mod_id(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    value
        .get("id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// 从 mods.toml 提取 modId（TOML 简易解析：`modId = "xxx"`）
fn read_mods_toml_ids(path: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("modId") {
            if let Some(rest) = rest.trim_start().strip_prefix('=') {
                let id = rest.trim().trim_matches('"').to_string();
                if !id.is_empty() {
                    ids.push(id);
                }
            }
        }
    }
    ids
}

/// 探测元数据：加载器 + modId + 项目名 + 版本（内部复用 detect_loader）
pub fn detect_metadata(workspace: &Path) -> (Loader, Vec<String>, Vec<String>, Option<String>) {
    let (loader, mod_ids, _) = detect_loader(workspace);
    match loader {
        Loader::Fabric => {
            let Ok(content) = std::fs::read_to_string(workspace.join("fabric.mod.json")) else {
                return (loader, mod_ids, Vec::new(), None);
            };
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
                return (loader, mod_ids, Vec::new(), None);
            };
            let project_names = value
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .into_iter()
                .collect();
            let version = value
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (loader, mod_ids, project_names, version)
        }
        Loader::NeoForge | Loader::Forge => {
            let path = if loader == Loader::NeoForge {
                workspace.join("META-INF/neoforge.mods.toml")
            } else {
                workspace.join("META-INF/mods.toml")
            };
            let Ok(content) = std::fs::read_to_string(path) else {
                return (loader, mod_ids, Vec::new(), None);
            };
            let project_names = extract_toml_values(&content, "displayName");
            let version = extract_toml_values(&content, "version").into_iter().next();
            (loader, mod_ids, project_names, version)
        }
        Loader::Unknown => (loader, mod_ids, Vec::new(), None),
    }
}

/// 正则提取 TOML 键值（支持双引号与单引号）
fn extract_toml_values(text: &str, key: &str) -> Vec<String> {
    let pattern = format!(r#"\b{key}\s*=\s*(?:"([^"]+)"|'([^']+)')"#);
    let Ok(regex) = regex::Regex::new(&pattern) else {
        return Vec::new();
    };
    regex
        .captures_iter(text)
        .filter_map(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string())
        })
        .collect()
}
