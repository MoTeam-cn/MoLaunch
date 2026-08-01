//! 嵌入资源释放与库检测
//!
//! - `resolve_embedded_jar`: 从缓存目录释放指定 jar，返回路径
//! - `has_library`: 检查版本 JSON 的 libraries 中是否包含指定库名

/// 解析嵌入资源 jar 的缓存路径
///
/// 首次使用从嵌入资源释放。
pub(super) fn resolve_embedded_jar(
    resource_name: &str,
    cache_rel: &str,
) -> Option<std::path::PathBuf> {
    use crate::utils::cache;
    if !cache::exists(cache_rel) {
        if let Err(e) = crate::resources::extract_resource(resource_name, &cache::path(cache_rel)) {
            crate::log_warn!("[Launch] 释放 {} 失败: {}", resource_name, e);
            return None;
        }
    }
    Some(cache::path(cache_rel))
}

/// 检查版本 JSON 的 libraries 中是否包含指定库名（如 "org.lwjgl:lwjgl:3.4.1"）
pub(super) fn has_library(json: &serde_json::Value, lib_name: &str) -> bool {
    if let Some(libraries) = json["libraries"].as_array() {
        for lib in libraries {
            if let Some(name) = lib["name"].as_str() {
                if name == lib_name {
                    // 还需通过 rules 校验（平台适配）
                    let rules: Option<Vec<serde_json::Value>> =
                        lib.get("rules").and_then(|v| v.as_array()).cloned();
                    if crate::minecraft::version::libraries::check_rules(&rules) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
