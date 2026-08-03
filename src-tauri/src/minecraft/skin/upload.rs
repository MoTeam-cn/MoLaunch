//! 皮肤上传与 profile 刷新（multipart/form-data，Mojang 官方 API）

use crate::http;
use crate::log_info;
use crate::log_warn;

/// 皮肤上传端点
const SKIN_UPLOAD_URL: &str = "https://api.minecraftservices.com/minecraft/profile/skins";

/// Minecraft profile 端点（用于上传/装备后刷新本地缓存的 profile_json）
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

/// 上传/修改皮肤
///
/// `variant`: "classic"（Steve 模型）或 "slim"（Alex 模型）
/// `png_data`: PNG 文件二进制内容
pub async fn upload_skin(
    access_token: &str,
    png_data: Vec<u8>,
    variant: &str,
) -> Result<(), String> {
    log_info!(
        "[Skin] 上传皮肤: model={}, size={} 字节",
        variant,
        png_data.len()
    );

    let client = http::get_client();

    // 使用 multipart/form-data
    let form = reqwest::multipart::Form::new()
        .text("variant", variant.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(png_data)
                .file_name("skin.png")
                .mime_str("image/png")
                .map_err(|e| format!("mime error: {}", e))?,
        );

    let response = client
        .post(SKIN_UPLOAD_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "*/*")
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("upload request error: {}", e))?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log_warn!("[Skin] 皮肤上传失败: {} - {}", status, body);
        return Err(format!("upload HTTP {}: {}", status, body));
    }

    log_info!("[Skin] 皮肤上传成功");
    Ok(())
}

/// 重新获取玩家档案（用于上传皮肤/装备披风后刷新本地缓存的 profile_json）
///
/// 返回最新的 profile JSON 字符串，调用方应将其写入 state.auth.current_user.profile_json
pub async fn fetch_profile(access_token: &str) -> Result<String, String> {
    log_info!("[Skin] 刷新 Minecraft profile");

    let client = http::get_client();
    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("profile request error: {}", e))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log_warn!("[Skin] profile 刷新失败: {} - {}", status, body_text);
        return Err(format!("profile HTTP {}: {}", status, body_text));
    }

    log_info!("[Skin] profile 刷新成功");
    Ok(body_text)
}
