//! 皮肤管理命令 list 域（皮肤/披风信息查询与缓存 URL 获取）

use crate::error_util::log_err;
use crate::minecraft::image_cache::{self, CachedImage};
use crate::minecraft::skin;
use crate::state::AppState;
use tauri::AppHandle;

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
            let persisted = state
                .auth_storage
                .load()
                .await
                .map_err(log_err("Failed to load auth storage"))?;
            persisted
                .ms_accounts
                .iter()
                .find(|a| &a.uuid == uuid)
                .map(|a| a.profile_json.as_str())
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
        Some(url) => Ok(Some(
            image_cache::get_image_url(&url, Some(app.clone())).await,
        )),
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
        Some(url) => Ok(Some(
            image_cache::get_image_url(&url, Some(app.clone())).await,
        )),
        None => Ok(None),
    }
}
