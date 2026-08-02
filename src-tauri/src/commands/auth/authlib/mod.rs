//! authlib-injector 外置登录命令（yggdrasil 协议）
//!
//! 子模块：types（公开类型）/ login（登录与选角）/ account（账号管理 CRUD/切换）
//! / skin（皮肤披风）/ helpers（共享辅助：账号加载、PNG 校验）。
//! 多角色首次登录返回 NeedSelect，选定后用 refresh 指定 profile 完成；
//! 切换已保存账号走 validate→refresh→密码重登 三步降级。

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
