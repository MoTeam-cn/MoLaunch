//! 启动参数相关类型
//!
//! `LaunchArguments` 与 `AuthInfo`（含脱敏 Debug 实现）。

use serde::{Deserialize, Serialize};

/// Launch arguments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchArguments {
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub main_class: String,
    pub classpath: String,
    pub version_id: String,
    pub game_dir: String,
    pub assets_dir: String,
    pub asset_index: String,
    pub auth_info: AuthInfo,
}

/// Auth info
///
/// 注意：手动实现 Debug，access_token 和 client_token 脱敏为 "***"，
/// 避免误用 {:?} 打印时泄露 token 到日志文件
#[derive(Clone, Serialize, Deserialize)]
pub struct AuthInfo {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub client_token: String,
    pub login_type: String,
    /// authlib 登录的 yggdrasil API 根地址（仅 AuthlibInjector 登录时有值）
    /// 启动游戏时用于构建 -javaagent:authlib-injector.jar 和
    /// -Dauthlibinjector.yggdrasil.prefetched 参数
    #[serde(default)]
    pub server_url: Option<String>,
}

impl std::fmt::Debug for AuthInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthInfo")
            .field("username", &self.username)
            .field("uuid", &self.uuid)
            .field("access_token", &"***")
            .field("client_token", &"***")
            .field("login_type", &self.login_type)
            .field("server_url", &self.server_url)
            .finish()
    }
}
