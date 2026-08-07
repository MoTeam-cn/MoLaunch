//! authlib-injector 外置登录命令（yggdrasil 协议）
mod account;
pub(crate) mod helpers;
mod login;
mod skin;
mod types;

pub use account::{get_authlib_accounts, remove_authlib_account, switch_authlib_account};
pub use login::{authlib_fetch_server_meta, authlib_login, authlib_select_profile};
pub use skin::{
    authlib_delete_cape, authlib_delete_skin, authlib_get_skin_info, authlib_upload_cape,
    authlib_upload_skin,
};
pub use types::{AuthlibAccountInfo, AuthlibLoginResult, AuthlibServerMeta, PendingAuthlibLogin};
