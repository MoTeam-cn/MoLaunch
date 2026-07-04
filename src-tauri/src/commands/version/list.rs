use crate::{log_error, log_info};
use crate::minecraft::download;
use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;
use tauri::State;

use super::types::{VersionInfo, VersionListResult};

/// Get version list
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionListResult, String> {
    log_info!("Fetching version list");

    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    let source_mode = DownloadSourceMode::from_str(&config.download_source);
    drop(config);

    let result = download::fetch_version_list(mirror_url.as_deref(), source_mode).await.map_err(|e| {
        log_error!("Failed to list versions: {}", e);
        e.to_string()
    })?;

    let (latest_release, latest_snapshot) = download::get_latest_versions(&result.value);
    let entries = download::parse_version_list(&result.value);

    let versions: Vec<VersionInfo> = entries.iter().map(|e| {
        // 将时间字符串转换为Unix时间戳
        let release_time = parse_timestamp(&e.release_time);
        VersionInfo {
            id: e.id.clone(),
            version_type: e.version_type.clone(),
            release_time,
            url: e.url.clone(),
        }
    }).collect();

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
    // 尝试解析ISO 8601格式
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(time_str) {
        #[allow(deprecated)]
        return dt.timestamp();
    }
    // 尝试解析其他格式
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(time_str, "%Y-%m-%dT%H:%M:%S") {
        #[allow(deprecated)]
        return dt.timestamp();
    }
    0
}
