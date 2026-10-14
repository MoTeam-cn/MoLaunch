//! 《用户协议》远程配置拉取（GET https://molaunch.moiu.cn/agreement.json）
//!
//! 与 announcement.rs 同一模式：放在 Rust 侧发起，绕开 WebView 的 CORS 限制，
//! 因此官网侧无需为该文件配置 Access-Control-Allow-Origin。
//! 任何失败都返回 success=false，由前端回退本地默认配置——协议拉不到不该阻塞启动。

use serde::Serialize;

use crate::handler;
use crate::log_debug;
use crate::utils::dispatcher::Dispatcher;

/// 官网协议清单地址（与官网仓库 src/logic/agreement.ts 的 AGREEMENT_MANIFEST_URL 一致）
const AGREEMENT_MANIFEST_URL: &str = "https://molaunch.moiu.cn/agreement.json";
/// 协议拉取超时
const AGREEMENT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// 协议拉取响应
///
/// success=false 时 config 为 None，前端回退本地默认。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAgreementFetchResponse {
    pub success: bool,
    /// 原样透传官网清单（字段与前端 UserAgreementRemoteConfig 对齐）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    pub error: String,
}

impl UserAgreementFetchResponse {
    /// 失败响应（协议拉不到不是错误，前端回退本地默认即可）
    fn failed(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            config: None,
            error: msg.into(),
        }
    }
}

/// 注册《用户协议》远程配置拉取 action
pub fn register(d: &mut Dispatcher) {
    d.register(
        "user_agreement_fetch",
        handler!(state, _app, _params, {
            let resp = fetch(&state).await;
            serde_json::to_value(resp).map_err(|e| e.to_string())
        }),
    );
}

/// 拉取官网协议清单
///
/// 带 `?ts=` 时间戳查询参数：官网 /agreement.json 是固定路径、不带内容哈希，
/// CDN 可能缓存旧版本；查询串参与缓存键，可确保每次都拿到最新协议版本。
async fn fetch(_state: &crate::state::AppState) -> UserAgreementFetchResponse {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let url = format!("{AGREEMENT_MANIFEST_URL}?ts={ts}");

    let resp = match crate::http::get_client()
        .get(&url)
        .timeout(AGREEMENT_TIMEOUT)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log_debug!("[UserAgreement] 拉取失败（忽略）: {}", e);
            return UserAgreementFetchResponse::failed(e.to_string());
        }
    };

    if !resp.status().is_success() {
        log_debug!("[UserAgreement] 官网返回 HTTP {}", resp.status());
        return UserAgreementFetchResponse::failed(format!("HTTP {}", resp.status()));
    }

    match resp.json::<serde_json::Value>().await {
        Ok(v) => UserAgreementFetchResponse {
            success: true,
            config: Some(v),
            error: String::new(),
        },
        Err(e) => {
            log_debug!("[UserAgreement] 响应解析失败（忽略）: {}", e);
            UserAgreementFetchResponse::failed(e.to_string())
        }
    }
}
