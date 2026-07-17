//! 皮肤与披风管理模块
//!
//! 参考 PCL2 的实现：
//! - 皮肤 URL 直接从 profile_json 的 skins[].url 获取（textures.minecraft.net）
//! - 头像通过下载皮肤 PNG 后由前端 canvas 裁剪 (8,8,8,8) 区域（PCL2 的方式）
//! - 上传/修改皮肤（multipart/form-data）
//! - 装备/取消披风

use crate::http;
use crate::log_info;
use crate::log_warn;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────
// 关于不使用 sources::fetch_with_fallback 的说明
// ─────────────────────────────────────────────────────────────
// 本模块的所有 HTTP 请求目标为 Mojang 官方 API：
//   - textures.minecraft.net（皮肤/披风 PNG 二进制）
//   - api.minecraftservices.com（profile / skins / capes）
// 这些端点没有 BMCLAPI 镜像，且 PNG 下载为二进制流，
// 而 sources::fetch_with_fallback 仅返回 String 文本。
// 因此本模块直接使用 http::get_client()，不经过 sources 模块。

/// 皮肤上传端点
const SKIN_UPLOAD_URL: &str = "https://api.minecraftservices.com/minecraft/profile/skins";

/// 披风管理端点
const CAPE_ACTIVE_URL: &str = "https://api.minecraftservices.com/minecraft/profile/capes/active";

/// Minecraft profile 端点（用于上传/装备后刷新本地缓存的 profile_json）
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

// ============================================================
// 数据结构
// ============================================================

/// 皮肤信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
    pub alias: Option<String>,
}

/// 披风信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapeInfo {
    pub id: String,
    pub state: String,
    pub alias: String,
    /// 中文名（由 alias 映射）
    pub display_name: String,
    /// 披风 PNG 下载地址（来自 profile_json 的 capes[].url）
    pub url: Option<String>,
}

/// 皮肤/披风完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinCapeInfo {
    pub skins: Vec<SkinInfo>,
    pub capes: Vec<CapeInfo>,
}

// ============================================================
// 披风别名中文映射（与 PCL2 一致）
// ============================================================

fn cape_display_name(alias: &str) -> String {
    let map = [
        ("Migrator", "迁移者披风"),
        ("MapMaker", "Realms 地图制作者披风"),
        ("Moderator", "Mojira 管理员披风"),
        ("Translator-Chinese", "Crowdin 中文翻译者披风"),
        ("Translator", "Crowdin 翻译者披风"),
        ("Cobalt", "Cobalt 披风"),
        ("Vanilla", "原版披风"),
        ("Minecon2011", "Minecon 2011 参与者披风"),
        ("Minecon2012", "Minecon 2012 参与者披风"),
        ("Minecon2013", "Minecon 2013 参与者披风"),
        ("Minecon2015", "Minecon 2015 参与者披风"),
        ("Minecon2016", "Minecon 2016 参与者披风"),
        ("Cherry Blossom", "樱花披风"),
        ("15th Anniversary", "15 周年纪念披风"),
        ("Purple Heart", "紫色心形披风"),
        ("Follower's", "追随者披风"),
        ("MCC 15th Year", "MCC 15 周年披风"),
        ("Minecraft Experience", "村民救援披风"),
        ("Mojang Office", "Mojang 办公室披风"),
        ("Home", "家园披风"),
        ("Menace", "入侵披风"),
        ("Yearn", "渴望披风"),
        ("Common", "普通披风"),
        ("Pan", "薄煎饼披风"),
        ("Founder's", "创始人披风"),
        ("Copper", "铜披风"),
        ("Zombie Horse", "僵尸马披风"),
        ("Builder", "建造者披风"),
        ("Crafter", "工匠披风"),
    ];
    for (key, name) in map.iter() {
        if alias == *key {
            return name.to_string();
        }
    }
    alias.to_string()
}

// ============================================================
// 核心逻辑
// ============================================================

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
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(SkinCapeInfo { skins, capes })
}

/// 获取皮肤 PNG 下载 URL（从 profile 解析，参考 PCL2）
///
/// PCL2 的逻辑：
/// 1. 优先从 profile_json 的 skins[].url 取
/// 2. 将 http:// 替换为 https://（minecraft.net 域名才替换）
pub fn get_skin_url(profile_json: &str) -> Option<String> {
    let info = parse_skin_cape_info(profile_json).ok()?;
    info.skins
        .iter()
        .find(|s| s.state == "ACTIVE")
        .or_else(|| info.skins.first())
        .map(|s| {
            // PCL2: If SkinUrl.Contains("minecraft.net/") Then SkinUrl.Replace("http://", "https://")
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
    let response = client
        .get(skin_url)
        .send()
        .await
        .map_err(|e| format!("download skin request error: {}", e))?;

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

/// 获取当前已装备披风的下载 URL（从 profile 解析）
pub fn get_cape_url(profile_json: &str) -> Option<String> {
    let info = parse_skin_cape_info(profile_json).ok()?;
    info.capes
        .iter()
        .find(|c| c.state == "ACTIVE")
        .and_then(|c| c.url.clone())
}

/// 下载披风 PNG 二进制数据
pub async fn download_cape_png(cape_url: &str) -> Result<Vec<u8>, String> {
    log_info!("[Skin] 下载披风: {}", cape_url);

    let client = http::get_client();
    let response = client
        .get(cape_url)
        .send()
        .await
        .map_err(|e| format!("download cape request error: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        log_warn!("[Skin] 披风下载失败: {} - {}", status, body);
        return Err(format!("download cape HTTP {}: {}", status, body));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("read cape bytes error: {}", e))?;

    log_info!("[Skin] 披风已下载: {} 字节", bytes.len());
    Ok(bytes.to_vec())
}

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

    // 使用 multipart/form-data（与 PCL2 一致）
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

/// 装备披风
pub async fn equip_cape(access_token: &str, cape_id: &str) -> Result<(), String> {
    log_info!("[Skin] 装备披风: {}", cape_id);

    let client = http::get_client();
    let body = serde_json::json!({ "capeId": cape_id });

    let response = client
        .put(CAPE_ACTIVE_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("equip cape request error: {}", e))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log_warn!("[Skin] 装备披风失败: {} - {}", status, body_text);
        return Err(format!("equip cape HTTP {}: {}", status, body_text));
    }

    log_info!("[Skin] 披风装备成功");
    Ok(())
}

/// 取消披风
pub async fn unequip_cape(access_token: &str) -> Result<(), String> {
    log_info!("[Skin] 取消披风装备");

    let client = http::get_client();

    let response = client
        .delete(CAPE_ACTIVE_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("unequip cape request error: {}", e))?;

    let status = response.status();
    let body_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        log_warn!("[Skin] 取消披风失败: {} - {}", status, body_text);
        return Err(format!("unequip cape HTTP {}: {}", status, body_text));
    }

    log_info!("[Skin] 披风已取消装备");
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
