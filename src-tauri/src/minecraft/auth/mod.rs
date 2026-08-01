//! 认证模块

pub mod authlib;
pub mod microsoft;
pub mod storage;

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
/// 这样与官方启动器等保持一致，避免离线账号 UUID 因启动器不同而漂移。
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

/// 调整离线 UUID 以匹配指定的皮肤变体
///
/// Minecraft 离线模式判定皮肤变体的方法：取 UUID 第 8/16/24/32 位十六进制字符，
/// 将其异或后对 2 取模，结果为 0 对应 Steve（classic），为 1 对应 Alex（slim）。
/// 通过暴力递增 UUID 末位十六进制字符，直到判定结果与目标变体一致。
pub fn adjust_uuid_for_skin_variant(uuid: &str, slim: bool) -> String {
    let mut chars: Vec<char> = uuid.to_lowercase().chars().collect();
    let target: u32 = if slim { 1 } else { 0 };

    loop {
        // 提取所有十六进制字符（跳过连字符）
        let hex: Vec<char> = chars
            .iter()
            .copied()
            .filter(|c| c.is_ascii_hexdigit())
            .collect();

        // 第 8/16/24/32 位 hex 异或后 mod 2
        let v = hex[7].to_digit(16).unwrap()
            ^ hex[15].to_digit(16).unwrap()
            ^ hex[23].to_digit(16).unwrap()
            ^ hex[31].to_digit(16).unwrap();

        if v % 2 == target {
            return chars.iter().collect();
        }

        // 暴力递增最后一个十六进制字符（带进位）
        let mut i = chars.len();
        while i > 0 {
            i -= 1;
            if chars[i].is_ascii_hexdigit() {
                let val = chars[i].to_digit(16).unwrap();
                if val < 15 {
                    chars[i] = char::from_digit(val + 1, 16).unwrap();
                    break;
                } else {
                    chars[i] = '0';
                }
            }
        }
    }
}

/// 验证用户名是否有效
///
/// 允许 1-16 字符，支持中文、字母、数字、下划线、连字符
/// （离线模式仅本地使用，不与 Mojang 服务器交互，宽松限制即可）
pub fn validate_username(username: &str) -> bool {
    let chars: Vec<char> = username.chars().collect();
    let len = chars.len();

    // 长度检查：1-16 字符（按 Unicode 字符数计算，非字节数）
    if !(1..=16).contains(&len) {
        return false;
    }

    // 字符检查：允许中文、字母、数字、下划线、连字符
    chars
        .iter()
        .all(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
}
