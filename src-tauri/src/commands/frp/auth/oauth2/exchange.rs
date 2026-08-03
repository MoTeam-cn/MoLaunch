//! OAuth2 授权码交换：授权 URL 构建 + 回调 HTTP 服务处理

use std::collections::HashMap;

/// OAuth2 回调解析结果
pub(super) struct OAuth2Callback {
    pub(super) code: String,
}

/// 构建 OAuth2 授权 URL（用户交互层，标准 OAuth2 流程）
pub(super) fn build_authorize_url(
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

    let sep = if authorize_url.contains('?') {
        '&'
    } else {
        '?'
    };
    format!("{}{}{}", authorize_url, sep, query)
}

/// 等待 OAuth2 回调，解析 code + state
pub(super) async fn wait_for_callback(
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
            let (k, v) = kv.split_once('=')?;
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
            "<html><body><h2>认证失败</h2><p>state 不匹配或缺少 code 参数</p></body></html>"
                .to_string(),
            false,
        )
    }
}

/// 从回调路径解析 code（已校验 state 后调用）
fn parse_callback_path_for_code(
    path: &str,
    expected_state: &str,
) -> Result<OAuth2Callback, String> {
    let query = path.split('?').nth(1).unwrap_or("");
    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            Some((k, v))
        })
        .collect();

    let code = params.get("code").copied().ok_or("回调缺少 code 参数")?;
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