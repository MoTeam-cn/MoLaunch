//! 皮肤上传与删除端点

use super::types::YggdrasilError;
use super::{delete_texture, join_url, parse_error};

/// PUT /api/user/profile/{uuid}/skin 上传皮肤
///
/// 参考 4.4.1 节：multipart/form-data，`model` 字段为 "slim" 或空串，
/// `file` 字段为 PNG 二进制（Content-Type: image/png），认证用 Bearer Token。
/// 成功响应 204 No Content。
pub async fn upload_skin(
    server_url: &str,
    access_token: &str,
    uuid: &str,
    png_bytes: Vec<u8>,
    model: &str,
) -> Result<(), YggdrasilError> {
    let path = format!("/api/user/profile/{}/skin", uuid);
    let url = join_url(server_url, &path);

    let form = reqwest::multipart::Form::new()
        .text("model", model.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(png_bytes)
                .file_name("skin.png")
                .mime_str("image/png")
                .map_err(|e| YggdrasilError {
                    status: 0,
                    message: format!("构造 multipart 失败: {}", e),
                    is_network: false,
                })?,
        );

    let client = crate::http::get_client();
    let resp = client
        .put(&url)
        .header("Authorization", format!("Bearer {}", access_token))
        .multipart(form)
        .send()
        .await
        .map_err(|e| YggdrasilError {
            status: 0,
            message: format!("上传皮肤请求失败: {}", e),
            is_network: true,
        })?;

    let status = resp.status().as_u16();
    if status == 204 {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(parse_error(status, text))
}

/// DELETE /api/user/profile/{uuid}/skin 删除皮肤
///
/// 参考 4.4.2 节：Bearer 认证，成功 204。
pub async fn delete_skin(
    server_url: &str,
    access_token: &str,
    uuid: &str,
) -> Result<(), YggdrasilError> {
    delete_texture(server_url, access_token, uuid, "skin").await
}
