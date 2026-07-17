//! 认证状态与本地认证结果

use serde::{Deserialize, Serialize};

/// 本地认证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAuthResult {
    /// 用户名
    pub name: String,
    /// UUID
    pub uuid: String,
    /// 访问令牌
    pub access_token: String,
    /// 客户端令牌
    pub client_token: String,
    /// 登录类型
    pub login_type: String,
    /// 微软登录时的档案信息
    pub profile_json: Option<String>,
}

/// 认证状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub current_user: Option<LocalAuthResult>,
    pub is_logged_in: bool,
}
