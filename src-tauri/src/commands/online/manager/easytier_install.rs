//! easytier 内核外部下载安装（放弃内置，按需从 GitHub 下载）
//! 版本查询 / 下载 / 解压安装 / 状态查询；镜像竞速选源。

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::handler;
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

/// easytier GitHub 仓库
const EASYTIER_REPO: &str = "EasyTier/EasyTier";
/// GitHub API 主源
const GITHUB_API_PRIMARY: &str = "https://api.github.com";
/// GitHub API 备选源（仅 API 功能）
const GITHUB_API_FALLBACK: &str = "https://github-api.mocdn.net";
/// 官方下载源
const GITHUB_DOWNLOAD_BASE: &str = "https://github.com";
/// 安装进度事件名
const EASYTIER_INSTALL_PROGRESS_EVENT: &str = "easytier-install-progress";
/// 版本标记文件名
const VERSION_FILE: &str = "version.txt";

/// 前端传入的 GitHub 镜像源（type: path 追加路径 / type: full 追加完整 URL）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubProxy {
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub base: String,
}

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
fn install_dir() -> Result<PathBuf, String> {
    crate::storage::appdata::ensure_appdata_subdir("easytier")
}

/// 当前平台 core 文件名
fn core_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "easytier-core.exe"
    } else {
        "easytier-core"
    }
}

/// 当前平台 cli 文件名（仅 Unix 补执行权限时使用）
#[cfg(unix)]
fn cli_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "easytier-cli.exe"
    } else {
        "easytier-cli"
    }
}

/// 查询最新版本号（主源失败回退备选；失败返回错误由前端提示）
pub async fn fetch_latest_release(client: &reqwest::Client) -> Result<String, String> {
    let primary = format!("{GITHUB_API_PRIMARY}/repos/{EASYTIER_REPO}/releases/latest");
    match fetch_tag_name(client, &primary).await {
        Ok(tag) => Ok(tag),
        Err(e) => {
            log_warn!("[EasyTier] GitHub API 主源失败: {e}，回退备选源");
            let fallback = format!("{GITHUB_API_FALLBACK}/repos/{EASYTIER_REPO}/releases/latest");
            fetch_tag_name(client, &fallback).await
        }
    }
}

/// 请求 release API 解析 tag_name（去 v 前缀）
async fn fetch_tag_name(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let resp = client
        .get(url)
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
fn asset_name(version: &str) -> String {
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
fn emit_progress(app: &tauri::AppHandle, phase: &'static str, percent: u8, message: &str) {
    let _ = app.emit(
        EASYTIER_INSTALL_PROGRESS_EVENT,
        serde_json::json!({ "phase": phase, "percent": percent, "message": message }),
    );
}

/// 下载 zip 到目标路径（官方 URL 优先，失败走镜像竞速）
async fn download_zip(
    client: &reqwest::Client,
    version: &str,
    target: &Path,
    proxies: &[GithubProxy],
    on_progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
) -> Result<(), String> {
    let asset = asset_name(version);
    let official =
        format!("{GITHUB_DOWNLOAD_BASE}/{EASYTIER_REPO}/releases/download/v{version}/{asset}");
    if download_to(client, &official, target, on_progress)
        .await
        .is_ok()
    {
        return Ok(());
    }
    log_warn!("[EasyTier] 官方源下载失败，尝试镜像竞速");
    let mut candidates = Vec::with_capacity(proxies.len());
    for p in proxies {
        let base = p.base.trim_end_matches('/');
        let url = if p.proxy_type == "path" {
            format!("{base}/{EASYTIER_REPO}/releases/download/v{version}/{asset}")
        } else {
            format!("{base}{official}")
        };
        candidates.push(url);
    }
    let fastest = pick_fastest(client, &candidates).await?;
    download_to(client, &fastest, target, on_progress).await
}

/// 镜像竞速：并发 HEAD + Range 0-1 测速（单请求 10s 超时），取响应最快者
async fn pick_fastest(client: &reqwest::Client, candidates: &[String]) -> Result<String, String> {
    let mut handles = Vec::with_capacity(candidates.len());
    for url in candidates {
        let client = client.clone();
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            let start = std::time::Instant::now();
            let resp = client
                .get(&url)
                .header("Range", "bytes=0-1")
                .timeout(Duration::from_secs(10))
                .send()
                .await;
            match resp {
                Ok(r) if r.status().is_success() || r.status().as_u16() == 206 => {
                    Some((start.elapsed(), url))
                }
                _ => None,
            }
        }));
    }
    let mut best: Option<(Duration, String)> = None;
    for h in handles {
        if let Ok(Some((elapsed, url))) = h.await {
            if best.as_ref().map(|(t, _)| elapsed < *t).unwrap_or(true) {
                best = Some((elapsed, url));
            }
        }
    }
    best.map(|(_, url)| url)
        .ok_or_else(|| "所有镜像源均不可用".to_string())
}

/// 流式下载文件（reqwest bytes_stream，按字节回调进度：done/total）
async fn download_to(
    client: &reqwest::Client,
    url: &str,
    target: &Path,
    on_progress: &(dyn Fn(u64, Option<u64>) + Send + Sync),
) -> Result<(), String> {
    use futures_util::StreamExt;
    use std::io::Write;
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let total = resp.content_length();
    let mut out = std::fs::File::create(target).map_err(|e| format!("创建文件失败: {e}"))?;
    let mut stream = resp.bytes_stream();
    let mut done: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("读取响应失败: {e}"))?;
        done += chunk.len() as u64;
        on_progress(done, total);
        out.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {e}"))?;
    }
    Ok(())
}

/// 解压 zip 到目标目录（剥离共享顶层目录 + Zip Slip 防护）
fn extract_zip_safely(zip_path: &Path, dst: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开 ZIP 失败: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {e}"))?;
    // 确定共享顶层前缀（所有条目同一根目录时剥离，扁平包不剥离）
    let mut roots = std::collections::HashSet::new();
    let mut flat = false;
    for name in archive.file_names() {
        if name.contains('/') {
            if let Some(root) = name.split('/').next().filter(|v| !v.is_empty()) {
                roots.insert(root.to_string());
            }
        } else if !name.is_empty() {
            flat = true;
        }
    }
    let prefix = if flat || roots.len() != 1 {
        String::new()
    } else {
        format!("{}/", roots.into_iter().next().unwrap())
    };
    let canonical_dst = dst.canonicalize().unwrap_or_else(|_| dst.to_path_buf());
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {e}"))?;
        let name = entry.name().to_string();
        let rel = name.strip_prefix(&prefix).unwrap_or(&name);
        if rel.is_empty() {
            continue;
        }
        let path = dst.join(rel);
        if rel.ends_with('/') {
            std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {e}"))?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
            if !parent
                .canonicalize()
                .map_err(|e| format!("canonicalize 失败: {e}"))?
                .starts_with(&canonical_dst)
            {
                return Err(format!("Zip Slip 检测: {rel}"));
            }
        }
        let mut out = std::fs::File::create(&path).map_err(|e| format!("创建文件失败: {e}"))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("写入文件失败: {e}"))?;
    }
    Ok(())
}

/// 递归移动目录内容（临时目录 → 安装目录）
fn move_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("创建目录失败: {e}"))?;
    for entry in std::fs::read_dir(src)
        .map_err(|e| format!("读取目录失败: {e}"))?
        .flatten()
    {
        let p = entry.path();
        let target = dst.join(entry.file_name());
        if p.is_dir() {
            move_dir_contents(&p, &target)?;
        } else {
            std::fs::rename(&p, &target).map_err(|e| format!("移动文件失败: {e}"))?;
        }
    }
    Ok(())
}

/// 下载并安装指定版本（下载 → 解压 → version.txt → 执行权限）
async fn install_version(
    state: &AppState,
    app: &tauri::AppHandle,
    version: &str,
) -> Result<(), String> {
    // Windows 下无法覆盖运行中的 exe：正在组网时拒绝重装，提示先退出
    if state.easytier.lock().await.is_some() {
        return Err("easytier 正在组网运行中，请先退出联机网络再更新内核".to_string());
    }
    let client = crate::http::build_client_with_user_agent("MoLaunch", Some(120_000));
    let dir = install_dir()?;
    let asset = asset_name(version);
    let zip_path = std::env::temp_dir().join(format!("molaunch-easytier-{asset}"));
    let _ = std::fs::remove_file(&zip_path);

    emit_progress(app, "download", 5, &format!("下载 easytier v{version}"));
    let proxies = state.github_proxies.lock().await.clone();
    // 下载进度：5%→80% 按字节映射，仅在百分比变化时推送（避免逐 chunk 刷屏）
    let last_pct = std::sync::atomic::AtomicU8::new(5);
    let on_progress = |done: u64, total: Option<u64>| {
        let pct = match total {
            Some(t) if t > 0 => 5 + (done.saturating_mul(75) / t) as u8,
            _ => 5,
        };
        if pct > last_pct.load(Ordering::Relaxed) {
            last_pct.store(pct, Ordering::Relaxed);
            emit_progress(app, "download", pct, &format!("下载中 {pct}%"));
        }
    };
    download_zip(&client, version, &zip_path, &proxies, &on_progress).await?;

    emit_progress(app, "extract", 80, "解压安装");
    let extract_dir =
        std::env::temp_dir().join(format!("molaunch-easytier-extract-{}", std::process::id()));
    if extract_dir.exists() {
        let _ = std::fs::remove_dir_all(&extract_dir);
    }
    std::fs::create_dir_all(&extract_dir).map_err(|e| format!("创建临时目录失败: {e}"))?;
    extract_zip_safely(&zip_path, &extract_dir)?;
    let _ = std::fs::remove_file(&zip_path);

    // 清空安装目录旧文件（防残留旧版本），再移动新文件
    if dir.exists() {
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| format!("读取安装目录失败: {e}"))?
            .flatten()
        {
            let p = entry.path();
            if p.is_dir() {
                let _ = std::fs::remove_dir_all(&p);
            } else {
                let _ = std::fs::remove_file(&p);
            }
        }
    }
    move_dir_contents(&extract_dir, &dir)?;
    let _ = std::fs::remove_dir_all(&extract_dir);

    // Unix 补执行权限
    #[cfg(unix)]
    {
        crate::minecraft::system::shell::make_executable(&dir.join(core_name()));
        crate::minecraft::system::shell::make_executable(&dir.join(cli_name()));
    }

    std::fs::write(dir.join(VERSION_FILE), version).map_err(|e| format!("写版本标记失败: {e}"))?;
    emit_progress(app, "done", 100, &format!("easytier v{version} 安装完成"));
    log_info!("[EasyTier] 已安装 v{version} 到 {}", dir.display());
    Ok(())
}

/// 下载安装最新版（`easytier_install` / `easytier_update` 共用）
async fn install_latest(state: &AppState, app: &tauri::AppHandle) -> Result<(), String> {
    let client = crate::http::build_client_with_user_agent("MoLaunch", Some(30_000));
    let version = fetch_latest_release(&client).await?;
    install_version(state, app, &version).await
}

/// 确保已安装：已安装直接返回 core 路径；未安装自动下载安装
pub async fn ensure_installed(state: &AppState, app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if is_installed() {
        return Ok(install_dir()?.join(core_name()));
    }
    if state.easytier_installing.swap(true, Ordering::SeqCst) {
        return Err("easytier 正在安装中，请稍候".to_string());
    }
    let result = install_latest(state, app).await;
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
                let client = crate::http::build_client_with_user_agent("MoLaunch", Some(30_000));
                fetch_latest_release(&client).await.unwrap_or_default()
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
                let result = install_latest(&state, &app).await;
                state.easytier_installing.store(false, Ordering::SeqCst);
                if let Err(e) = &result {
                    emit_progress(&app, "error", 0, e);
                }
                result.map(|_| serde_json::json!({ "success": true }))
            }),
        );
    }

    d.register(
        "set_github_proxies",
        handler!(state, _app, params, {
            let proxies: Vec<GithubProxy> =
                serde_json::from_value(params).map_err(|e| format!("参数解析失败: {e}"))?;
            *state.github_proxies.lock().await = proxies;
            serde_json::to_value(serde_json::json!({ "success": true })).map_err(|e| e.to_string())
        }),
    );
}
