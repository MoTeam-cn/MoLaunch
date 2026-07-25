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
    /// authlib 登录的 yggdrasil API 根地址（仅 authlib 登录有）
    /// 启动游戏时用于构建 -Dauthlibinjector.yggdrasil.prefetched 参数
    #[serde(default)]
    pub server_url: Option<String>,
    /// authlib 登录的服务器显示名（仅 authlib 登录有，用于 UI 展示）
    #[serde(default)]
    pub server_name: Option<String>,
}

/// 认证状态
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthState {
    pub current_user: Option<LocalAuthResult>,
    pub is_logged_in: bool,
}
