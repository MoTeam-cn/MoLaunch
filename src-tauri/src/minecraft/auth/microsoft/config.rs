//! 微软登录配置
//!
//! Client ID 通过编译时环境变量 `MOLAUNCH_MS_CLIENT_ID` 传入（参考 PCL2 的做法）。
//! 未设置时默认使用 Minecraft 官方启动器 ID，该 ID 已拥有 Minecraft API 权限，
//! 但不支持 v2.0 Device Code Flow，只能使用旧版 login.live.com 端点 + Web Auth Code Flow。
//!
//! 自定义 Client ID 需要通过 https://aka.ms/mce-reviewappid 申请 Minecraft API 权限，
//! 否则 MC Token 交换步骤会返回 403 "Invalid app registration"。

/// Minecraft 官方启动器 Client ID（已拥有 Minecraft API 权限）
const OFFICIAL_CLIENT_ID: &str = "00000000402b5328";

/// OAuth Client ID
///
/// 编译时通过环境变量 `MOLAUNCH_MS_CLIENT_ID` 覆盖：
/// ```sh
/// MOLAUNCH_MS_CLIENT_ID=your-custom-id cargo build
/// ```
pub const OAUTH_CLIENT_ID: &str = match option_env!("MOLAUNCH_MS_CLIENT_ID") {
    Some(id) => id,
    None => OFFICIAL_CLIENT_ID,
};

/// 是否使用官方 Client ID
pub fn is_official_client() -> bool {
    OAUTH_CLIENT_ID == OFFICIAL_CLIENT_ID
}

/// 是否使用 v2.0 端点（决定 RpsTicket 是否需要 `d=` 前缀）
///
/// v2.0 端点获取的 token 需要 `d={token}` 格式；
/// 旧版 login.live.com 端点获取的 token 直接使用，不加前缀。
pub fn use_v2_endpoints() -> bool {
    !is_official_client()
}

/// OAuth 端点配置
pub struct OAuthEndpoints {
    pub authorize_url: &'static str,
    pub token_url: &'static str,
    pub refresh_url: &'static str,
    pub redirect_uri: &'static str,
    pub scope: &'static str,
}

/// 获取当前 Client ID 对应的端点配置
pub fn endpoints() -> OAuthEndpoints {
    if is_official_client() {
        // 官方 ID：旧版 login.live.com 端点 + Web Auth Code Flow
        OAuthEndpoints {
            authorize_url: "https://login.live.com/oauth20_authorize.srf",
            token_url: "https://login.live.com/oauth20_token.srf",
            refresh_url: "https://login.live.com/oauth20_token.srf",
            redirect_uri: "https://login.live.com/oauth20_desktop.srf",
            scope: "service::user.auth.xboxlive.com::MBI_SSL",
        }
    } else {
        // 自定义 ID：v2.0 login.microsoftonline.com 端点 + Device Code Flow
        OAuthEndpoints {
            authorize_url: "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize",
            token_url: "https://login.microsoftonline.com/consumers/oauth2/v2.0/token",
            refresh_url: "https://login.live.com/oauth20_token.srf",
            redirect_uri: "https://login.microsoftonline.com/common/oauth2/nativeclient",
            scope: "XboxLive.signin offline_access",
        }
    }
}
