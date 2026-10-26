//! SLS WebTracking 上报通道（无需签名，只能写不能读）

use std::time::Duration;

use super::config::{is_configured, track_url, SEND_TIMEOUT_SECS};

/// SLS WebTracking 上报器
pub struct SlsSink {
    url: String,
    client: reqwest::Client,
}

impl SlsSink {
    /// 构造上报器；未配置 Project/Logstore 时返回 None（遥测降级为只入队）
    pub fn new() -> Option<Self> {
        if !is_configured() {
            return None;
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(SEND_TIMEOUT_SECS))
            .build()
            .ok()?;
        Some(Self {
            url: track_url(),
            client,
        })
    }

    /// 上报一条事件
    pub async fn send(&self, payload: &serde_json::Value) -> Result<(), String> {
        let response = self
            .client
            .post(&self.url)
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("请求失败: {}", e))?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP {}", response.status()))
        }
    }
}
