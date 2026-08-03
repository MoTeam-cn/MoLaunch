//! 披风管理：信息解析、下载、装备/取消装备（Mojang 官方 API）

use crate::http;
use crate::log_info;
use crate::log_warn;
use serde::{Deserialize, Serialize};

/// 披风管理端点
const CAPE_ACTIVE_URL: &str = "https://api.minecraftservices.com/minecraft/profile/capes/active";

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
    /// 缓存 URL（命中缓存时为 cache-image:// 本地 URL，未命中时为远程 URL）
    /// 由 get_skin_cape_info 命令填充，parse 时不设置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_url: Option<String>,
    /// 是否命中缓存
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached: Option<bool>,
}

/// 披风别名中文映射
pub(super) fn cape_display_name(alias: &str) -> String {
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

/// 获取当前已装备披风的下载 URL（从 profile 解析）
pub fn get_cape_url(profile_json: &str) -> Option<String> {
    let info = super::avatar::parse_skin_cape_info(profile_json).ok()?;
    info.capes
        .iter()
        .find(|c| c.state == "ACTIVE")
        .and_then(|c| c.url.clone())
}

/// 下载披风 PNG 二进制数据
pub async fn download_cape_png(cape_url: &str) -> Result<Vec<u8>, String> {
    crate::log_info!("[Skin] 下载披风: {}", cape_url);

    let client = http::get_client();
    let response = client
        .get(cape_url)
        .send()
        .await
        .map_err(|e| format!("download cape request error: {}", e))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        crate::log_warn!("[Skin] 披风下载失败: {} - {}", status, body);
        return Err(format!("download cape HTTP {}: {}", status, body));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("read cape bytes error: {}", e))?;

    log_info!("[Skin] 披风已下载: {} 字节", bytes.len());
    Ok(bytes.to_vec())
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