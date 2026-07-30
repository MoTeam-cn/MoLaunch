//! 联机模块统一分发逻辑（online_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发，覆盖认证、房间、信令、WebRTC、
//! 虚拟网卡、端口探测等 action；TUN 桥接 action 由 `tun_manager` 注册。

use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::AppHandle;

use crate::handler;
use crate::log_info;
use crate::log_warn;
use crate::log_error;
use crate::log_debug;
use crate::minecraft::online::auth::{
    build_login_request, build_refresh_request, build_register_request,
    finalize_credentials_with_login, finalize_credentials_with_refresh,
    finalize_credentials_with_register, generate_device_id, OnlineKeyPair,
};
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::storage::{DeviceCredentials, OnlineStorage};
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};


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


/// 从 AppState 配置中读取 api_server_url
async fn read_api_server_url(state: &AppState) -> String {
    let config = state.config.lock().await;
    config.online.api_server_url.clone()
}

/// 创建 OnlineStorage 实例（每次按需创建，开销极低：仅 Arc 克隆）
fn make_storage(state: &AppState) -> OnlineStorage {
    OnlineStorage::new(state.sdk.clone())
}

/// 创建 OnlineClient 实例
async fn make_client(state: &AppState) -> OnlineClient {
    OnlineClient::new(&read_api_server_url(state).await)
}

/// 从凭证构造 DeviceStatus（消除各 action 重复构造逻辑）
fn build_device_status(creds: &DeviceCredentials, api_server_url: String) -> DeviceStatus {
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
async fn refresh_credentials(
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
pub async fn load_creds_with_auto_refresh(
    state: &AppState,
) -> Result<DeviceCredentials, String> {
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
async fn login_fresh(state: &AppState) -> Result<DeviceCredentials, String> {
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


static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();
    // 查询当前设备状态（不发起网络请求，仅读本地凭证）
    d.register("auth_status", handler!(state, _app, _params, {
        let storage = make_storage(&state);
        let creds = storage.load().await.unwrap_or(None).unwrap_or_default();
        let api_server_url = read_api_server_url(&state).await;

        let status = DeviceStatus {
            registered: creds.is_registered(),
            logged_in: !creds.device_token.is_empty(),
            token_expired: creds.is_token_expired(),
            device_pk: creds.device_pk,
            device_id: creds.device_id,
            token_expires_at: creds.token_expires_at,
            last_login_at: creds.last_login_at,
            api_server_url,
        };
        serde_json::to_value(status).map_err(|e| e.to_string())
    }));

    // 获取服务器时间（用于测试 api-server 连通性 + 校准本地时间）
    d.register("auth_get_server_time", handler!(state, _app, _params, {
        let api_url = read_api_server_url(&state).await;
        log_debug!("[Online] auth_get_server_time 开始, api_server_url={}", api_url);
        let client = make_client(&state).await;
        let time_data = client.get_server_time().await
            .map_err(|e| {
                log_error!("[Online] auth_get_server_time 失败: {}", e);
                e.to_string()
            })?;
        let info = ServerTimeInfo {
            server_time: time_data.server_time,
            rfc3339: time_data.rfc3339,
            timezone: time_data.timezone,
            offset_seconds: time_data.offset_seconds,
        };
        log_debug!(
            "[Online] auth_get_server_time 成功, server_time={}, timezone={}",
            info.server_time, info.timezone
        );
        serde_json::to_value(info).map_err(|e| e.to_string())
    }));

    // 注册新设备
    d.register("auth_register", handler!(state, _app, _params, {
        let api_url = read_api_server_url(&state).await;
        log_info!("[Online] auth_register 开始, api_server_url={}", api_url);

        let storage = make_storage(&state);
        // 若已注册，先拒绝（前端应引导走登录流程）
        if let Some(existing) = storage.load().await.unwrap_or(None) {
            if existing.is_registered() {
                log_warn!("[Online] auth_register 拒绝: 设备已注册 (device_pk={})", existing.device_pk);
                return Err("设备已注册，请使用登录接口".to_string());
            }
        }

        // 生成新密钥对 + 设备 ID
        log_debug!("[Online] 生成 Ed25519 + X25519 密钥对");
        let kp = OnlineKeyPair::generate();
        let device_id = generate_device_id();
        log_debug!("[Online] 新设备 ID: {}", device_id);

        // 获取云端 RSA 公钥（从 JWKS）
        let client = make_client(&state).await;
        log_debug!("[Online] 拉取 JWKS 并提取 RSA 公钥");
        let server_rsa_pem = client.fetch_server_rsa_pem().await
            .map_err(|e| {
                log_error!("[Online] 获取云端 RSA 公钥失败: {}", e);
                format!("获取云端 RSA 公钥失败: {}", e)
            })?;

        // 构造注册请求
        log_debug!("[Online] 构造注册请求（RSA-OAEP 加密 content）");
        let (req, mut creds) = build_register_request(&kp, &device_id, &server_rsa_pem)
            .map_err(|e| {
                log_error!("[Online] 构造注册请求失败: {}", e);
                format!("构造注册请求失败: {}", e)
            })?;

        // 发起注册
        log_debug!("[Online] 发起注册 HTTP 请求");
        let resp = client.register(&req).await
            .map_err(|e| {
                log_error!("[Online] 注册请求失败: {}", e);
                format!("注册请求失败: {}", e)
            })?;
        let data = resp.data.ok_or_else(|| {
            log_error!("[Online] 注册失败: msg={}", resp.msg);
            format!("注册失败: {}", resp.msg)
        })?;

        // 完善凭证并持久化
        creds = finalize_credentials_with_register(creds, &data);
        storage.save(&creds).await.map_err(|e| {
            log_error!("[Online] 持久化设备凭证失败: {}", e);
            e.to_string()
        })?;

        log_info!(
            "[Online] 设备注册成功: device_pk={}, device_id={}",
            creds.device_pk, creds.device_id
        );

        serde_json::to_value(DeviceStatus {
            registered: true,
            logged_in: true,
            token_expired: false,
            device_pk: creds.device_pk,
            device_id: creds.device_id,
            token_expires_at: creds.token_expires_at,
            last_login_at: creds.last_login_at,
            api_server_url: read_api_server_url(&state).await,
        }).map_err(|e| e.to_string())
    }));

    // 登录设备（刷新 JWT）
    d.register("auth_login", handler!(state, _app, _params, {
        let api_url = read_api_server_url(&state).await;
        log_info!("[Online] auth_login 开始, api_server_url={}", api_url);

        let storage = make_storage(&state);
        let creds = storage.load().await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "设备未注册，请先注册".to_string())?;
        if !creds.is_registered() {
            log_warn!("[Online] auth_login 拒绝: 设备未注册");
            return Err("设备未注册，请先注册".to_string());
        }
        log_debug!("[Online] 登录设备 device_pk={}", creds.device_pk);

        // 构造登录请求
        log_debug!("[Online] 构造登录请求（ECDH + AES-GCM 加密 content）");
        let req = build_login_request(&creds)
            .map_err(|e| {
                log_error!("[Online] 构造登录请求失败: {}", e);
                format!("构造登录请求失败: {}", e)
            })?;

        // 发起登录
        let client = make_client(&state).await;
        let resp = client.login(&req).await
            .map_err(|e| {
                log_error!("[Online] 登录请求失败: {}", e);
                format!("登录请求失败: {}", e)
            })?;
        let data = resp.data.ok_or_else(|| {
            log_error!("[Online] 登录失败: msg={}", resp.msg);
            format!("登录失败: {}", resp.msg)
        })?;

        // 更新凭证
        let updated = finalize_credentials_with_login(creds, &data);
        storage.save(&updated).await.map_err(|e| {
            log_error!("[Online] 持久化登录凭证失败: {}", e);
            e.to_string()
        })?;

        log_info!(
            "[Online] 设备登录成功: device_pk={}",
            updated.device_pk
        );

        serde_json::to_value(DeviceStatus {
            registered: true,
            logged_in: true,
            token_expired: false,
            device_pk: updated.device_pk,
            device_id: updated.device_id,
            token_expires_at: updated.token_expires_at,
            last_login_at: updated.last_login_at,
            api_server_url: read_api_server_url(&state).await,
        }).map_err(|e| e.to_string())
    }));

    // 登出设备（撤销 JWT，不清除本地密钥）
    d.register("auth_logout", handler!(state, _app, _params, {
        log_info!("[Online] auth_logout 开始");
        let storage = make_storage(&state);
        let mut creds = storage.load().await
            .map_err(|e| e.to_string())?
            .unwrap_or_default();
        if creds.device_token.is_empty() {
            log_warn!("[Online] auth_logout 拒绝: 未登录");
            return Err("未登录，无需登出".to_string());
        }

        let client = make_client(&state).await;
        client.logout(&creds.device_token).await
            .map_err(|e| {
                log_error!("[Online] 登出请求失败: {}", e);
                format!("登出请求失败: {}", e)
            })?;

        // 清除 JWT（保留密钥和 device_pk，下次登录直接用）
        creds.device_token.clear();
        creds.token_expires_at = 0;
        storage.save(&creds).await.map_err(|e| e.to_string())?;

        log_info!("[Online] 设备已登出");
        serde_json::to_value(serde_json::json!({ "success": true }))
            .map_err(|e| e.to_string())
    }));

    // 清除设备凭证（注销设备，删除本地密钥）
    d.register("auth_clear", handler!(_state, _app, _params, {
        log_info!("[Online] auth_clear 开始");
        OnlineStorage::clear().map_err(|e| {
            log_error!("[Online] 清除设备凭证失败: {}", e);
            e.to_string()
        })?;
        log_info!("[Online] 设备凭证已清除");
        serde_json::to_value(serde_json::json!({ "success": true }))
            .map_err(|e| e.to_string())
    }));

    // 用 refresh_token 续期 access token
    //
    // 前置条件：本地凭证已注册且持有未过期的 refresh_token。
    // 供前端「手动续期」按钮或 auth_init 内部流程调用。
    d.register("auth_refresh", handler!(state, _app, _params, {
        log_info!("[Online] auth_refresh 开始");
        let storage = make_storage(&state);
        let creds = storage.load().await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "设备未注册，请先注册".to_string())?;
        if !creds.is_registered() {
            return Err("设备未注册，请先注册".to_string());
        }

        let updated = refresh_credentials(&state, creds).await.map_err(|e| {
            log_warn!("[Online] auth_refresh 续期失败: {}", e);
            e
        })?;

        serde_json::to_value(build_device_status(&updated, read_api_server_url(&state).await))
            .map_err(|e| e.to_string())
    }));

    // 启动静默登录/注册（前端启动时调用一次）
    //
    // 决策树：
    // 1. 本地无凭证 → 自动注册（生成密钥对 + 调 auth_register）
    // 2. 本地有凭证 + access token 未过期 → 直接返回
    // 3. 本地有凭证 + access token 过期 + refresh_token 未过期 → 自动 refresh
    // 4. 本地有凭证 + access token 过期 + refresh_token 过期 → 自动 login
    // 5. 任何步骤失败 → 返回 DeviceStatus + error（前端 cloudConnected = false）
    d.register("auth_init", handler!(state, _app, _params, {
        log_info!("[Online] auth_init 开始");
        let api_url = read_api_server_url(&state).await;
        let storage = make_storage(&state);

        // 加载本地凭证（None = 首次启动未注册）
        let existing = storage.load().await.unwrap_or(None);

        // 检查凭证与服务端地址一致性：用户切换 api_server_url 后旧凭证对新域名无效
        let existing = match existing {
            Some(ref creds) if !creds.api_server_url.is_empty() && creds.api_server_url != api_url => {
                log_warn!(
                    "[Online] auth_init: 检测到 API 服务端地址切换 ({} → {})，旧凭证失效，重新注册",
                    creds.api_server_url,
                    api_url
                );
                None
            }
            other => other,
        };

        let creds = match existing {
            None => {
                // 首次启动：静默注册
                log_info!("[Online] auth_init: 本地无凭证，开始静默注册");
                let kp = OnlineKeyPair::generate();
                let device_id = generate_device_id();

                let client = make_client(&state).await;
                let server_rsa_pem = client.fetch_server_rsa_pem().await.map_err(|e| {
                    log_error!("[Online] auth_init: 获取云端 RSA 公钥失败: {}", e);
                    e.to_string()
                })?;

                let (req, mut new_creds) = build_register_request(&kp, &device_id, &server_rsa_pem)
                    .map_err(|e| {
                        log_error!("[Online] auth_init: 构造注册请求失败: {}", e);
                        format!("构造注册请求失败: {}", e)
                    })?;

                let resp = client.register(&req).await.map_err(|e| {
                    log_error!("[Online] auth_init: 注册请求失败: {}", e);
                    format!("注册请求失败: {}", e)
                })?;
                let data = resp.data.ok_or_else(|| {
                    log_error!("[Online] auth_init: 注册失败: msg={}", resp.msg);
                    format!("注册失败: {}", resp.msg)
                })?;

                new_creds = finalize_credentials_with_register(new_creds, &data);
                new_creds.api_server_url = api_url.clone();
                storage.save(&new_creds).await.map_err(|e| {
                    log_error!("[Online] auth_init: 持久化设备凭证失败: {}", e);
                    e.to_string()
                })?;
                log_info!(
                    "[Online] auth_init: 静默注册成功: device_pk={}",
                    new_creds.device_pk
                );
                new_creds
            }
            Some(creds) if !creds.is_registered() => {
                // 凭证存在但不完整（异常状态），拒绝静默操作，前端引导手动处理
                return Err("本地凭证不完整，请在设置页重新注册设备".to_string());
            }
            Some(creds) => {
                // 凭证完整：检查 token 状态
                if !creds.is_token_expired() {
                    // 本地 access token 未过期 → 直接返回，不主动 refresh。
                    // 若云端已撤销 token（如 RSA 密钥变更），后续业务请求会收到 code=1003，
                    // 由前端 onlineManager 的 1003 自动重试机制（refresh → login → register）兜底，
                    // 实现"仅在真正失效时才刷新"的无感刷新，避免每次启动都打 refresh 接口。
                    log_debug!("[Online] auth_init: access token 未过期，直接使用本地凭证");
                    creds
                } else if !creds.is_refresh_token_expired() {
                    // access token 过期 + refresh_token 可用 → 自动续期
                    log_info!("[Online] auth_init: access token 已过期，静默 refresh 续期");
                    match refresh_credentials(&state, creds).await {
                        Ok(updated) => updated,
                        Err(e) => {
                            log_warn!("[Online] auth_init: refresh 续期失败: {}，尝试重新登录", e);
                            // refresh 失败（如 refresh_token 被服务端撤销），降级走 login 流程
                            login_fresh(&state).await?
                        }
                    }
                } else {
                    // access token + refresh_token 均过期 → 重新登录
                    log_info!("[Online] auth_init: refresh_token 已过期，静默重新登录");
                    login_fresh(&state).await?
                }
            }
        };

        let status = build_device_status(&creds, api_url);
        serde_json::to_value(AuthInitResult { status, error: None })
            .map_err(|e| e.to_string())
    }));
    // 由 signaling_manager 模块统一注册，避免本文件超过 500 行
    crate::utils::signaling_manager::register_signaling_actions(&mut d);
    // 由 tun_manager 模块统一注册，提供 tun_start / tun_forward_to / tun_stop 三个 action
    crate::utils::tun_manager::register_tun_actions(&mut d);

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
