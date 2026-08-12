//! macOS / Linux 下载安装流程（转发到官方 plugin）

use tauri::AppHandle;
use tauri::Emitter;
// UpdaterExt 仅 macOS/Linux 下载安装路径使用
use tauri_plugin_updater::UpdaterExt;

use super::{PROGRESS_EVENT, PROGRESS_THROTTLE_BYTES};

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
        let app_clone = app.clone();
        let mut downloaded: u64 = 0;
        let mut last_emitted: u64 = 0;
        update
            .download_and_install(
                move |chunk_len, content_length| {
                    downloaded += chunk_len as u64;
                    let total = content_length.unwrap_or(0);
                    // 节流推送：累计满阈值或下载完成时 emit，避免每 chunk 一次事件
                    if downloaded - last_emitted >= PROGRESS_THROTTLE_BYTES
                        || (total > 0 && downloaded >= total)
                    {
                        last_emitted = downloaded;
                        let _ = app_clone.emit(
                            PROGRESS_EVENT,
                            serde_json::json!({
                                "downloaded": downloaded,
                                "total": total,
                            }),
                        );
                    }
                },
                || {},
            )
            .await
            .map_err(|e| format!("下载安装失败: {e}"))?;
        app.restart();
    }

    Ok(())
}
