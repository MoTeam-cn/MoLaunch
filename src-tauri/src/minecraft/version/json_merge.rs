//! JSON inheritance merge module
//! 参考 PCL2 的 JsonObject 属性实现

use crate::log_warn;
use std::path::Path;

/// Merge version JSON inheritance chain
/// 参考 PCL2 的处理方式：如果父版本不存在，仅记录警告，返回当前JSON
pub fn merge_version_json(
    json: &serde_json::Value,
    game_dir: &Path,
) -> anyhow::Result<serde_json::Value> {
    let mut current = json.clone();

    loop {
        let inherit_from = current
            .get("inheritsFrom")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if inherit_from.is_empty() {
            break;
        }

        let current_id = current.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if inherit_from == current_id {
            log_warn!("[JsonMerge] Self-referencing inherit: {}", inherit_from);
            break;
        }

        let parent_json_path = game_dir
            .join("versions")
            .join(inherit_from)
            .join(format!("{}.json", inherit_from));

        // 参考 PCL2：父版本不存在时仅警告，不报错
        if !parent_json_path.exists() {
            log_warn!(
                "[JsonMerge] Parent JSON not found: {} (inheritsFrom: {})",
                parent_json_path.display(),
                inherit_from
            );
            log_warn!("[JsonMerge] Continuing without parent merge, some features may not work");
            // 注意：不移除 inheritsFrom，让 get_asset_index_meta 的fallback能工作
            break;
        }

        let parent_content = std::fs::read_to_string(&parent_json_path)?;
        let mut parent_json: serde_json::Value = serde_json::from_str(&parent_content)?;

        // Recursively merge parent's inheritance
        parent_json = merge_version_json(&parent_json, game_dir)?;

        // Merge Libraries: child first, parent after (参考 PCL2 第576-584行)
        let mut merged_libs = serde_json::Value::Array(Vec::new());

        if let Some(child_libs) = current.get("libraries").and_then(|l| l.as_array()) {
            for lib in child_libs {
                if let serde_json::Value::Array(ref mut arr) = merged_libs {
                    arr.push(lib.clone());
                }
            }
        }

        if let Some(parent_libs) = parent_json.get("libraries").and_then(|l| l.as_array()) {
            for lib in parent_libs {
                if let serde_json::Value::Array(ref mut arr) = merged_libs {
                    arr.push(lib.clone());
                }
            }
        }

        // Merge other fields: child overrides parent (参考 PCL2 第582-583行)
        let mut merged = parent_json.clone();
        merge_json_values(&mut merged, &current);
        merged["libraries"] = merged_libs;

        current = merged;

        if let Some(obj) = current.as_object_mut() {
            obj.remove("inheritsFrom");
        }
    }

    Ok(current)
}

/// Merge two JSON values (source overrides target)
fn merge_json_values(target: &mut serde_json::Value, source: &serde_json::Value) {
    if let (serde_json::Value::Object(target_map), serde_json::Value::Object(source_map)) =
        (target, source)
    {
        for (key, value) in source_map {
            if key == "libraries" {
                continue;
            }
            if let Some(target_value) = target_map.get_mut(key) {
                if target_value.is_object() && value.is_object() {
                    merge_json_values(target_value, value);
                } else {
                    target_value.clone_from(value);
                }
            } else {
                target_map.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Merge multiple version JSONs (for merged install MC + Forge + etc)
pub fn merge_multiple_jsons(
    base: &serde_json::Value,
    others: &[serde_json::Value],
) -> serde_json::Value {
    let mut result = base.clone();

    for other in others {
        let mut other_clean = other.clone();
        if let Some(obj) = other_clean.as_object_mut() {
            obj.remove("releaseTime");
            obj.remove("time");
        }

        if let (Some(base_args), Some(other_args)) = (
            result.get("minecraftArguments").and_then(|a| a.as_str()),
            other_clean
                .get("minecraftArguments")
                .and_then(|a| a.as_str()),
        ) {
            let merged_args = merge_minecraft_arguments(base_args, other_args);
            result["minecraftArguments"] = serde_json::Value::String(merged_args);
        }

        merge_json_values(&mut result, &other_clean);
    }

    result
}

/// Merge minecraftArguments (deduplicate)
fn merge_minecraft_arguments(base: &str, other: &str) -> String {
    let base_args: Vec<&str> = base.split(' ').collect();
    let other_args: Vec<&str> = other.split(' ').collect();

    let mut merged: Vec<&str> = base_args.clone();

    for arg in other_args {
        if !merged.contains(&arg) {
            merged.push(arg);
        }
    }

    merged.join(" ")
}
