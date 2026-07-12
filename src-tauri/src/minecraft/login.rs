//! 登录模块
//! 支持离线登录，预留微软登录接口

use serde::{Deserialize, Serialize};

/// 登录类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginType {
    /// 离线登录
    Offline,
    /// 微软登录
    Microsoft,
    /// 第三方服务器（统一通行证）
    Nide,
    /// Authlib-Injector
    AuthlibInjector,
}

impl LoginType {
    pub fn name(&self) -> &str {
        match self {
            LoginType::Offline => "离线登录",
            LoginType::Microsoft => "微软登录",
            LoginType::Nide => "统一通行证",
            LoginType::AuthlibInjector => "Authlib-Injector",
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            LoginType::Offline => "Legacy".to_string(),
            LoginType::Microsoft => "Microsoft".to_string(),
            LoginType::Nide => "Nide".to_string(),
            LoginType::AuthlibInjector => "AuthlibInjector".to_string(),
        }
    }
}

/// 登录结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    /// 用户名
    pub username: String,
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

/// 登录错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginError {
    /// 错误消息
    pub message: String,
    /// 错误类型
    pub error_type: LoginErrorType,
}

/// 登录错误类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginErrorType {
    /// 网络错误
    Network,
    /// 认证失败
    AuthFailed,
    /// 账号问题
    AccountIssue,
    /// 服务器问题
    ServerIssue,
    /// 未知错误
    Unknown,
}

impl std::fmt::Display for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LoginError {}

/// 离线登录
pub fn login_offline(username: &str) -> Result<LoginResult, LoginError> {
    // 验证用户名
    if username.is_empty() {
        return Err(LoginError {
            message: "用户名不能为空".to_string(),
            error_type: LoginErrorType::AuthFailed,
        });
    }

    if username.len() > 16 {
        return Err(LoginError {
            message: "用户名不能超过16个字符".to_string(),
            error_type: LoginErrorType::AuthFailed,
        });
    }

    // 检查用户名字符
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(LoginError {
            message: "用户名只能包含字母、数字和下划线".to_string(),
            error_type: LoginErrorType::AuthFailed,
        });
    }

    // 生成UUID
    let uuid = crate::minecraft::auth::generate_offline_uuid(username);
    let access_token = uuid.clone();
    let client_token = uuid.clone();

    Ok(LoginResult {
        username: username.to_string(),
        uuid,
        access_token,
        client_token,
        login_type: LoginType::Offline,
        profile_json: None,
    })
}

/// 微软登录 (预留接口)
///
/// # 流程说明
/// 1. 获取设备码 (Device Code)
/// 2. 用户在浏览器中授权
/// 3. 轮询获取访问令牌
/// 4. 获取XBL令牌
/// 5. 获取XSTS令牌
/// 6. 获取Minecraft访问令牌
/// 7. 获取玩家信息
pub async fn login_microsoft(
    _device_code_callback: impl Fn(&str, &str),
) -> Result<LoginResult, LoginError> {
    // TODO: 实现微软登录
    // 这里预留接口，后续实现

    Err(LoginError {
        message: "微软登录功能暂未实现".to_string(),
        error_type: LoginErrorType::Unknown,
    })
}

/// 微软登录设备码响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
    pub message: String,
}

/// 微软令牌响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
    pub token_type: String,
}

/// XBL令牌响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XblTokenResponse {
    pub issue_instant: String,
    pub not_after: String,
    pub token: String,
    pub display_claims: serde_json::Value,
}

/// XSTS令牌响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XstsTokenResponse {
    pub issue_instant: String,
    pub not_after: String,
    pub token: String,
    pub display_claims: serde_json::Value,
}

/// Minecraft登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLoginResponse {
    pub username: String,
    pub roles: Vec<serde_json::Value>,
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
}

/// Minecraft档案响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftProfileResponse {
    pub id: String,
    pub name: String,
    pub skins: Vec<serde_json::Value>,
    pub capes: Vec<serde_json::Value>,
}

/// Authlib-Injector登录 (预留接口)
pub async fn login_authlib_injector(
    _server_url: &str,
    _username: &str,
    _password: &str,
) -> Result<LoginResult, LoginError> {
    // TODO: 实现Authlib-Injector登录
    Err(LoginError {
        message: "Authlib-Injector登录功能暂未实现".to_string(),
        error_type: LoginErrorType::Unknown,
    })
}

/// 统一通行证登录 (预留接口)
pub async fn login_nide(
    _server_id: &str,
    _username: &str,
    _password: &str,
) -> Result<LoginResult, LoginError> {
    // TODO: 实现统一通行证登录
    Err(LoginError {
        message: "统一通行证登录功能暂未实现".to_string(),
        error_type: LoginErrorType::Unknown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_login() {
        let result = login_offline("TestPlayer").unwrap();
        assert_eq!(result.username, "TestPlayer");
        assert_eq!(result.login_type, LoginType::Offline);
        assert!(!result.uuid.is_empty());
        assert!(!result.access_token.is_empty());
    }

    #[test]
    fn test_offline_login_empty_username() {
        let result = login_offline("");
        assert!(result.is_err());
    }

    #[test]
    fn test_offline_login_long_username() {
        let result = login_offline("ThisUsernameIsTooLong");
        assert!(result.is_err());
    }

    #[test]
    fn test_offline_login_invalid_chars() {
        let result = login_offline("User@Name");
        assert!(result.is_err());
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = crate::minecraft::auth::generate_offline_uuid("Player1");
        let uuid2 = crate::minecraft::auth::generate_offline_uuid("Player2");
        let uuid3 = crate::minecraft::auth::generate_offline_uuid("Player1");

        // 不同用户名应该生成不同的UUID
        assert_ne!(uuid1, uuid2);

        // 相同用户名应该生成相同的UUID
        assert_eq!(uuid1, uuid3);

        // 检查UUID格式（标准 UUID v3 字符串：36 字符，4 个连字符）
        assert_eq!(uuid1.len(), 36);
        assert_eq!(uuid1.chars().filter(|c| *c == '-').count(), 4);
        // UUID v3 的版本位（第 14 个字符，即第 3 段首位）应为 '3'
        assert_eq!(uuid1.chars().nth(14), Some('3'));
    }
}
