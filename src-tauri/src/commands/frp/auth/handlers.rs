//! 认证公开 API 处理函数：状态查询 / token 刷新 / 撤销 / 注入入口

use super::super::api_spec::load_api_spec;
use super::super::provider::{
    read_provider_manifest, resolve_auth_type, resolve_device_code_config, resolve_oauth2_config,
    SYSTEM_DEFAULT_ID,
};
use super::flows::{extract_flow_error, get_extractor};
use super::{device_code, flows, storage, AuthStatus};
use crate::state::AppState;

/// 查询指定厂商的认证状态
///
/// - auth_type=none：始终 authenticated=true
/// - auth_type=oauth2/device_code：检查 access_token 是否存在且未过期
///   - 已过期且存在 refresh_token → 自动续期（成功则 authenticated=true，失败则 refreshing=true）
/// - auth_type=api_key：检查 access_token（即 API Key）是否存在
///
/// expires_at 即使已过期也会返回，前端据此区分「即将过期」/「已过期」。
pub async fn get_auth_status(state: &AppState, provider_id: &str) -> Result<AuthStatus, String> {
    // 系统默认厂商无需认证
    if provider_id == SYSTEM_DEFAULT_ID {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type: "none".to_string(),
            expires_at: None,
            scopes: None,
            refreshing: false,
        });
    }

    let manifest = read_provider_manifest(provider_id)?;
    let auth_type = resolve_auth_type(provider_id, &manifest);

    if auth_type == "none" {
        return Ok(AuthStatus {
            provider_id: provider_id.to_string(),
            authenticated: true,
            auth_type,
            expires_at: None,
            scopes: None,
            refreshing: false,
        });
    }

    // 检查 access_token 是否存在（SDK 解密读取）
    let record = storage::load_token_record(provider_id).await?;
    let authenticated = record.is_some();

    // 检查是否过期（仅 oauth2 / device_code 有过期时间）
    let expires_at = if matches!(auth_type.as_str(), "oauth2" | "device_code") {
        record.as_ref().and_then(|r| r.expires_at)
    } else {
        None
    };

    // token 存在但已过期 -> authenticated=false
    let (authenticated, refreshing) = if authenticated {
        match expires_at {
            Some(exp) if exp <= storage::now_secs() => {
                // 已过期：有 refresh_token 则尝试静默续期
                let has_refresh = record
                    .as_ref()
                    .and_then(|r| r.refresh_token.as_ref())
                    .is_some();
                if has_refresh {
                    match refresh_token(state, provider_id).await {
                        Ok(()) => {
                            crate::log_info!(
                                "[Frp Auth] get_auth_status 自动续期成功: provider={}",
                                provider_id
                            );
                            (true, false)
                        }
                        Err(e) => {
                            crate::log_warn!(
                                "[Frp Auth] get_auth_status 自动续期失败: provider={} - {}",
                                provider_id,
                                e
                            );
                            (false, true)
                        }
                    }
                } else {
                    (false, false)
                }
            }
            Some(exp) => (exp > storage::now_secs(), false),
            None => (true, false), // api_key 无过期时间
        }
    } else {
        (false, false)
    };

    // 续期成功后重新读取记录，返回新 token 的过期时间（避免前端显示旧的过期时间）
    let (expires_at, scopes) = if authenticated {
        let new_record = storage::load_token_record(provider_id).await?;
        match new_record {
            Some(r) => {
                let exp = if matches!(auth_type.as_str(), "oauth2" | "device_code") {
                    r.expires_at
                } else {
                    None
                };
                (exp, r.scopes.clone())
            }
            None => (expires_at, record.as_ref().and_then(|r| r.scopes.clone())),
        }
    } else {
        (expires_at, record.as_ref().and_then(|r| r.scopes.clone()))
    };

    Ok(AuthStatus {
        provider_id: provider_id.to_string(),
        authenticated,
        auth_type,
        expires_at,
        scopes,
        refreshing,
    })
}

/// 刷新 token
///
/// access_token 过期前 5 分钟用 refresh_token 刷新。也可由用户手动触发。
///
/// 流程：加载 endpoints.json → 取 authFlows.oauth2.refresh（不存在时回退到 oauth2.token）
/// → 走 flows 引擎发送请求 → 解析响应并存储新 token。
pub async fn refresh_token(_state: &AppState, provider_id: &str) -> Result<(), String> {
    let manifest = read_provider_manifest(provider_id)?;
    let endpoints_file = manifest
        .api
        .as_ref()
        .map(|a| a.endpoints_file.as_str())
        .unwrap_or("api/endpoints.json");
    let spec = load_api_spec(provider_id, endpoints_file)?;
    let oauth2_flow = spec
        .auth_flows
        .as_ref()
        .and_then(|f| f.oauth2.as_ref())
        .ok_or_else(|| {
            format!(
                "厂商 {} endpoints.json 缺少 authFlows.oauth2 配置",
                provider_id
            )
        })?;
    // 优先使用 refresh，缺失时回退到 token（部分厂商用同一端点刷新）
    let refresh_flow = oauth2_flow.refresh.as_ref().unwrap_or(&oauth2_flow.token);

    let refresh_token = storage::load_token_record(provider_id)
        .await?
        .and_then(|r| r.refresh_token)
        .ok_or_else(|| format!("厂商 {} 无 refresh_token，请重新认证", provider_id))?;

    // 取 clientId/clientSecret（从 auth.json 按 authType 读取，oauth2 或 device_code 必居其一）
    let auth_type = resolve_auth_type(provider_id, &manifest);
    let (client_id, client_secret) = match auth_type.as_str() {
        "oauth2" => {
            let cfg = resolve_oauth2_config(provider_id, &manifest)?;
            // PKCE 公开客户端：refresh 同样不携带 client_secret
            let secret = if cfg.pkce { None } else { cfg.client_secret };
            (cfg.client_id, secret)
        }
        "device_code" => {
            let cfg = resolve_device_code_config(provider_id, &manifest)?;
            (cfg.client_id, cfg.client_secret)
        }
        _ => return Err(format!("厂商 {} 不支持 token 刷新", provider_id)),
    };

    crate::log_info!("[Frp Auth] 刷新 token: provider={}", provider_id);

    let ctx = flows::FlowContext {
        base_url: Some(spec.base_url.clone()),
        client_id,
        client_secret,
        refresh_token: Some(refresh_token),
        ..Default::default()
    };
    let resp = flows::send_flow_request(refresh_flow, &ctx).await?;

    if !resp.is_success() {
        let err = extract_flow_error(&resp, refresh_flow);
        crate::log_error!("[Frp Auth] 刷新 token 失败: HTTP {} - {}", resp.status, err);
        return Err(format!("刷新 token 失败: {}", err));
    }

    let access_token = resp
        .extract_field(get_extractor(refresh_flow, "accessToken"))
        .ok_or("刷新响应缺少 accessToken")?;
    let new_refresh_token = resp.extract_field(get_extractor(refresh_flow, "refreshToken"));
    let expires_in = resp
        .extract_field(get_extractor(refresh_flow, "expiresIn"))
        .and_then(|s| s.parse::<u64>().ok());

    storage::store_token_info(
        provider_id,
        &access_token,
        new_refresh_token.as_deref(),
        expires_in,
        None, // 刷新不改变 scopes
    )
    .await?;

    crate::log_info!("[Frp Auth] token 刷新成功: provider={}", provider_id);
    Ok(())
}

/// 撤销认证（删除所有存储的 token）
pub async fn revoke_auth(provider_id: &str) -> Result<(), String> {
    crate::log_info!("[Frp Auth] 撤销认证: provider={}", provider_id);

    // 删除加密 token 文件
    storage::delete_provider_auth(provider_id).await?;

    // 清除 Device Code 会话
    device_code::remove_device_code_session(provider_id);

    Ok(())
}

/// 读取 access_token（供 api_spec 模块调用厂商 API 时使用）
///
/// 仅读取已存储的 access_token，不检查过期、不自动刷新。
/// 调用方（api_spec::fetch_tunnels）应先调用 [`ensure_valid_token`] 确保有效。
pub async fn load_token(provider_id: &str) -> Result<String, String> {
    storage::load_token_record(provider_id)
        .await?
        .map(|r| r.access_token)
        .ok_or_else(|| format!("厂商 {} 未认证，请先完成认证", provider_id))
}

/// 确保厂商 token 有效（过期时自动刷新）
///
/// 供调用厂商 API 前统一使用：
/// 1. 无 token 记录 → 返回未认证错误（调用方提示用户先认证）
/// 2. token 未过期 → 直接返回
/// 3. token 已过期且存在 refresh_token → 自动调用 [`refresh_token`] 刷新后返回新 token
/// 4. token 已过期但无 refresh_token（api_key / 厂商未下发）→ 返回过期错误
///
/// 由 `get_auth_status` 与 `fetch_tunnels` 共用，保证「认证中心」与
/// 「拉取隧道列表」两条链路都能自动续期。
pub async fn ensure_valid_token(state: &AppState, provider_id: &str) -> Result<String, String> {
    let record = storage::load_token_record(provider_id).await?;
    let Some(record) = record else {
        return Err(format!("厂商 {} 未认证，请先完成认证", provider_id));
    };

    // token 未过期 → 直接返回
    let valid = match record.expires_at {
        Some(exp) => exp > storage::now_secs(),
        None => true, // api_key / 无过期时间字段
    };
    if valid {
        return Ok(record.access_token);
    }

    // 已过期：有 refresh_token 则自动续期
    if record.refresh_token.is_some() {
        crate::log_info!(
            "[Frp Auth] token 已过期，自动续期: provider={}",
            provider_id
        );
        refresh_token(state, provider_id).await?;
        return load_token(provider_id).await;
    }

    // 已过期且无 refresh_token → 无法续期
    Err(format!(
        "厂商 {} token 已过期且无 refresh_token，请重新认证",
        provider_id
    ))
}
