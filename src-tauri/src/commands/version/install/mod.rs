//! 版本安装命令（MC + 多加载器合并安装）
//!
//! 按关注点拆分为 4 个子模块：
//! - `loader_helpers`  install_single_loader + start_progress_ticker
//! - `version_naming`  resolve_unique_instance_name + find_loader_version_dir
//! - `cleanup`         cleanup_failed_install
//! - `mod.rs`          install_merged 主流程

mod cleanup;
mod loader_helpers;
mod version_naming;

use crate::minecraft::download::{self, types as download_types};
use crate::minecraft::isolation::{self, IsolationMode};
use crate::minecraft::loaders;
use crate::minecraft::version::{setup::VersionSetup, state::VersionType};
use crate::state::{AppState, DownloadStage, StageStatus};
use crate::{log_error, log_info, log_warn};
use std::sync::Arc;
use tauri::{Emitter, State};

use super::{sanitize_mc_version, sanitize_version_id};
use loader_helpers::install_single_loader;
use version_naming::{find_loader_version_dir, resolve_unique_instance_name};

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

    // 重置取消/暂停信号（确保每次安装都是干净状态）
    state
        .download_cancel_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
    state
        .download_pause_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);

    let base_name = instance_name.unwrap_or_else(|| mc_version.clone());
    let instance = resolve_unique_instance_name(&game_dir, &base_name);

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

    // 设置下载状态（统一方法：append_stages 追加，保留整合包 stages）
    //  - 整合包安装流程的 4 个阶段保留显示（前端按 group 分组折叠展开）
    //  - download_version_full 的 stage_callback(0..4) 是相对偏移，实际索引 = stage_offset + stage_index
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

    // progress callback：统一用 sync_stage_from_progress 同步 GlobalProgress 到 download_state
    // 速度/字节累加由统一方法处理，不再在此维护 speed_window
    let progress_callback = Arc::new(move |progress: download_types::GlobalProgress| {
        let mut ds = state_clone.lock().unwrap();
        ds.is_active = progress.is_active;
        let idx = ds.current_stage_index;
        ds.sync_stage_from_progress(
            idx,
            progress.downloaded_bytes,
            progress.total_bytes,
            progress.completed_files,
            progress.total_files,
            progress.current_speed,
        );
    });

    // Stage callback：统一用 set_current_stage 切换阶段（自动把前一阶段标记 Finished）
    let state_for_stage = state.download_state.clone();
    let stage_callback = Arc::new(move |stage_index: usize, _stage_name: &str| {
        let actual_index = stage_offset + stage_index;
        let mut ds = state_for_stage.lock().unwrap();
        ds.set_current_stage(actual_index);
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
        Some(state.download_cancel_flag.clone()),
        Some(state.download_pause_flag.clone()),
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

    // 检查取消信号：MC 下载阶段被取消则直接返回
    if state
        .download_cancel_flag
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        log_warn!("[Merged] 下载已被用户取消");
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(1);
        return Err("下载已取消".to_string());
    }

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

    // 检查取消信号：加载器安装前
    let cancelled = state
        .download_cancel_flag
        .load(std::sync::atomic::Ordering::Relaxed);

    // 安装各加载器（使用辅助函数消除重复代码）
    // 注意：第一个加载器会添加阶段，后续加载器只更新阶段
    if !cancelled {
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
    } // if !cancelled

    // 如果被取消，直接返回
    if cancelled {
        log_warn!("[Merged] 用户取消安装，跳过加载器安装");
        let mut ds = state.download_state.lock().unwrap();
        ds.mark_failed(1);
        return Err("下载已取消".to_string());
    }

    // 阶段完成标记延迟到所有任务（含 Fabric API）完成后
    // install-complete 事件也延迟到 mark_complete() 之后，避免前端提前关闭进度面板

    if loader_errors.is_empty() {
        // 安装成功后，如果有加载器，删除原版文件夹（参考 PCL2：只保留加载器版本文件夹）
        // 但需要先确保加载器版本的JSON已合并原版信息
        if has_any_loader {
            // 找到加载器版本目录（如 forge-26.2-65.0.3）
            let versions_dir = game_dir.join("versions");
            if let Some(dir_name) = find_loader_version_dir(&versions_dir, &mc_version) {
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
            find_loader_version_dir(&versions_dir, &mc_version)
                .unwrap_or_else(|| mc_version.clone())
        } else {
            mc_version.clone()
        };

        // 如果用户自定义了名称，需要重命名版本目录和修改 JSON
        let actual_version_id = if instance != final_version_id {
            log_info!("[Merged] 重命名版本: {} -> {}", final_version_id, instance);
            let old_dir = game_dir.join("versions").join(&final_version_id);
            let new_dir = game_dir.join("versions").join(&instance);

            // 重命名目录：如果目标已存在（整合包半成品目录），改为合并文件
            let rename_ok = if new_dir.exists() {
                // 整合包半成品目录：移动 MC 本体相关文件（jar/json/natives 等）到目标目录
                log_info!(
                    "[Merged] 目标目录已存在（整合包半成品），改为合并文件: {} -> {}",
                    old_dir.display(),
                    new_dir.display()
                );
                let mut merged_ok = true;
                if let Ok(entries) = std::fs::read_dir(&old_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let file_name = entry.file_name();
                        let target = new_dir.join(&file_name);
                        // 如果目标已存在同名文件（如 config 目录），跳过避免覆盖整合包配置
                        if target.exists() {
                            log_info!(
                                "[Merged] 跳过已存在的文件: {}",
                                target.display()
                            );
                            continue;
                        }
                        if let Err(e) = std::fs::rename(&path, &target) {
                            log_warn!("[Merged] 移动文件失败: {} -> {} : {}", path.display(), target.display(), e);
                            merged_ok = false;
                        }
                    }
                }
                // 删除空的 old_dir
                let _ = std::fs::remove_dir(&old_dir);
                merged_ok
            } else {
                std::fs::rename(&old_dir, &new_dir).is_ok()
            };

            if !rename_ok {
                log_warn!("[Merged] 重命名/合并目录失败");
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

        // ===== Fabric API 自动补充 =====
        // 参考 PCL2 PageDownloadInstall.xaml.vb FabricApi_Loaded + ModDownloadLib.vb McInstallLoader：
        // 安装 Fabric Loader 后自动下载最新兼容的 Fabric API 到 mods 目录
        if fabric_version.is_some() {
            // 检查取消信号：用户在加载器安装阶段取消
            if state
                .download_cancel_flag
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                log_warn!("[Merged] 用户取消安装，跳过 Fabric API");
            } else {
                log_info!("[Merged] 检测到 Fabric，开始自动补充 Fabric API");

            // 添加 Fabric API 安装阶段（参考 PCL2 的阶段管理）
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
            let effective_dir = isolation::get_effective_game_dir(
                &game_dir,
                &actual_version_id,
                mode_val,
                version_type,
            );
            let mods_dir = effective_dir.join("mods");
            std::fs::create_dir_all(&mods_dir).ok();

            // 查询兼容的 Fabric API 版本
            match loaders::fabric_api::list_versions(&mc_version).await {
                Ok(versions) if !versions.is_empty() => {
                    // 自动选择最新版本（列表已按发布日期降序排序）
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
                    let source_mode_val = {
                        let config = state.config.lock().await;
                        crate::minecraft::sources::DownloadSourceMode::from_str(&config.meta_source)
                    };

                    if let Err(e) = loaders::fabric_api::install(
                        &latest.download_url,
                        &latest.file_name,
                        &mods_dir,
                        latest.hash.as_deref(),
                        source_mode_val,
                        None,
                    )
                    .await
                    {
                        log_warn!("[Merged] Fabric API 安装失败（不阻断主流程）: {}", e);
                        // 标记阶段为失败但继续
                        let mut ds = state.download_state.lock().unwrap();
                        let idx = ds.stages.len() - 1;
                        ds.set_stage_status(idx, StageStatus::Failed, 0.0);
                    } else {
                        log_info!("[Merged] Fabric API 安装完成: {}", latest.file_name);
                        // 标记阶段完成
                        let mut ds = state.download_state.lock().unwrap();
                        let idx = ds.stages.len() - 1;
                        ds.set_stage_status(idx, StageStatus::Finished, 1.0);
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
            } // else (未取消)
        }

        // 所有任务完成（含 Fabric API），现在才标记整体完成
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.mark_complete();
        }

        // 安装完成事件：在 mark_complete() 之后发出，确保前端看到 is_complete=true 时进度面板已展示完毕
        let _ = app.emit(
            "install-complete",
            serde_json::json!({ "instance_name": instance }),
        );

        log_info!("[Merged] Install completed successfully");
        Ok(())
    } else {
        // 加载器安装失败，清理已下载的版本目录
        let error_msg = format!("部分加载器安装失败: {}", loader_errors.join(", "));
        log_warn!("[Merged] {}", error_msg);

        cleanup::cleanup_failed_install(&game_dir, &mc_version, fabric_version.as_deref());

        Err(error_msg)
    }
}
