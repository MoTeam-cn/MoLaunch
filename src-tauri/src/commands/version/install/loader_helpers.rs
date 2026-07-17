//! 加载器安装辅助函数
//!
//! - `install_single_loader` 通用加载器安装（更新/添加 stage + 调用 loaders::install_loader）
//! - `start_progress_ticker` 模拟进度上涨（在加载器安装期间给用户视觉反馈）

use crate::minecraft::loaders;
use crate::state::{AppState, StageStatus};
use crate::{log_error, log_info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// 安装单个加载器的通用辅助函数
/// 如果阶段已存在（最后一个阶段是加载器安装），则更新它；否则添加新阶段
pub(crate) async fn install_single_loader(
    state: &AppState,
    loader_type: loaders::LoaderType,
    loader_name: &str,
    loader_version: &str,
    mc_version: &str,
    game_dir: &std::path::Path,
    mirror_url: Option<&str>,
    max_threads: usize,
    source_mode: crate::minecraft::sources::DownloadSourceMode,
) -> Result<(), String> {
    // 检查是否已有加载器安装阶段（通过名称判断）
    let has_loader_stage = {
        let ds = state.download_state.lock().unwrap();
        ds.stages.last().map_or(false, |s| {
            s.name.contains("安装") || s.name.contains("加载器")
        })
    };

    if has_loader_stage {
        // 更新现有阶段
        let mut ds = state.download_state.lock().unwrap();
        if let Some(last) = ds.stages.last_mut() {
            last.name = format!("安装 {} {}", loader_name, loader_version);
            last.status = StageStatus::Loading;
            last.progress = 0.0;
        }
        ds.current_stage_index = ds.stages.len() - 1;
    } else {
        // 添加新阶段
        let mut ds = state.download_state.lock().unwrap();
        let mut stage = crate::state::DownloadStage::new(
            format!("安装 {} {}", loader_name, loader_version),
            30.0,
        );
        stage.status = StageStatus::Loading;
        ds.stages.push(stage);
        ds.current_stage_index = ds.stages.len() - 1;
    }

    log_info!("[Merged] Installing {} {}", loader_name, loader_version);

    // 启动进度模拟器
    let ticker_stop = start_progress_ticker(state.download_state.clone(), 5.0, 95.0);

    // 安装加载器
    match loaders::install_loader(
        loader_type,
        mc_version,
        loader_version,
        game_dir,
        mirror_url,
        max_threads,
        None,
        source_mode,
    )
    .await
    {
        Ok(_) => {
            ticker_stop.store(true, Ordering::Relaxed);
            log_info!(
                "[Merged] {} {} installed successfully",
                loader_name,
                loader_version
            );
            let mut ds = state.download_state.lock().unwrap();
            if let Some(last) = ds.stages.last_mut() {
                last.status = StageStatus::Finished;
                last.progress = 1.0;
            }
            Ok(())
        }
        Err(e) => {
            ticker_stop.store(true, Ordering::Relaxed);
            log_error!("[Merged] Failed to install {}: {}", loader_name, e);
            let mut ds = state.download_state.lock().unwrap();
            if let Some(last) = ds.stages.last_mut() {
                last.status = StageStatus::Failed;
            }
            Err(format!("{}: {}", loader_name, e))
        }
    }
}

/// 启动进度模拟器：缓慢上涨进度条，直到 stop 信号为 true
/// 从 start 增长到 cap，约 45-60秒完成
pub(crate) fn start_progress_ticker(
    state: Arc<std::sync::Mutex<crate::state::DownloadState>>,
    start: f64,
    cap: f64,
) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();

    tokio::spawn(async move {
        let mut current = start;
        // 每 500ms 更新一次，更平滑
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
        interval.tick().await;

        while !stop_clone.load(Ordering::Relaxed) {
            interval.tick().await;
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }
            let remaining = cap - current;
            if remaining <= 0.0 {
                break;
            }
            // 每次增长约 1%，从5%到95% 约 45秒完成
            let step = 1.0;
            current = (current + step).min(cap);

            let mut ds = state.lock().unwrap();
            // 更新最后一个阶段的进度（即当前加载器安装阶段）
            if let Some(last) = ds.stages.last_mut() {
                last.progress = current / 100.0;
            }
        }
    });

    stop
}
