//! 公告拉取（GET /v1/announcements）
//!
//! 公开接口（服务端已豁免 JWT），无需设备凭证。放在 Rust 侧发起是因为
//! WebView 的 origin 不在服务端 CORS 白名单内，前端直接 fetch 会被拦。
//! 任何失败都降级为空列表：公告拉不到不该影响启动。

use serde::{Deserialize, Serialize};

use crate::handler;
use crate::log_debug;
use crate::utils::dispatcher::Dispatcher;

/// 公告拉取超时
const ANNOUNCEMENT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// 公开公告（字段与服务端 PublicAnnouncement 一一对应，保持 snake_case）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicAnnouncement {
    pub id: String,
    pub title: String,
    pub content: String,
    pub level: String,
    pub priority: i32,
    pub starts_at: i64,
    pub ends_at: i64,
    pub link_url: String,
    pub dismissible: bool,
}

/// 公告拉取响应
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementListResponse {
    pub success: bool,
    pub announcements: Vec<PublicAnnouncement>,
    pub error: String,
}

impl AnnouncementListResponse {
    /// 失败响应（拉不到公告不是错误，前端静默跳过即可）
    fn failed(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            announcements: Vec::new(),
            error: msg.into(),
        }
    }
}

/// 注册公告 actions
pub fn register(d: &mut Dispatcher) {
    d.register(
        "announcement_list",
        handler!(state, _app, _params, {
            let resp = fetch(&state).await;
            serde_json::to_value(resp).map_err(|e| e.to_string())
        }),
    );
}

/// 拉取当前生效公告
///
/// 通道与版本由本端推导，与更新检查同一套口径：开发者模式可覆盖分支，
/// 否则按当前版本后缀（alpha / beta / stable）判定。
async fn fetch(state: &crate::state::AppState) -> AnnouncementListResponse {
    let base = super::read_api_server_url(state).await;
    if base.is_empty() {
        return AnnouncementListResponse::failed("未配置 API 服务器地址");
    }

    let version = env!("CARGO_PKG_VERSION");
    let channel = {
        let branch = crate::commands::system::developer::get_update_branch();
        if branch == "auto" {
            crate::utils::client_type::channel_name(version).to_string()
        } else {
            branch
        }
    };

    let url = format!("{}/v1/announcements", base.trim_end_matches('/'));
    let resp = match crate::http::get_client()
        .get(&url)
        .query(&[("channel", channel.as_str()), ("version", version)])
        .timeout(ANNOUNCEMENT_TIMEOUT)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            log_debug!("[Announcement] 拉取失败（忽略）: {}", e);
            return AnnouncementListResponse::failed(e.to_string());
        }
    };

    if !resp.status().is_success() {
        log_debug!("[Announcement] 服务端返回 HTTP {}", resp.status());
        return AnnouncementListResponse::failed(format!("HTTP {}", resp.status()));
    }

    let json: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(e) => {
            log_debug!("[Announcement] 响应解析失败（忽略）: {}", e);
            return AnnouncementListResponse::failed(e.to_string());
        }
    };
    if json.get("code").and_then(|c| c.as_i64()) != Some(1) {
        return AnnouncementListResponse::failed("服务端返回非成功状态");
    }

    let announcements = json
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    AnnouncementListResponse {
        success: true,
        announcements,
        error: String::new(),
    }
}
