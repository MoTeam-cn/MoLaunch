//! 自动更新命令模块（统一入口，平台内部分流）
//!
//! - **Windows 便携版**：自实现下载 + 启动 updater.exe 子进程替换 exe
//!   （绕过 Windows 文件锁，无需 NSIS installer）
//! - **macOS / Linux**：转发到 `tauri-plugin-updater` 官方 plugin
//!   （复用其下载/验签/替换/重启全流程）
//!
//! 前端通过 `system_manager` 统一调用 `check_update` / `download_and_install_update`，
//! 不需要关心平台差异。
//!
//! See: docs/updater/design.md §4 Windows 便携版 updater

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;
// UpdaterExt 仅 macOS/Linux 下载安装路径使用（download_and_install_unix）
#[cfg(not(target_os = "windows"))]
use tauri_plugin_updater::UpdaterExt;

/// updater endpoint（与 tauri.conf.json plugins.updater.endpoints[0] 一致）
const UPDATER_ENDPOINT: &str = "https://api.molaunch.moiu.cn/v1/updates/manifest/raw?target={{target}}&arch={{arch}}&current_version={{current_version}}";

/// 获取当前平台的目标三元组（用于 updater endpoint 模板替换）
fn platform_target() -> &'static str {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match (os, arch) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("windows", "aarch64") => "aarch64-pc-windows-msvc",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => "unknown",
    }
}

/// 简单 semver 比较：manifest_version > current_version 返回 true
fn is_version_newer(manifest: &str, current: &str) -> bool {
    fn parse_semver(s: &str) -> Option<(u64, u64, u64)> {
        let s = s.trim_start_matches('v');
        let mut parts = s.split(|c: char| c == '.' || c == '-');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        Some((major, minor, patch))
    }
    match (parse_semver(manifest), parse_semver(current)) {
        (Some(m), Some(c)) => m > c,
        _ => manifest != current,
    }
}

/// 更新信息（check_update 返回，download_and_install_update 接收）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// 是否有可用更新
    pub available: bool,
    /// 新版本号
    pub version: String,
    /// 更新日志
    pub notes: String,
    /// 是否强制更新（来自 manifest 扩展字段 force_update）
    pub force_update: bool,
    /// 下载 URL（presigned URL，Windows 自实现下载用）
    #[serde(default)]
    pub download_url: String,
    /// 签名（base64，Windows 预留验签用）
    #[serde(default)]
    pub signature: String,
}

impl Default for UpdateInfo {
    fn default() -> Self {
        Self {
            available: false,
            version: String::new(),
            notes: String::new(),
            force_update: false,
            download_url: String::new(),
            signature: String::new(),
        }
    }
}

/// 检查更新（所有平台统一入口）
///
/// 使用 `crate::http::get_client()` 发起请求（走用户配置的代理），
/// 手动解析 manifest JSON 并比较版本，不依赖 tauri-plugin-updater 的内部 HTTP 客户端。
/// macOS/Linux 下载安装仍转发到官方 plugin（`download_and_install_unix`）。
pub async fn check_update(_app: &AppHandle) -> Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    let target = platform_target();
    let arch = std::env::consts::ARCH;

    let url = UPDATER_ENDPOINT
        .replace("{{target}}", target)
        .replace("{{arch}}", arch)
        .replace("{{current_version}}", current_version);

    log::info!("[Updater] 检查更新: {}", url);

    let response = crate::http::get_client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("检查更新失败: {e}"))?;

    // 204/304 = 无更新
    if response.status() == 204 || response.status() == 304 {
        log::info!("[Updater] 服务器返回 {}（无可用更新）", response.status());
        return Ok(UpdateInfo::default());
    }

    if !response.status().is_success() {
        return Err(format!("检查更新失败: HTTP {}", response.status()));
    }

    let json: Value = response
        .json()
        .await
        .map_err(|e| format!("解析更新信息失败: {e}"))?;

    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or("");
    if version.is_empty() {
        log::info!("[Updater] manifest 无 version 字段（无可用更新）");
        return Ok(UpdateInfo::default());
    }

    // 版本比较：manifest 版本必须大于当前版本才算有更新
    if !is_version_newer(version, current_version) {
        log::info!("[Updater] 当前版本 {} 已是最新", current_version);
        return Ok(UpdateInfo::default());
    }

    log::info!(
        "[Updater] 发现新版本: {} -> {}",
        current_version,
        version
    );

    Ok(UpdateInfo {
        available: true,
        version: version.to_string(),
        notes: json
            .get("notes")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        force_update: json
            .get("force_update")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        download_url: json
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        signature: json
            .get("signature")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

/// 下载并安装更新（平台内部分流）
///
/// - **Windows**：自实现下载 + 启动 updater.exe 子进程
/// - **macOS / Linux**：转发到官方 plugin 的 download_and_install()
pub async fn download_and_install(app: &AppHandle, info: UpdateInfo) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        download_and_install_windows(app, info).await
    }
    #[cfg(not(target_os = "windows"))]
    {
        download_and_install_unix(app).await
    }
}

/// Windows 便携版下载安装流程
///
/// 1. 下载新 exe 到临时目录
/// 2. 释放 updater.exe 到 AppData
/// 3. 启动 updater.exe 子进程（传递旧 exe 路径、新 exe 路径、主进程 PID）
/// 4. 主程序退出（updater 接管替换文件）
#[cfg(target_os = "windows")]
async fn download_and_install_windows(app: &AppHandle, info: UpdateInfo) -> Result<(), String> {
    if info.download_url.is_empty() {
        return Err("下载 URL 为空".into());
    }

    // 1. 下载新 exe 到临时目录
    let temp_dir = std::env::temp_dir().join("molaunch_update");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("创建临时目录失败: {e}"))?;
    let new_exe = temp_dir.join("MoLaunch_new.exe");

    log::info!("[Updater] 开始下载新版本: {}", info.download_url);
    let response = crate::http::get_client()
        .get(&info.download_url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {e}"))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("下载读取失败: {e}"))?;
    std::fs::write(&new_exe, &bytes)
        .map_err(|e| format!("写入临时文件失败: {e}"))?;
    log::info!(
        "[Updater] 下载完成: {} ({} bytes)",
        new_exe.display(),
        bytes.len()
    );

    // 2. 释放 updater.exe 到 AppData
    let updater_path = crate::resources::extract_updater()
        .map_err(|e| format!("释放 updater.exe 失败: {e}"))?;

    // 3. 获取当前 exe 路径和 PID
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("获取当前 exe 路径失败: {e}"))?;
    let pid = std::process::id();

    log::info!(
        "[Updater] 启动 updater.exe: old={}, new={}, pid={}",
        current_exe.display(),
        new_exe.display(),
        pid
    );

    // 4. 启动 updater.exe 子进程（传递 --signature 供 updater 二次校验）
    std::process::Command::new(&updater_path)
        .arg("--old-exe")
        .arg(&current_exe)
        .arg("--new-exe")
        .arg(&new_exe)
        .arg("--pid")
        .arg(pid.to_string())
        .arg("--signature")
        .arg(&info.signature)
        .spawn()
        .map_err(|e| format!("启动 updater.exe 失败: {e}"))?;

    // 5. 退出主程序（updater 接管）
    app.exit(0);

    Ok(())
}

/// macOS / Linux 下载安装流程（转发到官方 plugin）
#[cfg(not(target_os = "windows"))]
async fn download_and_install_unix(app: &AppHandle) -> Result<(), String> {
    let updater = app
        .updater()
        .map_err(|e| format!("updater 初始化失败: {e}"))?;
    let update = updater
        .check()
        .await
        .map_err(|e| format!("检查更新失败: {e}"))?;

    if let Some(update) = update {
        update
            .download_and_install(|_| {})
            .await
            .map_err(|e| format!("下载安装失败: {e}"))?;
        app.restart();
    }

    Ok(())
}
