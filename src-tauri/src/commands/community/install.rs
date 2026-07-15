//! 社区资源下载安装命令
//!
//! 参考 PCL2 PageDownloadCompDetail Save_Click / Install_Click
//! 下载资源文件到指定版本目录

use crate::log_info;
use crate::minecraft::community::types::ResourceType;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, State};

/// 下载安装请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    /// 下载 URL
    pub url: String,
    /// 文件名
    pub file_name: String,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 目标版本 ID（安装到哪个版本目录）
    pub version_id: Option<String>,
    /// 文件 SHA1（用于校验）
    pub hash: Option<String>,
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
    drop(config);

    let target_dir = resolve_install_dir(&game_dir, req.resource_type, req.version_id.as_deref());

    // 确保目录存在
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;
    }

    let target_path = target_dir.join(&req.file_name);

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
        req.file_name,
        size,
        target_path.display()
    );

    Ok(DownloadResult {
        path: target_path.to_string_lossy().to_string(),
        size,
    })
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
