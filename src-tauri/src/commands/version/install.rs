use crate::{log_error, log_info, log_warn};
use crate::minecraft::download::{self, manager as download_manager};
use crate::minecraft::loaders;
use crate::state::{AppState, StageStatus};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{Emitter, State};

/// 启动进度模拟器：缓慢上涨进度条，直到 stop 信号为 true
/// - 从 `start` 开始，每 500ms 增长一小段，上限 `cap`
/// - 返回 stop 信号，设为 true 即停止模拟
fn start_progress_ticker(
    state: Arc<std::sync::Mutex<crate::state::DownloadState>>,
    start: f64,
    cap: f64,
) -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();

    tokio::spawn(async move {
        let mut current = start;
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
        interval.tick().await; // 跳过第一次立即触发

        while !stop_clone.load(Ordering::Relaxed) {
            interval.tick().await;
            if stop_clone.load(Ordering::Relaxed) {
                break;
            }
            // 每次增长 1%~3%，越接近 cap 越慢
            let remaining = cap - current;
            if remaining <= 0.0 {
                break;
            }
            let step = (remaining * 0.05).clamp(0.5, 3.0);
            current = (current + step).min(cap);

            let mut ds = state.lock().unwrap();
            if ds.stages.len() > 5 {
                ds.stages[5].progress = current / 100.0;
            }
        }
    });

    stop
}

/// Merged install (MC + loader)
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn install_merged(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    mc_version: String,
    forge_version: Option<String>,
    neoforge_version: Option<String>,
    fabric_version: Option<String>,
    optifine_version: Option<String>,
    liteloader_version: Option<String>,
    instance_name: Option<String>,
) -> Result<(), String> {
    log_info!("Merged install: mc={}, forge={:?}, neoforge={:?}, fabric={:?}, optifine={:?}",
        mc_version, forge_version, neoforge_version, fabric_version, optifine_version);

    // 设置下载状态
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = true;
        ds.is_complete = false;
        ds.current_stage_index = 0;
        ds.global_speed = 0;
        ds.global_bytes_downloaded = 0;
        ds.global_bytes_total = 0;
        ds.error_code = 0;
        // 重置所有阶段
        for stage in ds.stages.iter_mut() {
            stage.progress = 0.0;
            stage.status = StageStatus::Waiting;
            stage.bytes_downloaded = 0;
            stage.bytes_total = 0;
        }
    }

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    let chunk_count = config.chunk_count as usize;
    let speed_limit = config.max_download_speed;
    let source_mode = crate::minecraft::sources::DownloadSourceMode::from_str(&config.download_source);
    drop(config);

    let game_path = game_dir.as_path();
    let state_clone = state.download_state.clone();

    // 滑动窗口速度计算
    let speed_window = Arc::new(std::sync::Mutex::new(VecDeque::<(u64, Instant)>::new()));

    // Create progress callback (不发射事件，只更新状态)
    let sw = speed_window.clone();
    let progress_callback = Arc::new(move |progress: download_manager::GlobalProgress| {
        {
            let mut ds = state_clone.lock().unwrap();
            ds.is_active = progress.is_active;

            let idx = ds.current_stage_index;
            if idx < ds.stages.len() {
                let stage = &mut ds.stages[idx];
                stage.bytes_downloaded = progress.downloaded_bytes;
                stage.bytes_total = progress.total_bytes;
                stage.files_downloaded = progress.completed_files;
                stage.files_total = progress.total_files;
                if progress.total_bytes > 0 {
                    stage.progress = (progress.downloaded_bytes as f64 / progress.total_bytes as f64).min(1.0);
                } else {
                    // 如果 total_bytes 为 0（全部跳过），设为完成
                    stage.progress = 1.0;
                }
                stage.status = StageStatus::Loading;
            }

            // 从所有阶段计算全局进度
            let mut total_downloaded = 0u64;
            let mut total_size = 0u64;
            for stage in &ds.stages {
                // 只累加未完成或进行中的阶段，跳过等待中的阶段
                if stage.status == StageStatus::Finished || stage.status == StageStatus::Loading {
                    total_downloaded += stage.bytes_downloaded;
                    total_size += stage.bytes_total;
                }
            }
            ds.global_bytes_downloaded = total_downloaded;
            ds.global_bytes_total = total_size;

            {
                let mut window = sw.lock().unwrap();
                window.push_back((ds.global_bytes_downloaded, Instant::now()));
                if window.len() > 10 { window.pop_front(); }
                if window.len() >= 2 {
                    let (first_bytes, first_time) = window.front().unwrap();
                    let (last_bytes, last_time) = window.back().unwrap();
                    let bytes_diff = last_bytes.saturating_sub(*first_bytes);
                    let time_diff = last_time.duration_since(*first_time).as_secs_f64();
                    if time_diff > 0.0 {
                        ds.global_speed = (bytes_diff as f64 / time_diff) as u64;
                    }
                }
            }
        }
    });

    // Stage callback (更新阶段状态)
    let state_for_stage = state.download_state.clone();
    let stage_callback = Arc::new(move |stage_index: usize, _stage_name: &str| {
        let mut ds = state_for_stage.lock().unwrap();
        if ds.current_stage_index < ds.stages.len() && stage_index > 0 {
            let prev = ds.current_stage_index;
            if prev < ds.stages.len() {
                ds.stages[prev].status = StageStatus::Finished;
                ds.stages[prev].progress = 1.0;
            }
        }
        ds.current_stage_index = stage_index;
        if stage_index < ds.stages.len() {
            ds.stages[stage_index].status = StageStatus::Loading;
            ds.stages[stage_index].progress = 0.0;
            ds.stages[stage_index].bytes_downloaded = 0;
            ds.stages[stage_index].bytes_total = 0;
        }
    });

    // Step 1: 下载原版 MC（直接调用内部函数，不触发事件）
    log_info!("[Merged] Downloading base MC version: {}", mc_version);
    let result = download::download_version_full(
        &mc_version,
        game_path,
        mirror_url.as_deref(),
        max_threads,
        chunk_count,
        speed_limit,
        source_mode,
        Some(progress_callback),
        Some(stage_callback),
    ).await.map_err(|e| {
        log_error!("Failed to download MC version: {}", e);
        e.to_string()
    })?;

    log_info!("[Merged] MC download completed: libs {}/{}, assets {}/{}",
        result.libs_downloaded, result.libs_total,
        result.assets_downloaded, result.assets_total);

    // Step 2: 安装加载器（更新阶段状态）
    {
        let mut ds = state.download_state.lock().unwrap();
        // 标记前面的阶段完成
        for stage in ds.stages.iter_mut() {
            if stage.status == StageStatus::Loading {
                stage.status = StageStatus::Finished;
                stage.progress = 1.0;
            }
        }
    }

    let mut loader_errors = Vec::new();
    let has_any_loader = forge_version.is_some() || neoforge_version.is_some() || 
                         fabric_version.is_some() || optifine_version.is_some() || 
                         liteloader_version.is_some();

    if let Some(forge_ver) = forge_version {
        // 更新阶段：Forge 安装
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.current_stage_index = 5;
            if ds.stages.len() > 5 {
                ds.stages[5].name = format!("安装 Forge {}", forge_ver);
                ds.stages[5].status = StageStatus::Loading;
                ds.stages[5].progress = 0.0;
            }
        }
        log_info!("[Merged] Installing Forge {}", forge_ver);
        
        // 创建进度回调，更新阶段进度
        let state_for_progress = state.download_state.clone();
        let progress_callback = Arc::new(move |progress: f64| {
            let mut ds = state_for_progress.lock().unwrap();
            if ds.stages.len() > 5 {
                ds.stages[5].progress = progress / 100.0;
            }
        });
        
        // 启动进度模拟器，防止进度卡住
        let ticker_stop = start_progress_ticker(state.download_state.clone(), 5.0, 95.0);
        
        match loaders::install_loader(
            loaders::LoaderType::Forge,
            &mc_version,
            &forge_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            Some(progress_callback),
            source_mode,
        ).await {
            Ok(_) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_info!("[Merged] Forge {} installed successfully", forge_ver);
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Finished;
                    ds.stages[5].progress = 1.0;
                }
            }
            Err(e) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_error!("[Merged] Failed to install Forge: {}", e);
                loader_errors.push(format!("Forge: {}", e));
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Failed;
                }
            }
        }
    }

    if let Some(neoforge_ver) = neoforge_version {
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.current_stage_index = 5;
            if ds.stages.len() > 5 {
                ds.stages[5].name = format!("安装 NeoForge {}", neoforge_ver);
                ds.stages[5].status = StageStatus::Loading;
                ds.stages[5].progress = 0.0;
            }
        }
        log_info!("[Merged] Installing NeoForge {}", neoforge_ver);
        let ticker_stop = start_progress_ticker(state.download_state.clone(), 5.0, 95.0);
        match loaders::install_loader(
            loaders::LoaderType::NeoForge,
            &mc_version,
            &neoforge_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            None,
            source_mode,
        ).await {
            Ok(_) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_info!("[Merged] NeoForge {} installed successfully", neoforge_ver);
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Finished;
                    ds.stages[5].progress = 1.0;
                }
            }
            Err(e) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_error!("[Merged] Failed to install NeoForge: {}", e);
                loader_errors.push(format!("NeoForge: {}", e));
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Failed;
                }
            }
        }
    }

    if let Some(fabric_ver) = fabric_version {
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.current_stage_index = 5;
            if ds.stages.len() > 5 {
                ds.stages[5].name = format!("安装 Fabric {}", fabric_ver);
                ds.stages[5].status = StageStatus::Loading;
                ds.stages[5].progress = 0.0;
            }
        }
        log_info!("[Merged] Installing Fabric {}", fabric_ver);
        let ticker_stop = start_progress_ticker(state.download_state.clone(), 5.0, 95.0);
        match loaders::install_loader(
            loaders::LoaderType::Fabric,
            &mc_version,
            &fabric_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            None,
            source_mode,
        ).await {
            Ok(_) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_info!("[Merged] Fabric {} installed successfully", fabric_ver);
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Finished;
                    ds.stages[5].progress = 1.0;
                }
            }
            Err(e) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_error!("[Merged] Failed to install Fabric: {}", e);
                loader_errors.push(format!("Fabric: {}", e));
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Failed;
                }
            }
        }
    }

    if let Some(optifine_ver) = optifine_version {
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.current_stage_index = 5;
            if ds.stages.len() > 5 {
                ds.stages[5].name = format!("安装 OptiFine {}", optifine_ver);
                ds.stages[5].status = StageStatus::Loading;
                ds.stages[5].progress = 0.0;
            }
        }
        log_info!("[Merged] Installing OptiFine {}", optifine_ver);
        let ticker_stop = start_progress_ticker(state.download_state.clone(), 5.0, 95.0);
        match loaders::install_loader(
            loaders::LoaderType::OptiFine,
            &mc_version,
            &optifine_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            None,
            source_mode,
        ).await {
            Ok(_) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_info!("[Merged] OptiFine {} installed successfully", optifine_ver);
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Finished;
                    ds.stages[5].progress = 1.0;
                }
            }
            Err(e) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_error!("[Merged] Failed to install OptiFine: {}", e);
                loader_errors.push(format!("OptiFine: {}", e));
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Failed;
                }
            }
        }
    }

    if let Some(liteloader_ver) = liteloader_version {
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.current_stage_index = 5;
            if ds.stages.len() > 5 {
                ds.stages[5].name = format!("安装 LiteLoader {}", liteloader_ver);
                ds.stages[5].status = StageStatus::Loading;
                ds.stages[5].progress = 0.0;
            }
        }
        log_info!("[Merged] Installing LiteLoader {}", liteloader_ver);
        let ticker_stop = start_progress_ticker(state.download_state.clone(), 5.0, 95.0);
        match loaders::install_loader(
            loaders::LoaderType::LiteLoader,
            &mc_version,
            &liteloader_ver,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            None,
            source_mode,
        ).await {
            Ok(_) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_info!("[Merged] LiteLoader {} installed successfully", liteloader_ver);
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Finished;
                    ds.stages[5].progress = 1.0;
                }
            }
            Err(e) => {
                ticker_stop.store(true, Ordering::Relaxed);
                log_error!("[Merged] Failed to install LiteLoader: {}", e);
                loader_errors.push(format!("LiteLoader: {}", e));
                let mut ds = state.download_state.lock().unwrap();
                if ds.stages.len() > 5 {
                    ds.stages[5].status = StageStatus::Failed;
                }
            }
        }
    }

    // 完成：设置最终状态
    let instance = instance_name.unwrap_or_else(|| mc_version.clone());
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = false;
        ds.is_complete = true;
        for stage in ds.stages.iter_mut() {
            stage.status = StageStatus::Finished;
            stage.progress = 1.0;
        }
    }
    let _ = app.emit("install-complete", serde_json::json!({ "instance_name": instance }));

    if loader_errors.is_empty() {
        // 安装成功后，如果有加载器，删除原版文件夹（参考 PCL2：只保留加载器版本文件夹）
        if has_any_loader {
            let mc_version_dir = game_dir.join("versions").join(&mc_version);
            if mc_version_dir.exists() {
                match std::fs::remove_dir_all(&mc_version_dir) {
                    Ok(_) => log_info!("[Merged] 已删除原版目录: {}", mc_version_dir.display()),
                    Err(e) => log_warn!("[Merged] 删除原版目录失败: {}", e),
                }
            }
        }
        log_info!("[Merged] Install completed successfully");
        Ok(())
    } else {
        // 加载器安装失败，清理已下载的版本目录
        let error_msg = format!("部分加载器安装失败: {}", loader_errors.join(", "));
        log_warn!("[Merged] {}", error_msg);
        
        let versions_dir = game_dir.join("versions");
        
        // 删除原版目录
        let mc_version_dir = versions_dir.join(&mc_version);
        if mc_version_dir.exists() {
            match std::fs::remove_dir_all(&mc_version_dir) {
                Ok(_) => log_info!("[Merged] 已清理原版目录: {}", mc_version_dir.display()),
                Err(e) => log_error!("[Merged] 清理原版目录失败: {}", e),
            }
        }
        
        // 删除加载器创建的目录（如 1.20.1-forge-47.4.20）
        let loader_patterns = vec![
            format!("{}-forge-", mc_version),
            format!("{}-neoforge-", mc_version),
            format!("fabric-"),
            format!("{}-LiteLoader", mc_version),
        ];
        
        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                for pattern in &loader_patterns {
                    if dir_name.contains(pattern) {
                        match std::fs::remove_dir_all(entry.path()) {
                            Ok(_) => log_info!("[Merged] 已清理加载器目录: {}", entry.path().display()),
                            Err(e) => log_error!("[Merged] 清理加载器目录失败: {}", e),
                        }
                        break;
                    }
                }
            }
        }
        
        Err(error_msg)
    }
}
