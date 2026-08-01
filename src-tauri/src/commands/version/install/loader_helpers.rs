//! 加载器安装辅助函数
//!
//! - `install_single_loader` 通用加载器安装（更新/添加 stage + 调用 loaders::install_loader）
//! - `start_progress_ticker` 分段线性伪进度（在加载器安装期间给用户视觉反馈）

use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::loaders;
use crate::state::{AppState, StageStatus};
use crate::{log_error, log_info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Forge/NeoForge 伪进度曲线：cap 95%，总时长 ~120 秒
///
/// 0→30% @1.5%/s = 20秒
/// 30→60% @1.0%/s = 30秒
/// 60→85% @0.6%/s = 41秒
/// 85→95% @0.3%/s = 33秒
///
/// 永不到 100%，真正完成后由调用方跳 100%。前端在 95% 后用伪进度补丁继续小数点上涨。
const FORGE_TICKER: &[(f64, f64)] = &[(30.0, 1.5), (60.0, 1.0), (85.0, 0.6), (95.0, 0.3)];

/// Fabric 伪进度曲线：cap 95%，总时长 ~64 秒
///
/// 0→40% @3%/s = 13秒
/// 40→70% @2%/s = 15秒
/// 70→90% @1%/s = 20秒
/// 90→95% @0.3%/s = 16秒
const FABRIC_TICKER: &[(f64, f64)] = &[(40.0, 3.0), (70.0, 2.0), (90.0, 1.0), (95.0, 0.3)];

/// 整合包解析伪进度曲线：0→90% @5%/s
///
/// 解析完成前缓慢上涨，解析完成后 stop 并跳 100%
const PARSE_TICKER: &[(f64, f64)] = &[(90.0, 5.0)];

/// 分段线性曲线进度计算
///
/// `segments` 为 `[(cap, speed_per_sec), ...]`，表示每个分段的目标值和速度
/// 例如 `[(50.0, 4.0), (80.0, 3.0), (100.0, 1.0)]` 表示：
/// - 0% → 50%：每秒 4%（12.5 秒）
/// - 50% → 80%：每秒 3%（10 秒）
/// - 80% → 100%：每秒 1%（20 秒）
fn compute_linear_progress(elapsed_secs: f64, segments: &[(f64, f64)]) -> f64 {
    let mut current = 0.0;
    let mut remaining_time = elapsed_secs;

    for (cap, speed) in segments {
        let segment_width = cap - current;
        let segment_duration = if *speed > 0.0 {
            segment_width / speed
        } else {
            0.0
        };
        if remaining_time <= segment_duration {
            return current + remaining_time * speed;
        }
        current = *cap;
        remaining_time -= segment_duration;
    }

    current
}

#[allow(clippy::too_many_arguments)]
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
    _max_threads: usize,
    _source_mode: crate::minecraft::sources::DownloadSourceMode,
) -> Result<(), String> {
    // 检查是否已有加载器安装阶段（通过名称判断）
    let has_loader_stage = {
        let ds = state.download_state.lock().unwrap();
        ds.stages
            .last()
            .is_some_and(|s| s.name.contains("安装") || s.name.contains("加载器"))
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

    // 根据加载器类型选择伪进度曲线
    // Forge/NeoForge 安装较慢（含 Java 进程），用慢曲线；Fabric 纯 HTTP 下载，用快曲线
    let ticker_segments: &'static [(f64, f64)] = match loader_type {
        loaders::LoaderType::Fabric => FABRIC_TICKER,
        _ => FORGE_TICKER,
    };

    // 启动进度模拟器（分段线性曲线）
    // 统一由 ticker 管理伪进度，加载器 install 内部不需要手写 progress_callback
    let ticker_stop = start_progress_ticker(state, None, ticker_segments);

    // 构造下载配置（读 meta_source，保持 installer 历史行为）
    let config = DownloadManagerConfig::from_state_for_meta(state).await;

    // 安装加载器（progress_callback 传 None，进度由 ticker 统一管理）
    match loaders::install_loader(
        loader_type,
        mc_version,
        loader_version,
        game_dir,
        mirror_url,
        None,
        &config,
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

/// 启动进度模拟器：分段线性曲线上涨，直到 stop 信号为 true
///
/// `segments` 为 `[(cap, speed_per_sec), ...]`，例如 `[(50.0, 4.0), (80.0, 3.0), (100.0, 1.0)]`：
/// - 0% → 50%：每秒 4%（12.5 秒）
/// - 50% → 80%：每秒 3%（10 秒）
/// - 80% → 100%：每秒 1%（20 秒）
///
/// - `stage_index` 为 None 时更新最后一个阶段（兼容加载器安装场景）
/// - 每次更新后广播到 WS，让前端实时看到伪进度动画
pub(crate) fn start_progress_ticker(
    state: &AppState,
    stage_index: Option<usize>,
    segments: &'static [(f64, f64)],
) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let download_state = state.download_state.clone();
    let app_state = state.clone();

    tokio::spawn(async move {
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
            let current = compute_linear_progress(elapsed_secs, segments);

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

/// 整合包解析阶段的伪进度曲线（供 modpack.rs 调用）
///
/// 0→90% @5%/s，解析完成后 stop 并跳 100%
pub(crate) fn start_parse_ticker(state: &AppState, stage_index: usize) -> Arc<AtomicBool> {
    start_progress_ticker(state, Some(stage_index), PARSE_TICKER)
}
