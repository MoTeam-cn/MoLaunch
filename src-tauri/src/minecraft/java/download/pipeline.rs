//! Java 自动下载 5 阶段流水线编排
//!
//! fetching → matching → manifest → downloading → verifying

use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;

use crate::log_info;
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::manager::DownloadManager;
use crate::minecraft::download::types::{DownloadStatus, DownloadTask, GlobalProgress};
use crate::minecraft::sources::{build_replace_urls, DownloadSourceMode};

use super::constants::DOWNLOAD_DOMAIN_REPLACEMENTS;
use super::fetch;
use super::files;
use super::progress;
use super::r#match;
use super::verify;

/// 下载 Java Runtime
///
/// - `target_major`: 目标 Java 大版本号（如 21、17、8）
/// - `mode`: 下载源模式（Official / Mirror / Smart）
/// - `mirror_url`: 自定义镜像源 URL（None 或空则用 BMCLAPI）
/// - `app`: Tauri AppHandle（用于推送进度事件）
/// - `manager_config`: 通用 DownloadManager 配置（并发/分片/限速）
///
/// 返回下载的 Java 可执行文件路径（java.exe）
pub async fn download_java_runtime(
    target_major: u32,
    mode: DownloadSourceMode,
    mirror_url: Option<&str>,
    app: Option<&AppHandle>,
    manager_config: &DownloadManagerConfig,
) -> Result<PathBuf, String> {
    let client = crate::http::get_client();

    // 阶段 1：拉取 all.json
    progress::emit(app, "fetching", 0, 1, 0, 0, "正在获取 Java 索引...");
    let all_json = fetch::fetch_index(&client, mirror_url, mode).await?;

    // 阶段 2：匹配 component
    progress::emit(app, "matching", 0, 1, 0, 0, "正在匹配 Java 版本...");
    let (component, entry) = r#match::match_component(&all_json, target_major)?;
    log_info!(
        "[JavaDownload] Matched component: {} (version: {})",
        component,
        entry.version.name
    );

    // 阶段 3：拉取 manifest
    progress::emit(app, "manifest", 0, 1, 0, 0, "正在获取文件清单...");
    let manifest = fetch::fetch_manifest(&client, &entry.manifest.url, mirror_url, mode).await?;
    let files_to_download = files::filter_downloadable_files(&manifest);
    let total_files = files_to_download.len();
    let total_bytes = files::total_bytes(&files_to_download);
    log_info!("[JavaDownload] {} files to download", total_files);

    // 阶段 4：下载文件（复用通用 DownloadManager：文件级并发 + 分片 + SHA1 校验）
    let runtime_dir = files::get_runtime_dir(&component)?;
    progress::emit(
        app,
        "downloading",
        0,
        total_files,
        0,
        total_bytes,
        &format!(
            "正在下载 Java {} (共 {} 个文件)...",
            target_major, total_files
        ),
    );
    let tasks = files_to_download
        .iter()
        .map(|(path_str, file)| {
            let download_info = &file.downloads.as_ref().unwrap().raw;
            let local_path = runtime_dir.join(path_str);
            files::validate_path_traversal(path_str, &local_path, &runtime_dir)?;
            let urls = build_replace_urls(
                &download_info.url,
                mirror_url,
                DOWNLOAD_DOMAIN_REPLACEMENTS,
                mode,
            );
            Ok(DownloadTask {
                id: path_str.clone(),
                urls,
                local_path: local_path.to_string_lossy().to_string(),
                expected_size: download_info.size as i64,
                expected_hash: if download_info.sha1.is_empty() {
                    None
                } else {
                    Some(download_info.sha1.clone())
                },
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let progress_callback: Option<Arc<dyn Fn(GlobalProgress) + Send + Sync>> = app.map(|handle| {
        let handle = handle.clone();
        Arc::new(move |gp: GlobalProgress| {
            let done = gp.completed_files + gp.skipped_files;
            progress::emit(
                Some(&handle),
                "downloading",
                done,
                gp.total_files,
                gp.downloaded_bytes,
                gp.total_bytes,
                &format!("已下载 {} / {} 文件", done, gp.total_files),
            );
        }) as Arc<dyn Fn(GlobalProgress) + Send + Sync>
    });
    let results = DownloadManager::from_config(manager_config)
        .download_batch(tasks, progress_callback)
        .await;
    if let Some(failed) = results.iter().find(|r| r.status == DownloadStatus::Failed) {
        let _ = std::fs::remove_dir_all(&runtime_dir);
        return Err(format!(
            "下载文件失败: {} - {}",
            failed.task_id,
            failed.error.clone().unwrap_or_default()
        ));
    }
    for (path_str, file) in &files_to_download {
        if file.executable {
            let _exe_path = runtime_dir.join(path_str);
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(_exe_path, std::fs::Permissions::from_mode(0o755));
            }
        }
    }

    // 阶段 5：验证下载的 Java
    progress::emit(app, "verifying", 0, 1, 0, 0, "正在验证 Java...");
    let java_exe = files::find_java_exe(&runtime_dir)?;
    progress::emit(
        app,
        "verifying",
        0,
        1,
        0,
        0,
        &format!("验证 Java: {}", java_exe.display()),
    );
    verify::verify_downloaded_java(&java_exe);

    // 完成
    progress::emit(
        app,
        "done",
        total_files,
        total_files,
        total_bytes,
        total_bytes,
        "下载完成",
    );

    Ok(java_exe)
}
