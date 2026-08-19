//! easytier 内核外部下载安装（放弃内置，按需从 GitHub 下载）
//! 版本查询 / 状态查询 / IPC 注册；安装实现见 `easytier_download`（镜像优先 + 官方保底，走 DownloadManager 分片下载）。

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;

use serde::Serialize;
use tauri::Emitter;

use crate::handler;
use crate::log_warn;
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;
use crate::utils::github_download::GithubProxy;

/// easytier GitHub 仓库
pub(super) const EASYTIER_REPO: &str = "EasyTier/EasyTier";
/// GitHub API 主源
const GITHUB_API_PRIMARY: &str = "https://api.github.com";
/// GitHub API 备选源（仅 API 功能）
const GITHUB_API_FALLBACK: &str = "https://github-api.mocdn.net";
/// 安装进度事件名
const EASYTIER_INSTALL_PROGRESS_EVENT: &str = "easytier-install-progress";
/// 版本标记文件名
pub(super) const VERSION_FILE: &str = "version.txt";

/// `easytier_install_status` 返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EasyTierInstallStatus {
    pub installed: bool,
    pub version: String,
    pub latest_version: String,
    pub downloading: bool,
}

/// 安装目录（`<appdata>/.Molaunch/easytier/`）
pub(super) fn install_dir() -> Result<PathBuf, String> {
    crate::storage::appdata::ensure_appdata_subdir("easytier")
}

/// 当前平台 core 文件名
pub(super) fn core_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "easytier-core.exe"
    } else {
        "easytier-core"
    }
}

/// 当前平台 cli 文件名（Unix 补执行权限 / 白名单解压时使用）
pub(super) fn cli_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "easytier-cli.exe"
    } else {
        "easytier-cli"
    }
}

/// 查询最新版本号（主源失败回退备选；失败返回错误由前端提示）
pub async fn fetch_latest_release() -> Result<String, String> {
    let client = crate::http::get_client();
    let primary = format!("{GITHUB_API_PRIMARY}/repos/{EASYTIER_REPO}/releases/latest");
    match fetch_tag_name(&client, &primary).await {
        Ok(tag) => Ok(tag),
        Err(e) => {
            log_warn!("[EasyTier] GitHub API 主源失败: {e}，回退备选源");
            let fallback = format!("{GITHUB_API_FALLBACK}/repos/{EASYTIER_REPO}/releases/latest");
            fetch_tag_name(&client, &fallback).await
        }
    }
}

/// 请求 release API 解析 tag_name（去 v 前缀，单请求 30s 超时）
async fn fetch_tag_name(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let value: serde_json::Value = resp.json().await.map_err(|e| format!("解析失败: {e}"))?;
    let tag = value
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "响应缺少 tag_name".to_string())?;
    Ok(tag.trim_start_matches('v').to_string())
}

/// 当前平台 release 资产名（`easytier-{os}-{arch}-v{version}.zip`）
pub(super) fn asset_name(version: &str) -> String {
    let (os, arch) = if cfg!(target_os = "windows") {
        (
            "windows",
            if cfg!(target_arch = "x86_64") {
                "x86_64"
            } else {
                "aarch64"
            },
        )
    } else if cfg!(target_os = "macos") {
        (
            "macos",
            if cfg!(target_arch = "x86_64") {
                "x86_64"
            } else {
                "aarch64"
            },
        )
    } else {
        (
            "linux",
            if cfg!(target_arch = "x86_64") {
                "x86_64"
            } else {
                "aarch64"
            },
        )
    };
    format!("easytier-{os}-{arch}-v{version}.zip")
}

/// 已安装版本（读 version.txt）
pub fn installed_version() -> Option<String> {
    let dir = install_dir().ok()?;
    std::fs::read_to_string(dir.join(VERSION_FILE))
        .ok()
        .map(|s| s.trim().to_string())
}

/// 是否已安装（core 文件存在）
pub fn is_installed() -> bool {
    install_dir()
        .map(|d| d.join(core_name()).is_file())
        .unwrap_or(false)
}

/// 推送安装进度事件
pub(super) fn emit_progress(
    app: &tauri::AppHandle,
    phase: &'static str,
    percent: u8,
    message: &str,
) {
    let _ = app.emit(
        EASYTIER_INSTALL_PROGRESS_EVENT,
        serde_json::json!({ "phase": phase, "percent": percent, "message": message }),
    );
}

/// 确保已安装：已安装直接返回 core 路径；未安装自动下载安装
pub async fn ensure_installed(state: &AppState, app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if is_installed() {
        return Ok(install_dir()?.join(core_name()));
    }
    if state.easytier_installing.swap(true, Ordering::SeqCst) {
        return Err("easytier 正在安装中，请稍候".to_string());
    }
    let result = super::easytier_download::install_latest(state, app).await;
    state.easytier_installing.store(false, Ordering::SeqCst);
    result?;
    Ok(install_dir()?.join(core_name()))
}

/// 注册 easytier 安装动作到 dispatcher
pub fn register(d: &mut Dispatcher) {
    d.register(
        "easytier_install_status",
        handler!(state, _app, _params, {
            let downloading = state.easytier_installing.load(Ordering::SeqCst);
            let (installed, version) = (is_installed(), installed_version().unwrap_or_default());
            // 已安装时查询最新版本（提示更新）；未安装无需查询
            let latest_version = if installed {
                fetch_latest_release().await.unwrap_or_default()
            } else {
                String::new()
            };
            serde_json::to_value(EasyTierInstallStatus {
                installed,
                version,
                latest_version,
                downloading,
            })
            .map_err(|e| e.to_string())
        }),
    );

    // 下载安装最新版（`easytier_install` 与 `easytier_update` 语义相同，共用实现）
    for action in ["easytier_install", "easytier_update"] {
        d.register(
            action,
            handler!(state, app, _params, {
                if state.easytier_installing.swap(true, Ordering::SeqCst) {
                    return Err("easytier 正在安装中，请稍候".to_string());
                }
                let result = super::easytier_download::install_latest(&state, &app).await;
                state.easytier_installing.store(false, Ordering::SeqCst);
                if let Err(e) = &result {
                    emit_progress(&app, "error", 0, e);
                }
                result.map(|_| serde_json::json!({ "success": true }))
            }),
        );
    }

    // 取消安装（设置取消标志，下载链实时检查后中断；解压/移动阶段在阶段间检查）
    d.register(
        "easytier_cancel",
        handler!(state, _app, _params, {
            state.easytier_cancel.store(true, Ordering::SeqCst);
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );

    d.register(
        "set_github_proxies",
        handler!(state, _app, params, {
            let proxies: Vec<GithubProxy> =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            *state.github_proxies.lock().await = proxies.clone();
            // 持久化到配置（重启不丢失）
            crate::commands::system::update_config(&state, |config| {
                config.online.github_proxies = proxies;
            })
            .await?;
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );
}
