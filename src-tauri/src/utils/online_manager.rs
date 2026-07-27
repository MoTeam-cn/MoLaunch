//! 联机模块统一分发逻辑（online_manager 的工具实现）
//!
//! 使用 `utils::dispatcher::Dispatcher` 注册式分发。
//! 阶段一注册 6 个认证相关 action，阶段二/三补充房间/信令/WebRTC/虚拟网卡/端口探测。
//!
//! 阶段三子任务 5 新增 3 个 TUN 桥接 action（由 `tun_manager` 注册）：
//! - `tun_start`：创建 TUN 接口 + 启动读写循环 + emit `online://tun-packet-out` 事件
//! - `tun_forward_to`：前端 DataChannel 收到消息后调用，解码协议帧并写入 TUN
//! - `tun_stop`：停止桥接，销毁 TUN 接口

use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::AppHandle;

use crate::handler;
use crate::log_info;
use crate::log_warn;
use crate::log_error;
use crate::log_debug;
use crate::minecraft::online::auth::{
    build_login_request, build_register_request, finalize_credentials_with_login,
    finalize_credentials_with_register, generate_device_id, OnlineKeyPair,
};
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::storage::OnlineStorage;
use crate::state::AppState;
use crate::utils::dispatcher::{ActionRequest, Dispatcher};

// ============================================================
// 参数 / 返回类型
// ============================================================

/// 设备状态返回
#[derive(Debug, Clone, Serialize)]
pub struct DeviceStatus {
    pub registered: bool,
    pub logged_in: bool,
    pub token_expired: bool,
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

// ============================================================
// 辅助函数
// ============================================================

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

// ============================================================
// Dispatcher 注册
// ============================================================

static DISPATCHER: Lazy<Dispatcher> = Lazy::new(|| {
    let mut d = Dispatcher::new();

    // === 设备认证 ===

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

    // === 信令相关 action（阶段二：房间创建/加入/退出/踢人/保活等）===
    // 由 signaling_manager 模块统一注册，避免本文件超过 500 行
    crate::utils::signaling_manager::register_signaling_actions(&mut d);

    // === TUN 桥接管理 action（阶段三子任务 5：数据分发打通）===
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
