//! 加载器 JSON 合并（重装产物合并回当前版本 JSON）

use std::path::Path;

use crate::log_info;
use crate::minecraft::version::json_merge::merge_version_json;

/// 将新生成的加载器 JSON 合并进当前版本 JSON
///
/// - minecraftArguments：token 去重合并
/// - arguments：jvm/game 数组追加去重
/// - libraries：同名库以加载器为准，其余保留
/// - 其余字段：加载器覆盖（保留当前版本 id，去除继承）
pub(crate) fn merge_loader_json_into(
    game_dir: &Path,
    version_id: &str,
    existing: &serde_json::Value,
    fresh_dir: &Path,
) -> Result<(), String> {
    let fresh_dir_name = fresh_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "无法解析加载器版本目录名".to_string())?;
    let fresh_json_path = fresh_dir.join(format!("{}.json", fresh_dir_name));
    let fresh_content = std::fs::read_to_string(&fresh_json_path)
        .map_err(|e| format!("读取加载器 JSON 失败: {}", e))?;
    let fresh_json: serde_json::Value =
        serde_json::from_str(&fresh_content).map_err(|e| format!("解析加载器 JSON 失败: {}", e))?;

    // 尝试解析继承链（原版目录存在时），失败则保留加载器 JSON 原样
    let fresh_merged =
        merge_version_json(&fresh_json, game_dir).unwrap_or_else(|_| fresh_json.clone());

    let mut target = existing.clone();

    merge_minecraft_args(&mut target, &fresh_merged);
    merge_fields(&mut target, &fresh_merged);
    merge_argument_arrays(&mut target, &fresh_merged);
    merge_libraries_dedup(&mut target, &fresh_merged);

    target["id"] = serde_json::Value::String(version_id.to_string());
    if let Some(obj) = target.as_object_mut() {
        obj.remove("inheritsFrom");
    }

    let json_path = game_dir
        .join("versions")
        .join(version_id)
        .join(format!("{}.json", version_id));
    let new_content = serde_json::to_string_pretty(&target)
        .map_err(|e| format!("序列化版本 JSON 失败: {}", e))?;
    std::fs::write(&json_path, new_content).map_err(|e| format!("写入版本 JSON 失败: {}", e))?;

    log_info!("[RepairLoader] 已合并加载器 JSON: {}", version_id);
    Ok(())
}

/// minecraftArguments：按空格 token 去重合并
pub(crate) fn merge_minecraft_args(target: &mut serde_json::Value, fresh: &serde_json::Value) {
    let base = target["minecraftArguments"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let Some(other) = fresh["minecraftArguments"].as_str() else {
        return;
    };
    if other.is_empty() {
        return;
    }
    let mut merged: Vec<&str> = base.split(' ').collect();
    for arg in other.split(' ') {
        if !merged.contains(&arg) {
            merged.push(arg);
        }
    }
    target["minecraftArguments"] = serde_json::Value::String(merged.join(" "));
}

/// 其余字段递归合并（source 覆盖 target），跳过单独处理的键
pub(crate) fn merge_fields(target: &mut serde_json::Value, source: &serde_json::Value) {
    let (Some(target_map), Some(source_map)) = (target.as_object_mut(), source.as_object()) else {
        return;
    };
    for (key, value) in source_map {
        match key.as_str() {
            "libraries" | "arguments" | "minecraftArguments" | "id" | "inheritsFrom" => continue,
            _ => {}
        }
        if let Some(target_value) = target_map.get_mut(key) {
            if target_value.is_object() && value.is_object() {
                merge_fields(target_value, value);
            } else {
                target_value.clone_from(value);
            }
        } else {
            target_map.insert(key.clone(), value.clone());
        }
    }
}

/// arguments：jvm/game 数组追加去重，避免覆盖原版参数
pub(crate) fn merge_argument_arrays(target: &mut serde_json::Value, fresh: &serde_json::Value) {
    let Some(fresh_args) = fresh["arguments"].as_object() else {
        return;
    };
    if !target["arguments"].is_object() {
        target["arguments"] = fresh["arguments"].clone();
        return;
    }
    let target_args = target["arguments"].as_object_mut().unwrap();
    for (key, fresh_val) in fresh_args {
        if let Some(fresh_arr) = fresh_val.as_array() {
            if let Some(target_arr) = target_args.get_mut(key).and_then(|v| v.as_array_mut()) {
                for item in fresh_arr {
                    if !target_arr.contains(item) {
                        target_arr.push(item.clone());
                    }
                }
            } else {
                target_args.insert(key.clone(), fresh_val.clone());
            }
        } else if !target_args.contains_key(key) {
            target_args.insert(key.clone(), fresh_val.clone());
        }
    }
}

/// libraries：同名库以 fresh 为准，其余保留
pub(crate) fn merge_libraries_dedup(target: &mut serde_json::Value, fresh: &serde_json::Value) {
    let Some(fresh_libs) = fresh["libraries"].as_array() else {
        return;
    };
    if !target["libraries"].is_array() {
        target["libraries"] = serde_json::Value::Array(fresh_libs.clone());
        return;
    }
    let target_libs = target["libraries"].as_array_mut().unwrap();
    for lib in fresh_libs {
        let name = lib["name"].as_str().unwrap_or_default();
        if let Some(existing) = target_libs
            .iter_mut()
            .find(|l| l["name"].as_str() == Some(name))
        {
            *existing = lib.clone();
        } else {
            target_libs.push(lib.clone());
        }
    }
}
