//! 公共 frps 服务器 action 注册（对接 apiServer `/v1/frp/*`）。
//!
//! 仅含列出服务器。列表接口直接返回完整连接信息（公共 token + 地址端口），
//! 客户端无需再走分配/释放/续期链路。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::minecraft::online::client::OnlineClient;
use crate::minecraft::online::frp::PublicFrpServer;
use crate::minecraft::online::storage::DeviceCredentials;
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

/// 加载设备凭证（需已注册），access token 过期时自动 refresh
async fn load_creds(state: &AppState) -> Result<DeviceCredentials, String> {
    crate::commands::online::manager::load_creds_with_auto_refresh(state).await
}

/// 创建 OnlineClient（读取配置中的 api_server_url）
async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

/// 注册公共 frps 服务器相关 action
pub fn register(d: &mut Dispatcher) {
    // 列出可用的公共 frps 服务器（GET /v1/frp/servers）
    //
    // 明文响应，自动携带 JWT。前端据此展示服务器列表供用户选择。
    d.register(
        "list_public_servers",
        handler!(state, _app, _params, {
            let creds = load_creds(&state).await?;
            let client = make_client(&state).await;
            log_debug!("[Frp] list_public_servers");
            let result = client.frp_list_public_servers(&creds).await.map_err(|e| {
                log_error!("[Frp] list_public_servers 失败: {}", e);
                e.to_string()
            })?;
            if result.code != 1 {
                return Err(format!(
                    "列出公共服务器失败 [code={}]: {}",
                    result.code, result.msg
                ));
            }
            let servers: Vec<PublicFrpServer> = result.data.unwrap_or_default();
            serde_json::to_value(servers).map_err(|e| e.to_string())
        }),
    );
}
