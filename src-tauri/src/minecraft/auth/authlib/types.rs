//! yggdrasil 协议数据结构
//!
//! 参考 authlib-injector 规范：https://github.com/yushijinhun/authlib-injector
//! 所有请求/响应结构均与 yggdrasil API 规范兼容（如 LittleSkin、Blessing Skin Server）。

use serde::{Deserialize, Serialize};

// ===== 请求结构 =====

/// `/authserver/authenticate` 请求体
///
/// `rename_all = "camelCase"`：yggdrasil 规范要求请求字段为 camelCase
/// （如 `requestUser` 而非 `request_user`），否则严格的服务器会拒绝。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub agent: Agent,
    pub username: String,
    pub password: String,
    pub request_user: bool,
}

impl AuthenticateRequest {
    pub fn new(username: String, password: String) -> Self {
        Self {
            agent: Agent::default(),
            username,
            password,
            request_user: true,
        }
    }
}

/// `/authserver/authenticate` 请求中的 agent 字段
#[derive(Debug, Serialize)]
pub struct Agent {
    pub name: String,
    pub version: u32,
}

impl Default for Agent {
    fn default() -> Self {
        Self {
            name: "Minecraft".to_string(),
            version: 1,
        }
    }
}

/// `/authserver/refresh` 请求体
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub access_token: String,
    pub client_token: Option<String>,
    pub selected_profile: Option<ProfileId>,
    pub request_user: bool,
}

/// `/authserver/validate` 请求体
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateRequest {
    pub access_token: String,
    pub client_token: Option<String>,
}

/// `/authserver/refresh` 中指定的 profile（仅含 id+name）
#[derive(Debug, Serialize)]
pub struct ProfileId {
    pub id: String,
    pub name: String,
}

// ===== 响应结构 =====

/// `/authserver/authenticate` 与 `/authserver/refresh` 共用响应体
///
/// `rename_all = "camelCase"`：yggdrasil 规范要求响应字段为 camelCase
/// （如 `accessToken` 而非 `access_token`）。
/// 缺失此标注会导致 `missing field access_token` 反序列化错误。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub access_token: String,
    pub client_token: String,
    #[serde(default)]
    pub available_profiles: Vec<Profile>,
    #[serde(default)]
    pub selected_profile: Option<Profile>,
    #[serde(default)]
    pub user: Option<User>,
}

/// yggdrasil 角色（profile）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
}

/// yggdrasil 用户信息
#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(default)]
    pub properties: Vec<UserProperty>,
}

/// 用户属性
#[derive(Debug, Deserialize)]
pub struct UserProperty {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub signature: Option<String>,
}

/// 服务器根元数据（GET / 响应）
///
/// yggdrasil 标准格式：元信息在 `meta` 对象内（serverName/implementationName/links），
/// 顶层含 `signaturePublickey`。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMetadata {
    #[serde(default)]
    pub meta: serde_json::Value,
    /// 签名公钥（部分服务器在根级别提供，用于 prefetched 校验）
    #[serde(default)]
    pub signature_publickey: Option<String>,
}

impl ServerMetadata {
    /// 从 meta 对象提取服务器名
    pub fn server_name(&self) -> String {
        self.meta
            .get("serverName")
            .and_then(|v| v.as_str())
            .unwrap_or("未知服务器")
            .to_string()
    }

    /// 从 meta.links 提取注册链接
    pub fn register_url(&self) -> Option<String> {
        self.meta
            .get("links")
            .and_then(|l| l.get("register"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// 从 meta.links 提取主页链接
    pub fn homepage_url(&self) -> Option<String> {
        self.meta
            .get("links")
            .and_then(|l| l.get("homepage"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

/// yggdrasil API 错误响应
#[derive(Debug, Deserialize)]
pub struct ApiError {
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub error_message: String,
    #[serde(default)]
    pub cause: String,
}

impl ApiError {
    pub fn message(&self) -> String {
        if !self.error_message.is_empty() {
            self.error_message.clone()
        } else if !self.error.is_empty() {
            self.error.clone()
        } else {
            "未知错误".to_string()
        }
    }
}

// ============================================================
// 角色属性与材质（GET /sessionserver/session/minecraft/profile/{uuid}）
//
// 参考 yggdrasil-api-analysis.md 3.2 节与 4.3.1 节。
// ============================================================

/// 角色属性（properties 数组元素）
///
/// `value` 通常是 Base64 编码的 JSON（textures 属性）或纯字符串（uploadableTextures）。
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub signature: Option<String>,
}

/// 完整角色信息（GET /sessionserver/session/minecraft/profile/{uuid} 响应）
///
/// 4.3.1 节：成功返回 200 + 完整角色信息（含属性），失败返回 204 No Content。
#[derive(Debug, Clone, Deserialize)]
pub struct ProfileInfo {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub properties: Vec<ProfileProperty>,
}

/// textures 属性解码后的材质信息（Base64 解码后 JSON 反序列化）
///
/// 格式见 yggdrasil-api-analysis.md 3.2 节。
/// 字段为 camelCase（`profileId` / `profileName`）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesPayload {
    pub timestamp: u64,
    pub profile_id: String,
    pub profile_name: String,
    pub textures: Textures,
}

/// textures 字段（含 SKIN / CAPE）
///
/// yggdrasil 规范字段名为大写（`SKIN` / `CAPE`），通过显式 `rename` 标注兼容。
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Textures {
    #[serde(default, rename = "SKIN")]
    pub skin: Option<TextureUrl>,
    #[serde(default, rename = "CAPE")]
    pub cape: Option<TextureUrl>,
}

/// 单个材质 URL（SKIN / CAPE）
#[derive(Debug, Clone, Deserialize)]
pub struct TextureUrl {
    pub url: String,
    #[serde(default)]
    pub metadata: Option<TextureMetadata>,
}

/// 材质元数据（仅皮肤有，含 model 字段）
#[derive(Debug, Clone, Deserialize)]
pub struct TextureMetadata {
    #[serde(default)]
    pub model: String,
}

/// 皮肤披风信息（前端展示用，由 `parse_skin_cape_info` 解析 ProfileInfo 后生成）
///
/// 一次性返回皮肤 URL、模型、披风 URL 与可上传材质类型，避免前端再次解码 Base64。
#[derive(Debug, Clone, Serialize)]
pub struct SkinCapeInfo {
    /// 皮肤 URL（无皮肤时为 None）
    pub skin_url: Option<String>,
    /// 皮肤模型（"default" 或 "slim"）
    pub skin_model: String,
    /// 披风 URL（无披风时为 None）
    pub cape_url: Option<String>,
    /// 可上传的材质类型（"skin" / "cape" / "skin,cape"，空串表示不能上传）
    ///
    /// 来自 `uploadableTextures` 属性，详见 yggdrasil-api-analysis.md 3.2 节。
    pub uploadable_textures: String,
}
