//! 认证状态与本地认证结果

use serde::{Deserialize, Serialize};

/// 本地认证结果
///
/// 安全说明：
/// - `access_token` / `client_token` 标记 `#[serde(skip)]`，**不会序列化到 IPC 返回前端**。
///   启动游戏时由 `build_launch_config` 直接从后端 `auth_storage` 读取 token 注入启动参数，
///   前端无需也无法访问 token 明文。
/// - `profile_json` 保留序列化：微软账号皮肤/披风信息（前端 `useSkinOperations` /
///   `AccountCard` 解析用于头像显示），不含 token。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAuthResult {
    /// 用户名
    pub name: String,
    /// UUID
    pub uuid: String,
    /// 访问令牌（不暴露给前端，启动时由后端直接注入）
    #[serde(skip)]
    pub access_token: String,
    /// 客户端令牌（不暴露给前端）
    #[serde(skip)]
    pub client_token: String,
    /// 登录类型
    pub login_type: String,
    /// 微软登录时的档案信息（含皮肤/披风 URL，不含 token）
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
