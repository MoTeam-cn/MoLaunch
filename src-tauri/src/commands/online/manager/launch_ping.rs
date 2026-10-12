//! 启动上报（设备活跃统计）
//!
//! 每次启动上报一次，由 `auth_init` 在设备凭证就绪后触发。
//! 后台发送、不阻塞启动、失败静默（只记日志）——统计不该影响用户能否启动。

use crate::minecraft::online::storage::DeviceCredentials;
use crate::{log_debug, log_warn};

/// 启动上报端点
const LAUNCH_PING_PATH: &str = "/v1/launch/ping";

/// 启动上报超时
const LAUNCH_PING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// 后台发送一次启动上报
///
/// 任何失败只记日志，绝不影响启动流程。无凭证或本地无签名密钥时直接跳过
/// （服务端 fail-closed：缺签名一律 403）。
pub fn spawn(base_url: String, creds: DeviceCredentials) {
    if creds.device_token.is_empty() || creds.device_pk.is_empty() {
        return;
    }
    tokio::spawn(async move {
        if let Err(e) = send(&base_url, &creds).await {
            log_debug!("[LaunchPing] 启动上报失败（忽略）: {}", e);
        }
    });
}

/// 发送一次启动上报
async fn send(base_url: &str, creds: &DeviceCredentials) -> Result<(), String> {
    let (sign_query, lx) =
        crate::utils::update_sign::build_ping_sign_query(&creds.device_pk, &creds.device_id);
    if lx.is_none() {
        log_warn!("[LaunchPing] 本地无可用签名密钥，跳过启动上报");
        return Err("本地无可用签名密钥".to_string());
    }

    let url = format!(
        "{}{}{}",
        base_url.trim_end_matches('/'),
        LAUNCH_PING_PATH,
        sign_query
    );
    let body = serde_json::json!({ "device_id": creds.device_id });

    let client = crate::http::get_client();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", creds.device_token))
        .json(&body)
        .timeout(LAUNCH_PING_TIMEOUT)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    Ok(())
}
