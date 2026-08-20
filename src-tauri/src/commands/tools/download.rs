//! 外部下载工具（从 `commands/system/download.rs` 迁移）
//! 用户输入 URL + 文件名，通过 `DownloadManager` 下载到 `.Molaunch/Download/`。复用全局
//! `download_state` 和 `download_cancel_flag`/`download_pause_flag`，前端无需额外修改即可
//! 获得暂停/取消/进度查询。函数不再标 `#[tauri::command]`，参数改 typed params，返回值用
//! `serde_json::to_value` 包装为 `serde_json::Value`。

use std::path::PathBuf;

use crate::log_info;
use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::download::DownloadSession;
use crate::state::AppState;
use crate::storage::Storage;
use crate::utils::path::sanitize_file_name;

use super::types::{
    DeleteDownloadParams, DownloadFileParams, ExternalDownloadEntry, ExternalDownloadResult,
};

/// 下载外部 URL 到 `.Molaunch/Download/` 目录
///
/// - 校验 http/https 协议
/// - 校验文件名安全性
/// - 复用 DownloadSession（支持进度 / 暂停 / 取消），进度写入 download_state（分组"外部下载"）
/// - 支持按任务覆盖：自定义 UA / 线程数 / 分片数 / 限速（对应高级设置）
/// - 返回保存路径与文件大小
pub async fn download_file(
    state: &AppState,
    params: DownloadFileParams,
) -> Result<serde_json::Value, String> {
    let url = params.url;
    let file_name = params.file_name;

    // URL 安全校验（协议白名单 + 拒绝内网/回环/链路本地，防 SSRF）
    crate::utils::net::validate_public_http_url(&url)?;

    // 文件名安全校验
    sanitize_file_name(&file_name)?;

    // 解析下载目录：优先使用 config 中的自定义目录，否则用默认 .Molaunch/Download/
    let download_dir = resolve_external_download_dir(state).await;
    crate::utils::fs::ensure_dir(&download_dir)?;
    let save_path: PathBuf = download_dir.join(&file_name);

    log_info!(
        "[ExternalDownload] 开始下载: {} -> {}",
        url,
        save_path.display()
    );

    // 构造下载管理器：从全局配置读取默认值，再按任务覆盖高级设置
    let mut manager_config = DownloadManagerConfig::from_state(state).await;
    manager_config.apply_overrides(
        params.max_threads,
        params.chunk_count,
        params.max_speed,
        params.user_agent,
    );
    let manager =
        crate::minecraft::download::manager::DownloadManager::from_config(&manager_config);

    // 启动 DownloadSession：统一 reset_stages + flag 重置 + manager 构造
    let session = DownloadSession::start_grouped_with_manager(
        state,
        "外部下载",
        vec![(&file_name, 1.0)],
        manager,
        false,
    )
    .await;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.version_name = file_name.clone();
    }

    // 构造下载任务（直接使用原始 URL，不经过 cdn_urls —— 外部 URL 不应走镜像策略）
    let task = crate::minecraft::download::types::DownloadTask {
        id: format!("external_{}", file_name),
        urls: vec![url.clone()],
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
    entries.sort_by_key(|b| std::cmp::Reverse(b.modified));
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
    serde_json::to_value(()).map_err(|e| e.to_string())
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
