//! 微软登录命令
//!
//! 两种流程：
//! - **Web Auth Code Flow**（官方 ID）：浏览器授权 → 拦截回调 code → 换取 token → 完成登录链
//! - **Device Code Flow**（自定义 ID）：设备码 → 用户浏览器输入 → 轮询 → 完成登录链

use crate::log_info;
use crate::log_warn;
use crate::minecraft::auth::microsoft;
use crate::state::{AppState, LocalAuthResult};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

/// 登录流程配置（前端根据此结果决定使用哪种 UI）
#[derive(Debug, Clone, Serialize)]
pub struct LoginConfig {
    /// "web" = Web Auth Code Flow, "device_code" = Device Code Flow
    pub flow: String,
}

/// 设备码信息（返回给前端显示）
#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeInfo {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

/// 轮询结果
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status")]
pub enum PollResult {
    Pending,
    Success { auth: LocalAuthResult },
    Declined,
    Expired,
    Error { message: String },
}

/// 获取登录流程配置
#[tauri::command]
pub async fn ms_login_get_config() -> Result<LoginConfig, String> {
    let flow = if microsoft::is_official_client() {
        "web"
    } else {
        "device_code"
    };
    Ok(LoginConfig {
        flow: flow.to_string(),
    })
}

/// Web Auth Code Flow：打开 Webview 窗口让用户登录
#[tauri::command]
pub async fn ms_login_web_start(app: AppHandle) -> Result<(), String> {
    use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

    let auth_url = microsoft::build_auth_url();
    let ep = microsoft::config::endpoints();
    let redirect_uri = ep.redirect_uri.to_string();
    let app_handle = app.clone();

    log_info!("Opening web auth window: {}", auth_url);

    let url = tauri::Url::parse(&auth_url).map_err(|e| e.to_string())?;

    WebviewWindowBuilder::new(&app, "ms-auth", WebviewUrl::External(url))
        .title("Microsoft Login")
        .inner_size(800.0, 600.0)
        .on_navigation(move |url| {
            if url.as_str().starts_with(&redirect_uri) {
                if let Some(code) = url
                    .query_pairs()
                    .find(|(k, _)| k == "code")
                    .map(|(_, v)| v.to_string())
                {
                    let _ = app_handle.emit("ms-auth-code", code);
                }
                if let Some(win) = app_handle.get_webview_window("ms-auth") {
                    let _ = win.close();
                }
                false
            } else {
                true
            }
        })
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Web Auth Code Flow：用授权码完成登录链
#[tauri::command]
pub async fn ms_login_web_exchange(
    app: AppHandle,
    state: State<'_, AppState>,
    code: String,
) -> Result<PollResult, String> {
    let _ = app.emit("ms-login-progress", "exchanging");
    let app_handle = app.clone();

    let oauth_token = match microsoft::exchange_auth_code(&code).await {
        Ok(t) => t,
        Err(e) => return Ok(PollResult::Error { message: e.to_string() }),
    };

    let refresh_token = oauth_token.refresh_token.clone().unwrap_or_default();
    complete_login(&app_handle, &state, &oauth_token.access_token, &refresh_token).await
}

/// Device Code Flow：申请设备码
#[tauri::command]
pub async fn ms_login_request_device_code() -> Result<DeviceCodeInfo, String> {
    log_info!("Requesting Microsoft device code");
    let r = microsoft::request_device_code().await.map_err(|e| e.to_string())?;
    Ok(DeviceCodeInfo {
        user_code: r.user_code,
        verification_uri: r.verification_uri,
        device_code: r.device_code,
        expires_in: r.expires_in,
        interval: r.interval,
        message: r.message,
    })
}

/// Device Code Flow：轮询授权状态
#[tauri::command]
pub async fn ms_login_poll(
    app: AppHandle,
    state: State<'_, AppState>,
    device_code: String,
) -> Result<PollResult, String> {
    let poll_result = match microsoft::poll_device_code(&device_code).await {
        Ok(result) => result,
        Err(e) => match e.error_code.as_deref() {
            Some("authorization_declined") => return Ok(PollResult::Declined),
            Some("expired_token") => return Ok(PollResult::Expired),
            _ => return Ok(PollResult::Error { message: e.to_string() }),
        },
    };

    match poll_result {
        None => Ok(PollResult::Pending),
        Some(token) => {
            let _ = app.emit("ms-login-progress", "exchanging");
            let refresh_token = token.refresh_token.clone().unwrap_or_default();
            complete_login(&app, &state, &token.access_token, &refresh_token).await
        }
    }
}

/// 微软登录：使用 Refresh Token 静默刷新
#[tauri::command]
pub async fn ms_login_refresh(state: State<'_, AppState>) -> Result<LocalAuthResult, String> {
    log_info!("Attempting silent Microsoft token refresh");
    let refresh_token = state.auth_storage.get_current_refresh_token().await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No refresh token available".to_string())?;

    let result = microsoft::login_with_refresh_token(&refresh_token, |_| {})
        .await.map_err(|e| e.to_string())?;

    if let Err(e) = state.auth_storage.update_ms_token(
        &result.uuid, &result.access_token, &result.refresh_token, result.expires_at
    ).await { log_warn!("Failed to update persisted token: {}", e); }

    let auth_result = to_local_auth(&result);
    { let mut auth = state.auth.lock().await;
      auth.current_user = Some(auth_result.clone()); auth.is_logged_in = true; }
    log_info!("Microsoft token refreshed successfully");
    Ok(auth_result)
}

/// 转换为 LocalAuthResult
fn to_local_auth(r: &microsoft::MicrosoftLoginResult) -> LocalAuthResult {
    LocalAuthResult {
        name: r.username.clone(), uuid: r.uuid.clone(),
        access_token: r.access_token.clone(), client_token: String::new(),
        login_type: "Microsoft".to_string(), profile_json: Some(r.profile_json.clone()),
    }
}

/// 完成 Token 交换链并持久化（Web Flow 和 Device Code Flow 共用）
async fn complete_login(
    app: &AppHandle, state: &State<'_, AppState>,
    access_token: &str, refresh_token: &str,
) -> Result<PollResult, String> {
    let app_handle = app.clone();
    match microsoft::complete_login_chain(access_token, refresh_token, |step| {
        let _ = app_handle.emit("ms-login-progress", step);
    }).await {
        Ok(login_result) => {
            if let Err(e) = state.auth_storage.save_ms_login(&login_result).await {
                log_warn!("Failed to persist Microsoft login: {}", e);
            }
            let auth_result = to_local_auth(&login_result);
            { let mut auth = state.auth.lock().await;
              auth.current_user = Some(auth_result.clone()); auth.is_logged_in = true; }
            log_info!("Microsoft login successful: user={}", login_result.username);
            Ok(PollResult::Success { auth: auth_result })
        }
        Err(e) => { log_warn!("Login chain failed: {}", e);
            Ok(PollResult::Error { message: e.to_string() }) }
    }
}
