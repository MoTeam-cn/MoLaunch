//! Classpath 构建
//!
//! 参考 PCL2 的 McLibListGet，递归处理继承版本与 libraries，拼装 Java classpath。

use crate::log_info;
use crate::minecraft::utils::maven::maven_to_relative_path;
use std::path::Path;

/// Build classpath
pub(super) fn build_classpath(game_dir: &Path, json: &serde_json::Value) -> anyhow::Result<String> {
    let mut entries = Vec::new();

    // 参考 PCL2 的 McLibListGet 函数
    // 递归查找最深层的继承版本来获取原版jar
    let jar_version = find_original_version(game_dir, json);
    let version_jar = game_dir
        .join("versions")
        .join(&jar_version)
        .join(format!("{}.jar", jar_version));

    if version_jar.exists() {
        entries.push(version_jar.to_string_lossy().to_string());
    } else {
        log_info!(
            "[Classpath] Warning: Main jar not found: {}",
            version_jar.display()
        );
    }

    if let Some(libraries) = json["libraries"].as_array() {
        for lib in libraries {
            // 应用 rules 过滤（平台适配）
            let rules: Option<Vec<serde_json::Value>> = lib
                .get("rules")
                .and_then(|v| v.as_array())
                .map(|a| a.clone());
            if !crate::minecraft::version::libraries::check_rules(&rules) {
                continue;
            }

            // 解析 maven name 判断是否有 classifier（如 "natives-windows"）
            // 有 classifier 的是 native 包，应通过 extract_natives 处理，不放入 classpath
            // 但对于"无 natives 字段但有 classifier"的新格式，需要特殊处理（见 extract_natives）
            let has_classifier = lib["name"]
                .as_str()
                .map(|n| n.split(':').count() > 3)
                .unwrap_or(false);

            // 优先用 downloads.artifact.path（更准确）
            if let Some(artifact) = lib.get("downloads").and_then(|d| d.get("artifact")) {
                if let Some(path) = artifact["path"].as_str() {
                    // 跳过 native 包（有 classifier 且 classifier 含 "natives"）
                    let is_native = has_classifier
                        && path.contains("natives-");
                    if is_native {
                        continue;
                    }
                    let lib_path = game_dir.join("libraries").join(path);
                    if lib_path.exists() {
                        entries.push(lib_path.to_string_lossy().to_string());
                    }
                }
            } else if let Some(name) = lib["name"].as_str() {
                // 没有 downloads.artifact，用 maven name 解析路径
                let path = maven_to_relative_path(name);
                let lib_path = game_dir.join("libraries").join(&path);
                if lib_path.exists() {
                    entries.push(lib_path.to_string_lossy().to_string());
                }
            }
        }
    }

    Ok(entries.join(if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    }))
}

/// 递归查找最深层的继承版本（参考 PCL2 的 McLibListGet）
fn find_original_version(game_dir: &Path, json: &serde_json::Value) -> String {
    // 检查是否有 jar 字段指定
    if let Some(jar) = json.get("jar").and_then(|v| v.as_str()) {
        return jar.to_string();
    }

    // 检查 inheritsFrom
    if let Some(inherits_from) = json.get("inheritsFrom").and_then(|v| v.as_str()) {
        if !inherits_from.is_empty() {
            // 加载父版本JSON
            let parent_json_path = game_dir
                .join("versions")
                .join(inherits_from)
                .join(format!("{}.json", inherits_from));
            if parent_json_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&parent_json_path) {
                    if let Ok(parent_json) = serde_json::from_str::<serde_json::Value>(&content) {
                        // 递归查找
                        return find_original_version(game_dir, &parent_json);
                    }
                }
            }
            // 如果父版本不存在，返回inheritsFrom作为版本名
            return inherits_from.to_string();
        }
    }

    // 没有继承，使用当前版本
    json.get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string()
}
