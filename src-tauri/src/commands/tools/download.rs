//! 外部下载工具（从 `commands/system/download.rs` 迁移）
//!
//! 用户输入 URL + 文件名，通过 `DownloadManager` 下载到 `.Molaunch/Download/` 目录。
//! 复用全局 `download_state` 和 `download_cancel_flag` / `download_pause_flag`，
//! 前端无需额外修改即可获得暂停 / 取消 / 进度查询能力。
//!
//! 与旧版差异：函数不再标注 `#[tauri::command]`，参数改为 typed params，
//! 返回值统一用 `serde_json::to_value` 包装为 `serde_json::Value`。

use std::path::PathBuf;
use std::sync::Arc;

use crate::log_info;
use crate::state::AppState;
use crate::state::DownloadStage;
use crate::storage::Storage;
use crate::utils::path::sanitize_file_name;

use super::types::{
    DeleteDownloadParams, DownloadFileParams, ExternalDownloadEntry, ExternalDownloadResult,
};

/// 下载外部 URL 到 `.Molaunch/Download/` 目录
///
/// - 校验 http/https 协议
/// - 校验文件名安全性
/// - 复用 DownloadManager（支持进度 / 暂停 / 取消），进度写入 download_state（分组"外部下载"）
/// - 返回保存路径与文件大小
pub async fn download_file(
    state: &AppState,
    params: DownloadFileParams,
) -> Result<serde_json::Value, String> {
    let url = params.url;
    let file_name = params.file_name;

    // 协议白名单校验
    let lower_url = url.to_lowercase();
    if !lower_url.starts_with("http://") && !lower_url.starts_with("https://") {
        return Err("下载地址必须以 http:// 或 https:// 开头".to_string());
    }

    // 文件名安全校验
    sanitize_file_name(&file_name)?;

    // 解析下载目录：优先使用 config 中的自定义目录，否则用默认 .Molaunch/Download/
    let download_dir = resolve_external_download_dir(state).await;
    std::fs::create_dir_all(&download_dir)
        .map_err(|e| format!("创建下载目录失败: {}", e))?;
    let save_path: PathBuf = download_dir.join(&file_name);

    log_info!(
        "[ExternalDownload] 开始下载: {} -> {}",
        url,
        save_path.display()
    );

    // 重置 download_state，注册单阶段任务（分组"外部下载"）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.reset_stages(vec![DownloadStage::new_grouped(
            &file_name,
            1.0,
            "外部下载",
        )]);
        ds.version_name = file_name.clone();
    }
    // 重置 cancel/pause flag（防止上次任务残留状态影响新任务）
    state
        .download_cancel_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);

    // 构造下载任务（直接使用原始 URL，不经过 cdn_urls —— 外部 URL 不应走镜像策略）
    let task = crate::minecraft::download::types::DownloadTask {
        id: format!("external_{}", file_name),
        urls: vec![url.clone()],
        local_path: save_path.to_string_lossy().to_string(),
        expected_size: 0,
        expected_hash: None,
    };

    // 进度回调：sync_stage_from_progress 统一同步到 download_state
    let cb_state = state.download_state.clone();
    let progress_callback: Arc<
        dyn Fn(crate::minecraft::download::types::GlobalProgress) + Send + Sync,
    > = Arc::new(move |p| {
        let mut ds = cb_state.lock().unwrap();
        ds.sync_stage_from_progress(
            0,
            p.downloaded_bytes,
            p.total_bytes,
            p.completed_files,
            p.total_files,
            p.current_speed,
        );
    });

    let config = state.config.lock().await;
    let chunk_count = config.download.chunk_count.max(1) as usize;
    drop(config);

    let manager = crate::minecraft::download::manager::DownloadManager::new(
        4,
        chunk_count,
        0,
        crate::minecraft::sources::DownloadSourceMode::Smart,
    )
    .with_cancel_flag(state.download_cancel_flag.clone())
    .with_pause_flag(state.download_pause_flag.clone());

    let results = manager.download_batch(vec![task], Some(progress_callback)).await;

    let result = results.first().ok_or("下载结果为空")?;

    use crate::minecraft::download::types::DownloadStatus;
    if result.status != DownloadStatus::Completed && result.status != DownloadStatus::Skipped {
        let err = result
            .error
            .clone()
            .unwrap_or_else(|| "未知错误".to_string());
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.mark_failed(1);
        }
        return Err(err);
    }

    let size = std::fs::metadata(&save_path).map(|m| m.len()).unwrap_or(0);

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_complete();
    }

    log_info!(
        "[ExternalDownload] 下载完成: {} ({} bytes)",
        file_name,
        size
    );

    let result = ExternalDownloadResult {
        path: save_path.to_string_lossy().to_string(),
        size,
        file_name,
    };
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 解析外部下载目录：优先使用 config 中的自定义目录，否则用默认 `.Molaunch/Download/`
///
/// 提取为公共 helper，避免 `download_file` / `get_download_dir` /
/// `list_downloads` / `delete_download` 四处重复实现。
pub async fn resolve_external_download_dir(state: &AppState) -> PathBuf {
    let config = state.config.lock().await;
    config
        .external_download_dir
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| Storage::instance().download_dir())
}

/// 获取外部下载目录路径（自定义或默认 `.Molaunch/Download/`）
pub async fn get_download_dir(state: &AppState) -> Result<serde_json::Value, String> {
    let dir = resolve_external_download_dir(state)
        .await
        .to_string_lossy()
        .to_string();
    serde_json::to_value(&dir).map_err(|e| e.to_string())
}

/// 列举下载目录下的已下载文件（自定义或默认 `.Molaunch/Download/`）
pub async fn list_downloads(state: &AppState) -> Result<serde_json::Value, String> {
    let dir = resolve_external_download_dir(state).await;
    if !dir.exists() {
        let empty: Vec<ExternalDownloadEntry> = vec![];
        return serde_json::to_value(&empty).map_err(|e| e.to_string());
    }

    let mut entries = Vec::new();
    let read = std::fs::read_dir(&dir).map_err(|e| format!("读取下载目录失败: {}", e))?;

    for entry in read.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        // 过滤分片下载临时文件（.part0 / .part1 / ... .partN）
        // 这些是 DownloadManager 分片下载过程中产生的临时分片，合并后会被删除，
        // 不应出现在"已下载文件"列表中
        if is_chunk_part_file(&name) {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let modified = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        entries.push(ExternalDownloadEntry {
            name,
            size,
            modified,
        });
    }

    // 按修改时间倒序排列（最新在前）
    entries.sort_by(|a, b| b.modified.cmp(&a.modified));
    serde_json::to_value(&entries).map_err(|e| e.to_string())
}

/// 删除下载目录下的指定文件（自定义或默认 `.Molaunch/Download/`）
pub async fn delete_download(
    state: &AppState,
    params: DeleteDownloadParams,
) -> Result<serde_json::Value, String> {
    sanitize_file_name(&params.file_name)?;
    let path = resolve_external_download_dir(state)
        .await
        .join(&params.file_name);
    if !path.exists() {
        return Err(format!("文件不存在: {}", params.file_name));
    }
    std::fs::remove_file(&path).map_err(|e| format!("删除文件失败: {}", e))?;
    log_info!("[ExternalDownload] 已删除: {}", params.file_name);
    serde_json::to_value(&()).map_err(|e| e.to_string())
}

/// 判断文件名是否为分片下载的临时分片文件（形如 `xxx.part0` / `xxx.part1` / ... / `xxx.partN`）
///
/// DownloadManager 分片下载时会在目标文件同目录创建 `.partN` 临时文件，
/// 合并成功后会被删除，但下载过程中若用户刷新"已下载文件"列表会看到这些临时文件。
/// 通过此过滤避免将临时分片展示给用户。
fn is_chunk_part_file(name: &str) -> bool {
    // 查找最后一个 ".part" 子串
    let Some(part_idx) = name.rfind(".part") else {
        return false;
    };
    let suffix = &name[part_idx + 5..]; // ".part" 之后的部分
    // 后缀必须全部为数字（如 "0" / "12"），且非空
    !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit())
}
