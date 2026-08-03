//! yggdrasil HTTP 通用辅助函数
//!
//! join_url / parse_error / delete_texture 供 auth/meta/profile/skin/cape 子模块复用。

use super::super::types::ApiError;
use super::types::YggdrasilError;

/// 拼接完整 URL（自动去除 server_url 末尾的 `/`）
pub(crate) fn join_url(server_url: &str, path: &str) -> String {
    let base = server_url.trim_end_matches('/');
    format!("{}{}", base, path)
}

/// 解析 yggdrasil 错误响应
pub(crate) fn parse_error(status: u16, body: String) -> YggdrasilError {
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
pub(crate) async fn delete_texture(
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