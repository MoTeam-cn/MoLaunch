//! 并发下载与检测结果类型：DetectedModpack / download_files_concurrent

use crate::log_info;
use crate::minecraft::download::DownloadSession;
use crate::state::AppState;

/// 检测到的整合包信息（detect_modpack_format 的返回值）
///
/// `archive_base_folder` 为整合包关键文件所在的层级前缀（如 `""` 或 `"subfolder/"`），
/// 用于后续构造 overrides 完整前缀。
pub struct DetectedModpack {
    pub format: super::super::types::ModpackFormat,
    /// 关键文件所在层级前缀（如 `""` 或 `"subfolder/"`），已含末尾 `/`（根目录为空字符串）
    pub archive_base_folder: String,
    /// CF manifest.json 或 MCBBS manifest.json/mcbbs.packmeta 的原始内容
    pub manifest_content: Option<String>,
    /// MR modrinth.index.json 的原始内容
    pub index_content: Option<String>,
    /// HMCL modpack.json 的原始内容
    pub hmcl_content: Option<String>,
    /// MMC mmc-pack.json 的原始内容
    pub mmc_content: Option<String>,
    /// MMC instance.cfg 的原始内容（仅 MMC 格式有值，用于配置迁移）
    pub mmc_cfg_content: Option<String>,
    /// LauncherPack 内层整合包在 zip 中的完整路径（如 `modpack.zip` 或 `subfolder/modpack.mrpack`）
    pub launcher_inner_path: Option<String>,
}

/// 并发下载多个文件，进度汇总到 download_state 的指定 stage
///
/// 走 DownloadSession::attach：自动按文件大小走分片或普通下载，与 MC 本体/库/assets
/// 走同一套下载基础设施。max_threads 由 session 内部从 config 读取，避免双重数据源。
pub async fn download_files_concurrent(
    state: &AppState,
    stage_index: usize,
    files: &[(Vec<String>, String, u64)], // (urls, target_path, file_size)
    _precomputed_total: u64,
) -> Result<(), String> {
    use crate::minecraft::download::types::DownloadTask;

    if files.is_empty() {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_bytes(stage_index, 1, 1);
        return Ok(());
    }

    // 构造 DownloadTask 列表
    let tasks: Vec<DownloadTask> = files
        .iter()
        .enumerate()
        .map(|(i, (urls, path, size))| DownloadTask {
            id: format!("modpack_{}", i),
            urls: urls.clone(),
            local_path: path.clone(),
            expected_size: *size as i64,
            expected_hash: None,
        })
        .collect();

    let total_count = files.len() as u64;

    // 子流程接入：仅构造 manager + callback（stages / flag 已由 install_modpack 处理）
    // manager 内部从 config 读取 max_threads/chunk_count/speed_limit/source_mode
    let session = DownloadSession::attach(state).await;
    let progress_callback = session.make_progress_callback(state, stage_index);
    let results = session
        .manager()
        .download_batch(tasks, Some(progress_callback))
        .await;

    // 收集失败（取消导致的失败不逐个打印，避免刷屏）
    let is_cancelled = state
        .download_cancel_flag
        .load(std::sync::atomic::Ordering::Relaxed);
    let mut errors: Vec<String> = Vec::new();
    for (i, r) in results.iter().enumerate() {
        if r.status != crate::minecraft::download::types::DownloadStatus::Completed
            && r.status != crate::minecraft::download::types::DownloadStatus::Skipped
        {
            let (urls, path, _) = &files[i];
            let err = r.error.clone().unwrap_or_else(|| format!("{:?}", r.status));
            if !is_cancelled {
                log_info!("[Community] 下载失败: {} → {}", path, err);
                log_info!("[Community] 尝试过的 URL: {}", urls.join(" | "));
            }
            errors.push(format!("{}: {}", urls.join(" | "), err));
        }
    }

    if !errors.is_empty() {
        if is_cancelled {
            log_info!("[Community] 下载已取消，{} 个文件未完成", errors.len());
            return Err("下载已取消".to_string());
        }
        log_info!("[Community] 共 {} 个文件下载失败：", errors.len());
        for (i, e) in errors.iter().enumerate() {
            log_info!("[Community] 失败 #{}: {}", i + 1, e);
        }
        return Err(format!(
            "部分文件下载失败 ({}/{}): 首个错误={}",
            errors.len(),
            total_count,
            errors[0]
        ));
    }

    Ok(())
}