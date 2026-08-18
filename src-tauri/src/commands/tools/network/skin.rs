//! 正版玩家皮肤获取：
//! 1. `api.mojang.com/users/profiles/minecraft/{name}` → UUID
//! 2. `sessionserver.mojang.com/session/minecraft/profile/{uuid}` → properties.textures.value（base64）
//! 3. base64 解码 → textures.SKIN.url / CAPE.url / metadata.model（slim）
//! 4. 下载 SKIN / CAPE PNG → base64 data URI 返回（前端直接预览）
//!
//! 保存走 `skin_save_image`（base64 写 PNG 文件，带文件头与大小校验）。

use base64::Engine;
use serde_json::Value;

use crate::http::{get_client, request_error_msg};
use crate::log_info;
use crate::log_warn;
use crate::state::AppState;

use super::super::types::{SkinFetchParams, SkinFetchResult, SkinSaveImageParams};

const UUID_API: &str = "https://api.mojang.com/users/profiles/minecraft/";
const PROFILE_API: &str = "https://sessionserver.mojang.com/session/minecraft/profile/";
/// 皮肤/披风 PNG 大小上限（Mojang 官方皮肤限制约 24KB，这里放宽到 1MB）
const MAX_PNG_SIZE: usize = 1024 * 1024;

/// 获取正版玩家皮肤（玩家名 → UUID → textures → 下载皮肤/披风 PNG）
///
/// 失败时返回带 `error` 字段的 `SkinFetchResult`（而非 Err），前端在页面内展示原因。
pub async fn fetch_skin(
    _state: &AppState,
    params: SkinFetchParams,
) -> Result<serde_json::Value, String> {
    let name = params.name.trim().to_string();
    let result = match fetch_skin_inner(&name).await {
        Ok(r) => r,
        Err(e) => {
            log_warn!("[SkinFetch] failed: name={} err={}", name, e);
            SkinFetchResult {
                name,
                uuid: String::new(),
                skin_model: "classic".to_string(),
                skin_url: String::new(),
                skin_image: String::new(),
                cape_url: None,
                cape_image: None,
                error: e,
            }
        }
    };
    if result.error.is_empty() {
        log_info!(
            "[SkinFetch] success: name={} uuid={} model={} has_cape={}",
            result.name,
            result.uuid,
            result.skin_model,
            result.cape_url.is_some()
        );
    }
    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// 保存皮肤图片（base64 → 指定路径 PNG 文件）
pub async fn save_skin_image(
    _state: &AppState,
    params: SkinSaveImageParams,
) -> Result<serde_json::Value, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&params.image_base64)
        .map_err(|e| format!("图片 base64 解码失败: {}", e))?;

    // PNG 文件头校验 + 大小限制（参照 authlib read_png_file 的本地皮肤校验）
    const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < PNG_MAGIC.len() || &bytes[..PNG_MAGIC.len()] != PNG_MAGIC {
        return Err("图片不是有效的 PNG 文件".to_string());
    }
    if bytes.len() > MAX_PNG_SIZE {
        return Err(format!("图片大小超过 {}KB 限制", MAX_PNG_SIZE / 1024));
    }

    let path = std::path::Path::new(&params.save_path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }
    }
    std::fs::write(path, &bytes).map_err(|e| format!("保存文件失败: {}", e))?;
    log_info!("[SkinFetch] saved image to {}", path.display());
    Ok(serde_json::json!({ "success": true }))
}

async fn fetch_skin_inner(name: &str) -> Result<SkinFetchResult, String> {
    if name.is_empty() {
        return Err("请输入玩家名".to_string());
    }

    // 1. 玩家名 → UUID
    let (status, body) =
        crate::http::get_text_with_status(&format!("{UUID_API}{}", urlencoding::encode(name)))
            .await
            .map_err(|e| format!("获取玩家 UUID 失败: {}", e))?;
    if status == 404 || status == 400 {
        return Err(format!("未找到正版玩家「{}」，请确认名称拼写", name));
    }
    if status != 200 {
        return Err(format!("获取玩家 UUID 失败（HTTP {}）", status));
    }
    let profile_json: Value =
        serde_json::from_str(&body).map_err(|e| format!("玩家档案解析失败: {}", e))?;
    let uuid = profile_json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if uuid.is_empty() {
        return Err(format!("未找到正版玩家「{}」，请确认名称拼写", name));
    }

    // 2. UUID → textures base64
    let client = get_client();
    let profile_resp = client
        .get(format!("{PROFILE_API}{}", uuid))
        .send()
        .await
        .map_err(|e| format!("获取皮肤档案失败: {}", request_error_msg(&e)))?;
    if profile_resp.status() != 200 {
        return Err(format!(
            "获取皮肤档案失败（HTTP {}）",
            profile_resp.status()
        ));
    }
    let profile_body: Value = profile_resp
        .json()
        .await
        .map_err(|e| format!("皮肤档案解析失败: {}", e))?;
    let texture_b64 = profile_body
        .get("properties")
        .and_then(|p| p.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|prop| prop.get("name").and_then(|v| v.as_str()) == Some("textures"))
        })
        .and_then(|prop| prop.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if texture_b64.is_empty() {
        return Err("该玩家未设置自定义皮肤".to_string());
    }

    // 3. base64 解码 textures JSON
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&texture_b64)
        .map_err(|e| format!("皮肤数据解码失败: {}", e))?;
    let texture_json: Value =
        serde_json::from_slice(&decoded).map_err(|e| format!("皮肤数据解析失败: {}", e))?;
    let textures = texture_json.get("textures").cloned().unwrap_or_default();
    let skin_obj = textures.get("SKIN").cloned().unwrap_or_default();
    let skin_url = skin_obj
        .get("url")
        .and_then(|v| v.as_str())
        .map(|u| u.replace("http://", "https://"))
        .unwrap_or_default();
    if skin_url.is_empty() {
        return Err("该玩家未设置自定义皮肤".to_string());
    }
    let skin_model = skin_obj
        .get("metadata")
        .and_then(|m| m.get("model"))
        .and_then(|v| v.as_str())
        .unwrap_or("classic")
        .to_string();
    let cape_url = textures
        .get("CAPE")
        .and_then(|c| c.get("url"))
        .and_then(|v| v.as_str())
        .map(|u| u.replace("http://", "https://"));

    // 4. 下载皮肤 / 披风 PNG → base64 data URI
    let skin_image = download_image(&client, &skin_url).await?;
    let cape_image = match &cape_url {
        Some(url) => Some(download_image(&client, url).await?),
        None => None,
    };

    Ok(SkinFetchResult {
        name: profile_json
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(name)
            .to_string(),
        uuid,
        skin_model,
        skin_url,
        skin_image,
        cape_url,
        cape_image,
        error: String::new(),
    })
}

/// 下载图片并转 base64 data URI
async fn download_image(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("下载皮肤图片失败: {}", request_error_msg(&e)))?;
    if resp.status() != 200 {
        return Err(format!("下载皮肤图片失败（HTTP {}）", resp.status()));
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("读取皮肤图片失败: {}", e))?;
    if bytes.is_empty() {
        return Err("皮肤图片内容为空".to_string());
    }
    Ok(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}
