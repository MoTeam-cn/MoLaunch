//! 披风上传与删除端点

use super::types::YggdrasilError;
use super::{delete_texture, join_url, parse_error};

/// PUT /api/user/profile/{uuid}/cape 上传披风
///
/// 参考 4.4.1 节：multipart/form-data，仅含 `file` 字段（PNG），Bearer 认证。
pub async fn upload_cape(
    server_url: &str,
    access_token: &str,
    uuid: &str,
    png_bytes: Vec<u8>,
) -> Result<(), YggdrasilError> {
    let path = format!("/api/user/profile/{}/cape", uuid);
    let url = join_url(server_url, &path);

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(png_bytes)
            .file_name("cape.png")
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
            message: format!("上传披风请求失败: {}", e),
            is_network: true,
        })?;

    let status = resp.status().as_u16();
    if status == 204 {
        return Ok(());
    }
    let text = resp.text().await.unwrap_or_default();
    Err(parse_error(status, text))
}

/// DELETE /api/user/profile/{uuid}/cape 删除披风
pub async fn delete_cape(
    server_url: &str,
    access_token: &str,
    uuid: &str,
) -> Result<(), YggdrasilError> {
    delete_texture(server_url, access_token, uuid, "cape").await
}
