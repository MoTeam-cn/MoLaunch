//! 账号管理命令（列表/删除/切换/登出/状态恢复）

use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::microsoft;
use crate::minecraft::auth::storage::StoredOfflineAccount;
use crate::state::{AppState, LocalAuthResult};
use serde::Serialize;
use tauri::State;

/// 已存储的微软账号信息
#[derive(Debug, Clone, Serialize)]
pub struct MsAccountInfo {
    pub username: String,
    pub uuid: String,
    pub expires_at: u64,
    pub is_expired: bool,
}

/// 已存储的离线账号信息
#[derive(Debug, Clone, Serialize)]
pub struct OfflineAccountInfo {
    pub username: String,
    pub uuid: String,
    pub skin: Option<String>,
}

/// 获取已存储的微软账号列表
#[tauri::command]
pub async fn get_ms_accounts(state: State<'_, AppState>) -> Result<Vec<MsAccountInfo>, String> {
    log_info!("[Startup][IPC] get_ms_accounts called");
    let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;
    Ok(persisted
        .ms_accounts
        .iter()
        .map(|a| MsAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            expires_at: a.expires_at,
            is_expired: microsoft::is_token_expired(a.expires_at),
        })
        .collect())
}

/// 删除已存储的微软账号
#[tauri::command]
pub async fn remove_ms_account(state: State<'_, AppState>, uuid: String) -> Result<(), String> {
    log_info!("Removing Microsoft account: {}", uuid);
    state
        .auth_storage
        .remove_ms_account(&uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 切换到已存储的微软账号
#[tauri::command]
pub async fn switch_ms_account(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<LocalAuthResult, String> {
    log_info!("Switching to Microsoft account: {}", uuid);

    let account = state
        .auth_storage
        .get_ms_account(&uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Account not found".to_string())?;

    let (access_token, refresh_token, expires_at) =
        if microsoft::is_token_expired(account.expires_at) {
            log_info!("Token expired, refreshing...");
            let r = microsoft::login_with_refresh_token(&account.refresh_token, |_| {})
                .await
                .map_err(|e| e.to_string())?;
            if let Err(e) = state
                .auth_storage
                .update_ms_token(&uuid, &r.access_token, &r.refresh_token, r.expires_at)
                .await
            {
                log_warn!("Failed to update persisted token: {}", e);
            }
            (r.access_token, r.refresh_token, r.expires_at)
        } else {
            (
                account.access_token.clone(),
                account.refresh_token,
                account.expires_at,
            )
        };

    let auth_result = LocalAuthResult {
        name: account.username.clone(),
        uuid: account.uuid.clone(),
        access_token: access_token.clone(),
        client_token: String::new(),
        login_type: "Microsoft".to_string(),
        profile_json: Some(account.profile_json.clone()),
    };

    // 更新当前用户（持久化）
    {
        let mut persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;
        persisted.current_user = Some(crate::minecraft::auth::storage::CurrentUser {
            name: account.username.clone(),
            uuid: account.uuid.clone(),
            access_token,
            client_token: String::new(),
            login_type: "Microsoft".to_string(),
            profile_json: Some(account.profile_json.clone()),
            refresh_token: Some(refresh_token),
            expires_at: Some(expires_at),
        });
        state
            .auth_storage
            .save(&persisted)
            .await
            .map_err(|e| e.to_string())?;
    }

    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    log_info!("Switched to Microsoft account: {}", account.username);
    Ok(auth_result)
}

/// 获取已存储的离线账号列表
#[tauri::command]
pub async fn get_offline_accounts(
    state: State<'_, AppState>,
) -> Result<Vec<OfflineAccountInfo>, String> {
    log_info!("[Startup][IPC] get_offline_accounts called");
    let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;
    Ok(persisted
        .offline_accounts
        .iter()
        .map(|a| OfflineAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            skin: a.skin.clone(),
        })
        .collect())
}

/// 设置离线账号的皮肤选择
#[tauri::command]
pub async fn set_offline_skin(
    state: State<'_, AppState>,
    uuid: String,
    skin: Option<String>,
) -> Result<(), String> {
    log_info!("Setting offline skin: uuid={}, skin={:?}", uuid, skin);
    state
        .auth_storage
        .set_offline_skin(&uuid, skin.as_deref())
        .await
        .map_err(|e| e.to_string())
}

/// 保存自定义皮肤文件并设置到离线账号
///
/// 将用户选择的 PNG 文件复制到 `<app_data>/custom_skins/<uuid>.png`，
/// 然后把 `custom:<path>|<variant>` 写入离线账号的 skin 字段。
#[tauri::command]
pub async fn save_custom_skin(
    state: State<'_, AppState>,
    uuid: String,
    file_path: String,
    variant: Option<String>,
) -> Result<String, String> {
    let variant = variant.unwrap_or_else(|| "classic".to_string());

    // 读取源文件
    let png_data = std::fs::read(&file_path).map_err(|e| format!("读取皮肤文件失败: {}", e))?;

    // 验证 PNG 文件头
    if png_data.len() < 8 || png_data[0..5] != [0x89, 0x50, 0x4E, 0x47, 0x0D] {
        return Err("文件不是有效的 PNG 格式".to_string());
    }

    // 验证文件大小（< 1MB，比 Mojang 的 24KB 宽松，因为是本地使用）
    if png_data.len() > 1024 * 1024 {
        return Err("皮肤文件过大（超过 1MB）".to_string());
    }

    // 保存到 app data 目录
    let skin_dir = crate::storage::Storage::instance()
        .base_dir()
        .join("custom_skins");
    std::fs::create_dir_all(&skin_dir).map_err(|e| format!("创建皮肤目录失败: {}", e))?;

    let dest_path = skin_dir.join(format!("{}.png", uuid));
    std::fs::write(&dest_path, &png_data).map_err(|e| format!("保存皮肤文件失败: {}", e))?;

    // 构建 skin 字段：custom:/path|slim 或 custom:/path|classic
    let skin_value = format!("custom:{}|{}", dest_path.display(), variant);

    // 写入注册表
    state
        .auth_storage
        .set_offline_skin(&uuid, Some(&skin_value))
        .await
        .map_err(|e| e.to_string())?;

    log_info!(
        "Saved custom skin: uuid={}, file={}, variant={}",
        uuid,
        dest_path.display(),
        variant
    );

    Ok(skin_value)
}

/// 删除已存储的离线账号
#[tauri::command]
pub async fn remove_offline_account(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<(), String> {
    log_info!("Removing offline account: {}", uuid);
    state
        .auth_storage
        .remove_offline_account(&uuid)
        .await
        .map_err(|e| e.to_string())
}

/// 切换到已存储的离线账号
#[tauri::command]
pub async fn switch_offline_account(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<LocalAuthResult, String> {
    log_info!("Switching to offline account: {}", uuid);

    let account: StoredOfflineAccount = state
        .auth_storage
        .get_offline_account(&uuid)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "离线账号不存在".to_string())?;

    let auth_result = LocalAuthResult {
        name: account.username.clone(),
        uuid: account.uuid.clone(),
        access_token: account.uuid.clone(),
        client_token: account.uuid.clone(),
        login_type: "Legacy".to_string(),
        profile_json: None,
    };

    // 更新当前用户（持久化）
    {
        let mut persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;
        persisted.current_user = Some(crate::minecraft::auth::storage::CurrentUser {
            name: account.username.clone(),
            uuid: account.uuid.clone(),
            access_token: account.uuid.clone(),
            client_token: account.uuid.clone(),
            login_type: "Legacy".to_string(),
            profile_json: None,
            refresh_token: None,
            expires_at: None,
        });
        state
            .auth_storage
            .save(&persisted)
            .await
            .map_err(|e| e.to_string())?;
    }

    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    log_info!("Switched to offline account: {}", account.username);
    Ok(auth_result)
}

/// 获取当前登录状态（优先内存，其次磁盘恢复）
#[tauri::command]
pub async fn get_login_status(
    state: State<'_, AppState>,
) -> Result<Option<LocalAuthResult>, String> {
    log_info!("[Startup][IPC] get_login_status called");
    {
        let auth = state.auth.lock().await;
        if auth.current_user.is_some() {
            log_info!("[Startup][IPC] get_login_status: returning in-memory user");
            return Ok(auth.current_user.clone());
        }
    }

    let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;

    if let Some(user) = persisted.current_user {
        if user.login_type == "Microsoft" {
            if let (Some(expires_at), Some(refresh_token)) = (user.expires_at, &user.refresh_token)
            {
                if microsoft::is_token_expired(expires_at) {
                    log_info!("Token expired on restore, attempting silent refresh...");
                    match microsoft::login_with_refresh_token(refresh_token, |_| {}).await {
                        Ok(r) => {
                            if let Err(e) = state
                                .auth_storage
                                .update_ms_token(
                                    &user.uuid,
                                    &r.access_token,
                                    &r.refresh_token,
                                    r.expires_at,
                                )
                                .await
                            {
                                log_warn!("Failed to update persisted token: {}", e);
                            }
                            let auth_result = LocalAuthResult {
                                name: r.username.clone(),
                                uuid: r.uuid.clone(),
                                access_token: r.access_token.clone(),
                                client_token: String::new(),
                                login_type: "Microsoft".to_string(),
                                profile_json: Some(r.profile_json.clone()),
                            };
                            let mut auth = state.auth.lock().await;
                            auth.current_user = Some(auth_result.clone());
                            auth.is_logged_in = true;
                            return Ok(Some(auth_result));
                        }
                        Err(e) => log_warn!("Silent refresh failed on restore: {}", e),
                    }
                }
            }
        }

        let auth_result = LocalAuthResult {
            name: user.name,
            uuid: user.uuid,
            access_token: user.access_token,
            client_token: user.client_token,
            login_type: user.login_type,
            profile_json: user.profile_json,
        };
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
        Ok(Some(auth_result))
    } else {
        Ok(None)
    }
}

/// 登出
#[tauri::command]
pub async fn logout(state: State<'_, AppState>) -> Result<(), String> {
    {
        let mut auth = state.auth.lock().await;
        auth.current_user = None;
        auth.is_logged_in = false;
    }
    if let Err(e) = state.auth_storage.clear_current_user().await {
        log_warn!("Failed to clear persisted auth: {}", e);
    }
    log_info!("User logged out");
    Ok(())
}
