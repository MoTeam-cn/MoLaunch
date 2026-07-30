//! Fabric API 自动安装
//!
//! 安装 Fabric Loader 后自动下载最新兼容的 Fabric API 到 mods 目录。
//! 失败不阻断主流程，仅标记阶段为失败。

use crate::minecraft::download::config::DownloadManagerConfig;
use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::version::state::VersionType;
use crate::state::{AppState, DownloadStage, StageStatus};
use crate::{log_info, log_warn};

/// 自动安装 Fabric API（如果用户选择了 Fabric Loader）
///
/// - 添加 "安装 Fabric API" 阶段
/// - 查询兼容的最新 Fabric API 版本
/// - 下载到 mods 目录（考虑版本隔离）
/// - 更新阶段状态
pub(crate) async fn auto_install_fabric_api(
    state: &AppState,
    game_dir: &std::path::Path,
    mc_version: &str,
    actual_version_id: &str,
    version_type: VersionType,
) {
    // 检查取消信号
    if state
        .download_cancel_flag
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        log_warn!("[Merged] 用户取消安装，跳过 Fabric API");
        return;
    }

    log_info!("[Merged] 检测到 Fabric，开始自动补充 Fabric API");

    // 添加 Fabric API 安装阶段
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.append_stages(vec![DownloadStage::new_grouped(
            "安装 Fabric API",
            10.0,
            "MC本体安装",
        )]);
        let new_idx = ds.stages.len() - 1;
        ds.set_current_stage(new_idx);
    }

    // 获取 mods 目录（考虑版本隔离）
    let isolation_mode_val = state.config.lock().await.isolation_mode;
    let mode_val = IsolationMode::from_u32(isolation_mode_val);
    let effective_dir =
        isolation::get_effective_game_dir(game_dir, actual_version_id, mode_val, version_type);
    let mods_dir = effective_dir.join("mods");
    std::fs::create_dir_all(&mods_dir).ok();

    // 查询兼容的 Fabric API 版本
    match crate::minecraft::loaders::fabric_api::list_versions(mc_version).await {
        Ok(versions) if !versions.is_empty() => {
            let latest = &versions[0];
            log_info!(
                "[Merged] 自动选择 Fabric API: {} ({})",
                latest.version_number,
                latest.file_name
            );

            // 更新阶段名称为具体版本
            {
                let mut ds = state.download_state.lock().unwrap();
                let idx = ds.stages.len() - 1;
                ds.stages[idx].name = format!("Fabric API {}", latest.version_number);
            }

            // 下载安装
            let config = DownloadManagerConfig::from_state_for_meta(state).await;

            match crate::minecraft::loaders::fabric_api::install(
                &latest.download_url,
                &latest.file_name,
                &mods_dir,
                latest.hash.as_deref(),
                &config,
                None,
            )
            .await
            {
                Ok(_) => {
                    log_info!("[Merged] Fabric API 安装完成: {}", latest.file_name);
                    let mut ds = state.download_state.lock().unwrap();
                    let idx = ds.stages.len() - 1;
                    ds.set_stage_status(idx, StageStatus::Finished, 1.0);
                }
                Err(e) => {
                    log_warn!("[Merged] Fabric API 安装失败（不阻断主流程）: {}", e);
                    let mut ds = state.download_state.lock().unwrap();
                    let idx = ds.stages.len() - 1;
                    ds.set_stage_status(idx, StageStatus::Failed, 0.0);
                }
            }
        }
        Ok(_) => {
            log_warn!("[Merged] 未找到兼容 MC {} 的 Fabric API 版本", mc_version);
            let mut ds = state.download_state.lock().unwrap();
            let idx = ds.stages.len() - 1;
            ds.set_stage_status(idx, StageStatus::Finished, 1.0);
        }
        Err(e) => {
            log_warn!("[Merged] 查询 Fabric API 版本失败（不阻断主流程）: {}", e);
            let mut ds = state.download_state.lock().unwrap();
            let idx = ds.stages.len() - 1;
            ds.set_stage_status(idx, StageStatus::Finished, 1.0);
        }
    }
}
