//! 皮肤管理分发层（skin_manager 的 dispatch 转发 + download_url_to_file 工具）

use crate::log_info;
use crate::log_warn;
use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::AppHandle;

/// 皮肤管理 action 分发入口
///
/// 接收 `ActionRequest { action, params }`，转发到
/// `super::manager::dispatch` 进行 action 分发。
pub(crate) async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    super::manager::dispatch(state, app, req).await
}

/// 下载指定 URL 的图片到本地文件
///
/// 用于"下载当前皮肤到本地"功能：前端已有皮肤 URL（来自 get_skin_url），
/// 用户选择保存位置后，后端直接从 URL 下载并写入文件，避免 base64 中转开销。
///
/// 特殊处理：当 URL 为 `cache-image.localhost` 或 `cache-image://` 格式时，
/// 这是 Tauri WebView 内部虚拟 URL（由 register_uri_scheme_protocol 注册），
/// 后端 reqwest 无法访问。此时直接从本地缓存文件读取。
pub async fn download_url_to_file(url: String, path: String) -> Result<(), String> {
    log_info!("[Skin] 下载 URL 到文件: {} -> {}", url, path);

    // 识别 Tauri WebView 内部虚拟 URL（cache-image scheme），直接从本地缓存读取
    if let Some(bytes) = crate::minecraft::image_cache::read_cache_by_url(&url) {
        std::fs::write(&path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;
        log_info!("[Skin] 从缓存复制到: {} ({} 字节)", path, bytes.len());
        return Ok(());
    } else if crate::minecraft::image_cache::is_cache_url(&url) {
        // 是虚拟 URL 但缓存文件不存在
        return Err(format!("缓存文件不存在: {}", url));
    }

    // 普通 HTTP URL：用 reqwest 下载
    let client = crate::http::get_client();
    let response = client.get(&url).send().await.map_err(|e| {
        format!(
            "download request error: {}",
            crate::http::request_error_msg(&e)
        )
    })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log_warn!("[Skin] 下载失败: {} - {}", status, body);
        return Err(format!("download HTTP {}: {}", status, body));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("read bytes error: {}", e))?;

    std::fs::write(&path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;

    log_info!("[Skin] 已保存到: {} ({} 字节)", path, bytes.len());
    Ok(())
}
