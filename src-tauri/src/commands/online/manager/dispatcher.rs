//! 联机模块统一分发逻辑（online_manager 的命令层实现）
//! 认证 action 按类别拆分到子模块，本文件保留返回类型、辅助函数与 DISPATCHER 入口。

use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::AppHandle;

use super::{auth_actions, auth_register_login, lan_fake, signaling_manager, tun};
use crate::log_error;
use crate::log_info;
use crate::minecraft::online::auth::{
    build_login_request, build_refresh_request, finalize_credentials_with_login,
    finalize_credentials_with_refresh,
};
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::storage::{DeviceCredentials, OnlineStorage};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// 返回类型

/// 设备状态返回
///
/// 安全说明：`device_pk` 标记 `#[serde(skip)]`，不暴露给前端。
/// 前端无需自己的 device_pk（房间管理操作中用到的是其他参与者的 device_pk，
/// 来自服务器房间状态而非 DeviceStatus）。后端 `build_login_request` 等
/// 内部逻辑直接从 `OnlineStorage` 读取 device_pk，不依赖前端回传。
#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatus {
    pub registered: bool,
    pub logged_in: bool,
    pub token_expired: bool,
    #[serde(skip)]
    #[allow(dead_code)]
    pub device_pk: String,
    pub device_id: String,
    pub token_expires_at: u64,
    pub last_login_at: u64,
    pub api_server_url: String,
}

/// 服务器时间返回
#[derive(Debug, Clone, Serialize)]
pub struct ServerTimeInfo {
    pub server_time: u64,
    pub rfc3339: String,
    pub timezone: String,
    pub offset_seconds: i32,
}

/// 启动静默登录/注册结果
///
/// 前端启动时调用 `auth_init`，根据 `status` 更新 `cloudConnected`：
/// - `error = None` 且 `status.logged_in = true` → 联机就绪
/// - `error = Some(msg)` → 联机不可用，前端展示 msg 并将 cloudConnected 置 false
#[derive(Debug, Clone, Serialize)]
pub struct AuthInitResult {
    pub status: DeviceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// 辅助函数（子模块共用）

/// 从 AppState 配置中读取 api_server_url
pub(crate) async fn read_api_server_url(state: &AppState) -> String {
    let config = state.config.lock().await;
    config.online.api_server_url.clone()
}

/// 创建 OnlineStorage 实例（每次按需创建，开销极低：仅 Arc 克隆）
pub(crate) fn make_storage(state: &AppState) -> OnlineStorage {
    OnlineStorage::new(state.sdk.clone())
}

/// 创建 OnlineClient 实例
pub(crate) async fn make_client(state: &AppState) -> OnlineClient {
    OnlineClient::new(&read_api_server_url(state).await)
}

/// 从凭证构造 DeviceStatus（消除各 action 重复构造逻辑）
pub(crate) fn build_device_status(
    creds: &DeviceCredentials,
    api_server_url: String,
) -> DeviceStatus {
    DeviceStatus {
        registered: creds.is_registered(),
        logged_in: !creds.device_token.is_empty(),
        token_expired: creds.is_token_expired(),
        device_pk: creds.device_pk.clone(),
        device_id: creds.device_id.clone(),
        token_expires_at: creds.token_expires_at,
        last_login_at: creds.last_login_at,
        api_server_url,
    }
}

/// 用本地 refresh_token 续期 access token
///
/// 调用前置条件：`creds.refresh_token` 非空且未过期（由调用方校验）。
/// 流程：client.refresh → finalize_credentials_with_refresh → save → 返回新凭证。
/// 供 `auth_refresh` action、`auth_init` 启动续期、信令 action 自动续期共用。
pub(crate) async fn refresh_credentials(
    state: &AppState,
    creds: DeviceCredentials,
) -> Result<DeviceCredentials, String> {
    if creds.refresh_token.is_empty() {
        return Err("本地未持有 refresh_token，无法续期".to_string());
    }
    if creds.is_refresh_token_expired() {
        return Err("refresh_token 已过期，请重新登录".to_string());
    }

    let client = make_client(state).await;
    let req = build_refresh_request(&creds).map_err(|e| {
        log_error!("[Online] 构造 refresh 请求失败: {}", e);
        format!("构造 refresh 请求失败: {}", e)
    })?;
    let resp = client.refresh(&req).await.map_err(|e| {
        log_error!("[Online] refresh 请求失败: {}", e);
        format!("refresh 请求失败: {}", e)
    })?;
    let data = resp.data.ok_or_else(|| {
        log_error!("[Online] refresh 失败: msg={}", resp.msg);
        format!("refresh 失败: {}", resp.msg)
    })?;

    let mut updated = finalize_credentials_with_refresh(creds, &data);
    // 确保旧版凭证（无 api_server_url 字段）在 refresh 后也补上当前服务端地址
    if updated.api_server_url.is_empty() {
        updated.api_server_url = read_api_server_url(state).await;
    }
    let storage = make_storage(state);
    storage.save(&updated).await.map_err(|e| {
        log_error!("[Online] 持久化 refresh 凭证失败: {}", e);
        e.to_string()
    })?;

    log_info!(
        "[Online] access token 已续期: device_pk={}, token_expires_at={}",
        updated.device_pk,
        updated.token_expires_at
    );
    Ok(updated)
}

/// 加载本地凭证，若 access token 过期则自动 refresh 续期
///
/// 供信令等业务 action 在调用 call_v1 前统一过闸，避免 401 中断。
/// - access token 未过期 → 直接返回凭证
/// - access token 过期 + refresh_token 可用 → 自动续期后返回新凭证
/// - access token 过期 + refresh_token 也过期 → 返回错误（前端引导重新登录）
pub async fn load_creds_with_auto_refresh(state: &AppState) -> Result<DeviceCredentials, String> {
    let storage = make_storage(state);
    let creds = storage
        .load()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "设备未注册，请先注册".to_string())?;
    if !creds.is_registered() {
        return Err("设备未注册，请先注册".to_string());
    }
    // 检查 api_server_url 一致性：切换服务端地址后旧凭证对新域名无效
    let api_url = read_api_server_url(state).await;
    if !creds.api_server_url.is_empty() && creds.api_server_url != api_url {
        return Err(format!(
            "API 服务端地址已切换 ({} → {})，请重新初始化联机",
            creds.api_server_url, api_url
        ));
    }
    if !creds.is_token_expired() {
        return Ok(creds);
    }

    log_info!("[Online] access token 已过期，尝试 refresh 续期");
    refresh_credentials(state, creds).await
}

/// 用本地密钥重新登录（ECDH + AES-GCM），刷新 access_token + refresh_token
///
/// 供 `auth_init` 在 refresh_token 过期或 refresh 失败时降级使用。
/// 与 `auth_login` action 共用 MoSign-v1 登录协议，但不返回 DeviceStatus，
/// 仅返回更新后的凭证，由调用方决定如何构造返回值。
pub(crate) async fn login_fresh(state: &AppState) -> Result<DeviceCredentials, String> {
    let storage = make_storage(state);
    let creds = storage
        .load()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "设备未注册，请先注册".to_string())?;
    if !creds.is_registered() {
        return Err("设备未注册，请先注册".to_string());
    }

    let req = build_login_request(&creds).map_err(|e| {
        log_error!("[Online] login_fresh: 构造登录请求失败: {}", e);
        format!("构造登录请求失败: {}", e)
    })?;

    let client = make_client(state).await;
    let resp = client.login(&req).await.map_err(|e| {
        log_error!("[Online] login_fresh: 登录请求失败: {}", e);
        format!("登录请求失败: {}", e)
    })?;
    let data = resp.data.ok_or_else(|| {
        log_error!("[Online] login_fresh: 登录失败: msg={}", resp.msg);
        format!("登录失败: {}", resp.msg)
    })?;

    let mut updated = finalize_credentials_with_login(creds, &data);
    // 确保旧版凭证（无 api_server_url 字段）在 login 后也补上当前服务端地址
    if updated.api_server_url.is_empty() {
        updated.api_server_url = read_api_server_url(state).await;
    }
    storage.save(&updated).await.map_err(|e| {
        log_error!("[Online] login_fresh: 持久化登录凭证失败: {}", e);
        e.to_string()
    })?;

    log_info!(
        "[Online] login_fresh: 重新登录成功: device_pk={}",
        updated.device_pk
    );
    Ok(updated)
}

// DISPATCHER 入口

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    auth_actions::register(&mut d);
    auth_register_login::register(&mut d);
    // 由 signaling_manager 模块统一注册
    signaling_manager::register_signaling_actions(&mut d);
    // 由 tun_manager 模块统一注册，提供 tun_start / tun_forward_to / tun_stop 三个 action
    tun::register_tun_actions(&mut d);
    // MC 局域网伪装：lan_fake_server_start / lan_fake_server_stop
    lan_fake::register_lan_fake_actions(&mut d);

    d
});

/// 分发入口
pub async fn dispatch(
    state: AppState,
    app: AppHandle,
    req: ActionRequest,
) -> Result<serde_json::Value, String> {
    DISPATCHER.dispatch(state, app, req).await
}
