//! macOS / Linux 下载安装流程（转发到官方 plugin）

use tauri::AppHandle;
// UpdaterExt 仅 macOS/Linux 下载安装路径使用
use tauri_plugin_updater::UpdaterExt;

/// macOS / Linux 下载安装流程（转发到官方 plugin）
pub(super) async fn download_and_install_unix(app: &AppHandle) -> Result<(), String> {
    let updater = app
        .updater()
        .map_err(|e| format!("updater 初始化失败: {e}"))?;
    let update = updater
        .check()
        .await
        .map_err(|e| format!("检查更新失败: {e}"))?;

    if let Some(update) = update {
        update
            .download_and_install(|_, _| {}, || {})
            .await
            .map_err(|e| format!("下载安装失败: {e}"))?;
        app.restart();
    }

    Ok(())
}
