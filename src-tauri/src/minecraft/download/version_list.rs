//! 版本清单相关：获取、解析版本列表与版本 JSON URL

use serde::{Deserialize, Serialize};

use super::super::sources::{self, DownloadSourceMode};

/// 版本列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionListResult {
    pub source_name: String,
    pub is_official: bool,
    pub value: serde_json::Value,
}

/// 版本条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    pub id: String,
    pub version_type: String,
    pub time: String,
    pub release_time: String,
    pub url: String,
}

/// 获取版本列表
pub async fn fetch_version_list(
    mirror_url: Option<&str>,
    source_mode: DownloadSourceMode,
) -> anyhow::Result<VersionListResult> {
    let urls = sources::build_urls(
        mirror_url,
        &format!(
            "{}/mc/game/version_manifest.json",
            sources::MOJANG_LAUNCHERMETA
        ),
        sources::BMCLAPI_VERSION_MANIFEST,
        source_mode,
    );

    let content = sources::fetch_with_fallback(&urls).await?;
    let json: serde_json::Value = serde_json::from_str(&content)?;

    let source_name = if urls
        .first()
        .map_or(false, |u: &String| u.contains("bmclapi"))
    {
        "BMCLAPI"
    } else if mirror_url.is_some()
        && urls
            .first()
            .map_or(false, |u: &String| u.starts_with(mirror_url.unwrap_or("")))
    {
        "Mirror"
    } else {
        "Mojang"
    };

    Ok(VersionListResult {
        source_name: source_name.to_string(),
        is_official: source_name == "Mojang",
        value: json,
    })
}

/// 获取版本 JSON URL
pub fn get_version_json_url(version_list: &serde_json::Value, version_id: &str) -> Option<String> {
    if let Some(versions) = version_list["versions"].as_array() {
        for version in versions {
            if let Some(id) = version["id"].as_str() {
                if id == version_id {
                    return version["url"].as_str().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

/// 解析版本列表
pub fn parse_version_list(version_list: &serde_json::Value) -> Vec<VersionEntry> {
    let mut entries = Vec::new();
    if let Some(versions) = version_list["versions"].as_array() {
        for version in versions {
            if let (Some(id), Some(version_type), Some(time), Some(release_time), Some(url)) = (
                version["id"].as_str(),
                version["type"].as_str(),
                version["time"].as_str(),
                version["releaseTime"].as_str(),
                version["url"].as_str(),
            ) {
                // 检测愚人节版本，修正 type
                let actual_type =
                    if super::super::fools::detect_fool(id, version_type, release_time).is_some() {
                        "fool"
                    } else {
                        version_type
                    };

                entries.push(VersionEntry {
                    id: id.to_string(),
                    version_type: actual_type.to_string(),
                    time: time.to_string(),
                    release_time: release_time.to_string(),
                    url: url.to_string(),
                });
            }
        }
    }
    entries
}

/// 获取最新版本
pub fn get_latest_versions(version_list: &serde_json::Value) -> (Option<String>, Option<String>) {
    let latest_release = version_list["latest"]["release"]
        .as_str()
        .map(|s| s.to_string());
    let latest_snapshot = version_list["latest"]["snapshot"]
        .as_str()
        .map(|s| s.to_string());
    (latest_release, latest_snapshot)
}
