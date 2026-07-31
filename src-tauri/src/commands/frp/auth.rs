//! Frp 厂商认证模块
//!
//! 实现 OAuth2 / Device Code / API Key 三种认证流程，token 使用 OS 密钥存储
//! （Windows Credential Manager / macOS Keychain / Linux Secret Service）。
//!
//! 参见 FRP_MANAGER_DESIGN.md §6（认证体系设计）。
//!
//! keyring key 格式：service=`frp:<provider_id>`，username=`access_token` /
//! `refresh_token` / `expires_at` / `scopes`。token 过期前 5 分钟自动刷新。
//!
//! 浏览器跳转走 `crate::minecraft::system::shell::open_url`（项目约束），
//! HTTP 请求复用 `crate::http::get_client()`，OAuth2 回调用 `tokio::net::TcpListener`
//! 监听 127.0.0.1。

use super::provider::{read_provider_manifest, SYSTEM_DEFAULT_ID};
use super::{ApiKeyConfig, AuthConfig, DeviceCodeConfig, OAuth2Config};
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::state::AppState;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// 返回类型
// ============================================================

/// 认证状态（get_auth_status 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub provider_id: String,
    /// 是否已认证（有有效 token）
    pub authenticated: bool,
    /// 认证类型：none / oauth2 / device_code / api_key
    pub auth_type: String,
    /// token 过期时间（Unix 秒），已过期时仍返回供前端展示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// OAuth2 流程结果（start_oauth2 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2Result {
    /// token 过期时间（Unix 秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// Device Code 流程启动结果（start_device_code 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeResult {
    /// 用户码（前端显示给用户输入）
    pub user_code: String,
    /// 验证链接（用户访问此 URL 输入用户码）
    pub verification_uri: String,
    /// 过期时间（秒）
    pub expires_in: u64,
    /// 轮询间隔（秒）
    pub interval: u64,
}

/// Device Code 轮询结果（poll_device_code 返回）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodePollResult {
    /// 状态：pending / success / expired / declined / slow_down
    pub status: String,
    /// token 过期时间（仅 status=success 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// 权限范围（仅 status=success 时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

// ============================================================
// Device Code 会话存储（内存，进程级）
// ============================================================

/// Device Code 会话（start_device_code 写入，poll_device_code 读取）
#[derive(Clone)]
struct DeviceCodeSession {
    device_code: String,
    /// 会话过期时间（Unix 秒）
    expires_at: u64,
    /// 轮询间隔（秒，存储供前端查询，poll_device_code 不直接使用）
    #[allow(dead_code)]
    interval: u64,
    /// tokenUrl（从 manifest 读取，避免轮询时重复读取）
    token_url: String,
    /// clientId
    client_id: String,
}

static DEVICE_CODE_SESSIONS: Lazy<Mutex<HashMap<String, DeviceCodeSession>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// ============================================================
// keyring 密钥存储辅助
// ============================================================

/// 密钥存储的 key 列表
const KEY_ACCESS_TOKEN: &str = "access_token";
const KEY_REFRESH_TOKEN: &str = "refresh_token";
const KEY_EXPIRES_AT: &str = "expires_at";
const KEY_SCOPES: &str = "scopes";

/// 构造 keyring Entry
///
/// service = `frp:<provider_id>`，username = 具体键名。
/// keyring 不可用时返回明确错误。
fn keyring_entry(provider_id: &str, key: &str) -> Result<keyring::Entry, String> {
    let service = format!("frp:{}", provider_id);
    keyring::Entry::new(&service, key).map_err(|e| {
        log_error!("[Frp Auth] keyring 不可用 (provider={}): {}", provider_id, e);
        format!("OS 密钥存储不可用: {}", e)
    })
}

/// 存储 token 值到 keyring
fn store_secret(provider_id: &str, key: &str, value: &str) -> Result<(), String> {
    let entry = keyring_entry(provider_id, key)?;
    entry.set_password(value).map_err(|e| {
        log_error!("[Frp Auth] 存储密钥失败 (provider={}, key={}): {}", provider_id, key, e);
        format!("存储密钥失败: {}", e)
    })
}

/// 读取 token 值（不存在返回 None，keyring 不可用返回 Err）
fn load_secret(provider_id: &str, key: &str) -> Result<Option<String>, String> {
    let entry = keyring_entry(provider_id, key)?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => {
            log_error!("[Frp Auth] 读取密钥失败 (provider={}, key={}): {}", provider_id, key, e);
            Err(format!("OS 密钥存储不可用: {}", e))
        }
    }
}

/// 删除 token 值（不存在视为成功）
fn delete_secret(provider_id: &str, key: &str) -> Result<(), String> {
    let entry = keyring_entry(provider_id, key)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("删除密钥失败: {}", e)),
    }
}

// ============================================================
// token 存储辅助（封装 access_token / refresh_token / expires_at / scopes）
// ============================================================

/// 存储完整 token 信息
fn store_token_info(
    provider_id: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<u64>,
    scopes: Option<&Vec<String>>,
) -> Result<(), String> {
    store_secret(provider_id, KEY_ACCESS_TOKEN, access_token)?;
    if let Some(rt) = refresh_token {
        store_secret(provider_id, KEY_REFRESH_TOKEN, rt)?;
    }
    if let Some(secs) = expires_in {
        let expires_at = now_secs() + secs;
        store_secret(provider_id, KEY_EXPIRES_AT, &expires_at.to_string())?;
    }
    if let Some(sc) = scopes {
        let json = serde_json::to_string(sc).map_err(|e| format!("序列化 scopes 失败: {}", e))?;
        store_secret(provider_id, KEY_SCOPES, &json)?;
    }
    Ok(())
}

/// 读取 token 过期时间（Unix 秒）
fn load_expires_at(provider_id: &str) -> Result<Option<u64>, String> {
    match load_secret(provider_id, KEY_EXPIRES_AT)? {
        Some(s) => s.parse::<u64>().map(Some).map_err(|e| format!("解析过期时间失败: {}", e)),
        None => Ok(None),
    }
}

/// 读取权限范围
fn load_scopes(provider_id: &str) -> Result<Option<Vec<String>>, String> {
    match load_secret(provider_id, KEY_SCOPES)? {
        Some(s) => {
            let scopes: Vec<String> =
                serde_json::from_str(&s).map_err(|e| format!("解析 scopes 失败: {}", e))?;
            Ok(Some(scopes))
        }
        None => Ok(None),
    }
}

// ============================================================
// 公共函数
// ============================================================

/// 查询指定厂商的认证状态
///
/// - auth_type=none：始终 authenticated=true
/// - auth_type=oauth2/device_code：检查 access_token 是否存在且未过期
/// - auth_type=api_key：检查 access_token（即 API Key）是否存在
///
/// expires_at 即使已过期也会返回，前端据此区分「即将过期」/「已过期」。
pub async fn get_auth_status(provider_id: &str) -> Result<AuthStatus, String> {
    // 系统默认厂商无需认证
    if provider_id == SYSTEM_DEFAULT_ID {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type: "none".to_string(),
            expires_at: None,
            scopes: None,
        });
    }

    let manifest = read_provider_manifest(provider_id)?;
    let auth_type = manifest.auth.auth_type.clone();

    if auth_type == "none" {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type,
            expires_at: None,
            scopes: None,
        });
    }

    // 检查 access_token 是否存在
    let access_token = load_secret(provider_id, KEY_ACCESS_TOKEN)?;
    let authenticated = access_token.is_some();

    // 检查是否过期（仅 oauth2 / device_code 有过期时间）
    let expires_at = if matches!(auth_type.as_str(), "oauth2" | "device_code") {
        load_expires_at(provider_id)?
    } else {
        None
    };

    // token 存在但已过期 -> authenticated=false
    let authenticated = if authenticated {
        match expires_at {
            Some(exp) => exp > now_secs(),
            None => true, // api_key 无过期时间
        }
    } else {
        false
    };

    let scopes = load_scopes(provider_id)?;

    Ok(AuthStatus {
        provider_id: provider_id.to_string(),
        authenticated,
        auth_type,
        expires_at,
        scopes,
    })
}

/// 启动 OAuth2 授权流程
///
/// 流程（参见 §6.3）：
/// 1. 本地启动 HTTP 服务监听 127.0.0.1:redirectPort
/// 2. 通过 shell 模块打开浏览器跳转授权页
/// 3. 等待浏览器回调（最长 5 分钟）
/// 4. 用 code 换取 token
/// 5. 存储 token 到 OS 密钥存储
pub async fn start_oauth2(_state: &AppState, provider_id: &str) -> Result<OAuth2Result, String> {
    let manifest = read_provider_manifest(provider_id)?;
    let config = require_oauth2_config(&manifest.auth, provider_id)?;

    log_info!("[Frp Auth] 启动 OAuth2 流程: provider={}", provider_id);

    // 1. 启动本地 HTTP 服务接收 callback
    let bind_addr = format!("127.0.0.1:{}", config.redirect_port);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| format!("绑定回调端口 {} 失败: {}", config.redirect_port, e))?;
    log_debug!("[Frp Auth] 回调服务监听: {}", bind_addr);

    let redirect_uri = format!("http://localhost:{}", config.redirect_port);

    // 2. 生成 state（CSRF 防护）并构建授权 URL
    let state = generate_state();
    let authorize_url = build_authorize_url(&config.authorize_url, &config.client_id, &redirect_uri, &config.scopes, &state);

    // 3. 打开浏览器（走 shell 模块）
    crate::minecraft::system::shell::open_url(&authorize_url)
        .map_err(|e| format!("打开浏览器失败: {}", e))?;

    // 4. 等待回调（5 分钟超时）
    let callback = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        wait_for_callback(&listener, &state),
    )
    .await
    .map_err(|_| {
        log_error!("[Frp Auth] OAuth2 回调超时: provider={}", provider_id);
        "OAuth2 授权超时（5 分钟内未完成）".to_string()
    })??;

    // 5. 用 code 换取 token
    let token_resp = exchange_code_for_token(
        &config.token_url,
        &config.client_id,
        &redirect_uri,
        &callback.code,
    )
    .await?;

    // 6. 存储 token
    let expires_at = token_resp.expires_in.map(|secs| now_secs() + secs);
    let scopes = token_resp.scope.as_ref().map(|s| parse_scopes(s));
    let scopes_for_store = scopes.as_ref().or(Some(&config.scopes));
    let access_token = token_resp.access_token.as_deref().ok_or("OAuth2 响应缺少 access_token")?;
    store_token_info(
        provider_id,
        access_token,
        token_resp.refresh_token.as_deref(),
        token_resp.expires_in,
        scopes_for_store,
    )?;

    log_info!(
        "[Frp Auth] OAuth2 认证成功: provider={}, expires_at={:?}",
        provider_id, expires_at
    );

    Ok(OAuth2Result { expires_at, scopes })
}

/// 启动 Device Code 流程
///
/// 流程（参见 §6.4）：
/// 1. POST deviceCodeUrl 获取设备码
/// 2. 返回用户码 + 验证链接 + 倒计时给前端展示
/// 3. 将 device_code 存入内存会话（供 poll_device_code 使用）
pub async fn start_device_code(_state: &AppState, provider_id: &str) -> Result<DeviceCodeResult, String> {
    let manifest = read_provider_manifest(provider_id)?;
    let config = require_device_code_config(&manifest.auth, provider_id)?;

    log_info!("[Frp Auth] 启动 Device Code 流程: provider={}", provider_id);

    // 1. 请求设备码
    let client = crate::http::get_client();
    let scope_str = config.scopes.join(" ");
    let resp = client
        .post(&config.device_code_url)
        .form(&[
            ("client_id", config.client_id.as_str()),
            ("scope", scope_str.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("请求设备码失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("请求设备码失败: HTTP {}", resp.status()));
    }

    let body: DeviceCodeResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析设备码响应失败: {}", e))?;

    // 2. 存入内存会话
    let interval = body.interval.unwrap_or(config.poll_interval);
    let expires_at = now_secs() + body.expires_in;
    {
        let mut sessions = DEVICE_CODE_SESSIONS.lock().map_err(|e| format!("会话锁 poisoned: {}", e))?;
        sessions.insert(
            provider_id.to_string(),
            DeviceCodeSession {
                device_code: body.device_code.clone(),
                expires_at,
                interval,
                token_url: config.token_url.clone(),
                client_id: config.client_id.clone(),
            },
        );
    }

    log_info!(
        "[Frp Auth] Device Code 已获取: provider={}, user_code={}, expires_in={}s",
        provider_id, body.user_code, body.expires_in
    );

    Ok(DeviceCodeResult {
        user_code: body.user_code,
        verification_uri: body.verification_uri,
        expires_in: body.expires_in,
        interval,
    })
}

/// 轮询 Device Code token
///
/// 前端按 interval 调用此函数，后端用 device_code 向 tokenUrl 发起请求：
/// - pending → 继续轮询
/// - success → 存储 token，返回成功
/// - expired → 设备码过期，需重新发起
/// - declined → 用户拒绝授权
/// - slow_down → 轮询过快，前端应增大间隔
pub async fn poll_device_code(_state: &AppState, provider_id: &str) -> Result<DeviceCodePollResult, String> {
    // 1. 读取会话
    let session = {
        let sessions = DEVICE_CODE_SESSIONS.lock().map_err(|e| format!("会话锁 poisoned: {}", e))?;
        sessions.get(provider_id).cloned().ok_or_else(|| {
            format!("未找到 {} 的 Device Code 会话，请先调用 start_device_code", provider_id)
        })?
    };

    // 会话已过期
    if now_secs() > session.expires_at {
        remove_device_code_session(provider_id);
        return Ok(DeviceCodePollResult {
            status: "expired".to_string(),
            expires_at: None,
            scopes: None,
        });
    }

    // 2. 轮询 token
    let client = crate::http::get_client();
    let resp = client
        .post(&session.token_url)
        .form(&[
            (
                "grant_type",
                "urn:ietf:params:oauth:grant-type:device_code",
            ),
            ("device_code", session.device_code.as_str()),
            ("client_id", session.client_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("轮询 token 失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("轮询 token 失败: HTTP {}", resp.status()));
    }

    let body: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析轮询响应失败: {}", e))?;

    // 3. 处理结果
    if let Some(err) = &body.error {
        let status = match err.as_str() {
            "authorization_pending" => "pending",
            "expired_token" => {
                remove_device_code_session(provider_id);
                "expired"
            }
            "access_denied" => {
                remove_device_code_session(provider_id);
                "declined"
            }
            "slow_down" => "slow_down",
            other => {
                log_error!("[Frp Auth] 未知 device code 错误: {}", other);
                remove_device_code_session(provider_id);
                return Err(format!("设备码授权失败: {}", other));
            }
        };
        return Ok(DeviceCodePollResult {
            status: status.to_string(),
            expires_at: None,
            scopes: None,
        });
    }

    // 4. 成功 -> 存储 token
    let access_token = body.access_token.ok_or("token 响应缺少 access_token")?;
    let expires_at = body.expires_in.map(|secs| now_secs() + secs);
    let scopes = body.scope.as_ref().map(|s| parse_scopes(s));

    // 读取 manifest 中的 scopes 作为回退
    let manifest = read_provider_manifest(provider_id)?;
    let config = require_device_code_config(&manifest.auth, provider_id)?;
    let scopes_for_store = scopes.as_ref().or(Some(&config.scopes));

    store_token_info(
        provider_id,
        &access_token,
        body.refresh_token.as_deref(),
        body.expires_in,
        scopes_for_store,
    )?;
    remove_device_code_session(provider_id);

    log_info!(
        "[Frp Auth] Device Code 认证成功: provider={}, expires_at={:?}",
        provider_id, expires_at
    );

    Ok(DeviceCodePollResult {
        status: "success".to_string(),
        expires_at,
        scopes,
    })
}

/// 刷新 token
///
/// access_token 过期前 5 分钟用 refresh_token 刷新。
/// 也可由用户手动触发（AuthCenter 刷新按钮）。
pub async fn refresh_token(_state: &AppState, provider_id: &str) -> Result<(), String> {
    let manifest = read_provider_manifest(provider_id)?;

    // 获取 tokenUrl + clientId（oauth2 或 device_code）
    let (token_url, client_id) = if let Some(ref oauth2) = manifest.auth.oauth2 {
        (oauth2.token_url.clone(), oauth2.client_id.clone())
    } else if let Some(ref dc) = manifest.auth.device_code {
        (dc.token_url.clone(), dc.client_id.clone())
    } else {
        return Err(format!("厂商 {} 不支持 token 刷新", provider_id));
    };

    let refresh_token = load_secret(provider_id, KEY_REFRESH_TOKEN)?
        .ok_or_else(|| format!("厂商 {} 无 refresh_token，请重新认证", provider_id))?;

    log_info!("[Frp Auth] 刷新 token: provider={}", provider_id);

    let client = crate::http::get_client();
    let resp = client
        .post(&token_url)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("client_id", client_id.as_str()),
        ])
        .send()
        .await
        .map_err(|e| format!("刷新 token 请求失败: {}", e))?;

    if !resp.status().is_success() {
        // 刷新失败（refresh_token 过期等），清除 token 引导重新认证
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        log_error!("[Frp Auth] 刷新 token 失败: HTTP {} {}", status, body);
        return Err(format!("刷新 token 失败: HTTP {}", status));
    }

    let token_resp: TokenResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析刷新响应失败: {}", e))?;

    let access_token = token_resp.access_token.ok_or("刷新响应缺少 access_token")?;

    store_token_info(
        provider_id,
        &access_token,
        token_resp.refresh_token.as_deref(),
        token_resp.expires_in,
        None, // 刷新不改变 scopes
    )?;

    log_info!("[Frp Auth] token 刷新成功: provider={}", provider_id);
    Ok(())
}

/// 撤销认证（删除所有存储的 token）
pub async fn revoke_auth(provider_id: &str) -> Result<(), String> {
    log_info!("[Frp Auth] 撤销认证: provider={}", provider_id);

    // 清除 keyring 中的所有密钥
    delete_secret(provider_id, KEY_ACCESS_TOKEN)?;
    delete_secret(provider_id, KEY_REFRESH_TOKEN)?;
    delete_secret(provider_id, KEY_EXPIRES_AT)?;
    delete_secret(provider_id, KEY_SCOPES)?;

    // 清除 Device Code 会话
    remove_device_code_session(provider_id);

    Ok(())
}

/// 读取 access_token（供 api_schema 模块调用厂商 API 时使用）
///
/// 仅读取已存储的 access_token，不检查过期、不自动刷新。
/// 调用方（api_schema::fetch_vendor_config）应先调用 refresh_token 确保有效。
pub async fn load_token(provider_id: &str) -> Result<String, String> {
    load_secret(provider_id, KEY_ACCESS_TOKEN)?
        .ok_or_else(|| format!("厂商 {} 未认证，请先完成认证", provider_id))
}

/// 保存 API Key（auth_type=api_key 时由前端调用）
///
/// API Key 直接作为 access_token 存储，无过期时间、无 refresh_token。
pub async fn save_api_key(provider_id: &str, api_key: &str) -> Result<(), String> {
    let manifest = read_provider_manifest(provider_id)?;
    if manifest.auth.auth_type != "api_key" {
        return Err(format!("厂商 {} 不使用 API Key 认证", provider_id));
    }
    if api_key.trim().is_empty() {
        return Err("API Key 不能为空".to_string());
    }

    store_secret(provider_id, KEY_ACCESS_TOKEN, api_key.trim())?;
    log_info!("[Frp Auth] API Key 已保存: provider={}", provider_id);
    Ok(())
}

// ============================================================
// 内部辅助函数
// ============================================================

/// 获取 OAuth2Config（不存在则报错）
fn require_oauth2_config<'a>(auth: &'a AuthConfig, provider_id: &str) -> Result<&'a OAuth2Config, String> {
    auth.oauth2
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 auth.oauth2 配置", provider_id))
}

/// 获取 DeviceCodeConfig（不存在则报错）
fn require_device_code_config<'a>(
    auth: &'a AuthConfig,
    provider_id: &str,
) -> Result<&'a DeviceCodeConfig, String> {
    auth.device_code
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 auth.device_code 配置", provider_id))
}

/// 获取 ApiKeyConfig（不存在则报错）
#[allow(dead_code)]
fn require_api_key_config<'a>(auth: &'a AuthConfig, provider_id: &str) -> Result<&'a ApiKeyConfig, String> {
    auth.api_key
        .as_ref()
        .ok_or_else(|| format!("厂商 {} 的 manifest 缺少 auth.api_key 配置", provider_id))
}

/// 当前 Unix 时间戳（秒）
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// 生成 OAuth2 state 参数（CSRF 防护）
///
/// 基于系统时间纳秒 + 进程 ID 生成，非密码学安全但足以防止本地回调伪造。
fn generate_state() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    format!("{:x}{:x}", nanos, pid)
}

/// 解析 scope 字符串（空格分隔）为 Vec
fn parse_scopes(scope_str: &str) -> Vec<String> {
    scope_str
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// 移除 Device Code 会话
fn remove_device_code_session(provider_id: &str) {
    if let Ok(mut sessions) = DEVICE_CODE_SESSIONS.lock() {
        sessions.remove(provider_id);
    }
}

/// 构建 OAuth2 授权 URL
fn build_authorize_url(
    authorize_url: &str,
    client_id: &str,
    redirect_uri: &str,
    scopes: &[String],
    state: &str,
) -> String {
    let scope_str = scopes.join(" ");
    let params: Vec<(String, String)> = vec![
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("response_type", "code"),
        ("scope", &scope_str),
        ("state", state),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();

    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let sep = if authorize_url.contains('?') { '&' } else { '?' };
    format!("{}{}{}", authorize_url, sep, query)
}

/// 等待 OAuth2 回调，解析 code + state
async fn wait_for_callback(
    listener: &tokio::net::TcpListener,
    expected_state: &str,
) -> Result<OAuth2Callback, String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let (mut socket, _) = listener
        .accept()
        .await
        .map_err(|e| format!("接受回调连接失败: {}", e))?;

    // 读取 HTTP 请求（最多 4KB）
    let mut buf = vec![0u8; 4096];
    let n = socket
        .read(&mut buf)
        .await
        .map_err(|e| format!("读取回调请求失败: {}", e))?;
    let request = String::from_utf8_lossy(&buf[..n]);

    // 解析请求行：GET /?code=xxx&state=yyy HTTP/1.1
    let request_line = request.lines().next().unwrap_or("");
    let path = request_line.split_whitespace().nth(1).unwrap_or("");

    // 回复浏览器（无论成功失败都关闭连接）
    let (html, ok) = parse_callback_path(path, expected_state);
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n{}",
        html
    );
    let _ = socket.write_all(response.as_bytes()).await;
    let _ = socket.flush().await;

    if ok {
        parse_callback_path_for_code(path, expected_state)
    } else {
        Err("OAuth2 回调无效：state 不匹配或缺少 code 参数".to_string())
    }
}

/// 解析回调路径，返回 (HTML 响应, 是否成功)
fn parse_callback_path(path: &str, expected_state: &str) -> (String, bool) {
    let query = path.split('?').nth(1).unwrap_or("");
    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|kv| {
            let mut parts = kv.splitn(2, '=');
            let k = parts.next()?;
            let v = parts.next()?;
            Some((k, v))
        })
        .collect();

    let code = params.get("code").copied().unwrap_or("");
    let state = params.get("state").copied().unwrap_or("");

    if !code.is_empty() && state == expected_state {
        (
            "<html><body><h2>认证成功</h2><p>请返回 MoLaunch 应用</p></body></html>".to_string(),
            true,
        )
    } else {
        (
            "<html><body><h2>认证失败</h2><p>state 不匹配或缺少 code 参数</p></body></html>".to_string(),
            false,
        )
    }
}

/// 从回调路径解析 code（已校验 state 后调用）
fn parse_callback_path_for_code(path: &str, expected_state: &str) -> Result<OAuth2Callback, String> {
    let query = path.split('?').nth(1).unwrap_or("");
    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|kv| {
            let mut parts = kv.splitn(2, '=');
            let k = parts.next()?;
            let v = parts.next()?;
            Some((k, v))
        })
        .collect();

    let code = params
        .get("code")
        .copied()
        .ok_or("回调缺少 code 参数")?;
    let state = params.get("state").copied().unwrap_or("");

    if state != expected_state {
        return Err("OAuth2 state 不匹配（可能的 CSRF 攻击）".to_string());
    }

    Ok(OAuth2Callback {
        code: urlencoding::decode(code)
            .map(|c| c.to_string())
            .unwrap_or_else(|_| code.to_string()),
    })
}

/// 用 authorization code 换取 token
async fn exchange_code_for_token(
    token_url: &str,
    client_id: &str,
    redirect_uri: &str,
    code: &str,
) -> Result<TokenResponse, String> {
    let client = crate::http::get_client();
    let resp = client
        .post(token_url)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client_id),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await
        .map_err(|e| format!("token 交换请求失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        log_error!("[Frp Auth] token 交换失败: HTTP {} {}", status, body);
        return Err(format!("token 交换失败: HTTP {}", status));
    }

    resp.json()
        .await
        .map_err(|e| format!("解析 token 响应失败: {}", e))
}

// ============================================================
// 内部类型
// ============================================================

/// OAuth2 回调解析结果
struct OAuth2Callback {
    code: String,
}

/// OAuth2 / Device Code token 端点响应
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
    /// 错误字段（Device Code 轮询时使用）
    error: Option<String>,
    #[allow(dead_code)]
    error_description: Option<String>,
}

/// Device Code 端点响应
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    /// 轮询间隔（秒），部分服务端可能不返回
    interval: Option<u64>,
}
