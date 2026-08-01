//! authlib 命令的公开类型定义
//!
//! 包含登录结果、账号信息、服务器元数据、多角色待处理登录上下文。

use serde::Serialize;

use crate::minecraft::auth::authlib::{Profile, ServerMetadata};
use crate::state::LocalAuthResult;

/// authlib 登录结果
///
/// - `Success`：单角色或服务器已选定角色，含可直接使用的 `LocalAuthResult`
/// - `NeedSelect`：多角色且无 selected_profile，前端需弹窗让用户选择
///
/// 安全说明：`NeedSelect` 中的 `access_token` / `client_token` 标记 `#[serde(skip)]`，
/// 不会序列化到 IPC 返回前端。前端选定 profile 后调用 `authlib_select_profile`，
/// 后端从 `state.authlib_pending`（内存暂存）取出 token 完成刷新，不依赖前端回传。
#[derive(Debug, Serialize)]
#[serde(tag = "status")]
pub enum AuthlibLoginResult {
    #[serde(rename = "success")]
    Success { user: LocalAuthResult },
    #[serde(rename = "need_select")]
    NeedSelect {
        #[serde(skip)]
        access_token: String,
        #[serde(skip)]
        client_token: String,
        available_profiles: Vec<Profile>,
    },
}

/// 已保存的 authlib 账号信息（前端列表展示用）
#[derive(Debug, Clone, Serialize)]
pub struct AuthlibAccountInfo {
    /// 登录账号（邮箱或用户名）
    pub username: String,
    /// 选中的角色 UUID
    pub uuid: String,
    /// 选中的角色名
    pub player_name: String,
    /// yggdrasil API 根地址
    pub server_url: String,
    /// 服务器显示名
    pub server_name: String,
}

/// 服务器元数据（前端登录页展示用）
#[derive(Debug, Serialize)]
pub struct AuthlibServerMeta {
    /// 服务器名（从 meta.serverName 提取）
    pub server_name: String,
    /// 注册链接（从 meta.links.register 提取）
    pub register_url: Option<String>,
    /// 主页链接（从 meta.links.homepage 提取）
    pub homepage_url: Option<String>,
}

impl From<ServerMetadata> for AuthlibServerMeta {
    fn from(meta: ServerMetadata) -> Self {
        Self {
            server_name: meta.server_name(),
            register_url: meta.register_url(),
            homepage_url: meta.homepage_url(),
        }
    }
}

/// 多角色登录的待处理上下文
///
/// `authlib_login` 返回 `NeedSelect` 时暂存到 AppState，
/// 前端选定 profile 后 `authlib_select_profile` 取出使用。
#[derive(Debug, Clone)]
pub struct PendingAuthlibLogin {
    pub server_url: String,
    pub server_name: String,
    pub username: String,
    pub password: String,
    pub access_token: String,
    pub client_token: String,
}
