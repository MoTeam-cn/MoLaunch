//! 远程版本清单获取（list_versions + 时间戳解析）

use crate::minecraft::download;
use crate::minecraft::fools;
use crate::state::AppState;
use crate::{log_error, log_info};

use super::super::types::{VersionInfo, VersionListResult};

/// Get version list
pub async fn list_versions(state: &AppState) -> Result<VersionListResult, String> {
    log_info!("Fetching version list");

    let (mirror_url, source_mode) = crate::state::resolve_mirror_and_source(state).await;

    let result = download::fetch_version_list(mirror_url.as_deref(), source_mode)
        .await
        .map_err(|e| {
            log_error!("Failed to list versions: {}", e);
            e.to_string()
        })?;

    let (latest_release, latest_snapshot) = download::get_latest_versions(&result.value);
    let entries = download::parse_version_list(&result.value);

    let versions: Vec<VersionInfo> = entries
        .iter()
        .map(|e| {
            let release_time = parse_timestamp(&e.release_time);
            let description = if e.version_type == "fool" {
                fools::get_fool_description(&e.id)
            } else {
                None
            };
            VersionInfo {
                id: e.id.clone(),
                version_type: e.version_type.clone(),
                release_time,
                url: e.url.clone(),
                description,
            }
        })
        .collect();

    log_info!("Found {} versions", versions.len());
    Ok(VersionListResult {
        versions,
        latest_release: latest_release.unwrap_or_default(),
        latest_snapshot: latest_snapshot.unwrap_or_default(),
        source_name: result.source_name,
    })
}

/// 解析时间字符串为Unix时间戳
fn parse_timestamp(time_str: &str) -> i64 {
    // 使用统一的时间解析工具，支持 RFC3339 / naive datetime / 纯日期
    match crate::utils::datetime::parse_utc(time_str) {
        #[allow(deprecated)]
        Some(dt) => dt.timestamp(),
        None => 0,
    }
}
