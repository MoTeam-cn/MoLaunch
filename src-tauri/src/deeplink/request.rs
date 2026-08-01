//! 深度链接请求的解析与结构定义
//!
//! 职责：把原始 URL 字符串（如 `molaunch://run?version=1.20.1`）解析为
//! 结构化 [`DeeplinkRequest`]，供路由分发与 handler 使用。

use std::collections::HashMap;

use url::Url;

use crate::log_info;

/// 解析后的深度链接请求
///
/// 示例：`molaunch://run?version=1.20.1&auto=true`
/// - raw: `molaunch://run?version=1.20.1&auto=true`（原始串，调试用）
/// - scheme: `molaunch`
/// - host: `run`（路由键，注册时匹配）
/// - path: `""`（host 之后的路径段，如 `molaunch://run/abc` → `/abc`）
/// - query: `{ "version": "1.20.1", "auto": "true" }`（url-decode 后）
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeeplinkRequest {
    /// 原始 URL 字符串
    pub raw: String,
    /// 协议名（不含 `://`，固定为 `molaunch`）
    pub scheme: String,
    /// 路由键（`molaunch://run` 的 `run`），空表示无 host
    pub host: String,
    /// host 之后的路径段（以 `/` 开头，无则为空串）
    pub path: String,
    /// 查询参数（已 URL 解码）
    pub query: HashMap<String, String>,
}

impl DeeplinkRequest {
    /// 读取查询参数并解析为指定类型，缺失/非法返回默认值
    pub fn get<T: std::str::FromStr>(&self, key: &str) -> Option<T> {
        self.query.get(key).and_then(|v| v.parse().ok())
    }

    /// 读取查询参数（字符串形式）
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }
}

/// 解析 `molaunch://` URL 为结构化请求
///
/// 解析失败（非 molaunch 协议 / URL 非法）返回 None。
pub fn parse(raw: &str) -> Option<DeeplinkRequest> {
    let url = Url::parse(raw).ok()?;
    let scheme = url.scheme().to_string();
    if scheme != "molaunch" {
        log_info!("[Deeplink] 忽略非 molaunch 协议: {}", raw);
        return None;
    }
    let host = url.host_str().unwrap_or("").to_string();
    let path = url.path().to_string();
    let mut query = HashMap::new();
    for (k, v) in url.query_pairs() {
        query.insert(k.into_owned(), v.into_owned());
    }
    Some(DeeplinkRequest {
        raw: raw.to_string(),
        scheme,
        host,
        path,
        query,
    })
}
