//! install_merged 整合安装编排逻辑（原聚合入口 mod.rs 中的实现收敛于此）
//!
//! 编排流程：1.下载 MC 本体 → 2.批量安装加载器（stages.rs）→ 3.合并 JSON+重命名版本目录
//! （post_install.rs）→ 4.保存 setup.ini+创建隔离目录（setup_persist.rs）→ 5.自动安装
//! Fabric API（fabric_api.rs）。各阶段详细实现拆分到对应子模块，本文件只做编排。

use crate::minecraft::sources::DownloadSourceMode;
use crate::state::AppState;
use crate::{log_error, log_info, log_warn};
use tauri::{AppHandle, Emitter};

use super::super::{sanitize_mc_version, sanitize_version_id};
use super::cleanup::cleanup_failed_install;
use super::download::download_base_mc;
use super::fabric_api::auto_install_fabric_api;
use super::post_install::merge_and_rename_version;
use super::setup_persist::save_setup_and_create_isolation;
use super::stages::install_all_loaders;
use super::version_naming::resolve_unique_instance_name;

use super::super::list::detect_version_type_from_dir;

/// Merged install (MC + loader)
///
/// 注：已聚合为 `version_install_manager` IPC 入口，本函数由
/// `install_manager::dispatch` 反序列化参数后调用。
#[allow(clippy::too_many_arguments)]
pub async fn install_merged(
    app: &AppHandle,
    state: &AppState,
    mc_version: String,
    forge_version: Option<String>,
    neoforge_version: Option<String>,
    fabric_version: Option<String>,
    optifine_version: Option<String>,
    liteloader_version: Option<String>,
    instance_name: Option<String>,
) -> Result<(), String> {
    sanitize_mc_version(&mc_version)?;
    if let Some(ref name) = instance_name {
        sanitize_version_id(name)?;
    }
    log_info!(
        "Merged install: mc={}, forge={:?}, neoforge={:?}, fabric={:?}, optifine={:?}",
        mc_version,
        forge_version,
        neoforge_version,
        fabric_version,
        optifine_version
    );

    let game_dir = crate::state::resolve_game_dir_from_state(state).await;

    // 重置取消/暂停信号（确保每次安装都是干净状态）
    state
        .download_cancel_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);

    // 清空上一次下载的 stages（避免累积导致前端显示历史阶段）
    // 修复：之前 append_stages 会保留旧 stages，多次安装后 stages 数组越来越长
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.stages.clear();
        ds.current_stage_index = 0;
        ds.is_active = false;
        ds.is_complete = false;
        ds.global_speed = 0;
        ds.global_bytes_downloaded = 0;
        ds.global_bytes_total = 0;
        ds.error_code = 0;
        ds.version_name = instance_name.clone().unwrap_or_else(|| mc_version.clone());
    }

    let base_name = instance_name.unwrap_or_else(|| mc_version.clone());
    let instance = resolve_unique_instance_name(&game_dir, &base_name);
    if instance != base_name {
        log_info!(
            "[Merged] Version name '{}' already exists, using '{}'",
            base_name,
            instance
        );
    }

    let has_any_loader = forge_version.is_some()
        || neoforge_version.is_some()
        || fabric_version.is_some()
        || optifine_version.is_some()
        || liteloader_version.is_some();

    // 设置下载阶段（append_stages 追加，保留整合包 stages）
    let stage_offset;
    let mut new_stages = vec![
        crate::state::DownloadStage::new_grouped("版本清单", 2.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("版本信息", 3.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("客户端", 5.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("库文件", 15.0, "MC本体安装"),
        crate::state::DownloadStage::new_grouped("资源文件", 20.0, "MC本体安装"),
    ];
    if has_any_loader {
        new_stages.push(crate::state::DownloadStage::new_grouped(
            "加载器安装",
            30.0,
            "MC本体安装",
        ));
    }
    {
        let mut ds = state.download_state.lock().unwrap();
        stage_offset = ds.append_stages(new_stages);
    }

    // 读取下载相关配置
    let (mirror_url, loader_source_mode) = crate::state::resolve_mirror_and_source(state).await;
    let (max_threads, download_source_mode) = {
        let config = state.config.lock().await;
        (
            config.download.max_threads as usize,
            DownloadSourceMode::from_str(&config.download.source),
        )
    };

    // Step 1: 下载原版 MC（失败清理/取消检查/阶段标记已封装在 download_base_mc）
    download_base_mc(
        state,
        app,
        &mc_version,
        game_dir.as_path(),
        mirror_url.as_deref(),
        download_source_mode,
        stage_offset,
        fabric_version.as_deref(),
    )
    .await?;

    // Step 2: 安装加载器
    let cancelled = state
        .download_cancel_flag
        .load(std::sync::atomic::Ordering::Relaxed);

    let loader_errors = if has_any_loader && !cancelled {
        install_all_loaders(
            state,
            &mc_version,
            &game_dir,
            &forge_version,
            &neoforge_version,
            &fabric_version,
            &optifine_version,
            &liteloader_version,
            mirror_url.as_deref(),
            max_threads,
            loader_source_mode,
        )
        .await
    } else {
        Vec::new()
    };

    if cancelled {
        log_warn!("[Merged] 用户取消安装，跳过加载器安装");
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(1);
        return Err("下载已取消".to_string());
    }

    if !loader_errors.is_empty() {
        let msg = format!("部分加载器安装失败: {}", loader_errors.join("; "));
        log_error!("[Merged] {}", msg);
        // 加载器安装失败：清理已下载的 MC 本体和加载器目录
        cleanup_failed_install(&game_dir, &mc_version, fabric_version.as_deref());
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(1);
        let _ = app.emit(
            "install-merged-progress",
            serde_json::json!({ "stage": "failed", "message": msg }),
        );
        return Err(msg);
    }

    // Step 3: 安装后处理（JSON 合并 + 重命名）
    let actual_version_id =
        merge_and_rename_version(&game_dir, &mc_version, &instance, has_any_loader);

    let version_type = detect_version_type_from_dir(&game_dir, &actual_version_id);

    // Step 4: 保存 setup.ini + 创建隔离目录
    save_setup_and_create_isolation(
        state,
        &game_dir,
        &actual_version_id,
        &mc_version,
        version_type,
    )
    .await;

    // Step 5: 自动安装 Fabric API（仅 Fabric 用户）
    if fabric_version.is_some() {
        auto_install_fabric_api(
            state,
            &game_dir,
            &mc_version,
            &actual_version_id,
            version_type,
        )
        .await;
    }

    // 完成
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_complete();
    }
    // 广播最终完成状态（WS 推送 is_complete=true，前端据此触发 finishDownload）
    {
        let ds = state.download_state.lock().unwrap();
        let snapshot = super::super::download::build_snapshot(&ds, &ds.version_name, false);
        let _ = state.progress_tx.send(snapshot);
    }
    let _ = app.emit(
        "install-merged-progress",
        serde_json::json!({ "stage": "completed", "version_id": actual_version_id }),
    );
    log_info!("[Merged] install_merged 完成: {}", actual_version_id);
    Ok(())
}
