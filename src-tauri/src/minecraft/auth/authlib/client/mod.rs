//! yggdrasil HTTP 客户端：认证/刷新/校验、服务器元数据、authlib-injector.jar 下载、角色与皮肤披风管理。
//! server_url 为 yggdrasil API 根（如 `https://littleskin.cn/api/yggdrasil`），
//! 请求统一走 `crate::http`。子模块：types（错误/元数据类型）、meta（服务器元数据/authlib-injector）、
//! auth（authenticate/validate/refresh）、profile（角色属性/纹理解析）、skin（皮肤）、cape（披风）。

mod auth;
mod cape;
mod meta;
mod profile;
mod skin;
mod types;

pub use auth::{authenticate, refresh, validate};
pub use cape::{delete_cape, upload_cape};
pub use meta::{ensure_authlib_injector_jar, fetch_authlib_injector_meta, fetch_server_metadata};
pub use profile::{fetch_profile, parse_skin_cape_info};
pub use skin::{delete_skin, upload_skin};
pub use types::{AuthlibInjectorMeta, YggdrasilError};

use super::types::ApiError;

/// 拼接完整 URL（自动去除 server_url 末尾的 `/`）
pub(super) fn join_url(server_url: &str, path: &str) -> String {
    let base = server_url.trim_end_matches('/');
    format!("{}{}", base, path)
}

/// 解析 yggdrasil 错误响应
pub(super) fn parse_error(status: u16, body: String) -> YggdrasilError {
    let message = serde_json::from_str::<ApiError>(&body)
        .map(|e| e.message())
        .unwrap_or_else(|_| {
            if body.is_empty() {
                format!("HTTP {}", status)
            } else {
                body.chars().take(200).collect()
            }
        });
    YggdrasilError {
        status,
        message,
        is_network: false,
    }
}

/// 通用删除材质（skin / cape）
///
/// DELETE /api/user/profile/{uuid}/{textureType}，Bearer 认证，成功 204。
pub(super) async fn delete_texture(
    server_url: &str,
    access_token: &str,
    uuid: &str,
    texture_type: &str,
) -> Result<(), YggdrasilError> {
    let path = format!("/api/user/profile/{}/{}", uuid, texture_type);
    let url = join_url(server_url, &path);

    let client = crate::http::get_client();
    let resp = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: format!("删除材质请求失败: {}", e),
            is_network: true,
        })?;

    let status = resp.status().as_u16();
    if status == 204 {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(parse_error(status, text))
}
