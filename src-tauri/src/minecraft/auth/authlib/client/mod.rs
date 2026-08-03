//! yggdrasil HTTP 客户端：认证/刷新/校验、服务器元数据、authlib-injector.jar 下载、角色与皮肤披风管理。
//! server_url 为 yggdrasil API 根（如 `https://littleskin.cn/api/yggdrasil`），
//! 请求统一走 `crate::http`。子模块：helpers（URL/错误/删除材质）、types（错误/元数据类型）、
//! meta（服务器元数据/authlib-injector）、auth（authenticate/validate/refresh）、
//! profile（角色属性/纹理解析）、skin（皮肤）、cape（披风）。

mod auth;
mod cape;
mod helpers;
mod meta;
mod profile;
mod skin;
mod types;

pub use auth::{authenticate, refresh, validate};
pub use cape::{delete_cape, upload_cape};
pub use meta::{ensure_authlib_injector_jar, fetch_authlib_injector_meta, fetch_server_metadata};
pub use profile::{fetch_profile, parse_skin_cape_info};
pub use skin::{delete_skin, upload_skin};
pub use types::{AuthlibInjectorMeta, YggdrasilError};

pub(super) use helpers::{delete_texture, join_url, parse_error};
