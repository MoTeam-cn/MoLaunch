//! 时间同步接口：GET /v3/time

use crate::http::get_client;
use crate::minecraft::online::client_types::{ClientError, TimeData, TimeResponse};
use crate::minecraft::online::http_log;

use super::OnlineClient;

impl OnlineClient {
    /// 获取服务器时间（GET /v3/time）
    pub async fn get_server_time(&self) -> Result<TimeData, ClientError> {
        let url = format!("{}/v3/time", self.base_url);
        crate::log_debug!("[Online] GET {}", url);
        let resp = get_client().get(&url).send().await?;
        let status = resp.status().as_u16();
        let body = resp.text().await?;
        http_log::log_http_request("GET", "/v3/time", status, &http_log::extract_req_id(&body));
        crate::log_debug!(
            "[Online] /v3/time 响应 status={}, body_len={}",
            status,
            body.len()
        );
        if status != 200 {
            crate::log_warn!("[Online] /v3/time 非 200: status={}, body={}", status, body);
            return Err(ClientError::HttpStatus { status, body });
        }
        let parsed: TimeResponse = serde_json::from_str(&body)?;
        parsed.data.ok_or_else(|| ClientError::Business {
            code: parsed.code,
            msg: parsed.msg,
        })
    }
}
