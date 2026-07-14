//! 皮肤管理命令
//!
//! 提供 Tauri 命令接口，供前端调用：
//! - 获取皮肤/披风信息
//! - 下载皮肤 PNG（前端裁剪为 2D 头像）
//! - 上传皮肤
//! - 装备/取消披风

use crate::minecraft::skin;
use crate::state::AppState;
use crate::log_warn;
use tauri::State;

/// 获取当前账号的皮肤/披风信息（从 profile_json 解析）
#[tauri::command]
pub async fn get_skin_cape_info(state: State<'_, AppState>) -> Result<skin::SkinCapeInfo, String> {
    let auth = state.auth.lock().await;
    let profile_json = auth
        .current_user
        .as_ref()
        .and_then(|u| u.profile_json.as_ref())
        .ok_or("No profile data, please login with Microsoft account first")?;
    skin::parse_skin_cape_info(profile_json)
}

/// 获取皮肤 PNG 下载 URL（从 profile 解析，参考 PCL2）
#[tauri::command]
pub async fn get_skin_url(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let auth = state.auth.lock().await;
    let profile_json = auth
        .current_user
        .as_ref()
        .and_then(|u| u.profile_json.as_ref())
        .ok_or("No profile data")?;
    Ok(skin::get_skin_url(profile_json))
}

/// 下载皮肤 PNG，返回 base64 编码的 PNG 数据
///
/// 可传入 uuid 指定账号（用于预加载非当前账号的皮肤）；
/// 不传或传 null 则使用当前登录用户。
/// 前端收到 base64 后用 canvas 裁剪 (8,8,8,8) 区域作为头像（PCL2 的方式）
#[tauri::command]
pub async fn download_skin_png(
    state: State<'_, AppState>,
    uuid: Option<String>,
) -> Result<String, String> {
    use base64::Engine;
    let skin_url = {
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
                let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;
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
        skin::get_skin_url(&profile_json).ok_or("No active skin found in profile")?
    };

    let png_data = skin::download_skin_png(&skin_url).await?;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(format!("data:image/png;base64,{}", base64_data))
}

/// 下载当前已装备披风的 PNG，返回 base64 编码数据
///
/// 前端用于在 2D 人物预览中合成披风显示
#[tauri::command]
pub async fn download_cape_png(state: State<'_, AppState>) -> Result<Option<String>, String> {
    use base64::Engine;
    let cape_url = {
        let auth = state.auth.lock().await;
        let profile_json = auth
            .current_user
            .as_ref()
            .and_then(|u| u.profile_json.as_ref())
            .ok_or("No profile data")?;
        skin::get_cape_url(profile_json)
    };

    let Some(cape_url) = cape_url else {
        return Ok(None);
    };

    let png_data = skin::download_cape_png(&cape_url).await?;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(Some(format!("data:image/png;base64,{}", base64_data)))
}

/// 上传/修改皮肤
///
/// `variant`: "classic"（Steve 模型）或 "slim"（Alex 模型）
/// `file_path`: PNG 文件本地路径（后端直接读取，避免前端 base64 转换）
#[tauri::command]
pub async fn upload_skin(
    state: State<'_, AppState>,
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
    match skin::fetch_profile(&access_token).await {
        Ok(new_profile) => {
            let mut auth = state.auth.lock().await;
            if let Some(user) = auth.current_user.as_mut() {
                user.profile_json = Some(new_profile);
            }
        }
        Err(e) => log_warn!("Failed to refresh profile after skin upload: {}", e),
    }

    Ok(())
}

/// 装备披风
#[tauri::command]
pub async fn equip_cape(state: State<'_, AppState>, cape_id: String) -> Result<(), String> {
    let access_token = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .map(|u| u.access_token.clone())
            .ok_or("Not logged in")?
    };

    skin::equip_cape(&access_token, &cape_id).await?;

    // 装备成功后刷新本地 profile 缓存，确保前端能读到最新披风
    match skin::fetch_profile(&access_token).await {
        Ok(new_profile) => {
            let mut auth = state.auth.lock().await;
            if let Some(user) = auth.current_user.as_mut() {
                user.profile_json = Some(new_profile);
            }
        }
        Err(e) => log_warn!("Failed to refresh profile after cape equip: {}", e),
    }

    Ok(())
}

/// 取消披风
#[tauri::command]
pub async fn unequip_cape(state: State<'_, AppState>) -> Result<(), String> {
    let access_token = {
        let auth = state.auth.lock().await;
        auth.current_user
            .as_ref()
            .map(|u| u.access_token.clone())
            .ok_or("Not logged in")?
    };

    skin::unequip_cape(&access_token).await?;

    // 取消成功后刷新本地 profile 缓存
    match skin::fetch_profile(&access_token).await {
        Ok(new_profile) => {
            let mut auth = state.auth.lock().await;
            if let Some(user) = auth.current_user.as_mut() {
                user.profile_json = Some(new_profile);
            }
        }
        Err(e) => log_warn!("Failed to refresh profile after cape unequip: {}", e),
    }

    Ok(())
}

/// 将 data URL（如 data:image/png;base64,xxxx）保存到本地文件
///
/// 用于"下载当前皮肤到本地"功能：前端已通过 download_skin_png 拿到 dataURL，
/// 用户选择保存位置后调用此命令写入文件。
#[tauri::command]
pub async fn save_data_url_to_file(data_url: String, path: String) -> Result<(), String> {
    // 解析 data URL：data:image/png;base64,<base64 数据>
    let base64_data = data_url
        .find(",")
        .map(|i| &data_url[i + 1..])
        .ok_or_else(|| "Invalid data URL: missing comma".to_string())?;

    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data.trim())
        .map_err(|e| format!("Base64 解码失败: {}", e))?;

    std::fs::write(&path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;

    crate::log_info!("[Skin] Saved data URL to: {} ({} bytes)", path, bytes.len());
    Ok(())
}
