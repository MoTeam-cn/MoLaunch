//! 皮肤管理命令 upload 域（上传皮肤）

use crate::log_warn;
use crate::minecraft::image_cache;
use crate::minecraft::skin;
use crate::state::AppState;

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