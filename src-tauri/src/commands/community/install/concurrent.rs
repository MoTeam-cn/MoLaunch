//! 社区资源下载安装 - 并发下载与 zip 操作
//!
//! 包含：
//! - `download_files_concurrent`：多文件并发下载，进度汇总到 download_state 的指定 stage
//! - `extract_overrides`：从整合包 zip 解压 overrides / client-overrides 到 instance 目录
//! - `detect_modpack_format`：检测整合包格式（CF manifest.json / MR modrinth.index.json）

use crate::log_info;
use crate::state::AppState;
use std::sync::Arc;

/// 并发下载多个文件，进度汇总到 download_state 的指定 stage
///
/// 统一走 DownloadManager：自动按文件大小走分片下载（>1MB/chunk 走 chunk::download_chunked）
/// 或普通下载（小文件直连），与 MC 本体/库/assets 走同一套下载基础设施。
/// 进度通过 `sync_stage_from_progress` 统一同步到 download_state（速度/字节累加由统一方法处理）。
pub(super) async fn download_files_concurrent(
    state: &AppState,
    stage_index: usize,
    files: &[(Vec<String>, String, u64)], // (urls, target_path, file_size)
    max_threads: usize,
    _precomputed_total: u64,
) -> Result<(), String> {
    use crate::minecraft::download::manager::DownloadManager;
    use crate::minecraft::download::types::DownloadTask;
    use crate::minecraft::sources::DownloadSourceMode;

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

    // 进度回调：DownloadManager 已内置 300ms timer + 滑动窗口速度计算
    // 直接用 sync_stage_from_progress 统一同步，无需额外 timer / 原子计数器 / 速度计算
    let progress_state = state.download_state.clone();
    let progress_stage_index = stage_index;
    let progress_callback: Arc<
        dyn Fn(crate::minecraft::download::types::GlobalProgress) + Send + Sync,
    > = Arc::new(move |p| {
        let mut ds = progress_state.lock().unwrap();
        ds.sync_stage_from_progress(
            progress_stage_index,
            p.downloaded_bytes,
            p.total_bytes,
            p.completed_files,
            p.total_files,
            p.current_speed,
        );
    });

    // 用 DownloadManager 下载（自动分片 + 多线程 + 重试 + URL fallback）
    let config = state.config.lock().await;
    let chunk_count = config.chunk_count.max(1) as usize;
    drop(config);
    let manager = DownloadManager::new(max_threads, chunk_count, 0, DownloadSourceMode::Smart)
        .with_cancel_flag(state.download_cancel_flag.clone())
        .with_pause_flag(state.download_pause_flag.clone());
    let results = manager.download_batch(tasks, Some(progress_callback)).await;

    // 收集失败
    let mut errors: Vec<String> = Vec::new();
    for (i, r) in results.iter().enumerate() {
        if r.status != crate::minecraft::download::types::DownloadStatus::Completed
            && r.status != crate::minecraft::download::types::DownloadStatus::Skipped
        {
            let (urls, path, _) = &files[i];
            let err = r.error.clone().unwrap_or_else(|| format!("{:?}", r.status));
            log_info!("[Community] 下载失败: {} → {}", path, err);
            log_info!("[Community] 尝试过的 URL: {}", urls.join(" | "));
            errors.push(format!("{}: {}", urls.join(" | "), err));
        }
    }

    if !errors.is_empty() {
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

/// 从 zip 解压 overrides（和 client-overrides）到 instance 目录
pub(super) fn extract_overrides(
    archive: &mut zip::ZipArchive<std::fs::File>,
    instance_dir: &std::path::Path,
    state: &AppState,
) -> Result<(), String> {
    use std::io::Read;
    let mut count: usize = 0;
    let total = archive.len();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;
        let name = entry.name().to_string();

        // CF/MR overrides/ 前缀 → 去掉前缀复制到 instance 目录
        // MR client-overrides/ 前缀 → 同样去掉前缀复制到 instance 目录（覆盖 overrides）
        let relative = if name.starts_with("overrides/") {
            &name["overrides/".len()..]
        } else if name.starts_with("client-overrides/") {
            &name["client-overrides/".len()..]
        } else {
            continue;
        };

        if relative.is_empty() || relative.ends_with('/') {
            continue;
        }

        let target = instance_dir.join(relative);
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                crate::utils::fs::ensure_dir(parent)?;
            }
        }

        if entry.is_file() {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("读取文件失败: {}", e))?;
            std::fs::write(&target, &buf).map_err(|e| format!("写入文件失败: {}", e))?;
            count += 1;
        }

        // 每 10 个文件更新一次进度
        if count % 10 == 0 {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_bytes(3, count as u64, total as u64);
        }
    }

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_bytes(3, count as u64, total as u64);
    }
    log_info!("[Community] overrides 解压完成 ({} 个文件)", count);
    Ok(())
}

/// 检测整合包格式，返回 (format, cf_manifest_content, mr_index_content)
pub(super) fn detect_modpack_format(
    archive: &mut zip::ZipArchive<std::fs::File>,
) -> Result<(super::types::ModpackFormat, Option<String>, Option<String>), String> {
    use super::types::ModpackFormat;
    use std::io::Read;
    let mut cf_content: Option<String> = None;
    let mut mr_content: Option<String> = None;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;
        let name = entry.name().to_string();
        let is_root = !name.contains('/');

        if is_root && name == "manifest.json" {
            let mut s = String::new();
            entry
                .read_to_string(&mut s)
                .map_err(|e| format!("读取 manifest.json 失败: {}", e))?;
            cf_content = Some(s);
        } else if is_root && name == "modrinth.index.json" {
            let mut s = String::new();
            entry
                .read_to_string(&mut s)
                .map_err(|e| format!("读取 modrinth.index.json 失败: {}", e))?;
            mr_content = Some(s);
        }
    }

    let format = match (&cf_content, &mr_content) {
        (Some(_), _) => ModpackFormat::Curseforge,
        (_, Some(_)) => ModpackFormat::Modrinth,
        (None, None) => {
            return Err(
                "无法识别的整合包格式：未找到 manifest.json 或 modrinth.index.json".to_string(),
            );
        }
    };

    Ok((format, cf_content, mr_content))
}
