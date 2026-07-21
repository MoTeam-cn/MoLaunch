//! Maven 坐标转路径工具
//!
//! 统一 Maven 坐标（如 `net.minecraftforge:forge:1.20.1-47.2.0`）到本地文件路径的转换。

use std::path::{Path, PathBuf};

/// 将 Maven 坐标转为相对路径字符串
///
/// 如 `group:artifact:version[:classifier]` → `group/artifact/version/artifact-version[-classifier].jar`
pub fn maven_to_relative_path(name: &str) -> String {
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 3 {
        return name.to_string();
    }
    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    let version = parts[2];
    let classifier = if parts.len() > 3 { parts[3] } else { "" };
    if classifier.is_empty() {
        format!(
            "{}/{}/{}/{}-{}.jar",
            group, artifact, version, artifact, version
        )
    } else {
        format!(
            "{}/{}/{}/{}-{}-{}.jar",
            group, artifact, version, artifact, version, classifier
        )
    }
}

/// 将 Maven 坐标转为绝对路径 PathBuf
///
/// 基于 `maven_to_relative_path` 拼接 `game_dir/libraries/` 前缀。
pub fn maven_to_local_path(name: &str, game_dir: &Path) -> PathBuf {
    game_dir
        .join("libraries")
        .join(maven_to_relative_path(name))
}
