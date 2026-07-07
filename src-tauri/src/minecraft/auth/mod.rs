//! 认证模块

use serde::{Deserialize, Serialize};

/// 登录类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoginType {
    /// 离线登录
    Legacy,
    /// 微软正版
    Microsoft,
    /// 第三方服务器（统一通行证）
    Nide,
    /// Authlib-Injector
    AuthlibInjector,
}

/// 登录结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    /// 用户名
    pub name: String,
    /// UUID
    pub uuid: String,
    /// 访问令牌
    pub access_token: String,
    /// 客户端令牌
    pub client_token: String,
    /// 登录类型
    pub login_type: LoginType,
    /// 微软登录时的档案信息
    pub profile_json: Option<String>,
}

/// 离线登录
pub fn login_offline(username: &str) -> LoginResult {
    let uuid = generate_offline_uuid(username);
    let access_token = uuid.clone();
    let client_token = uuid.clone();
    
    LoginResult {
        name: username.to_string(),
        uuid,
        access_token,
        client_token,
        login_type: LoginType::Legacy,
        profile_json: None,
    }
}

/// 生成离线UUID
/// 使用标准 Minecraft 离线 UUID 算法（UUID v3），即对 `"OfflinePlayer:" + username` 取 MD5，
/// 然后按 RFC 4122 v3 格式化（设置 version 位为 3，variant 位为 0b10xx）。
/// 这样与官方启动器、PCL2 等保持一致，避免离线账号 UUID 因启动器不同而漂移。
pub fn generate_offline_uuid(username: &str) -> String {
    let digest = md5::compute(format!("OfflinePlayer:{}", username));
    let mut bytes = digest.0;

    // 设置 version 位为 3（UUID v3）：清零 byte[6] 高 4 位后置为 0011
    bytes[6] = (bytes[6] & 0x0F) | 0x30;
    // 设置 variant 位为 0b10xx：清零 byte[8] 高 2 位后置为 10
    bytes[8] = (bytes[8] & 0x3F) | 0x80;

    // 格式化为标准 UUID 字符串（8-4-4-4-12，小写十六进制）
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// 验证用户名是否有效
pub fn validate_username(username: &str) -> bool {
    // 用户名长度检查
    if username.len() < 3 || username.len() > 16 {
        return false;
    }
    
    // 用户名字符检查（只允许字母、数字、下划线）
    username.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}