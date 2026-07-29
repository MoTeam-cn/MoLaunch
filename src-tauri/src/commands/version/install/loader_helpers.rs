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

    // 启动进度模拟器（对数曲线，前期快后期慢）
    // 统一由 ticker 管理伪进度，加载器 install 内部不需要手写 progress_callback
    let ticker_stop = start_progress_ticker(state, None, 5.0, 95.0);

    // 安装加载器（progress_callback 传 None，进度由 ticker 统一管理）
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

/// 启动进度模拟器：对数曲线上涨（前期快后期慢），直到 stop 信号为 true
///
/// 使用 `current = start + (cap - start) * (1 - exp(-elapsed / tau))` 曲线：
/// - 1 秒后约 30%（快速安装也能看到明显进度）
/// - 3 秒后约 60%
/// - 10 秒后约 92%
/// - 30 秒后约 95%（卡在上限，等安装完成跳 100%）
///
/// - `stage_index` 为 None 时更新最后一个阶段（兼容加载器安装场景）
/// - 每次更新后广播到 WS，让前端实时看到伪进度动画
pub(crate) fn start_progress_ticker(
    state: &AppState,
    stage_index: Option<usize>,
    start: f64,
    cap: f64,
) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let download_state = state.download_state.clone();
    let app_state = state.clone();

    tokio::spawn(async move {
        let tau = 3.0; // 时间常数：控制曲线上升速度
        let mut elapsed_ms: u64 = 0;
        let step_ms: u64 = 200;
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(step_ms));
        interval.tick().await; // 跳过第一次立即触发

        while !stop_clone.load(Ordering::Relaxed) {
            interval.tick().await;
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }
            elapsed_ms += step_ms;
            let elapsed_secs = elapsed_ms as f64 / 1000.0;
            // 对数曲线：1 - exp(-t/tau)
            let factor = 1.0 - (-elapsed_secs / tau).exp();
            let current = start + (cap - start) * factor;

            {
                let mut ds = download_state.lock().unwrap();
                let stage = match stage_index {
                    Some(idx) => ds.stages.get_mut(idx),
                    None => ds.stages.last_mut(),
                };
                if let Some(stage) = stage {
                    stage.progress = current / 100.0;
                }
            }
            // 广播到 WS，让前端实时看到伪进度动画
            crate::commands::version::download::broadcast_current(&app_state);
        }
    });

    stop
}
