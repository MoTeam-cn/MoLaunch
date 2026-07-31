//! 认证 action 注册（注册/登录/启动初始化类）：auth_register、auth_login、auth_init。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::log_warn;
use crate::minecraft::online::auth::{
    build_login_request, build_register_request, finalize_credentials_with_login,
    finalize_credentials_with_register, generate_device_id, OnlineKeyPair,
};
use crate::utils::dispatcher::Dispatcher;

use super::{AuthInitResult, DeviceStatus};

/// 注册注册/登录/初始化相关 action
pub fn register(d: &mut Dispatcher) {
    register_auth_register(d);
    register_auth_login(d);
    register_auth_init(d);
}

// 注册新设备
fn register_auth_register(d: &mut Dispatcher) {
    d.register("auth_register", handler!(state, _app, _params, {
        let api_url = super::read_api_server_url(&state).await;
        log_info!("[Online] auth_register 开始, api_server_url={}", api_url);

        let storage = super::make_storage(&state);
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
        let client = super::make_client(&state).await;
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
            api_server_url: super::read_api_server_url(&state).await,
        }).map_err(|e| e.to_string())
    }));
}

// 登录设备（刷新 JWT）
fn register_auth_login(d: &mut Dispatcher) {
    d.register("auth_login", handler!(state, _app, _params, {
        let api_url = super::read_api_server_url(&state).await;
        log_info!("[Online] auth_login 开始, api_server_url={}", api_url);

        let storage = super::make_storage(&state);
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
        let client = super::make_client(&state).await;
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
            api_server_url: super::read_api_server_url(&state).await,
        }).map_err(|e| e.to_string())
    }));
}

// 启动静默登录/注册（前端启动时调用一次）
//
// 决策树：
// 1. 本地无凭证 → 自动注册（生成密钥对 + 调 auth_register）
// 2. 本地有凭证 + access token 未过期 → 直接返回
// 3. 本地有凭证 + access token 过期 + refresh_token 未过期 → 自动 refresh
// 4. 本地有凭证 + access token 过期 + refresh_token 过期 → 自动 login
// 5. 任何步骤失败 → 返回 DeviceStatus + error（前端 cloudConnected = false）
fn register_auth_init(d: &mut Dispatcher) {
    d.register("auth_init", handler!(state, _app, _params, {
        log_info!("[Online] auth_init 开始");
        let api_url = super::read_api_server_url(&state).await;
        let storage = super::make_storage(&state);

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

                let client = super::make_client(&state).await;
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
                    match super::refresh_credentials(&state, creds).await {
                        Ok(updated) => updated,
                        Err(e) => {
                            log_warn!("[Online] auth_init: refresh 续期失败: {}，尝试重新登录", e);
                            // refresh 失败（如 refresh_token 被服务端撤销），降级走 login 流程
                            super::login_fresh(&state).await?
                        }
                    }
                } else {
                    // access token + refresh_token 均过期 → 重新登录
                    log_info!("[Online] auth_init: refresh_token 已过期，静默重新登录");
                    super::login_fresh(&state).await?
                }
            }
        };

        let status = super::build_device_status(&creds, api_url);
        serde_json::to_value(AuthInitResult { status, error: None })
            .map_err(|e| e.to_string())
    }));
}
