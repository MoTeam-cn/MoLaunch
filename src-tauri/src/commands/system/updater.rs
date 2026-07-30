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
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

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
/// 复用 `tauri-plugin-updater` 的 check() 获取 manifest，从中提取扩展字段
/// （force_update / url / signature）返回给前端。
pub async fn check_update(app: &AppHandle) -> Result<UpdateInfo, String> {
    let updater = app.updater().map_err(|e| format!("updater 初始化失败: {e}"))?;

    // 服务器在"无可用更新"时可能返回空 manifest（缺少 version 字段），
    // tauri-plugin-updater 内部反序列化会报 "missing field `version`"。
    // 捕获此 serde 错误并视为"无更新"，避免向用户报错。
    let update = match updater.check().await {
        Ok(u) => u,
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("missing field") {
                log::info!("[Updater] 服务器返回空 manifest（无可用更新）: {}", err_str);
                return Ok(UpdateInfo::default());
            }
            return Err(format!("检查更新失败: {err_str}"));
        }
    };

    match update {
        Some(update) => {
            // raw_json 是 manifest 接口返回的原始 JSON，包含 MoLaunch 扩展字段
            let raw = &update.raw_json;
            let force_update = raw
                .get("force_update")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let download_url = raw
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let signature = raw
                .get("signature")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            Ok(UpdateInfo {
                available: true,
                version: update.version.clone(),
                notes: update.body.clone().unwrap_or_default(),
                force_update,
                download_url,
                signature,
            })
        }
        None => Ok(UpdateInfo::default()),
    }
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
