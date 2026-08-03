//! 本地库路径格式化
//!
//! 库本地路径拼接（含路径穿越防护）、artifact 解析与 natives 命名后缀。

use std::path::Path;

/// 拼接本地库路径（含路径穿越防护）
///
/// `artifact_path` 为库 JSON 中的相对 path，返回 `<game_dir>/libraries/<path>`。
/// 路径含 `..` 视为不安全，返回 None（调用方应跳过该条目）。
pub(super) fn artifact_local_path(artifact_path: &str, game_dir: &Path) -> Option<String> {
    if artifact_path.contains("..") {
        crate::log_warn!(
            "[Libraries] Skip path traversal in artifact path: {}",
            artifact_path
        );
        return None;
    }
    Some(
        game_dir
            .join("libraries")
            .join(artifact_path.replace('/', std::path::MAIN_SEPARATOR_STR))
            .to_string_lossy()
            .to_string(),
    )
}

/// natives 本地 maven 路径（在标准路径上追加 `-natives-{arch}` 后缀）
pub(super) fn maven_natives_path(name: &str, game_dir: &Path, arch: &str) -> String {
    super::super::maven_to_path(name, game_dir).replace(".jar", &format!("-natives-{}.jar", arch))
}

/// 从 `downloads.artifact`（或根级字段）解析 (url, local_path, size, sha1)
///
/// 优先读取 `downloads.artifact`；缺失时回退到根级别 size/sha1（Fabric 格式）。
/// 路径含 `..` 时返回 None，调用方应跳过该条目。
pub(super) fn resolve_artifact(
    library: &serde_json::Value,
    name: &str,
    game_dir: &Path,
    root_url: Option<&str>,
) -> Option<(Option<String>, String, i64, Option<String>)> {
    if let Some(artifact) = library.get("downloads").and_then(|d| d.get("artifact")) {
        let url = artifact["url"].as_str().or(root_url).map(|s| s.to_string());
        let path = if let Some(p) = artifact["path"].as_str() {
            artifact_local_path(p, game_dir)?
        } else {
            super::super::maven_to_path(name, game_dir)
        };
        let size = artifact["size"].as_i64().unwrap_or(0);
        let sha1 = artifact["sha1"].as_str().map(|s| s.to_string());
        Some((url, path, size, sha1))
    } else {
        // 没有 downloads.artifact：从根级别读取 size/sha1（Fabric 格式）
        let size = library["size"].as_i64().unwrap_or(0);
        let sha1 = library["sha1"].as_str().map(|s| s.to_string());
        Some((
            root_url.map(|s| s.to_string()),
            super::super::maven_to_path(name, game_dir),
            size,
            sha1,
        ))
    }
}
