//! Java 自动下载 5 阶段流水线编排
//!
//! fetching → matching → manifest → downloading → verifying

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex, OnceLock};
use tauri::AppHandle;

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

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

/// 每个 component 一把安装锁
///
/// 同一份 runtime 被两个安装任务同时写会互相覆盖对方刚写好的文件：
/// 两边都以为自己在装同一份 Java，实际每人只写了一半，最后谁都验证不过。
/// 锁按 component 分——装 Java 8 不必等装 Java 21。
fn install_locks() -> &'static StdMutex<HashMap<String, Arc<AsyncMutex<()>>>> {
    static LOCKS: OnceLock<StdMutex<HashMap<String, Arc<AsyncMutex<()>>>>> = OnceLock::new();
    LOCKS.get_or_init(|| StdMutex::new(HashMap::new()))
}

/// 排队拿这个 component 的安装锁（前一个装完才轮到）
async fn lock_install(component: &str) -> OwnedMutexGuard<()> {
    let lock = {
        let mut map = install_locks()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        map.entry(component.to_string())
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    };
    lock.lock_owned().await
}

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

    // 同一 component 的安装串行化：并发写同一份 runtime 只会互相覆盖
    let _install_guard = lock_install(&component).await;

    // 阶段 3：拉取 manifest
    progress::emit(app, "manifest", 0, 1, 0, 0, "正在获取文件清单...");
    let manifest = fetch::fetch_manifest(&client, &entry.manifest.url, mirror_url, mode).await?;
    let files_to_download = files::filter_downloadable_files(&manifest);
    let total_files = files_to_download.len();
    let total_bytes = files::total_bytes(&files_to_download);
    log_info!("[JavaDownload] {} files to download", total_files);

    // 阶段 4：下载文件（复用通用 DownloadManager：文件级并发 + 分片 + SHA1 校验）
    let runtime_dir = files::get_runtime_dir(&component)?;
    // 安装开始前取样：这个目录与官方启动器共用，只有本次新建的才允许在失败时整删
    let dir_existed = runtime_dir.exists();
    // 装到一半的 java.exe 能跑但缺模块：挂上标记，让 Java 扫描跳过它
    files::mark_install_started(&runtime_dir);
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
        files::cleanup_failed_install(&runtime_dir, dir_existed);
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
    if let Err(e) = verify::verify_downloaded_java(&java_exe) {
        // 验证都过不了就不是「装好了」，按安装失败处理
        files::cleanup_failed_install(&runtime_dir, dir_existed);
        return Err(e);
    }

    // 下载与验证都过了，这个 runtime 才算装好
    files::clear_install_marker(&runtime_dir);

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

#[cfg(test)]
#[path = "pipeline_test.rs"]
mod pipeline_test;
