//! yggdrasil 协议（authlib-injector 外置登录）模块
//!
//! 实现参考：
//! - authlib-injector 规范：https://github.com/yushijinhun/authlib-injector
//!
//! 模块结构：
//! - `types`:  yggdrasil API 请求/响应数据结构（含角色属性与材质）
//! - `client`: HTTP 客户端，封装认证端点 + 皮肤管理端点（fetch_profile / upload / delete）
//! - `login`:  登录编排（validate → refresh → authenticate 三步降级）
//!
//! 服务器地址（server_url）为 yggdrasil API 根地址，如 `https://littleskin.cn/api/yggdrasil`。
//! 账号本身携带 server_url，与微软/离线账号平级，可在账号管理中自由切换。

pub mod client;
pub mod login;
pub mod types;

pub use client::{
    delete_cape, delete_skin, ensure_authlib_injector_jar, fetch_authlib_injector_meta,
    fetch_profile, fetch_server_metadata, parse_skin_cape_info, upload_cape, upload_skin,
    AuthlibInjectorMeta,
};
pub use login::{login_with_cached_token, login_with_password, refresh_with_profile, LoginOutcome};
pub use types::{
    Profile, ProfileInfo, ProfileProperty, ServerMetadata, SkinCapeInfo, TextureMetadata,
    TextureUrl, Textures, TexturesPayload,
};
