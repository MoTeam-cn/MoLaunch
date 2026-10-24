//! GitHub 备用更新清单（Release 里的静态 latest.json）
//!
//! 该源不需要设备凭证：清单里自带通道、平台下载地址、签名与强制更新标记，
//! 由发版流程在 Release 资产中维护。灰度能力仅在云端通道提供。

use serde_json::Value;

use super::UpdateInfo;
use crate::api_paths::{GITHUB_MANIFEST_CHANNEL, GITHUB_MANIFEST_STABLE};
use crate::log_info;
use crate::log_warn;

/// GitHub 清单请求超时（GitHub 首包较慢，比云端通道多给一些）
const GITHUB_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(8);

/// 通道 → 清单地址
///
/// `stable` 用「最新正式 Release」的固定路径；`beta` / `alpha` 用滚动 tag
/// （channel-beta / channel-alpha）下的同名资产，发版时覆盖上传。
pub(super) fn manifest_url(channel: &str) -> String {
    if channel == "stable" {
        GITHUB_MANIFEST_STABLE.to_string()
    } else {
        GITHUB_MANIFEST_CHANNEL.replace("{{channel}}", channel)
    }
}

/// 系统标识 → 清单里的 platforms 键（与 tauri 平台命名保持一致）
pub(super) fn platform_key(os: &str, arch: &str) -> String {
    let arch = match arch {
        "aarch64" | "arm64" => "aarch64",
        _ => "x86_64",
    };
    match os {
        "macos" => format!("darwin-{}", arch),
        "windows" => format!("windows-{}", arch),
        _ => format!("linux-{}", arch),
    }
}

/// 从清单中挑出「版本号更高」的通道条目，返回 (条目, 实际通道)
///
/// 候选来自通道层级（beta → beta + stable，alpha → 三档全部），所以正式版发布后
/// 测试版用户也能拿到正式版；版本号相同（或无法比较）时优先请求通道。
pub(super) fn pick_best_channel_entry<'a>(
    manifest: &'a Value,
    channel: &str,
) -> Option<(&'a Value, &'static str)> {
    let channels = manifest.get("channels")?;
    let mut best: Option<(&'a Value, &'static str)> = None;
    for &name in super::check::channel_tiers(channel) {
        let Some(entry) = channels.get(name).filter(|v| v.is_object()) else {
            continue;
        };
        let version = entry.get("version").and_then(|v| v.as_str()).unwrap_or("");
        match best {
            None => best = Some((entry, name)),
            Some((current, _)) => {
                let current_version = current
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if super::check::is_version_newer(version, current_version) {
                    best = Some((entry, name));
                }
            }
        }
    }
    best
}

/// 从通道条目里取指定平台的下载信息（地址 + 可选签名）
pub(super) fn pick_platform<'a>(entry: &'a Value, key: &str) -> Option<(&'a str, &'a str)> {
    let platform = entry.get("platforms")?.get(key)?;
    let url = platform.get("url").and_then(|v| v.as_str())?;
    if url.is_empty() {
        return None;
    }
    let signature = platform
        .get("signature")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Some((url, signature))
}

/// 通过 GitHub 静态清单检查更新
pub(super) async fn check_github(
    channel: &str,
    current_version: &str,
) -> Result<UpdateInfo, String> {
    let url = manifest_url(channel);
    log_info!("[Updater] 检查更新（GitHub 清单）: {}", url);

    let client = crate::http::get_client();
    let response = client
        .get(&url)
        .timeout(GITHUB_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                format!(
                    "检查更新失败：GitHub 清单请求超时（{} 秒）",
                    GITHUB_TIMEOUT.as_secs()
                )
            } else {
                format!("检查更新失败：GitHub 清单网络错误: {}", e)
            }
        })?;

    if !response.status().is_success() {
        return Err(format!(
            "检查更新失败: GitHub 清单 HTTP {}",
            response.status()
        ));
    }

    let manifest: Value = response
        .json()
        .await
        .map_err(|e| format!("检查更新失败：解析 GitHub 清单失败: {}", e))?;

    let Some((entry, served_channel)) = pick_best_channel_entry(&manifest, channel) else {
        log_warn!("[Updater] GitHub 清单缺少通道 {} 且无 stable 回退", channel);
        return Ok(UpdateInfo::default());
    };
    if served_channel != channel {
        log_info!(
            "[Updater] GitHub 清单跨通道命中：请求 {} → 实际 {}",
            channel,
            served_channel
        );
    }

    let version = entry.get("version").and_then(|v| v.as_str()).unwrap_or("");
    if version.is_empty() {
        log_info!("[Updater] GitHub 清单无 version 字段（无可用更新）");
        return Ok(UpdateInfo::default());
    }
    if !super::check::is_version_newer(version, current_version) {
        log_info!(
            "[Updater] 当前版本 {} 已是最新（GitHub 清单）",
            current_version
        );
        return Ok(UpdateInfo::default());
    }

    let key = platform_key(std::env::consts::OS, std::env::consts::ARCH);
    let Some((download_url, signature)) = pick_platform(entry, &key) else {
        log_warn!("[Updater] GitHub 清单缺少平台 {} 的下载信息", key);
        return Ok(UpdateInfo::default());
    };
    crate::utils::net::validate_updater_download_url(download_url)?;

    log_info!(
        "[Updater] 发现新版本（GitHub 清单）: {} -> {}",
        current_version,
        version
    );
    Ok(UpdateInfo {
        available: true,
        version: version.to_string(),
        notes: entry
            .get("notes")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        force_update: entry
            .get("force_update")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        download_url: download_url.to_string(),
        signature: signature.to_string(),
        channel: served_channel.to_string(),
    })
}

#[cfg(test)]
#[path = "github_test.rs"]
mod github_test;
