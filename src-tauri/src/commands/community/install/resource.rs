//! 资源文件下载命令（download_resource / download_resource_to_path / install_resource /
//! format_download_filename / get_resource_install_path）

use std::path::PathBuf;

use crate::log_info;
use crate::minecraft::community::types::ResourceType;
use crate::minecraft::download::DownloadSession;
use crate::state::AppState;
use tauri::AppHandle;

use super::helpers::{apply_filename_format, resolve_install_dir};
use super::types::{DownloadRequest, DownloadResult};

/// 下载资源文件到游戏目录（用于"快速安装"）
///
/// 走 DownloadSession（封装 DownloadManager + reset_stages + flag 重置 + callback 工厂），
/// 支持 多 URL fallback + 分片 + 暂停/取消，
/// 进度通过 `download_state` 统一通道，前端在下载管理页面展示。
///
/// - Mod → versions/{vid}/mods/
/// - ResourcePack → versions/{vid}/resourcepacks/
/// - Shader → versions/{vid}/shaderpacks/
/// - DataPack → versions/{vid}/datapacks/
pub async fn download_resource(
    state: &AppState,
    req: DownloadRequest,
) -> Result<DownloadResult, String> {
    log_info!("[Community] Downloading {} from {}", req.file_name, req.url);

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;
    let config = state.config.lock().await;
    let final_file_name = apply_filename_format(
        &req.file_name,
        req.translated_name.as_deref(),
        config.community.filename_format,
    );
    drop(config);

    let target_dir = resolve_install_dir(&game_dir, req.resource_type, req.version_id.as_deref());

    // 确保目录存在
    if !target_dir.exists() {
        crate::utils::fs::ensure_dir(&target_dir)?;
    }

    let target_path = target_dir.join(&final_file_name);

    // 启动 DownloadSession：统一 reset_stages + flag 重置 + manager 构造
    // （之前手工 lock + reset_stages + store flag + DownloadManager::new 6 步合并为 1 行）
    let session =
        DownloadSession::start_grouped(state, "社区资源", vec![(&final_file_name, 1.0)], false)
            .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = final_file_name.clone();
    }

    // 构造下载任务（cdn_urls 根据 source 策略生成多 URL fallback）
    let task = crate::minecraft::download::types::DownloadTask {
        id: format!("community_resource_{}", final_file_name),
        urls: crate::minecraft::sources::cdn_urls(&req.url),
        local_path: target_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    // 统一进度回调工厂（消除闭包复制）
    let progress_callback = session.make_progress_callback(state, 0);

    let results = session
        .manager()
        .download_batch(vec![task], Some(progress_callback))
        .await;

    let result = results.first().ok_or("下载结果为空")?;

    use crate::minecraft::download::types::DownloadStatus;
    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        session.mark_failed(state, 1);
        return Err(err);
    }

    let size = std::fs::metadata(&target_path)
        .map(|m| m.len())
        .unwrap_or(0);

    session.mark_complete(state);

    log_info!(
        "[Community] Downloaded {} ({} bytes) to {}",
        final_file_name,
        size,
        target_path.display()
    );

    Ok(DownloadResult {
        path: target_path.to_string_lossy().to_string(),
        size,
    })
}

/// 根据用户设置的 `community_filename_format` 格式化下载文件名
///
/// 详情页"下载到任意路径"流程使用：前端在弹保存对话框前调用此命令，
/// 获取格式化后的文件名作为默认名，避免使用原始名导致设置不生效。
pub async fn format_download_filename(
    state: &AppState,
    file_name: String,
    translated_name: Option<String>,
) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(apply_filename_format(
        &file_name,
        translated_name.as_deref(),
        config.community.filename_format,
    ))
}

/// 下载资源文件到自定义路径（用户通过文件管理器选择）
///
/// 走 DownloadSession（封装 DownloadManager + reset_stages + flag 重置 + callback 工厂），
/// 支持 多 URL fallback + 分片 + 暂停/取消，
/// 进度通过 `download_state` 统一通道，前端在下载管理页面展示。
pub async fn download_resource_to_path(
    state: &AppState,
    _app: &AppHandle,
    url: String,
    file_name: String,
    save_path: String,
) -> Result<DownloadResult, String> {
    log_info!("[Community] 下载 {} 到 {}", file_name, save_path);

    let save_path = PathBuf::from(&save_path);

    // 确保父目录存在
    if let Some(parent) = save_path.parent() {
        if !parent.exists() {
            crate::utils::fs::ensure_dir(parent)?;
        }
    }

    // 路径安全：canonicalize 后校验目标必须位于下载目录内（防任意路径写入）
    let download_dir = crate::commands::tools::download::resolve_external_download_dir(state).await;
    crate::utils::fs::ensure_dir(&download_dir)?;
    let download_canon = download_dir
        .canonicalize()
        .map_err(|e| format!("下载目录不可用: {}", e))?;
    let parent = save_path
        .parent()
        .ok_or_else(|| "目标路径无效".to_string())?;
    let parent_canon = parent
        .canonicalize()
        .map_err(|e| format!("目标目录不可用: {}", e))?;
    let file_name_part = save_path
        .file_name()
        .ok_or_else(|| "目标路径无效".to_string())?;
    let save_path = parent_canon.join(file_name_part);
    if !save_path.starts_with(&download_canon) {
        return Err("下载路径超出下载目录范围".to_string());
    }

    // 启动 DownloadSession：统一 reset_stages + flag 重置 + manager 构造
    let session =
        DownloadSession::start_grouped(state, "社区资源", vec![(&file_name, 1.0)], false).await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = file_name.clone();
    }

    // 构造下载任务（cdn_urls 根据 source 策略生成多 URL fallback）
    let task = crate::minecraft::download::types::DownloadTask {
        id: format!("community_{}", file_name),
        urls: crate::minecraft::sources::cdn_urls(&url),
        local_path: save_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    // 统一进度回调工厂（消除闭包复制）
    let progress_callback = session.make_progress_callback(state, 0);

    let results = session
        .manager()
        .download_batch(vec![task], Some(progress_callback))
        .await;

    let result = results.first().ok_or("下载结果为空")?;

    use crate::minecraft::download::types::DownloadStatus;
    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        session.mark_failed(state, 1);
        return Err(err);
    }

    let size = std::fs::metadata(&save_path).map(|m| m.len()).unwrap_or(0);

    session.mark_complete(state);

    log_info!("[Community] 下载完成: {} ({} bytes)", file_name, size);

    Ok(DownloadResult {
        path: save_path.to_string_lossy().to_string(),
        size,
    })
}

/// 安装资源文件（与 download_resource 相同，语义化命名）
pub async fn install_resource(
    state: &AppState,
    req: DownloadRequest,
) -> Result<DownloadResult, String> {
    download_resource(state, req).await
}

/// 获取资源默认安装路径（用于前端显示"打开文件夹"）
pub async fn get_resource_install_path(
    state: &AppState,
    resource_type: ResourceType,
    version_id: Option<String>,
) -> Result<String, String> {
    let game_dir = crate::state::resolve_game_dir_from_state(state).await;

    let path = resolve_install_dir(&game_dir, resource_type, version_id.as_deref());
    Ok(path.to_string_lossy().to_string())
}
