//! 加载器探测：fabric.mod.json / mods.toml / neoforge.mods.toml 元数据提取

use std::path::Path;

use super::super::types::Loader;

/// 探测顺序：fabric → neoforge → forge
const PROBES: &[(&str, Loader)] = &[
    ("fabric.mod.json", Loader::Fabric),
    ("META-INF/neoforge.mods.toml", Loader::NeoForge),
    ("META-INF/mods.toml", Loader::Forge),
];

/// TOML 元数据文件路径
fn toml_path(loader: Loader) -> Option<&'static str> {
    match loader {
        Loader::NeoForge => Some("META-INF/neoforge.mods.toml"),
        Loader::Forge => Some("META-INF/mods.toml"),
        _ => None,
    }
}

/// 探测加载器：返回 (加载器, modId 列表, 项目名列表)
pub fn detect_loader(workspace: &Path) -> (Loader, Vec<String>, Vec<String>) {
    for (relative, loader) in PROBES {
        let path = workspace.join(relative);
        if !path.is_file() {
            continue;
        }
        let mod_ids = match loader {
            Loader::Fabric => read_fabric_mod_id(&path).into_iter().collect(),
            _ => read_toml_values(&path, "modId"),
        };
        return (*loader, mod_ids, Vec::new());
    }
    (Loader::Unknown, Vec::new(), Vec::new())
}

fn read_fabric_mod_id(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    value
        .get("id")
        .and_then(|field| field.as_str())
        .map(str::to_string)
}

/// 逐行扫描 TOML 键值：`key = "value"` / `key = 'value'`
fn read_toml_values(path: &Path, key: &str) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let mut values = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let value = rest.trim().trim_matches(['"', '\'']).trim();
        if !value.is_empty() {
            values.push(value.to_string());
        }
    }
    values
}

/// 探测元数据：加载器 + modId + 项目名 + 版本
pub fn detect_metadata(workspace: &Path) -> (Loader, Vec<String>, Vec<String>, Option<String>) {
    let (loader, mod_ids, _) = detect_loader(workspace);

    if loader == Loader::Fabric {
        let Ok(content) = std::fs::read_to_string(workspace.join("fabric.mod.json")) else {
            return (loader, mod_ids, Vec::new(), None);
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            return (loader, mod_ids, Vec::new(), None);
        };
        let project_names = value
            .get("name")
            .and_then(|field| field.as_str())
            .map(str::to_string)
            .into_iter()
            .collect();
        let version = value
            .get("version")
            .and_then(|field| field.as_str())
            .map(str::to_string);
        return (loader, mod_ids, project_names, version);
    }

    let Some(relative) = toml_path(loader) else {
        return (loader, mod_ids, Vec::new(), None);
    };
    let path = workspace.join(relative);
    let project_names = read_toml_values(&path, "displayName");
    let version = read_toml_values(&path, "version").into_iter().next();
    (loader, mod_ids, project_names, version)
}
