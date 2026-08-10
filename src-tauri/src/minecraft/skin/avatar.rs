//! 皮肤/头像信息：从 profile_json 解析皮肤、皮肤 URL 处理与 PNG 下载

use crate::http;
use crate::log_info;
use crate::log_warn;
use serde::{Deserialize, Serialize};

use super::cape::{cape_display_name, CapeInfo};

/// 皮肤信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
    pub alias: Option<String>,
    /// 缓存 URL（命中缓存时为 cache-image:// 本地 URL，未命中时为远程 URL）
    /// 由 get_skin_cape_info 命令填充，parse 时不设置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_url: Option<String>,
    /// 是否命中缓存
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<bool>,
}

/// 皮肤/披风完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinCapeInfo {
    pub skins: Vec<SkinInfo>,
    pub capes: Vec<CapeInfo>,
}

/// 从 profile_json 解析皮肤/披风信息
pub fn parse_skin_cape_info(profile_json: &str) -> Result<SkinCapeInfo, String> {
    let profile: serde_json::Value =
        serde_json::from_str(profile_json).map_err(|e| format!("parse profile error: {}", e))?;

    let skins = profile
        .get("skins")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| {
                    Some(SkinInfo {
                        id: s.get("id")?.as_str()?.to_string(),
                        state: s.get("state")?.as_str()?.to_string(),
                        url: s.get("url")?.as_str()?.to_string(),
                        variant: s
                            .get("variant")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        alias: s.get("alias").and_then(|v| v.as_str()).map(String::from),
                        cached_url: None,
                        cached: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let capes = profile
        .get("capes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| {
                    let alias = c.get("alias")?.as_str()?.to_string();
                    Some(CapeInfo {
                        id: c.get("id")?.as_str()?.to_string(),
                        state: c.get("state")?.as_str()?.to_string(),
                        alias: alias.clone(),
                        display_name: cape_display_name(&alias),
                        url: c.get("url").and_then(|v| v.as_str()).map(String::from),
                        cached_url: None,
                        cached: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(SkinCapeInfo { skins, capes })
}

/// 获取皮肤 PNG 下载 URL（从 profile 解析）
///
/// 逻辑：
/// 1. 优先从 profile_json 的 skins[].url 取
/// 2. 将 http:// 替换为 https://（minecraft.net 域名才替换）
pub fn get_skin_url(profile_json: &str) -> Option<String> {
    let info = parse_skin_cape_info(profile_json).ok()?;
    info.skins
        .iter()
        .find(|s| s.state == "ACTIVE")
        .or_else(|| info.skins.first())
        .map(|s| {
            // 如果 URL 包含 minecraft.net/，将 http:// 替换为 https://
            if s.url.contains("minecraft.net/") {
                s.url.replace("http://", "https://")
            } else {
                s.url.clone()
            }
        })
}

/// 下载皮肤 PNG 二进制数据
///
/// 直接从 textures.minecraft.net 下载，返回 PNG 字节流
pub async fn download_skin_png(skin_url: &str) -> Result<Vec<u8>, String> {
    log_info!("[Skin] 下载皮肤: {}", skin_url);

    let client = http::get_client();
    let response = client.get(skin_url).send().await.map_err(|e| {
        format!(
            "download skin request error: {}",
            crate::http::request_error_msg(&e)
        )
    })?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log_warn!("[Skin] 皮肤下载失败: {} - {}", status, body);
        return Err(format!("download skin HTTP {}: {}", status, body));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("read skin bytes error: {}", e))?;

    log_info!("[Skin] 皮肤已下载: {} 字节", bytes.len());
    Ok(bytes.to_vec())
}
