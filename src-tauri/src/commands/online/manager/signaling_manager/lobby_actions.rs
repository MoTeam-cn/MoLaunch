//! 大厅浏览 action 注册（Scaffolding 收敛版）：聚合列表（按整合包）+ 某整合包下公开房间列表。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::utils::dispatcher::Dispatcher;

use super::LobbyListParams;

/// 注册大厅浏览相关 action
pub fn register(d: &mut Dispatcher) {
    register_list_lobby_packages(d);
    register_list_lobby_rooms(d);
}

fn register_list_lobby_packages(d: &mut Dispatcher) {
    d.register(
        "lobby_list_packages",
        handler!(state, _app, _params, {
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] lobby_list_packages");
            let result = client
                .signaling_list_lobby_packages(&creds)
                .await
                .map_err(|e| {
                    log_error!("[Online] lobby_list_packages 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}

fn register_list_lobby_rooms(d: &mut Dispatcher) {
    d.register(
        "lobby_list_rooms",
        handler!(state, _app, params, {
            let p: LobbyListParams = serde_json::from_value(params).unwrap_or_default();
            let creds = super::load_creds(&state).await?;
            let client = super::make_client(&state).await;
            log_debug!("[Online] lobby_list_rooms: package_id={:?}", p.package_id);
            let query = crate::minecraft::online::signaling::LobbyListQuery {
                package_id: p.package_id.clone(),
                page: p.page,
                page_size: p.page_size,
            };
            let result = client
                .signaling_list_lobby_rooms(&creds, &query)
                .await
                .map_err(|e| {
                    log_error!("[Online] lobby_list_rooms 失败: {}", e);
                    e.to_string()
                })?;
            serde_json::to_value(result).map_err(|e| e.to_string())
        }),
    );
}
