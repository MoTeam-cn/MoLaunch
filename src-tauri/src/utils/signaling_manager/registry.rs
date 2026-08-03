//! 信令 action 注册入口 + 共享辅助（凭证加载 / 客户端创建）

use crate::minecraft::online::client::OnlineClient;
use crate::state::AppState;
use crate::utils::dispatcher::Dispatcher;

/// 加载设备凭证（需已注册）
///
/// 若 access token 已过期，自动调用 refresh_token 续期；refresh_token 也过期时返回错误。
/// 复用 `online_manager::load_creds_with_auto_refresh`，避免信令 action 各自处理续期逻辑。
pub(super) async fn load_creds(
    state: &AppState,
) -> Result<crate::minecraft::online::storage::DeviceCredentials, String> {
    crate::commands::online::manager::load_creds_with_auto_refresh(state).await
}

/// 创建 OnlineClient
pub(super) async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

/// 注册全部信令 action 到 dispatcher
pub fn register_signaling_actions(d: &mut Dispatcher) {
    super::room_actions::register(d);
    super::session_actions::register(d);
    super::whitelist_actions::register(d);
    super::lobby_actions::register(d);
}