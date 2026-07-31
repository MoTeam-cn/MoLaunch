//! 大厅浏览 action 注册（联机大厅阶段 5）：公开房间列表查询、分类列表查询。

use crate::handler;
use crate::log_debug;
use crate::log_error;
use crate::minecraft::online::signaling::LobbyListQuery;
use crate::utils::dispatcher::Dispatcher;

use super::LobbyListParams;

/// 注册大厅浏览相关 action
pub fn register(d: &mut Dispatcher) {
    register_list_lobby_rooms(d);
    register_list_lobby_categories(d);
}

fn register_list_lobby_rooms(d: &mut Dispatcher) {
    d.register("lobby_list_rooms", handler!(state, _app, params, {
        let p: LobbyListParams = serde_json::from_value(params)
            .unwrap_or_default();
        let creds = super::load_creds(&state).await?;
        let client = super::make_client(&state).await;
        log_debug!(
            "[Online] lobby_list_rooms: lobby={:?}, page={:?}, size={:?}, loader={:?}, keyword={:?}",
            p.lobby_id, p.page, p.page_size, p.loader, p.keyword
        );
        let query = LobbyListQuery {
            lobby_id: p.lobby_id,
            page: p.page,
            page_size: p.page_size,
            has_modpack: p.has_modpack,
            loader: p.loader,
            game_version: p.game_version,
            keyword: p.keyword,
        };
        let result = client.signaling_list_lobby_rooms(&creds, &query).await
            .map_err(|e| {
                log_error!("[Online] lobby_list_rooms 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}

fn register_list_lobby_categories(d: &mut Dispatcher) {
    d.register("lobby_list_categories", handler!(state, _app, _params, {
        let creds = super::load_creds(&state).await?;
        let client = super::make_client(&state).await;
        log_debug!("[Online] lobby_list_categories");
        let result = client.signaling_list_lobby_categories(&creds).await
            .map_err(|e| {
                log_error!("[Online] lobby_list_categories 失败: {}", e);
                e.to_string()
            })?;
        serde_json::to_value(result).map_err(|e| e.to_string())
    }));
}
