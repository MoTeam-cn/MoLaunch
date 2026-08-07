//! 启动配置中的认证与离线隔离处理。

use crate::minecraft::launch::AuthInfo;
use crate::state::AppState;
use crate::{log_info, log_warn};

/// 从后端认证存储构建认证信息，并校验请求 UUID。
pub(super) async fn resolve_auth(
    state: &AppState,
    username: String,
    uuid: String,
    login_type: Option<String>,
) -> AuthInfo {
    let login_type = login_type.unwrap_or_else(|| "Legacy".to_string());
    let (access_token, client_token, server_url) = match state.auth_storage.load().await {
        Ok(auth_state) => match auth_state.current_user {
            Some(current) if current.uuid == uuid => (
                current.access_token,
                current.client_token,
                current.server_url,
            ),
            Some(current) => {
                log_warn!(
                    "当前登录账号 UUID ({}) 与请求的 UUID ({}) 不一致，使用空 token",
                    current.uuid,
                    uuid
                );
                (String::new(), String::new(), None)
            }
            None => (String::new(), String::new(), None),
        },
        Err(e) => {
            log_warn!("从 auth_storage 加载 token 失败: {}，使用空 token", e);
            (String::new(), String::new(), None)
        }
    };

    AuthInfo {
        username,
        uuid,
        access_token,
        client_token,
        login_type,
        server_url,
    }
}

/// 应用离线账号皮肤的 UUID 变体与资源包，并清理非离线账号的资源包。
pub(super) async fn apply_offline_skin(
    state: &AppState,
    auth_info: AuthInfo,
    game_dir: &std::path::Path,
    version_id: &str,
) -> AuthInfo {
    let is_legacy = auth_info.login_type == "Legacy";
    let auth_info = if is_legacy {
        match state.auth_storage.load().await {
            Ok(auth_state) => match auth_state
                .offline_accounts
                .iter()
                .find(|account| account.uuid == auth_info.uuid)
            {
                Some(account) => match account.skin.as_deref() {
                    Some(skin_name) => {
                        let slim = if skin_name.starts_with("custom:") {
                            skin_name.contains("|slim")
                        } else {
                            matches!(
                                skin_name,
                                "Alex" | "Ari" | "Efe" | "Makena" | "Noor" | "Sunny" | "Zuri"
                            )
                        };
                        let adjusted_uuid = crate::minecraft::auth::adjust_uuid_for_skin_variant(
                            &auth_info.uuid,
                            slim,
                        );
                        if adjusted_uuid != auth_info.uuid {
                            log_info!(
                                "离线皮肤 UUID 调整: {} -> {} (skin={}, slim={})",
                                auth_info.uuid,
                                adjusted_uuid,
                                skin_name,
                                slim
                            );
                        }
                        AuthInfo {
                            uuid: adjusted_uuid,
                            ..auth_info
                        }
                    }
                    None => auth_info,
                },
                None => auth_info,
            },
            Err(e) => {
                log_warn!("加载离线账号皮肤失败: {}, 使用原始 UUID", e);
                auth_info
            }
        }
    } else {
        auth_info
    };

    if is_legacy {
        let skin_to_apply = state.auth_storage.load().await.ok().and_then(|state| {
            state
                .offline_accounts
                .iter()
                .find(|account| account.uuid == auth_info.uuid)
                .and_then(|account| account.skin.clone())
        });
        if let Err(e) = crate::minecraft::launch::skin_resourcepack::apply_skin_resourcepack(
            game_dir,
            version_id,
            skin_to_apply.as_deref(),
        ) {
            log_warn!("离线皮肤资源包生成失败: {}", e);
        }
    } else {
        crate::minecraft::launch::skin_resourcepack::remove_skin_resourcepack(game_dir);
    }

    auth_info
}
