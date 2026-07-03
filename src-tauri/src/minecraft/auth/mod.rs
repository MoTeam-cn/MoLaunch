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
/// 参考PCL2的实现，使用用户名的哈希值生成符合UUID v3格式的标识符
pub fn generate_offline_uuid(username: &str) -> String {
    // 计算用户名的稳定哈希值
    let hash = get_stable_hash_code(username);
    
    // 转换为16进制字符串
    let hash_hex = format!("{:016X}", hash);
    let length_hex = format!("{:016X}", username.len() as u64);
    
    // 组合
    let combined = format!("{}{}", length_hex, hash_hex);
    
    // 确保长度为32个字符
    let combined = if combined.len() < 32 {
        format!("{:0<32}", combined)
    } else {
        combined[..32].to_string()
    };
    
    // 格式化为UUID格式，设置版本为3
    let mut uuid_chars: Vec<char> = combined.chars().collect();
    uuid_chars[12] = '3'; // 版本号
    uuid_chars[16] = '9'; // 变体
    
    let uuid_str: String = uuid_chars.into_iter().collect();
    
    // 格式化为标准UUID格式
    format!(
        "{}-{}-{}-{}-{}",
        &uuid_str[0..8],
        &uuid_str[8..12],
        &uuid_str[12..16],
        &uuid_str[16..20],
        &uuid_str[20..32]
    )
}

/// 获取字符串的稳定哈希值
/// 参考Java的String.hashCode()实现
fn get_stable_hash_code(s: &str) -> u64 {
    let mut hash: i64 = 0;
    let mut multiplier: i64 = 1;
    
    for ch in s.chars().rev() {
        let char_value = ch as i64;
        hash = hash.wrapping_add(char_value.wrapping_mul(multiplier));
        multiplier = multiplier.wrapping_mul(31);
    }
    
    // 转换为无符号64位整数
    hash as u64
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