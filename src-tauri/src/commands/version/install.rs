use crate::minecraft::download::{self, types as download_types};
use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::loaders;
use crate::minecraft::version::{setup::VersionSetup, state::VersionType};
use crate::state::{AppState, StageStatus};
use crate::{log_error, log_info, log_warn};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{Emitter, State};

use super::{sanitize_mc_version, sanitize_version_id};

/// 安装单个加载器的通用辅助函数
/// 如果阶段已存在（最后一个阶段是加载器安装），则更新它；否则添加新阶段
async fn install_single_loader(
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
fn start_progress_ticker(
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

    // 处理版本名称：如果已存在则追加后缀 (1), (2) 等
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let base_name = instance_name.unwrap_or_else(|| mc_version.clone());
    let instance = {
        let versions_dir = game_dir.join("versions");
        if !versions_dir.join(&base_name).exists() {
            base_name.clone()
        } else {
            // 版本名已存在，追加后缀
            let mut counter = 1;
            loop {
                let candidate = format!("{}({})", base_name, counter);
                if !versions_dir.join(&candidate).exists() {
                    break candidate;
                }
                counter += 1;
            }
        }
    };

    if instance != base_name {
        log_info!(
            "[Merged] Version name '{}' already exists, using '{}'",
            base_name,
            instance
        );
    }

    // 预添加加载器安装阶段（状态为 Waiting，让用户从一开始就看到）
    let has_any_loader = forge_version.is_some()
        || neoforge_version.is_some()
        || fabric_version.is_some()
        || optifine_version.is_some()
        || liteloader_version.is_some();

    // 设置下载状态
    // 重要：install_merged 启动时**追加**新阶段，不清空已有 stages
    //  - 整合包安装流程的 4 个阶段保留显示（前端按 group 分组折叠展开）
    //  - download_version_full 的 stage_callback(0..4) 是相对偏移，实际索引 = stage_offset + stage_index
    //  - stage_offset = 追加前 stages 长度
    let stage_offset;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = true;
        ds.is_complete = false;
        ds.global_speed = 0;
        ds.global_bytes_downloaded = 0;
        ds.global_bytes_total = 0;
        ds.error_code = 0;

        stage_offset = ds.stages.len();
        ds.current_stage_index = stage_offset;

        // 追加标准 MC 下载 5 阶段（与 download_version_full 的 stage_callback 索引对应）
        // 全部归入"MC本体安装"分组，加载器阶段也归入此分组
        ds.stages.extend([
            crate::state::DownloadStage::new_grouped("版本清单", 2.0, "MC本体安装"),
            crate::state::DownloadStage::new_grouped("版本信息", 3.0, "MC本体安装"),
            crate::state::DownloadStage::new_grouped("客户端", 5.0, "MC本体安装"),
            crate::state::DownloadStage::new_grouped("库文件", 15.0, "MC本体安装"),
            crate::state::DownloadStage::new_grouped("资源文件", 20.0, "MC本体安装"),
        ]);
        // 如果需要安装加载器，追加阶段（索引 = stage_offset + 5）
        if has_any_loader {
            ds.stages.push(crate::state::DownloadStage::new_grouped(
                "加载器安装",
                30.0,
                "MC本体安装",
            ));
        }
    }

    let config = state.config.lock().await;
    let mirror_url = config.mirror_url.clone();
    let max_threads = config.max_download_threads as usize;
    let chunk_count = config.chunk_count as usize;
    let speed_limit = config.max_download_speed;
    let source_mode =
        crate::minecraft::sources::DownloadSourceMode::from_str(&config.download_source);
    drop(config);

    let game_path = game_dir.as_path();
    let state_clone = state.download_state.clone();

    // 滑动窗口速度计算
    let speed_window = Arc::new(std::sync::Mutex::new(VecDeque::<(u64, Instant)>::new()));

    // Create progress callback (不发射事件，只更新状态)
    let sw = speed_window.clone();
    let progress_callback = Arc::new(move |progress: download_types::GlobalProgress| {
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
                    stage.progress =
                        (progress.downloaded_bytes as f64 / progress.total_bytes as f64).min(1.0);
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
                if window.len() > 10 {
                    window.pop_front();
                }
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
    // download_version_full 传 0..4 相对索引，加上 stage_offset 得到实际索引
    // 注意：第一次 stage_callback(0) 时 actual_index = stage_offset（即 MC 本体第一个阶段），
    // 此时 current_stage_index 已经是 stage_offset，不能把 stage_offset 自己标记为 Finished
    let state_for_stage = state.download_state.clone();
    let stage_callback = Arc::new(move |stage_index: usize, _stage_name: &str| {
        let actual_index = stage_offset + stage_index;
        let mut ds = state_for_stage.lock().unwrap();
        let prev = ds.current_stage_index;
        // 只有切换到新阶段（actual_index > prev）才把前一阶段标记为 Finished
        // 避免 stage_callback(0) 时误把 stage_offset（MC本体第一个）标记为 Finished
        if actual_index > prev && prev < ds.stages.len() {
            ds.stages[prev].status = StageStatus::Finished;
            ds.stages[prev].progress = 1.0;
        }
        ds.current_stage_index = actual_index;
        if actual_index < ds.stages.len() {
            ds.stages[actual_index].status = StageStatus::Loading;
            ds.stages[actual_index].progress = 0.0;
            ds.stages[actual_index].bytes_downloaded = 0;
            ds.stages[actual_index].bytes_total = 0;
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
    )
    .await
    .map_err(|e| {
        log_error!("Failed to download MC version: {}", e);
        e.to_string()
    })?;

    log_info!(
        "[Merged] MC download completed: libs {}/{}, assets {}/{}",
        result.libs_downloaded,
        result.libs_total,
        result.assets_downloaded,
        result.assets_total
    );

    // 标记前面的阶段完成
    {
        let mut ds = state.download_state.lock().unwrap();
        for stage in ds.stages.iter_mut() {
            if stage.status == StageStatus::Loading {
                stage.status = StageStatus::Finished;
                stage.progress = 1.0;
            }
        }
    }

    let mut loader_errors = Vec::new();

    // 安装各加载器（使用辅助函数消除重复代码）
    // 注意：第一个加载器会添加阶段，后续加载器只更新阶段
    if let Some(forge_ver) = forge_version {
        if let Err(e) = install_single_loader(
            &state,
            loaders::LoaderType::Forge,
            "Forge",
            &forge_ver,
            &mc_version,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    if let Some(neoforge_ver) = neoforge_version {
        if let Err(e) = install_single_loader(
            &state,
            loaders::LoaderType::NeoForge,
            "NeoForge",
            &neoforge_ver,
            &mc_version,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    if let Some(ref fabric_ver) = fabric_version {
        if let Err(e) = install_single_loader(
            &state,
            loaders::LoaderType::Fabric,
            "Fabric",
            fabric_ver,
            &mc_version,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    if let Some(optifine_ver) = optifine_version {
        if let Err(e) = install_single_loader(
            &state,
            loaders::LoaderType::OptiFine,
            "OptiFine",
            &optifine_ver,
            &mc_version,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    if let Some(liteloader_ver) = liteloader_version {
        if let Err(e) = install_single_loader(
            &state,
            loaders::LoaderType::LiteLoader,
            "LiteLoader",
            &liteloader_ver,
            &mc_version,
            &game_dir,
            mirror_url.as_deref(),
            max_threads,
            source_mode,
        )
        .await
        {
            loader_errors.push(e);
        }
    }

    // 完成：设置最终状态
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.is_active = false;
        ds.is_complete = true;
        for stage in ds.stages.iter_mut() {
            stage.status = StageStatus::Finished;
            stage.progress = 1.0;
        }
    }
    let _ = app.emit(
        "install-complete",
        serde_json::json!({ "instance_name": instance }),
    );

    if loader_errors.is_empty() {
        // 安装成功后，如果有加载器，删除原版文件夹（参考 PCL2：只保留加载器版本文件夹）
        // 但需要先确保加载器版本的JSON已合并原版信息
        if has_any_loader {
            // 找到加载器版本目录（如 forge-26.2-65.0.3）
            let versions_dir = game_dir.join("versions");
            if let Ok(entries) = std::fs::read_dir(&versions_dir) {
                for entry in entries.flatten() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    if dir_name.starts_with(&format!("{}-forge-", mc_version))
                        || dir_name.starts_with(&format!("{}-neoforge-", mc_version))
                        || (dir_name.starts_with("fabric-")
                            && dir_name.ends_with(&format!("-{}", mc_version)))
                        || dir_name.starts_with(&format!("{}-OptiFine", mc_version))
                        || dir_name.starts_with(&format!("{}-LiteLoader", mc_version))
                    {
                        // 合并加载器版本的JSON（删除inheritsFrom）
                        let loader_json_path = versions_dir
                            .join(&dir_name)
                            .join(format!("{}.json", dir_name));
                        if loader_json_path.exists() {
                            if let Ok(content) = std::fs::read_to_string(&loader_json_path) {
                                if let Ok(json) =
                                    serde_json::from_str::<serde_json::Value>(&content)
                                {
                                    if json.get("inheritsFrom").is_some() {
                                        // 合并原版JSON
                                        if let Ok(merged) = crate::minecraft::version::json_merge::merge_version_json(&json, &game_dir) {
                                            if let Ok(new_content) = serde_json::to_string_pretty(&merged) {
                                                let _ = std::fs::write(&loader_json_path, new_content);
                                                log_info!("[Merged] 已合并JSON并删除inheritsFrom: {}", dir_name);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                }
            }

            // 删除原版目录
            let mc_version_dir = game_dir.join("versions").join(&mc_version);
            if mc_version_dir.exists() {
                match std::fs::remove_dir_all(&mc_version_dir) {
                    Ok(_) => log_info!("[Merged] 已删除原版目录: {}", mc_version_dir.display()),
                    Err(e) => log_warn!("[Merged] 删除原版目录失败: {}", e),
                }
            }
        }

        // 确定最终的版本目录名
        let final_version_id = if has_any_loader {
            let versions_dir = game_dir.join("versions");
            let mut found_id = None;
            if let Ok(entries) = std::fs::read_dir(&versions_dir) {
                for entry in entries.flatten() {
                    let dir_name = entry.file_name().to_string_lossy().to_string();
                    if dir_name.starts_with(&format!("{}-forge-", mc_version))
                        || dir_name.starts_with(&format!("{}-neoforge-", mc_version))
                        || (dir_name.starts_with("fabric-")
                            && dir_name.ends_with(&format!("-{}", mc_version)))
                        || dir_name.starts_with(&format!("{}-OptiFine", mc_version))
                        || dir_name.starts_with(&format!("{}-LiteLoader", mc_version))
                    {
                        found_id = Some(dir_name);
                        break;
                    }
                }
            }
            found_id.unwrap_or_else(|| mc_version.clone())
        } else {
            mc_version.clone()
        };

        // 如果用户自定义了名称，需要重命名版本目录和修改 JSON
        let actual_version_id = if instance != final_version_id {
            log_info!("[Merged] 重命名版本: {} -> {}", final_version_id, instance);
            let old_dir = game_dir.join("versions").join(&final_version_id);
            let new_dir = game_dir.join("versions").join(&instance);

            // 重命名目录
            if let Err(e) = std::fs::rename(&old_dir, &new_dir) {
                log_warn!("[Merged] 重命名目录失败: {}", e);
                final_version_id.clone()
            } else {
                // 重命名 JSON 文件
                let old_json = new_dir.join(format!("{}.json", final_version_id));
                let new_json = new_dir.join(format!("{}.json", instance));
                if old_json.exists() {
                    if let Err(e) = std::fs::rename(&old_json, &new_json) {
                        log_warn!("[Merged] 重命名 JSON 失败: {}", e);
                    }
                }

                // 修改 JSON 中的 id 字段
                if new_json.exists() {
                    if let Ok(content) = std::fs::read_to_string(&new_json) {
                        if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                            json["id"] = serde_json::Value::String(instance.clone());
                            if let Ok(new_content) = serde_json::to_string_pretty(&json) {
                                let _ = std::fs::write(&new_json, new_content);
                            }
                        }
                    }
                }

                // 重命名 JAR 文件
                let old_jar = new_dir.join(format!("{}.jar", final_version_id));
                let new_jar = new_dir.join(format!("{}.jar", instance));
                if old_jar.exists() {
                    let _ = std::fs::rename(&old_jar, &new_jar);
                }

                instance.clone()
            }
        } else {
            final_version_id.clone()
        };

        let version_dir = game_dir.join("versions").join(&actual_version_id);

        // 保存 setup.ini（参考 PCL2：记录版本元数据）
        // 注意：前面的加载器安装代码已经 move 了这些变量，这里通过版本目录名推断
        let version_type = if actual_version_id.contains("-forge-") {
            VersionType::Forge
        } else if actual_version_id.contains("-neoforge-") {
            VersionType::NeoForge
        } else if actual_version_id.starts_with("fabric-") {
            VersionType::Fabric
        } else if actual_version_id.contains("-OptiFine") {
            VersionType::OptiFine
        } else if actual_version_id.contains("-LiteLoader") {
            VersionType::LiteLoader
        } else {
            VersionType::Release
        };

        let setup = VersionSetup::new(
            &mc_version,
            version_type,
            None, // Forge 版本号从目录名或 JSON 中提取
            None,
            None,
            None,
            None,
            None,
        );
        if let Err(e) = setup.save(&version_dir) {
            log_warn!("[Merged] 保存 setup.ini 失败: {}", e);
        } else {
            log_info!("[Merged] 已保存 setup.ini: {}", version_dir.display());
        }

        // 根据版本隔离设置创建隔离目录（参考 PCL2：安装时即创建）
        let isolation_mode = state.config.lock().await.isolation_mode;
        let mode = IsolationMode::from_u32(isolation_mode);
        if isolation::should_isolate(mode, version_type) {
            log_info!(
                "[Merged] 创建隔离目录: {} (模式: {}, 类型: {:?})",
                actual_version_id,
                isolation_mode,
                version_type
            );
            // 根据版本类型创建不同的目录结构
            let result = if version_type.is_modded() {
                isolation::ensure_modded_dirs(&version_dir)
            } else {
                isolation::ensure_isolated_dirs(&version_dir)
            };
            if let Err(e) = result {
                log_warn!("[Merged] 创建隔离目录失败: {}", e);
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
        // 注意：fabric 目录命名为 `fabric-{fabric_version}-{mc_version}`，
        // 之前用 `fabric-` 前缀过宽（会误删任意含 "fabric-" 的目录），
        // 改为仅在知道 fabric_version 时构造精确匹配。
        let mut loader_patterns = vec![
            format!("{}-forge-", mc_version),
            format!("{}-neoforge-", mc_version),
            format!("{}-LiteLoader", mc_version),
        ];
        if let Some(fv) = fabric_version.as_ref() {
            loader_patterns.push(format!("fabric-{}-{}", fv, mc_version));
        }

        if let Ok(entries) = std::fs::read_dir(&versions_dir) {
            for entry in entries.flatten() {
                let dir_name = entry.file_name().to_string_lossy().to_string();
                for pattern in &loader_patterns {
                    if dir_name.contains(pattern) {
                        match std::fs::remove_dir_all(entry.path()) {
                            Ok(_) => {
                                log_info!("[Merged] 已清理加载器目录: {}", entry.path().display())
                            }
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
