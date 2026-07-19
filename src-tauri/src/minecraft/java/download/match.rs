//! Java 版本与 Mojang component 匹配
//!
//! Mojang 的 all.json 按 component 名分类（如 java-runtime-gamma）。
//! version.name 格式通常为 "{major}.{minor}.{patch}"，可用于匹配。
//! 参考 PCL2 ModJava 的匹配策略：先精确 key，再模糊 version.name，最后按约定回退。

use serde_json::Value;

use super::types::JavaRuntimeEntry;

/// 根据 Java 大版本号匹配 Mojang component
///
/// 返回 `(component_key, JavaRuntimeEntry)`；未匹配返回 `None`。
pub fn match_component(
    all_json: &Value,
    target_major: u32,
) -> Option<(String, JavaRuntimeEntry)> {
    let platform = if cfg!(target_arch = "aarch64") {
        "windows-arm64"
    } else {
        "windows-x64"
    };

    let platform_node = all_json.get(platform)?;
    let components = platform_node.as_object()?;

    let target_str = target_major.to_string();

    // 1. 精确匹配 component key（如 "21"、"17"、"8"）
    if let Some(arr) = components.get(&target_str).and_then(|v| v.as_array()) {
        if let Some(first) = arr.first() {
            if let Ok(entry) = serde_json::from_value::<JavaRuntimeEntry>(first.clone()) {
                return Some((target_str.clone(), entry));
            }
        }
    }

    // 2. 模糊匹配 version.name 以 target_major 开头
    for (key, arr) in components {
        if let Some(arr) = arr.as_array() {
            for item in arr {
                if let Ok(entry) = serde_json::from_value::<JavaRuntimeEntry>(item.clone()) {
                    if entry.version.name.starts_with(&format!("{}.", target_str))
                        || entry.version.name == target_str
                    {
                        return Some((key.clone(), entry));
                    }
                }
            }
        }
    }

    // 3. 回退：按 component 名约定匹配
    let fallback_key = match target_major {
        21 => "java-runtime-gamma",
        17 => "java-runtime-alpha",
        8 => "java-runtime-legacy",
        _ => return None,
    };
    if let Some(arr) = components.get(fallback_key).and_then(|v| v.as_array()) {
        if let Some(first) = arr.first() {
            if let Ok(entry) = serde_json::from_value::<JavaRuntimeEntry>(first.clone()) {
                return Some((fallback_key.to_string(), entry));
            }
        }
    }

    None
}
