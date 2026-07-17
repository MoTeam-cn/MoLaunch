//! 社区资源下载安装命令
//!
//! 参考 PCL2 PageDownloadCompDetail Save_Click / Install_Click
//! 下载资源文件到指定版本目录

use crate::log_info;
use crate::minecraft::community::secure_storage;
use crate::minecraft::community::types::{Platform, ResourceType};
use crate::state::{AppState, DownloadStage, StageStatus};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// 格式化字节数为人类可读大小（如 29.6 MB），用于日志输出
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// 下载安装请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    /// 下载 URL
    pub url: String,
    /// 文件名（原始名，后端会根据 community_filename_format 重命名）
    pub file_name: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 目标版本 ID（安装到哪个版本目录）
    pub version_id: Option<String>,
    /// 文件 SHA1（用于校验）
    pub hash: Option<String>,
    /// 译名（可选，来自 mcmod 数据库，用于按 filename_format 拼接新文件名）
    pub translated_name: Option<String>,
}

/// 根据 `community_filename_format` 拼接文件名
///
/// 格式（参考 PCL2 PageSetupSystem FilenameFormat）：
/// - 0: 【译名】原名
/// - 1: [译名] 原名（默认）
/// - 2: 译名-原名
/// - 3: 原名-译名
/// - 4: 仅原名
///
/// 无译名时统一返回原名。扩展名（含 .jar.disabled 等多段后缀）原样保留。
pub fn apply_filename_format(original: &str, translated: Option<&str>, format: u8) -> String {
    let translated = match translated {
        Some(t) if !t.is_empty() => t,
        _ => return original.to_string(),
    };

    // 分离扩展名（保留 .jar.disabled / .jar.old 等多段后缀）
    let (stem, ext) = match original.rfind('.') {
        Some(pos) => {
            // .disabled / .old 是禁用后缀，继续向前找主扩展名
            let first_ext = &original[pos..];
            if first_ext == ".disabled" || first_ext == ".old" {
                let base = &original[..pos];
                if let Some(p2) = base.rfind('.') {
                    (base[..p2].to_string(), original[p2..].to_string())
                } else {
                    (original.to_string(), String::new())
                }
            } else {
                (original[..pos].to_string(), first_ext.to_string())
            }
        }
        None => (original.to_string(), String::new()),
    };

    let new_stem = match format {
        0 => format!("【{}】{}", translated, stem),
        1 => format!("[{}] {}", translated, stem),
        2 => format!("{}-{}", translated, stem),
        3 => format!("{}-{}", stem, translated),
        _ => stem.clone(), // 4 = 仅原名
    };

    if ext.is_empty() {
        new_stem
    } else {
        format!("{}{}", new_stem, ext)
    }
}

/// 下载安装结果
#[derive(Debug, Serialize)]
pub struct DownloadResult {
    pub path: String,
    pub size: u64,
}

/// 社区资源下载进度事件
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunityDownloadProgress {
    /// 文件名
    pub file_name: String,
    /// 已下载字节
    pub downloaded: u64,
    /// 总字节（未知则为 0）
    pub total: u64,
    /// 下载速度（字节/秒）
    pub speed: u64,
    /// 是否完成
    pub completed: bool,
    /// 错误信息
    pub error: Option<String>,
}

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

/// 解析安装目录
fn resolve_install_dir(
    game_dir: &PathBuf,
    resource_type: ResourceType,
    version_id: Option<&str>,
) -> PathBuf {
    let subdir = resource_type.install_subdir();
    if let Some(vid) = version_id {
        if !vid.is_empty() && !subdir.is_empty() {
            game_dir.join("versions").join(vid).join(subdir)
        } else if !subdir.is_empty() {
            game_dir.join(subdir)
        } else {
            game_dir.clone()
        }
    } else if !subdir.is_empty() {
        game_dir.join(subdir)
    } else {
        game_dir.clone()
    }
}

// ============================================================================
// 整合包安装（参考 PCL2 ModModpack.vb ModpackInstall）
// ============================================================================
//
// PCL2 的整合包安装流程：
//   1. 下载原始整合包到 versions/{InstanceName}/原始整合包.{zip|mrpack}
//   2. 用 zip 根目录关键文件判定格式：
//        - manifest.json        → CurseForge 整合包
//        - modrinth.index.json  → Modrinth 整合包
//   3. CF 路径：必须配置 API Key，否则在最开始就报错（用户需求）
//      - 解析 manifest.json → minecraft.version + modLoaders + files[]
//      - POST /v1/mods/files 批量查询下载信息 → 下载所有 mods
//      - 解压 overrides/* 到 instance 目录
//   4. MR 路径：无需 API Key
//      - 解析 modrinth.index.json → dependencies + files[]
//      - 遍历 files[] 直接下载（含 downloads URL）
//      - 解压 overrides + client-overrides
//
// 进度通过 state.download_state 共享（与版本下载共用 DownloadPanel 展示）。
// 完成后前端调用 install_merged 安装游戏本体（使用 manifest 中的 mc_version + loader）。

/// 整合包安装请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallModpackRequest {
    /// 来源平台
    pub platform: Platform,
    /// 下载 URL
    pub download_url: String,
    /// 原始文件名（如 MyModpack-1.0.zip / .mrpack）
    pub file_name: String,
    /// 整合包实例名（用于 versions/{instance_name}/ 目录）
    pub instance_name: String,
}

/// 整合包格式
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModpackFormat {
    Curseforge,
    Modrinth,
}

/// 整合包安装结果
///
/// 完成整合包专属部分（下载原始包、下载依赖 mods、复制 overrides）后返回。
/// 前端拿到结果后调用 `install_merged` 安装游戏本体（使用返回的 mc_version + loader 信息）。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallModpackResult {
    /// 识别出的整合包格式
    pub format: ModpackFormat,
    /// 整合包内 minecraft.version
    pub game_version: String,
    /// 加载器名称（forge / fabric / quilt / neoforge / liteloader），空表示原版
    pub loader: String,
    /// 加载器版本
    pub loader_version: String,
    /// 原始整合包保存路径
    pub archive_path: String,
    /// instance 目录
    pub instance_dir: String,
}

/// 安装整合包
///
/// 完整流程（参考 PCL2 ModpackInstall + PageDownloadCompDetail.Install_Click）：
/// 1. CF 平台前置检查 API Key（未启用或为空立即报错）
/// 2. 下载原始整合包到 versions/{instance}/
/// 3. 检测格式 + 解析 manifest/modrinth.index.json
/// 4. 下载依赖文件（CF: POST /v1/mods/files 批量查询 → 下载；MR: 直接下载 files[]）
/// 5. 解压 overrides 到 instance 目录
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

    // 3. 下载原始整合包（Stage 0）— 直接交给 DownloadManager
    // DownloadManager 内部会自动探测文件大小（expected_size=0 时用 GET + Range: bytes=0-0）
    // 并据此判断是否走分片下载，无需在这里手动探测
    let archive_path = instance_dir.join(&req.file_name);
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(0, StageStatus::Loading, 0.0);
    }

    log_info!("[Community] 下载整合包到: {}", archive_path.display());

    use crate::minecraft::download::manager::DownloadManager;
    use crate::minecraft::download::types::{DownloadTask, DownloadStatus};
    use crate::minecraft::sources::DownloadSourceMode;

    let archive_task = DownloadTask {
        id: "modpack_archive".to_string(),
        urls: vec![req.download_url.clone()],
        local_path: archive_path.to_string_lossy().to_string(),
        expected_size: 0, // 由 DownloadManager 自动探测
        expected_hash: None,
    };

    // stage 0 的进度回调：统一用 sync_stage_from_progress 同步 GlobalProgress 到 download_state
    let stage0_state = state.download_state.clone();
    let stage0_callback: Arc<dyn Fn(crate::minecraft::download::types::GlobalProgress) + Send + Sync> =
        Arc::new(move |p| {
            let mut ds = stage0_state.lock().unwrap();
            ds.sync_stage_from_progress(
                0,
                p.downloaded_bytes,
                p.total_bytes,
                p.completed_files,
                p.total_files,
                p.current_speed,
            );
        });

    let config = state.config.lock().await;
    let chunk_count = config.chunk_count.max(1) as usize;
    drop(config);
    let archive_manager = DownloadManager::new(4, chunk_count, 0, DownloadSourceMode::Smart);
    let archive_results = archive_manager
        .download_batch(vec![archive_task], Some(stage0_callback))
        .await;

    let archive_err = archive_results
        .first()
        .and_then(|r| {
            if r.status != DownloadStatus::Completed && r.status != DownloadStatus::Skipped {
                r.error.clone()
            } else {
                None
            }
        });

    if let Some(err) = archive_err {
        let msg = format!("下载整合包失败: {}", err);
        {
            let mut ds = state.download_state.lock().unwrap();
            ds.set_stage_status(0, StageStatus::Failed, 0.0);
            ds.mark_failed(1);
        }
        log_info!("[Community] 整合包安装失败: {}", msg);
        return Err(msg);
    }

    let archive_size = std::fs::metadata(&archive_path)
        .map(|m| m.len())
        .unwrap_or(0);

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(0, StageStatus::Finished, 1.0);
    }
    log_info!(
        "[Community] 整合包下载完成: {} ({})",
        req.file_name,
        format_bytes(archive_size)
    );

    // 4. 打开 zip，检测格式（Stage 1）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(1, StageStatus::Loading, 0.0);
    }
    let file = std::fs::File::open(&archive_path)
        .map_err(|e| format!("打开整合包失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| format!("解析 zip 失败: {}（可能不是有效的整合包）", e))?;

    let (format, manifest_content, index_content) = detect_modpack_format(&mut archive)?;

    // 解析 manifest/index，保留结构体供后续使用（避免二次 unwrap move）
    let (game_version, loader, loader_version, mod_files_count, cf_manifest, mr_index) = match format {
        ModpackFormat::Curseforge => {
            let manifest: CfManifest = serde_json::from_str(manifest_content.as_deref().unwrap())
                .map_err(|e| format!("解析 manifest.json 失败: {}", e))?;
            let gv = manifest.minecraft.version.clone();
            let (loader, ver) = manifest
                .minecraft
                .mod_loaders
                .iter()
                .find(|l| l.primary)
                .or_else(|| manifest.minecraft.mod_loaders.first())
                .map(|l| parse_cf_loader_id(&l.id))
                .unwrap_or((String::new(), String::new()));
            let count = manifest.files.len();
            (gv, loader, ver, count, Some(manifest), None)
        }
        ModpackFormat::Modrinth => {
            let index: MrIndex = serde_json::from_str(index_content.as_deref().unwrap())
                .map_err(|e| format!("解析 modrinth.index.json 失败: {}", e))?;
            let gv = index.dependencies.get("minecraft").cloned().unwrap_or_default();
            let (loader, ver) = ["fabric-loader", "quilt-loader", "forge", "neoforge"]
                .iter()
                .find_map(|key| {
                    index.dependencies.get(*key).map(|v| {
                        let (ln, vv) = parse_mr_loader(key, v);
                        (ln.to_string(), vv)
                    })
                })
                .unwrap_or((String::new(), String::new()));
            let count = index.files.len();
            (gv, loader, ver, count, None, Some(index))
        }
    };

    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(1, StageStatus::Finished, 1.0);
    }
    log_info!(
        "[Community] 整合包格式={:?} game={} loader={}{} mods={}",
        format,
        game_version,
        loader,
        if loader_version.is_empty() { String::new() } else { format!("@{}", loader_version) },
        mod_files_count
    );

    // 5. 下载依赖文件（Stage 2）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(2, StageStatus::Loading, 0.0);
    }
    let mods_dir = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_dir)
        .map_err(|e| format!("创建 mods 目录失败: {}", e))?;

    match format {
        ModpackFormat::Curseforge => {
            let manifest = cf_manifest.expect("CF manifest 应已解析");
            install_cf_mods(
                &state,
                &manifest.files,
                &mods_dir,
                max_threads,
                &instance_dir,
            )
            .await?;
        }
        ModpackFormat::Modrinth => {
            let index = mr_index.expect("MR index 应已解析");
            install_mr_files(
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

    // 6. 复制 overrides（Stage 3）
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(3, StageStatus::Loading, 0.0);
    }
    extract_overrides(&mut archive, &instance_dir, &state)?;
    {
        let mut ds = state.download_state.lock().unwrap();
        ds.set_stage_status(3, StageStatus::Finished, 1.0);
        // 不调用 mark_complete()：前端会紧接着调用 install_merged 安装 MC 本体，
        // 轮询必须继续。mark_complete 由 install_merged 在全部完成后调用。
    }

    log_info!("[Community] 整合包安装完成: {}", req.instance_name);

    Ok(InstallModpackResult {
        format,
        game_version,
        loader,
        loader_version,
        archive_path: archive_path.to_string_lossy().to_string(),
        instance_dir: instance_dir.to_string_lossy().to_string(),
    })
}

// ============================================================================
// download_state 操作已统一到 state::DownloadState 的方法中：
//   reset_stages / set_stage_status / set_stage_bytes / sync_stage_from_progress / mark_complete / mark_failed
// 不再在此文件维护私有辅助函数，与 install_merged 共用同一套逻辑
// ============================================================================

// ============================================================================
// 下载辅助
// ============================================================================

/// 并发下载多个文件，进度汇总到 download_state 的指定 stage
///
/// 统一走 DownloadManager：自动按文件大小走分片下载（>1MB/chunk 走 chunk::download_chunked）
/// 或普通下载（小文件直连），与 MC 本体/库/assets 走同一套下载基础设施。
/// 进度通过 `sync_stage_from_progress` 统一同步到 download_state（速度/字节累加由统一方法处理）。
async fn download_files_concurrent(
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
    let progress_callback: Arc<dyn Fn(crate::minecraft::download::types::GlobalProgress) + Send + Sync> =
        Arc::new(move |p| {
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
    let manager = DownloadManager::new(max_threads, chunk_count, 0, DownloadSourceMode::Smart);
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

// ============================================================================
// CurseForge 整合包
// ============================================================================

/// CF manifest.json 结构
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfManifest {
    minecraft: CfMinecraft,
    #[serde(default)]
    files: Vec<CfManifestFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfMinecraft {
    version: String,
    #[serde(default)]
    mod_loaders: Vec<CfModLoader>,
}

#[derive(Debug, Deserialize)]
struct CfModLoader {
    id: String,
    primary: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // project_id / required 暂未参与依赖过滤，未来按 optional=false 跳过非必要 mod 时启用
struct CfManifestFile {
    project_id: i64,
    file_id: i64,
    #[serde(default)]
    required: bool,
}

/// POST /v1/mods/files 批量查询响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CfFilesBatchResponse {
    data: Vec<CfFileEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct CfFileEntry {
    file_id: i64,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    download_url: Option<String>,
    #[serde(default)]
    file_length: u64,
}

/// 安装 CF 整合包依赖 mods
///
/// POST /v1/mods/files 批量查询所有 file_id 的下载信息，然后并发下载到 mods 目录。
async fn install_cf_mods(
    state: &AppState,
    manifest_files: &[CfManifestFile],
    mods_dir: &std::path::Path,
    max_threads: usize,
    _instance_dir: &std::path::Path,
) -> Result<(), String> {
    if manifest_files.is_empty() {
        log_info!("[Community] CF manifest 无依赖 mods");
        return Ok(());
    }

    // 1. 批量查询下载信息
    let file_ids: Vec<i64> = manifest_files.iter().map(|f| f.file_id).collect();
    log_info!("[Community] CF 批量查询 {} 个文件", file_ids.len());

    let (_enabled, api_key) = secure_storage::get_config_async().await;
    let key = api_key.ok_or("CF API Key 丢失")?;

    let client = crate::http::get_client();
    let resp = client
        .post("https://api.curseforge.com/v1/mods/files")
        .header("x-api-key", &key)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "fileIds": file_ids }))
        .send()
        .await
        .map_err(|e| format!("CF 批量查询失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("CF 批量查询失败: HTTP {}", resp.status()));
    }

    let batch: CfFilesBatchResponse = resp
        .json()
        .await
        .map_err(|e| format!("CF 批量查询响应解析失败: {}", e))?;

    log_info!("[Community] CF 批量查询返回 {} 个文件", batch.data.len());

    // 2. 构造下载列表（CF 通常只有一个 download_url，包装为单元素数组）
    let mut download_list: Vec<(Vec<String>, String, u64)> = Vec::with_capacity(batch.data.len());
    let mut total_bytes: u64 = 0;
    for entry in &batch.data {
        let primary_url = entry
            .download_url
            .clone()
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| construct_cf_edge_url(entry.file_id, &entry.file_name));
        let target = mods_dir.join(&entry.file_name);
        download_list.push((vec![primary_url], target.to_string_lossy().to_string(), entry.file_length));
        total_bytes += entry.file_length;
    }

    log_info!(
        "[Community] CF 下载 {} 个文件，总大小 {}",
        download_list.len(),
        format_bytes(total_bytes)
    );

    // 3. 并发下载
    download_files_concurrent(state, 2, &download_list, max_threads, total_bytes).await?;

    log_info!("[Community] CF mods 下载完成 ({} 个)", download_list.len());
    Ok(())
}

/// 构造 CF edge 下载 URL（当 download_url 为空时的 fallback）
fn construct_cf_edge_url(file_id: i64, file_name: &str) -> String {
    let id_str = file_id.to_string();
    if id_str.len() >= 6 {
        let (p1, p2) = id_str.split_at(id_str.len() - 4);
        format!("https://edge.forgecdn.net/files/{}/{}", p1, p2)
    } else {
        format!("https://edge.forgecdn.net/files/0/{}", file_name)
    }
}

// ============================================================================
// Modrinth 整合包
// ============================================================================

/// MR modrinth.index.json 结构
#[derive(Debug, Deserialize)]
struct MrIndex {
    #[serde(default)]
    dependencies: std::collections::HashMap<String, String>,
    #[serde(default)]
    files: Vec<MrFile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MrFile {
    path: String,
    #[serde(default)]
    downloads: Vec<String>,
    #[serde(default)]
    file_size: u64,
}

/// 安装 MR 整合包依赖文件
///
/// 遍历 files[] 直接下载（path 相对于 instance 目录，如 mods/xxx.jar）。
async fn install_mr_files(
    state: &AppState,
    mr_files: &[MrFile],
    instance_dir: &std::path::Path,
    max_threads: usize,
) -> Result<(), String> {
    if mr_files.is_empty() {
        log_info!("[Community] MR index 无依赖文件");
        return Ok(());
    }

    let mut download_list: Vec<(Vec<String>, String, u64)> = Vec::with_capacity(mr_files.len());
    let mut total_bytes: u64 = 0;
    for f in mr_files {
        if f.downloads.is_empty() {
            log_info!("[Community] MR 文件无下载 URL，跳过: {}", f.path);
            continue;
        }
        // Modrinth 的 downloads 是数组，包含多个镜像源，全部传入供 fallback
        let urls: Vec<String> = f.downloads.iter().filter(|u| !u.is_empty()).cloned().collect();
        if urls.is_empty() {
            log_info!("[Community] MR 文件所有 URL 为空，跳过: {}", f.path);
            continue;
        }
        let target = instance_dir.join(&f.path);
        download_list.push((urls, target.to_string_lossy().to_string(), f.file_size));
        total_bytes += f.file_size;
    }

    log_info!(
        "[Community] MR 下载 {} 个文件，总大小 {}",
        download_list.len(),
        format_bytes(total_bytes)
    );

    download_files_concurrent(state, 2, &download_list, max_threads, total_bytes).await?;

    log_info!("[Community] MR 文件下载完成");
    Ok(())
}

// ============================================================================
// overrides 解压
// ============================================================================

/// 从 zip 解压 overrides（和 client-overrides）到 instance 目录
fn extract_overrides(
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
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("创建目录失败: {}", e))?;
            }
        }

        if entry.is_file() {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("读取文件失败: {}", e))?;
            std::fs::write(&target, &buf)
                .map_err(|e| format!("写入文件失败: {}", e))?;
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

// ============================================================================
// 格式检测
// ============================================================================

/// 检测整合包格式，返回 (format, cf_manifest_content, mr_index_content)
fn detect_modpack_format(
    archive: &mut zip::ZipArchive<std::fs::File>,
) -> Result<(ModpackFormat, Option<String>, Option<String>), String> {
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
            return Err("无法识别的整合包格式：未找到 manifest.json 或 modrinth.index.json".to_string());
        }
    };

    Ok((format, cf_content, mr_content))
}

// ============================================================================
// 辅助解析函数
// ============================================================================

/// 解析 CF loader id（如 "forge-36.2.39"）→ (loader_name, version)
fn parse_cf_loader_id(id: &str) -> (String, String) {
    if let Some(pos) = id.find('-') {
        (id[..pos].to_string(), id[pos + 1..].to_string())
    } else {
        (id.to_string(), String::new())
    }
}

/// 解析 MR loader key/value → (loader_name, version)
fn parse_mr_loader(key: &str, value: &str) -> (&'static str, String) {
    match key {
        "fabric-loader" => ("fabric", value.split('/').next().unwrap_or("").to_string()),
        "quilt-loader" => ("quilt", value.split('/').next().unwrap_or("").to_string()),
        "forge" => ("forge", value.to_string()),
        "neoforge" => ("neoforge", value.to_string()),
        _ => ("", value.to_string()),
    }
}
