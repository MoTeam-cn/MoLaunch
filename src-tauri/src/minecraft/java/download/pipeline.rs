//! Java 自动下载 5 阶段流水线编排
//!
//! fetching → matching → manifest → downloading → verifying

use std::path::PathBuf;
use tauri::AppHandle;

use crate::log_info;
use crate::minecraft::sources::DownloadSourceMode;

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
///
/// 返回下载的 Java 可执行文件路径（java.exe）
pub async fn download_java_runtime(
    target_major: u32,
    mode: DownloadSourceMode,
    mirror_url: Option<&str>,
    app: Option<&AppHandle>,
) -> Result<PathBuf, String> {
    let client = crate::http::get_client();

    // 阶段 1：拉取 all.json
    progress::emit(app, "fetching", 0, 1, 0, 0, "正在获取 Java 索引...");
    let all_json = fetch::fetch_index(&client, mirror_url, mode).await?;

    // 阶段 2：匹配 component
    progress::emit(app, "matching", 0, 1, 0, 0, "正在匹配 Java 版本...");
    let (component, entry) =
        r#match::match_component(&all_json, target_major).ok_or_else(|| {
            format!(
                "未找到适配 Java {} 的 Mojang Runtime（platform: windows-x64）",
                target_major
            )
        })?;
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

    // 阶段 4：下载文件
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
    files::download_all_files(
        &client,
        &files_to_download,
        &runtime_dir,
        mirror_url,
        mode,
        app,
    )
    .await?;

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
