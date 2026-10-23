//! 联机大厅开关守卫（云端联机能力的前置校验）
//!
//! 开关关闭时统一拒绝「房间 / 大厅 / 公共 FRP 服务器列表」等云端联机能力；
//! 设备注册、启动上报、更新检查、公告拉取属基础联网项，不受此开关约束。

use crate::log_warn;
use crate::state::AppState;

/// 联机大厅关闭时的统一拒绝文案（与设置页开关卡片标题口径一致）
pub const LOBBY_DISABLED_MESSAGE: &str =
    "联机大厅未启用：请先在「设置 → 联机」页开启「启用联机大厅（房间与搭桥）」后再使用";

/// 联机大厅开关是否开启（配置项 `online.lobby_enabled`）
pub async fn is_lobby_enabled(state: &AppState) -> bool {
    state.config.lock().await.online.lobby_enabled
}

/// 需要云端联机能力的 action 守卫：开关关闭时直接拒绝
///
/// 前端已对大厅/房间/公共服务器分类做提示与封存，这里做同源兜底——
/// 旧版客户端或直接调用 IPC 也必须受开关约束。
pub async fn require_lobby_enabled(state: &AppState) -> Result<(), String> {
    if is_lobby_enabled(state).await {
        Ok(())
    } else {
        log_warn!("[Online] 联机大厅未启用，已拒绝云端联机操作");
        Err(LOBBY_DISABLED_MESSAGE.to_string())
    }
}
