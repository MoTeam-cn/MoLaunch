//! 角色属性查询与皮肤/披风纹理解析

use super::types::YggdrasilError;
use super::{join_url, parse_error};
use crate::minecraft::auth::authlib::types::{ProfileInfo, SkinCapeInfo, TexturesPayload};

// yggdrasil 皮肤管理端点（参考 yggdrasil-api-analysis.md 4.3 / 4.4 节）
//
// 与认证端点不同，皮肤上传/删除需要 `Authorization: Bearer {accessToken}`
// 和 multipart/form-data，因此不能复用 `crate::http::post_json_with_status`，
// 但仍通过 `crate::http::get_client()` 复用全局 reqwest 客户端（统一 UA / 代理）。

/// GET /sessionserver/session/minecraft/profile/{uuid} 查询角色属性
///
/// 参考 4.3.1 节：返回完整角色信息（含 properties，textures 为 Base64 编码）。
/// 失败（角色不存在）返回 204 No Content，由调用方转为错误。
pub async fn fetch_profile(server_url: &str, uuid: &str) -> Result<ProfileInfo, YggdrasilError> {
    let path = format!("/sessionserver/session/minecraft/profile/{}", uuid);
    let url = join_url(server_url, &path);
    crate::log_debug!("[Authlib] fetch_profile 请求: {}", url);
    let (status, text) =
        crate::http::get_text_with_status(&url)
            .await
            .map_err(|e| YggdrasilError {
                status: 0,
                message: e.to_string(),
                is_network: true,
            })?;
    crate::log_debug!(
        "[Authlib] fetch_profile 响应: status={}, body 长度={} 字节",
        status,
        text.len()
    );
    // 204 表示角色不存在（规范定义 4.3.1 失败响应）
    if status == 204 || text.is_empty() {
        crate::log_debug!("[Authlib] fetch_profile 角色不存在 (uuid={})", uuid);
        return Err(YggdrasilError {
            status,
            message: format!("角色 {} 不存在", uuid),
            is_network: false,
        });
    }
    if status != 200 {
        crate::log_debug!(
            "[Authlib] fetch_profile 非 200 响应: status={}, body 前 200 字符={}",
            status,
            text.chars().take(200).collect::<String>()
        );
        return Err(parse_error(status, text));
    }
    let profile = serde_json::from_str::<ProfileInfo>(&text).map_err(|e| {
        crate::log_debug!(
            "[Authlib] fetch_profile 反序列化失败: {}, body 前 200 字符={}",
            e,
            text.chars().take(200).collect::<String>()
        );
        YggdrasilError {
            status: 0,
            message: format!("解析角色属性失败: {}", e),
            is_network: false,
        }
    })?;
    crate::log_debug!(
        "[Authlib] fetch_profile 解析成功: id={}, name={}, properties 数量={}",
        profile.id,
        profile.name,
        profile.properties.len()
    );
    Ok(profile)
}

/// 从 ProfileInfo 解析出皮肤披风信息
///
/// 遍历 `properties` 数组：
/// - `textures` 属性：Base64 解码为 `TexturesPayload`，提取 SKIN/CAPE URL 与 model
/// - `uploadableTextures` 属性：直接读字符串（"skin" / "cape" / "skin,cape"）
///
/// 解析失败时不阻断流程，仅跳过对应字段（保留默认值）。
pub fn parse_skin_cape_info(profile: &ProfileInfo) -> SkinCapeInfo {
    let mut skin_url = None;
    let mut skin_model = "default".to_string();
    let mut cape_url = None;
    let mut uploadable = String::new();

    crate::log_debug!(
        "[Authlib] parse_skin_cape_info 开始，properties 数量={}",
        profile.properties.len()
    );

    for (idx, prop) in profile.properties.iter().enumerate() {
        crate::log_debug!(
            "[Authlib] properties[{}]: name={}, value 前 80 字符={}",
            idx,
            prop.name,
            prop.value.chars().take(80).collect::<String>()
        );
        match prop.name.as_str() {
            "textures" => match decode_textures(&prop.value) {
                Some(payload) => {
                    crate::log_debug!(
                            "[Authlib] textures 解码成功: profile_id={}, profile_name={}, skin={:?}, cape={:?}",
                            payload.profile_id,
                            payload.profile_name,
                            payload.textures.skin.as_ref().map(|s| &s.url),
                            payload.textures.cape.as_ref().map(|c| &c.url)
                        );
                    if let Some(skin) = payload.textures.skin {
                        skin_url = Some(skin.url);
                        if let Some(meta) = skin.metadata {
                            if meta.model == "slim" {
                                skin_model = "slim".to_string();
                            }
                        }
                    }
                    cape_url = payload.textures.cape.map(|c| c.url);
                }
                None => {
                    crate::log_debug!(
                            "[Authlib] textures 解码失败（base64 或 JSON 解析失败），value 前 100 字符={}",
                            prop.value.chars().take(100).collect::<String>()
                        );
                }
            },
            "uploadableTextures" => {
                uploadable = prop.value.clone();
                crate::log_debug!("[Authlib] uploadableTextures 命中: value={}", uploadable);
            }
            _ => {
                crate::log_debug!("[Authlib] 跳过未识别的 property: {}", prop.name);
            }
        }
    }

    let result = SkinCapeInfo {
        skin_url,
        skin_model,
        cape_url,
        uploadable_textures: uploadable,
    };
    crate::log_debug!(
        "[Authlib] parse_skin_cape_info 完成: skin_url={:?}, skin_model={}, cape_url={:?}, uploadable_textures={:?}",
        result.skin_url,
        result.skin_model,
        result.cape_url,
        result.uploadable_textures
    );
    result
}

/// Base64 解码 textures 属性的 value 并反序列化为 TexturesPayload
///
/// 失败返回 None，调用方按"无 textures"处理。
fn decode_textures(b64: &str) -> Option<TexturesPayload> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(b64.as_bytes())
        .ok()?;
    let json = String::from_utf8(bytes).ok()?;
    crate::log_debug!(
        "[Authlib] decode_textures JSON 解码成功，前 200 字符={}",
        json.chars().take(200).collect::<String>()
    );
    serde_json::from_str(&json).ok()
}
