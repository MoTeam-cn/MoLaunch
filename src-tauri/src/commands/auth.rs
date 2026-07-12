//! 认证命令
//!
//! 支持离线登录和微软登录（Device Code Flow）。
//! 微软登录流程：
//! 1. 前端调用 `ms_login_start` 获取设备码
//! 2. 前端展示设备码并打开浏览器，用户授权
//! 3. 前端轮询调用 `ms_login_poll` 直到成功/失败
//! 4. 成功后 Token 自动持久化，支持会话恢复
//! 5. Token 过期时调用 `ms_login_refresh` 静默刷新

use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::microsoft;
use crate::state::{AppState, LocalAuthResult};
use serde::Serialize;
use tauri::State;

// ============================================================
// 离线登录
// ============================================================

/// 离线登录
#[tauri::command]
pub async fn login_offline(
    state: State<'_, AppState>,
    username: String,
) -> Result<LocalAuthResult, String> {
    log_info!("Offline login attempt for user: {}", username);

    // 验证用户名
    if !crate::minecraft::auth::validate_username(&username) {
        return Err(
            "Username must be 3-16 characters and contain only letters, numbers, and underscores"
                .to_string(),
        );
    }

    // 使用本地实现进行离线登录
    let result = crate::minecraft::auth::login_offline(&username);

    // 转换为本地认证结果
    let auth_result = LocalAuthResult {
        name: result.name.clone(),
        uuid: result.uuid.clone(),
        access_token: result.access_token,
        client_token: result.client_token,
        login_type: "Legacy".to_string(),
        profile_json: None,
    };

    // 保存认证状态到内存
    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    // 持久化到磁盘
    if let Err(e) = state
        .auth_storage
        .save_offline_login(&username, &result.uuid)
        .await
    {
        log_warn!("Failed to persist offline login: {}", e);
    }

    log_info!("Offline login successful for user: {}", username);
    Ok(auth_result)
}

// ============================================================
// 微软登录 - 设备码流程
// ============================================================

/// 设备码信息（返回给前端展示）
#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

/// 微软登录步骤 1：申请设备码
#[tauri::command]
pub async fn ms_login_start() -> Result<DeviceCodeInfo, String> {
    log_info!("Starting Microsoft device code login flow");

    let response = microsoft::request_device_code()
        .await
        .map_err(|e| e.to_string())?;

    Ok(DeviceCodeInfo {
        user_code: response.user_code,
        verification_uri: response.verification_uri,
        device_code: response.device_code,
        expires_in: response.expires_in,
        interval: response.interval,
        message: response.message,
    })
}

/// 轮询结果
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum PollResult {
    /// 等待用户授权中，继续轮询
    Pending,
    /// 授权成功，登录完成
    Success {
        name: String,
        uuid: String,
        access_token: String,
        client_token: String,
        login_type: String,
        profile_json: Option<String>,
    },
}

/// 微软登录步骤 2：轮询设备码授权结果
///
/// 前端每隔 interval 秒调用一次，直到返回 Success 或报错。
/// 轮询成功后自动完成后续 Token 交换链并持久化。
#[tauri::command]
pub async fn ms_login_poll(
    state: State<'_, AppState>,
    device_code: String,
) -> Result<PollResult, String> {
    log_info!("Polling Microsoft device code authorization");

    let poll_result = microsoft::poll_device_code(&device_code)
        .await
        .map_err(|e| e.to_string())?;

    match poll_result {
        Some(oauth_response) => {
            // 授权成功，完成后续 Token 交换链
            log_info!("Device code authorized, completing login chain");

            let refresh_token = oauth_response.refresh_token.as_deref().unwrap_or("");

            let login_result =
                microsoft::complete_login_chain(&oauth_response.access_token, refresh_token)
                    .await
                    .map_err(|e| e.to_string())?;

            // 持久化到磁盘
            if let Err(e) = state.auth_storage.save_ms_login(&login_result).await {
                log_warn!("Failed to persist Microsoft login: {}", e);
            }

            // 更新内存状态
            let auth_result = LocalAuthResult {
                name: login_result.username.clone(),
                uuid: login_result.uuid.clone(),
                access_token: login_result.access_token.clone(),
                client_token: String::new(),
                login_type: "Microsoft".to_string(),
                profile_json: Some(login_result.profile_json.clone()),
            };

            {
                let mut auth = state.auth.lock().await;
                auth.current_user = Some(auth_result.clone());
                auth.is_logged_in = true;
            }

            log_info!(
                "Microsoft login successful: user={}, uuid={}",
                login_result.username,
                login_result.uuid
            );

            Ok(PollResult::Success {
                name: auth_result.name,
                uuid: auth_result.uuid,
                access_token: auth_result.access_token,
                client_token: auth_result.client_token,
                login_type: auth_result.login_type,
                profile_json: auth_result.profile_json,
            })
        }
        None => {
            // 授权待定，继续轮询
            Ok(PollResult::Pending)
        }
    }
}

/// 微软登录：使用 Refresh Token 静默刷新
///
/// 当 Token 过期时调用，无需用户交互。
/// 如果 Refresh Token 也已失效，返回错误要求重新登录。
#[tauri::command]
pub async fn ms_login_refresh(state: State<'_, AppState>) -> Result<LocalAuthResult, String> {
    log_info!("Attempting silent Microsoft token refresh");

    // 获取当前用户的 refresh_token
    let refresh_token = state
        .auth_storage
        .get_current_refresh_token()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "当前未登录微软账号或无 refresh token".to_string())?;

    let login_result = microsoft::login_with_refresh_token(&refresh_token)
        .await
        .map_err(|e| e.to_string())?;

    // 更新持久化的 Token
    if let Err(e) = state
        .auth_storage
        .update_ms_token(
            &login_result.uuid,
            &login_result.access_token,
            &login_result.refresh_token,
            login_result.expires_at,
        )
        .await
    {
        log_warn!("Failed to update persisted token: {}", e);
    }

    // 更新内存状态
    let auth_result = LocalAuthResult {
        name: login_result.username.clone(),
        uuid: login_result.uuid.clone(),
        access_token: login_result.access_token.clone(),
        client_token: String::new(),
        login_type: "Microsoft".to_string(),
        profile_json: Some(login_result.profile_json.clone()),
    };

    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    log_info!("Microsoft token refreshed successfully");
    Ok(auth_result)
}

// ============================================================
// 已存储的微软账号管理
// ============================================================

/// 已存储的微软账号信息（列表用）
#[derive(Debug, Clone, Serialize)]
pub struct MsAccountInfo {
    pub username: String,
    pub uuid: String,
    pub expires_at: u64,
    pub is_expired: bool,
}

/// 获取已存储的微软账号列表
#[tauri::command]
pub async fn get_ms_accounts(state: State<'_, AppState>) -> Result<Vec<MsAccountInfo>, String> {
    let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;

    let accounts: Vec<MsAccountInfo> = persisted
        .ms_accounts
        .iter()
        .map(|a| MsAccountInfo {
            username: a.username.clone(),
            uuid: a.uuid.clone(),
            expires_at: a.expires_at,
            is_expired: microsoft::is_token_expired(a.expires_at),
        })
        .collect();

    Ok(accounts)
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

/// 切换到已存储的微软账号（设为当前用户）
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
        .ok_or_else(|| "未找到指定的微软账号".to_string())?;

    // 如果 Token 已过期，尝试静默刷新
    let (access_token, refresh_token, expires_at) =
        if microsoft::is_token_expired(account.expires_at) {
            log_info!("Token expired, refreshing...");
            let login_result = microsoft::login_with_refresh_token(&account.refresh_token)
                .await
                .map_err(|e| e.to_string())?;

            // 更新持久化
            if let Err(e) = state
                .auth_storage
                .update_ms_token(
                    &uuid,
                    &login_result.access_token,
                    &login_result.refresh_token,
                    login_result.expires_at,
                )
                .await
            {
                log_warn!("Failed to update persisted token: {}", e);
            }

            (
                login_result.access_token,
                login_result.refresh_token,
                login_result.expires_at,
            )
        } else {
            (
                account.access_token,
                account.refresh_token,
                account.expires_at,
            )
        };

    let auth_result = LocalAuthResult {
        name: account.username.clone(),
        uuid: account.uuid.clone(),
        access_token,
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
            access_token: auth_result.access_token.clone(),
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

    // 更新内存状态
    {
        let mut auth = state.auth.lock().await;
        auth.current_user = Some(auth_result.clone());
        auth.is_logged_in = true;
    }

    log_info!("Switched to Microsoft account: {}", account.username);
    Ok(auth_result)
}

// ============================================================
// 通用认证命令
// ============================================================

/// 获取当前登录状态
///
/// 优先从内存读取；如果内存为空，尝试从磁盘恢复（会话恢复）。
#[tauri::command]
pub async fn get_login_status(
    state: State<'_, AppState>,
) -> Result<Option<LocalAuthResult>, String> {
    // 先检查内存
    {
        let auth = state.auth.lock().await;
        if auth.current_user.is_some() {
            return Ok(auth.current_user.clone());
        }
    }

    // 内存为空，尝试从磁盘恢复
    let persisted = state.auth_storage.load().await.map_err(|e| e.to_string())?;

    if let Some(user) = persisted.current_user {
        // 如果是微软登录且 Token 已过期，尝试静默刷新
        if user.login_type == "Microsoft" {
            if let Some(expires_at) = user.expires_at {
                if microsoft::is_token_expired(expires_at) {
                    if let Some(refresh_token) = &user.refresh_token {
                        log_info!("Token expired on restore, attempting silent refresh...");
                        match microsoft::login_with_refresh_token(refresh_token).await {
                            Ok(login_result) => {
                                // 更新持久化
                                if let Err(e) = state
                                    .auth_storage
                                    .update_ms_token(
                                        &user.uuid,
                                        &login_result.access_token,
                                        &login_result.refresh_token,
                                        login_result.expires_at,
                                    )
                                    .await
                                {
                                    log_warn!("Failed to update persisted token: {}", e);
                                }

                                let auth_result = LocalAuthResult {
                                    name: login_result.username.clone(),
                                    uuid: login_result.uuid.clone(),
                                    access_token: login_result.access_token.clone(),
                                    client_token: String::new(),
                                    login_type: "Microsoft".to_string(),
                                    profile_json: Some(login_result.profile_json.clone()),
                                };

                                let mut auth = state.auth.lock().await;
                                auth.current_user = Some(auth_result.clone());
                                auth.is_logged_in = true;

                                return Ok(Some(auth_result));
                            }
                            Err(e) => {
                                log_warn!("Silent refresh failed on restore: {}", e);
                                // 刷新失败，返回 None 要求重新登录
                                return Ok(None);
                            }
                        }
                    }
                }
            }
        }

        // 离线登录或微软登录 Token 未过期
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
    // 清除内存状态
    {
        let mut auth = state.auth.lock().await;
        auth.current_user = None;
        auth.is_logged_in = false;
    }

    // 清除持久化的当前用户
    if let Err(e) = state.auth_storage.clear_current_user().await {
        log_warn!("Failed to clear persisted auth: {}", e);
    }

    log_info!("User logged out");
    Ok(())
}
