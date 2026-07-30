//! 皮肤管理命令
//!
//! 提供皮肤/披风管理的子模块函数，供 `skin_manager` dispatcher 调用：
//! - 获取皮肤/披风信息
//! - 获取皮肤/披风 PNG 下载 URL（带本地缓存，方案 C）
//! - 上传皮肤
//! - 装备/取消披风
//! - 下载 URL 图片到本地文件

use crate::error_util::log_err;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::image_cache::{self, CachedImage};
use crate::minecraft::skin;
use crate::state::AppState;
use crate::utils::dispatcher::ActionRequest;
use tauri::{AppHandle, State};

/// 统一皮肤管理 IPC 入口
///
/// 接收 `ActionRequest { action, params }` 请求体，转发到
/// `crate::utils::skin_manager::dispatch` 进行 action 分发。
#[tauri::command]
pub async fn skin_manager(
    state: State<'_, AppState>,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    let state = state.inner().clone();
    crate::utils::skin_manager::dispatch(state, app, req).await
}


/// 获取当前账号的皮肤/披风信息（从 profile_json 解析）
///
/// 返回前会对每个 skin/cape 的 url 做缓存处理，填充 cached_url 和 cached 字段：
/// - 缓存命中：cached_url 为 cache-image:// 本地 URL，cached: true
/// - 缓存未命中：cached_url 为远程 URL，cached: false，后端异步下载完成后 emit image-cached 事件
pub async fn get_skin_cape_info(
    state: &AppState,
    app: &AppHandle,
) -> Result<skin::SkinCapeInfo, String> {
    let auth = state.auth.lock().await;
    let profile_json = auth
        .current_user
        .as_ref()
        .and_then(|u| u.profile_json.as_ref())
        .ok_or("No profile data, please login with Microsoft account first")?;
    let mut info = skin::parse_skin_cape_info(profile_json)?;
    drop(auth);

    // 对每个 skin 填充缓存 URL
    for skin in info.skins.iter_mut() {
        let cached = image_cache::get_image_url(&skin.url, Some(app.clone())).await;
        skin.cached_url = Some(cached.url);
        skin.cached = Some(cached.cached);
    }

    // 对每个 cape 填充缓存 URL
    for cape in info.capes.iter_mut() {
        if let Some(ref url) = cape.url {
            let cached = image_cache::get_image_url(url, Some(app.clone())).await;
            cape.cached_url = Some(cached.url);
            cape.cached = Some(cached.cached);
        }
    }

    Ok(info)
}

/// 获取皮肤 PNG URL（带本地缓存）
///
/// 可传入 uuid 指定账号（用于预加载非当前账号的皮肤）；
/// 不传或传 null 则使用当前登录用户。
///
/// 返回 `CachedImage`：
/// - `cached: true` 表示返回的是本地缓存 URL（无需网络）
/// - `cached: false` 表示返回的是远程 URL，后端会异步下载到缓存，完成后 emit `image-cached` 事件
pub async fn get_skin_url(
    state: &AppState,
    app: &AppHandle,
    uuid: Option<String>,
) -> Result<Option<CachedImage>, String> {
    let auth = state.auth.lock().await;
    // 优先用 uuid 从 current_user 或 ms_accounts 查找 profile_json
    let profile_json: String = if let Some(ref uuid) = uuid {
        // 先看 current_user
        if auth
            .current_user
            .as_ref()
            .map(|u| &u.uuid == uuid)
            .unwrap_or(false)
        {
            auth.current_user
                .as_ref()
                .and_then(|u| u.profile_json.as_ref())
                .ok_or("No profile data")?
                .clone()
        } else {
            // 从 ms_accounts 查找
            let persisted = state.auth_storage.load().await.map_err(log_err("Failed to load auth storage"))?;
            persisted
                .ms_accounts
                .iter()
                .find(|a| &a.uuid == uuid)
                .and_then(|a| Some(a.profile_json.as_str()))
                .ok_or("No profile data")?
                .to_string()
        }
    } else {
        auth.current_user
            .as_ref()
            .and_then(|u| u.profile_json.as_ref())
            .ok_or("No profile data")?
            .clone()
    };
    drop(auth);

    let remote_url = skin::get_skin_url(&profile_json);
    match remote_url {
        Some(url) => Ok(Some(image_cache::get_image_url(&url, Some(app.clone())).await)),
        None => Ok(None),
    }
}

/// 获取当前已装备披风的下载 URL（带本地缓存）
///
/// 返回 `CachedImage`：
/// - `cached: true` 表示返回的是本地缓存 URL（无需网络）
/// - `cached: false` 表示返回的是远程 URL，后端会异步下载到缓存，完成后 emit `image-cached` 事件
pub async fn get_cape_url(
    state: &AppState,
    app: &AppHandle,
) -> Result<Option<CachedImage>, String> {
    let auth = state.auth.lock().await;
    let profile_json = auth
        .current_user
        .as_ref()
        .and_then(|u| u.profile_json.as_ref())
        .ok_or("No profile data")?;
    let remote_url = skin::get_cape_url(profile_json);
    drop(auth);

    match remote_url {
        Some(url) => Ok(Some(image_cache::get_image_url(&url, Some(app.clone())).await)),
        None => Ok(None),
    }
}

/// 上传/修改皮肤
///
/// `variant`: "classic"（Steve 模型）或 "slim"（Alex 模型）
/// `file_path`: PNG 文件本地路径（后端直接读取，避免前端 base64 转换）
pub async fn upload_skin(
    state: &AppState,
    file_path: String,
    variant: String,
) -> Result<(), String> {
    let access_token = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .map(|u| u.access_token.clone())
            .ok_or("Not logged in")?
    };

    if !["classic", "slim"].contains(&variant.as_str()) {
        return Err("variant must be 'classic' or 'slim'".to_string());
    }

    // 在阻塞任务中读取文件，避免阻塞异步运行时
    let png_data = tokio::task::spawn_blocking(move || {
        std::fs::read(&file_path).map_err(|e| format!("read file error: {}", e))
    })
    .await
    .map_err(|e| format!("task join error: {}", e))??;

    skin::upload_skin(&access_token, png_data, &variant).await?;

    // 上传成功后刷新本地 profile 缓存，确保前端能读到最新皮肤
    let old_profile_json = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .and_then(|u| u.profile_json.clone())
    };

    match skin::fetch_profile(&access_token).await {
        Ok(new_profile) => {
            let mut auth = state.auth.lock().await;
            if let Some(user) = auth.current_user.as_mut() {
                user.profile_json = Some(new_profile);
            }
        }
        Err(e) => log_warn!("Failed to refresh profile after skin upload: {}", e),
    }

    // 失效旧皮肤缓存（URL 变化会自动失效，但显式清理更稳妥）
    invalidate_skin_cache(&old_profile_json);

    Ok(())
}

/// 装备披风
pub async fn equip_cape(state: &AppState, cape_id: String) -> Result<(), String> {
    let access_token = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .map(|u| u.access_token.clone())
            .ok_or("Not logged in")?
    };

    skin::equip_cape(&access_token, &cape_id).await?;

    // 装备成功后刷新本地 profile 缓存，确保前端能读到最新披风
    let old_profile_json = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .and_then(|u| u.profile_json.clone())
    };

    match skin::fetch_profile(&access_token).await {
        Ok(new_profile) => {
            let mut auth = state.auth.lock().await;
            if let Some(user) = auth.current_user.as_mut() {
                user.profile_json = Some(new_profile);
            }
        }
        Err(e) => log_warn!("Failed to refresh profile after cape equip: {}", e),
    }

    // 失效旧披风缓存
    invalidate_cape_cache(&old_profile_json);

    Ok(())
}

/// 取消披风
pub async fn unequip_cape(state: &AppState) -> Result<(), String> {
    let access_token = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .map(|u| u.access_token.clone())
            .ok_or("Not logged in")?
    };

    skin::unequip_cape(&access_token).await?;

    // 取消成功后刷新本地 profile 缓存
    let old_profile_json = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .and_then(|u| u.profile_json.clone())
    };

    match skin::fetch_profile(&access_token).await {
        Ok(new_profile) => {
            let mut auth = state.auth.lock().await;
            if let Some(user) = auth.current_user.as_mut() {
                user.profile_json = Some(new_profile);
            }
        }
        Err(e) => log_warn!("Failed to refresh profile after cape unequip: {}", e),
    }

    // 失效旧披风缓存
    invalidate_cape_cache(&old_profile_json);

    Ok(())
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
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("download request error: {}", e))?;

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

/// 失效旧皮肤缓存（上传新皮肤后调用）
fn invalidate_skin_cache(old_profile_json: &Option<String>) {
    if let Some(json) = old_profile_json {
        if let Some(old_url) = skin::get_skin_url(json) {
            if let Err(e) = image_cache::invalidate(&old_url) {
                log_warn!("[Skin] 失效旧皮肤缓存失败: {}", e);
            }
        }
    }
}

/// 失效旧披风缓存（装备/取消披风后调用）
fn invalidate_cape_cache(old_profile_json: &Option<String>) {
    if let Some(json) = old_profile_json {
        if let Some(old_url) = skin::get_cape_url(json) {
            if let Err(e) = image_cache::invalidate(&old_url) {
                log_warn!("[Skin] 失效旧披风缓存失败: {}", e);
            }
        }
    }
}
