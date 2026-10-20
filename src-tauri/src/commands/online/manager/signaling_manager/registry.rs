//! 信令 action 注册入口 + 共享辅助（凭证加载 / 客户端创建）

use crate::log_warn;
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
    super::super::load_creds_with_auto_refresh(state).await
}

/// 创建 OnlineClient
pub(super) async fn make_client(state: &AppState) -> OnlineClient {
    let base_url = {
        let config = state.config.lock().await;
        config.online.api_server_url.clone()
    };
    OnlineClient::new(&base_url)
}

/// 联机大厅关闭时的统一拒绝文案（与前端封存提示口径一致）
pub(super) const LOBBY_DISABLED_MESSAGE: &str =
    "联机大厅未启用：请先在「设置 → 联机」页开启「启用联机大厅（与云端交互）」后再使用";

/// 联机大厅开关是否开启（配置项 `online.lobby_enabled`）
pub(super) async fn is_lobby_enabled(state: &AppState) -> bool {
    state.config.lock().await.online.lobby_enabled
}

/// 需要云端联机能力的 action 守卫：开关关闭时直接拒绝
///
/// 前端已对大厅/房间分类封存，这里做同源兜底——旧版客户端或直接调用 IPC
/// 也必须受开关约束。只拦「创建 / 加入 / 大厅浏览」；`room_close` /
/// `room_heartbeat` / `room_get` 不拦，避免关闭开关后卡在房间内无法退出。
pub(super) async fn require_lobby_enabled(state: &AppState) -> Result<(), String> {
    if is_lobby_enabled(state).await {
        Ok(())
    } else {
        log_warn!("[Online] 联机大厅未启用，已拒绝房间/大厅操作");
        Err(LOBBY_DISABLED_MESSAGE.to_string())
    }
}

/// 注册全部信令 action 到 dispatcher
pub fn register_signaling_actions(d: &mut Dispatcher) {
    super::room_actions::register(d);
    super::lobby_actions::register(d);
}
