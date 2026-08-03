//! 皮肤管理命令 cape 域（装备 / 取消披风）

use crate::log_warn;
use crate::minecraft::image_cache;
use crate::minecraft::skin;
use crate::state::AppState;

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