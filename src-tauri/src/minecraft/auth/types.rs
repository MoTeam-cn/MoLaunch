//! 认证基础类型
//!
//! `LoginType` 登录类型枚举与 `LoginResult` 登录结果结构。

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
