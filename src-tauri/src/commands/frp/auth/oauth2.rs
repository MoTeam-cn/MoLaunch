//! OAuth2 授权流程
//!
//! 流程（参见设计文档 §6.3）：本地启动 HTTP 服务监听 redirectPort 接收回调，
//! 浏览器跳转走 `crate::minecraft::system::shell::open_url`，token 交换在后端完成。

use super::super::provider::read_provider_manifest;
use super::storage::{
    generate_state, now_secs, parse_scopes, require_oauth2_config, store_token_info,
};
use super::{OAuth2Result, TokenResponse};
use crate::log_debug;
use crate::log_error;
use crate::log_info;
use crate::state::AppState;
use std::collections::HashMap;

/// 启动 OAuth2 授权流程
pub(super) async fn start_oauth2(
    _state: &AppState,
    provider_id: &str,
) -> Result<OAuth2Result, String> {
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
    let authorize_url = build_authorize_url(
        &config.authorize_url,
        &config.client_id,
        &redirect_uri,
        &config.scopes,
        &state,
    );

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
    let access_token = token_resp
        .access_token
        .as_deref()
        .ok_or("OAuth2 响应缺少 access_token")?;
    store_token_info(
        provider_id,
        access_token,
        token_resp.refresh_token.as_deref(),
        token_resp.expires_in,
        scopes_for_store,
    )?;

    log_info!(
        "[Frp Auth] OAuth2 认证成功: provider={}, expires_at={:?}",
        provider_id,
        expires_at
    );

    Ok(OAuth2Result { expires_at, scopes })
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

/// OAuth2 回调解析结果
struct OAuth2Callback {
    code: String,
}
