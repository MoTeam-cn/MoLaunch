//! 社区资源下载安装命令
//!
//! 参考 PCL2 PageDownloadCompDetail Save_Click / Install_Click
//! 下载资源文件到指定版本目录
//!
//! 模块结构：
//! - types.rs: DownloadRequest / DownloadResult / CommunityDownloadProgress /
//!   InstallModpackRequest / ModpackFormat / InstallModpackResult / ModpackInfo 数据类型
//! - helpers.rs: format_bytes / apply_filename_format / resolve_install_dir /
//!   parse_cf_loader_id / parse_mr_loader / extract_mr_project_id / construct_cf_edge_url 纯函数
//! - concurrent.rs: download_files_concurrent / extract_overrides / detect_modpack_format
//! - curseforge.rs: CF 整合包 manifest 数据结构 + install_cf_mods
//! - modrinth.rs: MR 整合包 index 数据结构 + install_mr_files
//! - modpack_stages.rs: install_modpack 阶段辅助（download_modpack_archive + parse_modpack_info）
//! - mod.rs: 所有 #[tauri::command] 命令（tauri::command 宏在定义处生成 __cmd__ 符号，
//!   不能移到子模块后用 pub use 重导出，故命令函数必须留在 mod.rs）

mod concurrent;
mod curseforge;
mod helpers;
mod modpack_stages;
mod modrinth;
mod types;

use crate::log_info;
use crate::minecraft::community::secure_storage;
use crate::minecraft::community::types::{Platform, ResourceType};
use crate::state::{AppState, DownloadStage, StageStatus};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

use helpers::{apply_filename_format, resolve_install_dir};
use modpack_stages::{download_modpack_archive, parse_modpack_info};

// 对外暴露：保持 lib.rs invoke_handler 注册路径完全向后兼容
// （pub use 同时也会把项带入当前作用域供本文件内使用，无需重复 use）
pub use types::{
    CommunityDownloadProgress, DownloadRequest, DownloadResult, InstallModpackRequest,
    InstallModpackResult, ModpackFormat,
};

/// 下载资源文件到游戏目录（保留原逻辑，用于"快速安装"）
///
/// 参考 PCL2 Save_Click：
/// - Mod → versions/{vid}/mods/
/// - ResourcePack → versions/{vid}/resourcepacks/
/// - Shader → versions/{vid}/shaderpacks/
/// - DataPack → versions/{vid}/datapacks/
#[tauri::command]
pub async fn download_resource(
    state: State<'_, AppState>,
    req: DownloadRequest,
) -> Result<DownloadResult, String> {
    log_info!(
        "[Community] Downloading {} from {}",
        req.file_name,
        req.url
    );

    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    // 根据 community_filename_format 拼接文件名（参考 PCL2 Save_Click）
    let final_file_name = apply_filename_format(
        &req.file_name,
        req.translated_name.as_deref(),
        config.community_filename_format,
    );
    drop(config);

    let target_dir = resolve_install_dir(&game_dir, req.resource_type, req.version_id.as_deref());

    // 确保目录存在
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let target_path = target_dir.join(&final_file_name);

    // 下载文件
    let client = crate::http::get_client();
    let resp = client
        .get(&req.url)
        .send()
        .await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("下载失败: HTTP {}", resp.status()));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取响应数据失败: {}", e))?;

    let size = bytes.len() as u64;

    // 写入文件
    std::fs::write(&target_path, &bytes)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    log_info!(
        "[Community] Downloaded {} ({} bytes) to {}",
        final_file_name,
        size,
        target_path.display()
    );

    Ok(DownloadResult {
        path: target_path.to_string_lossy().to_string(),
        size,
    })
}

/// 根据用户设置的 `community_filename_format` 格式化下载文件名
///
/// 详情页"下载到任意路径"流程使用：前端在弹保存对话框前调用此命令，
/// 获取格式化后的文件名作为默认名，避免使用原始名导致设置不生效。
#[tauri::command]
pub async fn format_download_filename(
    state: State<'_, AppState>,
    file_name: String,
    translated_name: Option<String>,
) -> Result<String, String> {
    let config = state.config.lock().await;
    Ok(apply_filename_format(
        &file_name,
        translated_name.as_deref(),
        config.community_filename_format,
    ))
}

/// 下载资源文件到自定义路径（用户通过文件管理器选择）
/// 流式下载 + 实时进度推送（参考 DownloadManager 的进度回调）
#[tauri::command]
pub async fn download_resource_to_path(
    app: AppHandle,
    url: String,
    file_name: String,
    save_path: String,
) -> Result<DownloadResult, String> {
    log_info!("[Community] 流式下载 {} 到 {}", file_name, save_path);

    let save_path = PathBuf::from(&save_path);

    // 确保父目录存在
    if let Some(parent) = save_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        }
    }

    let client = crate::http::get_client();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            let _ = app.emit("community-download-progress", CommunityDownloadProgress {
                file_name: file_name.clone(),
                downloaded: 0,
                total: 0,
                speed: 0,
                completed: false,
                error: Some(format!("下载请求失败: {}", e)),
            });
            format!("下载请求失败: {}", e)
        })?;

    if !resp.status().is_success() {
        let err = format!("下载失败: HTTP {}", resp.status());
        let _ = app.emit("community-download-progress", CommunityDownloadProgress {
            file_name: file_name.clone(),
            downloaded: 0,
            total: 0,
            speed: 0,
            completed: false,
            error: Some(err.clone()),
        });
        return Err(err);
    }

    let total_size = resp.content_length().unwrap_or(0);
    let mut file = std::fs::File::create(&save_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    use std::io::Write;

    use futures_util::StreamExt;
    let mut stream = resp.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut last_emit = std::time::Instant::now();
    let mut last_bytes: u64 = 0;
    let start_time = std::time::Instant::now();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("读取数据块失败: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("写入文件失败: {}", e))?;
        downloaded += chunk.len() as u64;

        // 每 300ms 推送一次进度（参考 DownloadManager 的 300ms 回调）
        let now = std::time::Instant::now();
        if now.duration_since(last_emit).as_millis() >= 300 {
            let elapsed = now.duration_since(last_emit).as_secs_f64().max(0.001);
            let speed = ((downloaded - last_bytes) as f64 / elapsed) as u64;
            let _ = app.emit("community-download-progress", CommunityDownloadProgress {
                file_name: file_name.clone(),
                downloaded,
                total: total_size,
                speed,
                completed: false,
                error: None,
            });
            last_emit = now;
            last_bytes = downloaded;
        }
    }

    file.flush().map_err(|e| format!("刷新文件失败: {}", e))?;
    drop(file);

    let size = downloaded;
    let elapsed = start_time.elapsed().as_secs_f64().max(0.001);
    let avg_speed = (size as f64 / elapsed) as u64;

    // 推送完成事件
    let _ = app.emit("community-download-progress", CommunityDownloadProgress {
        file_name: file_name.clone(),
        downloaded: size,
        total: total_size,
        speed: avg_speed,
        completed: true,
        error: None,
    });

    log_info!(
        "[Community] 下载完成: {} ({} bytes, {:.1}s)",
        file_name,
        size,
        elapsed
    );

    Ok(DownloadResult {
        path: save_path.to_string_lossy().to_string(),
        size,
    })
}

/// 安装资源文件（与 download_resource 相同，语义化命名）
#[tauri::command]
pub async fn install_resource(
    state: State<'_, AppState>,
    req: DownloadRequest,
) -> Result<DownloadResult, String> {
    download_resource(state, req).await
}

/// 获取资源默认安装路径（用于前端显示"打开文件夹"）
#[tauri::command]
pub async fn get_resource_install_path(
    state: State<'_, AppState>,
    resource_type: ResourceType,
    version_id: Option<String>,
) -> Result<String, String> {
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    drop(config);

    let path = resolve_install_dir(&game_dir, resource_type, version_id.as_deref());
    Ok(path.to_string_lossy().to_string())
}

/// 安装整合包
///
/// 完整流程（参考 PCL2 ModpackInstall + PageDownloadCompDetail.Install_Click）：
/// 1. CF 平台前置检查 API Key（未启用或为空立即报错）
/// 2. 下载原始整合包到 versions/{instance}/（委托 modpack_stages::download_modpack_archive）
/// 3. 检测格式 + 解析 manifest/modrinth.index.json（委托 modpack_stages::parse_modpack_info）
/// 4. 下载依赖文件（CF: install_cf_mods / MR: install_mr_files）
/// 5. 解压 overrides 到 instance 目录（concurrent::extract_overrides）
///
/// 进度通过 `state.download_state` 推送（与版本下载共用 DownloadPanel 展示）。
/// 完成后前端调用 `install_merged` 安装游戏本体。
#[tauri::command]
pub async fn install_modpack(
    state: State<'_, AppState>,
    req: InstallModpackRequest,
) -> Result<InstallModpackResult, String> {
    log_info!(
        "[Community] 开始安装整合包: platform={} instance={} url={}",
        req.platform.as_str(),
        req.instance_name,
        req.download_url
    );

    // 1. CF 平台前置检查 API Key
    if req.platform == Platform::CurseForge {
        let (enabled, api_key) = secure_storage::get_config_async().await;
        if !enabled {
            return Err(
                "CurseForge 整合包安装需要 API Key。请在「设置 → 社区资源」中启用 CurseForge 官方源并填写 API Key。"
                    .to_string(),
            );
        }
        let key_empty = api_key.as_deref().map_or(true, |k| k.is_empty());
        if key_empty {
            return Err(
                "CurseForge 整合包安装需要 API Key。已在设置中启用但未填写 API Key，请补全后重试。"
                    .to_string(),
            );
        }
        log_info!("[Community] CF API Key 检查通过");
    }

    // 解析游戏目录
    let config = state.config.lock().await;
    let game_dir = crate::state::resolve_game_dir(&config.game_dir);
    let max_threads = config.max_download_threads.max(1) as usize;
    drop(config);

    let instance_dir = game_dir.join("versions").join(&req.instance_name);
    std::fs::create_dir_all(&instance_dir)
        .map_err(|e| format!("创建整合包目录失败: {}", e))?;

    // 2. 重置 download_state，设置整合包专用 stages（统一方法）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.reset_stages(vec![
            DownloadStage::new_grouped("下载整合包", 10.0, "整合包安装"),
            DownloadStage::new_grouped("解析整合包", 1.0, "整合包安装"),
            DownloadStage::new_grouped("下载 MOD", 40.0, "整合包安装"),
            DownloadStage::new_grouped("复制配置文件", 5.0, "整合包安装"),
        ]);
    }

    // 3. Stage 0：下载原始整合包（委托 modpack_stages）
    let archive_path = instance_dir.join(&req.file_name);
    download_modpack_archive(&state, &archive_path, &req.download_url, &req.file_name).await?;

    // 4. Stage 1：打开 zip + 检测格式 + 解析 manifest/index（委托 modpack_stages::parse_modpack_info）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(1, StageStatus::Loading, 0.0);
    }
    let file = std::fs::File::open(&archive_path)
        .map_err(|e| format!("打开整合包失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;

    let (format, manifest_content, index_content) = concurrent::detect_modpack_format(&mut archive)?;
    let info = parse_modpack_info(format, manifest_content.as_deref(), index_content.as_deref())?;

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(1, StageStatus::Finished, 1.0);
    }
    log_info!(
        "[Community] 整合包格式={:?} game={} loader={}{} mods={}",
        info.format,
        info.game_version,
        info.loader,
        if info.loader_version.is_empty() { String::new() } else { format!("@{}", info.loader_version) },
        info.mod_files_count
    );

    // 5. Stage 2：下载依赖文件
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(2, StageStatus::Loading, 0.0);
    }
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)
        .map_err(|e| format!("创建 mods 目录失败: {}", e))?;

    match info.format {
        ModpackFormat::Curseforge => {
            let manifest = info.cf_manifest.expect("CF manifest 应已解析");
            curseforge::install_cf_mods(
                &state,
                &manifest.files,
                &mods_dir,
                max_threads,
                &instance_dir,
            )
            .await?;
        }
        ModpackFormat::Modrinth => {
            let index = info.mr_index.expect("MR index 应已解析");
            modrinth::install_mr_files(
                &state,
                &index.files,
                &instance_dir,
                max_threads,
            )
            .await?;
        }
    }
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(2, StageStatus::Finished, 1.0);
    }

    // 6. Stage 3：复制 overrides
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(3, StageStatus::Loading, 0.0);
    }
    concurrent::extract_overrides(&mut archive, &instance_dir, &state)?;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(3, StageStatus::Finished, 1.0);
        // 不调用 mark_complete()：前端会紧接着调用 install_merged 安装 MC 本体，
        // 轮询必须继续。mark_complete 由 install_merged 在全部完成后调用。
    }

    log_info!("[Community] 整合包安装完成: {}", req.instance_name);

    Ok(InstallModpackResult {
        format: info.format,
        game_version: info.game_version,
        loader: info.loader,
        loader_version: info.loader_version,
        archive_path: archive_path.to_string_lossy().to_string(),
        instance_dir: instance_dir.to_string_lossy().to_string(),
    })
}
